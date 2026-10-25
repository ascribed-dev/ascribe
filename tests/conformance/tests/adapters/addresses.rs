//! The adapter for the addresses the base published against the working
//! tree's (tag `addresses`, SPEC §7.10).
//!
//! A case is a project case with a `base/` beside `files/`: the base's
//! content, in its own `base/files/`, under the case's content model, or
//! `base/ascribe.toml` when the base's differs, and the base's own
//! `base/ascribe.addresses` when it had one. `base/renames.txt` lists the
//! renames git would see, a line `<old> -> <new>` each. Its diagnostics are what
//! `ascribe check` adds when it compares them: each build's addresses at the
//! base and now (`ascribe_resolve::addresses`), compared by
//! `ascribe_check::addresses::compare` with the case's `ascribe.addresses`,
//! and the file's own checks, `ascribe_check::addresses::entries`.
//! A case without `base/` has a base with no project, which published
//! nothing. A case expects them in its top-level `diagnostics`. Reading the base
//! from git, and a base it can't read, are tested with the command, in
//! `crates/ascribe-cli/tests/all/addresses.rs`.

use std::collections::BTreeMap;

use ascribe_check::Project;
use ascribe_check::addresses::{Base, Here};
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
        let (file, text) = match (now.addresses(), now.addresses_text()) {
            (Some(Ok(file)), Some(text)) => (file, text),
            (Some(Err(_)), _) => {
                return Err(AdapterError("ascribe.addresses isn't valid".to_owned()));
            }
            // Without one, the base's says what deleting it dropped.
            _ => {
                let entries = base(case)?
                    .addresses()
                    .and_then(|read| read.as_ref().ok())
                    .map(|file| file.moved.len() + file.retired.len())
                    .ok_or_else(|| {
                        AdapterError("the case has no ascribe.addresses, nor its base".to_owned())
                    })?;
                let found = vec![ascribe_check::addresses::file_deleted(entries)];
                return to_conformance(&now, found).map(Some);
            }
        };
        let index = now.index();
        let builds = every_build(&index);
        let here = Here {
            file,
            text,
            index: &index,
            builds: &builds,
        };
        // A case without `base/` is one whose base has no project.
        if !case.dir.join("base").is_dir() {
            let mut found = vec![ascribe_check::addresses::base_has_no_project("the base")];
            found.extend(ascribe_check::addresses::entries(&here, None));
            return to_conformance(&now, found).map(Some);
        }
        let mut base = Base::of(&base(case)?.index());
        base.renames = renames(case)?;
        let mut found = ascribe_check::addresses::compare(&here, &base);
        found.extend(ascribe_check::addresses::entries(&here, Some(&base)));
        to_conformance(&now, found).map(Some)
    }
}

/// The renames git would see, from the case's `base/renames.txt`: a line
/// `<old> -> <new>` for each, as content paths.
fn renames(case: &Case) -> Result<BTreeMap<String, String>, AdapterError> {
    let path = case.dir.join("base").join("renames.txt");
    if !path.is_file() {
        return Ok(BTreeMap::new());
    }
    let text = std::fs::read_to_string(&path)
        .map_err(|e| AdapterError(format!("couldn't read {}: {e}", path.display())))?;
    Ok(text
        .lines()
        .filter_map(|line| line.split_once(" -> "))
        .map(|(old, new)| (old.trim().to_owned(), new.trim().to_owned()))
        .collect())
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
