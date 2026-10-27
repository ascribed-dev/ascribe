//! Comparing two versions of a project, build by build and page by page.

use std::borrow::Cow;
use std::collections::{BTreeMap, BTreeSet, HashMap};

use ascribe_core::RelPath;
use ascribe_model::addresses::Key;
use ascribe_resolve::openapi::embedded_operations;
use ascribe_resolve::{AstroRouter, FileKind, Project, Resolution, ResolvedPage};
use serde::Serialize;

use crate::align::{self, Found};
use crate::tree::{Anchor, Node, PageTree, TreeBuilder, lf};
use crate::words::diff_words;

/// The name `because` gives the content model.
pub(crate) const MODEL_FILE: &str = "ascribe.toml";

/// Whether a name [`PageDiff::because`] gives is an operation's, a spec's
/// path from the project root, `#`, and a fragment
/// (`api/openapi.yaml#createWidget`): it has a `#`, and isn't a content
/// file's path, which ends in `.md`, or a snippet's address, whose source's
/// name and `:` come before its path.
pub(crate) fn names_operation(name: &str) -> bool {
    name.split_once('#')
        .is_some_and(|(spec, _)| !spec.contains(':') && !name.ends_with(".md"))
}

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

/// The source files git sees renamed between the base and the working
/// tree, each old content path with its new one, as [`crate::renames`]
/// lists them.
pub type Renames = BTreeMap<String, String>;

/// What changed on one page of a build.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub struct PageDiff {
    /// The page's content path.
    pub path: String,
    /// For a page that moved, the content path it was at: git sees its
    /// file renamed, or `ascribe.addresses` records the move. The page is
    /// listed once, at `path`, compared with the page as it was at
    /// `moved_from`; `status` is `changed`, and its `counts` and `changes`
    /// are what differs besides the move, none for a page that only moved.
    /// Absent for a page that didn't move.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub moved_from: Option<String>,
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
    /// (`code:app.py#main`); then the operations of a declared API whose spec
    /// changed what the page writes for them, an `@operation`'s method, path,
    /// summary, or deprecation, or a link's address, each by its spec's path
    /// from the project root, `#`, and the fragment the page names it by
    /// (`api/openapi.yaml#createWidget`); and `ascribe.toml` (last) when the
    /// content model is a cause. The kinds are told apart by their form: a
    /// content file's path ends in `.md`, a snippet's address has its
    /// source's name and `:` before its path, and an operation's has a `#`
    /// and neither.
    pub because: Vec<String>,
    /// What changed about the page itself besides its blocks, in this
    /// order: `title`, `frontmatter`, `availability` (the page-level one),
    /// and `route`. Empty for an added or removed page. A moved page's
    /// route moves with it, so it isn't listed for one.
    pub page_changed: Vec<&'static str>,
    /// The pages among `because` whose only change is that they moved, each
    /// by where it was and where it is: a link on this page leads to its new
    /// address. Absent when there's none.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub moved_links: Vec<MovedPage>,
    /// How many changes of each kind.
    pub counts: Counts,
    /// The block-level changes, in the page's order, a removed block where
    /// it was. Empty for an added or removed page, and for every page when
    /// the report's `blocks_omitted` is `true`.
    pub changes: Vec<Change>,
}

impl PageDiff {
    /// What changed on the page, in a few words: "2 changed, 1 added",
    /// "added", or "moved from guides/configure.md, no change to the page",
    /// and the files the change comes through, each written by `cause`:
    /// "(through _fragments/prereqs.md)". A page that changed only because a
    /// page it links to moved says so, "(a link changed because
    /// guides/configure.md moved)"; when its own file changed too, the move
    /// isn't named here, since that page is listed as moved.
    pub fn describe(&self, cause: impl Fn(&str) -> String) -> String {
        let mut text = self.summary_with(&cause);
        let through: Vec<String> = self
            .because
            .iter()
            .filter(|c| !self.moved_links.iter().any(|m| m.to == **c))
            .map(|c| cause(c))
            .collect();
        let mut reasons = Vec::new();
        if !through.is_empty() {
            let lead = if self.own_file_changed {
                "also through"
            } else {
                "through"
            };
            reasons.push(format!("{lead} {}", through.join(", ")));
        }
        if !self.own_file_changed && !self.moved_links.is_empty() {
            let moved: Vec<String> = self.moved_links.iter().map(|m| cause(&m.from)).collect();
            let moved: Vec<&str> = moved.iter().map(String::as_str).collect();
            let links = if moved.len() == 1 {
                "a link changed"
            } else {
                "links changed"
            };
            reasons.push(format!("{links} because {} moved", and_list(&moved)));
        }
        if !reasons.is_empty() {
            text.push_str(&format!(" ({})", reasons.join("; ")));
        }
        text
    }

    /// What changed on the page, without where from: "2 changed, 1 added,
    /// title changed", "added", or "moved from guides/configure.md, no
    /// change to the page".
    pub fn summary(&self) -> String {
        self.summary_with(&str::to_owned)
    }

    /// Whether this is the page at `path`, a content path: where it is, or
    /// where it moved from.
    pub fn is(&self, path: &str) -> bool {
        self.path == path || self.moved_from.as_deref() == Some(path)
    }

    /// [`PageDiff::summary`], with the path a page moved from written by
    /// `path`, as [`PageDiff::describe`] writes its causes.
    pub fn summary_with(&self, path: &dyn Fn(&str) -> String) -> String {
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
                if let Some(from) = &self.moved_from {
                    if parts.is_empty() {
                        parts.push("no change to the page".to_owned());
                    }
                    parts.insert(0, format!("moved from {}", path(from)));
                }
                parts.join(", ")
            }
        }
    }
}

/// A page that moved: where it was, and where it is, by content path.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
pub struct MovedPage {
    /// Where it was, in the base.
    pub from: String,
    /// Where it is now.
    pub to: String,
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

/// The files whose text differs between two versions, whether the content
/// model's does, and the pages that moved.
struct ChangedFiles {
    files: BTreeSet<RelPath>,
    model: bool,
    moves: Moves,
}

impl ChangedFiles {
    fn between(base: Option<Side<'_>>, now: Side<'_>, renames: &Renames) -> ChangedFiles {
        let Some(base) = base else {
            return ChangedFiles {
                files: now.project.files().map(|f| f.path.clone()).collect(),
                model: true,
                moves: Moves::default(),
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
            moves: Moves::between(base, now, renames),
        }
    }
}

/// The pages that moved between two versions: each page of the base whose
/// file isn't there now, paired with a page the base didn't have, as git
/// sees the file renamed (`renames`) or as the working tree's
/// `ascribe.addresses` records the move. Each page is paired once, git's
/// renames first.
#[derive(Default)]
struct Moves {
    /// Each moved page's path before, by its path now.
    from: BTreeMap<RelPath, RelPath>,
    /// Each moved page's path now, by its path before.
    to: BTreeMap<RelPath, RelPath>,
    /// The paths now of the moved pages whose file reads as it did, so that
    /// moving is all that changed about them.
    only_moved: BTreeSet<RelPath>,
}

impl Moves {
    fn between(base: Side<'_>, now: Side<'_>, renames: &Renames) -> Moves {
        let is_page = |project: &Project, path: &RelPath| {
            project.file(path).is_some_and(|f| f.kind == FileKind::Page)
        };
        let mut pairs: Vec<(String, String)> = renames
            .iter()
            .map(|(old, new)| (old.clone(), new.clone()))
            .collect();
        if let Some(file) = now.project.addresses_file() {
            pairs.extend(file.moved.iter().filter_map(|(key, entry)| {
                let Some(Key::Source(old)) = Key::parse(key) else {
                    return None;
                };
                Some((old.to_owned(), file.follow(&entry.value)?))
            }));
        }
        let mut moves = Moves::default();
        for (old, new) in pairs {
            let (Ok(old), Ok(new)) = (RelPath::parse(&old), RelPath::parse(&new)) else {
                continue;
            };
            let moved = is_page(base.project, &old)
                && now.project.file(&old).is_none()
                && is_page(now.project, &new)
                && base.project.file(&new).is_none();
            if !moved || moves.to.contains_key(&old) || moves.from.contains_key(&new) {
                continue;
            }
            let text = |project: &Project, path: &RelPath| {
                project.file(path).map(|f| lf(&f.source).into_owned())
            };
            if text(base.project, &old) == text(now.project, &new) {
                moves.only_moved.insert(new.clone());
            }
            moves.from.insert(new.clone(), old.clone());
            moves.to.insert(old, new);
        }
        moves
    }
}

/// What changed in each of `builds` (by name) between `base` and `now`. A
/// build that `base` doesn't have publishes every page as added; with no
/// `base` at all (the project didn't exist), so does every build. A name
/// that isn't a build of `now` is skipped.
///
/// A page that moved is listed once, where it is now, with
/// [`PageDiff::moved_from`]: one whose file git sees renamed (`renames`,
/// from [`crate::renames`]; empty where git isn't asked), or whose move
/// `now`'s `ascribe.addresses` records.
pub fn compare_builds(
    base: Option<Side<'_>>,
    now: Side<'_>,
    builds: &[&str],
    renames: &Renames,
) -> Vec<BuildDiff> {
    let changed = ChangedFiles::between(base, now, renames);
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
        let mut pages: Vec<PageDiff> = paths
            .into_iter()
            .filter(|path| reach.as_ref().is_none_or(|reach| reach.contains(*path)))
            .filter_map(|path| sides.compare(path, &changed))
            .collect();
        // An older line's pages are under its folder, as its output's are.
        if let Some(build) = now.project.model().build(name) {
            for page in &mut pages {
                page.route = line_route(now.project.model(), build, &page.route);
            }
        }
        out.push(BuildDiff {
            build: (*name).to_owned(),
            pages,
        });
    }
    out
}

/// A page's route in the line `build` is, from its route under the
/// project's base path: under the line's folder for an older line, and as
/// it is otherwise.
fn line_route(
    model: &ascribe_model::ContentModel,
    build: &ascribe_model::Build,
    route: &str,
) -> String {
    // The current line keeps the build's declared name and the site's base path.
    let current = model.build(build.declared_name());
    let (Some(_), Some(current)) = (build.line_folder(), current) else {
        return route.to_owned();
    };
    let site = ascribe_emit::line_base(model, current);
    let line = ascribe_emit::line_base(model, build);
    match route.strip_prefix(site.as_str()) {
        Some(rest) => format!("{line}{rest}"),
        None if route == site.trim_end_matches('/') => line.trim_end_matches('/').to_owned(),
        None => route.to_owned(),
    }
}

/// What changed on the page at `path` in the build `build` between `base`
/// and `now`, as [`compare_builds`] reports it; `None` when nothing did, the
/// page is in neither version, `now` has no such build, or the page moved
/// away from `path` (it's reported where it is now).
pub fn compare_page_in(
    base: Option<Side<'_>>,
    now: Side<'_>,
    build: &str,
    path: &RelPath,
    renames: &Renames,
) -> Option<PageDiff> {
    let changed = ChangedFiles::between(base, now, renames);
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
    let changed = ChangedFiles::between(base, now, &Renames::new());
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
    // A file that names an operation of an API whose spec differs: what it
    // writes from the spec can differ.
    let specs = base.project.apis().changed(now.project.apis());
    if !specs.is_empty() {
        for project in sides {
            for file in project.files() {
                let names_one = project
                    .resolutions(&file.path)
                    .iter()
                    .cloned()
                    .chain(embedded_operations(project, file))
                    .any(
                        |r| matches!(&r, Resolution::Operation { api, .. } if specs.contains(api)),
                    );
                if names_one {
                    touched.insert(&file.path);
                }
            }
        }
    }

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
        let base_page = |path: &RelPath| self.base.as_ref().and_then(|(_, r)| r.page(path));
        let now_page = self.now.1.page(path);
        // A page that moved, in a build that published it at both places, is
        // compared with where it was, and isn't reported there.
        if let Some(new) = changed.moves.to.get(path)
            && now_page.is_none()
            && self.now.1.page(new).is_some()
            && base_page(path).is_some()
        {
            return None;
        }
        let moved = changed
            .moves
            .from
            .get(path)
            .filter(|_| now_page.is_some())
            .and_then(|old| Some((old, base_page(old)?)));
        let (was_path, base_page) = match moved {
            Some((old, page)) => (old, Some(page)),
            None => (path, base_page(path)),
        };
        compare_page(
            path,
            was_path,
            base_page.as_ref().zip(self.base.as_ref().map(|(b, _)| *b)),
            now_page.as_ref().map(|p| (p, self.now.0)),
            changed,
        )
    }
}

/// What changed on a page, and how many of its `because` are snippets: the
/// page at `path` now, compared with the page at `was_path` in the base,
/// which is `path` unless it moved.
fn compare_page(
    path: &RelPath,
    was_path: &RelPath,
    base: Option<(&ResolvedPage, Side<'_>)>,
    now: Option<(&ResolvedPage, Side<'_>)>,
    changed: &ChangedFiles,
) -> Option<(PageDiff, usize)> {
    let moved_from = (was_path != path).then(|| was_path.to_string());
    let own_file_changed = if moved_from.is_some() {
        !changed.moves.only_moved.contains(path)
    } else {
        changed.files.contains(path)
    };
    let page_only = |status, route: &str| {
        let because = if !own_file_changed && changed.model {
            vec![MODEL_FILE.to_owned()]
        } else {
            Vec::new()
        };
        PageDiff {
            path: path.to_string(),
            moved_from: None,
            route: route.to_owned(),
            status,
            own_file_changed,
            because,
            page_changed: Vec::new(),
            moved_links: Vec::new(),
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
    if moved_from.is_none() && was_tree.hash == now_tree.hash && was_tree.route == now_tree.route {
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
        (
            "route",
            moved_from.is_none() && was_tree.route != now_tree.route,
        ),
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
    // A page that moved is named once, where it is now.
    let moves = &changed.moves;
    let causes: BTreeSet<&RelPath> = used
        .iter()
        .filter(|p| *p != path && *p != was_path && changed.files.contains(*p))
        .map(|p| moves.to.get(p).unwrap_or(p))
        .filter(|p| *p != path)
        .collect();
    let moved_links: Vec<MovedPage> = causes
        .iter()
        .filter(|p| moves.only_moved.contains(**p))
        .filter_map(|p| {
            Some(MovedPage {
                from: moves.from.get(*p)?.to_string(),
                to: p.to_string(),
            })
        })
        .collect();
    let mut because: Vec<String> = causes.iter().map(ToString::to_string).collect();
    // A snippet whose code differs: named by its address, as a fragment is
    // by its path.
    let snippets = changed_uses(&was_tree, &now_tree, |n| n.snippet.iter().collect());
    let snippet_count = snippets.len();
    because.extend(snippets);
    // An operation the spec changed, named by the spec and the fragment.
    because.extend(changed_uses(&was_tree, &now_tree, |n| {
        n.operations.iter().collect()
    }));
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
        moved_from,
        route: now_tree.route,
        status: PageStatus::Changed,
        own_file_changed,
        because,
        page_changed,
        moved_links,
        counts,
        changes,
    };
    Some((diff, snippet_count))
}

/// The names of what both versions of a page use, by `uses` of each node,
/// whose fingerprints differ between them, in order: the snippets whose code
/// differs, by address, or the operations whose spec writes them
/// differently. One only one version uses was added or removed in the page
/// itself, which the page's own file says.
fn changed_uses(
    was: &PageTree,
    now: &PageTree,
    uses: impl Fn(&Node) -> Vec<&(String, u64)>,
) -> Vec<String> {
    let used = |tree: &PageTree| {
        let mut out: BTreeMap<String, BTreeSet<u64>> = BTreeMap::new();
        for node in &tree.nodes {
            node.visit(&mut |n| {
                for (name, hash) in uses(n) {
                    out.entry(name.clone()).or_default().insert(*hash);
                }
            });
        }
        out
    };
    let was = used(was);
    used(now)
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
