//! `[checks]`: the level a project sets for each configurable check.
//!
//! A key is a check's slug. Its value is a level (`page-size = "warning"`), or
//! a table with `level` and that check's own settings ([`SETTINGS`]). Only
//! the checks the diagnostics registry marks configurable can be named: a
//! diagnostic about whether the project is valid can't be lowered or turned
//! off.
//!
//! `[checks.links]` isn't a check: it's how `ascribe report links` checks the
//! external links.
//! `[checks.vale]` isn't a check: it's how the prose is checked with Vale.

use ascribe_core::{DiagnosticSlug, diagnostics};
use toml::de::DeValue;

use crate::loader::Loader;
use crate::model::{CheckLevel, CheckSetting, Checks, LinkSettings, ValeSettings, ValeSource};
use crate::names::suggest;
use crate::toml_util::{V, entries, join, sp, to_yaml};
use crate::vale::Preset;

/// The settings each check takes besides `level`, by slug. `limit` is a whole
/// number above 0, or for `image-large` a size ([`parse_size`]).
pub const SETTINGS: &[(DiagnosticSlug, &[&str])] = &[
    (diagnostics::PAGE_SIZE, &["limit"]),
    (diagnostics::IMAGE_LARGE, &["limit"]),
];

impl Loader<'_> {
    /// `[checks]`.
    pub(crate) fn checks(&mut self, v: Option<&V<'_>>) -> Checks {
        let mut settings = Vec::new();
        let mut links = LinkSettings::default();
        let mut vale = None;
        let Some(t) = v.and_then(|v| self.as_table("checks", v)) else {
            return Checks {
                settings,
                links,
                vale,
            };
        };
        for (key, span, value) in entries(t) {
            if key == "links" {
                links = self.links(value);
                continue;
            }
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
            let mut setting = CheckSetting {
                slug,
                level: None,
                limit: None,
            };
            match value.get_ref() {
                DeValue::String(_) => setting.level = self.level(&path, value),
                DeValue::Table(table) => {
                    let own = SETTINGS
                        .iter()
                        .find(|(s, _)| *s == slug)
                        .map_or(&[][..], |(_, keys)| *keys);
                    let allowed: Vec<&str> = std::iter::once("level")
                        .chain(own.iter().copied())
                        .collect();
                    self.check_keys(&path, table, &allowed);
                    setting.level = table
                        .get("level")
                        .and_then(|l| self.level(&join(&path, "level"), l));
                    if own.contains(&"limit") {
                        let at = join(&path, "limit");
                        setting.limit = table.get("limit").and_then(|l| {
                            if slug == diagnostics::IMAGE_LARGE {
                                self.size(&at, l)
                            } else {
                                self.limit(&at, l)
                            }
                        });
                    }
                }
                _ => self.wrong_type(&path, value, "a level or a table"),
            }
            settings.push(setting);
        }
        Checks {
            settings,
            links,
            vale,
        }
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

    /// `[checks.links]`.
    fn links(&mut self, v: &V<'_>) -> LinkSettings {
        const PATH: &str = "checks.links";
        let mut links = LinkSettings::default();
        let Some(t) = self.as_table(PATH, v) else {
            return links;
        };
        self.check_keys(PATH, t, &["command", "ignore"]);
        if let Some(command) = t
            .get("command")
            .and_then(|c| self.string(&join(PATH, "command"), c, true))
        {
            links.command = command;
        }
        if let Some(list) = t.get("ignore") {
            let at = join(PATH, "ignore");
            for (host, span) in self.strings(&at, list).unwrap_or_default() {
                if is_host_pattern(&host) {
                    links.ignore.push(host);
                } else {
                    self.push(
                        self.issue(diagnostics::MODEL_WRONG_TYPE, span)
                            .with_arg("key", at.as_str())
                            .with_arg("expected", HOST)
                            .with_arg("found", format!("`\"{host}\"`")),
                    );
                }
            }
        }
        links
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

    /// A limit: a whole number above 0.
    fn limit(&mut self, path: &str, v: &V<'_>) -> Option<u64> {
        let n = match v.get_ref() {
            DeValue::Integer(_) => to_yaml(v.get_ref()).as_u64().filter(|n| *n > 0),
            _ => None,
        };
        if n.is_none() {
            self.wrong_type(path, v, "a whole number above 0");
        }
        n
    }

    /// A level: `off`, `advice`, `warning`, or `error`.
    fn level(&mut self, path: &str, v: &V<'_>) -> Option<CheckLevel> {
        let name = self.choice(path, v, &CheckLevel::NAMES)?;
        CheckLevel::from_name(&name)
    }
}

/// What a host pattern is, in a message.
const HOST: &str = "a host, such as \"example.com\" or \"*.example.com\"";

/// Whether `text` is a host name, or `*.` and a host name: letters, digits,
/// `-`, and `.` (and anything outside ASCII, for an international name), with
/// no scheme, port, or path.
fn is_host_pattern(text: &str) -> bool {
    let host = text.strip_prefix("*.").unwrap_or(text);
    !host.is_empty()
        && !host.starts_with('.')
        && !host.contains("..")
        && host
            .chars()
            .all(|c| c.is_alphanumeric() || c == '-' || c == '.' || !c.is_ascii())
}

/// What a size is, in a message.
const SIZE: &str = "a size, such as \"500 KB\"";

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

    // Two checks that aren't configurable, made so, and two that are, each
    // with a setting of its own.
    const CONFIGURABLE: &[DiagnosticSlug] = &[
        diagnostics::BINDING_BLANK_LINE,
        diagnostics::CONTAINER_NESTING_DEEP,
        diagnostics::PAGE_SIZE,
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
    fn a_check_s_own_settings() {
        let checks = load("[checks.page-size]\nlevel = \"warning\"\nlimit = 20_000\n").unwrap();
        assert_eq!(checks.limit(diagnostics::PAGE_SIZE), Some(20_000));
        assert_eq!(
            checks.level(diagnostics::PAGE_SIZE),
            Some(CheckLevel::Warning)
        );
        let found = load("[checks.page-size]\nlimit = 0\n").unwrap_err();
        assert_eq!(found, [(diagnostics::MODEL_WRONG_TYPE, "0".to_owned())]);
        let found = load("[checks.page-size]\nlimit = \"big\"\n").unwrap_err();
        assert_eq!(
            found,
            [(diagnostics::MODEL_WRONG_TYPE, "\"big\"".to_owned())]
        );
        // `limit` is page-size's own.
        let found = load("[checks.binding-blank-line]\nlimit = 3\n").unwrap_err();
        assert_eq!(
            found,
            [(diagnostics::MODEL_UNKNOWN_KEY, "limit".to_owned())]
        );
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

    #[test]
    fn links_take_a_command_and_hosts_to_leave_out() {
        let checks = load(
            "[checks.links]\ncommand = \"bin/lychee\"\n\
             ignore = [\"example.com\", \"*.internal.example\"]\n",
        )
        .unwrap();
        assert_eq!(checks.links.command, "bin/lychee");
        assert!(checks.links.ignores("Example.com"));
        assert!(!checks.links.ignores("www.example.com"));
        assert!(checks.links.ignores("docs.internal.example"));
        assert!(!checks.links.ignores("internal.example"));
        assert!(!checks.links.ignores("notinternal.example"));
        assert_eq!(load("").unwrap().links.command, "lychee");

        let found =
            load("[checks.links]\nignore = [\"https://example.com/\"]\nretries = 3\n").unwrap_err();
        assert_eq!(
            found,
            [
                (
                    diagnostics::MODEL_WRONG_TYPE,
                    "\"https://example.com/\"".to_owned()
                ),
                (diagnostics::MODEL_UNKNOWN_KEY, "retries".to_owned()),
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
