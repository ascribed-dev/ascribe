//! Comparing two versions of a project, build by build and page by page.

use std::borrow::Cow;
use std::collections::{BTreeMap, BTreeSet, HashMap};

use ascribe_core::RelPath;
use ascribe_resolve::{AstroRouter, Project, ResolvedPage};
use serde::Serialize;

use crate::align::{self, Found};
use crate::tree::{Anchor, Node, PageTree, TreeBuilder, lf};
use crate::words::diff_words;

/// The name `because` gives the content model.
pub(crate) const MODEL_FILE: &str = "ascribe.toml";

/// One version of a project.
#[derive(Clone, Copy)]
pub struct Side<'a> {
    /// The project's source index.
    pub project: &'a Project,
    /// The content model's text.
    pub model_text: &'a str,
}

/// What changed in one build.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
pub struct BuildDiff {
    /// The build's name.
    pub build: String,
    /// The pages that changed, in path order.
    pub pages: Vec<PageDiff>,
}

/// What changed on one page of a build.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub struct PageDiff {
    /// The page's content path.
    pub path: String,
    /// Its route: the new one, or for a removed page the old one.
    pub route: String,
    /// Whether the build publishes it only now, only before, or in both
    /// with a difference.
    pub status: PageStatus,
    /// Whether the page's own file changed (or exists on one side only).
    pub own_file_changed: bool,
    /// The other changed files the page's change can come from: fragments it
    /// includes and pages its links take a title or a heading from, in path
    /// order; then the snippets whose code changed, by address
    /// (`code:app.py#main`); and `ascribe.toml` (last) when the content model
    /// is a cause.
    pub because: Vec<String>,
    /// What changed about the page itself besides its blocks, in this
    /// order: `title`, `frontmatter`, `availability` (the page-level one),
    /// and `route`. Empty for an added or removed page.
    pub page_changed: Vec<&'static str>,
    /// How many changes of each kind.
    pub counts: Counts,
    /// The block-level changes, in the page's order, a removed block where
    /// it was. Empty for an added or removed page.
    pub changes: Vec<Change>,
}

impl PageDiff {
    /// What changed on the page, in a few words: "2 changed, 1 added", or
    /// "added", and the files the change comes through, each written by
    /// `cause`: "(through _fragments/prereqs.md)".
    pub fn describe(&self, cause: impl Fn(&str) -> String) -> String {
        let mut text = self.summary();
        if !self.because.is_empty() {
            let lead = if self.own_file_changed {
                "also through"
            } else {
                "through"
            };
            let causes: Vec<String> = self.because.iter().map(|c| cause(c)).collect();
            text.push_str(&format!(" ({lead} {})", causes.join(", ")));
        }
        text
    }

    /// What changed on the page, without where from: "2 changed, 1 added,
    /// title changed", or "added".
    pub fn summary(&self) -> String {
        match self.status {
            PageStatus::Added => "added".to_owned(),
            PageStatus::Removed => "removed".to_owned(),
            PageStatus::Changed => {
                let c = self.counts;
                let mut parts: Vec<String> = [
                    (c.changed, "changed"),
                    (c.added, "added"),
                    (c.removed, "removed"),
                    (c.moved, "moved"),
                ]
                .iter()
                .filter(|(n, _)| *n > 0)
                .map(|(n, kind)| format!("{n} {kind}"))
                .collect();
                if !self.page_changed.is_empty() {
                    parts.push(format!("{} changed", and_list(&self.page_changed)));
                }
                parts.join(", ")
            }
        }
    }
}

/// `a`, `a and b`, `a, b, and c`.
fn and_list(items: &[&str]) -> String {
    match items {
        [] => String::new(),
        [one] => (*one).to_owned(),
        [a, b] => format!("{a} and {b}"),
        [rest @ .., last] => format!("{}, and {last}", rest.join(", ")),
    }
}

/// Whether a page is new, gone, or different.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum PageStatus {
    /// The build publishes it now, and didn't before.
    Added,
    /// The build published it before, and doesn't now.
    Removed,
    /// The build publishes it in both, and it differs.
    Changed,
}

/// How many changes of each kind a page has.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
pub struct Counts {
    /// Blocks whose content changed.
    pub changed: usize,
    /// Blocks only in the new version.
    pub added: usize,
    /// Blocks only in the old version.
    pub removed: usize,
    /// Blocks in both, somewhere else.
    pub moved: usize,
}

/// One block's change.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
pub struct Change {
    /// What happened to it.
    pub kind: ChangeKind,
    /// Where it's written now: every kind but `removed`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub now: Option<Anchor>,
    /// Where it was written: every kind but `added`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub was: Option<Anchor>,
    /// For changed prose, the words that differ.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub words: Option<Words>,
    /// For a removed block, and a moved block's old place: the block of the
    /// new version it came after, among its siblings. Absent when it was
    /// first.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub after: Option<Anchor>,
    /// For a removed block, and a moved block's old place: the block of the
    /// new version it was inside. Absent at the top of the page.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent: Option<Anchor>,
    /// For a removed block, its text, whitespace collapsed, so it can be
    /// shown where it was.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
}

/// The kinds of block change.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum ChangeKind {
    /// The block's content changed.
    Changed,
    /// The block is new.
    Added,
    /// The block is gone.
    Removed,
    /// The same block is somewhere else.
    Moved,
}

/// The words that differ inside a changed block of prose. Ranges are
/// `[start, end)` in characters (Unicode scalar values) of each side's
/// `text`, which is the block's text with whitespace collapsed.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub struct Words {
    /// Ranges in `now_text`: words added or replacing others.
    pub now: Vec<[usize; 2]>,
    /// Ranges in `was_text`: words removed or replaced.
    pub was: Vec<[usize; 2]>,
    /// The block's text now.
    pub now_text: String,
    /// The block's text before.
    pub was_text: String,
}

/// The files whose text differs between two versions, and whether the
/// content model's does.
struct ChangedFiles {
    files: BTreeSet<RelPath>,
    model: bool,
}

impl ChangedFiles {
    fn between(base: Option<Side<'_>>, now: Side<'_>) -> ChangedFiles {
        let Some(base) = base else {
            return ChangedFiles {
                files: now.project.files().map(|f| f.path.clone()).collect(),
                model: true,
            };
        };
        let mut files = BTreeSet::new();
        let old: BTreeMap<&RelPath, Cow<'_, str>> = base
            .project
            .files()
            .map(|f| (&f.path, lf(&f.source)))
            .collect();
        for file in now.project.files() {
            if old.get(&file.path) != Some(&lf(&file.source)) {
                files.insert(file.path.clone());
            }
        }
        for path in old.keys() {
            if now.project.file(path).is_none() {
                files.insert((*path).clone());
            }
        }
        ChangedFiles {
            files,
            model: base.model_text != now.model_text,
        }
    }
}

/// What changed in each of `builds` (by name) between `base` and `now`. A
/// build that `base` doesn't have publishes every page as added; with no
/// `base` at all (the project didn't exist), so does every build. A name
/// that isn't a build of `now` is skipped.
pub fn compare_builds(base: Option<Side<'_>>, now: Side<'_>, builds: &[&str]) -> Vec<BuildDiff> {
    let changed = ChangedFiles::between(base, now);
    let reach = base.and_then(|base| reachable(base, now));
    let routers = Routers::new(base, now);
    let mut out = Vec::new();
    for name in builds {
        let Some(sides) = routers.build(name) else {
            continue;
        };
        let mut paths: BTreeSet<&RelPath> = now.project.pages().map(|p| &p.path).collect();
        if let Some((b, _)) = &sides.base {
            paths.extend(b.project.pages().map(|p| &p.path));
        }
        // A page nothing it uses changed in resolves the same on both sides.
        let pages = paths
            .into_iter()
            .filter(|path| reach.as_ref().is_none_or(|reach| reach.contains(*path)))
            .filter_map(|path| sides.compare(path, &changed))
            .collect();
        out.push(BuildDiff {
            build: (*name).to_owned(),
            pages,
        });
    }
    out
}

/// What changed on the page at `path` in the build `build` between `base`
/// and `now`, as [`compare_builds`] reports it; `None` when nothing did, the
/// page is in neither version, or `now` has no such build.
pub fn compare_page_in(
    base: Option<Side<'_>>,
    now: Side<'_>,
    build: &str,
    path: &RelPath,
) -> Option<PageDiff> {
    let changed = ChangedFiles::between(base, now);
    Routers::new(base, now)
        .build(build)?
        .compare(path, &changed)
}

/// Of the pages at `paths`, those that changed in the build `build` between
/// `base` and `now` apart from their snippets' code: their own file changed
/// (or is in one version only), or their resolved content differs through
/// another file they use, such as a fragment, or through the content model.
pub fn changed_apart_from_snippets(
    base: Option<Side<'_>>,
    now: Side<'_>,
    build: &str,
    paths: &[&RelPath],
) -> BTreeSet<RelPath> {
    let changed = ChangedFiles::between(base, now);
    let routers = Routers::new(base, now);
    let Some(sides) = routers.build(build) else {
        return BTreeSet::new();
    };
    paths
        .iter()
        .filter(|path| {
            sides
                .compare_parts(path, &changed)
                .is_some_and(|(diff, snippets)| {
                    diff.own_file_changed || diff.because.len() > snippets
                })
        })
        .map(|path| (*path).clone())
        .collect()
}

/// The pages whose resolved form can differ between `base` and `now`, in
/// any build; `None` when any page can, because the content model differs.
///
/// A page resolves from its own file, the files it includes, the files it
/// links to (a link's text can be the target's title or heading, its URL
/// the target's route and page id, and whether it links at all depends on
/// whether the build publishes the target), and the glossary's pages.
/// Nothing else is shared between pages: a route is a function of the
/// page's own path (`Router::route`), heading ids are assigned within one
/// page, and two pages that take the same route are found only by the site
/// output (`AstroRouter::collisions`), never in a resolved page. So a
/// page can differ only when one of those files differs: its text, what its
/// references resolve to (a file appearing or going away, an image found),
/// or its snippets' code. This is the reach the language server's
/// incremental update uses ([`ascribe_resolve::Affected::re_resolve`]),
/// found from the two versions' indexes instead of from a list of changes,
/// so nothing `git` says is taken on trust.
fn reachable(base: Side<'_>, now: Side<'_>) -> Option<BTreeSet<RelPath>> {
    if base.model_text != now.model_text {
        return None;
    }
    let sides = [base.project, now.project];
    let mut touched: BTreeSet<&RelPath> = BTreeSet::new();
    for file in now.project.files() {
        if differs(base.project, now.project, file) {
            touched.insert(&file.path);
        }
    }
    touched.extend(
        base.project
            .files()
            .filter(|f| now.project.file(&f.path).is_none())
            .map(|f| &f.path),
    );

    // Who includes what, in either version, by the target's path and by its
    // path with case folded (an include of a twin that differs only in case
    // has a problem that names it).
    let mut includers: HashMap<String, BTreeSet<&RelPath>> = HashMap::new();
    for project in sides {
        for file in project.files() {
            for target in file.includes.iter().filter_map(|i| i.target.as_ref()) {
                includers
                    .entry(target.as_str().to_lowercase())
                    .or_default()
                    .insert(&file.path);
            }
        }
    }
    let with_includers = |start: &mut dyn Iterator<Item = &RelPath>| {
        let mut seen: BTreeSet<RelPath> = BTreeSet::new();
        let mut queue: Vec<RelPath> = start.cloned().collect();
        while let Some(next) = queue.pop() {
            if !seen.insert(next.clone()) {
                continue;
            }
            for file in includers
                .get(&next.as_str().to_lowercase())
                .into_iter()
                .flatten()
            {
                if !seen.contains(*file) {
                    queue.push((*file).clone());
                }
            }
        }
        seen
    };

    let mut reached = with_includers(&mut touched.into_iter());
    let glossary = ascribe_resolve::glossary_targets(now.project.model());
    if reached.iter().any(|p| glossary.contains(p)) {
        return None;
    }
    let linkers: Vec<RelPath> = reached
        .iter()
        .flat_map(|target| sides.into_iter().flat_map(|p| p.links_to(target)))
        .map(|site| site.file.clone())
        .collect();
    reached.extend(with_includers(&mut linkers.iter()));
    Some(reached)
}

/// Whether a file of `now` reads differently in `base`: it isn't there, its
/// text differs, or the same text resolves differently (a reference's target
/// appeared or went away, a snippet's code changed).
fn differs(base: &Project, now: &Project, file: &ascribe_resolve::FileIndex) -> bool {
    let Some(was) = base.file(&file.path) else {
        return true;
    };
    was.source != file.source
        || base.resolutions(&file.path) != now.resolutions(&file.path)
        || file
            .snippets
            .iter()
            .any(|s| base.snippet_at(&file.path, s.span) != now.snippet_at(&file.path, s.span))
}

/// Each version's router.
struct Routers<'a> {
    base: Option<(Side<'a>, AstroRouter)>,
    now: (Side<'a>, AstroRouter),
}

/// One build's resolvers, in each version that has it.
struct BuildSides<'a> {
    base: Option<(Side<'a>, ascribe_resolve::BuildResolver<'a>)>,
    now: (Side<'a>, ascribe_resolve::BuildResolver<'a>),
}

impl<'a> Routers<'a> {
    fn new(base: Option<Side<'a>>, now: Side<'a>) -> Routers<'a> {
        let router = |side: Side<'a>| {
            (
                side,
                AstroRouter::from_consumer(&side.project.model().consumer),
            )
        };
        Routers {
            base: base.map(router),
            now: router(now),
        }
    }

    fn build(&self, name: &str) -> Option<BuildSides<'_>> {
        let (now, now_router) = &self.now;
        let now_build = now.project.model().build(name)?;
        let base = self.base.as_ref().and_then(|(b, router)| {
            b.project
                .model()
                .build(name)
                .map(|build| (*b, b.project.resolver(build, router)))
        });
        Some(BuildSides {
            base,
            now: (*now, now.project.resolver(now_build, now_router)),
        })
    }
}

impl BuildSides<'_> {
    fn compare(&self, path: &RelPath, changed: &ChangedFiles) -> Option<PageDiff> {
        self.compare_parts(path, changed).map(|(diff, _)| diff)
    }

    /// [`BuildSides::compare`], with how many of the page's `because` are
    /// snippets.
    fn compare_parts(&self, path: &RelPath, changed: &ChangedFiles) -> Option<(PageDiff, usize)> {
        let now_page = self.now.1.page(path);
        let base_page = self.base.as_ref().and_then(|(_, r)| r.page(path));
        compare_page(
            path,
            base_page.as_ref().zip(self.base.as_ref().map(|(b, _)| *b)),
            now_page.as_ref().map(|p| (p, self.now.0)),
            changed,
        )
    }
}

/// What changed on a page, and how many of its `because` are snippets.
fn compare_page(
    path: &RelPath,
    base: Option<(&ResolvedPage, Side<'_>)>,
    now: Option<(&ResolvedPage, Side<'_>)>,
    changed: &ChangedFiles,
) -> Option<(PageDiff, usize)> {
    let own_file_changed = changed.files.contains(path);
    let page_only = |status, route: &str| {
        let because = if !own_file_changed && changed.model {
            vec![MODEL_FILE.to_owned()]
        } else {
            Vec::new()
        };
        PageDiff {
            path: path.to_string(),
            route: route.to_owned(),
            status,
            own_file_changed,
            because,
            page_changed: Vec::new(),
            counts: Counts::default(),
            changes: Vec::new(),
        }
    };
    let ((was_page, base), (now_page, now)) = match (base, now) {
        (None, None) => return None,
        (None, Some((page, _))) => return Some((page_only(PageStatus::Added, &page.route), 0)),
        (Some((page, _)), None) => return Some((page_only(PageStatus::Removed, &page.route), 0)),
        (Some(b), Some(n)) => (b, n),
    };
    let was_tree: PageTree = TreeBuilder::new(base.project).page(was_page);
    let now_tree: PageTree = TreeBuilder::new(now.project).page(now_page);
    if was_tree.hash == now_tree.hash && was_tree.route == now_tree.route {
        return None;
    }
    let found = align::compare(&was_tree.nodes, &now_tree.nodes);
    let page_changed: Vec<&'static str> = [
        ("title", was_tree.title != now_tree.title),
        ("frontmatter", was_tree.frontmatter != now_tree.frontmatter),
        (
            "availability",
            was_tree.availability != now_tree.availability,
        ),
        ("route", was_tree.route != now_tree.route),
    ]
    .into_iter()
    .filter_map(|(name, differs)| differs.then_some(name))
    .collect();

    // The files the changes come from, and whether the model shapes them.
    let mut used: BTreeSet<RelPath> = BTreeSet::new();
    let mut model_shaped = false;
    let mut note = |node: &Node, deep: bool| {
        let mut add = |n: &Node| {
            used.extend(n.files.iter().cloned());
            model_shaped |= n.uses_model;
        };
        if deep {
            node.visit(&mut add)
        } else {
            add(node)
        }
    };
    for f in &found {
        match f {
            Found::Changed { was, now } => {
                note(was, false);
                note(now, false);
            }
            Found::Added { now } => note(now, true),
            Found::Removed { was, .. } => note(was, true),
            Found::Moved { was, now, .. } => {
                note(was, false);
                note(now, false);
            }
        }
    }
    let mut because: Vec<String> = used
        .iter()
        .filter(|p| *p != path && changed.files.contains(*p))
        .map(ToString::to_string)
        .collect();
    // A snippet whose code differs: named by its address, as a fragment is
    // by its path.
    let snippets = changed_snippets(&was_tree, &now_tree);
    let snippet_count = snippets.len();
    because.extend(snippets);
    if changed.model && (model_shaped || (because.is_empty() && !own_file_changed)) {
        because.push(MODEL_FILE.to_owned());
    }

    let mut counts = Counts::default();
    let changes: Vec<Change> = found
        .into_iter()
        .map(|f| {
            let change = to_change(f);
            match change.kind {
                ChangeKind::Changed => counts.changed += 1,
                ChangeKind::Added => counts.added += 1,
                ChangeKind::Removed => counts.removed += 1,
                ChangeKind::Moved => counts.moved += 1,
            }
            change
        })
        .collect();
    let diff = PageDiff {
        path: path.to_string(),
        route: now_tree.route,
        status: PageStatus::Changed,
        own_file_changed,
        because,
        page_changed,
        counts,
        changes,
    };
    Some((diff, snippet_count))
}

/// The addresses of the snippets on both versions of a page whose code
/// differs between them, in order. A snippet only one version has was added
/// or removed in the page itself, which the page's own file says.
fn changed_snippets(was: &PageTree, now: &PageTree) -> Vec<String> {
    let snippets = |tree: &PageTree| {
        let mut out: BTreeMap<String, BTreeSet<u64>> = BTreeMap::new();
        for node in &tree.nodes {
            node.visit(&mut |n| {
                if let Some((address, hash)) = &n.snippet {
                    out.entry(address.clone()).or_default().insert(*hash);
                }
            });
        }
        out
    };
    let was = snippets(was);
    snippets(now)
        .into_iter()
        .filter(|(address, hashes)| was.get(address).is_some_and(|w| w != hashes))
        .map(|(address, _)| address)
        .collect()
}

fn to_change(found: Found<'_>) -> Change {
    let empty = Change {
        kind: ChangeKind::Changed,
        now: None,
        was: None,
        words: None,
        after: None,
        parent: None,
        text: None,
    };
    match found {
        Found::Changed { was, now } => {
            let words = (was.prose && now.prose && was.text != now.text)
                .then(|| diff_words(&was.text, &now.text))
                .flatten()
                .map(|w| Words {
                    now: w.now,
                    was: w.was,
                    now_text: now.text.clone(),
                    was_text: was.text.clone(),
                });
            Change {
                now: Some(now.anchor.clone()),
                was: Some(was.anchor.clone()),
                words,
                ..empty
            }
        }
        Found::Added { now } => Change {
            kind: ChangeKind::Added,
            now: Some(now.anchor.clone()),
            ..empty
        },
        Found::Removed { was, after, parent } => Change {
            kind: ChangeKind::Removed,
            was: Some(was.anchor.clone()),
            after,
            parent,
            text: Some(was.all_text()),
            ..empty
        },
        Found::Moved {
            was,
            now,
            after,
            parent,
        } => Change {
            kind: ChangeKind::Moved,
            now: Some(now.anchor.clone()),
            was: Some(was.anchor.clone()),
            after,
            parent,
            ..empty
        },
    }
}
