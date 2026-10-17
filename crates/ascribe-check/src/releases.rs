//! What `[releases]` says about a version an availability spec names, in a
//! page's mark or in `ascribe.toml`'s features and builds: whether the list
//! can place it (`available-version-ahead`, `available-line-undeclared`),
//! and whether a mark still does anything (`available-same-in-every-line`,
//! `available-in-no-line`).

use std::cmp::Ordering;

use ascribe_core::availability::{
    AvailabilitySpec, Detail, Entry, Version, entry_text, without_versions_through,
};
use ascribe_core::{Applicability, Fix, Issue, Location, Span, TextEdit, diagnostics};
use ascribe_model::{AvailabilityMode, ContentModel, Placement, Releases, in_line};

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

/// The issues for each entry of `spec` that no output shows the version of,
/// or that no output has at all. `at` gives where an entry is reported; a
/// fix is offered only when `fixes` says the entry's span is its own.
pub(crate) fn inert_issues(
    model: &ContentModel,
    spec: &AvailabilitySpec,
    at: &dyn Fn(Span) -> Location,
    fixes: bool,
) -> Vec<Issue> {
    let same = !model
        .checks
        .is_off(diagnostics::AVAILABLE_SAME_IN_EVERY_LINE);
    let none = !model.checks.is_off(diagnostics::AVAILABLE_IN_NO_LINE);
    let mut out = Vec::new();
    for entry in &spec.entries {
        let target = entry.target.text.as_str();
        let Some(releases) = model.releases_of(target) else {
            continue;
        };
        let (Some(oldest), versions) = (releases.oldest_live(), entry_versions(entry)) else {
            continue;
        };
        if versions.is_empty() || shown(model, releases, spec) {
            continue;
        }
        let steps = steps(entry);
        let location = at(entry.span);
        let issue = |slug| {
            Issue::new(slug, location)
                .with_arg("entry", entry_text(entry))
                .with_arg("target", target)
                .with_arg("oldest", oldest.text.clone())
        };
        let lines: Vec<&Version> = releases.planned.iter().chain(&releases.live).collect();
        if !lines.iter().any(|line| in_line_of(model, &steps, line)) {
            if none {
                out.push(issue(diagnostics::AVAILABLE_IN_NO_LINE));
            }
            continue;
        }
        // What a line's output shows of the entry, so the two can't disagree.
        let alone = AvailabilitySpec {
            entries: vec![entry.clone()],
            span: entry.span,
        };
        let simpler = without_versions_through(&alone, target, oldest)
            .and_then(|spec| spec.entries.into_iter().next())
            .filter(|e| entry_versions(e).is_empty());
        let last = steps.last().map_or("ga", |(_, state)| state);
        if same
            && let Some(simpler) = simpler
            && model.state_is_available(last) == Some(true)
        {
            let simpler = entry_text(&simpler);
            let mut issue = issue(diagnostics::AVAILABLE_SAME_IN_EVERY_LINE)
                .with_arg("simpler", simpler.clone());
            if fixes {
                issue = issue.with_fix(Fix {
                    title: format!("Write `{simpler}`"),
                    file: location.file,
                    edits: vec![TextEdit::replace(entry.span, simpler)],
                    // No output shows the version, so none changes.
                    applicability: Applicability::Safe,
                });
            }
            out.push(issue);
        }
    }
    out
}

/// Whether a build shows a mark about `releases`' target in `spec`: any
/// build that publishes the content and isn't versioned by the target, or
/// filtered for it at a version in the window or after it. A build filtered
/// for another target publishes the content only when `spec` names that
/// target, or its dimension.
fn shown(model: &ContentModel, releases: &Releases, spec: &AvailabilitySpec) -> bool {
    let target = releases.target.as_str();
    let names = |other: &str| {
        let dimension = model.dimension_of_value(other).map(|d| d.name.as_str());
        spec.entries
            .iter()
            .any(|e| e.target.text == other || Some(e.target.text.as_str()) == dimension)
    };
    model.builds.iter().any(|build| {
        if build.versions.as_deref() == Some(target) {
            return false;
        }
        match &build.availability {
            AvailabilityMode::Filter { target: t, version } if t == target => {
                // Before the window, the mark still decides what's shown.
                version.as_ref().is_some_and(|v| {
                    releases.oldest_live().is_some_and(|oldest| {
                        !in_line(oldest, v) && v.compare(oldest) == Ordering::Less
                    })
                })
            }
            AvailabilityMode::Filter { target: t, .. } => names(t),
            AvailabilityMode::Badge => true,
        }
    })
}

/// An entry as steps: each state with the version it starts at, `None` for
/// a state in effect at every version.
fn steps(entry: &Entry) -> Vec<(Option<&Version>, &str)> {
    match &entry.detail {
        Detail::None => vec![(None, "ga")],
        Detail::Version(v) => vec![(Some(v), "ga")],
        Detail::State { state, version } => vec![(version.as_ref(), state.text.as_str())],
        Detail::History(steps) => steps
            .iter()
            .map(|s| (Some(&s.version), s.state.text.as_str()))
            .collect(),
    }
}

/// Whether content with `steps` is in `line`: whether a state that counts
/// as available is in effect at some version in it, the one at its start
/// or one that starts inside it.
fn in_line_of(model: &ContentModel, steps: &[(Option<&Version>, &str)], line: &Version) -> bool {
    let available = |state: &str| model.state_is_available(state) == Some(true);
    let at_start = steps
        .iter()
        .rev()
        .find(|(v, _)| v.is_none_or(|v| v.compare(line) != Ordering::Greater));
    at_start.is_some_and(|(_, state)| available(state))
        || steps
            .iter()
            .any(|(v, state)| v.is_some_and(|v| in_line(line, v)) && available(state))
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used)]

    use ascribe_core::availability::parse_availability;
    use ascribe_core::{FileId, apply_edits};

    use super::*;

    const MODEL: &str = "spec = \"0.1\"\n\
        [dimensions.deployment]\nvalues = [\"cloud\", \"self-managed\"]\nversionless = [\"cloud\"]\n\
        [releases.self-managed]\nlive = [\"3.4\", \"3.5\"]\n\
        [builds.site]\nversions = \"self-managed\"\n";

    /// The mark without its versions, keeping its state, as a safe fix.
    #[test]
    fn a_mark_every_line_has_is_offered_without_its_versions() {
        let model = ascribe_model::load_str(MODEL, FileId::new(0)).expect("the model loads");
        let at = |span: Span| Location::new(FileId::new(1), span);
        for (text, fixed) in [
            ("cloud, self-managed 3.1", "cloud, self-managed"),
            ("self-managed deprecated 3.2", "self-managed deprecated"),
            ("self-managed (preview 3.1, ga 3.3)", "self-managed"),
        ] {
            let spec = parse_availability(text, 0).expect("a spec");
            let issues = inert_issues(&model, &spec, &at, true);
            assert_eq!(issues.len(), 1, "{text}");
            let fix = &issues[0].fixes[0];
            assert_eq!(fix.applicability, Applicability::Safe);
            assert_eq!(apply_edits(text, &fix.edits).expect("edits apply"), fixed);
        }
        for text in ["self-managed 3.4", "self-managed preview 3.4"] {
            let spec = parse_availability(text, 0).expect("a spec");
            assert_eq!(inert_issues(&model, &spec, &at, true).len(), 1, "{text}");
        }
    }

    /// A version inside the oldest line but after its start is one the
    /// line's output shows, so the mark isn't reported.
    #[test]
    fn a_version_inside_the_oldest_line_is_not_offered() {
        let model = ascribe_model::load_str(MODEL, FileId::new(0)).expect("the model loads");
        let at = |span: Span| Location::new(FileId::new(1), span);
        for text in [
            "self-managed 3.4.1",
            "self-managed preview 3.4.2",
            "self-managed (preview 3.2, ga 3.4.1)",
            "self-managed 3.5",
        ] {
            let spec = parse_availability(text, 0).expect("a spec");
            assert!(inert_issues(&model, &spec, &at, true).is_empty(), "{text}");
        }
    }
}
