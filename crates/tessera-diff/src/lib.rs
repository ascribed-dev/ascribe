//! What changed between a git revision and the working tree, as readers will
//! see it.
//!
//! - [`git`]: running `git`, finding the repository and the base revision.
//! - [`GitFs`] and [`Revision`]: a project as it is at a revision, through
//!   the same [`tessera_resolve::FileSystem`] the disk and the editor use.
//!
//! The binary links no git library: everything goes through the `git`
//! executable, so nothing else in Ascribe depends on `git` being present.

pub mod git;
mod gitfs;

pub use git::{Base, Repository};
pub use gitfs::{GitFs, Revision};

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
        "no base to compare with: none of origin/HEAD, main, and master exists; pass --base <REV>"
    )]
    NoDefaultBranch,
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
