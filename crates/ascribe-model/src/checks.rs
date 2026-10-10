//! `[checks]`: the level a project sets for each configurable check.
//!
//! A key is a check's slug. Its value is a level (`page-size = "warning"`), or
//! a table with `level` and that check's own settings. Only the checks the
//! diagnostics registry marks configurable can be named: a diagnostic about
//! whether the project is valid can't be lowered or turned off.
//!
//! `[checks.vale]` isn't a check: it's how the prose is checked with Vale.

use ascribe_core::{DiagnosticSlug, diagnostics};
use toml::de::DeValue;

use crate::loader::Loader;
use crate::model::{CheckLevel, CheckSetting, Checks, ValeSettings, ValeSource};
use crate::names::suggest;
use crate::toml_util::{V, entries, join, sp};
use crate::vale::Preset;

impl Loader<'_> {
    /// `[checks]`.
    pub(crate) fn checks(&mut self, v: Option<&V<'_>>) -> Checks {
        let mut settings = Vec::new();
        let mut vale = None;
        let Some(t) = v.and_then(|v| self.as_table("checks", v)) else {
            return Checks { settings, vale };
        };
        for (key, span, value) in entries(t) {
            if key == "vale" {
                vale = self.vale(value, span);
                continue;
            }
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
        Checks { settings, vale }
    }

    /// `[checks.vale]`.
    fn vale(&mut self, v: &V<'_>, key_span: ascribe_core::Span) -> Option<ValeSettings> {
        const PATH: &str = "checks.vale";
        let t = self.as_table(PATH, v)?;
        self.check_keys(
            PATH,
            t,
            &[
                "preset",
                "config",
                "command",
                "in-check",
                "max-level",
                "off",
            ],
        );
        let preset = t.get("preset");
        let config = t.get("config");
        let source = match (preset, config) {
            (Some(_), Some(c)) => {
                self.push(self.issue(diagnostics::MODEL_CHECKS_VALE, sp(c)));
                None
            }
            (None, None) => {
                self.push(
                    self.issue(diagnostics::MODEL_CHECKS_VALE, key_span)
                        .with_variant("neither"),
                );
                None
            }
            (Some(p), None) => self
                .choice(&join(PATH, "preset"), p, &Preset::names())
                .map(ValeSource::Preset),
            (None, Some(c)) => self
                .string(&join(PATH, "config"), c, true)
                .map(ValeSource::Config),
        };
        let command = match t.get("command") {
            Some(c) => self.string(&join(PATH, "command"), c, true)?,
            None => ValeSettings::COMMAND.to_owned(),
        };
        let in_check = match t.get("in-check") {
            Some(b) => self.boolean(&join(PATH, "in-check"), b)?,
            None => false,
        };
        let max_level = match t.get("max-level") {
            Some(l) => {
                let name = self.choice(&join(PATH, "max-level"), l, &CheckLevel::NAMES[1..])?;
                CheckLevel::from_name(&name)?
            }
            None => CheckLevel::Error,
        };
        let off = match t.get("off") {
            Some(list) => self.vale_off(list, source.as_ref())?,
            None => Vec::new(),
        };
        Some(ValeSettings {
            source: source?,
            command,
            in_check,
            max_level,
            off,
            span: key_span,
        })
    }

    /// `[checks.vale] off`: rules of the preset, which only a preset has.
    fn vale_off(&mut self, list: &V<'_>, source: Option<&ValeSource>) -> Option<Vec<String>> {
        let names = self.strings("checks.vale.off", list)?;
        let preset = match source? {
            ValeSource::Preset(name) => Preset::find(name)?,
            ValeSource::Config(_) => {
                self.push(
                    self.issue(diagnostics::MODEL_CHECKS_VALE, sp(list))
                        .with_variant("off"),
                );
                return None;
            }
        };
        let rules = preset.rules();
        let mut ok = true;
        for (name, span) in &names {
            if !rules.contains(name) {
                self.push(
                    self.issue(diagnostics::MODEL_CHECKS_VALE, *span)
                        .with_variant("off-rule")
                        .with_arg("preset", preset.name)
                        .with_arg("rule", name)
                        .with_arg("rules", rules.join(", ")),
                );
                ok = false;
            }
        }
        ok.then(|| names.into_iter().map(|(name, _)| name).collect())
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

    fn vale(text: &str) -> Result<ValeSettings, Vec<(DiagnosticSlug, String)>> {
        load(text).map(|c| c.vale.unwrap())
    }

    #[test]
    fn vale_takes_a_preset_or_a_config() {
        let preset = vale("[checks.vale]\npreset = \"quiet\"\n").unwrap();
        assert_eq!(preset.source, ValeSource::Preset("quiet".to_owned()));
        assert_eq!(preset.command, "vale");
        assert!(!preset.in_check);
        assert_eq!(preset.max_level, CheckLevel::Error);
        let config = vale(
            "[checks.vale]\nconfig = \".vale.ini\"\ncommand = \"bin/vale\"\n\
             in-check = true\nmax-level = \"advice\"\n",
        )
        .unwrap();
        assert_eq!(config.source, ValeSource::Config(".vale.ini".to_owned()));
        assert_eq!(config.command, "bin/vale");
        assert!(config.in_check);
        assert_eq!(config.max_level, CheckLevel::Advice);
        assert_eq!(load("").unwrap().vale, None);
    }

    #[test]
    fn vale_needs_exactly_one_source() {
        let both = vale("[checks.vale]\npreset = \"quiet\"\nconfig = \"x\"\n").unwrap_err();
        assert_eq!(both, [(diagnostics::MODEL_CHECKS_VALE, "\"x\"".to_owned())]);
        let neither = vale("[checks.vale]\ncommand = \"vale\"\n").unwrap_err();
        assert_eq!(
            neither,
            [(diagnostics::MODEL_CHECKS_VALE, "vale".to_owned())]
        );
        let unknown = vale("[checks.vale]\npreset = \"loud\"\n").unwrap_err();
        assert_eq!(
            unknown,
            [(diagnostics::MODEL_INVALID_VALUE, "\"loud\"".to_owned())]
        );
        let level = vale("[checks.vale]\npreset = \"quiet\"\nmax-level = \"off\"\n").unwrap_err();
        assert_eq!(
            level,
            [(diagnostics::MODEL_INVALID_VALUE, "\"off\"".to_owned())]
        );
    }

    #[test]
    fn vale_off_names_rules_of_the_preset() {
        let off =
            vale("[checks.vale]\npreset = \"quiet\"\noff = [\"Ascribe.Repeated\"]\n").unwrap();
        assert_eq!(off.off, ["Ascribe.Repeated"]);
        let unknown =
            vale("[checks.vale]\npreset = \"quiet\"\noff = [\"Ascribe.Nope\"]\n").unwrap_err();
        assert_eq!(
            unknown,
            [(
                diagnostics::MODEL_CHECKS_VALE,
                "\"Ascribe.Nope\"".to_owned()
            )]
        );
        let with_config =
            vale("[checks.vale]\nconfig = \".vale.ini\"\noff = [\"A.B\"]\n").unwrap_err();
        assert_eq!(
            with_config,
            [(diagnostics::MODEL_CHECKS_VALE, "[\"A.B\"]".to_owned())]
        );
    }
}
