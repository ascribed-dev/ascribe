//! The archive: one line of a versioned build rendered once more, by
//! Ascribe alone, into a folder of HTML that needs nothing else
//! (`ascribe versions export`, the archive contract in
//! `docs/content/contracts/archive.md`).
//!
//! The archive is a fourth emitter, `archive`, that `ascribe build` never
//! writes. Each page is one complete HTML file, `install.md` as
//! `install.html`: the page rendered as [`render_site_html`] renders the
//! site output, with no script, under a statement of what it is, with a
//! sidebar of the line's navigation, the contents of the page from its
//! headings, and links to the previous and next pages in reading order.
//! Beside the pages are the assets they use, mirrored as the plain output
//! places them, the stylesheet (the element library's, then the archive's
//! own), a contents page, every page in one document for printing
//! (`all.html`), a `robots.txt`, and the search: its script and its index
//! ([`search`]).
//!
//! Every link between pages is relative, so the folder works from a disk
//! and at any address. A link out of the line, and a link to an address
//! under the site that no page of the line has, is written as its text,
//! and listed in [`Archive::unlinked`].
//!
//! [`preview`] renders any build the same way, for `ascribe preview`: the
//! same files, under a statement that the folder is a preview.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;
use std::sync::{Mutex, PoisonError};

use ascribe_core::{RelPath, names};
use ascribe_model::Build;
use ascribe_model::Segment;
use ascribe_resolve::{AstroRouter, Project, ResolvedBuild, ResolvedPage, SharedSection};

use crate::assets::encode_path;
use crate::emitter::{EmitContext, Emitter, PageContext, emit};
use crate::error::EmitError;
use crate::render::{decode, encode, parse_existing, render_site_html, write_attributes};
use crate::site::blocks::{Renderer, availability_element};
use crate::site::frontmatter;
use crate::store::{Contents, EmittedFile, FileKind};

mod search;

pub use search::{LARGE_SEARCH_INDEX, SEARCH_INDEX, SEARCH_SCRIPT};

/// The archive's stylesheet, at the top of its folder.
pub const ARCHIVE_STYLESHEET: &str = "style.css";
/// Every page of the archive in one document, at the top of its folder.
pub const ARCHIVE_ALL: &str = "all.html";
/// The contents page when the line's root page isn't there to be
/// `index.html`.
const INDEX: &str = "index.html";
/// The contents page when the line's root page is `index.html`.
const CONTENTS: &str = "contents.html";
/// What keeps crawlers out of a host that serves the folder at its own
/// origin.
const ROBOTS: &str = "robots.txt";

/// The element library's stylesheet, as `@ascribed/elements` ships it.
const ELEMENTS_CSS: &str = include_str!("../../../../packages/elements/css/style.css");
/// The archive's own: its layout, the statement, and print.
const ARCHIVE_CSS: &str = include_str!("archive.css");

/// What an archive says about where the documentation went.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ArchiveOptions {
    /// The address of the current documentation, which the statement at the
    /// top of every page links to.
    pub current: Option<String>,
}

/// One line of a versioned build, as an archive's files.
#[derive(Debug)]
pub struct Archive {
    /// Every file, sorted by path.
    pub files: Vec<EmittedFile>,
    /// How many pages it has.
    pub pages: usize,
    /// The contents page: `index.html`, or `contents.html` when the line's
    /// root page is `index.html`.
    pub contents: RelPath,
    /// Each link written as its text, by page.
    pub unlinked: Vec<Unlinked>,
    /// The size of the search index, `search-index.js`, in bytes.
    pub search_index: usize,
}

/// A link the archive writes as its text: a link out of the line, or to an
/// address no page of the line has.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Unlinked {
    /// The page it's on, by its source path.
    pub page: RelPath,
    /// Where it led.
    pub href: String,
    /// For a link out of the line, the line it led to.
    pub line: Option<String>,
    /// Its text.
    pub text: String,
}

/// Renders `build`, one line of a versioned build, as an archive: every
/// file, in memory. [`crate::OutputDir::replace_archive`] writes them.
///
/// `project` is the project's source index and `root` its root, the folder
/// of `ascribe.toml`.
///
/// # Errors
///
/// `build` isn't a line of a versioned build, or a page can't be rendered.
pub fn archive(
    project: &Project,
    root: &Path,
    build: &Build,
    options: &ArchiveOptions,
) -> Result<Archive, EmitError> {
    let (Some(line), Some(target)) = (&build.line, build.versions.as_deref()) else {
        return Err(EmitError::Invalid {
            message: format!(
                "build `{}` isn't a line of a versioned build, so it can't be archived",
                build.name
            ),
        });
    };
    let index = project.for_build(build);
    let index = index.as_ref();
    let model = index.model();
    let router = AstroRouter::from_consumer(&model.consumer);
    let resolved = index.resolve_build(build, &router);
    let what = format!(
        "{} {}",
        model.value_label(target).unwrap_or(target),
        line.line.text
    );
    let link = match &options.current {
        Some(url) => format!(
            "<span class=\"archive-current\"> <a href=\"{}\">Read the current documentation.</a></span>",
            encode(url)
        ),
        None => String::new(),
    };
    let statement = Statement {
        html: format!(
            "<header class=\"archive-statement\">\n<p>Documentation for {}. This version is no longer maintained.{link}</p>\n</header>\n",
            encode(&what)
        ),
        what,
        search: None,
    };
    render(index, root, build, &resolved, statement)
}

/// Renders `build`, any build, as an archive renders a line, for `ascribe
/// preview`: every file, in memory, under a statement that it's a preview.
/// [`crate::OutputDir::replace_archive`] writes them.
///
/// A page with errors is rendered as far as it resolves, as the editor's
/// preview renders it; a link to an address no page has is written as its
/// text, as in an archive.
///
/// # Errors
///
/// A page can't be rendered.
pub fn preview(project: &Project, root: &Path, build: &Build) -> Result<Archive, EmitError> {
    let index = project.for_build(build);
    let index = index.as_ref();
    let router = AstroRouter::from_consumer(&index.model().consumer);
    let resolved = index.resolve_build(build, &router);
    let statement = Statement {
        what: format!("{} preview", build.name),
        html: format!(
            "<header class=\"archive-statement\">\n<p>A preview of build <code>{}</code>, written by <code>ascribe preview</code>. The published site has its own look.</p>\n</header>\n",
            encode(&build.name)
        ),
        search: Some(format!("Search build {}", build.name)),
    };
    render(index, root, build, &resolved, statement)
}

/// Renders a resolved build as an archive's files, under `statement`.
fn render(
    index: &Project,
    root: &Path,
    build: &Build,
    resolved: &ResolvedBuild,
    statement: Statement,
) -> Result<Archive, EmitError> {
    let emitter = ArchiveEmitter::new(index, resolved, statement);
    let cx = EmitContext::new(index, root, build);
    let emission = emit(&emitter, &cx, resolved)?;
    let pages = emission
        .files
        .iter()
        .filter(|f| f.kind == FileKind::Page)
        .count();
    let search_index = emission
        .files
        .iter()
        .find(|f| f.path.as_str() == SEARCH_INDEX)
        .map_or(0, |f| match &f.contents {
            Contents::Text(text) => text.len(),
            _ => 0,
        });
    let unlinked = emitter
        .unlinked
        .into_inner()
        .unwrap_or_else(PoisonError::into_inner);
    Ok(Archive {
        files: emission.files,
        pages,
        contents: emitter.nav.contents,
        unlinked: unlinked.into_iter().collect(),
        search_index,
    })
}

/// The file an archive writes a page to, from the archive's folder: its
/// source path with `.html` for its extension.
pub fn page_file(page: &RelPath) -> RelPath {
    let path = page.as_str();
    let stem = match page.extension() {
        Some(extension) => &path[..path.len() - extension.len() - 1],
        None => path,
    };
    RelPath::parse(&format!("{stem}.html")).unwrap_or_else(|_| page.clone())
}

/// The statement at the top of every page.
struct Statement {
    /// What the folder is, after each page's title: the line, as a person
    /// reads it, `self-managed 3.4`.
    what: String,
    /// The statement, as HTML.
    html: String,
    /// What the search box is labeled, when it isn't the script's own
    /// "Search this version".
    search: Option<String>,
}

/// A page in the archive's navigation.
struct NavPage {
    source: RelPath,
    output: RelPath,
    /// Its label as HTML, in the sidebar, the contents page, and previous
    /// and next.
    label: String,
    /// Its title, as HTML.
    title: String,
}

/// A part of the sidebar and the contents page: a page, or a nested
/// section with its own page, when it has one, and its parts.
enum Part {
    Page(usize),
    Section {
        label: String,
        page: Option<usize>,
        parts: Vec<Part>,
    },
}

/// The way around: the line's navigation ([`ascribe_resolve::navigation`]).
struct Nav {
    /// Every page: those in the navigation in reading order, then those it
    /// leaves out, being `unlisted`, in path order.
    pages: Vec<NavPage>,
    /// How many of `pages` are in the navigation, and so have a previous
    /// and a next.
    listed: usize,
    /// The groups: the root's pages, with no label, then each top section,
    /// its label as HTML, and its parts.
    groups: Vec<(Option<String>, Vec<Part>)>,
    /// The contents page.
    contents: RelPath,
}

impl Nav {
    fn new(project: &Project, build: &ResolvedBuild) -> Nav {
        let navigation = project.navigation();
        let share = navigation.share(project, build);
        let mut pages: Vec<NavPage> = share
            .root
            .reading_order()
            .into_iter()
            .map(|p| NavPage {
                source: p.page.path.clone(),
                output: page_file(&p.page.path),
                label: label_html(p.label.as_str(), p.formatted),
                title: title_html(p.page),
            })
            .collect();
        let listed = pages.len();
        let at: BTreeMap<RelPath, usize> = pages
            .iter()
            .enumerate()
            .map(|(i, p)| (p.source.clone(), i))
            .collect();
        for page in &build.pages {
            if !at.contains_key(&page.path) {
                pages.push(NavPage {
                    source: page.path.clone(),
                    output: page_file(&page.path),
                    label: title_html(page),
                    title: title_html(page),
                });
            }
        }
        let index = |p: &ascribe_resolve::NavPage<'_>| at.get(&p.page.path).copied();
        let root = &share.root;
        // A top section's own page beside its folder is that section's.
        let beside: Vec<&RelPath> = root
            .sections
            .iter()
            .filter_map(|s| s.page.as_ref())
            .map(|p| &p.page.path)
            .collect();
        let mut groups = vec![(
            None,
            root.page
                .iter()
                .chain(&root.pages)
                .filter(|p| !beside.contains(&&p.page.path))
                .filter_map(|p| index(p).map(Part::Page))
                .collect::<Vec<_>>(),
        )];
        for section in &root.sections {
            groups.push((
                Some(label_html(&section.label, section.formatted)),
                parts(section, true, &index),
            ));
        }
        groups.retain(|(_, parts)| !parts.is_empty());
        let root_page = pages.iter().any(|p| p.output.as_str() == INDEX);
        let contents = RelPath::parse(if root_page { CONTENTS } else { INDEX })
            .unwrap_or_else(|_| RelPath::root());
        Nav {
            pages,
            listed,
            groups,
            contents,
        }
    }
}

/// A section's parts: its own page, with `own`, then its pages, each nested
/// section whose own page is beside its folder in that page's place, and
/// the other nested sections after them.
fn parts(
    section: &SharedSection<'_>,
    own: bool,
    index: &dyn Fn(&ascribe_resolve::NavPage<'_>) -> Option<usize>,
) -> Vec<Part> {
    let mut out = Vec::new();
    if own && let Some(page) = section.page.as_ref().and_then(index) {
        out.push(Part::Page(page));
    }
    let mut placed = vec![false; section.sections.len()];
    for page in &section.pages {
        let nested = section.sections.iter().enumerate().find(|(at, n)| {
            !placed[*at]
                && n.page
                    .as_ref()
                    .is_some_and(|p| p.page.path == page.page.path)
        });
        match nested {
            Some((at, nested)) => {
                placed[at] = true;
                out.push(nested_part(nested, index));
            }
            None => out.extend(index(page).map(Part::Page)),
        }
    }
    for (nested, placed) in section.sections.iter().zip(placed) {
        if !placed {
            out.push(nested_part(nested, index));
        }
    }
    out
}

fn nested_part(
    section: &SharedSection<'_>,
    index: &dyn Fn(&ascribe_resolve::NavPage<'_>) -> Option<usize>,
) -> Part {
    Part::Section {
        label: label_html(&section.label, section.formatted),
        page: section.page.as_ref().and_then(index),
        parts: parts(section, false, index),
    }
}

/// A label as HTML: its formatted form, or its text.
fn label_html(label: &str, formatted: Option<&[Segment]>) -> String {
    match formatted {
        Some(segments) => frontmatter::html(segments),
        None => encode(label),
    }
}

/// A page's title as HTML: its formatted title, its title, or its path.
fn title_html(page: &ResolvedPage) -> String {
    match (page.formatted_title(), &page.title) {
        (Some(segments), _) => frontmatter::html(segments),
        (None, Some(title)) => encode(title),
        (None, None) => encode(page.path.as_str()),
    }
}

/// A page's title as text, for its `<title>`.
fn title_text(page: &ResolvedPage) -> String {
    page.title.clone().unwrap_or_else(|| page.path.to_string())
}

/// The archive emitter.
struct ArchiveEmitter {
    nav: Nav,
    /// Each page's route, without its trailing `/`, and the file it's
    /// written to.
    routes: BTreeMap<String, RelPath>,
    statement: Statement,
    /// The links written as their text, gathered as pages are rendered.
    unlinked: Mutex<BTreeSet<Unlinked>>,
}

/// What becomes of a link.
enum LinkAction {
    /// It stays as it is.
    Keep,
    /// It leads to this address in the archive.
    Href(String),
    /// It's written as its text: a link out of the line, with the line, or
    /// to an address no page has.
    Unwrap(Option<String>, String),
}

impl ArchiveEmitter {
    fn new(project: &Project, build: &ResolvedBuild, statement: Statement) -> ArchiveEmitter {
        let routes = build
            .pages
            .iter()
            .map(|p| (p.route.trim_end_matches('/').to_owned(), page_file(&p.path)))
            .collect();
        ArchiveEmitter {
            nav: Nav::new(project, build),
            routes,
            statement,
            unlinked: Mutex::new(BTreeSet::new()),
        }
    }

    /// The page's content as HTML, its links rewritten for the archive, as
    /// the page written at `cx.output` has them.
    fn body(&self, cx: &PageContext<'_>, page: &ResolvedPage) -> String {
        let renderer = Renderer {
            page: cx,
            model: cx.emit.model,
            anchors: false,
        };
        let markdown = renderer.blocks(&page.blocks).join("\n\n");
        self.links(&render_site_html(&markdown), &page.path, &cx.output)
    }

    /// `html` with each link to a page of the line made relative to
    /// `output`'s folder, and each link out of the line, or to an address
    /// under the site that no page has, written as its text.
    fn links(&self, html: &str, page: &RelPath, output: &RelPath) -> String {
        let dir = output.parent().unwrap_or_else(RelPath::root);
        let mut out = String::with_capacity(html.len());
        let mut at = 0;
        while let Some(found) = html[at..].find("<a ") {
            let start = at + found;
            out.push_str(&html[at..start]);
            // comrak escapes `>` in every attribute value, so the tag ends
            // at the first.
            let Some(end) = html[start..].find('>').map(|e| start + e) else {
                at = start;
                break;
            };
            let attributes = parse_existing(&html[start + "<a".len()..end]);
            let action = attributes
                .as_ref()
                .map_or(LinkAction::Keep, |a| self.link_action(a, &dir));
            match (action, attributes) {
                (LinkAction::Href(href), Some(mut attributes)) => {
                    if let Some(slot) = attributes.iter_mut().find(|(n, _)| n == "href") {
                        slot.1 = href;
                    }
                    out.push_str(&format!("<a{}>", write_attributes(&attributes, &[])));
                    at = end + 1;
                }
                (LinkAction::Unwrap(line, href), _) => {
                    // A link holds no link, so it ends at the next `</a>`.
                    let inner_end = html[end + 1..]
                        .find("</a>")
                        .map_or(html.len(), |e| end + 1 + e);
                    let inner = &html[end + 1..inner_end];
                    out.push_str(inner);
                    self.unlinked
                        .lock()
                        .unwrap_or_else(PoisonError::into_inner)
                        .insert(Unlinked {
                            page: page.clone(),
                            href,
                            line,
                            text: text_of(inner),
                        });
                    at = (inner_end + "</a>".len()).min(html.len());
                }
                _ => {
                    out.push_str(&html[start..=end]);
                    at = end + 1;
                }
            }
        }
        out.push_str(&html[at..]);
        out
    }

    fn link_action(&self, attributes: &[(String, String)], dir: &RelPath) -> LinkAction {
        let get = |name: &str| {
            attributes
                .iter()
                .find(|(n, _)| n == name)
                .map(|(_, v)| v.clone())
        };
        let Some(href) = get("href") else {
            return LinkAction::Keep;
        };
        if let Some(line) = get(names::DATA_RELEASE_LINE) {
            return LinkAction::Unwrap(Some(line), href);
        }
        if !href.starts_with('/') || href.starts_with("//") {
            return LinkAction::Keep;
        }
        let (path, fragment) = match href.find('#') {
            Some(at) => (&href[..at], &href[at..]),
            None => (href.as_str(), ""),
        };
        let path = path.split('?').next().unwrap_or_default();
        match self.routes.get(path.trim_end_matches('/')) {
            Some(file) => LinkAction::Href(format!("{}{fragment}", relative(dir, file))),
            None => LinkAction::Unwrap(None, href),
        }
    }

    /// The whole HTML file of a page: its head, the statement, the sidebar,
    /// and the page, with the previous and next pages.
    fn page_html(&self, cx: &PageContext<'_>, page: &ResolvedPage, body: &str) -> String {
        let output = &cx.output;
        let dir = output.parent().unwrap_or_else(RelPath::root);
        let at = self.nav.pages.iter().position(|p| p.source == page.path);
        let mut main = format!("<h1>{}</h1>\n", title_html(page));
        if let Some(availability) = &page.availability {
            main.push_str(&availability_element(
                cx.emit.model,
                "page",
                &availability.spec.entries,
            ));
            main.push('\n');
        }
        main.push_str(&on_this_page(page));
        main.push_str(body);
        main.push_str(&self.pager(at, &dir));
        let title = format!("{} · {}", title_text(page), self.statement.what);
        self.document(&title, &dir, Some(output), &main)
    }

    /// The previous and next pages, from the page at `at` in reading order.
    fn pager(&self, at: Option<usize>, dir: &RelPath) -> String {
        let Some(at) = at.filter(|at| *at < self.nav.listed) else {
            return String::new();
        };
        let mut links = Vec::new();
        if let Some(previous) = at.checked_sub(1).and_then(|i| self.nav.pages.get(i)) {
            links.push(format!(
                "<a rel=\"prev\" href=\"{}\">Previous: {}</a>",
                encode(&relative(dir, &previous.output)),
                previous.label
            ));
        }
        if let Some(next) = self
            .nav
            .pages
            .get(at + 1)
            .filter(|_| at + 1 < self.nav.listed)
        {
            links.push(format!(
                "<a rel=\"next\" href=\"{}\">Next: {}</a>",
                encode(&relative(dir, &next.output)),
                next.label
            ));
        }
        if links.is_empty() {
            return String::new();
        }
        format!(
            "<nav class=\"archive-pager\" aria-label=\"Previous and next\">\n{}\n</nav>\n",
            links.join("\n")
        )
    }

    /// An HTML document of the archive: its head, the statement, the
    /// sidebar with `current` marked (none in `all.html`, which has no
    /// sidebar), and `main`.
    fn document(
        &self,
        title: &str,
        dir: &RelPath,
        current: Option<&RelPath>,
        main: &str,
    ) -> String {
        let stylesheet = RelPath::parse(ARCHIVE_STYLESHEET).unwrap_or_else(|_| RelPath::root());
        let mut out = String::new();
        out.push_str("<!doctype html>\n<html>\n<head>\n<meta charset=\"utf-8\">\n");
        out.push_str("<meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\n");
        out.push_str("<meta name=\"robots\" content=\"noindex\">\n");
        // The archive's own scripts, from the folder: `'self'` where it's
        // served, and `file:` opened from a disk, which `'self'` doesn't
        // match in every browser (decision 76).
        out.push_str(
            "<meta http-equiv=\"Content-Security-Policy\" content=\"script-src 'self' file:\">\n",
        );
        out.push_str(&format!("<title>{}</title>\n", encode(title)));
        out.push_str(&format!(
            "<link rel=\"stylesheet\" href=\"{}\">\n</head>\n<body>\n",
            encode(&relative(dir, &stylesheet))
        ));
        out.push_str(&self.statement.html);
        match current {
            Some(current) => {
                out.push_str("<div class=\"archive-layout\">\n");
                out.push_str(&self.sidebar(dir, current));
                out.push_str(&format!("<main class=\"archive-page\">\n{main}</main>\n"));
                out.push_str("</div>\n");
            }
            None => out.push_str(&format!("<main class=\"archive-all\">\n{main}</main>\n")),
        }
        out.push_str("</body>\n</html>\n");
        out
    }

    /// The sidebar: the contents page, then each folder's pages, with the
    /// page at `current` marked.
    fn sidebar(&self, dir: &RelPath, current: &RelPath) -> String {
        let mark = |output: &RelPath| {
            if output == current {
                " aria-current=\"page\""
            } else {
                ""
            }
        };
        let mut out = String::from("<nav class=\"archive-sidebar\" aria-label=\"Pages\">\n");
        // The search box, which the script draws in its place: with
        // scripts off, there's none.
        let file = |name: &str| RelPath::parse(name).unwrap_or_else(|_| RelPath::root());
        let label = self
            .statement
            .search
            .as_ref()
            .map(|label| format!(" data-label=\"{}\"", encode(label)))
            .unwrap_or_default();
        out.push_str(&format!(
            "<script src=\"{}\" data-index=\"{}\" data-root=\"{}\"{label} defer></script>\n",
            encode(&relative(dir, &file(SEARCH_SCRIPT))),
            encode(&relative(dir, &file(SEARCH_INDEX))),
            encode(&root_of(dir)),
        ));
        out.push_str(&format!(
            "<p><a href=\"{}\"{}>Contents</a></p>\n",
            encode(&relative(dir, &self.nav.contents)),
            mark(&self.nav.contents)
        ));
        for (name, parts) in &self.nav.groups {
            if let Some(name) = name {
                out.push_str(&format!("<p class=\"archive-folder\">{name}</p>\n"));
            }
            out.push_str(&self.list(parts, dir, Some(current)));
        }
        out.push_str("</nav>\n");
        out
    }

    /// `parts` as a list, each link relative to `dir`, nested sections as
    /// lists in their place, with the page at `current` marked.
    fn list(&self, parts: &[Part], dir: &RelPath, current: Option<&RelPath>) -> String {
        let link = |at: usize, text: Option<&str>| {
            self.nav.pages.get(at).map(|page| {
                let mark = if current == Some(&page.output) {
                    " aria-current=\"page\""
                } else {
                    ""
                };
                format!(
                    "<a href=\"{}\"{mark}>{}</a>",
                    encode(&relative(dir, &page.output)),
                    text.unwrap_or(&page.label)
                )
            })
        };
        let mut out = String::from("<ul>\n");
        for part in parts {
            match part {
                Part::Page(at) => {
                    if let Some(link) = link(*at, None) {
                        out.push_str(&format!("<li>{link}</li>\n"));
                    }
                }
                Part::Section { label, page, parts } => {
                    let head = page
                        .and_then(|at| link(at, Some(label)))
                        .unwrap_or_else(|| format!("<span>{label}</span>"));
                    out.push_str(&format!(
                        "<li>{head}\n{}</li>\n",
                        self.list(parts, dir, current)
                    ));
                }
            }
        }
        out.push_str("</ul>\n");
        out
    }

    /// The contents page: every page in the navigation, by section.
    fn contents_html(&self) -> String {
        let dir = self.nav.contents.parent().unwrap_or_else(RelPath::root);
        let mut main = String::from("<h1>Contents</h1>\n");
        for (name, parts) in &self.nav.groups {
            if let Some(name) = name {
                main.push_str(&format!("<h2>{name}</h2>\n"));
            }
            main.push_str(&self.list(parts, &dir, None));
        }
        let all = RelPath::parse(ARCHIVE_ALL).unwrap_or_else(|_| RelPath::root());
        main.push_str(&format!(
            "<p class=\"archive-all-link\"><a href=\"{}\">Every page in one document</a>, to print or save as one PDF.</p>\n",
            encode(&relative(&dir, &all))
        ));
        let title = format!("Contents · {}", self.statement.what);
        self.document(&title, &dir, Some(&self.nav.contents), &main)
    }

    /// `all.html`: every page in reading order, in one document, each
    /// page's links made relative to the top of the folder.
    fn all_html(&self, cx: &EmitContext<'_>, build: &ResolvedBuild) -> String {
        let by_path: BTreeMap<&RelPath, &ResolvedPage> =
            build.pages.iter().map(|p| (&p.path, p)).collect();
        let mut main = String::new();
        for nav in &self.nav.pages {
            let Some(page) = by_path.get(&nav.source) else {
                continue;
            };
            let page_cx = PageContext::new(cx, nav.output.clone(), self);
            let body = self.body(&page_cx, page);
            let dir = nav.output.parent().unwrap_or_else(RelPath::root);
            main.push_str("<article>\n");
            main.push_str(&format!("<h1>{}</h1>\n", nav.title));
            if let Some(availability) = &page.availability {
                main.push_str(&availability_element(
                    cx.model,
                    "page",
                    &availability.spec.entries,
                ));
                main.push('\n');
            }
            main.push_str(&rebase(&body, &dir));
            main.push_str("</article>\n");
        }
        let title = format!("Every page · {}", self.statement.what);
        self.document(&title, &RelPath::root(), None, &main)
    }
}

impl ArchiveEmitter {
    /// `search-index.js`: every page's text by heading, in reading order.
    fn search_index(&self, build: &ResolvedBuild) -> String {
        let by_path: BTreeMap<&RelPath, &ResolvedPage> =
            build.pages.iter().map(|p| (&p.path, p)).collect();
        let pages: Vec<search::IndexPage> = self
            .nav
            .pages
            .iter()
            .filter_map(|nav| {
                let page = by_path.get(&nav.source)?;
                Some(search::index_page(page, &nav.output, title_text(page)))
            })
            .collect();
        search::index_script(&pages)
    }
}

impl Emitter for ArchiveEmitter {
    fn name(&self) -> &'static str {
        "archive"
    }

    fn page_path(&self, page: &RelPath) -> RelPath {
        page_file(page)
    }

    fn render_page(&self, cx: &PageContext<'_>, page: &ResolvedPage) -> Result<String, EmitError> {
        let body = self.body(cx, page);
        Ok(self.page_html(cx, page, &body))
    }

    fn generated(
        &self,
        cx: &EmitContext<'_>,
        build: &ResolvedBuild,
    ) -> Result<Vec<EmittedFile>, EmitError> {
        let generated = |path: &str, text: String| -> Result<EmittedFile, EmitError> {
            Ok(EmittedFile {
                path: RelPath::parse(path).map_err(|e| EmitError::Invalid {
                    message: format!("bad path for a generated file: {e}"),
                })?,
                kind: FileKind::Generated,
                source: None,
                url: None,
                contents: Contents::Text(text),
            })
        };
        Ok(vec![
            generated(ARCHIVE_STYLESHEET, stylesheet())?,
            generated(ROBOTS, "User-agent: *\nDisallow: /\n".to_owned())?,
            generated(self.nav.contents.as_str(), self.contents_html())?,
            generated(ARCHIVE_ALL, self.all_html(cx, build))?,
            generated(SEARCH_SCRIPT, search::SCRIPT.to_owned())?,
            generated(SEARCH_INDEX, self.search_index(build))?,
        ])
    }
}

/// The archive's stylesheet: the element library's, then the archive's
/// own.
fn stylesheet() -> String {
    format!(
        "{}\n\n{}\n",
        ELEMENTS_CSS.trim_end(),
        ARCHIVE_CSS.trim_end()
    )
}

/// The address of `file` from the folder `dir`, both in the archive.
fn relative(dir: &RelPath, file: &RelPath) -> String {
    encode_path(
        &file
            .relative_from(dir)
            .unwrap_or_else(|| format!("./{file}")),
    )
}

/// The address of the top of the folder from the folder `dir`: `./`, or
/// `../` for each folder down.
fn root_of(dir: &RelPath) -> String {
    let depth = dir.segments().count();
    if depth == 0 {
        "./".to_owned()
    } else {
        "../".repeat(depth)
    }
}

/// `html` written for a page in the folder `dir`, with each relative
/// address in an `href` or `src` (one starting `./` or `../`, as the
/// archive writes them) made relative to the top of the folder.
fn rebase(html: &str, dir: &RelPath) -> String {
    if dir.is_root() {
        return html.to_owned();
    }
    let mut out = String::with_capacity(html.len());
    let mut at = 0;
    while let Some(found) = html[at..].find("=\"") {
        let value_start = at + found + 2;
        let name = &html[..at + found];
        let is_url = name.ends_with(" href") || name.ends_with(" src");
        out.push_str(&html[at..value_start]);
        at = value_start;
        let Some(len) = html[value_start..].find('"') else {
            break;
        };
        let value = &html[value_start..value_start + len];
        if is_url && (value.starts_with("./") || value.starts_with("../")) {
            let decoded = decode(value);
            let (path, fragment) = match decoded.find('#') {
                Some(i) => (&decoded[..i], &decoded[i..]),
                None => (decoded.as_str(), ""),
            };
            match dir.join(path) {
                Ok(joined) if joined.is_inside() => {
                    out.push_str(&encode(&format!("./{joined}{fragment}")));
                }
                _ => out.push_str(value),
            }
        } else {
            out.push_str(value);
        }
        at = value_start + len;
    }
    out.push_str(&html[at..]);
    out
}

/// The contents of a page from its headings, levels 2 and 3, linking to
/// each: nothing for a page with none.
fn on_this_page(page: &ResolvedPage) -> String {
    let items: Vec<String> = page
        .headings()
        .into_iter()
        .filter(|(_, ids)| (2..=3).contains(&ids.level))
        .map(|(_, ids)| {
            format!(
                "<li class=\"archive-level-{}\"><a href=\"#{}\">{}</a></li>",
                ids.level,
                encode(&ids.page_id),
                encode(&ids.text)
            )
        })
        .collect();
    if items.is_empty() {
        return String::new();
    }
    format!(
        "<nav class=\"archive-on-this-page\" aria-label=\"On this page\">\n<ul>\n{}\n</ul>\n</nav>\n",
        items.join("\n")
    )
}

/// The text of a piece of HTML: its tags removed and its entities decoded.
fn text_of(html: &str) -> String {
    let mut out = String::new();
    let mut inside = false;
    for ch in html.chars() {
        match ch {
            '<' => inside = true,
            '>' => inside = false,
            _ if !inside => out.push(ch),
            _ => {}
        }
    }
    decode(&out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_page_is_written_as_html_beside_its_source() {
        let file = |p: &str| page_file(&RelPath::parse(p).unwrap()).to_string();
        assert_eq!(file("install.md"), "install.html");
        assert_eq!(file("guide/index.md"), "guide/index.html");
        assert_eq!(file("v1.2/notes.md"), "v1.2/notes.html");
    }

    #[test]
    fn rebasing_makes_addresses_relative_to_the_top() {
        let dir = RelPath::parse("guide/deep").unwrap();
        assert_eq!(
            rebase(
                "<a href=\"../index.html#x\">a</a><img src=\"./a.png\" alt=\"./b\" /><a href=\"https://x/\">c</a>",
                &dir
            ),
            "<a href=\"./guide/index.html#x\">a</a><img src=\"./guide/deep/a.png\" alt=\"./b\" /><a href=\"https://x/\">c</a>"
        );
    }

    #[test]
    fn text_of_html_drops_tags() {
        assert_eq!(text_of("<em>a</em> &amp; b"), "a & b");
    }
}
