//! The addresses a base revision published, for `ascribe check` to compare
//! with the working tree's (`ascribe_check::addresses`), which can't read
//! git itself. The base is chosen as `ascribe diff` chooses it.

use std::path::Path;

use ascribe_resolve::addresses::{BuildAddresses, every_build};

use crate::{Base, DiffError, Repository, Revision};

/// The addresses a base revision published.
#[derive(Clone, Debug)]
pub struct BaseAddresses {
    /// The base.
    pub base: Base,
    /// Each build's addresses there, in its content model's order; `None`
    /// when there's no project at the base.
    pub builds: Option<Vec<BuildAddresses>>,
}

impl BaseAddresses {
    /// The base as a message names it: the revision asked for, and its
    /// commit when that's something else.
    pub fn named(&self) -> String {
        let short = &self.base.commit[..self.base.commit.len().min(12)];
        if self.base.requested.starts_with(short) {
            format!("`{}`", self.base.requested)
        } else {
            format!("`{}` ({short})", self.base.requested)
        }
    }
}

/// The addresses the project at `root` published at the base: `requested`,
/// or the repository's default branch; its merge base with `HEAD` unless
/// `exact`. Listed with the library, with no output written.
///
/// # Errors
///
/// The base can't be read: no repository, no git, a revision that isn't
/// there, or no merge base.
pub fn base_addresses(
    root: &Path,
    requested: Option<&str>,
    exact: bool,
) -> Result<BaseAddresses, DiffError> {
    let repo = Repository::discover(root)?;
    let base = repo.base(requested, exact)?;
    let builds = Revision::read(&repo, base.compared())?.map(|r| every_build(&r.project()));
    Ok(BaseAddresses { base, builds })
}
