//! The navigation: a project's sections and pages in order, which every
//! sidebar is drawn from (SPEC §7.8).
//!
//! [`Navigation::of`] makes the project's, once, from the source index and
//! `ascribe.nav` when the project has one that reads:
//!
//! 1. **The sections:** the content root and every folder under it with a
//!    page, at any depth. Fragments, and so folders whose names start with
//!    `_`, are left out. A section's own page is its `index.md`, or else the
//!    page beside the folder with its name (`checks/diagnostics.md` for
//!    `checks/diagnostics/`).
//! 2. **The default order:** sections in path order, the root first; in
//!    each, its own page, then its pages by type in `ascribe.toml`'s order,
//!    then by title. A nested section whose own page is beside its folder
//!    sits where that page sits among its parent's pages; one with no page
//!    beside it comes after them, in path order.
//! 3. **The file's order over it:** the sections the file lists, in its
//!    order, then the rest in path order; in a section that lists `pages`,
//!    those pages in its order, then the rest by title. A name that's no
//!    page of the folder, or a folder with no pages, is passed over.
//!
//! [`Navigation::share`] is a build's share of it: the pages the build
//! publishes and offers (not `unlisted`), and the sections left with one,
//! each labeled as the build has its pages: a section's `label` from the
//! file, else its own page's title, else its folder's name with its first
//! letter capitalized, and "Start" for the root.

use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};

use ascribe_core::RelPath;
use ascribe_model::{ContentModel, NavFile, Segment};

use crate::build::{NAV_LABEL, ResolvedBuild, ResolvedPage};
use crate::index::{FileIndex, FileKind};
use crate::project::Project;

/// The root section's label, unless the file gives it one.
pub const ROOT_LABEL: &str = "Start";

/// A project's navigation, the same for every build.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Navigation {
    /// The content root's section.
    pub root: Section,
}

/// A section: a folder and its pages, in order.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Section {
    /// The folder, from the content root, ending in `/`; `""` for the root.
    pub folder: String,
    /// Its label from `ascribe.nav`, when the file gives it one.
    pub label: Option<String>,
    /// Its own page: its `index.md`, or the page beside the folder with its
    /// name, which is then one of the parent's [`Section::pages`] too.
    pub page: Option<RelPath>,
    /// Its pages in order, its own page not among them. The page beside a
    /// nested folder is one of them, and the nested section sits right
    /// after it.
    pub pages: Vec<RelPath>,
    /// Its nested sections in order: each one whose own page is among
    /// [`Section::pages`] in that page's order, then the rest.
    pub sections: Vec<Section>,
    /// Whether `ascribe.nav` lists it.
    pub listed: bool,
}

impl Section {
    /// Whether the section's own page is beside its folder, and so placed
    /// among its parent's pages.
    pub fn page_beside(&self) -> bool {
        self.page
            .as_ref()
            .is_some_and(|page| page.as_str() != format!("{}index.md", self.folder))
    }
}

/// A build's share of the navigation.
#[derive(Clone, Debug)]
pub struct BuildNavigation<'a> {
    /// The content root's section, which a build with no pages has empty.
    pub root: SharedSection<'a>,
}

/// A section as one build has it.
#[derive(Clone, Debug)]
pub struct SharedSection<'a> {
    /// The section of the project's navigation it's the share of.
    pub section: &'a Section,
    /// The folder, from the content root, ending in `/`; `""` for the root.
    pub folder: &'a str,
    /// Its label as plain text.
    pub label: String,
    /// Its label in pieces, when it's its own page's title and that's read
    /// with inline markup.
    pub formatted: Option<&'a [Segment]>,
    /// Its own page, when the build publishes and offers it.
    pub page: Option<NavPage<'a>>,
    /// Its pages in order.
    pub pages: Vec<NavPage<'a>>,
    /// Its nested sections that keep a page, in order.
    pub sections: Vec<SharedSection<'a>>,
}

impl<'a> SharedSection<'a> {
    /// Its pages in reading order, depth first: its own page, then its
    /// pages, each nested section whose own page is beside its folder right
    /// after that page, with its pages and nested sections, and the other
    /// nested sections after them, each from its own page.
    pub fn reading_order(&self) -> Vec<&NavPage<'a>> {
        let mut out = Vec::new();
        self.read(true, &mut out);
        out
    }

    fn read<'s>(&'s self, own: bool, out: &mut Vec<&'s NavPage<'a>>) {
        if own && let Some(page) = &self.page {
            out.push(page);
        }
        let mut read = vec![false; self.sections.len()];
        for page in &self.pages {
            out.push(page);
            for (at, nested) in self.sections.iter().enumerate() {
                if !read[at]
                    && nested
                        .page
                        .as_ref()
                        .is_some_and(|p| p.page.path == page.page.path)
                {
                    read[at] = true;
                    nested.read(false, out);
                }
            }
        }
        for (nested, read) in self.sections.iter().zip(read) {
            if !read {
                nested.read(true, out);
            }
        }
    }
}

/// A page as a build's navigation has it.
#[derive(Clone, Debug)]
pub struct NavPage<'a> {
    /// The page.
    pub page: &'a ResolvedPage,
    /// Its label as plain text: its `nav-label`, else its title, else its
    /// file name without `.md`.
    pub label: String,
    /// Its label in pieces, when it's read with inline markup, as the title
    /// is.
    pub formatted: Option<&'a [Segment]>,
    /// Its content type's name, when it has one.
    pub type_name: Option<&'a str>,
}

impl Navigation {
    /// The navigation of `project`, in the order `file` gives it over the
    /// default, or the default without one.
    pub fn of(project: &Project, file: Option<&NavFile>) -> Navigation {
        let model = project.model();
        let pages: Vec<&FileIndex> = project
            .files()
            .filter(|f| f.kind == FileKind::Page)
            .collect();
        let by_path: BTreeMap<&str, &FileIndex> =
            pages.iter().map(|f| (f.path.as_str(), *f)).collect();
        // Every folder with a page at any depth, and the root.
        let mut folders: BTreeSet<String> = BTreeSet::new();
        folders.insert(String::new());
        for page in &pages {
            let mut folder = String::new();
            let path = page.path.as_str();
            for segment in path.split('/').take(path.split('/').count() - 1) {
                folder.push_str(segment);
                folder.push('/');
                folders.insert(folder.clone());
            }
        }
        let maker = Maker {
            model,
            file,
            by_path,
            folders,
        };
        Navigation {
            root: maker.section(""),
        }
    }

    /// The share of `build`: the pages it publishes and offers, and the
    /// sections left with one. `project` is the one the navigation was made
    /// from, for a section's own page the build doesn't publish.
    pub fn share<'a>(
        &'a self,
        project: &'a Project,
        build: &'a ResolvedBuild,
    ) -> BuildNavigation<'a> {
        self.share_of(project, &build.pages)
    }

    /// [`Navigation::share`] of a build that publishes `pages`.
    pub fn share_of<'a>(
        &'a self,
        project: &'a Project,
        pages: &'a [ResolvedPage],
    ) -> BuildNavigation<'a> {
        let published: BTreeMap<&RelPath, &ResolvedPage> =
            pages.iter().map(|p| (&p.path, p)).collect();
        let sharer = Sharer { project, published };
        BuildNavigation {
            root: sharer
                .section(&self.root, true)
                .unwrap_or_else(|| SharedSection {
                    section: &self.root,
                    folder: "",
                    label: self
                        .root
                        .label
                        .clone()
                        .unwrap_or_else(|| ROOT_LABEL.to_owned()),
                    formatted: None,
                    page: None,
                    pages: Vec::new(),
                    sections: Vec::new(),
                }),
        }
    }
}

struct Maker<'a> {
    model: &'a ContentModel,
    file: Option<&'a NavFile>,
    by_path: BTreeMap<&'a str, &'a FileIndex>,
    folders: BTreeSet<String>,
}

impl Maker<'_> {
    /// The section of `folder`, with its nested sections.
    fn section(&self, folder: &str) -> Section {
        let entry = self.file.and_then(|f| f.section(folder));
        let own_index = format!("{folder}index.md");
        let page = if self.by_path.contains_key(own_index.as_str()) {
            Some(own_index.clone())
        } else {
            beside(folder).filter(|p| self.by_path.contains_key(p.as_str()))
        };
        // The pages directly in the folder, but its `index.md`.
        let mut pages: Vec<&FileIndex> = self
            .by_path
            .values()
            .filter(|f| parent(f.path.as_str()) == folder && f.path.as_str() != own_index)
            .copied()
            .collect();
        pages.sort_by(|a, b| self.default_order(a, b));
        if let Some(names) = entry.and_then(|e| e.pages.as_ref()) {
            let mut listed: Vec<&FileIndex> = Vec::new();
            for (name, _) in names {
                let path = format!("{folder}{name}");
                if let Some(at) = pages.iter().position(|f| f.path.as_str() == path) {
                    listed.push(pages.remove(at));
                }
            }
            pages.sort_by(|a, b| by_title(a, b));
            listed.extend(pages);
            pages = listed;
        }
        let pages: Vec<RelPath> = pages.into_iter().map(|f| f.path.clone()).collect();
        let mut nested: Vec<Section> = self
            .children(folder)
            .into_iter()
            .map(|child| self.section(&child))
            .collect();
        // Listed sections in the file's order, then the rest in path order
        // (already so); then each one placed by its page beside the folder
        // moves to where that page is.
        if let Some(file) = self.file {
            nested.sort_by_key(|s| file.position(&s.folder).unwrap_or(usize::MAX));
        }
        let (mut placed, rest): (Vec<Section>, Vec<Section>) = nested
            .into_iter()
            .partition(|s| s.page_beside() && s.page.as_ref().is_some_and(|p| pages.contains(p)));
        placed.sort_by_key(|s| {
            s.page
                .as_ref()
                .and_then(|p| pages.iter().position(|q| q == p))
        });
        placed.extend(rest);
        Section {
            folder: folder.to_owned(),
            label: entry.and_then(|e| e.label.clone()),
            page: page.and_then(|p| RelPath::parse(&p).ok()),
            pages,
            sections: placed,
            listed: entry.is_some(),
        }
    }

    /// The sections directly under `folder`, in path order.
    fn children(&self, folder: &str) -> Vec<String> {
        self.folders
            .iter()
            .filter(|f| !f.is_empty() && f.as_str() != folder)
            .filter(|f| parent(f.trim_end_matches('/')) == folder && f.starts_with(folder))
            .cloned()
            .collect()
    }

    /// By type in `ascribe.toml`'s order, a page with none last, then by
    /// title.
    fn default_order(&self, a: &FileIndex, b: &FileIndex) -> Ordering {
        let rank = |f: &FileIndex| {
            f.content_type(self.model)
                .and_then(|t| self.model.types.iter().position(|u| u.name == t.name))
                .unwrap_or(usize::MAX)
        };
        rank(a).cmp(&rank(b)).then_with(|| by_title(a, b))
    }
}

/// By title without case, then as written, a page with no title by its file
/// name, then by path.
fn by_title(a: &FileIndex, b: &FileIndex) -> Ordering {
    let title = |f: &FileIndex| f.title.clone().unwrap_or_else(|| stem(&f.path).to_owned());
    let (ta, tb) = (title(a), title(b));
    ta.to_lowercase()
        .cmp(&tb.to_lowercase())
        .then_with(|| ta.cmp(&tb))
        .then_with(|| a.path.cmp(&b.path))
}

struct Sharer<'a> {
    project: &'a Project,
    published: BTreeMap<&'a RelPath, &'a ResolvedPage>,
}

impl<'a> Sharer<'a> {
    /// `section` as the build has it; `None` when it keeps no page, unless
    /// it's the root.
    fn section(&self, section: &'a Section, root: bool) -> Option<SharedSection<'a>> {
        let page = section.page.as_ref().and_then(|p| self.page(p));
        let pages: Vec<NavPage<'a>> = section.pages.iter().filter_map(|p| self.page(p)).collect();
        let sections: Vec<SharedSection<'a>> = section
            .sections
            .iter()
            .filter_map(|s| self.section(s, false))
            .collect();
        if !root && page.is_none() && pages.is_empty() && sections.is_empty() {
            return None;
        }
        let (label, formatted) = match (&section.label, root) {
            (Some(label), _) => (label.clone(), None),
            (None, true) => (ROOT_LABEL.to_owned(), None),
            (None, false) => self.section_label(section),
        };
        Some(SharedSection {
            section,
            folder: &section.folder,
            label,
            formatted,
            page,
            pages,
            sections,
        })
    }

    /// A section's label when the file gives none: its own page's title, as
    /// the build has it or else as written, or its folder's name with its
    /// first letter capitalized.
    fn section_label(&self, section: &'a Section) -> (String, Option<&'a [Segment]>) {
        if let Some(own) = &section.page {
            if let Some(page) = self.published.get(own) {
                if let Some(title) = &page.title {
                    return (title.clone(), page.formatted_title());
                }
            } else if let Some(title) = self.project.file(own).and_then(|f| f.title.clone()) {
                return (title, None);
            }
        }
        (folder_label(&section.folder), None)
    }

    /// The page at `path`, when the build publishes and offers it.
    fn page(&self, path: &RelPath) -> Option<NavPage<'a>> {
        let page = *self.published.get(path)?;
        if page.listing == ascribe_model::Listing::Unlisted {
            return None;
        }
        let model = self.project.model();
        let (label, formatted) = match &page.nav_label {
            Some(own) => (own.text.clone(), own.formatted.as_deref()),
            None => (
                page.title.clone().unwrap_or_else(|| stem(path).to_owned()),
                page.formatted_title(),
            ),
        };
        Some(NavPage {
            page,
            label,
            formatted,
            type_name: self
                .project
                .file(path)
                .and_then(|f| f.content_type(model))
                .map(|t| t.name.as_str()),
        })
    }
}

/// A section's label when neither `ascribe.nav` nor its own page gives it
/// one: its folder's name with its first letter capitalized.
pub fn folder_label(folder: &str) -> String {
    let name = folder
        .trim_end_matches('/')
        .rsplit('/')
        .next()
        .unwrap_or_default();
    let mut chars = name.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().chain(chars).collect(),
        None => String::new(),
    }
}

/// A page's label as the source has it, before a build: its `nav-label` as
/// plain text, else its title, else its file name without `.md`.
pub fn file_label(file: &FileIndex, model: &ContentModel) -> String {
    let own = file
        .frontmatter
        .as_ref()
        .and_then(|f| f.get(NAV_LABEL))
        .and_then(|l| l.as_str());
    let inline = file
        .content_type(model)
        .and_then(|t| t.frontmatter.fields.iter().find(|f| f.name == "title"))
        .is_some_and(|f| f.inline.is_some());
    match own {
        Some(own) if inline => {
            ascribe_model::inline::plain_text(&ascribe_model::inline::parse(own, &|_| None))
        }
        Some(own) => own.to_owned(),
        None => file
            .title
            .clone()
            .unwrap_or_else(|| stem(&file.path).to_owned()),
    }
}

/// The folder a path is in, ending in `/`, or `""` at the content root.
fn parent(path: &str) -> &str {
    match path.rfind('/') {
        Some(at) => &path[..=at],
        None => "",
    }
}

/// The page beside `folder` with its name: `checks/diagnostics.md` for
/// `checks/diagnostics/`. None for the root.
fn beside(folder: &str) -> Option<String> {
    let name = folder.strip_suffix('/')?;
    Some(format!("{name}.md"))
}

/// A page's file name without `.md`.
fn stem(path: &RelPath) -> &str {
    let name = path.as_str().rsplit('/').next().unwrap_or_default();
    name.strip_suffix(".md").unwrap_or(name)
}
