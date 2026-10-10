//! The diagnostics registry: codes, severities, and message templates.
//!
//! `tests/conformance/diagnostics.toml` is the one source. It's embedded at
//! build time and parsed on first use, so every tool words and ranks a
//! problem the same way. [`Registry::global`] is the parsed registry.

use std::collections::HashMap;
use std::sync::OnceLock;

use ascribe_core::{DiagnosticSlug, Issue};

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

/// The kind of next step a diagnostic has: what the author, or their agent,
/// does about it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Next {
    /// Every instance has a fix Ascribe can apply: a diagnostic's fix, or
    /// the editor's quick fix.
    Fix,
    /// The author picks among things Ascribe can list: a link's target, a
    /// key's allowed values, a declared phrase.
    Choose,
    /// It needs writing or judgment.
    Write,
    /// Nothing in the source can fix it: it's set somewhere else, such as
    /// the hosting.
    Outside,
    /// It may be fine as it is.
    Review,
}

impl Next {
    /// Every kind, in the order the docs list them.
    pub const ALL: [Next; 5] = [
        Next::Fix,
        Next::Choose,
        Next::Write,
        Next::Outside,
        Next::Review,
    ];

    /// `"fix"`, `"choose"`, `"write"`, `"outside"`, or `"review"`.
    pub fn as_str(self) -> &'static str {
        match self {
            Next::Fix => "fix",
            Next::Choose => "choose",
            Next::Write => "write",
            Next::Outside => "outside",
            Next::Review => "review",
        }
    }

    /// The kind with this name.
    pub fn from_name(name: &str) -> Option<Next> {
        Next::ALL.into_iter().find(|n| n.as_str() == name)
    }
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
    /// How to fix the problem, in general: the entry's `fix` paragraph,
    /// which every entry that isn't retired has.
    pub fix: Option<String>,
    /// A short wrong-and-right example: the registry's `example`.
    pub example: Option<Example>,
    /// The kind of next step, which every entry that isn't retired has.
    pub next: Option<Next>,
    /// The named pieces of context an agent prompt about it carries, beyond
    /// the message and the line, such as `allowed-values`.
    pub evidence: Vec<String>,
    /// Whether a project may set its level in `[checks]`.
    pub configurable: bool,
    /// Whether it's retired: no longer reported.
    pub retired: bool,
}

/// A diagnostic's example: a page that has the problem, and the same page
/// without it. Each is checked under the explain model
/// (`tests/conformance/explain-model.toml`), with `model` laid over it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Example {
    /// A page, `page.md`, that has the problem.
    pub wrong: String,
    /// The same page, fixed.
    pub right: String,
    /// Content model text laid over the explain model, for an example that
    /// needs something it doesn't declare.
    pub model: Option<String>,
    /// Other files of the project, by content path, in path order.
    pub files: Vec<(String, String)>,
}

/// The diagnostics reference on the docs site: each code has an entry
/// there, [`Entry::docs`].
pub const REFERENCE: &str = concat!(ascribe_core::docs_site!(), "/reference/diagnostics/");

impl Entry {
    /// The address of the entry's section in the diagnostics reference,
    /// such as `…/reference/diagnostics/#asc036-link-target-missing`: its
    /// heading, `ASC036 link-target-missing`, as the site makes an anchor of
    /// it.
    pub fn docs(&self) -> String {
        format!("{REFERENCE}#{}-{}", self.code.to_lowercase(), self.slug)
    }

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
        let severity = Severity::from_name(&text("severity")?)?;
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
        let example = t.get("example").and_then(toml::Value::as_table).map(|e| {
            let text = |k: &str| e.get(k).and_then(toml::Value::as_str).map(str::to_owned);
            let mut files: Vec<(String, String)> = e
                .get("files")
                .and_then(toml::Value::as_table)
                .map(|f| {
                    f.iter()
                        .filter_map(|(k, v)| Some((k.clone(), v.as_str()?.to_owned())))
                        .collect()
                })
                .unwrap_or_default();
            files.sort();
            Example {
                wrong: text("wrong").unwrap_or_default(),
                right: text("right").unwrap_or_default(),
                model: text("model"),
                files,
            }
        });
        Some(Entry {
            code: text("code")?,
            slug: text("slug")?,
            severity,
            level,
            message: text("message")?,
            messages,
            fix: text("fix"),
            example,
            next: text("next").and_then(|n| Next::from_name(&n)),
            evidence: t
                .get("evidence")
                .and_then(toml::Value::as_array)
                .map(|a| {
                    a.iter()
                        .filter_map(|v| v.as_str().map(str::to_owned))
                        .collect()
                })
                .unwrap_or_default(),
            configurable: t
                .get("configurable")
                .and_then(toml::Value::as_bool)
                .unwrap_or(false),
            retired: t.contains_key("retired"),
        })
    }

    /// The entry for a slug.
    pub fn get(&self, slug: DiagnosticSlug) -> Option<&Entry> {
        self.entries.get(slug.as_str())
    }

    /// The entry with this slug, or this code (`ASC036`, in any case).
    pub fn find(&self, code_or_slug: &str) -> Option<&Entry> {
        self.entries.get(code_or_slug).or_else(|| {
            self.entries()
                .find(|e| e.code.eq_ignore_ascii_case(code_or_slug))
        })
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
    use ascribe_core::diagnostics;

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
            ascribe_core::Location::new(ascribe_core::FileId::new(0), 0..1),
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
