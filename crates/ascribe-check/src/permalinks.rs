//! The permalinks list, `ascribe.permalinks` (SPEC §7.5), against what the
//! project has: `permalinks-list-invalid` when the list can't be read,
//! `permalinks-list-behind` at each name it doesn't record as the project
//! has it, and `permalink-lost` at each entry the project doesn't have any
//! more. A project without the list gets none of them. What
//! `ascribe.addresses` says of a permalink, `go:<name>`, is checked with the
//! file's other entries (`addresses::entries`).
//!
//! They hold for every build, since the names are the project's: a
//! permalink is declared in a source file whether or not a build publishes
//! it.

use std::collections::BTreeMap;

use ascribe_core::{FileId, Issue, Location, diagnostics};
use ascribe_model::{Current, PermalinkList, RecordedPermalinks};
use ascribe_resolve::DeclaredPermalink;

use crate::project::{ADDRESSES_FILE_ID, PERMALINKS_FILE_ID};
use crate::{Diagnostic, Project};

/// What the checks found, by file: each lost entry and each problem
/// reading the list under the list's id, each name `ascribe.addresses`
/// records that the list is behind on under the file's id, and each
/// permalink the list is behind on under the file it's declared in, each
/// list in source order. `index` is the project's source index.
pub(crate) fn check(
    project: &Project,
    index: &ascribe_resolve::Project,
) -> BTreeMap<FileId, Vec<Diagnostic>> {
    let mut issues: Vec<Issue> = Vec::new();
    match project.permalinks() {
        None => {}
        Some(Err(problems)) => issues.extend(problems.iter().cloned()),
        Some(Ok(list)) => {
            let declared = index.declared_permalinks();
            let recorded = index.recorded_permalinks();
            let current = Current::new(DeclaredPermalink::places(&declared), &recorded);
            against_list(list, &current, &recorded, &declared, &mut issues);
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

/// The list against what the project has: each declared name, and each
/// name `ascribe.addresses` moves or retires, recorded as it is, and each
/// entry still had.
fn against_list(
    list: &PermalinkList,
    current: &Current,
    recorded: &RecordedPermalinks,
    declared: &[DeclaredPermalink],
    out: &mut Vec<Issue>,
) {
    let mut reported: Vec<&str> = Vec::new();
    for permalink in declared {
        let name = permalink.name.as_str();
        if reported.contains(&name) {
            continue;
        }
        let at = Location::new(permalink.at.file, permalink.at.span);
        let issue = match list.permalinks.get(name) {
            Some(entry) if current.declared[name].contains(&entry.place) => continue,
            Some(entry) => Issue::new(diagnostics::PERMALINKS_LIST_BEHIND, at)
                .with_variant("moved")
                .with_arg("name", name.to_owned())
                .with_arg("was", entry.place.clone()),
            None => Issue::new(diagnostics::PERMALINKS_LIST_BEHIND, at)
                .with_arg("name", name.to_owned()),
        };
        reported.push(name);
        out.push(issue);
    }
    for entry in &recorded.names {
        let name = entry.name.as_str();
        let listed = if let Some(target) = current.aliases.get(name) {
            list.aliases.get(name).is_some_and(|e| e.target == *target)
        } else if let Some(place) = current.moved.get(name) {
            list.moved.get(name).is_some_and(|e| e.place == *place)
        } else if let Some(reason) = current.retired.get(name) {
            list.retired.get(name).is_some_and(|e| e.reason == *reason)
        } else {
            continue;
        };
        if !listed {
            out.push(
                Issue::new(
                    diagnostics::PERMALINKS_LIST_BEHIND,
                    Location::new(ADDRESSES_FILE_ID, entry.span),
                )
                .with_variant("recorded")
                .with_arg("name", entry.name.clone()),
            );
        }
    }
    let at = |span| Location::new(PERMALINKS_FILE_ID, span);
    for (name, entry) in list.permalinks.iter().filter(|(n, _)| !current.has(n)) {
        out.push(
            Issue::new(diagnostics::PERMALINK_LOST, at(entry.span))
                .with_arg("name", name.clone())
                .with_arg("was", entry.place.clone()),
        );
    }
    for (name, entry) in list.aliases.iter().filter(|(n, _)| !current.has(n)) {
        out.push(
            Issue::new(diagnostics::PERMALINK_LOST, at(entry.span))
                .with_variant("alias")
                .with_arg("name", name.clone())
                .with_arg("target", entry.target.clone()),
        );
    }
    for (name, entry) in list.moved.iter().filter(|(n, _)| !current.has(n)) {
        out.push(
            Issue::new(diagnostics::PERMALINK_LOST, at(entry.span))
                .with_variant("moved")
                .with_arg("name", name.clone())
                .with_arg("was", entry.place.clone()),
        );
    }
    for (name, entry) in list.retired.iter().filter(|(n, _)| !current.has(n)) {
        out.push(
            Issue::new(diagnostics::PERMALINK_LOST, at(entry.span))
                .with_variant("retired")
                .with_arg("name", name.clone()),
        );
    }
}
