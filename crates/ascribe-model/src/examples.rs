//! Tested examples: a source's `ascribe.examples.toml`, which says which of
//! its files are examples and how to test them, and the observations
//! `ascribe observe` writes beside it, which say what a test run passed and
//! where.
//!
//! This module reads both files and writes the second. It runs nothing: the
//! commands a config names are started by `ascribe observe` alone, in the
//! binary. On the docs side both files are data, and every string read from
//! them is made safe to show with [`shown_text`] before it reaches a page.

use std::collections::BTreeMap;
use std::fmt::Write as _;

use ascribe_core::Span;
use sha2::{Digest, Sha256};
use toml::de::{DeTable, DeValue};

use crate::names::is_key;
use crate::pattern::{Pattern, PatternError};
use crate::toml_util::{V, describe, entries, sp};

/// The config's file name, at a source's root.
pub const EXAMPLES_FILE: &str = "ascribe.examples.toml";

/// The folder beside the config that holds its observations. Nothing in it
/// is an example, whatever `include` says.
pub const OBSERVATIONS_FOLDER: &str = "examples.observations";

/// The name of the observation `ascribe observe` writes.
pub const DEFAULT_OBSERVATION: &str = "default";

/// The observation format's `version`.
pub const OBSERVATION_VERSION: i64 = 1;

/// The longest line an environment command's output is kept to, in
/// characters.
pub const KEPT_LINE_LIMIT: usize = 200;

/// The longest a label or a value is shown, in characters, before it's cut
/// with `…`.
pub const SHOWN_LIMIT: usize = 60;

/// The environment entries every observation has, which a config can't
/// declare.
pub const NOTED_ENTRIES: [&str; 2] = ["os", "arch"];

/// The path of an observation, relative to the config's folder.
pub fn observation_path(name: &str) -> String {
    format!("{OBSERVATIONS_FOLDER}/{name}.toml")
}

/// Whether the file at `path`, relative to a source's folder, is the
/// source's config or one of its observations: the files that travel with a
/// source whatever its `include` says, and that no snippet can address.
pub fn is_examples_file(path: &str) -> bool {
    path == EXAMPLES_FILE
        || path
            .strip_prefix(OBSERVATIONS_FOLDER)
            .is_some_and(|rest| rest.starts_with('/'))
}

/// Where an example stands against an observation.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Status {
    /// The observation lists the file with the hash it has now.
    Current,
    /// The observation lists the file with another hash.
    Stale,
    /// No observation lists the file.
    Unobserved,
}

/// A problem in a config or an observation, at the value it's about.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Problem {
    /// Where in the file.
    pub span: Span,
    /// What's wrong, as a clause in lower case.
    pub detail: String,
}

/// `ascribe.examples.toml`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExamplesConfig {
    /// The files that are examples, relative to the config's folder.
    pub include: Vec<Pattern>,
    /// Files `include` matches that aren't examples.
    pub ignore: Vec<Pattern>,
    /// The line of shell that tests them, read only by `ascribe observe`.
    pub test: Option<String>,
    /// What to note about where the test ran, in the order written.
    pub environment: Vec<EnvironmentEntry>,
}

/// An `[environment]` entry.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EnvironmentEntry {
    /// Its key, which the observation records its line under.
    pub key: String,
    /// What a reader sees beside its value: the `label`, or the key.
    pub label: String,
    /// The line of shell that prints it.
    pub command: String,
}

impl ExamplesConfig {
    /// Reads a config's text.
    ///
    /// # Errors
    ///
    /// Every problem found, each at the value it's about.
    pub fn parse(text: &str) -> Result<ExamplesConfig, Vec<Problem>> {
        let mut problems = Vec::new();
        let config = read_config(text, &mut problems);
        match config {
            Some(config) if problems.is_empty() => Ok(config),
            _ => Err(problems),
        }
    }

    /// Whether the file at `path`, relative to the config's folder, is an
    /// example: `include` matches it, `ignore` doesn't, and it isn't an
    /// observation.
    pub fn is_example(&self, path: &str) -> bool {
        !path.starts_with(&format!("{OBSERVATIONS_FOLDER}/"))
            && self.include.iter().any(|p| p.matches(path))
            && !self.ignore.iter().any(|p| p.matches(path))
    }
}

fn read_config(text: &str, problems: &mut Vec<Problem>) -> Option<ExamplesConfig> {
    let parsed = parse_toml(text, problems)?;
    let root = parsed.get_ref();
    for (key, span, _) in entries(root) {
        if !matches!(key, "include" | "ignore" | "test" | "environment") {
            problems.push(Problem {
                span,
                detail: format!(
                    "it has no key `{key}`; its keys are `include`, `ignore`, `test`, and `[environment]`"
                ),
            });
        }
    }
    let include = match root.get("include") {
        None => {
            problems.push(Problem {
                span: Span::empty(0),
                detail: "it has no `include`, which says which files are examples".to_owned(),
            });
            Vec::new()
        }
        Some(v) => {
            let patterns = patterns("include", v, problems);
            if patterns.is_empty() && matches!(v.get_ref(), DeValue::Array(a) if a.is_empty()) {
                problems.push(Problem {
                    span: sp(v),
                    detail: "`include` is empty, so no file is an example".to_owned(),
                });
            }
            patterns
        }
    };
    let ignore = root
        .get("ignore")
        .map(|v| patterns("ignore", v, problems))
        .unwrap_or_default();
    let test = match root.get("test") {
        None => None,
        Some(v) => match v.get_ref() {
            DeValue::String(s) if !s.trim().is_empty() => Some(s.to_string()),
            DeValue::String(_) => {
                problems.push(Problem {
                    span: sp(v),
                    detail: "`test` is empty".to_owned(),
                });
                None
            }
            other => {
                problems.push(Problem {
                    span: sp(v),
                    detail: format!("`test` is {}, not a string", describe(other)),
                });
                None
            }
        },
    };
    let mut environment = Vec::new();
    if let Some(v) = root.get("environment") {
        match v.get_ref() {
            DeValue::Table(table) => {
                for (key, key_span, entry) in entries(table) {
                    if let Some(entry) = environment_entry(key, key_span, entry, problems) {
                        environment.push(entry);
                    }
                }
            }
            other => problems.push(Problem {
                span: sp(v),
                detail: format!("`environment` is {}, not a table", describe(other)),
            }),
        }
    }
    Some(ExamplesConfig {
        include,
        ignore,
        test,
        environment,
    })
}

fn patterns(key: &str, v: &V<'_>, problems: &mut Vec<Problem>) -> Vec<Pattern> {
    let DeValue::Array(items) = v.get_ref() else {
        problems.push(Problem {
            span: sp(v),
            detail: format!(
                "`{key}` is {}, not a list of patterns",
                describe(v.get_ref())
            ),
        });
        return Vec::new();
    };
    let mut out = Vec::new();
    for item in items.iter() {
        let DeValue::String(text) = item.get_ref() else {
            problems.push(Problem {
                span: sp(item),
                detail: format!(
                    "a pattern in `{key}` is {}, not a string",
                    describe(item.get_ref())
                ),
            });
            continue;
        };
        match Pattern::new(text) {
            Ok(pattern) => out.push(pattern),
            Err(e) => problems.push(Problem {
                span: sp(item),
                detail: format!("`{text}` in `{key}` isn't a pattern: {}", pattern_error(&e)),
            }),
        }
    }
    out
}

fn pattern_error(e: &PatternError) -> String {
    match e {
        PatternError::LeadingSlash => {
            "it starts with `/`, and patterns are from the config's folder".to_owned()
        }
        PatternError::Parent => "it has a `..` segment".to_owned(),
        PatternError::Syntax(s) => s.clone(),
    }
}

fn environment_entry(
    key: &str,
    key_span: Span,
    v: &V<'_>,
    problems: &mut Vec<Problem>,
) -> Option<EnvironmentEntry> {
    if NOTED_ENTRIES.contains(&key) {
        problems.push(Problem {
            span: key_span,
            detail: format!(
                "`environment.{key}` is noted in every observation without being asked for, so a config can't declare it"
            ),
        });
        return None;
    }
    if !is_key(key) {
        problems.push(Problem {
            span: key_span,
            detail: format!(
                "`{key}` isn't an environment entry's name: a lowercase letter, then lowercase letters, digits, or hyphens"
            ),
        });
        return None;
    }
    let DeValue::Table(table) = v.get_ref() else {
        problems.push(Problem {
            span: sp(v),
            detail: format!(
                "`environment.{key}` is {}, not a table with a `command` and a `label`",
                describe(v.get_ref())
            ),
        });
        return None;
    };
    for (name, span, _) in entries(table) {
        if !matches!(name, "label" | "command") {
            problems.push(Problem {
                span,
                detail: format!(
                    "`environment.{key}` has no key `{name}`; its keys are `label` and `command`"
                ),
            });
        }
    }
    let mut string = |name: &str| -> Option<String> {
        let value = table.get(name)?;
        match value.get_ref() {
            DeValue::String(s) if !s.trim().is_empty() => Some(s.to_string()),
            other => {
                let found = match other {
                    DeValue::String(_) => "empty",
                    other => describe(other),
                };
                problems.push(Problem {
                    span: sp(value),
                    detail: format!("`environment.{key}.{name}` is {found}, not a string"),
                });
                None
            }
        }
    };
    let label = string("label");
    let command = string("command");
    if !table.contains_key("command") {
        problems.push(Problem {
            span: sp(v),
            detail: format!("`environment.{key}` has no `command`"),
        });
    }
    Some(EnvironmentEntry {
        key: key.to_owned(),
        label: label.unwrap_or_else(|| key.to_owned()),
        command: command?,
    })
}

/// An observation: what a test run passed, and where.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Observation {
    /// The version of `ascribe` that wrote it.
    pub ascribe: String,
    /// The test command, as the config had it.
    pub command: String,
    /// The environment: `os` and `arch`, then each config entry's kept line,
    /// in the order written.
    pub environment: Vec<(String, String)>,
    /// Each example's hash ([`example_hash`]), by path from the config's
    /// folder.
    pub files: BTreeMap<String, String>,
}

impl Observation {
    /// Reads an observation's text.
    ///
    /// # Errors
    ///
    /// Every problem found, each at the value it's about.
    pub fn parse(text: &str) -> Result<Observation, Vec<Problem>> {
        let mut problems = Vec::new();
        let observation = read_observation(text, &mut problems);
        match observation {
            Some(observation) if problems.is_empty() => Ok(observation),
            _ => Err(problems),
        }
    }

    /// Where the example at `path` (from the config's folder), whose bytes
    /// are `bytes`, stands against this observation.
    pub fn status(&self, path: &str, bytes: &[u8]) -> Status {
        match self.files.get(path) {
            None => Status::Unobserved,
            Some(hash) if *hash == example_hash(bytes) => Status::Current,
            Some(_) => Status::Stale,
        }
    }

    /// An environment entry's value.
    pub fn value(&self, key: &str) -> Option<&str> {
        self.environment
            .iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v.as_str())
    }

    /// The observation as `ascribe observe` writes it: the same text for the
    /// same observation on every platform, the files by path.
    pub fn to_toml(&self) -> String {
        let mut out = String::from("# Written by `ascribe observe`. Don't edit it by hand.\n");
        let _ = writeln!(out, "version = {OBSERVATION_VERSION}");
        let _ = writeln!(out, "ascribe = {}", quoted(&self.ascribe));
        let _ = writeln!(out, "command = {}", quoted(&self.command));
        out.push_str("\n[environment]\n");
        for (key, value) in &self.environment {
            let _ = writeln!(out, "{key} = {}", quoted(value));
        }
        out.push_str("\n[files]\n");
        for (path, hash) in &self.files {
            let _ = writeln!(out, "{} = {}", quoted(path), quoted(hash));
        }
        out
    }

    /// What the page says it was tested with: each of `config`'s entries'
    /// label and shown value, in the config's order, then the operating
    /// system, as in `Node 24.9.0 and TypeScript 5.9.2 on Linux`. An entry
    /// the observation has no value for is left out. Every label and value
    /// is made safe to show and cut ([`shown_text`]).
    pub fn tested_line(&self, config: &ExamplesConfig) -> String {
        let parts: Vec<String> = config
            .environment
            .iter()
            .filter_map(|entry| {
                let value = self.value(&entry.key)?;
                Some(format!(
                    "{} {}",
                    shown_text(&entry.label),
                    shown_text(shown_value(value))
                ))
            })
            .collect();
        let os = os_name(self.value("os").unwrap_or_default());
        if parts.is_empty() {
            os
        } else {
            format!("{} on {os}", and_list(&parts))
        }
    }
}

fn read_observation(text: &str, problems: &mut Vec<Problem>) -> Option<Observation> {
    let parsed = parse_toml(text, problems)?;
    let root = parsed.get_ref();
    for (key, span, _) in entries(root) {
        if !matches!(
            key,
            "version" | "ascribe" | "command" | "environment" | "files"
        ) {
            problems.push(Problem {
                span,
                detail: format!("it has no key `{key}`"),
            });
        }
    }
    match root.get("version") {
        None => problems.push(Problem {
            span: Span::empty(0),
            detail: "it has no `version`".to_owned(),
        }),
        Some(v) => match v.get_ref() {
            DeValue::Integer(i) if i.as_str() == OBSERVATION_VERSION.to_string() => {}
            DeValue::Integer(i) => problems.push(Problem {
                span: sp(v),
                detail: format!(
                    "it's version {}, and this version of Ascribe reads version {OBSERVATION_VERSION}",
                    i.as_str()
                ),
            }),
            other => problems.push(Problem {
                span: sp(v),
                detail: format!("`version` is {}, not a number", describe(other)),
            }),
        },
    }
    let mut string = |key: &str| -> Option<String> {
        let Some(v) = root.get(key) else {
            problems.push(Problem {
                span: Span::empty(0),
                detail: format!("it has no `{key}`"),
            });
            return None;
        };
        match v.get_ref() {
            DeValue::String(s) => Some(s.to_string()),
            other => {
                problems.push(Problem {
                    span: sp(v),
                    detail: format!("`{key}` is {}, not a string", describe(other)),
                });
                None
            }
        }
    };
    let ascribe = string("ascribe");
    let command = string("command");
    let mut environment = Vec::new();
    match root.get("environment").map(|v| (v, v.get_ref())) {
        Some((_, DeValue::Table(table))) => {
            for (key, _, value) in entries(table) {
                match value.get_ref() {
                    DeValue::String(s) => environment.push((key.to_owned(), s.to_string())),
                    other => problems.push(Problem {
                        span: sp(value),
                        detail: format!("`environment.{key}` is {}, not a string", describe(other)),
                    }),
                }
            }
        }
        Some((v, other)) => problems.push(Problem {
            span: sp(v),
            detail: format!("`environment` is {}, not a table", describe(other)),
        }),
        None => problems.push(Problem {
            span: Span::empty(0),
            detail: "it has no `[environment]`".to_owned(),
        }),
    }
    let mut files = BTreeMap::new();
    match root.get("files").map(|v| (v, v.get_ref())) {
        Some((_, DeValue::Table(table))) => {
            for (path, _, hash) in entries(table) {
                match hash.get_ref() {
                    DeValue::String(s) if is_example_hash(s) => {
                        files.insert(path.to_owned(), s.to_string());
                    }
                    DeValue::String(s) => problems.push(Problem {
                        span: sp(hash),
                        detail: format!(
                            "the hash of `{path}`, `{s}`, isn't `sha256-lf:` and 64 lowercase hexadecimal digits"
                        ),
                    }),
                    other => problems.push(Problem {
                        span: sp(hash),
                        detail: format!("the hash of `{path}` is {}, not a string", describe(other)),
                    }),
                }
            }
        }
        Some((v, other)) => problems.push(Problem {
            span: sp(v),
            detail: format!("`files` is {}, not a table", describe(other)),
        }),
        None => {}
    }
    Some(Observation {
        ascribe: ascribe?,
        command: command?,
        environment,
        files,
    })
}

fn parse_toml<'i>(
    text: &'i str,
    problems: &mut Vec<Problem>,
) -> Option<toml::Spanned<DeTable<'i>>> {
    match DeTable::parse(text) {
        Ok(parsed) => Some(parsed),
        Err(e) => {
            let span = e
                .span()
                .map_or(Span::empty(0), |r| Span::new(r.start, r.end));
            problems.push(Problem {
                span,
                detail: e.message().trim().to_owned(),
            });
            None
        }
    }
}

/// An example file's hash, as an observation records it: SHA-256 of its
/// bytes with each CR LF read as LF, so a file hashes the same on a Windows
/// checkout and a Linux one (decision #458a). `sha256-lf:` and 64 lowercase
/// hexadecimal digits.
pub fn example_hash(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    let mut start = 0;
    for (i, pair) in bytes.windows(2).enumerate() {
        if pair == b"\r\n" {
            hasher.update(&bytes[start..i]);
            start = i + 1;
        }
    }
    hasher.update(&bytes[start..]);
    let mut out = String::with_capacity(10 + 64);
    out.push_str("sha256-lf:");
    for byte in hasher.finalize() {
        let _ = write!(out, "{byte:02x}");
    }
    out
}

fn is_example_hash(text: &str) -> bool {
    text.strip_prefix("sha256-lf:").is_some_and(|hex| {
        hex.len() == 64
            && hex
                .bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    })
}

/// What's kept of an environment command's output: its first line, trimmed,
/// cut at [`KEPT_LINE_LIMIT`] characters. `None` when it's empty.
pub fn kept_line(output: &str) -> Option<String> {
    let line = output.lines().next()?.trim();
    if line.is_empty() {
        return None;
    }
    Some(line.chars().take(KEPT_LINE_LIMIT).collect())
}

/// What's shown of a kept line: its first dotted number (`24.9.0` from
/// `v24.9.0`, `1.22.3` from `go version go1.22.3 darwin/arm64`), or the whole
/// line when it has none.
pub fn shown_value(kept: &str) -> &str {
    let bytes = kept.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i].is_ascii_digit() {
            let start = i;
            let mut end = i;
            let mut dotted = false;
            while end < bytes.len() && bytes[end].is_ascii_digit() {
                end += 1;
            }
            while end + 1 < bytes.len() && bytes[end] == b'.' && bytes[end + 1].is_ascii_digit() {
                dotted = true;
                end += 1;
                while end < bytes.len() && bytes[end].is_ascii_digit() {
                    end += 1;
                }
            }
            if dotted {
                return &kept[start..end];
            }
            i = end;
        } else {
            i += 1;
        }
    }
    kept
}

/// An operating system by a name a reader knows: `Linux`, `macOS`,
/// `Windows`, or the name as recorded, made safe to show.
pub fn os_name(os: &str) -> String {
    match os {
        "linux" => "Linux".to_owned(),
        "macos" => "macOS".to_owned(),
        "windows" => "Windows".to_owned(),
        other => shown_text(other),
    }
}

/// Text from a config or an observation, made safe to put in a line a page
/// shows: control characters become spaces, runs of white space one space,
/// and anything past [`SHOWN_LIMIT`] characters is cut, with `…`. Escaping
/// for the place it's shown (HTML, Markdown, an info string) is that
/// output's.
pub fn shown_text(text: &str) -> String {
    let mut out = String::new();
    let mut count = 0;
    let mut space = false;
    for c in text.trim().chars() {
        if c.is_control() || c.is_whitespace() {
            space = true;
            continue;
        }
        let pending = usize::from(space && !out.is_empty());
        space = false;
        if count + pending >= SHOWN_LIMIT {
            out.push('…');
            return out;
        }
        if pending == 1 {
            out.push(' ');
            count += 1;
        }
        out.push(c);
        count += 1;
    }
    out
}

/// `a`, `a and b`, `a, b, and c`.
fn and_list(parts: &[String]) -> String {
    match parts {
        [] => String::new(),
        [one] => one.clone(),
        [a, b] => format!("{a} and {b}"),
        [rest @ .., last] => format!("{}, and {last}", rest.join(", ")),
    }
}

/// A TOML basic string.
fn quoted(text: &str) -> String {
    let mut out = String::with_capacity(text.len() + 2);
    out.push('"');
    for c in text.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\t' => out.push_str("\\t"),
            '\r' => out.push_str("\\r"),
            c if c.is_control() => {
                let _ = write!(out, "\\u{:04X}", c as u32);
            }
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    const CONFIG: &str = r#"
include = ["examples/**"]
ignore = ["**/node_modules/**"]
test = "npm test"

[environment]
node = { label = "Node", command = "node --version" }
typescript = { label = "TypeScript", command = "npx tsc --version" }
"#;

    fn details(problems: Vec<Problem>) -> Vec<String> {
        problems.into_iter().map(|p| p.detail).collect()
    }

    #[test]
    fn reads_a_config() {
        let config = ExamplesConfig::parse(CONFIG).expect("valid");
        assert_eq!(config.test.as_deref(), Some("npm test"));
        assert_eq!(config.environment.len(), 2);
        assert_eq!(config.environment[1].label, "TypeScript");
        assert!(config.is_example("examples/a.ts"));
        assert!(!config.is_example("examples/node_modules/x/a.ts"));
        assert!(!config.is_example("src/a.ts"));
        let all = ExamplesConfig::parse("include = [\"**\"]\n").expect("valid");
        assert!(all.is_example("a.ts"));
        assert!(!all.is_example("examples.observations/default.toml"));
        let unlabelled = ExamplesConfig::parse(
            "include = [\"*\"]\n[environment.go]\ncommand = \"go version\"\n",
        )
        .expect("valid");
        assert_eq!(unlabelled.environment[0].label, "go");
    }

    #[test]
    fn the_files_that_travel_with_a_source() {
        assert!(is_examples_file("ascribe.examples.toml"));
        assert!(is_examples_file("examples.observations/default.toml"));
        assert!(is_examples_file("examples.observations/x/y.toml"));
        assert!(!is_examples_file("examples.observations"));
        assert!(!is_examples_file("examples.observationsx/a.toml"));
        assert!(!is_examples_file("code/ascribe.examples.toml"));
        assert!(!is_examples_file("examples/a.ts"));
    }

    #[test]
    fn an_examples_status() {
        let observation = Observation {
            ascribe: "0.3.0".to_owned(),
            command: "npm test".to_owned(),
            environment: Vec::new(),
            files: BTreeMap::from([("a.ts".to_owned(), example_hash(b"a\n"))]),
        };
        assert_eq!(observation.status("a.ts", b"a\r\n"), Status::Current);
        assert_eq!(observation.status("a.ts", b"b\n"), Status::Stale);
        assert_eq!(observation.status("b.ts", b"a\n"), Status::Unobserved);
    }

    #[test]
    fn config_problems() {
        let bad = |text: &str| details(ExamplesConfig::parse(text).expect_err(text));
        assert_eq!(
            bad("test = \"x\"\n"),
            ["it has no `include`, which says which files are examples"]
        );
        assert_eq!(
            bad("include = []\n"),
            ["`include` is empty, so no file is an example"]
        );
        assert_eq!(
            bad("include = [\"/a\"]\n"),
            [
                "`/a` in `include` isn't a pattern: it starts with `/`, and patterns are from the config's folder"
            ]
        );
        assert_eq!(
            bad("include = [\"*\"]\nother = 1\n"),
            [
                "it has no key `other`; its keys are `include`, `ignore`, `test`, and `[environment]`"
            ]
        );
        assert_eq!(
            bad("include = [\"*\"]\n[environment.os]\ncommand = \"uname\"\n"),
            [
                "`environment.os` is noted in every observation without being asked for, so a config can't declare it"
            ]
        );
        assert_eq!(
            bad("include = [\"*\"]\n[environment.node]\nlabel = \"Node\"\n"),
            ["`environment.node` has no `command`"]
        );
        assert_eq!(
            bad("include = [\"*\"]\n[environment.node]\ncommand = \"x\"\nlang = \"y\"\n"),
            ["`environment.node` has no key `lang`; its keys are `label` and `command`"]
        );
        assert!(bad("include = ")[0].contains("string"));
    }

    fn observation() -> Observation {
        Observation {
            ascribe: "0.3.0".into(),
            command: "npm test".into(),
            environment: vec![
                ("os".into(), "linux".into()),
                ("arch".into(), "x64".into()),
                ("node".into(), "v24.9.0".into()),
                ("typescript".into(), "Version 5.9.2".into()),
            ],
            files: BTreeMap::from([
                ("examples/b.ts".into(), example_hash(b"b\n")),
                ("examples/a \"q\".ts".into(), example_hash(b"a\n")),
            ]),
        }
    }

    #[test]
    fn observations_are_written_and_read_back() {
        let text = observation().to_toml();
        assert!(text.find("examples/a") < text.find("examples/b.ts"));
        let read = Observation::parse(&text).expect("it reads back");
        assert_eq!(read, observation());
        assert_eq!(read.to_toml(), text);
    }

    #[test]
    fn observation_problems() {
        let bad = |text: &str| details(Observation::parse(text).expect_err(text));
        assert_eq!(
            bad("version = 2\nascribe = \"1\"\ncommand = \"x\"\n[environment]\n"),
            ["it's version 2, and this version of Ascribe reads version 1"]
        );
        assert_eq!(
            bad(
                "version = 1\nascribe = \"1\"\ncommand = \"x\"\n[environment]\n[files]\n\"a\" = \"sha256:00\"\n"
            ),
            [
                "the hash of `a`, `sha256:00`, isn't `sha256-lf:` and 64 lowercase hexadecimal digits"
            ]
        );
    }

    #[test]
    fn hashes_read_crlf_as_lf() {
        assert_eq!(example_hash(b"a\r\nb\r\n"), example_hash(b"a\nb\n"));
        assert_eq!(example_hash(b"a\rb"), example_hash(b"a\rb"));
        assert_ne!(example_hash(b"a\rb"), example_hash(b"a\nb"));
        assert_eq!(example_hash(b"\r\n\r\n"), example_hash(b"\n\n"));
        assert_eq!(
            example_hash(b""),
            "sha256-lf:e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }

    #[test]
    fn shows_the_first_dotted_number() {
        assert_eq!(shown_value("v24.9.0"), "24.9.0");
        assert_eq!(shown_value("Version 5.9.2"), "5.9.2");
        assert_eq!(shown_value("go version go1.22.3 darwin/arm64"), "1.22.3");
        assert_eq!(shown_value("Python 3"), "Python 3");
        assert_eq!(shown_value("1. 2.x 3.4"), "3.4");
        assert_eq!(kept_line("v24.9.0\r\nmore\n"), Some("v24.9.0".to_owned()));
        assert_eq!(kept_line("\n"), None);
        assert_eq!(kept_line(&"x".repeat(500)).map(|l| l.len()), Some(200));
    }

    #[test]
    fn the_tested_line() {
        let config = ExamplesConfig::parse(CONFIG).expect("valid");
        assert_eq!(
            observation().tested_line(&config),
            "Node 24.9.0 and TypeScript 5.9.2 on Linux"
        );
        let none = ExamplesConfig::parse("include = [\"*\"]\n").expect("valid");
        assert_eq!(observation().tested_line(&none), "Linux");
        let three = ExamplesConfig::parse(
            "include = [\"*\"]\n[environment]\na = { command = \"x\" }\nb = { command = \"x\" }\nc = { command = \"x\" }\n",
        )
        .expect("valid");
        let mut obs = observation();
        obs.environment = vec![
            ("os".into(), "macos".into()),
            ("a".into(), "1.0".into()),
            ("b".into(), "2.0".into()),
            ("c".into(), "3.0".into()),
        ];
        assert_eq!(obs.tested_line(&three), "a 1.0, b 2.0, and c 3.0 on macOS");
    }

    #[test]
    fn shown_text_is_safe_and_cut() {
        assert_eq!(shown_text("a\nb\u{1b}[31m c"), "a b [31m c");
        let long = shown_text(&"x".repeat(5000));
        assert_eq!(long.chars().count(), SHOWN_LIMIT + 1);
        // A space at the limit isn't kept before the ellipsis.
        let spaced = shown_text(&format!("{} {}", "x".repeat(SHOWN_LIMIT), "y"));
        assert!(spaced.ends_with("x…"), "{spaced}");
        assert!(long.ends_with('…'));
        assert_eq!(shown_text("<script>"), "<script>");
    }
}
