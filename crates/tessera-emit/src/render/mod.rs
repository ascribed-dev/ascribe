//! [`render_site_html`]: site markdown to HTML, with the site output's
//! attribute markers applied.
//!
//! It is comrak's CommonMark rendering with raw HTML passed through (SPEC
//! §9.5, "HTML passthrough"), plus the three things the markers add: a
//! `<ascribe-attributes>` marker that ends a heading gives the heading its
//! attributes (its `id`), one directly after an image gives the `<img>` its
//! attributes, and one directly after a link gives the `<a>` its attributes
//! (a glossary link's `data-ascribe-term`). The editor preview renders with it. The Astro markdown
//! plugin (`@ascribed/astro`) does the same in Astro's pipeline, and
//! `tests/render/`'s fixtures keep the two equal.
//!
//! # How markers are found
//!
//! comrak renders a marker as the two raw inline HTML nodes' text, unchanged,
//! so the rules can be applied to the HTML it writes:
//!
//! - a marker directly followed by the closing tag of a heading, and that
//!   heading's opening tag has no attributes, applies to the heading; the
//!   marker and the whitespace and line breaks before it are removed;
//! - a marker directly after an `<img … />` tag, with nothing between, applies
//!   to the image; the marker is removed;
//! - a marker directly after an `</a>` tag, with nothing between, applies to
//!   the link it closes; the marker is removed. A link holds no link, so its
//!   opening tag is the last `<a ` before it.
//!
//! comrak escapes `<`, `>`, `&`, and `"` in every attribute value it writes,
//! so a tag ends at its first `>`, and the text of a heading or code span
//! can't be mistaken for a tag. Any other marker, and any text that only
//! looks like one, stays as raw HTML.
//!
//! # How anchors are applied
//!
//! A source anchor (contract §7) is an HTML block holding only an
//! `<!--ascribe-anchor …-->` comment, which comrak writes as it is, followed by
//! a line ending. It's removed with that line ending, and its attributes go on
//! the next tag when that tag, after only whitespace, opens the element the
//! anchor names: an element comrak writes, or the first tag of a raw HTML
//! block. A list's anchor also gives each of its items an anchor, found by
//! counting nested lists. comrak escapes `<` in text, so the next `<` is a tag.

use comrak_tessera::{Options, markdown_to_html};
use tessera_core::names;

/// The tag name of the attribute marker.
const MARKER: &str = names::ELEMENT_ATTRIBUTES;

/// Renders site markdown as HTML: CommonMark with raw HTML allowed, GFM's
/// tables, strikethrough, bare links, and task lists as Astro's defaults have
/// them, and the attribute markers applied.
pub fn render_site_html(markdown: &str) -> String {
    apply_anchors(&apply_markers(&markdown_to_html(markdown, &options())))
}

/// Renders one line of inline markdown as HTML, with no paragraph around it:
/// what a `<summary>` holds (element contract §5).
pub(crate) fn inline_html(markdown: &str) -> String {
    let html = markdown_to_html(markdown, &options());
    let html = html.trim();
    let html = html.strip_prefix("<p>").unwrap_or(html);
    html.strip_suffix("</p>").unwrap_or(html).to_owned()
}

fn options() -> Options<'static> {
    let mut options = Options::default();
    options.render.r#unsafe = true;
    options.extension.table = true;
    options.extension.strikethrough = true;
    options.extension.autolink = true;
    options.extension.tasklist = true;
    options
}

/// A marker found in rendered HTML.
struct Found {
    /// Where it starts and ends in the HTML.
    start: usize,
    end: usize,
    /// Its attributes, decoded, in order.
    attributes: Vec<(String, String)>,
}

/// A change to the HTML: replace `start..end` with `text`.
struct Edit {
    start: usize,
    end: usize,
    text: String,
}

/// Applies the marker rules to rendered HTML.
fn apply_markers(html: &str) -> String {
    let mut edits: Vec<Edit> = Vec::new();
    for marker in markers(html) {
        // A marker directly after an image applies to the image, even at the
        // end of a heading: the image rule is the more specific one.
        if let Some(edit) = image_edit(html, &marker) {
            edits.push(edit);
        } else if let Some(mut found) = link_edits(html, &marker) {
            edits.append(&mut found);
        } else if let Some(mut found) = heading_edits(html, &marker) {
            edits.append(&mut found);
        }
    }
    edits.sort_by_key(|e| e.start);
    let mut out = String::with_capacity(html.len());
    let mut at = 0;
    for edit in edits {
        if edit.start < at {
            continue;
        }
        out.push_str(&html[at..edit.start]);
        out.push_str(&edit.text);
        at = edit.end;
    }
    out.push_str(&html[at..]);
    out
}

/// Every well-formed marker in the HTML, in order: the open
/// tag with zero or more ` name="value"` attributes, then directly the
/// closing tag.
fn markers(html: &str) -> Vec<Found> {
    let open = format!("<{MARKER}");
    let close = format!("</{MARKER}>");
    let mut found = Vec::new();
    let mut from = 0;
    while let Some(at) = html[from..].find(&open) {
        let start = from + at;
        from = start + open.len();
        if let Some((attributes, tag_end)) = parse_open_tag(&html[start + open.len()..])
            && html[start + open.len() + tag_end..].starts_with(&close)
        {
            let end = start + open.len() + tag_end + close.len();
            found.push(Found {
                start,
                end,
                attributes,
            });
            from = end;
        }
    }
    found
}

/// Reads ` name="value"`s and the `>` after them. Returns the attributes with
/// their values decoded, and how much of `text` the attributes and the `>`
/// take.
fn parse_open_tag(text: &str) -> Option<(Vec<(String, String)>, usize)> {
    let mut attributes = Vec::new();
    let mut rest = text;
    loop {
        if let Some(after) = rest.strip_prefix('>') {
            return Some((attributes, text.len() - after.len()));
        }
        let after = rest.strip_prefix(' ')?;
        let name_len = attribute_name_len(after)?;
        let name = &after[..name_len];
        let after = after[name_len..].strip_prefix("=\"")?;
        let value_end = after.find(['"', '\n', '\r'])?;
        if !after[value_end..].starts_with('"') {
            return None;
        }
        attributes.push((name.to_owned(), decode(&after[..value_end])));
        rest = &after[value_end + 1..];
    }
}

/// The length of a name that starts with a lowercase letter and continues
/// with lowercase letters, digits, and hyphens (SPEC Appendix A `key`).
fn attribute_name_len(text: &str) -> Option<usize> {
    let mut chars = text.char_indices();
    let (_, first) = chars.next()?;
    if !first.is_ascii_lowercase() {
        return None;
    }
    let end = chars
        .find(|(_, c)| !(c.is_ascii_lowercase() || c.is_ascii_digit() || *c == '-'))
        .map_or(text.len(), |(i, _)| i);
    Some(end)
}

/// `&quot;`, `&amp;`, `&lt;`, and `&gt;` decoded; any other `&` is literal.
/// One pass, so `&amp;lt;` is `&lt;`.
fn decode(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    let mut rest = value;
    while let Some(at) = rest.find('&') {
        out.push_str(&rest[..at]);
        rest = &rest[at..];
        let (ch, len) = [
            ("&quot;", '"'),
            ("&amp;", '&'),
            ("&lt;", '<'),
            ("&gt;", '>'),
        ]
        .into_iter()
        .find(|(entity, _)| rest.starts_with(entity))
        .map_or(('&', 1), |(entity, ch)| (ch, entity.len()));
        out.push(ch);
        rest = &rest[len..];
    }
    out.push_str(rest);
    out
}

fn encode(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for ch in value.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            _ => out.push(ch),
        }
    }
    out
}

/// An anchor comment found in rendered HTML.
struct FoundAnchor {
    start: usize,
    /// Where the comment and the line ending after it end.
    end: usize,
    tag: String,
    source: String,
    via: Option<String>,
    items: Vec<String>,
}

/// Applies every source anchor in rendered HTML, and removes them all.
fn apply_anchors(html: &str) -> String {
    let mut edits: Vec<Edit> = Vec::new();
    for anchor in anchors(html) {
        edits.push(Edit {
            start: anchor.start,
            end: anchor.end,
            text: String::new(),
        });
        edits.extend(anchor_edits(html, &anchor));
    }
    edits.sort_by_key(|e| e.start);
    let mut out = String::with_capacity(html.len() + edits.len() * 48);
    let mut at = 0;
    for edit in edits {
        if edit.start < at {
            continue;
        }
        out.push_str(&html[at..edit.start]);
        out.push_str(&edit.text);
        at = edit.end;
    }
    out.push_str(&html[at..]);
    out
}

/// Every well-formed anchor comment: `<!--ascribe-anchor`, ` name="value"`
/// attributes with `tag` and `source` among them, and `-->`.
fn anchors(html: &str) -> Vec<FoundAnchor> {
    // The start of an anchor comment.
    let anchor = format!("<!--{}", names::COMMENT_ANCHOR);
    let mut found = Vec::new();
    let mut from = 0;
    while let Some(at) = html[from..].find(&anchor) {
        let start = from + at;
        from = start + anchor.len();
        let Some((attributes, len)) = parse_anchor(&html[from..]) else {
            continue;
        };
        let mut end = from + len;
        if html[end..].starts_with("\r\n") {
            end += 2;
        } else if html[end..].starts_with('\n') {
            end += 1;
        }
        let get = |name: &str| {
            attributes
                .iter()
                .find(|(n, _)| n == name)
                .map(|(_, v)| v.clone())
        };
        if let (Some(tag), Some(source)) = (get("tag"), get("source")) {
            found.push(FoundAnchor {
                start,
                end,
                tag: tag.to_ascii_lowercase(),
                source,
                via: get("via").filter(|v| !v.is_empty()),
                items: get("items")
                    .map(|v| v.split_whitespace().map(str::to_owned).collect())
                    .unwrap_or_default(),
            });
        }
        from = end;
    }
    found
}

/// Reads ` name="value"`s and the `-->` after them, as [`parse_open_tag`]
/// reads a marker's.
fn parse_anchor(text: &str) -> Option<(Vec<(String, String)>, usize)> {
    let mut attributes = Vec::new();
    let mut rest = text;
    loop {
        if let Some(after) = rest.strip_prefix("-->") {
            return Some((attributes, text.len() - after.len()));
        }
        let after = rest.strip_prefix(' ')?;
        let name_len = attribute_name_len(after)?;
        let name = &after[..name_len];
        let after = after[name_len..].strip_prefix("=\"")?;
        let value_end = after.find(['"', '\n', '\r'])?;
        if !after[value_end..].starts_with('"') {
            return None;
        }
        attributes.push((name.to_owned(), decode(&after[..value_end])));
        rest = &after[value_end + 1..];
    }
}

/// Where an anchor's attributes go: after the name of the tag that follows
/// it, if that tag opens the element it names, and on each item of a list.
fn anchor_edits(html: &str, anchor: &FoundAnchor) -> Vec<Edit> {
    let rest = &html[anchor.end..];
    let skipped = rest.len() - rest.trim_start().len();
    let open = anchor.end + skipped;
    let Some(name_end) = opens(html, open, &anchor.tag) else {
        return Vec::new();
    };
    let mut edits = vec![Edit {
        start: name_end,
        end: name_end,
        text: anchor_attributes(&anchor.source, anchor.via.as_deref()),
    }];
    if (anchor.tag == "ul" || anchor.tag == "ol") && !anchor.items.is_empty() {
        let items = list_items(html, name_end);
        let path = anchor.source.rsplit_once(':').map_or("", |(p, _)| p);
        if items.len() == anchor.items.len() {
            for (at, lines) in items.into_iter().zip(&anchor.items) {
                edits.push(Edit {
                    start: at,
                    end: at,
                    text: anchor_attributes(&format!("{path}:{lines}"), anchor.via.as_deref()),
                });
            }
        }
    }
    edits
}

/// When the HTML at `at` opens a `tag` element, where its name ends.
fn opens(html: &str, at: usize, tag: &str) -> Option<usize> {
    let rest = html.get(at..)?.strip_prefix('<')?;
    let name = rest.get(..tag.len())?;
    if !name.eq_ignore_ascii_case(tag) {
        return None;
    }
    match rest[tag.len()..].chars().next() {
        Some(' ' | '\t' | '\n' | '\r' | '>' | '/') => Some(at + 1 + tag.len()),
        _ => None,
    }
}

/// Where the name of each `<li>` of the list whose open tag's name ends at
/// `from` ends: the items at its own level, not those of lists inside it.
fn list_items(html: &str, from: usize) -> Vec<usize> {
    let mut items = Vec::new();
    let mut depth = 0usize;
    let mut at = from;
    while let Some(found) = html[at..].find('<') {
        let tag = at + found;
        at = tag + 1;
        if html[at..].starts_with("/ul>") || html[at..].starts_with("/ol>") {
            if depth == 0 {
                break;
            }
            depth -= 1;
        } else if opens(html, tag, "ul").is_some() || opens(html, tag, "ol").is_some() {
            depth += 1;
        } else if depth == 0 && opens(html, tag, "li").is_some() {
            items.push(tag + 3);
        }
    }
    items
}

fn anchor_attributes(source: &str, via: Option<&str>) -> String {
    let mut out = format!(" {}=\"{}\"", names::DATA_SOURCE, encode(source));
    if let Some(via) = via {
        out.push_str(&format!(" {}=\"{}\"", names::DATA_VIA, encode(via)));
    }
    out
}

/// A marker that ends a heading: the heading's opening tag
/// gets the attributes, and the marker and the whitespace before it go.
fn heading_edits(html: &str, marker: &Found) -> Option<Vec<Edit>> {
    let after = &html[marker.end..];
    let level = (1..=6u8).find(|n| after.starts_with(&format!("</h{n}>")))?;
    let open = format!("<h{level}>");
    let open_at = html[..marker.start].rfind(&open)?;
    // No other heading tag between them: a heading holds no headings.
    if html[open_at + open.len()..marker.start].contains(&format!("</h{level}>")) {
        return None;
    }
    let before = &html[..marker.start];
    let trimmed = before.trim_end_matches([' ', '\t', '\n', '\r']).len();
    let trimmed = trimmed.max(open_at + open.len());
    let mut tag = format!("<h{level}");
    tag.push_str(&write_attributes(&[], &marker.attributes));
    tag.push('>');
    Some(vec![
        Edit {
            start: open_at,
            end: open_at + open.len(),
            text: tag,
        },
        Edit {
            start: trimmed,
            end: marker.end,
            text: String::new(),
        },
    ])
}

/// A marker directly after an `<img … />` tag: the tag gets
/// the attributes, replacing any it has by the same name (§3), and the
/// marker goes.
fn image_edit(html: &str, marker: &Found) -> Option<Edit> {
    let before = &html[..marker.start];
    let body = before
        .strip_suffix(" />")
        .or_else(|| before.strip_suffix("/>"))?;
    let start = body.rfind("<img")?;
    let inside = &body[start + "<img".len()..];
    // The tag holds no `<`: comrak escapes it in every attribute value.
    if inside.contains(['<', '>']) {
        return None;
    }
    let existing = parse_existing(inside)?;
    let mut tag = String::from("<img");
    tag.push_str(&write_attributes(&existing, &marker.attributes));
    tag.push_str(" />");
    Some(Edit {
        start,
        end: marker.end,
        text: tag,
    })
}

/// A marker directly after a link's `</a>`: the link's opening tag gets the
/// attributes, replacing any it has by the same name (§3), and the marker
/// goes.
fn link_edits(html: &str, marker: &Found) -> Option<Vec<Edit>> {
    let before = html[..marker.start].strip_suffix("</a>")?;
    let start = before.rfind("<a ")?;
    // The tag holds no `>`: comrak escapes it in every attribute value.
    let tag_end = start + html[start..].find('>')?;
    let existing = parse_existing(&html[start + "<a".len()..tag_end])?;
    let mut tag = String::from("<a");
    tag.push_str(&write_attributes(&existing, &marker.attributes));
    tag.push('>');
    Some(vec![
        Edit {
            start,
            end: tag_end + 1,
            text: tag,
        },
        Edit {
            start: marker.start,
            end: marker.end,
            text: String::new(),
        },
    ])
}

/// The attributes comrak wrote on a tag, as `(name, decoded value)`.
fn parse_existing(text: &str) -> Option<Vec<(String, String)>> {
    let mut out = Vec::new();
    let mut rest = text.trim_start();
    while !rest.is_empty() {
        let eq = rest.find("=\"")?;
        let name = rest[..eq].trim();
        let after = &rest[eq + 2..];
        let end = after.find('"')?;
        out.push((name.to_owned(), decode(&after[..end])));
        rest = after[end + 1..].trim_start();
    }
    Some(out)
}

/// ` name="value"` for each attribute: the existing ones, with those the
/// marker sets replacing them, and the marker's other attributes after.
fn write_attributes(existing: &[(String, String)], marker: &[(String, String)]) -> String {
    let mut all: Vec<(String, String)> = existing.to_vec();
    for (name, value) in marker {
        match all.iter_mut().find(|(n, _)| n == name) {
            Some(slot) => slot.1 = value.clone(),
            None => all.push((name.clone(), value.clone())),
        }
    }
    all.iter()
        .map(|(name, value)| format!(" {name}=\"{}\"", encode(value)))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_marker_ending_a_heading_gives_it_an_id() {
        let html =
            render_site_html("## Setup <ascribe-attributes id=\"setup\"></ascribe-attributes>\n");
        assert_eq!(html, "<h2 id=\"setup\">Setup</h2>\n");
    }

    #[test]
    fn a_marker_after_an_image_gives_the_img_its_attributes() {
        let html = render_site_html(
            "![A](./a.png \"T\")<ascribe-attributes width=\"600\" loading=\"lazy\"></ascribe-attributes>\n",
        );
        assert_eq!(
            html,
            "<p><img src=\"./a.png\" alt=\"A\" title=\"T\" width=\"600\" loading=\"lazy\" /></p>\n"
        );
    }

    #[test]
    fn a_marker_after_a_link_gives_the_a_its_attributes() {
        let html = render_site_html(
            "An [*API* key](/g#api-key \"A token.\")<ascribe-attributes data-ascribe-term=\"api-key\"></ascribe-attributes> here.\n",
        );
        assert_eq!(
            html,
            "<p>An <a href=\"/g#api-key\" title=\"A token.\" data-ascribe-term=\"api-key\"><em>API</em> key</a> here.</p>\n"
        );
    }

    #[test]
    fn a_marker_after_an_image_in_a_link_applies_to_the_image() {
        let html = render_site_html(
            "[![A](./a.png)<ascribe-attributes width=\"1\"></ascribe-attributes>](/x)<ascribe-attributes data-x=\"2\"></ascribe-attributes>\n",
        );
        assert_eq!(
            html,
            "<p><a href=\"/x\" data-x=\"2\"><img src=\"./a.png\" alt=\"A\" width=\"1\" /></a></p>\n"
        );
    }

    #[test]
    fn a_marker_replaces_an_attribute_the_element_has() {
        let html =
            render_site_html("![A](./a.png)<ascribe-attributes alt=\"B\"></ascribe-attributes>\n");
        assert_eq!(html, "<p><img src=\"./a.png\" alt=\"B\" /></p>\n");
    }

    #[test]
    fn values_decode_and_encode() {
        let html = render_site_html(
            "## T <ascribe-attributes id=\"a&amp;b &lt;c&gt; &quot;d&quot; &e\"></ascribe-attributes>\n",
        );
        assert_eq!(
            html,
            "<h2 id=\"a&amp;b &lt;c&gt; &quot;d&quot; &amp;e\">T</h2>\n"
        );
    }

    #[test]
    fn the_second_of_two_adjacent_markers_does_not_apply_to_the_image() {
        let html = render_site_html(
            "![A](./a.png)<ascribe-attributes width=\"1\"></ascribe-attributes><ascribe-attributes width=\"2\"></ascribe-attributes>\n",
        );
        assert_eq!(
            html,
            "<p><img src=\"./a.png\" alt=\"A\" width=\"1\" /><ascribe-attributes width=\"2\"></ascribe-attributes></p>\n"
        );
    }

    #[test]
    fn inline_html_has_no_paragraph() {
        assert_eq!(
            inline_html("Show the `x` file"),
            "Show the <code>x</code> file"
        );
    }
}
