//! Permalinks across the project. `ascribe.toml`'s `[permalinks]` against
//! the content (SPEC §7.6): an alias that doesn't lead to a permalink
//! (`permalink-alias-target-missing`), a retired permalink's `to` that
//! leads nowhere (`permalink-retired-target-missing`), and a name both
//! there and in the content (`permalink-name-conflict`), each on its entry
//! in `ascribe.toml`. And the permalinks list, `ascribe.permalinks` (SPEC
//! §7.5), against what the project has: `permalinks-list-invalid` when the
//! list can't be read, `permalinks-list-behind` at each name it doesn't
//! record as the project has it, and `permalink-removed` at each entry the
//! project doesn't have any more. A project without the list gets none of
//! the last three.
//!
//! They hold for every build, since the names are the project's: a
//! permalink is declared in a source file whether or not a build publishes
//! it.

use std::collections::BTreeMap;

use ascribe_core::{FileId, Issue, Location, RelPath, diagnostics};
use ascribe_model::{Current, ModelPermalinks, PermalinkList};
use ascribe_resolve::{DeclaredPermalink, FileKind};

use crate::project::PERMALINKS_FILE_ID;
use crate::{Diagnostic, Project};

/// The id of `ascribe.toml`, where `[permalinks]`'s problems are.
const MODEL_FILE_ID: FileId = FileId::new(0);

/// What the checks found, by file: each problem with `[permalinks]` under
/// `ascribe.toml`'s id, each removed entry and each problem reading the
/// list under the list's id, and each permalink the list is behind on under
/// the file it's declared in, each list in source order. `index` is the
/// project's source index; `file` gives the project's id of one of its
/// files.
pub(crate) fn check(
    project: &Project,
    index: &ascribe_resolve::Project,
    file: &dyn Fn(FileId) -> FileId,
) -> BTreeMap<FileId, Vec<Diagnostic>> {
    let model = &project.model().permalinks;
    let list = project.permalinks();
    let mut issues: Vec<Issue> = Vec::new();
    if list.is_some() || !model.is_empty() {
        let declared = index.declared_permalinks();
        let places = DeclaredPermalink::places(&declared);
        names(model, index, &declared, file, &mut issues);
        match list {
            None => {}
            Some(Err(problems)) => issues.extend(problems.iter().cloned()),
            Some(Ok(list)) => {
                let current = Current::new(places, model);
                against_list(list, &current, model, &declared, file, &mut issues);
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

/// `[permalinks]` against the content.
fn names(
    model: &ModelPermalinks,
    index: &ascribe_resolve::Project,
    declared: &[DeclaredPermalink],
    file: &dyn Fn(FileId) -> FileId,
    out: &mut Vec<Issue>,
) {
    let declared_at = |name: &str| declared.iter().find(|p| p.name == name);
    for retired in &model.retired {
        let at = Location::new(MODEL_FILE_ID, retired.span);
        if let Some(permalink) = declared_at(&retired.name) {
            out.push(conflict(at, permalink, None, file));
            continue;
        }
        let (Some(to), Some(span)) = (&retired.to, retired.to_span) else {
            continue;
        };
        if let Some(issue) = retired_target(index, &retired.name, to) {
            out.push(issue.at(Location::new(MODEL_FILE_ID, span)));
        }
    }
    for alias in &model.aliases {
        // A name both retired and an alias, or an alias in a loop, is the
        // loader's `model-permalinks`, and a model with it doesn't load.
        let at = Location::new(MODEL_FILE_ID, alias.span);
        if let Some(permalink) = declared_at(&alias.name) {
            out.push(conflict(at, permalink, Some("alias"), file));
            continue;
        }
        let Some(end) = model.alias_end(&alias.name) else {
            continue;
        };
        if declared_at(end).is_some() {
            continue;
        }
        let issue = Issue::new(
            diagnostics::PERMALINK_ALIAS_TARGET_MISSING,
            Location::new(MODEL_FILE_ID, alias.target_span),
        )
        .with_arg("name", alias.name.clone())
        .with_arg("target", end.to_owned());
        out.push(if model.retired(end).is_some() {
            issue.with_variant("retired")
        } else {
            issue
        });
    }
}

/// `permalink-name-conflict` at `at`, an entry in `[permalinks]`, naming
/// where the content declares it.
fn conflict(
    at: Location,
    permalink: &DeclaredPermalink,
    variant: Option<&'static str>,
    file: &dyn Fn(FileId) -> FileId,
) -> Issue {
    let mut issue = Issue::new(diagnostics::PERMALINK_NAME_CONFLICT, at)
        .with_arg("name", permalink.name.clone())
        .with_arg("place", permalink.place())
        .with_related(
            Location::new(file(permalink.at.file), permalink.at.span),
            "declared as a permalink here",
        );
    if let Some(variant) = variant {
        issue = issue.with_variant(variant);
    }
    issue
}

/// Where a retired permalink's `to` leads, written as a link from the
/// content root: a page, and `#` and a heading's id. `None` when it leads
/// to a page, or to a heading on one; otherwise the problem, to be located.
fn retired_target(index: &ascribe_resolve::Project, name: &str, to: &str) -> Option<Unlocated> {
    let (path, id) = match to.split_once('#') {
        Some((path, id)) => (path, Some(id)),
        None => (to, None),
    };
    let problem = Unlocated {
        name: name.to_owned(),
        to: to.to_owned(),
        path: path.to_owned(),
        id: id.map(str::to_owned),
        variant: None,
    };
    let Some(page) = RelPath::parse(path.trim_start_matches('/'))
        .ok()
        .and_then(|p| index.file(&p).map(|f| (p, f.kind)))
    else {
        return Some(problem);
    };
    if page.1 != FileKind::Page {
        return Some(Unlocated {
            variant: Some("fragment"),
            ..problem
        });
    }
    match id {
        Some(id) if index.page_heading(&page.0, id).is_none() => Some(Unlocated {
            variant: Some("heading"),
            ..problem
        }),
        _ => None,
    }
}

/// `permalink-retired-target-missing`, before it's given a place.
struct Unlocated {
    name: String,
    to: String,
    path: String,
    id: Option<String>,
    variant: Option<&'static str>,
}

impl Unlocated {
    fn at(self, at: Location) -> Issue {
        let mut issue = Issue::new(diagnostics::PERMALINK_RETIRED_TARGET_MISSING, at)
            .with_arg("name", self.name)
            .with_arg("to", self.to)
            .with_arg("path", self.path);
        if let Some(id) = self.id {
            issue = issue.with_arg("id", id);
        }
        if let Some(variant) = self.variant {
            issue = issue.with_variant(variant);
        }
        issue
    }
}

/// The list against what the project has: each declared name, alias, and
/// retired permalink recorded as it is, and each entry still had.
fn against_list(
    list: &PermalinkList,
    current: &Current,
    model: &ModelPermalinks,
    declared: &[DeclaredPermalink],
    file: &dyn Fn(FileId) -> FileId,
    out: &mut Vec<Issue>,
) {
    let mut reported: Vec<&str> = Vec::new();
    for permalink in declared {
        let name = permalink.name.as_str();
        if reported.contains(&name) {
            continue;
        }
        let at = Location::new(file(permalink.at.file), permalink.at.span);
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
    for alias in &model.aliases {
        let Some(target) = current.aliases.get(&alias.name) else {
            continue;
        };
        if list
            .aliases
            .get(&alias.name)
            .is_none_or(|e| e.target != *target)
        {
            out.push(
                Issue::new(
                    diagnostics::PERMALINKS_LIST_BEHIND,
                    Location::new(MODEL_FILE_ID, alias.span),
                )
                .with_variant("alias")
                .with_arg("name", alias.name.clone()),
            );
        }
    }
    for retired in &model.retired {
        let Some(reason) = current.retired.get(&retired.name) else {
            continue;
        };
        if list
            .retired
            .get(&retired.name)
            .is_none_or(|e| e.reason != *reason)
        {
            out.push(
                Issue::new(
                    diagnostics::PERMALINKS_LIST_BEHIND,
                    Location::new(MODEL_FILE_ID, retired.span),
                )
                .with_variant("retired")
                .with_arg("name", retired.name.clone()),
            );
        }
    }
    let at = |span| Location::new(PERMALINKS_FILE_ID, span);
    for (name, entry) in list.permalinks.iter().filter(|(n, _)| !current.has(n)) {
        out.push(
            Issue::new(diagnostics::PERMALINK_REMOVED, at(entry.span))
                .with_arg("name", name.clone())
                .with_arg("was", entry.place.clone()),
        );
    }
    for (name, entry) in list.aliases.iter().filter(|(n, _)| !current.has(n)) {
        out.push(
            Issue::new(diagnostics::PERMALINK_REMOVED, at(entry.span))
                .with_variant("alias")
                .with_arg("name", name.clone())
                .with_arg("target", entry.target.clone()),
        );
    }
    for (name, entry) in list.retired.iter().filter(|(n, _)| !current.has(n)) {
        out.push(
            Issue::new(diagnostics::PERMALINK_REMOVED, at(entry.span))
                .with_variant("retired")
                .with_arg("name", name.clone()),
        );
    }
}
