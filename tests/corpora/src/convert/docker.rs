//! `docker/docs` to Ascribe.
//!
//! The source is Hugo Markdown: GitHub alerts (`> [!NOTE]`), the `tabs` and
//! `tab` shortcodes, `include`, and `## Heading {#id}`.

use std::collections::BTreeSet;

use serde_yaml::Value;

use super::{
    Converted, blank, closes_fence, description_of, first_heading, frontmatter, id_from_anchor,
    model_footer, model_header, note_images, opens_fence, push_title, split_frontmatter,
    strip_heading_id,
};

/// Converts the corpus.
pub fn convert(pages: &[(String, String)]) -> Converted {
    let mut out = Converted::default();
    let known: BTreeSet<&str> = pages.iter().map(|(p, _)| p.as_str()).collect();
    let mut states: BTreeSet<&'static str> = BTreeSet::new();
    for (path, text) in pages {
        // Hugo's `_index.md` is a section's page; a leading `_` would make
        // it a fragment here.
        let md = match path.rsplit_once('/') {
            Some((dir, "_index.md")) => format!("{dir}/index.md"),
            None if path == "_index.md" => "index.md".to_owned(),
            _ => path.clone(),
        };
        let (front, body) = split_frontmatter(text);
        let front: Option<Value> = front.and_then(|f| serde_yaml::from_str(f).ok());
        let body = convert_body(&md, body, &known, &mut out);
        let stem = md.rsplit('/').next().unwrap_or(&md).trim_end_matches(".md");
        let title = front
            .as_ref()
            .and_then(|f| f.get("title"))
            .and_then(Value::as_str)
            .map(str::to_owned)
            .or_else(|| first_heading(&body))
            .unwrap_or_else(|| stem.to_owned());
        let description = front.as_ref().and_then(description_of);
        // A sidebar badge is the closest thing Docker's pages have to a
        // lifecycle state.
        let badge = front
            .as_ref()
            .and_then(|f| f.get("params"))
            .and_then(|p| p.get("sidebar"))
            .and_then(|s| s.get("badge"))
            .and_then(|b| b.get("text"))
            .and_then(Value::as_str);
        let available = match badge {
            Some("Beta") => Some("docker beta"),
            Some("Experimental") => {
                states.insert("experimental");
                Some("docker experimental")
            }
            Some("Deprecated") => Some("docker deprecated"),
            _ => None,
        };
        if available.is_some() && !md.split('/').any(|s| s.starts_with('_') || s == "includes") {
            out.count("available");
        }
        let fragment = md.split('/').any(|s| s.starts_with('_') || s == "includes");
        let mut page = if fragment {
            String::new()
        } else {
            frontmatter(&title, description.as_deref(), available)
        };
        page.push_str(&body);
        out.files.insert(md, page);
    }
    let mut model = String::new();
    model_header(&mut model);
    model.push_str("[fragments]\npatterns = [\"includes/**\"]\n\n");
    model.push_str("[dimensions.product]\nlabel = \"Product\"\nvalues = [\"docker\"]\n\n");
    for state in &states {
        model.push_str(&format!("[lifecycle.{state}]\navailable = true\n\n"));
    }
    model_footer(&mut model);
    out.model = model;
    out
}

/// `{{< tab name="Go" >}}`: the name.
fn shortcode_arg<'a>(line: &'a str, name: &str) -> Option<&'a str> {
    let at = line.find(&format!("{name}=\""))? + name.len() + 2;
    let end = line[at..].find('"')?;
    Some(&line[at..at + end])
}

/// The shortcode's name: `tabs` for `{{< tabs group="x" >}}`, `/tab` for the
/// closing form.
fn shortcode(line: &str) -> Option<(&str, &str, bool)> {
    let t = line.trim();
    let (inner, percent) =
        if let Some(i) = t.strip_prefix("{{<").and_then(|r| r.strip_suffix(">}}")) {
            (i, false)
        } else if let Some(i) = t.strip_prefix("{{%").and_then(|r| r.strip_suffix("%}}")) {
            (i, true)
        } else {
            return None;
        };
    let inner = inner.trim();
    let (name, rest) = inner.split_once(char::is_whitespace).unwrap_or((inner, ""));
    Some((name, rest.trim(), percent))
}

fn convert_body(page: &str, body: &str, known: &BTreeSet<&str>, out: &mut Converted) -> String {
    let lines: Vec<&str> = body.lines().collect();
    let mut result: Vec<String> = Vec::new();
    let mut fence: Option<(char, usize)> = None;
    let mut tabs: Option<usize> = None;
    let mut ids: BTreeSet<String> = BTreeSet::new();
    let mut i = 0;
    while i < lines.len() {
        let line = lines[i];
        i += 1;
        if let Some(open) = fence {
            if closes_fence(line, open) {
                fence = None;
            }
            result.push(line.to_owned());
            continue;
        }
        if let Some(open) = opens_fence(line) {
            fence = Some(open);
            result.push(line.to_owned());
            continue;
        }
        let indent = &line[..line.len() - line.trim_start().len()];
        // A GitHub alert: `> [!NOTE]` and the quoted lines after it.
        if let Some(rest) = line.trim_start().strip_prefix("> [!")
            && let Some(kind) = rest.strip_suffix(']')
        {
            let attr = match kind.trim().to_ascii_lowercase().as_str() {
                "note" => Some(""),
                "tip" => Some(" {type=tip}"),
                "important" => Some(" {type=important}"),
                "warning" => Some(" {type=warning}"),
                "caution" => Some(" {type=caution}"),
                _ => None,
            };
            if let Some(attr) = attr {
                out.count("asides");
                result.push(format!("{indent}@note{attr}:"));
                while i < lines.len() && lines[i].trim_start().starts_with('>') {
                    let quoted = lines[i].trim_start().trim_start_matches('>');
                    let quoted = quoted.strip_prefix(' ').unwrap_or(quoted);
                    note_images(out, page, quoted);
                    result.push(if quoted.is_empty() {
                        String::new()
                    } else {
                        format!("{indent}{quoted}")
                    });
                    i += 1;
                }
                result.push(format!("{indent}@end"));
                continue;
            }
        }
        if let Some((name, rest, percent)) = shortcode(line) {
            match name {
                "tabs" if !percent => {
                    tabs = Some(0);
                    continue;
                }
                "/tabs" if !percent => {
                    if tabs.take().is_some_and(|arms| arms > 0) {
                        result.push(format!("{indent}@end"));
                    }
                    blank(&mut result);
                    continue;
                }
                "tab" if !percent && tabs.is_some() => {
                    let arms = tabs.get_or_insert(0);
                    *arms += 1;
                    let n = *arms;
                    out.count("tab arms");
                    if n == 1 {
                        out.count("tab groups");
                    }
                    let name = shortcode_arg(rest, "name")
                        .map_or_else(|| format!("Tab {n}"), str::to_owned);
                    push_title(&mut result, indent, &name);
                    result.push(format!("{indent}@variant:"));
                    continue;
                }
                "/tab" if !percent => {
                    blank(&mut result);
                    continue;
                }
                "include" => {
                    let file = rest.trim().trim_matches('"');
                    let target = format!("includes/{file}");
                    if !file.contains(' ') && known.contains(target.as_str()) {
                        out.count("includes");
                        blank(&mut result);
                        result.push(format!("{indent}@include: /{target}"));
                        continue;
                    }
                }
                _ => {}
            }
        }
        // `## Heading {#id}`.
        let trimmed = line.trim_start();
        if trimmed.starts_with('#') {
            let level = trimmed.chars().take_while(|&c| c == '#').count();
            if level <= 6 && trimmed[level..].starts_with(' ') {
                let (heading, anchor) = strip_heading_id(trimmed[level..].trim());
                if let Some(anchor) = anchor {
                    let id = id_from_anchor(anchor);
                    result.push(format!("{indent}{} {heading}", "#".repeat(level)));
                    if !id.is_empty() && ids.insert(id.clone()) {
                        out.count("heading ids");
                        result.push(format!("{indent}@id: {id}"));
                    }
                    continue;
                }
            }
        }
        // Links to a section's page follow the rename of `_index.md`.
        let line = line.replace("_index.md", "index.md");
        note_images(out, page, &line);
        result.push(line);
    }
    let mut text = result.join("\n");
    text.push('\n');
    text
}
