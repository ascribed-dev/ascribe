//! `github-alert`: a block quote written as one of GitHub's alerts, whose
//! first line is `[!NOTE]`, `[!TIP]`, `[!IMPORTANT]`, `[!WARNING]`, or
//! `[!CAUTION]`, in any case. Ascribe reads it as a plain block quote that
//! starts with the marker, and the Markdown output escapes the marker, so it
//! isn't an alert on GitHub either.
//!
//! Its fix rewrites the quote as the `@note` of the same type (SPEC §4.5): the
//! line form for one paragraph, the container for several blocks. A quote
//! that doesn't start its line, as one in a list item does, and one with
//! nothing after its marker get none.

use ascribe_core::{Applicability, Fix, Issue, Span, TextEdit, diagnostics};
use ascribe_syntax::{Block, BlockKind, BlockQuote};

use super::Ctx;
use super::comment::{line_start, next_line_start};

/// GitHub's alerts, each the `@note` type it is.
const KINDS: &[&str] = &["note", "tip", "important", "warning", "caution"];

impl Ctx<'_> {
    /// `github-alert` for the block quote `block`, when its first line is an
    /// alert's marker.
    pub(super) fn check_alert(&mut self, block: &Block, quote: &BlockQuote) {
        if self.model.checks.is_off(diagnostics::GITHUB_ALERT) {
            return;
        }
        let Some(first) = quote.children.first() else {
            return;
        };
        if !matches!(first.kind, BlockKind::Paragraph(_)) {
            return;
        }
        let source = self.source();
        let line = source[first.span.range()]
            .split(['\n', '\r'])
            .next()
            .unwrap_or_default();
        let written = line.trim_end_matches([' ', '\t']);
        let Some(kind) = alert_kind(written) else {
            return;
        };
        let marker = Span::new(first.span.start(), first.span.start() + written.len());
        let note = format!("@note {{type={kind}}}:");
        let fix = self.alert_fix(block, quote, marker, &note);
        let mut issue = Issue::new(diagnostics::GITHUB_ALERT, self.location(marker))
            .with_arg("marker", written.to_owned())
            .with_arg("note", note);
        if let Some(fix) = fix {
            issue = issue.with_fix(fix);
        }
        self.report(issue);
    }

    /// The fix that rewrites the quote as a note: `note` followed by its one
    /// paragraph, or `note` and `@end` around its blocks.
    fn alert_fix(&self, block: &Block, quote: &BlockQuote, marker: Span, note: &str) -> Option<Fix> {
        let source = self.source();
        let span = block.span;
        if line_start(source, span.start()) != span.start() {
            return None;
        }
        let rest = next_line_start(source, marker.end()).min(span.end());
        let content = unquote(&source[rest..span.end()])?;
        let content = content.trim_matches(['\n', '\r']);
        if content.trim().is_empty() {
            return None;
        }
        // The blocks after the marker: what's left of its paragraph, and the
        // quote's other blocks.
        let first = &quote.children[0];
        let more_of_first = first.span.end() > rest;
        let blocks = quote.children.len() - 1 + usize::from(more_of_first);
        let single = blocks == 1
            && (more_of_first || matches!(quote.children[1].kind, BlockKind::Paragraph(_)));
        let text = if single {
            format!("{note} {content}")
        } else {
            let newline = if source[span.range()].contains("\r\n") {
                "\r\n"
            } else {
                "\n"
            };
            format!("{note}{newline}{content}{newline}@end")
        };
        Some(Fix {
            title: format!("Replace with `{note}`"),
            file: self.id,
            edits: vec![TextEdit::replace(span, text)],
            // The note says what the quote meant to, in the type it named.
            applicability: Applicability::Safe,
        })
    }
}

/// The `@note` type of an alert's marker, as `[!NOTE]`, in any case.
fn alert_kind(written: &str) -> Option<&'static str> {
    let name = written.strip_prefix("[!")?.strip_suffix(']')?;
    KINDS
        .iter()
        .find(|k| k.eq_ignore_ascii_case(name))
        .copied()
}

/// The lines of a block quote with its markers taken off: up to three spaces,
/// `>`, and one space after it. A line without a marker continues a
/// paragraph, and is kept as it is. `None` when a tab follows a marker,
/// whose columns a rewrite could change.
fn unquote(text: &str) -> Option<String> {
    let mut out = String::with_capacity(text.len());
    for line in text.split_inclusive('\n') {
        let indent = line.len() - line.trim_start_matches(' ').len();
        let unquoted = match line[indent..].strip_prefix('>') {
            Some(after) if indent <= 3 => {
                if after.starts_with('\t') {
                    return None;
                }
                after.strip_prefix(' ').unwrap_or(after)
            }
            _ => line,
        };
        out.push_str(unquoted);
    }
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn markers_are_read_in_any_case() {
        assert_eq!(alert_kind("[!NOTE]"), Some("note"));
        assert_eq!(alert_kind("[!Caution]"), Some("caution"));
        assert_eq!(alert_kind("[!DANGER]"), None);
        assert_eq!(alert_kind("[!NOTE] Text"), None);
        assert_eq!(alert_kind("\\[!NOTE]"), None);
    }

    #[test]
    fn quote_markers_are_taken_off() {
        assert_eq!(
            unquote(">\n> One.\n>  - two\nlazy").as_deref(),
            Some("\nOne.\n - two\nlazy")
        );
        assert_eq!(unquote(">\tcode"), None);
    }
}
