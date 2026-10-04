//! Source anchors (site-render contract §7): with anchors on, each block of the
//! site output says which source file and lines it came from.
//!
//! A block the consumer's markdown pipeline renders (a heading, paragraph,
//! code block, list, table, block quote, thematic break, or raw HTML block)
//! gets an anchor comment, an HTML block of its own on the line before it:
//!
//! ```text
//! <!--ascribe-anchor tag="p" source="guides/install.md:12-14"-->
//! ```
//!
//! An element the emitter writes itself (`<ascribe-note>`, `<details>`, and the
//! rest) carries `data-ascribe-source` and `data-ascribe-via` on its own tag.

use tessera_core::{FileId, RelPath, Span};
use tessera_resolve::{IncludeSite, ResolvedBlock};

use super::element::escape;
use crate::emitter::EmitContext;

/// The start of every anchor comment.
pub(crate) const ANCHOR_START: &str = "<!--ascribe-anchor ";

/// Where a block came from, in the contract's grammar.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Anchor {
    /// `<path>:<first>-<last>`.
    pub(crate) source: String,
    /// The includes, `<path>:<line>` each, outermost first, separated by
    /// spaces; empty for a block written in the page itself.
    pub(crate) via: String,
}

impl Anchor {
    /// The anchor of a block.
    pub(crate) fn of(cx: &EmitContext<'_>, block: &ResolvedBlock) -> Option<Anchor> {
        Anchor::of_span(cx, block.file, block.span, &block.via)
    }

    /// The anchor of a span in a file reached through `via`.
    pub(crate) fn of_span(
        cx: &EmitContext<'_>,
        file: FileId,
        span: Span,
        via: &[IncludeSite],
    ) -> Option<Anchor> {
        let path = encode_path(cx.file_path(file)?);
        let (first, last) = lines(cx, file, span)?;
        let mut sites = Vec::with_capacity(via.len());
        for site in via {
            let at = cx.position(site.file, site.span.start())?;
            sites.push(format!(
                "{}:{}",
                encode_path(cx.file_path(site.file)?),
                at.line
            ));
        }
        Some(Anchor {
            source: format!("{path}:{first}-{last}"),
            via: sites.join(" "),
        })
    }

    /// The anchor of a directive that wraps the block after it: from the
    /// directive's first line through the block's last, when both are in one
    /// file reached the same way, else the directive's own.
    pub(crate) fn spanning(
        cx: &EmitContext<'_>,
        directive: &ResolvedBlock,
        bound: Option<&ResolvedBlock>,
    ) -> Option<Anchor> {
        match bound {
            Some(b) if b.file == directive.file && b.via == directive.via => Anchor::of_span(
                cx,
                directive.file,
                directive.span.cover(b.span),
                &directive.via,
            ),
            _ => Anchor::of(cx, directive),
        }
    }

    /// The anchor comment for a block the markdown pipeline renders as a
    /// `tag` element. `items` holds a list's items' lines.
    pub(crate) fn comment(&self, tag: &str, items: &[(u32, u32)]) -> String {
        let mut out = format!(
            "{ANCHOR_START}tag=\"{tag}\" source=\"{}\"",
            escape(&self.source)
        );
        if !self.via.is_empty() {
            out.push_str(&format!(" via=\"{}\"", escape(&self.via)));
        }
        if !items.is_empty() {
            let ranges: Vec<String> = items.iter().map(|(a, b)| format!("{a}-{b}")).collect();
            out.push_str(&format!(" items=\"{}\"", ranges.join(" ")));
        }
        out.push_str("-->");
        out
    }

    /// An element's text with the anchor's attributes added to its first tag,
    /// after the attributes it has. The emitter escapes `>` in every value it
    /// writes, so the tag ends at the first `>`.
    pub(crate) fn on_element(&self, element: String) -> String {
        let Some(end) = element.find('>') else {
            return element;
        };
        let mut attrs = format!(" data-ascribe-source=\"{}\"", escape(&self.source));
        if !self.via.is_empty() {
            attrs.push_str(&format!(" data-ascribe-via=\"{}\"", escape(&self.via)));
        }
        let mut out = element;
        out.insert_str(end, &attrs);
        out
    }
}

/// The first and last line of a span, counted from 1. A span that ends just
/// after a line ending ends on the line before.
pub(crate) fn lines(cx: &EmitContext<'_>, file: FileId, span: Span) -> Option<(u32, u32)> {
    let (start, end) = cx.range(file, span)?;
    let last = if end.line > start.line && end.column == 1 {
        end.line - 1
    } else {
        end.line
    };
    Some((start.line, last))
}

/// A content path as the contract writes it: `/` between segments, each
/// segment percent-encoded except for ASCII letters, digits, `-`, `.`, `_`,
/// and `~`.
fn encode_path(path: &RelPath) -> String {
    let segments: Vec<String> = path.segments().map(super::encode_segment).collect();
    segments.join("/")
}

/// The element an HTML block's text starts with, lowercased: the tag its
/// anchor names. `None` when it doesn't start with an open tag (a comment, a
/// closing tag, or text), which gets no anchor.
pub(crate) fn html_tag(literal: &str) -> Option<String> {
    let rest = literal.strip_prefix('<')?;
    if !rest.starts_with(|c: char| c.is_ascii_alphabetic()) {
        return None;
    }
    let name: String = rest
        .chars()
        .take_while(|c| c.is_ascii_alphanumeric() || *c == '-')
        .collect();
    let after = rest[name.len()..].chars().next();
    match after {
        None | Some(' ' | '\t' | '\n' | '\r' | '>' | '/') => Some(name.to_ascii_lowercase()),
        Some(_) => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_html_blocks_tag_is_its_first_open_tag() {
        assert_eq!(html_tag("<div class=\"x\">"), Some("div".to_owned()));
        assert_eq!(html_tag("<Custom-El>\n"), Some("custom-el".to_owned()));
        assert_eq!(html_tag("<br/>"), Some("br".to_owned()));
        assert_eq!(html_tag("<!-- note -->"), None);
        assert_eq!(html_tag("</div>"), None);
        assert_eq!(html_tag("text"), None);
    }

    #[test]
    fn an_anchor_goes_after_an_elements_attributes() {
        let anchor = Anchor {
            source: "a.md:3-9".to_owned(),
            via: "b.md:2".to_owned(),
        };
        assert_eq!(
            anchor.on_element("<ascribe-note type=\"tip\">\n\nx\n\n</ascribe-note>".to_owned()),
            "<ascribe-note type=\"tip\" data-ascribe-source=\"a.md:3-9\" data-ascribe-via=\"b.md:2\">\n\nx\n\n</ascribe-note>"
        );
        assert_eq!(
            anchor.comment("ul", &[(3, 4), (5, 9)]),
            "<!--ascribe-anchor tag=\"ul\" source=\"a.md:3-9\" via=\"b.md:2\" items=\"3-4 5-9\"-->"
        );
    }
}
