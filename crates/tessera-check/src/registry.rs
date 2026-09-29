//! The diagnostics registry: codes, severities, and message templates.
//!
//! `tests/conformance/diagnostics.toml` is the one source. It's embedded at
//! build time and parsed on first use, so every tool words and ranks a
//! problem the same way. [`Registry::global`] is the parsed registry.

use std::collections::HashMap;
use std::sync::OnceLock;

use tessera_core::{DiagnosticSlug, Issue};

use crate::Severity;

/// The registry file, embedded.
const SOURCE: &str = include_str!("../../../tests/conformance/diagnostics.toml");

/// Which level a diagnostic is checked at (SPEC §8.1).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Level {
    /// Each file on its own.
    File,
    /// Each expanded page, per build.
    Page,
}

/// One registry entry.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Entry {
    /// The code, such as `ASC001`.
    pub code: String,
    /// The slug.
    pub slug: String,
    /// The severity.
    pub severity: Severity,
    /// The level.
    pub level: Level,
    /// The main message template.
    pub message: String,
    /// Alternative templates, by variant name.
    pub messages: HashMap<String, String>,
}

impl Entry {
    /// Whether the message names the build: its template, or one of its
    /// variants, has a `{build}` placeholder. Such a message already says
    /// which builds it's about.
    pub fn names_build(&self) -> bool {
        std::iter::once(&self.message)
            .chain(self.messages.values())
            .any(|t| placeholders(t).iter().any(|name| name == "build"))
    }
}

/// The parsed diagnostics registry.
#[derive(Debug)]
pub struct Registry {
    entries: HashMap<String, Entry>,
    /// Slugs in registry (file) order.
    order: Vec<String>,
}

impl Registry {
    /// The registry embedded in this build.
    pub fn global() -> &'static Registry {
        static REGISTRY: OnceLock<Registry> = OnceLock::new();
        REGISTRY.get_or_init(|| Registry::parse(SOURCE))
    }

    /// Parses registry text. Entries that don't have every required field
    /// are left out; a test checks that none is.
    pub fn parse(text: &str) -> Registry {
        let mut entries = HashMap::new();
        let mut order = Vec::new();
        let table: toml::Table = text.parse().unwrap_or_default();
        let list = table
            .get("diagnostic")
            .and_then(toml::Value::as_array)
            .map(Vec::as_slice)
            .unwrap_or_default();
        for value in list {
            let Some(entry) = value.as_table().and_then(Registry::entry) else {
                continue;
            };
            order.push(entry.slug.clone());
            entries.insert(entry.slug.clone(), entry);
        }
        Registry { entries, order }
    }

    fn entry(t: &toml::Table) -> Option<Entry> {
        let text = |k: &str| t.get(k).and_then(toml::Value::as_str).map(str::to_owned);
        let severity = match text("severity")?.as_str() {
            "error" => Severity::Error,
            "warning" => Severity::Warning,
            _ => return None,
        };
        let level = match text("level")?.as_str() {
            "file" => Level::File,
            "page" => Level::Page,
            _ => return None,
        };
        let messages = t
            .get("messages")
            .and_then(toml::Value::as_table)
            .map(|m| {
                m.iter()
                    .filter_map(|(k, v)| Some((k.clone(), v.as_str()?.to_owned())))
                    .collect()
            })
            .unwrap_or_default();
        Some(Entry {
            code: text("code")?,
            slug: text("slug")?,
            severity,
            level,
            message: text("message")?,
            messages,
        })
    }

    /// The entry for a slug.
    pub fn get(&self, slug: DiagnosticSlug) -> Option<&Entry> {
        self.entries.get(slug.as_str())
    }

    /// Every entry, in registry order.
    pub fn entries(&self) -> impl Iterator<Item = &Entry> {
        self.order.iter().filter_map(|s| self.entries.get(s))
    }

    /// The message of an issue: its variant's template (or the main one)
    /// with the arguments filled in. A placeholder with no argument stays as
    /// written, so a missing argument is visible instead of silent (tests
    /// check that no issue leaves one).
    pub fn message(&self, issue: &Issue) -> String {
        let Some(entry) = self.get(issue.slug) else {
            return issue.slug.to_string();
        };
        let template = issue
            .variant
            .and_then(|v| entry.messages.get(v))
            .unwrap_or(&entry.message);
        debug_assert!(
            placeholders(template)
                .iter()
                .all(|name| issue.arg(name).is_some()),
            "`{}` ({:?}) is missing an argument for its message template",
            issue.slug,
            issue.variant,
        );
        render(template, |name| issue.arg(name).map(str::to_owned))
    }
}

/// Fills in a message template: `{name}` from `arg`, `{{` and `}}` as
/// literal braces. A `{name}` with no value stays as written.
pub fn render(template: &str, mut arg: impl FnMut(&str) -> Option<String>) -> String {
    let mut out = String::with_capacity(template.len());
    let mut rest = template;
    while let Some(i) = rest.find(['{', '}']) {
        out.push_str(&rest[..i]);
        let tail = &rest[i..];
        if tail.starts_with("{{") || tail.starts_with("}}") {
            out.push_str(&tail[..1]);
            rest = &tail[2..];
        } else if tail.starts_with('{')
            && let Some(close) = tail.find('}')
        {
            let name = &tail[1..close];
            match arg(name) {
                Some(value) => out.push_str(&value),
                None => out.push_str(&tail[..=close]),
            }
            rest = &tail[close + 1..];
        } else {
            out.push_str(&tail[..1]);
            rest = &tail[1..];
        }
    }
    out.push_str(rest);
    out
}

/// The placeholder names of a template, in order.
pub fn placeholders(template: &str) -> Vec<String> {
    let mut names = Vec::new();
    render(template, |name| {
        names.push(name.to_owned());
        Some(String::new())
    });
    names
}

#[cfg(test)]
mod tests {
    use super::*;
    use tessera_core::diagnostics;

    #[test]
    fn the_embedded_registry_parses_and_covers_every_slug() {
        let registry = Registry::global();
        for slug in diagnostics::ALL {
            assert!(
                registry.get(*slug).is_some(),
                "{slug} is not in the registry"
            );
        }
        assert_eq!(registry.entries().count(), diagnostics::ALL.len());
    }

    #[test]
    fn templates_fill_in_and_escape_braces() {
        let arg = |n: &str| (n == "key").then(|| "x".to_owned());
        assert_eq!(render("`{key}` and `\\{{`", arg), "`x` and `\\{`");
        assert_eq!(render("{other} {{}}", arg), "{other} {}");
        assert_eq!(placeholders("{a} {{b}} {c}"), ["a", "c"]);
    }

    #[test]
    fn messages_use_the_variant() {
        let issue = Issue::new(
            diagnostics::ATTRIBUTE_UNKNOWN_KEY,
            tessera_core::Location::new(tessera_core::FileId::new(0), 0..1),
        )
        .with_arg("name", "steps")
        .with_arg("key", "start")
        .with_variant("none");
        assert_eq!(
            Registry::global().message(&issue),
            "`@steps` takes no attributes, so `start` isn't allowed"
        );
    }
}
