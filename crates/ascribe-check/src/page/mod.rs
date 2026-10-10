//! Page-level checks (SPEC §8.1): what only the assembled page shows.
//!
//! A page-level check runs on a **resolved page**: includes expanded,
//! availability resolved, and one build's modes applied (SPEC §9.2). It covers
//! id uniqueness, links whose target is an id, and everything a build removes:
//!
//! | Row | Comes from |
//! |---|---|
//! | `id-duplicate`, `heading-duplicate-without-id` | the page's resolved headings |
//! | `link-id-missing` | the source index, for the links the build publishes |
//! | `include-id-missing`, `include-cycle` | include expansion |
//! | `variant-no-arm-survives`, `available-exceeds-scope` | build modes and the availability scope check |
//! | `link-id-removed`, `link-page-dropped` | link resolution, per build |
//! | `heading-level-skipped` | the page's resolved headings |
//! | `page-size` | the page's plain Markdown, per build |
//!
//! The entry points are [`check_project`], for one build, and
//! [`check_all_builds`], for every build of the project. `ascribe check`,
//! `ascribe build`, and the language server all call them, which is what keeps
//! their diagnostics identical.
//!
//! # Where a problem is reported, and how often
//!
//! A problem is reported at the source location that causes it. One inside
//! included content is reported at the include site (the outermost `@include`,
//! `via[0]`), with its location in the fragment as related information, not a
//! second diagnostic (SPEC §8.1). One that several places cause is
//! reported once, at the later of them.
//!
//! [`check_all_builds`] reports each distinct problem once, however many
//! builds it appears in, and lists them in [`Diagnostic::builds`].
//!
//! # Content no build publishes
//!
//! A build records only the problems in content it publishes, so
//! content that *no* build publishes (an arm none of the builds selects, a
//! page every build drops) would never be checked at page level.
//! [`check_all_builds`] therefore resolves the project once more with a build
//! that keeps everything (`switch` and `badge`) and reports the problems whose
//! cause is in content no real build publishes, marked
//! [`Diagnostic::unpublished`]. It costs nothing when a build already
//! keeps everything.
//!
//! A fragment that no page includes is part of no page, so it has no
//! page-level diagnostics of its own.

mod bridge;
mod collect;
mod size;

use std::cell::RefCell;
use std::collections::{BTreeSet, HashMap, HashSet};
use std::sync::Arc;

use ascribe_core::{FileId, RelPath, Span, diagnostics};
use ascribe_model::{AvailabilityMode, Build, CheckLevel, VariantMode};
use ascribe_resolve::{DefaultRouter, IncludeSite, ResolvedPage};

use crate::{Diagnostic, Project, check_files};

use bridge::Indexed;
pub(crate) use bridge::held_index;
use collect::{Found, Identity, LinkProblems, check_page, identity};

/// The name of the build that keeps everything, used to look for content no
/// build publishes. It never appears in a diagnostic.
const KEEP_EVERYTHING: &str = "(everything)";

/// Every diagnostic of one build: the file-level ones (SPEC §8.1), then the
/// page-level ones for `build`, each naming it in [`Diagnostic::builds`].
///
/// This is the one function every tool calls for a build. It indexes the
/// project each time; to check several builds, call [`check_all_builds`], or
/// keep a [`PageChecker`].
pub fn check_project(project: &Project, build: &Build) -> Vec<Diagnostic> {
    let mut out = check_files(project);
    out.extend(PageChecker::new(project).check(build));
    out
}

/// Every diagnostic of a set of builds: the file-level ones once, then the
/// page-level ones of each build, each distinct problem once with the builds
/// it appears in (as [`check_all_builds`] does, without the pass over content
/// no build publishes, which needs every build).
pub fn check_builds(project: &Project, builds: &[&Build]) -> Vec<Diagnostic> {
    let mut out = check_files(project);
    out.extend(PageChecker::new(project).check_builds(builds));
    out
}

/// The page-level diagnostics of one build, without the file-level ones.
pub fn check_pages(project: &Project, build: &Build) -> Vec<Diagnostic> {
    PageChecker::new(project).check(build)
}

/// Every diagnostic of the project: the file-level ones once, then the
/// page-level ones of every build, each distinct problem once with the builds
/// it appears in, then the problems in content no build publishes.
pub fn check_all_builds(project: &Project) -> Vec<Diagnostic> {
    let mut out = check_files(project);
    out.extend(PageChecker::new(project).check_all());
    out
}

/// The page-level checks over one indexed project, for checking several
/// builds without indexing it again.
pub struct PageChecker<'p> {
    project: &'p Project,
    indexed: Indexed<'p>,
    links: LinkProblems,
    errors_only: bool,
}

/// A project's source index as a [`PageChecker`] reads it, made once and
/// kept, so the checks of a project kept loaded don't index it each time.
/// It's of the project it was made from, which mustn't change.
pub struct PageIndex(Indexed<'static>);

impl PageIndex {
    /// Indexes the project, as [`PageChecker::new`] does.
    pub fn new(project: &Project) -> PageIndex {
        PageIndex(Indexed::new(project))
    }
}

impl<'p> PageChecker<'p> {
    /// A checker over `index`, made from `project` by [`PageIndex::new`]:
    /// the same checks as [`PageChecker::new`]'s, without indexing again.
    pub fn with_page_index(project: &'p Project, index: &'p PageIndex) -> PageChecker<'p> {
        PageChecker {
            project,
            indexed: index.0.borrowed(),
            links: RefCell::new(HashMap::new()),
            errors_only: false,
        }
    }

    /// Indexes the project.
    pub fn new(project: &'p Project) -> PageChecker<'p> {
        PageChecker {
            project,
            indexed: Indexed::new(project),
            links: RefCell::new(HashMap::new()),
            errors_only: false,
        }
    }

    /// A checker over an index the caller already has, so nothing is indexed
    /// again: the language server passes its incremental index (a
    /// `ascribe_resolve::Snapshot` dereferences to one). `index` must be an
    /// index of the same files as `project`, with the same file ids and
    /// texts; only [`PageChecker::check_resolved`] is meant for it (the other
    /// methods resolve every page).
    pub fn with_index(
        project: &'p Project,
        index: &'p ascribe_resolve::Project,
    ) -> PageChecker<'p> {
        PageChecker {
            project,
            indexed: Indexed::shared(index),
            links: RefCell::new(HashMap::new()),
            errors_only: false,
        }
    }

    /// The same checker, leaving out the checks that cost the most and can't
    /// report an error: `page-size`, which renders every page, unless
    /// `[checks]` raises it to an error. For a caller that only counts
    /// errors; its errors are the same.
    #[must_use]
    pub fn errors_only(mut self) -> PageChecker<'p> {
        self.errors_only = true;
        self
    }

    /// The page-level diagnostics of `build` for these resolved pages alone:
    /// what [`PageChecker::check`] reports for them, and nothing about the
    /// rest of the project. A diagnostic is located in the page's own file,
    /// except an `include-cycle`, which is located where the cycle closes (in
    /// a fragment), so the diagnostics located in a file are those of the
    /// pages that are the file or include it, transitively.
    pub fn check_resolved(&self, build: &Build, pages: &[&ResolvedPage]) -> Vec<Diagnostic> {
        let found = pages
            .iter()
            .flat_map(|page| self.page(build, page))
            .collect();
        self.finish(vec![(Some(build.name.as_str()), found)])
    }

    /// The page-level diagnostics of `build` for these pages alone, each
    /// resolved from the checker's index: what [`PageChecker::check`]
    /// reports for them. A path that isn't a page, or that the build drops,
    /// has none.
    pub fn check_pages(&self, build: &Build, pages: &[RelPath]) -> Vec<Diagnostic> {
        let router = DefaultRouter::from_consumer(&self.project.model().consumer);
        let found = pages
            .iter()
            .filter_map(|path| self.indexed.index.resolve_page(path, build, &router))
            .flat_map(|page| self.page(build, &page))
            .collect();
        self.finish(vec![(Some(build.name.as_str()), found)])
    }

    /// The page-level diagnostics of `build` for the pages that are one of
    /// `files` or include one, transitively: every page-level diagnostic
    /// located in those files or with a related place in them.
    pub fn check_reaching(&self, build: &Build, files: &[RelPath]) -> Vec<Diagnostic> {
        let index = &self.indexed.index;
        let mut pages: BTreeSet<RelPath> = BTreeSet::new();
        for file in files {
            pages.insert(file.clone());
            pages.extend(index.including_pages(file));
        }
        let pages: Vec<RelPath> = pages.into_iter().collect();
        self.check_pages(build, &pages)
    }

    /// The page-level diagnostics of one build.
    pub fn check(&self, build: &Build) -> Vec<Diagnostic> {
        let found = self.found(build, &mut |_| {});
        self.finish(vec![(Some(build.name.as_str()), found)])
    }

    /// The page-level diagnostics of several builds, each distinct problem
    /// once, naming the builds it appears in.
    pub fn check_builds(&self, builds: &[&Build]) -> Vec<Diagnostic> {
        let per_build = builds
            .iter()
            .map(|build| (Some(build.name.as_str()), self.found(build, &mut |_| {})))
            .collect();
        self.finish(per_build)
    }

    /// The page-level diagnostics of every build of the project, and of the
    /// content no build publishes. See the [module documentation](self).
    pub fn check_all(&self) -> Vec<Diagnostic> {
        let model = self.project.model();
        let mut published: HashSet<Published> = HashSet::new();
        let mut per_build = Vec::new();
        for build in &model.builds {
            let found = self.found(build, &mut |page| add_published(&mut published, page));
            per_build.push((Some(build.name.as_str()), found));
        }
        let keeps_everything = model.builds.iter().any(|b| {
            b.variants == VariantMode::Switch && b.availability == AvailabilityMode::Badge
        });
        // Content no build publishes is checked with a
        // build that keeps everything, and reported as belonging to no build.
        // A fragment no page includes is in no page, so
        // it isn't part of this pass either.
        if !keeps_everything {
            per_build.push((None, self.unpublished(&published)));
        }
        self.finish(per_build)
    }

    /// Every page-level problem of one build. `on_page` sees each page the
    /// build publishes.
    fn found(&self, build: &Build, on_page: &mut dyn FnMut(&ResolvedPage)) -> Vec<Found> {
        let router = DefaultRouter::from_consumer(&self.project.model().consumer);
        let resolved = self.indexed.index.resolve_build(build, &router);
        let mut found = Vec::new();
        for page in &resolved.pages {
            on_page(page);
            found.extend(self.page(build, page));
        }
        found
    }

    /// The page-level problems of one page a build publishes: those of its
    /// resolved form, and its size.
    fn page(&self, build: &Build, page: &ResolvedPage) -> Vec<Found> {
        let index = &self.indexed.index;
        let mut found = check_page(index, page, &self.links);
        let counted = !self.errors_only
            || index.model().checks.level(diagnostics::PAGE_SIZE) == Some(CheckLevel::Error);
        if counted
            && let Some(limit) = size::limit(index)
            && let Some(f) = size::page_size(index, self.project.root(), build, page, limit)
        {
            found.push(f);
        }
        found
    }

    /// The problems in content that no build publishes: those a build that
    /// keeps everything finds, whose cause is in a block no real build has.
    fn unpublished(&self, published: &HashSet<Published>) -> Vec<Found> {
        let everything = Build {
            name: KEEP_EVERYTHING.to_owned(),
            variants: VariantMode::Switch,
            availability: AvailabilityMode::Badge,
        };
        let router = DefaultRouter::from_consumer(&self.project.model().consumer);
        let resolved = self.indexed.index.resolve_build(&everything, &router);
        let mut found = Vec::new();
        for page in &resolved.pages {
            for f in check_page(&self.indexed.index, page, &self.links) {
                // A cause that no block holds (which shouldn't happen) is
                // reported: the conservative reading.
                let is_published = innermost(page, &f).is_some_and(|key| published.contains(&key));
                if !is_published {
                    found.push(f);
                }
            }
        }
        found
    }

    /// Merges the problems of each build (`None`: content no build
    /// publishes) into diagnostics: a problem found in several builds once,
    /// naming them, in file and source order.
    fn finish(&self, per_build: Vec<(Option<&str>, Vec<Found>)>) -> Vec<Diagnostic> {
        struct Merged {
            found: Found,
            builds: Vec<String>,
            unpublished: bool,
        }
        let per_build = largest_page_sizes(per_build);
        let mut merged: Vec<(Identity, Merged)> = Vec::new();
        for (build, list) in per_build {
            for found in list {
                let key = identity(&found.issue);
                match merged.iter_mut().find(|(k, _)| *k == key) {
                    Some((_, m)) => {
                        if let Some(build) = build
                            && !m.builds.iter().any(|b| b == build)
                        {
                            m.builds.push(build.to_owned());
                        }
                    }
                    None => merged.push((
                        key,
                        Merged {
                            found,
                            builds: build.map(str::to_owned).into_iter().collect(),
                            unpublished: build.is_none(),
                        },
                    )),
                }
            }
        }
        let mut out: Vec<Diagnostic> = merged
            .into_iter()
            .map(|(_, m)| {
                let mut issue = m.found.issue;
                // A row whose message names the build names them all: in
                // several builds, with its `builds` message variant (resolved
                if issue.arg("build").is_some() && m.builds.len() > 1 {
                    let names = m
                        .builds
                        .iter()
                        .map(|b| format!("`{b}`"))
                        .collect::<Vec<_>>()
                        .join(", ");
                    issue = issue.with_variant("builds").with_arg("builds", names);
                }
                self.to_project_ids(&mut issue);
                let mut d = Diagnostic::from_issue(&issue);
                d.builds = m.builds;
                d.unpublished = m.unpublished;
                d
            })
            .collect();
        out.sort_by(|a, b| {
            (a.location.file, a.location.span, a.slug).cmp(&(
                b.location.file,
                b.location.span,
                b.slug,
            ))
        });
        out
    }

    /// Renumbers the files of an issue from the index's ids to the checked
    /// project's.
    fn to_project_ids(&self, issue: &mut ascribe_core::Issue) {
        let renumber = |file: &mut FileId| *file = self.indexed.file(*file);
        renumber(&mut issue.location.file);
        for related in &mut issue.related {
            renumber(&mut related.location.file);
        }
        for fix in &mut issue.fixes {
            renumber(&mut fix.file);
        }
    }
}

/// The builds' `page-size` problems, each page's as it is in the build where
/// the page is largest, so a page over the limit in several builds is one
/// problem that names them.
fn largest_page_sizes(
    mut per_build: Vec<(Option<&str>, Vec<Found>)>,
) -> Vec<(Option<&str>, Vec<Found>)> {
    let mut largest: HashMap<FileId, ascribe_core::Issue> = HashMap::new();
    for (_, list) in &per_build {
        for f in list
            .iter()
            .filter(|f| f.issue.slug == diagnostics::PAGE_SIZE)
        {
            let file = f.issue.location.file;
            let bigger = largest
                .get(&file)
                .is_none_or(|l| size::size_of(&f.issue) > size::size_of(l));
            if bigger {
                largest.insert(file, f.issue.clone());
            }
        }
    }
    for (_, list) in &mut per_build {
        for f in list
            .iter_mut()
            .filter(|f| f.issue.slug == diagnostics::PAGE_SIZE)
        {
            if let Some(l) = largest.get(&f.issue.location.file) {
                // Every argument but the build, which says where it's found.
                let build = f.issue.arg("build").unwrap_or_default().to_owned();
                f.issue.args = l.args.clone();
                for arg in &mut f.issue.args {
                    if arg.name == "build" {
                        arg.value.clone_from(&build);
                    }
                }
            }
        }
    }
    per_build
}

/// A block of a page as one build publishes it: the page, the file and span
/// it's written at, and the includes it came through.
type Published = (RelPath, FileId, Span, Vec<IncludeSite>);

fn add_published(set: &mut HashSet<Published>, page: &ResolvedPage) {
    page.visit(&mut |b| {
        set.insert((page.path.clone(), b.file, b.span, b.via.to_vec()));
    });
}

/// The innermost block of `page` that holds a problem's cause.
fn innermost(page: &ResolvedPage, found: &Found) -> Option<Published> {
    let (file, span, via): &(FileId, Span, Arc<[IncludeSite]>) = &found.cause;
    let mut best: Option<Published> = None;
    page.visit(&mut |b| {
        let holds = b.file == *file
            && *b.via == **via
            && b.span.start() <= span.start()
            && span.end() <= b.span.end();
        let tighter = best
            .as_ref()
            .is_none_or(|(_, _, s, _)| b.span.len() < s.len());
        if holds && tighter {
            best = Some((page.path.clone(), b.file, b.span, b.via.to_vec()));
        }
    });
    best
}
