//! The glossary against the content (SPEC §5.4): a term's `link` that names
//! a heading its page doesn't have (`glossary-link-target-missing`), on the
//! `link` in `ascribe.toml`. The loader checks that the page is there
//! (`model-glossary-link`); a heading's id depends on parsing, so it's
//! checked here.
//!
//! It holds for every build, since the ids are the source's: a heading's
//! source id doesn't depend on a build.

use std::collections::BTreeMap;

use ascribe_core::{FileId, Issue, Location, RelPath, diagnostics};

use crate::{Diagnostic, Project};

/// The id of `ascribe.toml`, where the terms are.
const MODEL_FILE_ID: FileId = FileId::new(0);

/// What the check found, under `ascribe.toml`'s id, in source order.
/// `index` is the project's source index.
pub(crate) fn check(
    project: &Project,
    index: &ascribe_resolve::Project,
) -> BTreeMap<FileId, Vec<Diagnostic>> {
    let mut found = Vec::new();
    for term in &project.model().glossary.terms {
        let (Some((path, Some(heading))), Some(span)) = (&term.link, term.link_span) else {
            continue;
        };
        // A page the loader couldn't find has its own problem.
        let Ok(page) = RelPath::parse(path) else {
            continue;
        };
        if index.file(&page).is_none() || index.page_heading(&page, heading).is_some() {
            continue;
        }
        let issue = Issue::new(
            diagnostics::GLOSSARY_LINK_TARGET_MISSING,
            Location::new(MODEL_FILE_ID, span),
        )
        .with_arg("id", term.id.clone())
        .with_arg("link", format!("{path}#{heading}"))
        .with_arg("path", path.clone())
        .with_arg("heading", heading.clone());
        found.push(Diagnostic::from_issue(&issue));
    }
    found.sort_by_key(|d| d.location.span);
    let mut out = BTreeMap::new();
    if !found.is_empty() {
        out.insert(MODEL_FILE_ID, found);
    }
    out
}
