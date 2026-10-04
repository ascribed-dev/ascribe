//! Comparing two versions of a project, build by build and page by page.

use std::borrow::Cow;
use std::collections::{BTreeMap, BTreeSet};

use serde::Serialize;
use tessera_core::RelPath;
use tessera_resolve::{AstroRouter, Project, ResolvedPage};

use crate::align::{self, Found};
use crate::tree::{Anchor, Node, PageTree, TreeBuilder, lf};
use crate::words::diff_words;

/// The name `because` gives the content model.
const MODEL_FILE: &str = "ascribe.toml";

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
pub struct BuildDiff {
    /// The build's name.
    pub build: String,
    /// The pages that changed, in path order.
    pub pages: Vec<PageDiff>,
}

/// What changed on one page of a build.
#[derive(Clone, Debug, PartialEq, Serialize)]
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
    /// The other changed files the page's change can come from, in path
    /// order: fragments it includes, pages its links take a title or a
    /// heading from, and `ascribe.toml` (last) when the content model is a
    /// cause.
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

/// Whether a page is new, gone, or different.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
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
    let now_router = AstroRouter::from_consumer(&now.project.model().consumer);
    let base_router = base.map(|b| AstroRouter::from_consumer(&b.project.model().consumer));
    let mut out = Vec::new();
    for name in builds {
        let Some(now_build) = now.project.model().build(name) else {
            continue;
        };
        let now_resolver = now.project.resolver(now_build, &now_router);
        let base_resolver = match (base, &base_router) {
            (Some(b), Some(router)) => b
                .project
                .model()
                .build(name)
                .map(|build| (b, b.project.resolver(build, router))),
            _ => None,
        };
        let mut paths: BTreeSet<&RelPath> = now.project.pages().map(|p| &p.path).collect();
        if let Some((b, _)) = &base_resolver {
            paths.extend(b.project.pages().map(|p| &p.path));
        }
        let mut pages = Vec::new();
        for path in paths {
            let now_page = now_resolver.page(path);
            let base_page = base_resolver.as_ref().and_then(|(_, r)| r.page(path));
            let diff = compare_page(
                path,
                base_page
                    .as_ref()
                    .zip(base_resolver.as_ref().map(|(b, _)| *b)),
                now_page.as_ref().map(|p| (p, now)),
                &changed,
            );
            pages.extend(diff);
        }
        out.push(BuildDiff {
            build: (*name).to_owned(),
            pages,
        });
    }
    out
}

fn compare_page(
    path: &RelPath,
    base: Option<(&ResolvedPage, Side<'_>)>,
    now: Option<(&ResolvedPage, Side<'_>)>,
    changed: &ChangedFiles,
) -> Option<PageDiff> {
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
        (None, Some((page, _))) => return Some(page_only(PageStatus::Added, &page.route)),
        (Some((page, _)), None) => return Some(page_only(PageStatus::Removed, &page.route)),
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
    Some(PageDiff {
        path: path.to_string(),
        route: now_tree.route,
        status: PageStatus::Changed,
        own_file_changed,
        because,
        page_changed,
        counts,
        changes,
    })
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
