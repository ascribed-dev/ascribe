//! `[checks]`: the level a project sets for each configurable check.
//!
//! A key is a check's slug. Its value is a level (`page-size = "warning"`), or
//! a table with `level` and that check's own settings. Only the checks the
//! diagnostics registry marks configurable can be named: a diagnostic about
//! whether the project is valid can't be lowered or turned off.

use ascribe_core::{DiagnosticSlug, diagnostics};
use toml::de::DeValue;

use crate::loader::Loader;
use crate::model::{CheckLevel, CheckSetting, Checks};
use crate::names::suggest;
use crate::toml_util::{V, entries, join};

impl Loader<'_> {
    /// `[checks]`.
    pub(crate) fn checks(&mut self, v: Option<&V<'_>>) -> Checks {
        let mut settings = Vec::new();
        let Some(t) = v.and_then(|v| self.as_table("checks", v)) else {
            return Checks { settings };
        };
        for (key, span, value) in entries(t) {
            let Some(slug) = DiagnosticSlug::from_name(key) else {
                let mut issue = self
                    .issue(diagnostics::MODEL_UNKNOWN_KEY, span)
                    .with_arg("key", key)
                    .with_arg("table", "checks");
                let slugs = diagnostics::ALL.iter().map(|s| s.as_str());
                if let Some(s) = suggest(key, slugs) {
                    issue = issue.with_variant("suggestion").with_arg("suggestion", s);
                }
                self.push(issue);
                continue;
            };
            if !self.configurable.contains(&slug) {
                self.push(
                    self.issue(diagnostics::MODEL_CHECK_NOT_CONFIGURABLE, span)
                        .with_arg("check", key),
                );
                continue;
            }
            let path = join("checks", key);
            let level = match value.get_ref() {
                DeValue::String(_) => self.level(&path, value),
                DeValue::Table(table) => {
                    // A check's own settings join `level` here as checks
                    // gain them.
                    self.check_keys(&path, table, &["level"]);
                    table
                        .get("level")
                        .and_then(|l| self.level(&join(&path, "level"), l))
                }
                _ => {
                    self.wrong_type(&path, value, "a level or a table");
                    None
                }
            };
            settings.push(CheckSetting { slug, level });
        }
        Checks { settings }
    }

    /// A level: `off`, `advice`, `warning`, or `error`.
    fn level(&mut self, path: &str, v: &V<'_>) -> Option<CheckLevel> {
        let name = self.choice(path, v, &CheckLevel::NAMES)?;
        CheckLevel::from_name(&name)
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;
    use ascribe_core::FileId;

    use crate::loader::load_configurable;

    // No check is configurable yet, so these make two of them configurable.
    const CONFIGURABLE: &[DiagnosticSlug] = &[
        diagnostics::BINDING_BLANK_LINE,
        diagnostics::CONTAINER_NESTING_DEEP,
    ];

    fn load(checks: &str) -> Result<Checks, Vec<(DiagnosticSlug, String)>> {
        let text = format!("spec = \"0.1\"\n{checks}");
        load_configurable(&text, FileId::new(0), None, CONFIGURABLE)
            .map(|m| m.checks)
            .map_err(|issues| {
                issues
                    .into_iter()
                    .map(|i| {
                        let found = text.get(i.location.span.range()).unwrap_or("");
                        (i.slug, found.to_owned())
                    })
                    .collect()
            })
    }

    #[test]
    fn a_level_or_a_table_with_a_level() {
        let checks = load(
            "[checks]\nbinding-blank-line = \"advice\"\n\
             container-nesting-deep = { level = \"off\" }\n",
        )
        .unwrap();
        assert_eq!(
            checks.level(diagnostics::BINDING_BLANK_LINE),
            Some(CheckLevel::Advice)
        );
        assert_eq!(
            checks.level(diagnostics::CONTAINER_NESTING_DEEP),
            Some(CheckLevel::Off)
        );
        assert_eq!(checks.level(diagnostics::TITLE_NOT_ACCEPTED), None);
    }

    #[test]
    fn a_table_without_a_level_keeps_the_check_s_own() {
        let checks = load("[checks.binding-blank-line]\n").unwrap();
        assert_eq!(checks.settings.len(), 1);
        assert_eq!(checks.level(diagnostics::BINDING_BLANK_LINE), None);
    }

    #[test]
    fn levels_settings_and_shapes_are_checked() {
        let found = load(
            "[checks]\nbinding-blank-line = \"loud\"\n\
             container-nesting-deep = { level = \"warning\", limit = 3 }\n",
        )
        .unwrap_err();
        assert_eq!(
            found,
            [
                (diagnostics::MODEL_INVALID_VALUE, "\"loud\"".to_owned()),
                (diagnostics::MODEL_UNKNOWN_KEY, "limit".to_owned()),
            ]
        );
        let found = load("[checks]\nbinding-blank-line = 2\n").unwrap_err();
        assert_eq!(found, [(diagnostics::MODEL_WRONG_TYPE, "2".to_owned())]);
    }

    #[test]
    fn only_configurable_checks_can_be_named() {
        let found = load("[checks]\ntitle-not-accepted = \"off\"\nbinding-blank-lines = \"off\"\n")
            .unwrap_err();
        assert_eq!(
            found,
            [
                (
                    diagnostics::MODEL_CHECK_NOT_CONFIGURABLE,
                    "title-not-accepted".to_owned()
                ),
                (
                    diagnostics::MODEL_UNKNOWN_KEY,
                    "binding-blank-lines".to_owned()
                ),
            ]
        );
    }
}
