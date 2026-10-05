//! `docs/content/reference/diagnostics.md` is generated from the diagnostics registry. This test
//! renders it and fails when the file is out of date; run it with
//! `ASCRIBE_BLESS=1` to rewrite the file.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::path::PathBuf;

use tessera_conformance::{DiagnosticsRegistry, Entry, Level, Severity, Suite};

fn repo() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

#[test]
fn every_diagnostic_says_how_to_fix_it() {
    let registry = DiagnosticsRegistry::load(&Suite::bundled().diagnostics_path()).unwrap();
    let missing: Vec<&str> = registry
        .entries
        .iter()
        .filter(|e| e.retired.is_none() && e.fix.as_deref().is_none_or(str::is_empty))
        .map(|e| e.code.as_str())
        .collect();
    assert!(missing.is_empty(), "entries without a `fix`: {missing:?}");
}

#[test]
fn the_diagnostics_reference_is_current() {
    let registry = DiagnosticsRegistry::load(&Suite::bundled().diagnostics_path()).unwrap();
    let spec = std::fs::read_to_string(repo().join("SPEC.md")).unwrap();
    let rendered = render(&registry, &spec_anchors(&spec));
    let path = repo().join("docs/content/reference/diagnostics.md");
    if std::env::var_os("ASCRIBE_BLESS").is_some() {
        std::fs::write(&path, &rendered).unwrap();
        return;
    }
    let current = std::fs::read_to_string(&path).unwrap_or_default();
    assert!(
        current == rendered,
        "docs/content/reference/diagnostics.md is out of date: run \
         `ASCRIBE_BLESS=1 cargo test -p tessera-conformance --test docs`"
    );
}

/// The groups loader rules are listed in, as they're titled in the
/// diagnostics reference.
const RULE_GROUPS: &[(&str, &str)] = &[
    ("file", "The file"),
    ("project", "`[project]`"),
    ("content-types", "Content types, fields, and attributes"),
    (
        "names",
        "Dimensions, names, lifecycle states, notes, and features",
    ),
    ("phrases", "Phrases and the glossary"),
    ("widgets", "Widgets"),
    ("consumer", "The consumer, builds, and the editor"),
];

fn render(registry: &DiagnosticsRegistry, anchors: &BTreeMap<String, String>) -> String {
    let active: Vec<&Entry> = registry
        .entries
        .iter()
        .filter(|e| e.retired.is_none())
        .collect();
    let mut out = String::new();
    out.push_str(
        "---\n\
         title: Diagnostics\n\
         description: Every problem Ascribe reports, with its code and fix.\n\
         ---\n\n\
         <!-- Generated from tests/conformance/diagnostics.toml by \
         tests/conformance/tests/docs.rs. Edit the registry, then run \
         `ASCRIBE_BLESS=1 cargo test -p tessera-conformance --test docs`. -->\n\n\
         Every problem Ascribe reports, with its code, its name, and how to fix it. \
         `ascribe check`, `ascribe build`, and the editor report the same diagnostics, \
         with the same codes.\n\n\
         - An **error** makes `ascribe check` fail, and stops `ascribe build` from writing \
         anything. A **warning** doesn't, unless you pass `--deny-warnings`.\n\
         - A **file-level** diagnostic is about one file on its own. A **page-level** \
         diagnostic is about a page after its includes are expanded and a build's modes \
         are applied, so it can depend on the build; the message names the builds it \
         appears in.\n\
         - A diagnostic about `ascribe.toml` (a name that starts with `model-`) stops \
         everything else when it's an error: every other check depends on the content \
         model.\n\
         - In the messages below, `{name}` stands for a value filled in from your source.\n\n\
         In the editor, many diagnostics offer a quick fix. See [Editing](../guides/editor.md).\n\n",
    );

    // The index.
    out.push_str("| Code | Name | Severity | Level |\n|---|---|---|---|\n");
    for entry in &active {
        let _ = writeln!(
            out,
            "| [{code}](#{anchor}) | `{slug}` | {severity} | {level} |",
            code = entry.code,
            anchor = heading_anchor(entry),
            slug = entry.slug,
            severity = severity(entry.severity),
            level = level(entry.level),
        );
    }

    // Source-file diagnostics, grouped by the construct their SPEC row names,
    // in the order each construct first appears.
    out.push_str("\n## Source files\n");
    let mut constructs: Vec<(&str, Vec<(&Entry, &str)>)> = Vec::new();
    for entry in active.iter().filter(|e| !is_model(e)) {
        let row = entry.row.as_deref().unwrap_or_default();
        let (construct, condition) = row.split_once(" | ").unwrap_or((row, ""));
        match constructs.iter_mut().find(|(c, _)| *c == construct) {
            Some((_, entries)) => entries.push((entry, condition)),
            None => constructs.push((construct, vec![(entry, condition)])),
        }
    }
    for (construct, entries) in constructs {
        let _ = write!(out, "\n### {construct}\n");
        for (entry, condition) in entries {
            write_entry(&mut out, entry, Some(condition), anchors);
        }
    }

    out.push_str("\n## The content model\n");
    for (group, title) in RULE_GROUPS {
        let _ = write!(out, "\n### {title}\n");
        for entry in active
            .iter()
            .filter(|e| is_model(e) && model_group(e) == *group)
        {
            write_entry(&mut out, entry, None, anchors);
        }
    }

    let retired: Vec<&Entry> = registry
        .entries
        .iter()
        .filter(|e| e.retired.is_some())
        .collect();
    if !retired.is_empty() {
        out.push_str(
            "\n## Retired\n\nNo longer reported. Their codes and names aren't reused.\n\n",
        );
        for entry in retired {
            let _ = writeln!(
                out,
                "- {} `{}`: {}",
                entry.code,
                entry.slug,
                entry.retired.as_deref().unwrap_or_default()
            );
        }
    }
    out
}

fn is_model(entry: &Entry) -> bool {
    entry.slug.starts_with("model-")
}

/// The group a content-model diagnostic is listed in. The one-role rule for
/// names is a SPEC §8.2 row rather than a loader rule, so it has no `group`;
/// it goes with the other rules about names.
fn model_group(entry: &Entry) -> &str {
    entry.group.as_deref().unwrap_or("names")
}

fn write_entry(
    out: &mut String,
    entry: &Entry,
    condition: Option<&str>,
    anchors: &BTreeMap<String, String>,
) {
    let section = match anchors.get(&entry.spec) {
        Some(anchor) => format!(
            "[SPEC §{}]({{repo}}/blob/main/SPEC.md#{anchor})",
            entry.spec
        ),
        None => format!("SPEC §{}", entry.spec),
    };
    let _ = write!(
        out,
        "\n#### {} `{}`\n\n{} · {} level · {section}\n\n",
        entry.code,
        entry.slug,
        severity(entry.severity),
        level(entry.level).to_lowercase(),
    );
    if let Some(condition) = condition {
        let _ = write!(
            out,
            "**When:** {}.\n\n",
            escape_braces(&sentence(condition))
        );
    }
    let _ = write!(
        out,
        "**Message:** {}\n\n",
        escape_braces(&message(&entry.message))
    );
    let _ = writeln!(
        out,
        "**Fix:** {}",
        escape_braces(entry.fix.as_deref().unwrap_or_default())
    );
}

fn severity(severity: Severity) -> &'static str {
    match severity {
        Severity::Error => "Error",
        Severity::Warning => "Warning",
    }
}

fn level(level: Level) -> &'static str {
    match level {
        Level::File => "File",
        Level::Page => "Page",
    }
}

/// A condition from the spec's table, as a sentence: its first letter
/// capitalized, and the table's "(page level)" notes left out, since the
/// entry's level says so.
fn sentence(condition: &str) -> String {
    let text = condition
        .replace(" (page level, per build)", "")
        .replace(" (page level)", "")
        .replace(
            ", including ids from included content (page level)",
            ", including ids from included content",
        );
    let mut chars = text.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().chain(chars).collect(),
        None => text,
    }
}

/// A message template as it reads in the reference: a template's literal
/// braces (`{{`, `}}`) are single braces.
fn message(template: &str) -> String {
    template.replace("{{", "{").replace("}}", "}")
}

/// Text with a backslash before each `{` and `<` outside code spans, so a
/// placeholder such as `{value}` or `<version>` reads as itself, and is never
/// taken for one of the docs' phrases or for HTML.
fn escape_braces(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut fence = 0;
    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '`' {
            let mut run = 1;
            while chars.peek() == Some(&'`') {
                chars.next();
                run += 1;
            }
            fence = match fence {
                0 => run,
                open if open == run => 0,
                open => open,
            };
            out.extend(std::iter::repeat_n('`', run));
            continue;
        }
        if matches!(c, '{' | '<') && fence == 0 {
            out.push('\\');
        }
        out.push(c);
    }
    out
}

/// The anchor of an entry's heading, as GitHub computes it.
fn heading_anchor(entry: &Entry) -> String {
    slug(&format!("{} {}", entry.code, entry.slug))
}

/// SPEC.md's numbered section headings, by number: `3.3` to its anchor.
fn spec_anchors(spec: &str) -> BTreeMap<String, String> {
    let mut anchors = BTreeMap::new();
    for line in spec.lines() {
        let Some(text) = line
            .strip_prefix("## ")
            .or_else(|| line.strip_prefix("### "))
        else {
            continue;
        };
        let Some((number, _)) = text.split_once(' ') else {
            continue;
        };
        let number = number.trim_end_matches('.');
        if !number.is_empty() && number.chars().all(|c| c.is_ascii_digit() || c == '.') {
            anchors.insert(number.to_owned(), slug(text));
        }
    }
    anchors
}

/// A heading's anchor, as GitHub computes it: lowercase, with every character
/// that isn't a letter, digit, space, hyphen, or underscore removed, and
/// spaces turned into hyphens.
fn slug(text: &str) -> String {
    text.to_lowercase()
        .chars()
        .filter(|c| c.is_alphanumeric() || matches!(c, ' ' | '-' | '_'))
        .map(|c| if c == ' ' { '-' } else { c })
        .collect()
}
