//! Open documents: the editor's buffers, edited incrementally.

use lsp_types::{TextDocumentContentChangeEvent, Uri};
use tessera_core::LineIndex;

use crate::position::Encoding;

/// One open document.
#[derive(Debug)]
pub(crate) struct Doc {
    /// The URI as the client wrote it: diagnostics are published under it.
    pub uri: Uri,
    /// The version the client last sent.
    pub version: i32,
    /// The text at that version.
    pub text: String,
}

impl Doc {
    /// Applies `changes` in order, each relative to the text the one before it
    /// left (LSP's incremental sync), and records the new version.
    pub(crate) fn apply(
        &mut self,
        version: i32,
        changes: Vec<TextDocumentContentChangeEvent>,
        encoding: Encoding,
    ) {
        for change in changes {
            match change.range {
                None => self.text = change.text,
                Some(range) => {
                    let index = LineIndex::new(&self.text);
                    let start = encoding.offset_lenient(&index, &self.text, range.start);
                    let end = encoding
                        .offset_lenient(&index, &self.text, range.end)
                        .max(start);
                    self.text.replace_range(start..end, &change.text);
                }
            }
        }
        self.version = version;
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use lsp_types::{Position, Range};

    use super::*;

    fn doc(text: &str) -> Doc {
        Doc {
            uri: Uri::from_str("file:///a.md").expect("a uri"),
            version: 1,
            text: text.to_owned(),
        }
    }

    fn edit(a: (u32, u32), b: (u32, u32), text: &str) -> TextDocumentContentChangeEvent {
        TextDocumentContentChangeEvent {
            range: Some(Range::new(Position::new(a.0, a.1), Position::new(b.0, b.1))),
            range_length: None,
            text: text.to_owned(),
        }
    }

    #[test]
    fn edits_apply_in_order_and_count_columns_in_the_encoding() {
        // In UTF-16 the emoji is two units, so `b` is at column 3.
        let mut d = doc("a📄b\nsecond\n");
        d.apply(
            2,
            vec![edit((0, 3), (0, 4), "B"), edit((1, 0), (1, 0), "the ")],
            Encoding::Utf16,
        );
        assert_eq!(d.text, "a📄B\nthe second\n");
        assert_eq!(d.version, 2);
        // In UTF-8 it's four bytes: `b` is at column 5.
        let mut d = doc("a📄b\n");
        d.apply(2, vec![edit((0, 5), (0, 6), "B")], Encoding::Utf8);
        assert_eq!(d.text, "a📄B\n");
    }

    #[test]
    fn a_change_without_a_range_replaces_the_text() {
        let mut d = doc("old");
        d.apply(
            3,
            vec![TextDocumentContentChangeEvent {
                range: None,
                range_length: None,
                text: "new".into(),
            }],
            Encoding::Utf16,
        );
        assert_eq!(d.text, "new");
    }
}
