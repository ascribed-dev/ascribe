//! `memo-empty`: a container memo with nothing between its opening line and
//! its `@end` (SPEC §4.10). A line memo with no text is already an error
//! (`directive-primary`); an empty container says nothing either, and is
//! almost always a slip.
//!
//! Its fix removes the memo's lines, which changes no output: a memo is in
//! none. A memo that shares its opening line with a list marker gets none,
//! since removing it would leave the item's first line behind.

use ascribe_core::{Applicability, Fix, Issue, Span, TextEdit, diagnostics};
use ascribe_syntax::{Block, Container, raw_container_text};

use super::Ctx;
use super::comment::{line_start, next_line_start};

impl Ctx<'_> {
    /// `memo-empty` for the container `block`, when it's a closed memo
    /// with no text.
    pub(super) fn check_memo(&mut self, block: &Block, c: &Container) {
        if c.opener.name != "memo" || self.model.checks.is_off(diagnostics::MEMO_EMPTY) {
            return;
        }
        // An unclosed memo is `container-unclosed`, and runs to the end.
        let Some(end) = &c.end else {
            return;
        };
        let Some(text) = raw_container_text(self.source(), c) else {
            return;
        };
        if !text.trim().is_empty() {
            return;
        }
        let mut issue = Issue::new(diagnostics::MEMO_EMPTY, self.location(c.opener.span));
        if let Some(fix) = self.remove_memo(block.span.start(), end.span.end()) {
            issue = issue.with_fix(fix);
        }
        self.report(issue);
    }

    /// The fix that removes the lines from the one `start` is on through
    /// the one `end` is on, and a blank line after them when one is before
    /// them too, so the blocks around keep one blank line between them.
    fn remove_memo(&self, start: usize, end: usize) -> Option<Fix> {
        let source = self.source();
        let from = line_start(source, start);
        // A list marker before the memo starts its item's first line.
        if !source[from..start]
            .chars()
            .all(|c| matches!(c, '>' | ' ' | '\t'))
        {
            return None;
        }
        let mut to = next_line_start(source, end);
        let blank_before = from == 0 || is_blank(line_before(source, from));
        let after = &source[to..source[to..].find('\n').map_or(source.len(), |i| to + i + 1)];
        if blank_before && to < source.len() && is_blank(after) {
            to += after.len();
        }
        Some(Fix {
            title: "Remove the empty memo".to_owned(),
            file: self.id,
            edits: vec![TextEdit::delete(Span::new(from, to))],
            // A memo is in no output, and an empty one says nothing.
            applicability: Applicability::Safe,
        })
    }
}

/// The line before the one starting at `at`, which isn't the first.
fn line_before(source: &str, at: usize) -> &str {
    let before = source[..at]
        .strip_suffix('\n')
        .unwrap_or(&source[..at])
        .trim_end_matches('\r');
    &before[line_start(before, before.len())..]
}

/// Whether a line is blank, or only the markers of the blockquotes it's in.
fn is_blank(line: &str) -> bool {
    line.chars()
        .all(|c| matches!(c, '>' | ' ' | '\t' | '\r' | '\n'))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blank_lines_are_told_apart() {
        assert!(is_blank(""));
        assert!(is_blank("> \n"));
        assert!(!is_blank("Text.\n"));
        assert_eq!(line_before("a\nb\n\nc", 5), "");
        assert_eq!(line_before("a\nb\r\nc", 5), "b");
    }
}
