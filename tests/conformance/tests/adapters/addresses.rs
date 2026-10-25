//! The adapter for the addresses the base published against the working
//! tree's (tag `addresses`, SPEC §7.10).
//!
//! A case is a project case with a `base/` beside `files/`: the base's
//! content, in its own `base/files/`, under the case's content model, or
//! `base/ascribe.toml` when the base's differs. Its diagnostics are what
//! `ascribe check` adds when it compares them: each build's addresses at the
//! base and now (`ascribe_resolve::addresses`), compared by
//! `ascribe_check::addresses::compare` with the case's `ascribe.addresses`.
//! A case without `base/` has a base with no project, which published
//! nothing. A case expects them in its top-level `diagnostics`. Reading the base
//! from git, and a base it can't read, are tested with the command, in
//! `crates/ascribe-cli/tests/all/addresses.rs`.

use ascribe_check::Project;
use ascribe_conformance::{AdapterError, AdapterResult, Case, ConformanceAdapter, Diagnostic};
use ascribe_core::RelPath;
use ascribe_resolve::addresses::every_build;

use super::check::{project, to_conformance};

/// Handles the cases about the addresses the base published.
pub struct AddressesAdapter;

impl ConformanceAdapter for AddressesAdapter {
    fn name(&self) -> &str {
        "addresses"
    }

    fn handles_tag(&self, tag: &str) -> bool {
        tag == "addresses"
    }

    fn diagnostics(&self, case: &Case) -> AdapterResult<Vec<Diagnostic>> {
        let now = project(case)?;
        let file = match now.addresses() {
            Some(Ok(file)) => file,
            Some(Err(_)) => return Err(AdapterError("ascribe.addresses isn't valid".to_owned())),
            None => return Err(AdapterError("the case has no ascribe.addresses".to_owned())),
        };
        // A case without `base/` is one whose base has no project.
        if !case.dir.join("base").is_dir() {
            let found = vec![ascribe_check::addresses::base_has_no_project("the base")];
            return to_conformance(&now, found).map(Some);
        }
        let base = base(case)?;
        let index = now.index();
        let found = ascribe_check::addresses::compare(
            file,
            &index,
            &every_build(&base.index()),
            &every_build(&index),
        );
        to_conformance(&now, found).map(Some)
    }
}

/// The project in the case's `base/`.
fn base(case: &Case) -> Result<Project, AdapterError> {
    let dir = case.dir.join("base");
    let own = dir.join("ascribe.toml");
    let model_path = if own.is_file() {
        own
    } else {
        case.model.clone()
    };
    let text = std::fs::read_to_string(&model_path)
        .map_err(|e| AdapterError(format!("couldn't read {}: {e}", model_path.display())))?;
    let model = super::model::load_model(case, &text)
        .map_err(|_| AdapterError(format!("{} doesn't load", model_path.display())))?;
    let content_root = RelPath::parse("files").map_err(|e| AdapterError(e.to_string()))?;
    let sources =
        Project::read_sources(&dir, &content_root).map_err(|e| AdapterError(e.to_string()))?;
    Ok(Project::from_parts(dir, content_root, model, text, sources))
}
