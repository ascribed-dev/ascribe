//! The permalinks list, `ascribe.permalinks` (SPEC §7.5), against what the
//! content declares: `permalinks-list-invalid` when the list can't be read,
//! `permalinks-list-behind` at each permalink it doesn't have where it's
//! declared, and `permalink-removed` at each entry nothing declares any
//! more. A project without the list gets none of them.
//!
//! They hold for every build, since the list is the project's: a permalink
//! is declared in a source file whether or not a build publishes it.

use std::collections::BTreeMap;

use ascribe_core::{FileId, Issue, Location, diagnostics};
use ascribe_resolve::DeclaredPermalink;

use crate::project::PERMALINKS_FILE_ID;
use crate::{Diagnostic, Project};

/// What the list's checks found, by file: each removed entry and each
/// problem reading the list under the list's id, and each permalink the
/// list is behind on under the file it's declared in, each list in source
/// order. `index` is the project's source index; `file` gives the project's
/// id of one of its files.
pub(crate) fn check(
    project: &Project,
    index: &ascribe_resolve::Project,
    file: &dyn Fn(FileId) -> FileId,
) -> BTreeMap<FileId, Vec<Diagnostic>> {
    let mut issues: Vec<Issue> = Vec::new();
    match project.permalinks() {
        None => {}
        Some(Err(problems)) => issues.extend(problems.iter().cloned()),
        Some(Ok(list)) => {
            let declared = index.declared_permalinks();
            let places = DeclaredPermalink::places(&declared);
            let mut reported: Vec<&str> = Vec::new();
            for permalink in &declared {
                let name = permalink.name.as_str();
                if reported.contains(&name) {
                    continue;
                }
                let at = Location::new(file(permalink.at.file), permalink.at.span);
                let issue = match list.permalinks.get(name) {
                    Some(entry) if places[name].contains(&entry.place) => continue,
                    Some(entry) => Issue::new(diagnostics::PERMALINKS_LIST_BEHIND, at)
                        .with_variant("moved")
                        .with_arg("name", name.to_owned())
                        .with_arg("was", entry.place.clone()),
                    None => Issue::new(diagnostics::PERMALINKS_LIST_BEHIND, at)
                        .with_arg("name", name.to_owned()),
                };
                reported.push(name);
                issues.push(issue);
            }
            for (name, entry) in &list.permalinks {
                if !places.contains_key(name) {
                    issues.push(
                        Issue::new(
                            diagnostics::PERMALINK_REMOVED,
                            Location::new(PERMALINKS_FILE_ID, entry.span),
                        )
                        .with_arg("name", name.clone())
                        .with_arg("was", entry.place.clone()),
                    );
                }
            }
        }
    }
    let mut out: BTreeMap<FileId, Vec<Diagnostic>> = BTreeMap::new();
    for issue in &issues {
        out.entry(issue.location.file)
            .or_default()
            .push(Diagnostic::from_issue(issue));
    }
    for list in out.values_mut() {
        list.sort_by_key(|d| d.location.span);
    }
    out
}
