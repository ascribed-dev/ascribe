//! `withastro/docs` to Ascribe.
//!
//! The source is MDX: Starlight asides (`:::tip[Title]`), `<Steps>`,
//! `<Tabs>` and `<TabItem label="…">`, and `<Since v="…" />` for the version
//! a feature arrived in.

use std::collections::BTreeSet;

use serde_yaml::Value;

use super::{
    Converted, blank, closes_fence, description_of, first_heading, frontmatter, model_footer,
    model_header, note_images, opens_fence, prefixed, push_title, split_frontmatter, toml_quote,
};

/// Converts the corpus.
pub fn convert(pages: &[(String, String)]) -> Converted {
    let mut out = Converted::default();
    let mut targets: BTreeSet<String> = BTreeSet::new();
    for (path, text) in pages {
        let md = md_path(path);
        let (front, body) = split_frontmatter(text);
        let front: Option<Value> = front.and_then(|f| serde_yaml::from_str(f).ok());
        let body = convert_body(&md, body, &mut out, &mut targets);
        let stem = md.rsplit('/').next().unwrap_or(&md).trim_end_matches(".md");
        let title = front
            .as_ref()
            .and_then(|f| f.get("title"))
            .and_then(Value::as_str)
            .map(str::to_owned)
            .or_else(|| first_heading(&body))
            .unwrap_or_else(|| stem.to_owned());
        let description = front.as_ref().and_then(description_of);
        let mut page = frontmatter(&title, description.as_deref(), None);
        page.push_str(&body);
        out.files.insert(md, page);
    }
    let mut model = String::new();
    model_header(&mut model);
    if !targets.is_empty() {
        let values: Vec<String> = targets.iter().map(|t| toml_quote(t)).collect();
        model.push_str(&format!(
            "[dimensions.product]\nlabel = \"Product\"\nvalues = [{}]\n",
            values.join(", ")
        ));
    }
    model_footer(&mut model);
    out.model = model;
    out
}

fn md_path(path: &str) -> String {
    format!("{}.md", path.strip_suffix(".mdx").unwrap_or(path))
}

/// An attribute's value in a JSX tag: `label="npm"`.
fn attribute<'a>(tag: &'a str, name: &str) -> Option<&'a str> {
    let at = tag.find(&format!("{name}=\""))? + name.len() + 2;
    let end = tag[at..].find('"')?;
    Some(&tag[at..at + end])
}

/// A target name from a package name: `@astrojs/rss` is `astrojs-rss`.
fn target_of(pkg: Option<&str>) -> String {
    let Some(pkg) = pkg else {
        return "astro".to_owned();
    };
    let name: String = pkg
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() {
                c.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect();
    let name = name.trim_matches('-').to_owned();
    if name.chars().next().is_some_and(|c| c.is_ascii_alphabetic()) {
        name
    } else {
        format!("pkg-{name}")
    }
}

fn convert_body(
    page: &str,
    body: &str,
    out: &mut Converted,
    targets: &mut BTreeSet<String>,
) -> String {
    let mut lines: Vec<String> = Vec::new();
    let mut fence: Option<(char, usize)> = None;
    let mut in_aside = false;
    // Inside `<Tabs>`: the arms so far, and how far the current item is
    // indented (its content is dedented by that and two more).
    let mut tabs: Option<usize> = None;
    let mut in_item: Option<usize> = None;
    let mut swallow_blank = false;
    for raw in body.lines() {
        let line = match in_item {
            Some(strip) => {
                let lead = raw.len() - raw.trim_start().len();
                &raw[lead.min(strip)..]
            }
            _ => raw,
        };
        if let Some(open) = fence {
            if closes_fence(line, open) {
                fence = None;
            }
            lines.push(line.to_owned());
            continue;
        }
        if let Some(open) = opens_fence(line) {
            fence = Some(open);
            swallow_blank = false;
            lines.push(line.to_owned());
            continue;
        }
        let t = line.trim();
        if swallow_blank {
            if t.is_empty() {
                continue;
            }
            swallow_blank = false;
        }
        // Imports and exports are MDX, not content.
        if t.starts_with("import ") && t.contains(" from ") || t.starts_with("export ") {
            continue;
        }
        if let Some(rest) = t.strip_prefix(":::") {
            let indent = &line[..line.len() - line.trim_start().len()];
            if rest.is_empty() && in_aside {
                in_aside = false;
                lines.push(format!("{indent}@end"));
                continue;
            }
            let (name, title) = match rest.split_once('[') {
                Some((n, t)) => (n, t.strip_suffix(']').map(str::to_owned)),
                None => (rest, None),
            };
            let name = name.split('{').next().unwrap_or(name);
            let kind = match name {
                "note" => Some(""),
                "tip" => Some(" {type=tip}"),
                "caution" => Some(" {type=caution}"),
                "danger" => Some(" {type=warning}"),
                _ => None,
            };
            if let Some(attr) = kind {
                out.count("asides");
                in_aside = true;
                if let Some(title) = title {
                    push_title(&mut lines, indent, &title);
                }
                lines.push(format!("{indent}@note{attr}:"));
                continue;
            }
        }
        if t == "<Steps>" {
            out.count("steps");
            blank(&mut lines);
            lines.push("@steps".to_owned());
            swallow_blank = true;
            continue;
        }
        if t == "</Steps>" {
            blank(&mut lines);
            continue;
        }
        if t.starts_with("<Tabs") && !t.contains("</Tabs>") {
            tabs = Some(0);
            continue;
        }
        if t == "</Tabs>" {
            if tabs.take().is_some_and(|arms| arms > 0) {
                lines.push("@end".to_owned());
            }
            blank(&mut lines);
            continue;
        }
        if t.starts_with("<TabItem") && tabs.is_some() {
            let arms = tabs.get_or_insert(0);
            *arms += 1;
            let n = *arms;
            out.count("tab arms");
            if n == 1 {
                out.count("tab groups");
            }
            let label = attribute(t, "label").map_or_else(|| format!("Tab {n}"), str::to_owned);
            push_title(&mut lines, "", &label);
            lines.push("@variant:".to_owned());
            let indent = raw.len() - raw.trim_start().len();
            in_item = Some(indent + 2);
            continue;
        }
        if t == "</TabItem>" {
            in_item = None;
            blank(&mut lines);
            continue;
        }
        // `<p><Since v="4.10.3" /></p>`, or `<Since … />` alone.
        if let Some(at) = t.find("<Since ") {
            let tag = &t[at..];
            if let Some(v) = attribute(tag, "v") {
                let target = target_of(attribute(tag, "pkg"));
                let version: Vec<&str> = v.split('.').collect();
                if !version.is_empty() && version.iter().all(|p| p.parse::<u64>().is_ok()) {
                    out.count("available");
                    targets.insert(target.clone());
                    blank_unless_heading(&mut lines);
                    lines.push(format!("@available: {target} ga {v}"));
                    swallow_blank = true;
                    continue;
                }
            }
        }
        note_images(out, page, line);
        lines.push(prefixed("", line));
    }
    let mut text = lines.join("\n");
    text.push('\n');
    text
}

/// A blank line before a directive, unless it follows a heading (where it
/// describes the section and must touch nothing else).
fn blank_unless_heading(lines: &mut Vec<String>) {
    if lines.last().is_some_and(|l| l.starts_with('#')) {
        return;
    }
    blank(lines);
}
