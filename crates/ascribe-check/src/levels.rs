//! The levels a project sets in `[checks]`: how loudly each configurable
//! check speaks.

use ascribe_model::{CheckLevel, Checks};

use crate::{Diagnostic, Severity};

/// The diagnostics with the levels of `[checks]` applied: one whose check is
/// set to `off` is left out, and one set to another level is reported at it.
/// Every other diagnostic keeps its registry severity. Only configurable
/// checks can be set, so a diagnostic about validity is never changed.
pub fn apply_levels(checks: &Checks, diagnostics: Vec<Diagnostic>) -> Vec<Diagnostic> {
    if checks.settings.is_empty() {
        return diagnostics;
    }
    diagnostics
        .into_iter()
        .filter_map(|mut d| {
            d.severity = match checks.level(d.slug) {
                None => d.severity,
                Some(CheckLevel::Off) => return None,
                Some(CheckLevel::Advice) => Severity::Advice,
                Some(CheckLevel::Warning) => Severity::Warning,
                Some(CheckLevel::Error) => Severity::Error,
            };
            Some(d)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use ascribe_core::{FileId, Issue, Location, diagnostics};
    use ascribe_model::CheckSetting;

    fn diagnostic(slug: ascribe_core::DiagnosticSlug) -> Diagnostic {
        let at = Location::new(FileId::new(1), 0..1);
        let issue = Issue::new(slug, at)
            .with_arg("name", "note")
            .with_arg("depth", "3");
        Diagnostic::from_issue(&issue)
    }

    // No check is configurable yet, so these set the level of ones that
    // aren't: the loader would refuse them, and `apply_levels` doesn't care.
    fn checks(settings: &[(ascribe_core::DiagnosticSlug, Option<CheckLevel>)]) -> Checks {
        Checks {
            settings: settings
                .iter()
                .map(|&(slug, level)| CheckSetting { slug, level })
                .collect(),
            vale: None,
        }
    }

    #[test]
    fn a_level_changes_the_severity_and_off_leaves_it_out() {
        let found = vec![
            diagnostic(diagnostics::BINDING_BLANK_LINE),
            diagnostic(diagnostics::CONTAINER_NESTING_DEEP),
            diagnostic(diagnostics::TITLE_NOT_ACCEPTED),
        ];
        let set = checks(&[
            (diagnostics::BINDING_BLANK_LINE, Some(CheckLevel::Advice)),
            (diagnostics::CONTAINER_NESTING_DEEP, Some(CheckLevel::Off)),
        ]);
        let applied = apply_levels(&set, found);
        let got: Vec<_> = applied.iter().map(|d| (d.slug, d.severity)).collect();
        assert_eq!(
            got,
            [
                (diagnostics::BINDING_BLANK_LINE, Severity::Advice),
                (diagnostics::TITLE_NOT_ACCEPTED, Severity::Warning),
            ]
        );
    }

    #[test]
    fn a_table_without_a_level_keeps_the_registry_severity() {
        let set = checks(&[(diagnostics::BINDING_BLANK_LINE, None)]);
        let applied = apply_levels(&set, vec![diagnostic(diagnostics::BINDING_BLANK_LINE)]);
        assert_eq!(applied[0].severity, Severity::Warning);
    }
}
