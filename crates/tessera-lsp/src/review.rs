//! Review: the project compared with a git revision, its base.
//!
//! - `ascribe/review/setBase` resolves a revision as `ascribe diff` does (the
//!   merge base of it and `HEAD`), reads the project as it is there through
//!   [`Revision`], and keeps it beside the live snapshot. `null` drops it.
//! - `ascribe/review/changes` lists the pages of a build that differ from the
//!   base, computed against the current snapshot, so unsaved edits count.
//! - `ascribe/preview` with `review: true` adds the page's changes and the
//!   page as it was (`preview.rs`).
//!
//! The base is read when it's set, and not watched. Setting it again
//! resolves the revision again, which is cheap, and reads the project there
//! only when the commit compared with moved (a pull, rebase, or fetch since):
//! the editor does that when the preview regains focus. Nothing here runs
//! `git` until a base is set, so a server without `git` on the path works as
//! before, and `setBase` says why it can't.
//!
//! The base never changes once read, so it keeps what's computed from it: the
//! pages as they were, by build and path, and the last list of changed pages,
//! with the snapshot it was computed from.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, PoisonError};

use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value as Json;
use tessera_core::RelPath;
use tessera_diff::{Base, BaseInfo, DiffError, PageDiff, Repository, Revision, Side};
use tessera_model::ContentModel;
use tessera_resolve::{AstroRouter, Project, Snapshot, Version};

use crate::core::Core;
use crate::uri::normalize;

/// The method that sets or drops the base.
pub const SET_BASE_METHOD: &str = "ascribe/review/setBase";

/// The method that lists the changed pages.
pub const CHANGES_METHOD: &str = "ascribe/review/changes";

/// The parameters of `ascribe/review/setBase`.
#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetBaseParams {
    /// A revision (a branch, tag, or commit) to compare with, from where the
    /// branch left it; `null` to stop comparing and drop the base. Without
    /// one, the default branch: the first of `origin/HEAD`, `origin/main`,
    /// `origin/master`, `main`, and `master` that exists, as `ascribe diff`
    /// picks it.
    #[serde(default, deserialize_with = "present")]
    pub base: Option<Option<String>>,
}

/// `Some(None)` for `null`, so an absent field (`None`) can mean something
/// else.
fn present<'de, D: Deserializer<'de>>(d: D) -> Result<Option<Option<String>>, D::Error> {
    Option::<String>::deserialize(d).map(Some)
}

/// The answer to `ascribe/review/setBase`.
#[derive(Debug, Default, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SetBaseResult {
    /// What the base resolved to; `null` once it's dropped, or when it
    /// couldn't be set.
    pub base: Option<BaseInfo>,
    /// Why the base couldn't be set: no project, not a repository, an
    /// unknown revision, `git` missing. The base set before, if any, stays.
    pub problem: Option<String>,
}

/// The parameters of `ascribe/review/changes`.
#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChangesParams {
    /// The build. Without one, the editor's build (`[editor] build`).
    #[serde(default)]
    pub build: Option<String>,
}

/// The answer to `ascribe/review/changes`.
#[derive(Clone, Debug, Default, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ChangesResult {
    /// The build the pages are of.
    pub build: String,
    /// The base compared with; `null` when there is none.
    pub base: Option<BaseInfo>,
    /// The content root, as a path, which the pages' paths are relative to.
    pub content_root: Option<String>,
    /// The changed pages, in path order: `ascribe diff`'s pages without their
    /// `changes`, each with its `title`.
    pub pages: Vec<Json>,
    /// Why there are no pages to list, when that isn't because nothing
    /// changed.
    pub problem: Option<String>,
}

/// A base: what it resolved to, and the project as it was there.
pub(crate) struct ReviewBase {
    pub info: BaseInfo,
    /// The project's source index at the base; `None` when the project
    /// didn't exist there, so every page is new.
    pub project: Option<Project>,
    /// The content model's text there.
    pub model_text: String,
    /// The pages as they were, rendered, by build and path; `None` for a page
    /// the build didn't have.
    pages: Mutex<HashMap<(String, RelPath), Option<String>>>,
    /// The last answer to `ascribe/review/changes`, and what it was computed
    /// from.
    changes: Mutex<Option<(ChangesKey, ChangesResult)>>,
}

/// What a list of changed pages depends on besides the base: the loaded
/// project (its epoch), the snapshot's version and model revision, and the
/// build.
#[derive(Clone, Debug, PartialEq, Eq)]
struct ChangesKey {
    epoch: u64,
    version: Version,
    model_revision: u64,
    build: String,
}

impl ReviewBase {
    /// The base's side of a comparison.
    pub fn side(&self) -> Option<Side<'_>> {
        self.project.as_ref().map(|project| Side {
            project,
            model_text: &self.model_text,
        })
    }

    /// The page at `path` of `build` as it was, rendered with anchors; `None`
    /// when the build didn't have it.
    pub fn page_html(&self, build: &str, path: &RelPath) -> Option<String> {
        let key = (build.to_owned(), path.clone());
        let mut pages = self.pages.lock().unwrap_or_else(PoisonError::into_inner);
        pages
            .entry(key)
            .or_insert_with(|| {
                tessera_diff::html::page_html(self.project.as_ref()?, build, path)
                    .map(|(html, _)| html)
            })
            .clone()
    }
}

/// Resolves the base `requested` (the default branch when `None`) for the
/// project whose `ascribe.toml` is in `root`: the commit to compare with,
/// without reading the project there.
pub(crate) fn resolve_base(
    root: &Path,
    requested: Option<&str>,
) -> Result<(Repository, Base), String> {
    let repo = Repository::discover(root).map_err(|e| explain(&e))?;
    let base = repo.base(requested, false).map_err(|e| explain(&e))?;
    Ok((repo, base))
}

/// What a resolved base is, in answers.
pub(crate) fn info_of(base: &Base) -> BaseInfo {
    BaseInfo {
        requested: base.requested.clone(),
        commit: base.commit.clone(),
        merge_base: base.merge_base.clone(),
    }
}

/// Reads the project at a resolved base.
pub(crate) fn read_base(repo: &Repository, base: &Base) -> Result<ReviewBase, String> {
    let revision = Revision::read(repo, base.compared()).map_err(|e| explain(&e))?;
    let (project, model_text) = match revision {
        Some(revision) => (Some(revision.project()), revision.model_text),
        None => (None, String::new()),
    };
    Ok(ReviewBase {
        info: info_of(base),
        project,
        model_text,
        pages: Mutex::default(),
        changes: Mutex::default(),
    })
}

/// Why a base can't be set, in the editor's terms: `ascribe diff`'s messages
/// name its command-line options.
fn explain(error: &DiffError) -> String {
    match error {
        DiffError::GitNotFound => {
            "Review compares the project with a git revision, and git isn't on the path. Install git, or add it to the path, and start the review again.".to_owned()
        }
        DiffError::NotARepository { dir, .. } => format!(
            "Review compares the project with a git revision, and {dir} isn't in a git repository."
        ),
        DiffError::UnknownRevision(rev) => {
            format!("`{rev}` isn't a branch, tag, or commit of this repository.")
        }
        DiffError::NoDefaultBranch => {
            "There's no default branch to compare with: none of origin/HEAD, origin/main, origin/master, main, and master exists. Start the review again and type a revision.".to_owned()
        }
        DiffError::ShallowHistory(rev) => format!(
            "This clone doesn't have enough history to find where the branch left `{rev}`. Fetch more of it (`git fetch --unshallow`) and start the review again."
        ),
        DiffError::NoCommonHistory(rev) => format!(
            "`{rev}` and HEAD share no history, so there's no point where the branch left it to compare with."
        ),
        other => other.to_string(),
    }
}

/// What the changes request needs from the server's state, taken under the
/// lock so the comparison happens without it.
pub(crate) struct ChangesTarget {
    epoch: u64,
    snapshot: Snapshot,
    model: Arc<ContentModel>,
    model_text: String,
    content_root: PathBuf,
    review: Option<Arc<ReviewBase>>,
}

impl Core {
    /// The project's root, when a project has loaded.
    pub(crate) fn project_root(&self) -> Option<PathBuf> {
        self.loaded.as_ref().map(|l| l.root.clone())
    }

    /// Takes what `ascribe/review/changes` needs.
    pub(crate) fn changes_target(&self) -> Option<ChangesTarget> {
        let loaded = self.loaded.as_ref()?;
        Some(ChangesTarget {
            epoch: loaded.epoch,
            snapshot: loaded.inc.snapshot(),
            model: loaded.model.clone(),
            model_text: loaded.model_text.clone(),
            content_root: normalize(&loaded.root.join(loaded.layout.content_root.as_str())),
            review: self.review.clone(),
        })
    }
}

/// Answers `ascribe/review/changes`.
pub(crate) fn changes(target: Option<&ChangesTarget>, build_name: Option<&str>) -> ChangesResult {
    let Some(target) = target else {
        return ChangesResult {
            problem: Some("There is no project loaded, so there are no changed pages.".to_owned()),
            ..ChangesResult::default()
        };
    };
    let model = &*target.model;
    let build = match build_name {
        None => model.editor_default_build(),
        Some(name) => match model.build(name) {
            Some(build) => build,
            None => {
                return ChangesResult {
                    build: name.to_owned(),
                    problem: Some(format!("The content model has no build named {name}.")),
                    ..ChangesResult::default()
                };
            }
        },
    };
    let mut result = ChangesResult {
        build: build.name.clone(),
        content_root: Some(target.content_root.to_string_lossy().into_owned()),
        ..ChangesResult::default()
    };
    let Some(review) = &target.review else {
        result.problem = Some("Review is off for this project.".to_owned());
        return result;
    };
    result.base = Some(review.info.clone());
    let key = ChangesKey {
        epoch: target.epoch,
        version: target.snapshot.version(),
        model_revision: target.snapshot.model_revision(),
        build: build.name.clone(),
    };
    let mut last = review
        .changes
        .lock()
        .unwrap_or_else(PoisonError::into_inner);
    if let Some((computed, answer)) = &*last
        && *computed == key
    {
        return answer.clone();
    }
    let now = Side {
        project: target.snapshot.project(),
        model_text: &target.model_text,
    };
    let diffs = tessera_diff::compare_builds(review.side(), now, &[build.name.as_str()]);
    let router = AstroRouter::from_consumer(&model.consumer);
    let base_router = review
        .project
        .as_ref()
        .map(|p| AstroRouter::from_consumer(&p.model().consumer));
    for diff in diffs.into_iter().flat_map(|b| b.pages) {
        let path = RelPath::parse(&diff.path).ok();
        let title = path.as_ref().and_then(|path| {
            let now = target
                .snapshot
                .resolve_page(path, build, &router)
                .and_then(|p| p.title.clone());
            now.or_else(|| {
                let project = review.project.as_ref()?;
                let build = project.model().build(&build.name)?;
                project
                    .resolve_page(path, build, base_router.as_ref()?)
                    .and_then(|p| p.title)
            })
        });
        result.pages.push(summary(diff, title));
    }
    *last = Some((key, result.clone()));
    result
}

/// A changed page as the list shows it: `ascribe diff`'s page without its
/// block changes, and its title.
fn summary(diff: PageDiff, title: Option<String>) -> Json {
    let mut value = serde_json::to_value(diff).unwrap_or(Json::Null);
    if let Json::Object(map) = &mut value {
        map.remove("changes");
        map.insert("title".to_owned(), title.map_or(Json::Null, Json::String));
    }
    value
}
