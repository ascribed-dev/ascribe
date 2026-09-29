//! Inline content as CommonMark text: what the resolved inlines say, written
//! so that an unmodified parser reads back the same text.

use tessera_core::AssetUse;
use tessera_resolve::{LinkTarget, RefKind, ResolvedLink};
use tessera_syntax::{Image, Inline, InlineKind, Link, LinkForm};

use super::Renderer;
use crate::assets::{encode_path, markdown_destination, starts_reference};
use crate::labels::plain_text;

/// How inline content is written.
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct Style {
    /// Emphasis and strong markers are left out: the text goes inside bold.
    pub flat: bool,
    /// The content must stay on one line (headings, table cells): a soft break
    /// is a space.
    pub one_line: bool,
}

struct State {
    out: String,
    /// Whether the next character starts a line, where `#`, `>`, `-`, and
    /// list markers mean something.
    line_start: bool,
}

/// Writes `inlines`, using `links` (a block's resolved links, by span) to
/// resolve each link and image.
pub(crate) fn render(
    r: &Renderer<'_>,
    links: &[ResolvedLink],
    inlines: &[Inline],
    style: Style,
) -> String {
    let mut st = State {
        out: String::new(),
        line_start: true,
    };
    write(r, links, inlines, style, &mut st);
    st.out
}

fn write(
    r: &Renderer<'_>,
    links: &[ResolvedLink],
    inlines: &[Inline],
    style: Style,
    st: &mut State,
) {
    for inline in inlines {
        match &inline.kind {
            InlineKind::Text(text) => escape_into(text, st),
            // A tag is dropped, and its text is in the text around it (Q112).
            InlineKind::Html(text) => escape_into(&super::html_text(text), st),
            InlineKind::Phrase(p) => escape_into(&format!("{{{}}}", p.key), st),
            InlineKind::Code(code) => {
                st.out.push_str(&code_span(code));
                st.line_start = false;
            }
            InlineKind::SoftBreak if style.one_line => space(st),
            InlineKind::HardBreak if style.one_line => space(st),
            InlineKind::SoftBreak => {
                st.out.push('\n');
                st.line_start = true;
            }
            InlineKind::HardBreak => {
                st.out.push_str("\\\n");
                st.line_start = true;
            }
            InlineKind::Emphasis(children) => marked(r, links, children, "*", style, st),
            InlineKind::Strong(children) => marked(r, links, children, "**", style, st),
            InlineKind::Link(link) => link_inline(r, links, inline, link, style, st),
            InlineKind::Image(image) => image_inline(r, links, inline, image, st),
        }
    }
}

fn space(st: &mut State) {
    if !st.out.ends_with(' ') && !st.out.is_empty() {
        st.out.push(' ');
    }
}

/// Emphasis or strong, unless the text is going inside bold already.
fn marked(
    r: &Renderer<'_>,
    links: &[ResolvedLink],
    children: &[Inline],
    marker: &str,
    style: Style,
    st: &mut State,
) {
    if style.flat {
        write(r, links, children, style, st);
        return;
    }
    let mut inner = State {
        out: String::new(),
        line_start: false,
    };
    write(r, links, children, style, &mut inner);
    st.line_start = st.line_start && inner.out.is_empty();
    if inner.out.trim().is_empty() {
        st.out.push_str(&inner.out);
    } else {
        st.out.push_str(marker);
        st.out.push_str(&inner.out);
        st.out.push_str(marker);
        st.line_start = false;
    }
}

fn link_inline(
    r: &Renderer<'_>,
    links: &[ResolvedLink],
    node: &Inline,
    link: &Link,
    style: Style,
    st: &mut State,
) {
    let resolved = links
        .iter()
        .find(|l| l.span == node.span && l.kind == RefKind::Link);
    let destination = match resolved.map(|l| &l.target) {
        Some(LinkTarget::External) => Some(markdown_destination(&link.destination)),
        Some(LinkTarget::Page { url, .. }) => {
            Some(markdown_destination(&r.page.emit.absolute_url(url)))
        }
        Some(LinkTarget::Asset { path, fragment }) => Some(asset_destination(
            r,
            path,
            fragment.as_deref(),
            AssetUse::Link,
        )),
        // A link the build couldn't resolve is an error the checks report; if
        // one gets here anyway, its text stays and the link goes.
        Some(LinkTarget::Unresolved) => None,
        // A glossary link (SPEC §5.4): an ordinary link to a route or a URL.
        None => Some(markdown_destination(
            &r.page.emit.absolute_url(&link.destination),
        )),
    };
    let Some(destination) = destination else {
        write(r, links, &link.children, style, st);
        return;
    };
    if link.form == LinkForm::Autolink
        && matches!(resolved.map(|l| &l.target), Some(LinkTarget::External))
        && !destination.starts_with('<')
        && !link.destination.is_empty()
    {
        st.out.push_str(&format!("<{}>", link.destination));
        st.line_start = false;
        return;
    }
    let text = render(
        r,
        links,
        &link.children,
        Style {
            flat: style.flat,
            one_line: style.one_line,
        },
    );
    st.out.push_str(&format!(
        "[{text}]({destination}{})",
        title(link.title.as_deref())
    ));
    st.line_start = false;
}

fn image_inline(
    r: &Renderer<'_>,
    links: &[ResolvedLink],
    node: &Inline,
    image: &Image,
    st: &mut State,
) {
    let resolved = links
        .iter()
        .find(|l| l.span == node.span && l.kind == RefKind::Image);
    let alt = escape(&plain_text(&image.children), false);
    let destination = match resolved.map(|l| &l.target) {
        Some(LinkTarget::Asset { path, fragment }) => Some(asset_destination(
            r,
            path,
            fragment.as_deref(),
            AssetUse::Image,
        )),
        Some(LinkTarget::External) | None => Some(markdown_destination(&image.destination)),
        Some(LinkTarget::Page { .. } | LinkTarget::Unresolved) => None,
    };
    st.line_start = false;
    match destination {
        // The attribute block (`{width=600}`) has no plain-markdown form.
        // Resolved Q116: what plain markdown does with image attributes.
        Some(d) => st
            .out
            .push_str(&format!("![{alt}]({d}{})", title(image.title.as_deref()))),
        None => st.out.push_str(&alt),
    }
}

/// The reference to an asset's copy, as a markdown destination.
fn asset_destination(
    r: &Renderer<'_>,
    path: &tessera_core::RelPath,
    fragment: Option<&str>,
    usage: AssetUse,
) -> String {
    let placement = r.page.asset(path, usage);
    let mut raw = if placement.url.is_some() {
        placement.reference
    } else {
        encode_path(&placement.reference)
    };
    if let Some(fragment) = fragment {
        raw.push('#');
        raw.push_str(fragment);
    }
    markdown_destination(&raw)
}

/// ` "title"`, or nothing.
pub(crate) fn title(title: Option<&str>) -> String {
    let Some(title) = title.filter(|t| !t.is_empty()) else {
        return String::new();
    };
    let mut out = String::from(" \"");
    for ch in title.chars() {
        match ch {
            '"' | '\\' => {
                out.push('\\');
                out.push(ch);
            }
            '\n' | '\r' => out.push(' '),
            _ => out.push(ch),
        }
    }
    out.push('"');
    out
}

/// A code span holding `code`: the fence is a run of backticks the content
/// doesn't contain, padded with a space where the content would otherwise
/// touch it.
pub(crate) fn code_span(code: &str) -> String {
    let mut runs = std::collections::BTreeSet::new();
    let mut run = 0;
    for ch in code.chars().chain(std::iter::once(' ')) {
        if ch == '`' {
            run += 1;
        } else if run > 0 {
            runs.insert(run);
            run = 0;
        }
    }
    let n = (1..).find(|n| !runs.contains(n)).unwrap_or(1);
    let fence = "`".repeat(n);
    let pad = code.starts_with('`')
        || code.ends_with('`')
        || (code.starts_with(' ') && code.ends_with(' ') && !code.trim().is_empty());
    if pad {
        format!("{fence} {code} {fence}")
    } else {
        format!("{fence}{code}{fence}")
    }
}

fn escape_into(text: &str, st: &mut State) {
    if text.is_empty() {
        return;
    }
    st.out.push_str(&escape(text, st.line_start));
    st.line_start = st.line_start && text.chars().all(char::is_whitespace);
}

/// Text written so that CommonMark reads it back as `text`: the characters
/// that could start emphasis, a link, a code span, HTML, or a character
/// reference are backslash-escaped, and so is a character at the start of a
/// line that would start a heading, quote, list, or break there.
pub(crate) fn escape(text: &str, mut line_start: bool) -> String {
    let chars: Vec<char> = text.chars().collect();
    let mut out = String::with_capacity(text.len() + 4);
    let mut i = 0;
    while i < chars.len() {
        let ch = chars[i];
        let prev = i.checked_sub(1).map(|p| chars[p]);
        let next = chars.get(i + 1).copied();
        if line_start && ch.is_ascii_digit() {
            let end = (i..chars.len())
                .find(|&j| !chars[j].is_ascii_digit())
                .unwrap_or(chars.len());
            out.extend(&chars[i..end]);
            if matches!(chars.get(end), Some('.' | ')')) {
                out.push('\\');
                out.push(chars[end]);
                i = end + 1;
            } else {
                i = end;
            }
            line_start = false;
            continue;
        }
        let at_start = line_start && !ch.is_whitespace();
        let escape = match ch {
            '\\' | '`' | '*' | '[' | ']' | '<' => true,
            '_' => {
                !(prev.is_some_and(char::is_alphanumeric)
                    && next.is_some_and(char::is_alphanumeric))
            }
            '&' => starts_reference(&chars[i + 1..]),
            '~' => next == Some('~') || at_start,
            '#' | '>' | '+' | '-' | '=' => at_start,
            _ => false,
        };
        if escape {
            out.push('\\');
        }
        out.push(ch);
        if !ch.is_whitespace() {
            line_start = false;
        }
        i += 1;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn escapes_what_markdown_would_read() {
        assert_eq!(
            escape("a *b* _c_ d_e `f` [g] <h>", false),
            "a \\*b\\* \\_c\\_ d_e \\`f\\` \\[g\\] \\<h>"
        );
        assert_eq!(escape("R&D &amp; more", false), "R&D \\&amp; more");
        assert_eq!(escape("~~x~~ ~/y", false), "\\~~x\\~~ ~/y");
    }

    #[test]
    fn escapes_line_starts() {
        assert_eq!(escape("# not a heading", true), "\\# not a heading");
        assert_eq!(escape("> not a quote", true), "\\> not a quote");
        assert_eq!(escape("- not a list", true), "\\- not a list");
        assert_eq!(escape("1. not a list", true), "1\\. not a list");
        assert_eq!(escape("12) not a list", true), "12\\) not a list");
        assert_eq!(escape("1.5 is fine", true), "1\\.5 is fine");
        assert_eq!(escape("a # b > c - d", true), "a # b > c - d");
        assert_eq!(escape("# fine mid-line", false), "# fine mid-line");
    }

    #[test]
    fn code_spans_avoid_their_own_backticks() {
        assert_eq!(code_span("x"), "`x`");
        assert_eq!(code_span("a`b"), "``a`b``");
        assert_eq!(code_span("`a"), "`` `a ``");
        assert_eq!(code_span("a``b`c"), "```a``b`c```");
    }

    #[test]
    fn titles_are_quoted() {
        assert_eq!(title(Some("a \"b\"")), " \"a \\\"b\\\"\"");
        assert_eq!(title(None), "");
    }
}
