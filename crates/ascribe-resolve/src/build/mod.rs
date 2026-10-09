//! Build resolution (SPEC §9.2 steps 2 to 7, §9.3): turning expanded pages
//! into **resolved pages** for a given build.
//!
//! A [`BuildResolver`] holds one project, one build, and a [`Router`]. It runs
//! the passes below on a page, in SPEC §9.2's order, each a function of its
//! own with clear inputs and outputs:
//!
//! | Step | Pass | Module |
//! |---|---|---|
//! | 1 | Includes ([`Project::expand`]) | `expand.rs` |
//! | 2 | Availability: feature keys and inherited scopes | `availability.rs` |
//! | 3 | Build modes: variant selection and availability filter | `modes.rs` |
//! | 4 | Phrases | `phrases.rs` |
//! | 5 | Heading ids: every heading's page id | `ids.rs` |
//! | 6 | Links, and the assets that survive | `links.rs` |
//! | 7 | Glossary | `glossary.rs` |
//!
//! Passes 2 to 5 depend only on the page, so a page's headings and page ids
//! are known without resolving its links; that's what lets a link find its
//! target's page id, or find that the build removed the target. The resolver
//! runs those passes for a target page when a link needs it, and keeps the
//! result.
//!
//! What it does not do is **report** anything. Page-level problems are
//! recorded on the page ([`ResolvedPage::problems`]) for the page-level
//! checks:
//! `include-cycle` and `include-id-missing` from expansion,
//! `available-exceeds-scope`, `variant-no-arm-survives` (a group with no
//! surviving arm), `link-id-removed`, and `link-page-dropped`.
//!
//! ```
//! use std::sync::Arc;
//! use ascribe_core::{FileId, RelPath};
//! use ascribe_resolve::{DefaultRouter, Layout, MemoryFs, Project};
//!
//! let model = ascribe_model::load_str(
//!     "spec = \"0.1\"\n[dimensions.deployment]\nvalues = [\"cloud\", \"self-managed\"]\n\
//!      [builds.cloud]\nvariants = { deployment = \"cloud\" }\navailability = \"badge\"\n",
//!     FileId::new(0),
//! )
//! .expect("a valid model");
//! let layout = Layout::from_model(&model);
//! let fs = MemoryFs::new(&layout).with_source(
//!     "index.md",
//!     "---\ntitle: Home\n---\n\n@variant {deployment=cloud}:\nCloud.\n@variant {deployment=self-managed}:\nServer.\n@end\n",
//! );
//! let project = Project::load(Arc::new(model.clone()), layout, &fs);
//! let build = model.build("cloud").expect("a build");
//! let router = DefaultRouter::new();
//! let page = project
//!     .resolve_page(&RelPath::parse("index.md").expect("a path"), build, &router)
//!     .expect("a page the build publishes");
//! // One arm survives, so its content replaces the group.
//! assert_eq!(page.blocks.len(), 1);
//! ```

mod availability;
mod glossary;
mod ids;
mod inlines;
mod links;
mod modes;
mod phrases;
mod router;
mod tree;

use std::cell::RefCell;
use std::collections::{BTreeMap, HashMap};
use std::rc::Rc;
use std::sync::Arc;

use ascribe_core::{FileId, RelPath, Router, Slugger, Span};
use ascribe_model::{Build, Segment};

pub(crate) use glossary::Terms;
pub use glossary::glossary_targets;
pub use router::DefaultRouter;
pub use tree::{
    Annotation, Availability, DropReason, DroppedPage, FormattedField, GlossaryUse, HeadingIds,
    LinkTarget, Removal, Removed, ResolvedArm, ResolvedBlock, ResolvedBuild, ResolvedItem,
    ResolvedKind, ResolvedLink, ResolvedPage, ResolvedRow, Scope, Substitution,
};

use crate::index::FileKind;
use crate::project::Project;
use crate::slug::{default_slugger, slugger_by_name};

impl Project {
    /// A resolver for one build, to resolve several pages of it. It keeps what
    /// it works out about each page, so resolving every page of a build costs
    /// one pass over each.
    pub fn resolver<'p>(&'p self, build: &'p Build, router: &'p dyn Router) -> BuildResolver<'p> {
        BuildResolver::new(self, build, router)
    }

    /// Resolves one page for a build. `None` if the file isn't a page of the
    /// project, or the build doesn't publish it ([`Project::dropped`]).
    pub fn resolve_page(
        &self,
        page: &RelPath,
        build: &Build,
        router: &dyn Router,
    ) -> Option<ResolvedPage> {
        self.resolver(build, router).page(page)
    }

    /// Resolves every page of a build, and lists the pages it drops.
    pub fn resolve_build(&self, build: &Build, router: &dyn Router) -> ResolvedBuild {
        self.resolver(build, router).build()
    }

    /// Why the build doesn't publish a page, if it doesn't (SPEC §9.3): its
    /// `variant` frontmatter conflicts with a selection, or its `available`
    /// frontmatter makes it unavailable in a filter build. `None` for a page
    /// the build publishes, and for a file that isn't a page.
    pub fn dropped(&self, page: &RelPath, build: &Build) -> Option<DropReason> {
        let index = self.file(page)?;
        (index.kind == FileKind::Page)
            .then(|| modes::drop_reason(self.model(), index, build))
            .flatten()
    }

    /// What a build's modes take out of a page it publishes, and why: the
    /// same decisions [`Project::resolve_page`] makes, in the order the passes
    /// meet them.
    /// Only the outermost removal is listed; nothing inside removed content
    /// is. `None` if the file isn't a page, or the build doesn't publish it.
    pub fn removed(&self, page: &RelPath, build: &Build) -> Option<Vec<Removed>> {
        let index = self.file(page)?;
        if index.kind != FileKind::Page || self.dropped(page, build).is_some() {
            return None;
        }
        let model = self.model();
        let expanded = self.expand(page)?;
        let blocks = availability::annotate(
            self,
            &expanded.blocks,
            availability::page_availability(model, index),
            &mut Vec::new(),
        );
        let (_, removed) = modes::apply(blocks, build, model, &mut Vec::new());
        Some(removed)
    }
}

/// Page ids of a page's headings, by the file and span each is written at.
type HeadingTable = HashMap<(FileId, Span), String>;

/// Resolves the pages of one build. See the [module documentation](self).
pub struct BuildResolver<'p> {
    project: &'p Project,
    build: &'p Build,
    router: &'p dyn Router,
    slugger: Box<dyn Slugger>,
    /// Pages after passes 2 to 5, by path.
    staged: RefCell<BTreeMap<RelPath, Option<Rc<ResolvedPage>>>>,
    /// The page id of each heading of a staged page, by where it's written,
    /// so a link finds its target's heading without walking the page.
    headings: RefCell<BTreeMap<RelPath, Rc<HeadingTable>>>,
}

impl<'p> BuildResolver<'p> {
    /// A resolver for `build`, routing links with `router`.
    pub fn new(project: &'p Project, build: &'p Build, router: &'p dyn Router) -> Self {
        let slugger =
            slugger_by_name(&project.model().consumer.slugger).unwrap_or_else(default_slugger);
        BuildResolver {
            project,
            build,
            router,
            slugger,
            staged: RefCell::new(BTreeMap::new()),
            headings: RefCell::new(BTreeMap::new()),
        }
    }

    pub(crate) fn project(&self) -> &'p Project {
        self.project
    }

    pub(crate) fn router(&self) -> &'p dyn Router {
        self.router
    }

    pub(crate) fn build_name(&self) -> &str {
        &self.build.name
    }

    /// Whether the build publishes this file: it's a page, and not dropped.
    pub fn is_published(&self, path: &RelPath) -> bool {
        self.project
            .file(path)
            .is_some_and(|f| f.kind == FileKind::Page)
            && self.project.dropped(path, self.build).is_none()
    }

    /// A page's title as links show it: the frontmatter `title`, with the
    /// phrases the content model allows substituted, and in pieces when it
    /// sets `inline`.
    pub(crate) fn title_of(&self, path: &RelPath) -> Option<Vec<Segment>> {
        let page = self.stage(path)?;
        match page.formatted_title() {
            Some(segments) => Some(segments.to_vec()),
            None => page.title.clone().map(|t| vec![Segment::Text(t)]),
        }
    }

    /// A page after passes 2 to 5 (availability, build modes, phrases,
    /// heading ids), before its links. `None` for a file that isn't a page
    /// the build publishes.
    pub(crate) fn stage(&self, path: &RelPath) -> Option<Rc<ResolvedPage>> {
        if let Some(hit) = self.staged.borrow().get(path) {
            return hit.clone();
        }
        let staged = self.run_passes(path).map(Rc::new);
        self.staged
            .borrow_mut()
            .insert(path.clone(), staged.clone());
        staged
    }

    /// The page id, on `page` in this build, of the heading written at `span`
    /// in `file`. `None` if the build doesn't publish the page, or removes the
    /// heading. A fragment included twice has its headings twice; this is the
    /// first.
    pub(crate) fn page_id_of(&self, page: &RelPath, file: FileId, span: Span) -> Option<String> {
        if let Some(table) = self.headings.borrow().get(page) {
            return table.get(&(file, span)).cloned();
        }
        let staged = self.stage(page)?;
        let mut table = HeadingTable::new();
        for (block, ids) in staged.headings() {
            table
                .entry((block.file, block.span))
                .or_insert_with(|| ids.page_id.clone());
        }
        let found = table.get(&(file, span)).cloned();
        self.headings
            .borrow_mut()
            .insert(page.clone(), Rc::new(table));
        found
    }

    fn run_passes(&self, path: &RelPath) -> Option<ResolvedPage> {
        let project = self.project;
        let model = project.model();
        let index = project.file(path)?;
        if !self.is_published(path) {
            return None;
        }
        // Step 1: includes.
        let expanded = project.expand(path)?;
        // Step 2: availability.
        let page_availability = availability::page_availability(model, index);
        let mut scope_problems = Vec::new();
        let blocks = availability::annotate(
            project,
            &expanded.blocks,
            page_availability.clone(),
            &mut scope_problems,
        );
        // Step 3: build modes.
        let mut mode_problems = Vec::new();
        let (mut blocks, _) = modes::apply(blocks, self.build, model, &mut mode_problems);
        // A problem is about what the build publishes: one in content the
        // build removed isn't recorded.
        let live = modes::live(&blocks);
        let mut problems: Vec<_> = expanded
            .problems
            .iter()
            .filter(|p| modes::survives(&blocks, p))
            .cloned()
            .collect();
        problems.extend(
            scope_problems
                .into_iter()
                .filter(|(spec, _)| live.contains(&Arc::as_ptr(spec)))
                .map(|(_, problem)| problem),
        );
        problems.extend(mode_problems);
        // Step 4: phrases.
        phrases::substitute_blocks(project, &mut blocks);
        let (frontmatter, formatted) = match &index.frontmatter {
            Some(f) => {
                let (value, formatted) = phrases::frontmatter(model, path, f);
                (Some(value), formatted)
            }
            None => (None, Vec::new()),
        };
        let title = frontmatter
            .as_ref()
            .and_then(|f| f.get("title"))
            .and_then(|t| t.as_str())
            .map(str::to_owned);
        // Step 5: heading ids.
        ids::assign(project, &mut blocks, self.slugger.as_ref());
        Some(ResolvedPage {
            path: path.clone(),
            file: index.file,
            build: self.build.name.clone(),
            route: self.router.route(path),
            frontmatter,
            title,
            formatted,
            availability: page_availability,
            blocks,
            assets: Vec::new(),
            problems,
        })
    }

    /// Resolves one page: every pass, links and glossary included. `None` if
    /// the file isn't a page the build publishes.
    pub fn page(&self, path: &RelPath) -> Option<ResolvedPage> {
        let staged = self.stage(path)?;
        let mut page = (*staged).clone();
        // Step 6: links, and the assets that survive.
        links::resolve(self, &mut page);
        // Step 7: glossary.
        glossary::link_terms(self, &mut page);
        Some(page)
    }

    /// Resolves every page of the build, in path order, and lists the pages
    /// it drops.
    pub fn build(&self) -> ResolvedBuild {
        let mut pages = Vec::new();
        let mut dropped = Vec::new();
        for index in self.project.pages() {
            match self.project.dropped(&index.path, self.build) {
                Some(reason) => dropped.push(DroppedPage {
                    path: index.path.clone(),
                    reason,
                }),
                None => pages.extend(self.page(&index.path)),
            }
        }
        ResolvedBuild {
            build: self.build.name.clone(),
            pages,
            dropped,
        }
    }
}
