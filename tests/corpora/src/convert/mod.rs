//! Converting a corpus's constructs to Ascribe, at volume.
//!
//! The converters are scripts in the sense the phase asks for: text to text,
//! line by line, no parsing of Markdown beyond what is needed to leave code
//! alone. They are **not complete** (SPEC §9 has nothing to say about most of
//! what these docs use), and they aren't meant to be: they produce realistic
//! Ascribe (containers, groups, includes, phrases, availability, heading ids)
//! at the size of a real documentation set, for the performance tests and to
//! see what the parser does with it.
//!
//! | Corpus | Converts |
//! |---|---|
//! | Elastic | `:::{note}` and friends to `@note`, `:::{dropdown}` to `@details`, tab sets to labeled `@variant` groups, steppers to `@steps`, `:::{include}` to `@include`, `:::{image}` to an image, `{{subs}}` to phrases (values from `docset.yml`), `applies_to` frontmatter and `applies-item` to `@available`, `[anchor]` heading ids to `@id` |
//! | Astro | Starlight asides (`:::tip[Title]`) to `@note`, `<Steps>` to `@steps`, `<Tabs>` to labeled `@variant` groups, `<Since v="…" />` to `@available` |
//! | Docker | `> [!NOTE]` alerts to `@note`, `{{< tabs >}}` to labeled `@variant` groups, `{{% include %}}` to `@include`, `{#id}` heading ids to `@id`, `summary-bar` availability to `@available` |
//!
//! Everything else stays as it was, which is the point of a sample: the parser
//! meets the constructs the converter doesn't know, and real prose.

pub mod astro;
pub mod docker;
pub mod elastic;

use std::collections::{BTreeMap, BTreeSet};
use std::io;
use std::path::Path;

use crate::corpus::Corpus;

/// A corpus converted to an Ascribe project.
#[derive(Clone, Debug, Default)]
pub struct Converted {
    /// The `ascribe.toml`.
    pub model: String,
    /// The converted pages and fragments, by path relative to the content
    /// root, sorted.
    pub files: BTreeMap<String, String>,
    /// Local images the pages reference, by path relative to the content root:
    /// created empty when the project is written, since the corpora are
    /// fetched without their images.
    pub assets: BTreeSet<String>,
    /// How many of each construct were converted.
    pub stats: BTreeMap<&'static str, usize>,
}

/// The content root of every converted project.
pub const CONTENT_ROOT: &str = "docs";

impl Converted {
    /// Writes the project under `root` (which must exist): `ascribe.toml`, the
    /// files under the content root, and the assets as empty files.
    ///
    /// # Errors
    ///
    /// Any error creating a directory or writing a file.
    pub fn write_to(&self, root: &Path) -> io::Result<()> {
        std::fs::write(root.join("ascribe.toml"), &self.model)?;
        let write = |rel: &str, text: &str| -> io::Result<()> {
            let path = root.join(CONTENT_ROOT).join(rel);
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::write(path, text)
        };
        for (rel, text) in &self.files {
            write(rel, text)?;
        }
        for rel in &self.assets {
            if !self.files.contains_key(rel) {
                write(rel, "")?;
            }
        }
        Ok(())
    }

    /// Counts a converted construct.
    pub(crate) fn count(&mut self, what: &'static str) {
        *self.stats.entry(what).or_insert(0) += 1;
    }

    /// The number of pages and fragments.
    pub fn pages(&self) -> usize {
        self.files.len()
    }
}

/// Converts a corpus's pages (as [`crate::corpus::pages`] lists them, plus
/// the repository's `docset.yml` text for Elastic).
pub fn convert(corpus: Corpus, pages: &[(String, String)], extra: &Extra) -> Converted {
    match corpus {
        Corpus::Elastic => elastic::convert(pages, extra),
        Corpus::Astro => astro::convert(pages),
        Corpus::Docker => docker::convert(pages),
    }
}

/// Files of the checkout other than pages that a converter reads.
#[derive(Clone, Debug, Default)]
pub struct Extra {
    /// Elastic's `docset.yml` (its `subs:` are the phrases' values).
    pub docset: String,
}

// ---------------------------------------------------------------------------
// Shared helpers

/// A file's frontmatter (without its `---` lines) and the rest.
pub(crate) fn split_frontmatter(text: &str) -> (Option<&str>, &str) {
    let Some(rest) = text.strip_prefix("---\n") else {
        return (None, text);
    };
    if let Some(end) = rest.find("\n---\n") {
        return (Some(&rest[..end + 1]), &rest[end + 5..]);
    }
    if let Some(front) = rest.strip_suffix("\n---") {
        return (Some(front), "");
    }
    (None, text)
}

/// A string as a YAML double-quoted scalar.
pub(crate) fn yaml_quote(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' | '\r' | '\t' => out.push(' '),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

/// A string as a TOML basic string.
pub(crate) fn toml_quote(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' | '\r' | '\t' => out.push(' '),
            c if c.is_control() => {}
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

/// The text of the first `# ` heading outside code, if any.
pub(crate) fn first_heading(body: &str) -> Option<String> {
    let mut fence: Option<(char, usize)> = None;
    for line in body.lines() {
        if let Some(open) = fence {
            if closes_fence(line, open) {
                fence = None;
            }
            continue;
        }
        if let Some(open) = opens_fence(line) {
            fence = Some(open);
            continue;
        }
        if let Some(text) = line.strip_prefix("# ") {
            let text = strip_heading_id(text.trim()).0;
            if !text.is_empty() {
                return Some(text.to_owned());
            }
        }
    }
    None
}

/// A fence opener's character and length.
pub(crate) fn opens_fence(line: &str) -> Option<(char, usize)> {
    let t = line.trim_start();
    if line.len() - t.len() > 3 && !line.starts_with('\t') {
        // Indented four or more: a code block or a list item's content; list
        // items indent fences deeper than that, so allow any indent.
    }
    let c = t.chars().next()?;
    if c != '`' && c != '~' {
        return None;
    }
    let n = t.chars().take_while(|&x| x == c).count();
    // A backtick fence's info string can't contain a backtick.
    (n >= 3 && !(c == '`' && t[n..].contains('`'))).then_some((c, n))
}

/// Whether `line` closes a fence opened with `open`.
pub(crate) fn closes_fence(line: &str, open: (char, usize)) -> bool {
    let t = line.trim();
    let n = t.chars().take_while(|&x| x == open.0).count();
    n >= open.1 && n == t.len()
}

/// A heading's text and a trailing `[id]` or `{#id}`, if it has one.
pub(crate) fn strip_heading_id(text: &str) -> (&str, Option<&str>) {
    let text = text.trim_end();
    if let Some(inner) = text.strip_suffix('}')
        && let Some(at) = inner.rfind("{#")
    {
        let id = &inner[at + 2..];
        if !id.is_empty()
            && id
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || "-_.:".contains(c))
        {
            return (inner[..at].trim_end(), Some(id));
        }
    }
    if let Some(inner) = text.strip_suffix(']')
        && let Some(at) = inner.rfind(" [")
    {
        let id = &inner[at + 2..];
        if !id.is_empty()
            && id.chars().next().is_some_and(|c| c.is_ascii_alphanumeric())
            && id
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || "-_.:".contains(c))
        {
            return (inner[..at].trim_end(), Some(id));
        }
    }
    (text, None)
}

/// An id Ascribe accepts (letters, digits, hyphens) made from an anchor.
pub(crate) fn id_from_anchor(anchor: &str) -> String {
    let id: String = anchor
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
        .collect();
    id.trim_matches('-').to_owned()
}

/// A prefix that makes an emitted line sit inside a list item or container.
pub(crate) fn prefixed(prefix: &str, line: &str) -> String {
    if line.trim().is_empty() {
        String::new()
    } else {
        format!("{prefix}{line}")
    }
}

/// Appends `line`, making sure a title line (which must begin a block) has a
/// blank line, a heading, or a directive line above it.
pub(crate) fn push_title(out: &mut Vec<String>, indent: &str, title: &str) {
    if out.last().is_some_and(|l| !l.trim().is_empty()) {
        out.push(String::new());
    }
    let title = title.trim();
    // A title that starts with a dot escapes it (Q193): `.\.NET`.
    let escape = if title.starts_with('.') { "\\" } else { "" };
    out.push(format!("{indent}.{escape}{title}"));
}

/// Adds a blank line unless the output already ends with one.
pub(crate) fn blank(out: &mut Vec<String>) {
    if out.last().is_some_and(|l| !l.trim().is_empty()) {
        out.push(String::new());
    }
}

/// The common start of a converted project's `ascribe.toml`.
pub(crate) fn model_header(out: &mut String) {
    out.push_str(
        "# Generated by tessera-corpora from a corpus of real documentation. Not committed.\n\
         spec = \"0.1\"\n\n\
         [project]\n\
         content-root = \"docs\"\n\n\
         [types.page]\n\
         default = true\n\n\
         [types.page.frontmatter]\n\
         title = { type = \"string\", phrases = true }\n\
         description = \"string?\"\n\n",
    );
}

/// The last part of a converted project's `ascribe.toml`: one site build.
pub(crate) fn model_footer(out: &mut String) {
    out.push_str(
        "\n[builds.site]\nvariants = \"switch\"\navailability = \"badge\"\n\n[editor]\nbuild = \"site\"\n",
    );
}

/// `---`-delimited frontmatter for a converted page.
pub(crate) fn frontmatter(
    title: &str,
    description: Option<&str>,
    available: Option<&str>,
) -> String {
    let mut out = String::from("---\n");
    out.push_str(&format!("title: {}\n", yaml_quote(title)));
    if let Some(d) = description {
        out.push_str(&format!("description: {}\n", yaml_quote(d)));
    }
    if let Some(a) = available {
        out.push_str(&format!("available: {}\n", yaml_quote(a)));
    }
    out.push_str("---\n\n");
    out
}

/// A page's description from its frontmatter YAML, when it's a plain string.
pub(crate) fn description_of(front: &serde_yaml::Value) -> Option<String> {
    front
        .get("description")
        .and_then(serde_yaml::Value::as_str)
        .map(|s| s.split_whitespace().collect::<Vec<_>>().join(" "))
        .filter(|s| !s.is_empty())
}

/// Local image destinations in a line (`![alt](dest)`), for placeholders.
pub(crate) fn image_destinations(line: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut rest = line;
    while let Some(at) = rest.find("![") {
        rest = &rest[at + 2..];
        let Some(close) = rest.find("](") else { break };
        let after = &rest[close + 2..];
        let end = after.find([')', ' ']).unwrap_or(after.len());
        out.push(after[..end].to_owned());
        rest = &after[end..];
    }
    out
}

/// The content-root-relative path of a link or image destination written in
/// the page at `page`, when it's local and stays inside the content root.
pub(crate) fn resolve_local(page: &str, dest: &str) -> Option<String> {
    let dest = dest.split(['#', '?']).next().unwrap_or("");
    if dest.is_empty()
        || dest.contains("://")
        || dest.starts_with("mailto:")
        || dest.starts_with('{')
        || dest.starts_with('<')
    {
        return None;
    }
    let mut segments: Vec<&str> = if dest.starts_with('/') {
        Vec::new()
    } else {
        let mut dir: Vec<&str> = page.split('/').collect();
        dir.pop();
        dir
    };
    for segment in dest.split('/') {
        match segment {
            "" | "." => {}
            ".." => {
                segments.pop()?;
            }
            s => segments.push(s),
        }
    }
    (!segments.is_empty()).then(|| segments.join("/"))
}

/// Records the local images of `line` as assets.
pub(crate) fn note_images(out: &mut Converted, page: &str, line: &str) {
    for dest in image_destinations(line) {
        if let Some(path) = resolve_local(page, &dest)
            && !path.ends_with(".md")
        {
            out.assets.insert(path);
        }
    }
}
