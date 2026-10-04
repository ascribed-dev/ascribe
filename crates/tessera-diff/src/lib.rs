//! What changed between a git revision and the working tree, as readers will
//! see it: which pages of which builds changed, and which blocks on them
//! were added, removed, changed, or moved.
//!
//! The comparison is of **resolved pages**, per build, not of files: a page
//! whose own file didn't change but whose fragment, phrase, or build settings
//! did is reported, with the files its change comes from. A change that
//! doesn't reach the resolved tree (reformatting, rewrapped lines) isn't.
//!
//! - [`git`]: running `git`, finding the repository and the base revision.
//! - [`GitFs`] and [`Revision`]: a project as it is at a revision, through
//!   the same [`tessera_resolve::FileSystem`] the disk and the editor use.
//! - [`compare_builds`]: the comparison, as a [`BuildDiff`] per build.
//!
//! The binary links no git library: everything goes through the `git`
//! executable, so nothing else in Ascribe depends on `git` being present.
//!
//! ```no_run
//! use std::path::Path;
//! use tessera_diff::{BuildDiff, DiffError, Repository, Revision, Side, compare_builds};
//!
//! fn diff(now: &tessera_resolve::Project, now_model: &str) -> Result<Vec<BuildDiff>, DiffError> {
//!     let repo = Repository::discover(Path::new("docs"))?;
//!     let base = repo.base(None, false)?;
//!     let before = Revision::read(&repo, base.compared())?;
//!     let before_project = before.as_ref().map(Revision::project);
//!     let before_side = before
//!         .as_ref()
//!         .zip(before_project.as_ref())
//!         .map(|(r, project)| Side { project, model_text: &r.model_text });
//!     let now_side = Side { project: now, model_text: now_model };
//!     Ok(compare_builds(before_side, now_side, &["site"]))
//! }
//! ```

mod align;
mod compare;
pub mod git;
mod gitfs;
mod tree;
mod words;

use serde::Serialize;

pub use compare::{
    BuildDiff, Change, ChangeKind, Counts, PageDiff, PageStatus, Side, Words, compare_builds,
};
pub use git::{Base, Repository};
pub use gitfs::{GitFs, Revision};
pub use tree::{Anchor, encode_path};

/// The version of the JSON report. It changes only when a field is removed
/// or changes meaning; fields can be added without a new version.
pub const SCHEMA_VERSION: u32 = 1;

/// Why a comparison couldn't run.
#[derive(Debug, thiserror::Error)]
pub enum DiffError {
    /// The `git` executable isn't on the path.
    #[error("git not found: ascribe diff runs `git`, so it needs git installed and on the path")]
    GitNotFound,
    /// The project isn't in a git repository.
    #[error("{dir} isn't in a git repository: {message}")]
    NotARepository {
        /// The directory.
        dir: String,
        /// What git said.
        message: String,
    },
    /// A revision doesn't name a commit.
    #[error("`{0}` isn't a revision of this repository")]
    UnknownRevision(String),
    /// No base was given and there's no default branch to use.
    #[error(
        "no base to compare with: none of origin/HEAD, origin/main, origin/master, main, and master exists; pass --base <REV>"
    )]
    NoDefaultBranch,
    /// The clone is too shallow to reach the merge base.
    #[error(
        "this clone doesn't have enough history to find where the branch left `{0}`: fetch more of it (in GitHub Actions, `fetch-depth: 0` on actions/checkout), or pass --base-exact to compare with `{0}` itself"
    )]
    ShallowHistory(String),
    /// The base and `HEAD` have no commit in common.
    #[error(
        "`{0}` and HEAD share no history, so there's no merge base to compare with; pass --base-exact to compare with `{0}` itself"
    )]
    NoCommonHistory(String),
    /// The content root is above the repository's root.
    #[error("the content root `{0}` is outside the git repository")]
    OutsideRepository(String),
    /// A path couldn't be made.
    #[error("{0}")]
    Path(String),
    /// The content model at the base revision has errors.
    #[error("ascribe.toml at {commit} has errors")]
    BaseModel {
        /// The revision.
        commit: String,
        /// The content model's text there.
        text: String,
        /// The problems.
        issues: Vec<tessera_core::Issue>,
    },
    /// The content model at the base revision isn't UTF-8.
    #[error("ascribe.toml at {commit} isn't valid UTF-8")]
    BaseModelText {
        /// The revision.
        commit: String,
    },
    /// A git command failed.
    #[error("`{command}` failed: {message}")]
    Git {
        /// The command.
        command: String,
        /// What it printed, or why it couldn't run.
        message: String,
    },
}

/// The whole report, as `ascribe diff --format json` writes it.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Report {
    /// [`SCHEMA_VERSION`].
    pub schema_version: u32,
    /// The version of Ascribe that wrote it.
    pub ascribe_version: &'static str,
    /// What was compared with.
    pub base: BaseInfo,
    /// Where the project is.
    pub repository: RepositoryInfo,
    /// What changed, per build, in the order asked for.
    pub builds: Vec<BuildDiff>,
}

/// The base of a comparison, in the report.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct BaseInfo {
    /// The revision asked for, or the default branch used.
    pub requested: String,
    /// The commit it names.
    pub commit: String,
    /// The merge base of that commit and `HEAD`, which the comparison reads;
    /// `null` with `--base-exact`, which reads `commit`.
    pub merge_base: Option<String>,
}

/// The repository, in the report.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct RepositoryInfo {
    /// The repository's top-level directory, as git prints it.
    pub root: String,
    /// The project's folder inside it, with a trailing `/`, or empty when
    /// the project is at the repository's root.
    pub project_prefix: String,
}

impl Report {
    /// A report of `builds`, compared with `base` in `repo`.
    pub fn new(repo: &Repository, base: &Base, builds: Vec<BuildDiff>) -> Report {
        Report {
            schema_version: SCHEMA_VERSION,
            ascribe_version: env!("CARGO_PKG_VERSION"),
            base: BaseInfo {
                requested: base.requested.clone(),
                commit: base.commit.clone(),
                merge_base: base.merge_base.clone(),
            },
            repository: RepositoryInfo {
                root: repo.root.clone(),
                project_prefix: repo.prefix.clone(),
            },
            builds,
        }
    }

    /// Whether anything changed in any build.
    pub fn has_changes(&self) -> bool {
        self.builds.iter().any(|b| !b.pages.is_empty())
    }
}
