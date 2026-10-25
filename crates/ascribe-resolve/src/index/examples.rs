//! Code blocks marked as an example of an operation's request or response:
//! `request="<link>"` or `response="<link> <status>"` in a fenced block's
//! info string, beside `title` and `tested`. The value starts with a link,
//! which is indexed as one, so it's resolved and checked as a link is
//! (decision #920).

use ascribe_core::{RelPath, Span};
use ascribe_model::ContentModel;
use ascribe_syntax::{CodeBlock, LinkForm};

use super::refs::{RefKind, Reference, target_of};

/// Which body a marked block is an example of.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExampleOf {
    /// The operation's request body: `request="…"`.
    Request,
    /// One of its responses: `response="…"`.
    Response,
}

impl ExampleOf {
    /// The info string's attribute.
    pub fn attribute(self) -> &'static str {
        match self {
            ExampleOf::Request => "request",
            ExampleOf::Response => "response",
        }
    }
}

/// A fenced code block marked as an example of an operation's body.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExampleMarker {
    /// The code block.
    pub block: Span,
    /// The attribute in its info string, from its name to its closing quote.
    pub span: Span,
    /// Request or response.
    pub of: ExampleOf,
    /// The block's language: the info string's first word.
    pub language: String,
    /// The index, in the file's references, of the value's link.
    pub reference: usize,
    /// The response's status, after the link, as written.
    pub status: Option<String>,
    /// The media type, after the status for a response, or after the link
    /// for a request.
    pub media_type: Option<String>,
    /// The block's code.
    pub code: String,
    /// Where each line of the code starts in the file: the byte offset in
    /// the code, and the offset in the file it's at.
    pub lines: Vec<(usize, usize)>,
}

impl ExampleMarker {
    /// The file's span of the code's bytes `range`, on the line it starts on.
    pub fn span_of(&self, range: std::ops::Range<usize>) -> Span {
        let line = self
            .lines
            .partition_point(|(at, _)| *at <= range.start)
            .saturating_sub(1);
        let Some((code_at, file_at)) = self.lines.get(line).copied() else {
            return self.span;
        };
        let line_end = self
            .lines
            .get(line + 1)
            .map_or(self.code.len(), |(next, _)| next.saturating_sub(1));
        let start = range.start.max(code_at);
        let end = range.end.min(line_end).max(start);
        Span::new(file_at + (start - code_at), file_at + (end - code_at))
    }
}

/// The example markers on `code`, a block at `block` in `source`, pushing
/// each one's link onto `references`.
pub(crate) fn collect(
    code: &CodeBlock,
    block: Span,
    source: &str,
    written_in: &RelPath,
    model: &ContentModel,
    references: &mut Vec<Reference>,
    out: &mut Vec<ExampleMarker>,
) {
    let Some(info_span) = code.info_span.filter(|_| code.fenced) else {
        return;
    };
    let Some(info) = source.get(info_span.range()) else {
        return;
    };
    let language = info
        .split_whitespace()
        .next()
        .unwrap_or_default()
        .to_owned();
    for of in [ExampleOf::Request, ExampleOf::Response] {
        let Some((value, at)) = attribute(info, of.attribute()) else {
            continue;
        };
        let value_start = info_span.start() + at;
        let mut words = value.split(' ').filter(|w| !w.is_empty());
        let link = words.next().unwrap_or_default();
        let link_at = value_start + value.find(link).unwrap_or_default();
        let status = match of {
            ExampleOf::Request => None,
            ExampleOf::Response => words.next().map(str::to_owned),
        };
        let media_type = words.next().map(str::to_owned);
        let span = Span::new(
            value_start - of.attribute().len() - 2,
            value_start + value.len() + 1,
        );
        references.push(Reference {
            kind: RefKind::Link,
            span,
            destination_span: Some(Span::new(link_at, link_at + link.len())),
            destination: link.to_owned(),
            expanded_destination: link.to_owned(),
            form: LinkForm::Inline,
            text_empty: false,
            target: target_of(RefKind::Link, link, &[], written_in, model),
            example: true,
        });
        out.push(ExampleMarker {
            block,
            span,
            of,
            language: language.clone(),
            reference: references.len() - 1,
            status,
            media_type,
            code: code.literal.clone(),
            lines: lines(&code.literal, block, source),
        });
    }
}

/// The value of `key="…"` in an info string, and its offset there: from a
/// space, to the quote that's followed by a space or the end, as a title is
/// read.
fn attribute<'a>(info: &'a str, key: &str) -> Option<(&'a str, usize)> {
    let open = format!("{key}=\"");
    let mut from = 0;
    while let Some(found) = info[from..].find(&open) {
        let at = from + found;
        from = at + open.len();
        if at > 0 && !info[..at].ends_with(char::is_whitespace) {
            continue;
        }
        let value = &info[from..];
        let end = value.char_indices().find(|&(i, c)| {
            c == '"'
                && value[i + 1..]
                    .chars()
                    .next()
                    .is_none_or(char::is_whitespace)
        })?;
        return Some((&value[..end.0], from));
    }
    None
}

/// Where each line of a fenced block's code is in the file: the block's
/// lines after its opening fence, each ending with the code's line, after
/// whatever indentation and container markers come first.
fn lines(code: &str, block: Span, source: &str) -> Vec<(usize, usize)> {
    let Some(text) = source.get(block.start()..) else {
        return Vec::new();
    };
    let mut file_lines = text.split_inclusive('\n').scan(block.start(), |at, line| {
        let start = *at;
        *at += line.len();
        Some((start, line.trim_end_matches(['\n', '\r'])))
    });
    // The opening fence.
    file_lines.next();
    let mut out = Vec::new();
    let mut code_at = 0;
    for code_line in code.split_inclusive('\n') {
        let Some((file_at, file_line)) = file_lines.next() else {
            break;
        };
        let content = code_line.trim_end_matches(['\n', '\r']);
        let offset = if file_line.ends_with(content) {
            file_line.len() - content.len()
        } else {
            0
        };
        out.push((code_at, file_at + offset));
        code_at += code_line.len();
    }
    out
}
