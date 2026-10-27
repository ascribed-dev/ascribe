//! `llms.txt`: the index of a build's pages that agents read, as the [Web
//! Documentation Delivery Spec](https://agentdocsspec.com/spec/web/) lays it
//! out, for a content model with `[consumer] agents = true`.
//!
//! An index is an H1 with the site's name, a blockquote summary, and H2
//! sections of `- [Title](url): description` lines, one per page the build
//! publishes and offers (not one that's `unlisted`), from the build's page
//! records ([`crate::pages`]), each linking to the page's Markdown version
//! ([`AstroRouter::markdown_url`]). The name and the summary are the title
//! and description of the page at the base path. The sections are the
//! navigation's ([`crate::navigation`]): the root's pages first, under its
//! label, "Start" unless `ascribe.nav` gives another, then each top section
//! in order, its nested sections' pages folded into it in reading order. A
//! page keeps its title here, not its `nav-label`.
//!
//! An index longer than [`LLMS_TXT_LIMIT`] characters is split: the root
//! `llms.txt` keeps the root's pages and links to one `<folder>/llms.txt`
//! per top section, each a whole index of its own. A file
//! still longer than the limit is reported by `ascribe check`
//! (`llms-section-large`); it is written all the same.
//!
//! For a project with `[releases]`, an index's summary ends with the
//! releases the build describes ([`releases_described`]), which each
//! page's Markdown version says too (decision #1028).
//!
//! In a versioned build with more than one live line, each line has its
//! own: its heading names the line, `Loom (Self-managed 3.4)`, and an older
//! line's opens with a line pointing at the current line's index.
//!
//! The emitter writes the files ([`llms_files`]) and the checks measure the
//! same ones, so what is reported is what is published.

use ascribe_core::RelPath;
use ascribe_model::{AvailabilityMode, Build, ContentModel, Field, FieldRole};
use ascribe_syntax::{BlockKind, ParseOptions};
use serde_yaml_ng::Value;

use std::collections::BTreeMap;

use crate::AstroRouter;
use crate::build::ResolvedPage;
use crate::index::heading_text;
use crate::navigation::{BuildNavigation, folder_label};
use crate::pages::{PageRecord, PageRecords, current_line_router};

/// The version of the delivery spec the outputs for agents follow. Its
/// checker, `afdocs`, is pinned in `examples/astro-site/package.json`.
pub const DELIVERY_SPEC: &str = "0.6.0";

/// The most characters an `llms.txt` file holds before the index is split
/// by section: what the delivery spec says fits in one fetch (`llms-txt-size`).
pub const LLMS_TXT_LIMIT: usize = 50_000;

/// The most characters a page's description has before its line in
/// `llms.txt` is too long (`description-too-long`).
pub const DESCRIPTION_LIMIT: usize = 300;

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

/// A section of an index: a top section of the navigation, or the root's
/// pages.
struct Section<'a> {
    /// The folder's slug, the first segment of its pages' entry ids; `None`
    /// for the root's pages.
    key: Option<String>,
    heading: String,
    /// Whether the first entry is the section's own page.
    own: bool,
    entries: Vec<Entry<'a>>,
}

impl Section<'_> {
    /// The section's own page, when it's offered.
    fn index(&self) -> Option<&Entry<'_>> {
        self.entries.first().filter(|_| self.own)
    }
}

/// The `llms.txt` files of `build`, from its page records ([`crate::pages::records`])
/// in the sections of its navigation ([`crate::Navigation::share`]), in
/// their order: the root file first, then a file per section when the
/// index is split. A page that isn't offered, being `unlisted`, isn't in
/// any. `model` is the one the build's pages are routed with
/// ([`ContentModel::for_build`]).
pub fn llms_files(
    model: &ContentModel,
    build: &Build,
    records: &PageRecords,
    navigation: &BuildNavigation<'_>,
) -> Vec<LlmsFile> {
    let router = AstroRouter::from_consumer(&model.consumer);
    let absolute = |path: &str| absolute_url(model, &router.url_of(path));
    let line = line_header(model, build);
    let entries = entries(&router, records, |record| {
        absolute(&router.markdown_path(&record.source))
    });
    let root = entries.iter().find(|e| e.id == "index");
    let (name, summary) = name_and_summary(model, root);
    let root_page = root.map(|e| e.page.clone());
    let sections = sections(entries, navigation);
    let first_page = sections
        .iter()
        .flat_map(|s| s.entries.first())
        .map(|e| e.page.clone())
        .next();

    let whole = index(&line, &name, &summary, &sections.iter().collect::<Vec<_>>());
    if whole.chars().count() <= LLMS_TXT_LIMIT {
        return vec![LlmsFile {
            path: "llms.txt".to_owned(),
            url: absolute("llms.txt"),
            text: whole,
            page: root_page.or(first_page),
        }];
    }

    // Split: the root's pages stay in the root file, which links to a file
    // per top section.
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
            text: index(&line, &section.heading, &summary, &[section]),
            page: index_page
                .or(section.entries.first())
                .map(|e| e.page.clone()),
        });
    }
    let mut text = header(&line, &name, &summary);
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

/// The offered pages of `records` as entries, each with the address `url`
/// gives it.
fn entries<'a>(
    router: &AstroRouter,
    records: &'a PageRecords,
    url: impl Fn(&PageRecord) -> String,
) -> Vec<Entry<'a>> {
    records
        .pages
        .iter()
        .filter(|record| record.offered())
        .map(|record| {
            let id = router
                .markdown_path(&record.source)
                .trim_end_matches(".md")
                .to_owned();
            Entry {
                page: &record.source,
                title: record
                    .title
                    .as_deref()
                    .map(one_line)
                    .filter(|t| !t.is_empty())
                    .unwrap_or_else(|| id.clone()),
                url: url(record),
                description: record.description.clone(),
                id,
            }
        })
        .collect()
}

/// An index's name and summary: the title and description of the page at
/// the base path, `root`, or else the site's host and a sentence.
fn name_and_summary(model: &ContentModel, root: Option<&Entry<'_>>) -> (String, String) {
    let name = root.map_or_else(|| site_name(model), |e| e.title.clone());
    let summary = root.and_then(|e| e.description.clone()).unwrap_or_else(|| {
        "Every page of this documentation, each linked to its Markdown version.".to_owned()
    });
    (name, summary)
}

/// The entries in the navigation's sections ([`crate::navigation`]): the
/// root's own page and its pages first, under the root's label, then a
/// section for each top section, in order, with the pages of the sections
/// nested in it folded in, in reading order. A top section's own page comes
/// first in it, and its heading is its label from `ascribe.nav`, else that
/// page's title, else its folder's name. A section with no page offered is
/// left out, and an offered page the navigation doesn't place comes last in
/// the first section.
fn sections<'a>(entries: Vec<Entry<'a>>, navigation: &BuildNavigation<'_>) -> Vec<Section<'a>> {
    let order: Vec<RelPath> = entries.iter().map(|e| e.page.clone()).collect();
    let mut by_page: BTreeMap<RelPath, Entry<'a>> =
        entries.into_iter().map(|e| (e.page.clone(), e)).collect();
    let root = &navigation.root;
    // A top section's own page beside its folder is that section's.
    let beside: Vec<&RelPath> = root
        .sections
        .iter()
        .filter_map(|s| s.page.as_ref())
        .map(|p| &p.page.path)
        .collect();
    let mut start = Section {
        key: None,
        heading: root.label.clone(),
        own: false,
        entries: Vec::new(),
    };
    for page in root.page.iter().chain(&root.pages) {
        if beside.contains(&&page.page.path) {
            continue;
        }
        start.entries.extend(by_page.remove(&page.page.path));
    }
    let mut sections = vec![start];
    for nested in &root.sections {
        let entries: Vec<Entry<'a>> = nested
            .reading_order()
            .into_iter()
            .filter_map(|p| by_page.remove(&p.page.path))
            .collect();
        let Some(first) = entries.first() else {
            continue;
        };
        let own = nested
            .page
            .as_ref()
            .is_some_and(|p| p.page.path == *first.page);
        // The folder's slug: the first segment of a page's entry id in it.
        let key = entries
            .iter()
            .find(|e| e.page.segments().count() > 1)
            .unwrap_or(first)
            .id
            .split('/')
            .next()
            .unwrap_or_default()
            .to_owned();
        let heading = match (&nested.section.label, own) {
            (Some(label), _) => label.clone(),
            (None, true) => first.title.clone(),
            (None, false) => folder_label(nested.folder),
        };
        sections.push(Section {
            key: Some(key),
            heading,
            own,
            entries,
        });
    }
    for page in &order {
        if let Some(entry) = by_page.remove(page) {
            sections[0].entries.push(entry);
        }
    }
    sections.retain(|s| !s.entries.is_empty());
    sections
}

/// What an index of a line of a versioned build says about the line: the
/// words after its name, ` (Self-managed 3.4)`, which the compact index's
/// heading has too ([`compact_index`]), and in an older line, a paragraph
/// pointing at the current line's index, both empty for a build that
/// isn't versioned, or has one live line; and the releases the build
/// describes ([`releases_described`]), which its summary ends with.
#[derive(Default)]
struct LineHeader {
    suffix: String,
    pointer: Option<String>,
    releases: Option<String>,
}

fn line_header(model: &ContentModel, build: &Build) -> LineHeader {
    let releases = releases_described(model, build);
    let (Some(target), Some(line)) = (&build.versions, &build.line) else {
        return LineHeader {
            releases,
            ..LineHeader::default()
        };
    };
    if line.current && line.oldest.text == line.line.text {
        return LineHeader {
            releases,
            ..LineHeader::default()
        };
    }
    let label = model.value_label(target).unwrap_or(target);
    let suffix = format!(" ({label} {})", line.line.text);
    let current = model
        .releases_of(target)
        .and_then(|releases| releases.live.first());
    let pointer = match current {
        Some(current) if !line.current => {
            let url = absolute_url(model, &current_line_router(model, build).url_of("llms.txt"));
            Some(format!(
                "This is an older version. The current version is {}: [llms.txt]({url}).",
                current.text
            ))
        }
        _ => None,
    };
    LineHeader {
        suffix,
        pointer,
        releases,
    }
}

/// The sentences that say which releases `build` describes, for an agent
/// that has to know whether what it reads is in the release its user is on
/// (decision #1028): `This documentation describes Self-managed 3.5 and
/// 3.4.`, for each target `[releases]` lists, by its label. A line of a
/// versioned build describes its line, and a build that filters for a
/// version that version alone, with nothing of another target's; any other
/// build describes each target's live lines, and when a target has a
/// planned line, a second sentence says that what's marked with the planned
/// label is in none of them yet. `None` for a project without `[releases]`,
/// and for a build that filters for a target it lists no lines of.
pub fn releases_described(model: &ContentModel, build: &Build) -> Option<String> {
    let label = |target: &str| model.value_label(target).unwrap_or(target).to_owned();
    let mut described: Vec<(String, Vec<&str>)> = Vec::new();
    let mut planned = false;
    match &build.availability {
        AvailabilityMode::Filter { target, version } => {
            let version = version.as_ref()?;
            model.releases_of(target)?;
            described.push((label(target), vec![version.text.as_str()]));
        }
        AvailabilityMode::Badge => {
            for releases in &model.releases {
                let line = build
                    .line
                    .as_ref()
                    .filter(|_| build.versions.as_deref() == Some(releases.target.as_str()));
                let lines: Vec<&str> = match line {
                    Some(line) => vec![line.line.text.as_str()],
                    None => {
                        planned |= !releases.planned.is_empty() && !releases.live.is_empty();
                        releases.live.iter().map(|v| v.text.as_str()).collect()
                    }
                };
                if !lines.is_empty() {
                    described.push((label(&releases.target), lines));
                }
            }
        }
    }
    if described.is_empty() {
        return None;
    }
    let count: usize = described.iter().map(|(_, lines)| lines.len()).sum();
    let targets: Vec<String> = described
        .iter()
        .map(|(label, lines)| format!("{label} {}", in_words(lines)))
        .collect();
    let mut text = format!(
        "This documentation describes {}.",
        in_words(&targets.iter().map(String::as_str).collect::<Vec<_>>())
    );
    if planned {
        let planned_label = &model.planned_label;
        text.push_str(&if count == 1 {
            format!(" Anything marked {planned_label} isn't in that release yet.")
        } else {
            format!(" Anything marked {planned_label} isn't in any of those releases yet.")
        });
    }
    Some(text)
}

/// A list in words: `a`, `a and b`, `a, b, and c`.
fn in_words(items: &[&str]) -> String {
    match items {
        [] => String::new(),
        [one] => (*one).to_owned(),
        [first, second] => format!("{first} and {second}"),
        [rest @ .., last] => format!("{}, and {last}", rest.join(", ")),
    }
}

/// A whole index: the header, then each section.
fn index(line: &LineHeader, name: &str, summary: &str, sections: &[&Section<'_>]) -> String {
    let mut text = header(line, name, summary);
    for section in sections {
        push_section(&mut text, section);
    }
    text
}

fn header(line: &LineHeader, name: &str, summary: &str) -> String {
    let releases = line
        .releases
        .as_ref()
        .map(|r| format!(" {r}"))
        .unwrap_or_default();
    let mut text = format!(
        "# {}{}\n\n> {}{releases}\n",
        one_line(name),
        line.suffix,
        one_line(summary)
    );
    if let Some(pointer) = &line.pointer {
        text.push('\n');
        text.push_str(pointer);
        text.push('\n');
    }
    text
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
/// ([`description_text`]): its own, or its field's default
/// ([`written_description`]). `None` when it has none.
pub fn description(model: &ContentModel, page: &ResolvedPage) -> Option<String> {
    let field = description_field(model, page.frontmatter.as_ref())?;
    let empty = Value::Mapping(Default::default());
    let value = page.frontmatter.as_ref().unwrap_or(&empty);
    description_text(model, written_description(field, value)?)
}

/// The description a page with the frontmatter `value` has in its field
/// `field`, as written: its own string, or the field's default when it
/// leaves the field out or leaves it empty (decision #573). A field without
/// a default keeps an empty own string. `None` when there's neither, or the
/// value isn't a string. `ascribe_check::roles::description` is this.
pub fn written_description<'v>(field: &'v Field, value: &'v Value) -> Option<&'v str> {
    let own = value.get(field.name.as_str());
    if let Some(text) = own.and_then(own_description) {
        return Some(text);
    }
    match own {
        None | Some(Value::Null | Value::String(_)) => field
            .default
            .as_ref()
            .and_then(Value::as_str)
            .or_else(|| own?.as_str()),
        // Another type is `frontmatter-type-mismatch`.
        Some(_) => None,
    }
}

/// A page's own value for its description field, when it's a string with
/// something in it: the description the page writes itself, rather than
/// leaving to the field's default.
pub fn own_description(own: &Value) -> Option<&str> {
    own.as_str().filter(|s| !s.trim().is_empty())
}

/// A description as its line in an index shows it: on one line, with any
/// Markdown in it (a link, emphasis) as its text, so that it links nowhere.
/// `None` when that leaves nothing.
pub fn description_text(model: &ContentModel, raw: &str) -> Option<String> {
    let text = one_line(&markdown_text(model, raw));
    (!text.is_empty()).then_some(text)
}

/// The description field of a page with this frontmatter: the field its
/// content type marks `role = "description"`.
pub fn description_field<'m>(
    model: &'m ContentModel,
    frontmatter: Option<&serde_yaml_ng::Value>,
) -> Option<&'m Field> {
    let ty = model.type_of(frontmatter).found()?;
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

/// The most bytes a compact index holds ([`compact_index`]): the size of the
/// one first-party evaluation of a docs index in `AGENTS.md`, which found it
/// enough.
pub const COMPACT_INDEX_LIMIT: usize = 8 * 1024;

/// A compact index of a build's docs, for a product's `AGENTS.md`
/// ([`compact_index`]).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CompactIndex {
    /// Its Markdown.
    pub text: String,
    /// The pages it lists, in its order.
    pub listed: Vec<RelPath>,
    /// The offered pages it leaves out to stay under its limit, in the order
    /// they'd have come.
    pub left_out: Vec<RelPath>,
    /// The pages it lists without the description they have, to stay under
    /// its limit.
    pub without_description: Vec<RelPath>,
}

/// A compact index of the pages a build offers, from its page records
/// ([`crate::pages::records`]): the index's name and summary, then a line a
/// page, with its title, its address, and its description, grouped in
/// sections as `llms.txt` is ([`llms_files`]), at most `limit` bytes long.
///
/// A page's address is its Markdown version's, absolute when the content
/// model has a `site`, or the page's own when the build publishes no
/// Markdown versions; with `folder`, it's the Markdown version's path in
/// that folder (`folder/guides/setup.md`), for an index beside a copy of the
/// pages.
///
/// Of a line of a versioned build, its heading names the line as that
/// line's `llms.txt` does. An index longer than `limit` leaves out
/// descriptions first, from the last page back, then pages, from the last
/// back, and ends with a sentence that says what it left out and where every
/// page is. A page that isn't offered isn't in it.
pub fn compact_index(
    model: &ContentModel,
    build: &Build,
    records: &PageRecords,
    navigation: &BuildNavigation<'_>,
    folder: Option<&str>,
    limit: usize,
) -> CompactIndex {
    let router = AstroRouter::from_consumer(&model.consumer);
    let folder = folder.map(|f| f.trim_end_matches('/'));
    let entries = entries(&router, records, |record| match folder {
        Some(folder) => folder_path(folder, &router.markdown_path(&record.source)),
        None => match &record.markdown {
            Some(markdown) => absolute_url(model, markdown),
            None => record
                .canonical
                .clone()
                .unwrap_or_else(|| record.route.clone()),
        },
    });
    let root = entries.iter().find(|e| e.id == "index");
    let (name, summary) = name_and_summary(model, root);
    let sections = sections(entries, navigation);
    let everywhere = match folder {
        Some(folder) => Some(format!("`{}`", folder_path(folder, ""))),
        None => model.consumer.agents.then(|| {
            format!(
                "[llms.txt]({})",
                absolute_url(model, &router.url_of("llms.txt"))
            )
        }),
    };
    let line = line_header(model, build);
    let compact = Compact {
        name: &name,
        line: &line.suffix,
        summary: &summary,
        sections: &sections,
        everywhere: everywhere.as_deref(),
        limit,
    };
    let count: usize = sections.iter().map(|s| s.entries.len()).sum();
    let fits = |pages: usize, described: usize| compact.text(pages, described).len() <= limit;
    let (pages, described) = if fits(count, count) {
        (count, count)
    } else if fits(count, 0) {
        (count, largest(count, |k| fits(count, k)))
    } else {
        (largest(count, |m| fits(m, 0)), 0)
    };
    let mut index = CompactIndex {
        text: compact.text(pages, described),
        listed: Vec::new(),
        left_out: Vec::new(),
        without_description: Vec::new(),
    };
    let all = sections.iter().flat_map(|s| &s.entries).enumerate();
    for (at, entry) in all {
        if at >= pages {
            index.left_out.push(entry.page.clone());
            continue;
        }
        index.listed.push(entry.page.clone());
        if at >= described && entry.description.is_some() {
            index.without_description.push(entry.page.clone());
        }
    }
    index
}

/// The largest `n` below `count` for which `fits` holds, given that it holds
/// for every number below one it holds for; `0` when it holds for none.
fn largest(count: usize, fits: impl Fn(usize) -> bool) -> usize {
    let (mut low, mut high) = (0, count);
    // `fits(low)` is assumed, and `fits(high)` doesn't hold.
    while high - low > 1 {
        let mid = low + (high - low) / 2;
        if fits(mid) {
            low = mid;
        } else {
            high = mid;
        }
    }
    low
}

/// A path in a folder, as an index's address: `folder/path`, or `path` when
/// the folder is the index's own.
fn folder_path(folder: &str, path: &str) -> String {
    match folder {
        "" | "." => path.to_owned(),
        folder => format!("{folder}/{path}"),
    }
}

/// What a compact index is written from.
struct Compact<'a> {
    name: &'a str,
    /// The words after the name that name the line ([`LineHeader`]).
    line: &'a str,
    summary: &'a str,
    sections: &'a [Section<'a>],
    /// Where every page is, in Markdown, for the sentence about the pages
    /// left out.
    everywhere: Option<&'a str>,
    limit: usize,
}

impl Compact<'_> {
    /// The index with the first `pages` pages, the first `described` of them
    /// with their descriptions.
    fn text(&self, pages: usize, described: usize) -> String {
        let mut text = format!(
            "## {}{}\n\n> {}\n",
            one_line(self.name),
            self.line,
            one_line(self.summary)
        );
        let mut at = 0;
        let mut undescribed = 0;
        let mut left_out = 0;
        for section in self.sections {
            for (i, entry) in section.entries.iter().enumerate() {
                if at >= pages {
                    left_out += 1;
                    at += 1;
                    continue;
                }
                if i == 0 {
                    text.push_str(&format!("\n### {}\n\n", one_line(&section.heading)));
                }
                text.push_str(&format!(
                    "- [{}]({})",
                    link_text(&entry.title),
                    destination(&entry.url)
                ));
                match &entry.description {
                    Some(description) if at < described => {
                        text.push_str(": ");
                        text.push_str(description);
                    }
                    Some(_) => undescribed += 1,
                    None => {}
                }
                text.push('\n');
                at += 1;
            }
        }
        let size = size(self.limit);
        if left_out > 0 {
            let descriptions = if undescribed > 0 {
                ", and the descriptions of the rest,"
            } else {
                ""
            };
            text.push_str(&format!(
                "\nThis index leaves out {}{descriptions} to stay under {size}.",
                pages_count(left_out)
            ));
            if let Some(everywhere) = self.everywhere {
                text.push_str(&format!(" {everywhere} has every page."));
            }
            text.push('\n');
        } else if undescribed > 0 {
            text.push_str(&format!(
                "\nThis index leaves out the descriptions of {}, to stay under {size}.\n",
                pages_count(undescribed)
            ));
        }
        text
    }
}

/// A number of pages, in words: `1 page`, `3 pages`.
fn pages_count(n: usize) -> String {
    if n == 1 {
        "1 page".to_owned()
    } else {
        format!("{n} pages")
    }
}

/// A size in bytes, in words: `8 KB`, or `500 bytes`.
fn size(bytes: usize) -> String {
    if bytes >= 1024 && bytes.is_multiple_of(1024) {
        format!("{} KB", bytes / 1024)
    } else {
        format!("{bytes} bytes")
    }
}

/// An address as a link's destination: in angle brackets when it has a
/// space or a parenthesis, which would end it.
fn destination(url: &str) -> String {
    if url.contains([' ', '(', ')', '<', '>']) {
        format!("<{}>", url.replace('<', "%3C").replace('>', "%3E"))
    } else {
        url.to_owned()
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
    fn in_words_joins_a_list_as_a_sentence_does() {
        assert_eq!(in_words(&["a"]), "a");
        assert_eq!(in_words(&["a", "b"]), "a and b");
        assert_eq!(in_words(&["a", "b", "c"]), "a, b, and c");
    }
}
