//! The diagnostics registry, `tests/conformance/diagnostics.toml`.
//!
//! The harness reads it to check the slugs cases expect: every slug must be
//! registered, file-level slugs belong in a case's top-level `diagnostics`
//! and page-level ones under `builds.<name>.diagnostics`. The file's format
//! is documented in its header.

use std::collections::BTreeMap;
use std::fmt;
use std::path::Path;

use serde::Deserialize;

/// A problem reading the registry.
#[derive(Debug, thiserror::Error)]
pub enum RegistryError {
    /// The file couldn't be read.
    #[error("couldn't read {path}: {source}")]
    Io {
        /// The file.
        path: String,
        /// The underlying error.
        source: std::io::Error,
    },
    /// The file isn't valid TOML or doesn't match the format.
    #[error("invalid diagnostics registry: {0}")]
    Toml(#[from] toml::de::Error),
}

/// A diagnostic's severity (SPEC §8.2).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    /// The document isn't conforming, and a build fails.
    Error,
    /// Worth fixing; doesn't fail a build.
    Warning,
    /// Shown, and never fails `ascribe check`; not a SPEC §8.2 severity.
    Advice,
}

/// The kind of next step a diagnostic has.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Next {
    /// Every instance has a fix Ascribe can apply.
    Fix,
    /// The author picks among things Ascribe can list.
    Choose,
    /// It needs writing or judgment.
    Write,
    /// Nothing in the source can fix it.
    Outside,
    /// It may be fine as it is.
    Review,
}

impl fmt::Display for Next {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Next::Fix => "fix",
            Next::Choose => "choose",
            Next::Write => "write",
            Next::Outside => "outside",
            Next::Review => "review",
        })
    }
}

/// When a diagnostic is checked (SPEC §8.1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Level {
    /// Each file on its own.
    File,
    /// Each expanded page, per build.
    Page,
}

impl fmt::Display for Level {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Level::File => "file",
            Level::Page => "page",
        })
    }
}

/// One registry entry.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Entry {
    /// `ASC` and three digits.
    pub code: String,
    /// The stable kebab-case name cases and crates use.
    pub slug: String,
    /// Error or warning.
    pub severity: Severity,
    /// File or page.
    pub level: Level,
    /// The kind of next step; every entry that isn't retired has one.
    #[serde(default)]
    pub next: Option<Next>,
    /// The named pieces of context an agent prompt about it carries, beyond
    /// the message and the line. Every entry that isn't retired has the
    /// list, which may be empty.
    #[serde(default)]
    pub evidence: Option<Vec<String>>,
    /// Whether a project may set its level in `[checks]`.
    #[serde(default)]
    pub configurable: bool,
    /// For a `review` entry, where it reports its problems and so where one
    /// is acknowledged: `page`, `block`, or `entry`.
    #[serde(default)]
    pub place: Option<String>,
    /// The SPEC.md section the rule comes from.
    pub spec: String,
    /// The main message template.
    pub message: String,
    /// Alternative message templates, by name.
    #[serde(default)]
    pub messages: BTreeMap<String, String>,
    /// The SPEC §8.2 row, as `<construct> | <condition>`.
    #[serde(default)]
    pub row: Option<String>,
    /// The group of the diagnostics reference a loader rule is listed in.
    #[serde(default)]
    pub group: Option<String>,
    /// Why the diagnostic was retired, if it was.
    #[serde(default)]
    pub retired: Option<String>,
    /// How to fix the problem, for the diagnostics reference.
    #[serde(default)]
    pub fix: Option<String>,
    /// A page with the problem and the same page without it, for
    /// `ascribe explain`; `ascribe-query` reads and checks it.
    #[serde(default)]
    pub example: Option<toml::Table>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct File {
    diagnostic: Vec<Entry>,
}

/// The whole registry, in file order.
#[derive(Debug, Clone)]
pub struct DiagnosticsRegistry {
    /// Every entry, in file (and code) order.
    pub entries: Vec<Entry>,
}

impl DiagnosticsRegistry {
    /// Reads the registry at `path`.
    pub fn load(path: &Path) -> Result<DiagnosticsRegistry, RegistryError> {
        let text = std::fs::read_to_string(path).map_err(|source| RegistryError::Io {
            path: path.display().to_string(),
            source,
        })?;
        DiagnosticsRegistry::parse(&text)
    }

    /// Parses a registry's text.
    pub fn parse(text: &str) -> Result<DiagnosticsRegistry, RegistryError> {
        let file: File = toml::from_str(text)?;
        Ok(DiagnosticsRegistry {
            entries: file.diagnostic,
        })
    }

    /// The entry with this slug.
    pub fn get(&self, slug: &str) -> Option<&Entry> {
        self.entries.iter().find(|e| e.slug == slug)
    }
}

/// Every `{name}` placeholder in a template, or why the template is malformed.
/// `{{` and `}}` are literal braces.
pub fn placeholders(template: &str) -> Result<Vec<&str>, String> {
    let mut out = Vec::new();
    let mut rest = template;
    while let Some(i) = rest.find(['{', '}']) {
        let after = &rest[i + 1..];
        if rest[i..].starts_with("{{") || rest[i..].starts_with("}}") {
            rest = &rest[i + 2..];
            continue;
        }
        if rest.as_bytes()[i] == b'}' {
            return Err(format!("unmatched `}}` in {template:?}; write `}}}}`"));
        }
        let Some(end) = after.find('}') else {
            return Err(format!("unclosed `{{` in {template:?}"));
        };
        let name = &after[..end];
        let valid = name.chars().next().is_some_and(|c| c.is_ascii_alphabetic())
            && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '-');
        if !valid {
            return Err(format!(
                "`{{{name}}}` in {template:?} isn't a placeholder; write literal braces as `{{{{` and `}}}}`"
            ));
        }
        out.push(name);
        rest = &after[end + 1..];
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_placeholders_and_literal_braces() {
        assert_eq!(
            placeholders("`{key}` in `[{table}]`, {{ literal }}").unwrap(),
            vec!["key", "table"]
        );
        assert_eq!(
            placeholders("{role-a} {Type}").unwrap(),
            vec!["role-a", "Type"]
        );
        assert!(placeholders("{ deployment = x }").is_err());
        assert!(placeholders("open {key").is_err());
        assert!(placeholders("close }").is_err());
    }
}
