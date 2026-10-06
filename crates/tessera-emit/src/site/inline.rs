//! Inline content as site markdown: what the resolved inlines say, written so
//! that a CommonMark parser with raw HTML enabled reads back the same text.
//!
//! It differs from the plain output's inline writer in what the site output
//! keeps: raw inline HTML passes through unchanged (the Astro profile
//! requires `html = true` is only for the plain output), an image
//! followed by its attribute marker, links as the
//! consumer's routes and URLs, and a glossary link with its term's
//! definition as the title and a marker naming the term (element contract
//! §7).

use tessera_core::{AssetUse, AttributeValue, names};
use tessera_resolve::{LinkTarget, RefKind, ResolvedBlock, ResolvedLink};
use tessera_syntax::{Image, Inline, InlineKind, Link, LinkForm};

use super::blocks::Renderer;
use super::element::marker;
use crate::assets::{encode_path, markdown_destination};
use crate::labels::plain_text;
use crate::plain::inline::{code_span, escape, title};

/// How inline content is written.
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct Mode {
    /// The content must stay on one line (headings, table cells, summaries):
    /// a line break is a space.
    pub one_line: bool,
    /// An image is written as its alt text: the content is going into raw
    /// HTML (a `<summary>`), where a markdown image can't be processed.
    pub images_as_alt: bool,
}

struct State {
    out: String,
    /// Whether the next character starts a line, where `#`, `>`, `-`, and
    /// list markers mean something.
    line_start: bool,
}

/// Writes `inlines`, using `block`'s resolved links and glossary uses.
pub(crate) fn render(
    r: &Renderer<'_>,
    block: &ResolvedBlock,
    inlines: &[Inline],
    mode: Mode,
) -> String {
    let mut st = State {
        out: String::new(),
        line_start: true,
    };
    write(r, block, inlines, mode, &mut st);
    st.out
}

fn write(r: &Renderer<'_>, block: &ResolvedBlock, inlines: &[Inline], mode: Mode, st: &mut State) {
    for inline in inlines {
        match &inline.kind {
            InlineKind::Text(text) => escape_into(text, st),
            // Raw HTML passes through unchanged (SPEC §9.5, "HTML
            // passthrough"): an author's own `ascribe-attributes` element
            // gets its effect too.
            InlineKind::Html(html) => {
                st.out.push_str(html);
                st.line_start = false;
            }
            InlineKind::Phrase(p) => escape_into(&format!("{{{}}}", p.key), st),
            InlineKind::Code(code) => {
                st.out.push_str(&code_span(code));
                st.line_start = false;
            }
            InlineKind::SoftBreak | InlineKind::HardBreak if mode.one_line => space(st),
            InlineKind::SoftBreak => {
                st.out.push('\n');
                st.line_start = true;
            }
            InlineKind::HardBreak => {
                st.out.push_str("\\\n");
                st.line_start = true;
            }
            InlineKind::Emphasis(children) => marked(r, block, children, "*", mode, st),
            InlineKind::Strong(children) => marked(r, block, children, "**", mode, st),
            InlineKind::Link(link) => link_inline(r, block, inline, link, mode, st),
            InlineKind::Image(image) => image_inline(r, block, inline, image, mode, st),
        }
    }
}

fn space(st: &mut State) {
    if !st.out.ends_with(' ') && !st.out.is_empty() {
        st.out.push(' ');
    }
}

fn marked(
    r: &Renderer<'_>,
    block: &ResolvedBlock,
    children: &[Inline],
    marker: &str,
    mode: Mode,
    st: &mut State,
) {
    let mut inner = State {
        out: String::new(),
        line_start: false,
    };
    write(r, block, children, mode, &mut inner);
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

fn resolved<'a>(
    block: &'a ResolvedBlock,
    node: &Inline,
    kind: RefKind,
) -> Option<&'a ResolvedLink> {
    block
        .links
        .iter()
        .find(|l| l.span == node.span && l.kind == kind)
}

fn link_inline(
    r: &Renderer<'_>,
    block: &ResolvedBlock,
    node: &Inline,
    link: &Link,
    mode: Mode,
    st: &mut State,
) {
    let resolved = resolved(block, node, RefKind::Link);
    let mut term = None;
    let destination = match resolved.map(|l| &l.target) {
        Some(LinkTarget::External) => markdown_destination(&link.destination),
        Some(LinkTarget::Page { url, .. }) => markdown_destination(url),
        Some(LinkTarget::Asset { path, fragment }) => {
            asset_destination(r, path, fragment.as_deref(), AssetUse::Link)
        }
        // A link the build couldn't resolve is an error the checks report; if
        // one gets here anyway, its text stays and the link goes.
        Some(LinkTarget::Unresolved) => {
            write(r, block, &link.children, mode, st);
            return;
        }
        // A glossary link (SPEC §5.4): an ordinary link to the term's route,
        // with its definition as the title and a marker naming the term
        // (element contract §7).
        None => {
            term = r.glossary_term(block, link);
            markdown_destination(&link.destination)
        }
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
    let text = render(r, block, &link.children, mode);
    let shown = term
        .as_ref()
        .map(|(_, definition)| definition.as_str())
        .or(link.title.as_deref());
    st.out
        .push_str(&format!("[{text}]({destination}{})", title(shown)));
    // Directly after the link, with nothing between: the marker that gives
    // the `<a>` its attributes.
    if let Some((id, _)) = term {
        st.out
            .push_str(&marker(&[(names::DATA_TERM.to_owned(), id)]));
    }
    st.line_start = false;
}

fn image_inline(
    r: &Renderer<'_>,
    block: &ResolvedBlock,
    node: &Inline,
    image: &Image,
    mode: Mode,
    st: &mut State,
) {
    let resolved = resolved(block, node, RefKind::Image);
    let alt = escape(&plain_text(&image.children), false);
    st.line_start = false;
    if mode.images_as_alt {
        st.out.push_str(&alt);
        return;
    }
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
    let Some(destination) = destination else {
        st.out.push_str(&alt);
        return;
    };
    st.out.push_str(&format!(
        "![{alt}]({destination}{})",
        title(image.title.as_deref())
    ));
    // Directly after the image, with nothing between: the marker that gives
    // the `<img>` its attributes.
    let attributes = image_attributes(r, image);
    if !attributes.is_empty() {
        st.out.push_str(&marker(&attributes));
    }
}

/// The attributes an image carries: those written and the model's defaults,
/// in the order the content model declares them, then any it doesn't declare
/// in written order. A value set's members are joined with spaces; other
/// values are their text.
// A declared default reaches every image, as a widget's
// does.
fn image_attributes(r: &Renderer<'_>, image: &Image) -> Vec<(String, String)> {
    let written = image.attributes.as_ref().map(|a| &a.block);
    let mut out: Vec<(String, String)> = Vec::new();
    for declared in &r.model.image_attributes {
        let value = written
            .and_then(|b| b.get(&declared.key))
            .and_then(|a| a.value.as_ref())
            .map(value_text)
            .or_else(|| declared.default.as_ref().map(super::blocks::default_text));
        if let Some(value) = value {
            out.push((declared.key.clone(), value));
        }
    }
    if let Some(block) = written {
        for attribute in &block.attributes {
            let declared = r
                .model
                .image_attributes
                .iter()
                .any(|d| d.key == attribute.key);
            let repeated = out.iter().any(|(k, _)| *k == attribute.key);
            if let (false, false, Some(value)) = (declared, repeated, &attribute.value) {
                out.push((attribute.key.clone(), value_text(value)));
            }
        }
    }
    out
}

/// A value as the marker or an element attribute holds it.
pub(crate) fn value_text(value: &AttributeValue) -> String {
    value.members().join(" ")
}

/// The reference to an asset's copy, as a markdown destination.
fn asset_destination(
    r: &Renderer<'_>,
    path: &tessera_core::RelPath,
    fragment: Option<&str>,
    usage: AssetUse,
) -> String {
    let placement = r.page.asset(path, usage);
    // A published copy's reference is a URL, already encoded; a mirrored
    // one is a relative path, where `%`, `#`, and `?` need encoding.
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

fn escape_into(text: &str, st: &mut State) {
    if text.is_empty() {
        return;
    }
    st.out.push_str(&escape(text, st.line_start));
    st.line_start = st.line_start && text.chars().all(char::is_whitespace);
}
