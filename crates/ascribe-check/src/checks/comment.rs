//! `html-comment`: an HTML comment in a page or a fragment, which is in the
//! source of every published page, unlike a memo (SPEC §4.10). Vale's own
//! comments aren't reported.
//!
//! A block that's one comment and nothing else gets a fix that rewrites it
//! as a memo, where a memo has something to be about (SPEC §3.8): the block
//! after it, or the section it ends. A comment that ends a container with
//! no heading, or a page before its first heading, gets none, and neither
//! does one inside a paragraph: where its memo belongs is the writer's call.

use ascribe_core::{Applicability, Fix, Issue, Span, TextEdit, diagnostics};
use ascribe_syntax::{Block, BlockKind, Bound, HtmlBlock, PrimaryValue};

use super::Ctx;

/// The quick fix's title.
const TITLE: &str = "Rewrite as a memo";

impl Ctx<'_> {
    /// `html-comment` for each comment in the HTML block `blocks[index]`.
    pub(super) fn check_html_block(&mut self, blocks: &[Block], index: usize, html: &HtmlBlock) {
        if self.model.checks.is_off(diagnostics::HTML_COMMENT) {
            return;
        }
        let block = &blocks[index];
        let text = self.source().get(block.span.range()).unwrap_or("");
        let found: Vec<Span> = comments(text)
            .into_iter()
            .map(|(start, end)| Span::new(block.span.start() + start, block.span.start() + end))
            .collect();
        let fix = match found.as_slice() {
            [_] => self.memo_fix(blocks, index, html),
            _ => None,
        };
        for span in found {
            let comment = self.source().get(span.range()).unwrap_or("");
            if is_vale_comment(comment) {
                continue;
            }
            let mut issue = Issue::new(diagnostics::HTML_COMMENT, self.location(span));
            if let Some(fix) = &fix {
                issue = issue.with_fix(fix.clone());
            }
            self.report(issue);
        }
    }

    /// `html-comment` for an inline HTML comment, inside a paragraph or
    /// another inline context.
    pub(super) fn check_html_inline(&mut self, span: Span) {
        if self.model.checks.is_off(diagnostics::HTML_COMMENT) {
            return;
        }
        let text = self.source().get(span.range()).unwrap_or("");
        if text.starts_with("<!--") && !is_vale_comment(text) {
            let issue = Issue::new(diagnostics::HTML_COMMENT, self.location(span));
            self.report(issue);
        }
    }

    /// The fix that rewrites the HTML block `blocks[index]`, one comment and
    /// nothing else, as a memo: in its place when a block follows it in its
    /// container or it's at the top of a section, else at the top of the
    /// section it's the last block of.
    fn memo_fix(&self, blocks: &[Block], index: usize, html: &HtmlBlock) -> Option<Fix> {
        let lines = memo_lines(html.literal.trim())?;
        let source = self.source();
        let block = &blocks[index];
        let edits = match blocks.get(index + 1) {
            // A memo is about the block it touches: the blank lines between
            // them go with the comment.
            Some(next) if bindable(next) => {
                let end = line_start(source, next.span.start());
                let text = memo(&lines, "", &continuation(source, block.span.start()));
                vec![TextEdit::replace(Span::new(block.span.start(), end), text)]
            }
            // The last block of a section, or of a container.
            None
            | Some(Block {
                kind: BlockKind::Heading(_),
                ..
            }) => {
                let heading = (0..index)
                    .rev()
                    .find(|&k| matches!(blocks[k].kind, BlockKind::Heading(_)))?;
                // The heading-bound directives at the top of the section.
                let top = (heading + 1..index)
                    .find(|&k| blocks[k].binding() != Some(Bound::Heading))
                    .unwrap_or(index);
                // A directive bound to the comment would be left with
                // nothing to bind.
                if blocks[index - 1].binding() == Some(Bound::FollowingBlock) {
                    return None;
                }
                if top == index {
                    // At the top of the section already: it's about the
                    // section where it is.
                    let end = next_line_start(source, block.span.end());
                    let text = memo(&lines, "", &continuation(source, block.span.start()));
                    vec![TextEdit::replace(Span::new(block.span.start(), end), text)]
                } else {
                    let at = next_line_start(source, blocks[top - 1].span.end());
                    let prefix = continuation(source, blocks[heading].span.start());
                    let removed = Span::new(blocks[index - 1].span.end(), block.span.end());
                    vec![
                        TextEdit::insert(at, memo(&lines, &prefix, &prefix)),
                        TextEdit::delete(removed),
                    ]
                }
            }
            Some(_) => return None,
        };
        Some(Fix {
            title: TITLE.to_owned(),
            file: self.id,
            edits,
            // The comment may be another tool's, which has to stay one.
            applicability: Applicability::Unsafe,
        })
    }
}

/// The comments in a raw HTML block's text, as byte ranges of it. An
/// unclosed comment runs to the end, as an HTML block that opens one does.
fn comments(text: &str) -> Vec<(usize, usize)> {
    let mut out = Vec::new();
    let mut from = 0;
    while let Some(i) = text[from..].find("<!--") {
        let start = from + i;
        let body = &text[start + 4..];
        // `<!-->` and `<!--->` are comments too (CommonMark §6.6).
        let end = if body.starts_with('>') {
            start + 5
        } else if body.starts_with("->") {
            start + 6
        } else {
            body.find("-->").map_or(text.len(), |j| start + 4 + j + 3)
        };
        out.push((start, end));
        from = end;
    }
    out
}

/// A memo's lines from a comment that's the whole of its block, as written
/// between `<!--` and `-->`: the first line trimmed, the rest without the
/// indentation they share, without blank lines around them. `None` when the
/// comment isn't one comment alone, says nothing, or has a line that would
/// end a container memo.
fn memo_lines(literal: &str) -> Option<Vec<String>> {
    let inner = literal.strip_prefix("<!--")?.strip_suffix("-->")?;
    if inner.contains("-->") || inner.contains("<!--") {
        return None;
    }
    let mut lines: Vec<&str> = inner.lines().collect();
    let first = lines.first().map(|l| l.trim()).unwrap_or_default();
    let indent = lines
        .iter()
        .skip(1)
        .filter(|l| !l.trim().is_empty())
        .map(|l| l.len() - l.trim_start().len())
        .min()
        .unwrap_or(0);
    let mut out: Vec<String> = Vec::with_capacity(lines.len());
    if !lines.is_empty() {
        lines.remove(0);
        out.push(first.to_owned());
    }
    for line in lines {
        let line = line.get(indent..).unwrap_or_else(|| line.trim_start());
        out.push(line.trim_end().to_owned());
    }
    while out.first().is_some_and(|l| l.is_empty()) {
        out.remove(0);
    }
    while out.last().is_some_and(|l| l.is_empty()) {
        out.pop();
    }
    if out.is_empty() || out.iter().any(|l| l.trim() == "@end") {
        return None;
    }
    Some(out)
}

/// The memo, ending in a line break: a line memo for one line, a container
/// for more. `first` goes before its opening line and `rest` before each
/// line after it: the container prefixes of the line it's written on.
fn memo(lines: &[String], first: &str, rest: &str) -> String {
    match lines {
        [line] => format!("{first}@memo: {line}\n"),
        _ => {
            let mut out = format!("{first}@memo:\n");
            for line in lines {
                if line.is_empty() {
                    out.push_str(rest.trim_end());
                } else {
                    out.push_str(rest);
                    out.push_str(line);
                }
                out.push('\n');
            }
            out.push_str(rest);
            out.push_str("@end\n");
            out
        }
    }
}

/// The prefix a line after the one `offset` is on takes to stay in the same
/// container: the text before `offset` on its line, with each list marker
/// as spaces and each blockquote marker kept.
fn continuation(source: &str, offset: usize) -> String {
    source[line_start(source, offset)..offset]
        .chars()
        .map(|c| {
            if matches!(c, '>' | ' ' | '\t') {
                c
            } else {
                ' '
            }
        })
        .collect()
}

/// The start of the line `offset` is on.
pub(super) fn line_start(source: &str, offset: usize) -> usize {
    source[..offset].rfind(['\n', '\r']).map_or(0, |i| i + 1)
}

/// The start of the line after the one a block ending at `offset` ends on.
pub(super) fn next_line_start(source: &str, offset: usize) -> usize {
    if offset > 0 && source.as_bytes()[offset - 1] == b'\n' {
        return offset;
    }
    source[offset..]
        .find('\n')
        .map_or(source.len(), |i| offset + i + 1)
}

/// Whether a memo directly above `block` would be about it (SPEC §3.8).
fn bindable(block: &Block) -> bool {
    match &block.kind {
        BlockKind::Heading(_) | BlockKind::End(_) | BlockKind::Title(_) => false,
        // A directive line that's a block of its own, or one that stacks
        // over the block it binds.
        BlockKind::Directive(line) => {
            line.binding == Some(Bound::FollowingBlock)
                || line.name == "snippet"
                || matches!(line.primary, Some(PrimaryValue::Text(_)))
        }
        _ => true,
    }
}

/// Whether an HTML comment, `<!-- … -->` as written, is one of Vale's own:
/// its text starts with the word `vale`, as `<!-- vale off -->` and
/// `<!-- vale Ascribe.Repeated = NO -->` do. The prose given to Vale keeps
/// every comment, so these still work
/// (`ascribe_report::prose`).
fn is_vale_comment(comment: &str) -> bool {
    let text = comment.strip_prefix("<!--").unwrap_or(comment).trim_start();
    text.strip_prefix("vale").is_some_and(|rest| {
        rest.is_empty() || rest.starts_with(char::is_whitespace) || rest.starts_with("-->")
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn comments_are_found_closed_or_not() {
        assert_eq!(comments("<!-- a -->"), vec![(0, 10)]);
        assert_eq!(
            comments("<div>\n<!-- a -->\n<!-->"),
            vec![(6, 16), (17, 22)]
        );
        assert_eq!(comments("<!--->x"), vec![(0, 6)]);
        assert_eq!(comments("<!-- open\nstill"), vec![(0, 15)]);
        assert_eq!(comments("<div></div>"), Vec::new());
    }

    #[test]
    fn a_comment_s_lines_become_a_memo_s() {
        let lines = |s: &str| memo_lines(s);
        assert_eq!(
            lines("<!-- One line. -->"),
            Some(vec!["One line.".to_owned()])
        );
        assert_eq!(
            lines("<!--\n  First.\n    Indented.\n\n  Third.\n-->"),
            Some(vec![
                "First.".to_owned(),
                "  Indented.".to_owned(),
                String::new(),
                "Third.".to_owned()
            ])
        );
        assert_eq!(lines("<!-- -->"), None);
        assert_eq!(lines("<!-- a --> <!-- b -->"), None);
        assert_eq!(lines("<!-- a\n@end\n-->"), None);
    }

    #[test]
    fn a_memo_keeps_its_container() {
        let lines = ["a".to_owned(), String::new(), "b".to_owned()];
        assert_eq!(memo(&lines, "", "> "), "@memo:\n> a\n>\n> b\n> @end\n");
        assert_eq!(memo(&lines[..1], "- ", "  "), "- @memo: a\n");
        assert_eq!(continuation("x\n1. > y", 7), "   > ");
    }

    #[test]
    fn vale_s_comments_are_told_apart() {
        for vale in [
            "<!-- vale off -->",
            "<!--vale on-->",
            "<!-- vale A.B = NO -->",
        ] {
            assert!(is_vale_comment(vale), "{vale}");
        }
        for other in [
            "<!-- valet -->",
            "<!-- Vale off -->",
            "<!-- prettier-ignore -->",
        ] {
            assert!(!is_vale_comment(other), "{other}");
        }
    }
}
