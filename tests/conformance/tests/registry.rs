//! Checks the diagnostics registry, `tests/conformance/diagnostics.toml`,
//! against SPEC.md: every §8.2 row and the sections it cites.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

use tessera_conformance::registry::placeholders;
use tessera_conformance::{DiagnosticsRegistry, Entry, Level, Severity, Suite};

fn repo() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn read(rel: &str) -> String {
    std::fs::read_to_string(repo().join(rel)).unwrap_or_else(|e| panic!("{rel}: {e}"))
}

fn registry() -> DiagnosticsRegistry {
    DiagnosticsRegistry::load(&Suite::bundled().diagnostics_path()).unwrap()
}

/// Splits a markdown table row into cells, honoring `\|` escapes.
fn cells(line: &str) -> Vec<String> {
    let inner = line.trim().trim_start_matches('|').trim_end_matches('|');
    let mut out = vec![String::new()];
    let mut chars = inner.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '\\' if chars.peek() == Some(&'|') => {
                chars.next();
                out.last_mut().unwrap().push('|');
            }
            '|' => out.push(String::new()),
            c => out.last_mut().unwrap().push(c),
        }
    }
    out.into_iter().map(|c| c.trim().to_owned()).collect()
}

/// SPEC §8.2's rows: `<construct> | <condition>` and severity.
fn spec_rows() -> Vec<(String, Severity)> {
    let spec = read("SPEC.md");
    let section = spec
        .split("### 8.2 Diagnostics")
        .nth(1)
        .and_then(|s| s.split("### 8.3").next())
        .unwrap();
    section
        .lines()
        .filter(|l| l.starts_with("| ") && !l.starts_with("| Construct"))
        .map(|l| {
            let c = cells(l);
            let severity = match c[2].as_str() {
                "Error" => Severity::Error,
                "Warning" => Severity::Warning,
                other => panic!("severity {other:?}"),
            };
            (format!("{} | {}", c[0], c[1]), severity)
        })
        .collect()
}

fn templates(e: &Entry) -> Vec<&String> {
    std::iter::once(&e.message)
        .chain(e.messages.values())
        .collect()
}

#[test]
fn every_spec_row_has_exactly_one_entry() {
    let reg = registry();
    let rows = spec_rows();
    assert_eq!(rows.len(), 78, "SPEC §8.2 changed; update the registry");
    let mut by_row: BTreeMap<&str, Vec<&Entry>> = BTreeMap::new();
    for e in &reg.entries {
        if let Some(row) = &e.row {
            by_row.entry(row).or_default().push(e);
        }
    }
    for (row, severity) in &rows {
        let entries = by_row
            .remove(row.as_str())
            .unwrap_or_else(|| panic!("no entry for SPEC §8.2 row {row:?}"));
        let slugs: Vec<&str> = entries.iter().map(|e| e.slug.as_str()).collect();
        assert_eq!(slugs.len(), 1, "row {row:?} has several entries: {slugs:?}");
        let e = entries[0];
        assert_eq!(&e.severity, severity, "{}: severity of {row:?}", e.slug);
        // SPEC marks some page-level rows; the rest take their level from §8.1.
        if row.contains("(page level") {
            assert_eq!(e.level, Level::Page, "{}: {row:?} is page level", e.slug);
        }
    }
    assert!(
        by_row.is_empty(),
        "entries name rows SPEC §8.2 doesn't have: {:?}",
        by_row.keys()
    );
}

#[test]
fn loader_rules_are_file_level_and_grouped() {
    for e in registry().entries {
        let is_rule = e.slug.starts_with("model-");
        assert_eq!(
            e.group.is_some(),
            is_rule,
            "{}: `group` is for loader rules, which start with model-",
            e.slug
        );
        if is_rule {
            assert_eq!(e.level, Level::File, "{}", e.slug);
        }
    }
}

#[test]
fn codes_are_sequential_and_slugs_unique() {
    let reg = registry();
    let mut slugs = BTreeSet::new();
    for (i, e) in reg.entries.iter().enumerate() {
        assert_eq!(
            e.code,
            format!("ASC{:03}", i + 1),
            "codes go up by one, in file order"
        );
        assert!(slugs.insert(&e.slug), "{} appears twice", e.slug);
        let kebab = e.slug.split('-').all(|w| {
            !w.is_empty()
                && w.chars()
                    .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit())
        });
        assert!(kebab, "{} isn't kebab-case", e.slug);
        assert!(
            e.row.is_some() || e.group.is_some(),
            "{}: every entry comes from a §8.2 row or a loader rule",
            e.slug
        );
    }
}

#[test]
fn messages_are_well_formed() {
    for e in registry().entries {
        for t in templates(&e) {
            placeholders(t).unwrap_or_else(|err| panic!("{}: {err}", e.slug));
            assert!(!t.trim().is_empty(), "{}: empty message", e.slug);
        }
        for name in e.messages.keys() {
            assert!(
                name.chars().all(|c| c.is_ascii_lowercase() || c == '-'),
                "{}: message name {name:?}",
                e.slug
            );
        }
    }
}

#[test]
fn spec_sections_exist() {
    let spec = read("SPEC.md");
    let sections: BTreeSet<String> = spec
        .lines()
        .filter_map(|l| {
            let rest = l.strip_prefix("### ").or_else(|| l.strip_prefix("## "))?;
            let number = rest.split(' ').next()?.trim_end_matches('.');
            number
                .chars()
                .all(|c| c.is_ascii_digit() || c == '.')
                .then(|| number.to_owned())
        })
        .collect();
    assert!(sections.contains("8.2") && sections.contains("11"));
    for e in registry().entries {
        assert!(
            sections.contains(&e.spec),
            "{}: SPEC has no §{}",
            e.slug,
            e.spec
        );
    }
}
