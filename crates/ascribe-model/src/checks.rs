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
use crate::toml_util::{V, entries, join, sp};

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
            let mut limit = None;
            let level = match value.get_ref() {
                DeValue::String(_) => self.level(&path, value),
                DeValue::Table(table) => {
                    let mut keys = vec!["level"];
                    keys.extend(settings_of(slug));
                    self.check_keys(&path, table, &keys);
                    if let Some(v) = table.get("limit").filter(|_| keys.contains(&"limit")) {
                        limit = self.size(&join(&path, "limit"), v);
                    }
                    table
                        .get("level")
                        .and_then(|l| self.level(&join(&path, "level"), l))
                }
                _ => {
                    self.wrong_type(&path, value, "a level or a table");
                    None
                }
            };
            settings.push(CheckSetting { slug, level, limit });
        }
        Checks { settings }
    }

    /// A size: a whole number of bytes, or a string with a unit, `B`, `KB`,
    /// or `MB` (`"500 KB"`, `"1.5 MB"`); a kilobyte is 1,000 bytes.
    fn size(&mut self, path: &str, v: &V<'_>) -> Option<u64> {
        let parsed = match v.get_ref() {
            DeValue::Integer(i) => i.as_str().parse::<u64>().ok(),
            DeValue::String(s) => parse_size(s),
            _ => {
                self.wrong_type(path, v, SIZE);
                return None;
            }
        };
        if parsed.is_none() {
            let found = format!("`{}`", self.text_of(sp(v)));
            self.push(
                self.issue(diagnostics::MODEL_WRONG_TYPE, sp(v))
                    .with_arg("key", path)
                    .with_arg("expected", SIZE)
                    .with_arg("found", found),
            );
        }
        parsed
    }

    /// A level: `off`, `advice`, `warning`, or `error`.
    fn level(&mut self, path: &str, v: &V<'_>) -> Option<CheckLevel> {
        let name = self.choice(path, v, &CheckLevel::NAMES)?;
        CheckLevel::from_name(&name)
    }
}

/// What a size is, in a message.
const SIZE: &str = "a size, such as \"500 KB\"";

/// The settings a check takes in its table, besides `level`.
fn settings_of(slug: DiagnosticSlug) -> &'static [&'static str] {
    if slug == diagnostics::IMAGE_LARGE {
        &["limit"]
    } else {
        &[]
    }
}

/// A size in bytes, from a number and a unit: `B`, `KB` (1,000 bytes), or
/// `MB` (1,000,000), in either case, with or without a space.
fn parse_size(text: &str) -> Option<u64> {
    let text = text.trim();
    let split = text
        .find(|c: char| !(c.is_ascii_digit() || c == '.'))
        .unwrap_or(text.len());
    let (number, unit) = text.split_at(split);
    let scale: f64 = match unit.trim().to_ascii_uppercase().as_str() {
        "B" => 1.0,
        "KB" => 1_000.0,
        "MB" => 1_000_000.0,
        _ => return None,
    };
    let number: f64 = number.parse().ok()?;
    let bytes = (number * scale).round();
    // A size is finite and at most a few gigabytes, so it fits in a u64.
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    (bytes.is_finite() && (0.0..1e15).contains(&bytes)).then_some(bytes as u64)
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;
    use ascribe_core::FileId;

    use crate::loader::load_configurable;

    // Two checks that aren't configurable are made configurable here, and
    // one that is, with a setting of its own.
    const CONFIGURABLE: &[DiagnosticSlug] = &[
        diagnostics::BINDING_BLANK_LINE,
        diagnostics::CONTAINER_NESTING_DEEP,
        diagnostics::IMAGE_LARGE,
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

    #[test]
    fn a_limit_is_a_size() {
        let checks = load("[checks.image-large]\nlimit = \"1.5 MB\"\n").unwrap();
        assert_eq!(checks.limit(diagnostics::IMAGE_LARGE), Some(1_500_000));
        assert_eq!(checks.level(diagnostics::IMAGE_LARGE), None);
        let checks = load("[checks.image-large]\nlimit = 2048\nlevel = \"warning\"\n").unwrap();
        assert_eq!(checks.limit(diagnostics::IMAGE_LARGE), Some(2048));
        assert_eq!(parse_size("500kb"), Some(500_000));
        assert_eq!(parse_size("12 B"), Some(12));
        assert_eq!(parse_size("1 GB"), None);
        assert_eq!(parse_size("KB"), None);
    }

    #[test]
    fn a_limit_that_isnt_a_size_is_reported() {
        let found = load("[checks.image-large]\nlimit = \"big\"\n").unwrap_err();
        assert_eq!(
            found,
            [(diagnostics::MODEL_WRONG_TYPE, "\"big\"".to_owned())]
        );
        let found = load("[checks.container-nesting-deep]\nlimit = \"1 MB\"\n").unwrap_err();
        assert_eq!(
            found,
            [(diagnostics::MODEL_UNKNOWN_KEY, "limit".to_owned())]
        );
    }
}
