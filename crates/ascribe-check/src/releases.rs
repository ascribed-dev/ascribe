//! What `[releases]` says about a version an availability spec names, in a
//! page's mark or in `ascribe.toml`'s features and builds: whether the list
//! can place it (`available-version-ahead`, `available-line-undeclared`).

use ascribe_core::availability::{Detail, Entry, Version};
use ascribe_core::{Applicability, Fix, Issue, Location, Span, TextEdit, diagnostics};
use ascribe_model::{ContentModel, Placement};

/// The versions an entry names, in order.
pub(crate) fn entry_versions(entry: &Entry) -> Vec<&Version> {
    match &entry.detail {
        Detail::None | Detail::State { version: None, .. } => Vec::new(),
        Detail::Version(v)
        | Detail::State {
            version: Some(v), ..
        } => vec![v],
        Detail::History(steps) => steps.iter().map(|s| &s.version).collect(),
    }
}

/// The issues for each version in `entries` that its target's list can't
/// place. `at` gives where a version's span is reported; a fix is offered
/// only when `fixes` says the span is the version's own, so replacing it
/// leaves the rest of the spec alone.
pub(crate) fn placement_issues(
    model: &ContentModel,
    entries: &[Entry],
    at: &dyn Fn(Span) -> Location,
    fixes: bool,
) -> Vec<Issue> {
    let mut out = Vec::new();
    for entry in entries {
        let target = entry.target.text.as_str();
        let Some(releases) = model.releases_of(target) else {
            continue;
        };
        for version in entry_versions(entry) {
            if let Some(issue) =
                placement_issue(model, target, releases.place(version), version, at, fixes)
            {
                out.push(issue);
            }
        }
    }
    out
}

/// The issue for one version of `target` placed as `placement`, if any.
pub(crate) fn placement_issue(
    model: &ContentModel,
    target: &str,
    placement: Option<Placement<'_>>,
    version: &Version,
    at: &dyn Fn(Span) -> Location,
    fixes: bool,
) -> Option<Issue> {
    let location = at(version.span);
    let replace = |line: &Version| Fix {
        title: format!("Change to `{}`", line.text),
        file: location.file,
        edits: vec![TextEdit::replace(version.span, line.text.clone())],
        // The number meant may be another line's.
        applicability: Applicability::Unsafe,
    };
    match placement? {
        Placement::Ahead { newest } => {
            let mut issue = Issue::new(diagnostics::AVAILABLE_VERSION_AHEAD, location)
                .with_arg("target", target)
                .with_arg("version", version.text.clone())
                .with_arg("newest", newest.text.clone());
            if fixes {
                issue = issue.with_fix(replace(newest));
            }
            Some(issue)
        }
        Placement::Between { line, older, newer }
            if !model.checks.is_off(diagnostics::AVAILABLE_LINE_UNDECLARED) =>
        {
            let mut issue = Issue::new(diagnostics::AVAILABLE_LINE_UNDECLARED, location)
                .with_arg("target", target)
                .with_arg("version", version.text.clone())
                .with_arg("line", line)
                .with_arg("older", older.text.clone())
                .with_arg("newer", newer.text.clone());
            if fixes {
                issue = issue.with_fix(replace(older)).with_fix(replace(newer));
            }
            Some(issue)
        }
        Placement::In(..) | Placement::Between { .. } | Placement::Before => None,
    }
}
