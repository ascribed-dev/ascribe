//! `llms.txt`: the index of a build's pages that agents read, as the [Web
//! Documentation Delivery Spec](https://agentdocsspec.com/spec/web/) lays it
//! out, for a content model with `[consumer] agents = true`.
//!
//! An index is an H1 with the site's name, a blockquote summary, and H2
//! sections of `- [Title](url): description` lines, one per page the build
//! publishes, each linking to the page's Markdown version
//! ([`AstroRouter::markdown_url`]). The name and the summary are the title
//! and description of the page at the base path; pages in a folder are a
//! section named after the folder (the title of its index page, when it has
//! one), and the pages outside every folder come first, under "Pages".
//!
//! An index longer than [`LLMS_TXT_LIMIT`] characters is split: the root
//! `llms.txt` keeps the pages outside every folder and links to one
//! `<folder>/llms.txt` per section, each a whole index of its own. A file
//! still longer than the limit is reported by `ascribe check`
//! (`llms-section-large`); it is written all the same.
//!
//! The emitter writes the files ([`llms_files`]) and the checks measure the
//! same ones, so what is reported is what is published.

use ascribe_core::RelPath;
use ascribe_model::{ContentModel, Field, FieldRole, TypeMatch};
use ascribe_syntax::{BlockKind, ParseOptions};

use crate::AstroRouter;
use crate::build::ResolvedPage;
use crate::index::heading_text;

/// The version of the delivery spec the outputs for agents follow. Its
/// checker, `afdocs`, is pinned in `examples/astro-site/package.json`.
pub const DELIVERY_SPEC: &str = "0.6.0";

/// The most characters an `llms.txt` file holds before the index is split
/// by section: what the delivery spec says fits in one fetch (`llms-txt-size`).
pub const LLMS_TXT_LIMIT: usize = 50_000;

/// The most characters a page's description has before its line in
/// `llms.txt` is too long (`description-too-long`).
pub const DESCRIPTION_LIMIT: usize = 300;

/// The heading of the section of pages outside every folder.
const TOP_SECTION: &str = "Pages";

/// One `llms.txt` file of a build.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LlmsFile {
    /// Where it's published, relative to the base path: `llms.txt`, or
    /// `<folder>/llms.txt` for a section's.
    pub path: String,
    /// Its URL, absolute when the content model has a `site`.
    pub url: String,
    /// Its text.
    pub text: String,
    /// The page a problem with the file is reported at: its section's index
    /// page, or the page at the base path for the root file, or else the
    /// first page it lists.
    pub page: Option<RelPath>,
}

impl LlmsFile {
    /// Its length in characters, which is what the limit counts.
    pub fn chars(&self) -> usize {
        self.text.chars().count()
    }
}

/// A page as its line in an index shows it.
struct Entry<'a> {
    page: &'a RelPath,
    id: String,
    title: String,
    url: String,
    description: Option<String>,
}

/// A section of an index: the pages in one folder, or outside every folder.
struct Section<'a> {
    /// The folder's slug, the first segment of its pages' entry ids; `None`
    /// for the pages outside every folder.
    key: Option<String>,
    heading: String,
    entries: Vec<Entry<'a>>,
}

impl Section<'_> {
    /// The section's index page: the page whose entry id is the folder's.
    fn index(&self) -> Option<&Entry<'_>> {
        let key = self.key.as_deref()?;
        self.entries.iter().find(|e| e.id == key)
    }
}

/// The `llms.txt` files of the pages a build publishes, in `pages` order:
/// the root file first, then a file per section when the index is split.
pub fn llms_files<'a>(
    model: &ContentModel,
    pages: impl IntoIterator<Item = &'a ResolvedPage>,
) -> Vec<LlmsFile> {
    let router = AstroRouter::from_consumer(&model.consumer);
    let absolute = |path: &str| absolute_url(model, &router.url_of(path));
    let entries: Vec<Entry<'a>> = pages
        .into_iter()
        .map(|page| {
            let id = router
                .markdown_path(&page.path)
                .trim_end_matches(".md")
                .to_owned();
            Entry {
                page: &page.path,
                title: page
                    .title
                    .as_deref()
                    .map(one_line)
                    .filter(|t| !t.is_empty())
                    .unwrap_or_else(|| id.clone()),
                url: absolute(&router.markdown_path(&page.path)),
                description: description(model, page),
                id,
            }
        })
        .collect();
    let root = entries.iter().find(|e| e.id == "index");
    let name = root.map_or_else(|| site_name(model), |e| e.title.clone());
    let summary = root.and_then(|e| e.description.clone()).unwrap_or_else(|| {
        "Every page of this documentation, each linked to its Markdown version.".to_owned()
    });
    let root_page = root.map(|e| e.page.clone());
    let sections = sections(entries);
    let first_page = sections
        .iter()
        .flat_map(|s| s.entries.first())
        .map(|e| e.page.clone())
        .next();

    let whole = index(&name, &summary, &sections.iter().collect::<Vec<_>>());
    if whole.chars().count() <= LLMS_TXT_LIMIT {
        return vec![LlmsFile {
            path: "llms.txt".to_owned(),
            url: absolute("llms.txt"),
            text: whole,
            page: root_page.or(first_page),
        }];
    }

    // Split: the pages outside every folder stay in the root file, which
    // links to a file per folder.
    let mut files = Vec::new();
    let mut top: Vec<&Section<'_>> = Vec::new();
    let mut listing = Vec::new();
    for section in &sections {
        let Some(key) = &section.key else {
            top.push(section);
            continue;
        };
        let path = format!("{key}/llms.txt");
        let url = absolute(&path);
        let index_page = section.index();
        let about = index_page
            .and_then(|e| e.description.clone())
            .unwrap_or_else(|| match section.entries.len() {
                1 => "1 page".to_owned(),
                n => format!("{n} pages"),
            });
        listing.push(format!(
            "- [{}]({url}): {about}",
            link_text(&section.heading)
        ));
        let summary = index_page
            .and_then(|e| e.description.clone())
            .unwrap_or_else(|| {
                format!(
                    "The pages of {} under {}, each linked to its Markdown version.",
                    name, section.heading
                )
            });
        files.push(LlmsFile {
            path,
            url,
            text: index(&section.heading, &summary, &[section]),
            page: index_page
                .or(section.entries.first())
                .map(|e| e.page.clone()),
        });
    }
    let mut text = header(&name, &summary);
    for section in &top {
        push_section(&mut text, section);
    }
    if !listing.is_empty() {
        text.push_str("\n## Sections\n\n");
        text.push_str(&listing.join("\n"));
        text.push('\n');
    }
    files.insert(
        0,
        LlmsFile {
            path: "llms.txt".to_owned(),
            url: absolute("llms.txt"),
            text,
            page: root_page.or(first_page),
        },
    );
    files
}

/// The entries in sections, in the order their first pages come: the pages
/// outside every folder, then each folder's. A folder's index page comes
/// first in its section, as the root page does in the first.
fn sections(entries: Vec<Entry<'_>>) -> Vec<Section<'_>> {
    let folder_of = |e: &Entry<'_>| -> Option<String> {
        (e.page.segments().count() > 1)
            .then(|| e.id.split('/').next().map(str::to_owned))
            .flatten()
    };
    let folders: Vec<String> = entries.iter().filter_map(folder_of).collect();
    let mut sections: Vec<Section<'_>> = Vec::new();
    for entry in entries {
        // A page beside a folder with its name (`guides.md` and `guides/`)
        // is that folder's index page.
        let key =
            folder_of(&entry).or_else(|| folders.contains(&entry.id).then(|| entry.id.clone()));
        let at = match sections.iter().position(|s| s.key == key) {
            Some(at) => at,
            None => {
                sections.push(Section {
                    heading: match &key {
                        Some(_) => folder_heading(entry.page),
                        None => TOP_SECTION.to_owned(),
                    },
                    key: key.clone(),
                    entries: Vec::new(),
                });
                sections.len() - 1
            }
        };
        sections[at].entries.push(entry);
    }
    // The pages outside every folder come first.
    sections.sort_by_key(|s| s.key.is_some());
    for section in &mut sections {
        let first = match section.key.as_deref() {
            Some(key) => section.entries.iter().position(|e| e.id == key),
            None => section.entries.iter().position(|e| e.id == "index"),
        };
        if let Some(at) = first {
            let entry = section.entries.remove(at);
            if section.key.is_some() {
                section.heading = entry.title.clone();
            }
            section.entries.insert(0, entry);
        }
    }
    sections
}

/// A folder's name as a heading: its first segment as written, with `-` and
/// `_` as spaces and the first letter capitalized.
fn folder_heading(page: &RelPath) -> String {
    let folder = page
        .segments()
        .next()
        .unwrap_or_default()
        .replace(['-', '_'], " ");
    let mut chars = folder.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().chain(chars).collect(),
        None => folder,
    }
}

/// A whole index: the header, then each section.
fn index(name: &str, summary: &str, sections: &[&Section<'_>]) -> String {
    let mut text = header(name, summary);
    for section in sections {
        push_section(&mut text, section);
    }
    text
}

fn header(name: &str, summary: &str) -> String {
    format!("# {}\n\n> {}\n", one_line(name), one_line(summary))
}

fn push_section(text: &mut String, section: &Section<'_>) {
    text.push_str(&format!("\n## {}\n\n", one_line(&section.heading)));
    for entry in &section.entries {
        text.push_str(&format!("- [{}]({})", link_text(&entry.title), entry.url));
        if let Some(description) = &entry.description {
            text.push_str(": ");
            text.push_str(description);
        }
        text.push('\n');
    }
}

/// The site's name when there's no page at the base path: its host.
fn site_name(model: &ContentModel) -> String {
    let site = model.consumer.site.as_deref().unwrap_or_default();
    let host = site.split_once("://").map_or(site, |(_, rest)| rest);
    host.trim_end_matches('/').to_owned()
}

/// A root-relative URL made absolute with the site's origin, when there is one.
fn absolute_url(model: &ContentModel, url: &str) -> String {
    match model.consumer.site.as_deref() {
        Some(origin) => format!("{}{url}", origin.trim_end_matches('/')),
        None => url.to_owned(),
    }
}

/// Text on one line: runs of whitespace, line breaks among them, as one space.
fn one_line(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Text for inside a link's brackets.
fn link_text(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for ch in one_line(text).chars() {
        if matches!(ch, '\\' | '[' | ']') {
            out.push('\\');
        }
        out.push(ch);
    }
    out
}

/// A page's description, as its line in an index shows it
/// ([`description_text`]). `None` when it has none.
pub fn description(model: &ContentModel, page: &ResolvedPage) -> Option<String> {
    let field = description_field(model, page.path.as_str())?;
    let raw = page
        .frontmatter
        .as_ref()?
        .get(field.name.as_str())?
        .as_str()?;
    description_text(model, raw)
}

/// A description as its line in an index shows it: on one line, with any
/// Markdown in it (a link, emphasis) as its text, so that it links nowhere.
/// `None` when that leaves nothing.
pub fn description_text(model: &ContentModel, raw: &str) -> Option<String> {
    let text = one_line(&markdown_text(model, raw));
    (!text.is_empty()).then_some(text)
}

/// The description field of the page at `path`: the field its content type
/// marks `role = "description"`.
pub fn description_field<'m>(model: &'m ContentModel, path: &str) -> Option<&'m Field> {
    let TypeMatch::One(ty) = model.type_for(path) else {
        return None;
    };
    ty.frontmatter.field_with_role(FieldRole::Description)
}

/// The text of a line of Markdown: a paragraph's inline content without its
/// markup. Anything that doesn't parse as one paragraph is kept as written.
fn markdown_text(model: &ContentModel, text: &str) -> String {
    let parsed = ascribe_syntax::parse(text, &ParseOptions::default());
    match parsed.blocks.as_slice() {
        [block] => match &block.kind {
            BlockKind::Paragraph(p) => heading_text(&p.inlines, model),
            _ => text.to_owned(),
        },
        _ => text.to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn link_text_escapes_brackets_and_folds_lines() {
        assert_eq!(link_text("a [b]\n c\\"), "a \\[b\\] c\\\\");
    }

    #[test]
    fn a_folder_heading_is_its_name_capitalized() {
        let page = |p: &str| RelPath::parse(p).expect("a path");
        assert_eq!(folder_heading(&page("Guides/a.md")), "Guides");
        assert_eq!(
            folder_heading(&page("getting-started/a.md")),
            "Getting started"
        );
    }
}
