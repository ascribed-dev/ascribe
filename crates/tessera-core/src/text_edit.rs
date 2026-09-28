//! Edits to a file's text: diagnostic fixes, formatting, and refactoring.

use serde::Serialize;

use crate::Span;

/// Replaces the bytes in `span` with `new_text`.
///
/// An empty span inserts; an empty `new_text` deletes. The span is in the
/// file's text *before* any of the edits it's applied with (edits in a set
/// are simultaneous, as in LSP). Diagnostic fixes (phase 10), the formatter
/// (phase 23), and refactorings (phase 24) all produce these, and the
/// language server sends them to the editor unchanged.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize)]
pub struct TextEdit {
    /// The bytes to replace, in the original text.
    pub span: Span,
    /// The replacement.
    pub new_text: String,
}

impl TextEdit {
    /// Replaces `span` with `new_text`.
    pub fn replace(span: impl Into<Span>, new_text: impl Into<String>) -> TextEdit {
        TextEdit {
            span: span.into(),
            new_text: new_text.into(),
        }
    }

    /// Inserts `text` at `offset`.
    pub fn insert(offset: usize, text: impl Into<String>) -> TextEdit {
        TextEdit::replace(Span::empty(offset), text)
    }

    /// Deletes `span`.
    pub fn delete(span: impl Into<Span>) -> TextEdit {
        TextEdit::replace(span, String::new())
    }
}

/// Why a set of edits couldn't be applied.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum EditError {
    /// An edit's span goes past the end of the text, or doesn't start and
    /// end on `char` boundaries.
    #[error("edit {span} doesn't fit a text of {len} bytes on character boundaries")]
    OutOfBounds {
        /// The edit's span.
        span: Span,
        /// The text's length.
        len: usize,
    },
    /// Two edits change overlapping text.
    #[error("edits {first} and {second} overlap")]
    Overlap {
        /// The earlier edit's span.
        first: Span,
        /// The later edit's span.
        second: Span,
    },
}

/// Applies a set of simultaneous edits to `text`.
///
/// Edits may come in any order. They must not overlap: after sorting by
/// span, each edit must end at or before the next one starts. Several
/// insertions at the same offset are allowed, and apply in the order given;
/// an insertion at the start of a replaced span goes before the replacement.
pub fn apply_edits(text: &str, edits: &[TextEdit]) -> Result<String, EditError> {
    let mut sorted: Vec<&TextEdit> = edits.iter().collect();
    // Stable: equal spans keep their given order.
    sorted.sort_by_key(|e| (e.span.start(), e.span.end()));
    for e in &sorted {
        if e.span.end() > text.len()
            || !text.is_char_boundary(e.span.start())
            || !text.is_char_boundary(e.span.end())
        {
            return Err(EditError::OutOfBounds {
                span: e.span,
                len: text.len(),
            });
        }
    }
    for pair in sorted.windows(2) {
        if pair[0].span.end() > pair[1].span.start() {
            return Err(EditError::Overlap {
                first: pair[0].span,
                second: pair[1].span,
            });
        }
    }
    let mut out = String::with_capacity(text.len());
    let mut pos = 0;
    for e in sorted {
        out.push_str(&text[pos..e.span.start()]);
        out.push_str(&e.new_text);
        pos = e.span.end();
    }
    out.push_str(&text[pos..]);
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn applies_edits_given_in_any_order() {
        let text = "@note{type=tip}:hi";
        let edits = [
            TextEdit::insert(16, " "),
            TextEdit::insert(5, " "),
            TextEdit::replace(11..14, "caution"),
        ];
        assert_eq!(
            apply_edits(text, &edits).unwrap(),
            "@note {type=caution}: hi"
        );
    }

    #[test]
    fn insertions_at_one_offset_keep_their_order() {
        let edits = [
            TextEdit::insert(1, "b"),
            TextEdit::replace(1..2, "X"),
            TextEdit::insert(1, "c"),
        ];
        assert_eq!(apply_edits("a_d", &edits).unwrap(), "abcXd");
    }

    #[test]
    fn rejects_overlaps_and_bad_spans() {
        let overlap = [TextEdit::delete(0..3), TextEdit::delete(2..4)];
        assert!(matches!(
            apply_edits("abcdef", &overlap),
            Err(EditError::Overlap { .. })
        ));
        assert!(matches!(
            apply_edits("abc", &[TextEdit::delete(2..9)]),
            Err(EditError::OutOfBounds { .. })
        ));
        // Inside the two bytes of é.
        assert!(matches!(
            apply_edits("é", &[TextEdit::insert(1, "x")]),
            Err(EditError::OutOfBounds { .. })
        ));
    }
}
