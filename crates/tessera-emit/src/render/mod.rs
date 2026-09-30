//! [`render_site_html`]: site markdown to HTML, with the site output's
//! attribute markers applied.
//!
//! It is comrak's CommonMark rendering with raw HTML passed through (SPEC
//! §9.5, "HTML passthrough"), plus the two things the markers add: a
//! `<ascribe-attributes>` marker that ends a heading gives the heading its
//! attributes (its `id`), and one directly after an image gives the `<img>`
//! its attributes. The editor preview renders with it. The Astro markdown
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
//!   to the image; the marker is removed.
//!
//! comrak escapes `<`, `>`, `&`, and `"` in every attribute value it writes,
//! so a tag ends at its first `>`, and the text of a heading or code span
//! can't be mistaken for a tag. Any other marker, and any text that only
//! looks like one, stays as raw HTML.

use comrak::{Options, markdown_to_html};

/// The tag name of the attribute marker.
const MARKER: &str = "ascribe-attributes";

/// Renders site markdown as HTML: CommonMark with raw HTML allowed, GFM's
/// tables, strikethrough, bare links, and task lists as Astro's defaults have
/// them, and the attribute markers applied.
pub fn render_site_html(markdown: &str) -> String {
    apply_markers(&markdown_to_html(markdown, &options()))
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
