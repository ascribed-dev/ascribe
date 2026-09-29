//! `elastic/docs-content` to Ascribe.
//!
//! The source is `docs-builder` Markdown: MyST-style `:::{directive}` blocks
//! (fences of three or more colons that nest by using more), `{{substitution}}`
//! from `docset.yml`, `applies_to` frontmatter, and `# Heading [anchor]`.

use std::collections::{BTreeMap, BTreeSet};

use serde_yaml::Value;

use super::{
    Converted, Extra, blank, closes_fence, description_of, first_heading, frontmatter,
    id_from_anchor, model_footer, model_header, note_images, opens_fence, prefixed, push_title,
    split_frontmatter, strip_heading_id, toml_quote,
};

/// Targets that have no version line (SPEC §4.4): a hosted service.
const VERSIONLESS: &[&str] = &["serverless", "ess", "ech", "vectordb"];

/// Lifecycle states the content model has built in.
const BUILT_IN_STATES: &[&str] = &["ga", "preview", "beta", "deprecated", "removed"];

#[derive(Default)]
struct Ctx {
    /// Phrase keys used, as written in the converted text.
    phrases: BTreeSet<String>,
    /// Availability targets used.
    targets: BTreeSet<String>,
    /// Lifecycle states used beyond the built-in ones.
    states: BTreeSet<String>,
    /// Constructs converted.
    counts: BTreeMap<&'static str, usize>,
    /// Heading anchors that had to change to be ids (`a_b` to `a-b`), by page,
    /// so the links to them can change too.
    renamed: BTreeMap<String, BTreeMap<String, String>>,
}

impl Ctx {
    fn count(&mut self, what: &'static str) {
        *self.counts.entry(what).or_insert(0) += 1;
    }
}

/// Converts the corpus.
pub fn convert(pages: &[(String, String)], extra: &Extra) -> Converted {
    let mut ctx = Ctx::default();
    let mut out = Converted::default();
    for (path, text) in pages {
        let text = convert_page(path, text, &mut ctx, &mut out);
        out.files.insert(path.clone(), text);
    }
    let renamed = std::mem::take(&mut ctx.renamed);
    for (path, text) in &mut out.files {
        *text = rewrite_anchor_links(path, text, &renamed, &mut ctx.counts);
    }
    for (what, n) in std::mem::take(&mut ctx.counts) {
        *out.stats.entry(what).or_insert(0) += n;
    }
    out.model = model(&ctx, &subs(&extra.docset));
    out
}

/// Links to an anchor that became a different id follow it.
fn rewrite_anchor_links(
    page: &str,
    text: &str,
    renamed: &BTreeMap<String, BTreeMap<String, String>>,
    counts: &mut BTreeMap<&'static str, usize>,
) -> String {
    if renamed.is_empty() {
        return text.to_owned();
    }
    let mut out = String::with_capacity(text.len());
    let mut fence: Option<(char, usize)> = None;
    for line in text.lines() {
        if let Some(open) = fence {
            if closes_fence(line, open) {
                fence = None;
            }
            out.push_str(line);
        } else if let Some(open) = opens_fence(line) {
            fence = Some(open);
            out.push_str(line);
        } else {
            let mut rest = line;
            while let Some(at) = rest.find("](") {
                out.push_str(&rest[..at + 2]);
                rest = &rest[at + 2..];
                let end = rest.find([')', ' ']).unwrap_or(rest.len());
                let dest = &rest[..end];
                let replaced = dest.split_once('#').and_then(|(file, anchor)| {
                    let target = if file.is_empty() {
                        Some(page.to_owned())
                    } else {
                        super::resolve_local(page, file)
                    }?;
                    let id = renamed.get(&target)?.get(anchor)?;
                    Some(format!("{file}#{id}"))
                });
                match replaced {
                    Some(new) => {
                        *counts.entry("links to renamed heading ids").or_insert(0) += 1;
                        out.push_str(&new);
                    }
                    None => out.push_str(dest),
                }
                rest = &rest[end..];
            }
            out.push_str(rest);
        }
        out.push('\n');
    }
    out
}

/// `subs:` of `docset.yml`.
fn subs(docset: &str) -> BTreeMap<String, String> {
    let Ok(value) = serde_yaml::from_str::<Value>(docset) else {
        return BTreeMap::new();
    };
    let mut out = BTreeMap::new();
    if let Some(map) = value.get("subs").and_then(Value::as_mapping) {
        for (k, v) in map {
            if let (Some(k), Some(v)) = (k.as_str(), v.as_str()) {
                out.insert(normalize_key(k), v.to_owned());
            }
        }
    }
    out
}

fn model(ctx: &Ctx, subs: &BTreeMap<String, String>) -> String {
    let mut m = String::new();
    model_header(&mut m);
    m.push_str("[fragments]\npatterns = []\n\n");
    if !ctx.targets.is_empty() {
        let values: Vec<String> = ctx.targets.iter().map(|t| toml_quote(t)).collect();
        let versionless: Vec<String> = ctx
            .targets
            .iter()
            .filter(|t| VERSIONLESS.contains(&t.as_str()))
            .map(|t| toml_quote(t))
            .collect();
        m.push_str("[dimensions.applies-to]\nlabel = \"Applies to\"\n");
        m.push_str(&format!("values = [{}]\n", values.join(", ")));
        m.push_str(&format!("versionless = [{}]\n\n", versionless.join(", ")));
    }
    for state in &ctx.states {
        let available = state != "unavailable";
        m.push_str(&format!("[lifecycle.{state}]\navailable = {available}\n\n"));
    }
    m.push_str("[phrases]\n");
    for key in &ctx.phrases {
        let value = subs.get(key).cloned().unwrap_or_else(|| key.clone());
        m.push_str(&format!("{key} = {}\n", toml_quote(&value)));
    }
    model_footer(&mut m);
    m
}

fn is_fragment(path: &str) -> bool {
    path.split('/').any(|s| s.starts_with('_'))
}

fn convert_page(path: &str, text: &str, ctx: &mut Ctx, out: &mut Converted) -> String {
    let (front, body) = split_frontmatter(text);
    let front: Option<Value> = front.and_then(|f| serde_yaml::from_str(f).ok());
    let body = convert_body(path, body, ctx, out);
    if is_fragment(path) {
        return body;
    }
    let stem = path
        .rsplit('/')
        .next()
        .unwrap_or(path)
        .trim_end_matches(".md");
    let title = first_heading(&body).unwrap_or_else(|| stem.to_owned());
    let description = front.as_ref().and_then(description_of);
    let available = front
        .as_ref()
        .and_then(|f| f.get("applies_to"))
        .and_then(|a| spec_from_applies_to(a, ctx));
    if available.is_some() {
        ctx.count("applies_to frontmatter");
    }
    let mut page = frontmatter(&title, description.as_deref(), available.as_deref());
    page.push_str(&body);
    page
}

// ---------------------------------------------------------------------------
// Availability

/// `applies_to` frontmatter as an availability spec.
fn spec_from_applies_to(value: &Value, ctx: &mut Ctx) -> Option<String> {
    let mut entries = Vec::new();
    collect_targets(value, true, ctx, &mut entries);
    (!entries.is_empty()).then(|| entries.join(", "))
}

fn collect_targets(value: &Value, top: bool, ctx: &mut Ctx, out: &mut Vec<String>) {
    let Some(map) = value.as_mapping() else {
        return;
    };
    for (key, v) in map {
        let Some(key) = key.as_str() else { continue };
        let target = normalize_key(key);
        match v {
            Value::Mapping(inner) => {
                if (target == "deployment" || target == "product") && top {
                    collect_targets(v, false, ctx, out);
                } else if let Some(first) = inner.values().find_map(scalar) {
                    // `serverless: { security: ga, observability: ga }`: one
                    // target; the project type isn't a dimension.
                    if let Some(e) = entry(&target, &first, ctx) {
                        out.push(e);
                    }
                }
            }
            other => {
                if let Some(s) = scalar(other)
                    && let Some(e) = entry(&target, &s, ctx)
                {
                    out.push(e);
                }
            }
        }
    }
}

fn scalar(v: &Value) -> Option<String> {
    match v {
        Value::String(s) => Some(s.clone()),
        Value::Number(n) => Some(n.to_string()),
        Value::Bool(b) => Some(b.to_string()),
        _ => None,
    }
}

/// One target's spec: `stack ga 9.0`, `stack (preview 9.0, ga 9.1)`,
/// `serverless ga`.
fn entry(target: &str, value: &str, ctx: &mut Ctx) -> Option<String> {
    if !is_name_word(target) || BUILT_IN_STATES.contains(&target) {
        return None;
    }
    let versioned = !VERSIONLESS.contains(&target);
    let mut history: Vec<(String, Option<Vec<u64>>)> = Vec::new();
    for part in value.split(',') {
        let mut words = part.split_whitespace();
        let state = match words.next().map(str::to_lowercase) {
            Some(s) if s == "all" || s == "none" => "ga".to_owned(),
            Some(s) if BUILT_IN_STATES.contains(&s.as_str()) => s,
            Some(s) if s == "unavailable" || s == "experimental" => {
                ctx.states.insert(s.clone());
                s
            }
            Some(_) | None => "ga".to_owned(),
        };
        let version = words.next().and_then(parse_version);
        history.push((state, version));
    }
    ctx.targets.insert(target.to_owned());
    if !versioned {
        let (state, _) = history.into_iter().next()?;
        return Some(format!("{target} {state}"));
    }
    // A history lists each state at the version where it begins, oldest first;
    // a state with no version is in effect at every version, so it can't
    // share a history with others.
    if history.iter().any(|(_, v)| v.is_some()) {
        history.retain(|(_, v)| v.is_some());
    }
    history.sort_by(|a, b| a.1.cmp(&b.1));
    history.dedup_by(|b, a| a.1 == b.1);
    let show = |(state, version): &(String, Option<Vec<u64>>)| match version {
        Some(v) => format!(
            "{state} {}",
            v.iter().map(u64::to_string).collect::<Vec<_>>().join(".")
        ),
        None => state.clone(),
    };
    Some(match history.as_slice() {
        [one] => format!("{target} {}", show(one)),
        many => format!(
            "{target} ({})",
            many.iter().map(show).collect::<Vec<_>>().join(", ")
        ),
    })
}

/// `9.0+`, `=9.1`, `9.0-9.2`, `9.0.0` as the version a state begins at.
fn parse_version(word: &str) -> Option<Vec<u64>> {
    let word = word.trim_start_matches('=');
    let word = word.split('-').next().unwrap_or("").trim_end_matches('+');
    let parts: Option<Vec<u64>> = word.split('.').map(|p| p.parse().ok()).collect();
    parts.filter(|p| !p.is_empty())
}

fn is_name_word(s: &str) -> bool {
    let mut chars = s.chars();
    chars.next().is_some_and(|c| c.is_ascii_alphabetic())
        && chars.all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
}

/// The spec of an `applies-item`: `stack: ga 9.1, serverless: ga`.
fn spec_from_inline(arg: &str, ctx: &mut Ctx) -> Option<String> {
    // Split at each `key:` that starts an entry (after the start or a comma).
    let mut entries = Vec::new();
    let mut starts = vec![];
    let bytes = arg.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if (i == 0 || arg[..i].trim_end().ends_with(',')) && bytes[i].is_ascii_alphabetic() {
            let word_end = arg[i..]
                .find(|c: char| !(c.is_ascii_alphanumeric() || c == '_' || c == '.'))
                .map_or(arg.len(), |n| i + n);
            if arg[word_end..].starts_with(':') {
                starts.push((i, word_end));
                i = word_end;
            }
        }
        i += 1;
    }
    for (n, &(start, key_end)) in starts.iter().enumerate() {
        let end = starts.get(n + 1).map_or(arg.len(), |s| s.0);
        let key = arg[start..key_end].rsplit('.').next().unwrap_or("");
        let value = arg[key_end + 1..end].trim().trim_end_matches(',').trim();
        if let Some(e) = entry(&normalize_key(key), value, ctx) {
            entries.push(e);
        }
    }
    (!entries.is_empty()).then(|| entries.join(", "))
}

// ---------------------------------------------------------------------------
// Substitutions

/// A substitution key as a phrase key.
fn normalize_key(key: &str) -> String {
    key.trim()
        .to_lowercase()
        .chars()
        .map(|c| if c == '.' || c == '_' { '-' } else { c })
        .collect()
}

fn is_phrase_key(key: &str) -> bool {
    let mut chars = key.chars();
    chars.next().is_some_and(|c| c.is_ascii_lowercase())
        && chars.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
}

/// `{{key}}` as `{key}`, outside inline code (Ascribe never substitutes in a
/// code span, so `` `{{es}}` `` is left as written).
fn phrases_in(line: &str, ctx: &mut Ctx, in_code_spans_too: bool) -> String {
    let mut out = String::with_capacity(line.len());
    let mut rest = line;
    while !rest.is_empty() {
        let tick = if in_code_spans_too {
            None
        } else {
            rest.find('`')
        };
        let brace = rest.find("{{");
        match (tick, brace) {
            (Some(t), b) if b.is_none_or(|b| t < b) => {
                let n = rest[t..].chars().take_while(|&c| c == '`').count();
                let run = &rest[t..t + n];
                let after = &rest[t + n..];
                out.push_str(&rest[..t]);
                // The span ends at the next run of exactly this length.
                let mut search = 0;
                let mut close = None;
                while let Some(at) = after[search..].find(run) {
                    let at = search + at;
                    let next = after[at + n..].chars().next();
                    if next != Some('`') {
                        close = Some(at + n);
                        break;
                    }
                    search = at + n;
                    while after[search..].starts_with('`') {
                        search += 1;
                    }
                }
                match close {
                    Some(end) => {
                        if after[..end].contains("{{") {
                            ctx.count("substitutions left in code spans");
                        }
                        out.push_str(run);
                        out.push_str(&after[..end]);
                        rest = &after[end..];
                    }
                    None => {
                        out.push_str(run);
                        rest = after;
                    }
                }
            }
            (_, Some(b)) => {
                out.push_str(&rest[..b]);
                let after = &rest[b + 2..];
                if let Some(end) = after.find("}}") {
                    let key = normalize_key(&after[..end]);
                    if is_phrase_key(&key) && !after[..end].contains('{') {
                        ctx.phrases.insert(key.clone());
                        ctx.count("substitutions");
                        out.push('{');
                        out.push_str(&key);
                        out.push('}');
                        rest = &after[end + 2..];
                        continue;
                    }
                }
                out.push_str("{{");
                rest = after;
            }
            _ => {
                out.push_str(rest);
                break;
            }
        }
    }
    out
}

// ---------------------------------------------------------------------------
// The body

#[derive(Debug)]
enum Kind {
    Note,
    Details,
    TabSet {
        arms: usize,
    },
    Tab,
    Stepper {
        steps: usize,
    },
    Step,
    Include,
    Image {
        alt: Option<String>,
        path: String,
    },
    ApplySwitch,
    ApplyItem,
    /// A directive the converter doesn't know: left as written.
    Pass,
}

#[derive(Debug)]
struct Frame {
    kind: Kind,
    colons: usize,
    indent: String,
    /// What the frame adds to the indentation of everything inside it.
    prefix: String,
}

/// `   :::{name} argument`, as its parts.
fn opener(line: &str) -> Option<(&str, usize, &str, &str)> {
    let t = line.trim_start();
    let indent = &line[..line.len() - t.len()];
    let colons = t.chars().take_while(|&c| c == ':').count();
    if colons < 3 {
        return None;
    }
    let rest = t[colons..].strip_prefix('{')?;
    let close = rest.find('}')?;
    let name = &rest[..close];
    if name.is_empty()
        || !name
            .chars()
            .all(|c| c.is_ascii_lowercase() || c == '-' || c == '_')
    {
        return None;
    }
    Some((indent, colons, name, rest[close + 1..].trim()))
}

/// `:::`, alone on its line.
fn closer(line: &str) -> Option<usize> {
    let t = line.trim();
    let colons = t.chars().take_while(|&c| c == ':').count();
    (colons >= 3 && colons == t.len()).then_some(colons)
}

/// An option line of a directive: `:sync: npm`.
fn option(line: &str) -> Option<(&str, &str)> {
    let t = line.trim();
    let rest = t.strip_prefix(':')?;
    let end = rest.find(':')?;
    let key = &rest[..end];
    (!key.is_empty()
        && key
            .chars()
            .all(|c| c.is_ascii_lowercase() || c == '-' || c == '_'))
    .then(|| (key, rest[end + 1..].trim()))
}

fn convert_body(path: &str, body: &str, ctx: &mut Ctx, converted: &mut Converted) -> String {
    let lines: Vec<&str> = body.lines().collect();
    let mut out: Vec<String> = Vec::new();
    let mut stack: Vec<Frame> = Vec::new();
    let mut fence: Option<((char, usize), bool)> = None;
    let mut ids: BTreeSet<String> = BTreeSet::new();
    let mut in_comment = false;
    let mut i = 0;
    while i < lines.len() {
        let line = lines[i];
        i += 1;
        let prefix: String = stack.iter().map(|f| f.prefix.as_str()).collect();
        // An HTML comment is left alone: it can hold directive syntax as text.
        if in_comment {
            in_comment = !line.contains("-->");
            out.push(prefixed(&prefix, line));
            continue;
        }
        if fence.is_none() && line.trim_start().starts_with("<!--") {
            in_comment = line.rfind("<!--") > line.rfind("-->");
            out.push(prefixed(&prefix, line));
            continue;
        }
        if let Some((open, subs)) = fence {
            if closes_fence(line, open) {
                fence = None;
            }
            let text = if subs && fence.is_some() {
                phrases_in(line, ctx, true)
            } else {
                line.to_owned()
            };
            out.push(prefixed(&prefix, &text));
            continue;
        }
        if let Some(open) = opens_fence(line) {
            // A fence that opts in to substitution opts in to phrases.
            let info = line.trim_start().trim_start_matches(['`', '~']);
            let subs = info.split_whitespace().any(|w| w == "subs=true");
            let shown = if subs {
                ctx.count("fences with subs=true");
                line.replace("subs=true", "phrases=true")
            } else {
                line.to_owned()
            };
            fence = Some((open, subs));
            out.push(prefixed(&prefix, &shown));
            continue;
        }
        if let Some((indent, colons, name, arg)) = opener(line) {
            // Option lines directly under the opener.
            let mut options: Vec<(String, String)> = Vec::new();
            while i < lines.len() {
                match option(lines[i]) {
                    Some((k, v)) if closer(lines[i]).is_none() => {
                        options.push((k.to_owned(), v.to_owned()));
                        i += 1;
                    }
                    _ => break,
                }
            }
            let arg = phrases_in(arg, ctx, false);
            open_frame(
                &mut out, &mut stack, ctx, converted, path, indent, colons, name, &arg, &options,
                &prefix,
            );
            continue;
        }
        if let Some(colons) = closer(line)
            && stack.last().is_some_and(|f| f.colons == colons)
            && let Some(frame) = stack.pop()
        {
            let outer: String = stack.iter().map(|f| f.prefix.as_str()).collect();
            close_frame(&mut out, &mut stack, converted, path, frame, line, &outer);
            continue;
        }
        // A blank line under `@available` would keep it from touching its
        // block (SPEC §3.8).
        if line.trim().is_empty()
            && out
                .last()
                .is_some_and(|l| l.trim_start().starts_with("@available:"))
        {
            continue;
        }
        // An ordinary line.
        let mut text = phrases_in(line, ctx, false);
        note_images(converted, path, &text);
        let trimmed = text.trim_start();
        if trimmed.starts_with('#') {
            let indent_len = text.len() - trimmed.len();
            let level = trimmed.chars().take_while(|&c| c == '#').count();
            if level <= 6 && trimmed[level..].starts_with(' ') {
                let (heading, anchor) = strip_heading_id(trimmed[level..].trim());
                if let Some(anchor) = anchor {
                    let id = id_from_anchor(anchor);
                    let heading_line =
                        format!("{}{} {heading}", &text[..indent_len], "#".repeat(level));
                    out.push(prefixed(&prefix, &heading_line));
                    if !id.is_empty() && ids.insert(id.clone()) {
                        converted.count("heading ids");
                        if id != anchor {
                            ctx.renamed
                                .entry(path.to_owned())
                                .or_default()
                                .insert(anchor.to_owned(), id.clone());
                        }
                        out.push(prefixed(
                            &prefix,
                            &format!("{}@id: {id}", &text[..indent_len]),
                        ));
                    }
                    continue;
                }
            }
        }
        if !stack.is_empty() && text.trim().is_empty() {
            text.clear();
        }
        out.push(prefixed(&prefix, &text));
    }
    // Frames never closed: close them, so the output is well formed.
    while let Some(frame) = stack.pop() {
        let outer: String = stack.iter().map(|f| f.prefix.as_str()).collect();
        close_frame(&mut out, &mut stack, converted, path, frame, "", &outer);
    }
    let mut text = out.join("\n");
    text.push('\n');
    text
}

#[allow(clippy::too_many_arguments)]
fn open_frame(
    out: &mut Vec<String>,
    stack: &mut Vec<Frame>,
    ctx: &mut Ctx,
    converted: &mut Converted,
    path: &str,
    indent: &str,
    colons: usize,
    name: &str,
    arg: &str,
    options: &[(String, String)],
    prefix: &str,
) {
    let lead = format!("{prefix}{indent}");
    let mut frame = Frame {
        kind: Kind::Pass,
        colons,
        indent: lead.clone(),
        prefix: String::new(),
    };
    match name {
        "note" | "tip" | "important" | "warning" | "caution" => {
            ctx.count("asides");
            let attr = if name == "note" {
                String::new()
            } else {
                format!(" {{type={name}}}")
            };
            out.push(format!("{lead}@note{attr}:"));
            frame.kind = Kind::Note;
        }
        "admonition" => {
            ctx.count("asides");
            let title = if arg.is_empty() { "Note" } else { arg };
            push_title(out, &lead, title);
            out.push(format!("{lead}@note:"));
            frame.kind = Kind::Note;
        }
        "dropdown" => {
            ctx.count("dropdowns");
            let title = if arg.is_empty() { "Details" } else { arg };
            push_title(out, &lead, title);
            out.push(format!("{lead}@details:"));
            frame.kind = Kind::Details;
        }
        "tab-set" => frame.kind = Kind::TabSet { arms: 0 },
        "tab-item" => {
            ctx.count("tab arms");
            let n = match stack.last_mut() {
                Some(Frame {
                    kind: Kind::TabSet { arms },
                    ..
                }) => {
                    *arms += 1;
                    *arms
                }
                _ => 0,
            };
            if n == 1 {
                ctx.count("tab groups");
            }
            let title = if arg.is_empty() {
                format!("Tab {n}")
            } else {
                arg.to_owned()
            };
            // The group's indentation is the tab set's, not the item's.
            let group_indent = stack.last().map_or(lead.clone(), |f| f.indent.clone());
            push_title(out, &group_indent, &title);
            out.push(format!("{group_indent}@variant:"));
            frame.kind = Kind::Tab;
        }
        "stepper" => frame.kind = Kind::Stepper { steps: 0 },
        "step" => {
            let n = match stack.last_mut() {
                Some(Frame {
                    kind: Kind::Stepper { steps },
                    ..
                }) => {
                    *steps += 1;
                    *steps
                }
                _ => 1,
            };
            if n == 1 {
                ctx.count("steppers");
                blank(out);
                let group_indent = stack.last().map_or(lead.clone(), |f| f.indent.clone());
                out.push(format!("{group_indent}@steps"));
            }
            ctx.count("steps");
            let title = if arg.is_empty() {
                format!("Step {n}")
            } else {
                arg.to_owned()
            };
            let group_indent = stack.last().map_or(lead.clone(), |f| f.indent.clone());
            let marker = format!("{n}. ");
            out.push(format!("{group_indent}{marker}**{title}**"));
            frame.prefix = " ".repeat(marker.len());
            frame.kind = Kind::Step;
        }
        "include" => {
            ctx.count("includes");
            blank(out);
            out.push(format!("{lead}@include: {arg}"));
            frame.kind = Kind::Include;
        }
        "image" => {
            let alt = options
                .iter()
                .find(|(k, _)| k == "alt")
                .map(|(_, v)| v.clone());
            frame.kind = Kind::Image {
                alt,
                path: arg.to_owned(),
            };
        }
        "applies-switch" => frame.kind = Kind::ApplySwitch,
        "applies-item" => {
            if let Some(spec) = spec_from_inline(arg, ctx) {
                ctx.count("applies-item");
                blank(out);
                out.push(format!("{lead}@available: {spec}"));
            }
            frame.kind = Kind::ApplyItem;
        }
        _ => {
            ctx.count("directives left as written");
            let head = format!("{indent}{}{{{name}}} {arg}", ":".repeat(colons));
            out.push(prefixed(prefix, head.trim_end()));
            for (k, v) in options {
                let option = format!("{indent}:{k}: {v}");
                out.push(prefixed(prefix, option.trim_end()));
            }
        }
    }
    let _ = (converted, path);
    stack.push(frame);
}

fn close_frame(
    out: &mut Vec<String>,
    stack: &mut [Frame],
    converted: &mut Converted,
    path: &str,
    frame: Frame,
    original: &str,
    prefix: &str,
) {
    match frame.kind {
        Kind::Note | Kind::Details => out.push(format!("{}@end", frame.indent)),
        Kind::TabSet { arms } => {
            if arms > 0 {
                out.push(format!("{}@end", frame.indent));
            }
            blank(out);
        }
        Kind::Tab | Kind::Step | Kind::ApplyItem => blank(out),
        Kind::Stepper { .. } | Kind::Include | Kind::ApplySwitch => blank(out),
        Kind::Image { alt, path: src } => {
            converted.count("images");
            let text = format!("![{}]({src})", alt.unwrap_or_default());
            note_images(converted, path, &text);
            blank(out);
            out.push(format!("{}{text}", frame.indent));
            blank(out);
        }
        Kind::Pass => out.push(prefixed(prefix, original)),
    }
    let _ = stack;
}
