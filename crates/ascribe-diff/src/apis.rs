//! The specs a base revision's project declared, for `ascribe check`, the
//! editor, and `ascribe link` to know where each `operationId` was, so a
//! link to one renamed since is offered the operation now at its method and
//! path (`ascribe_resolve::openapi::ApiSpecs::renamed`). The base is chosen
//! as `ascribe diff` chooses it.

use std::path::Path;

use ascribe_resolve::openapi::ApiSpecs;

use crate::{Repository, Revision};

/// The specs the project at `root` declared at the base: `requested`, or
/// the repository's default branch; its merge base with `HEAD` unless
/// `exact`. Only the specs are read, through the revision's files, and none
/// of its pages are resolved.
///
/// `None` when there's no base to read: no repository or no git, a revision
/// that isn't there, no merge base, as in a shallow clone, or no project at
/// the base. A link to a missing operation is then offered the closest, as
/// without history, and nothing is reported.
pub fn base_apis(root: &Path, requested: Option<&str>, exact: bool) -> Option<ApiSpecs> {
    let repo = Repository::discover(root).ok()?;
    let base = repo.base(requested, exact).ok()?;
    let revision = Revision::read(&repo, base.compared()).ok()??;
    Some(ApiSpecs::read(&revision.model, &*revision.fs))
}
