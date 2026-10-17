//! The glossary against the content (SPEC §5.4): two terms, or a term and
//! an alias, with the same text (`glossary-term-duplicate`), reported at the
//! second, naming the first. Matching can't choose between them, so the
//! second is left out of the glossary.
//!
//! It holds for every build, since a term is the source's: its heading's
//! text and its `@term` don't depend on a build.

use std::collections::BTreeMap;

use ascribe_core::{FileId, Issue, Location, diagnostics};

use crate::Diagnostic;

/// What the check found, by file, in source order. `index` is the project's
/// source index; `file` gives the project's id of one of its files.
pub(crate) fn check(
    index: &ascribe_resolve::Project,
    file: &dyn Fn(FileId) -> FileId,
) -> BTreeMap<FileId, Vec<Diagnostic>> {
    let mut out: BTreeMap<FileId, Vec<Diagnostic>> = BTreeMap::new();
    for duplicate in index.glossary().duplicates {
        let issue = Issue::new(
            diagnostics::GLOSSARY_TERM_DUPLICATE,
            Location::new(file(duplicate.file), duplicate.span),
        )
        .with_arg("text", duplicate.text)
        .with_arg("first", duplicate.first)
        .with_arg("page", duplicate.first_page.to_string());
        out.entry(issue.location.file)
            .or_default()
            .push(Diagnostic::from_issue(&issue));
    }
    for list in out.values_mut() {
        list.sort_by_key(|d| d.location.span);
    }
    out
}
