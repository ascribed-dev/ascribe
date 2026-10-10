//! The diagnostics reference, `docs/content/reference/diagnostics.md`, includes
//! fragments generated from the diagnostics registry, in
//! `docs/content/_generated/diagnostics-*.md`. This test renders them and
//! fails when one is out of date; run it with `ASCRIBE_BLESS=1` to rewrite
//! them.
//!
//! It also checks the docs' phrases whose values come from another file, such
//! as the version, against that file.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::path::PathBuf;

use ascribe_conformance::registry::{Next, placeholders};
use ascribe_conformance::{DiagnosticsRegistry, Entry, Level, Severity, Suite};

fn repo() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

const BLESS: &str = "ASCRIBE_BLESS=1 cargo test -p ascribe-conformance --test docs";

/// The evidence an agent prompt can carry: what an entry's `evidence` may
/// name.
const EVIDENCE: &[&str] = &["allowed-values", "rule", "rule-link"];

/// The tables of `[checks]` that aren't a check's: the tools' settings. No
/// diagnostic can have their names.
const CHECK_TOOLS: &[&str] = &["vale", "links"];

/// No check ships without its next step: every diagnostic that isn't retired
/// has a kind of next step, a `fix` paragraph, an entry in the reference, and
/// the list of what its prompt carries. A `review` diagnostic can be
/// acknowledged and set in `[checks]`, so it's configurable. That every `fix`
/// diagnostic has a fix in the code is tested with the quick fixes, in
/// `crates/ascribe-lsp/tests/all/quick_fixes.rs`.
#[test]
fn every_diagnostic_has_its_next_step() {
    let registry = DiagnosticsRegistry::load(&Suite::bundled().diagnostics_path()).unwrap();
    let reference = render(&registry, &BTreeMap::new())
        .into_iter()
        .map(|(_, text)| text)
        .collect::<String>();
    let mut wrong = Vec::new();
    for e in &registry.entries {
        if CHECK_TOOLS.contains(&e.slug.as_str()) {
            wrong.push(format!("{}: `[checks.{}]` is for a tool", e.code, e.slug));
        }
        if e.retired.is_some() {
            if e.next.is_some() || e.evidence.is_some() || e.configurable {
                wrong.push(format!(
                    "{}: a retired entry has no `next`, `evidence`, or `configurable`",
                    e.code
                ));
            }
            continue;
        }
        if e.fix.as_deref().is_none_or(str::is_empty) {
            wrong.push(format!("{}: no `fix`", e.code));
        }
        if !reference.contains(&format!("#### {} `{}`", e.code, e.slug)) {
            wrong.push(format!("{}: no entry in the reference", e.code));
        }
        match e.next {
            None => wrong.push(format!("{}: no `next`", e.code)),
            Some(Next::Review) if !e.configurable => {
                wrong.push(format!(
                    "{}: a `review` diagnostic must be configurable",
                    e.code
                ));
            }
            Some(_) => {}
        }
        match &e.evidence {
            None => wrong.push(format!("{}: no `evidence` (it may be empty)", e.code)),
            Some(list) => {
                for name in list {
                    if !EVIDENCE.contains(&name.as_str()) {
                        wrong.push(format!("{}: unknown evidence `{name}`", e.code));
                    }
                }
                let values = std::iter::once(&e.message)
                    .chain(e.messages.values())
                    .any(|m| placeholders(m).is_ok_and(|p| p.contains(&"values")));
                if list.iter().any(|n| n == "allowed-values") && !values {
                    wrong.push(format!(
                        "{}: `allowed-values` needs a `{{values}}` placeholder",
                        e.code
                    ));
                }
            }
        }
    }
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}

#[test]
fn the_diagnostics_reference_is_current() {
    let registry = DiagnosticsRegistry::load(&Suite::bundled().diagnostics_path()).unwrap();
    let spec = std::fs::read_to_string(repo().join("SPEC.md")).unwrap();
    let fragments = render(&registry, &spec_anchors(&spec));
    check_fragments("diagnostics-", &fragments);
    let page =
        std::fs::read_to_string(repo().join("docs/content/reference/diagnostics.md")).unwrap();
    for (name, _) in &fragments {
        assert!(
            page.contains(&format!("@include: ../_generated/{name}\n")),
            "docs/content/reference/diagnostics.md doesn't include _generated/{name}"
        );
    }
}

/// Each phrase in `docs/ascribe.toml` whose value is also in another file
/// has the value that file has.
#[test]
fn the_docs_phrases_match_their_sources() {
    let docs: toml::Table = read("docs/ascribe.toml").parse().unwrap();
    let phrase = |key: &str| docs["phrases"][key].as_str().unwrap().to_owned();
    let mut wrong = Vec::new();
    let mut compare = |key: &str, file: &str, value: &str| {
        if phrase(key) != value {
            wrong.push(format!(
                "the phrase `{key}` is \"{}\" in docs/ascribe.toml, but \"{value}\" in {file}",
                phrase(key)
            ));
        }
    };

    // The version: the workspace's, and every package's.
    let cargo: toml::Table = read("Cargo.toml").parse().unwrap();
    let version = cargo["workspace"]["package"]["version"].as_str().unwrap();
    compare("version", "Cargo.toml", version);
    let mut packages: Vec<String> = std::fs::read_dir(repo().join("packages"))
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .filter(|name| {
            repo()
                .join("packages")
                .join(name)
                .join("package.json")
                .is_file()
        })
        .collect();
    packages.sort();
    for name in packages {
        let file = format!("packages/{name}/package.json");
        let manifest = package(&file);
        compare("version", &file, manifest["version"].as_str().unwrap());
    }

    // The least Node.js and VS Code versions, as their ranges' lower bounds.
    let cli = package("packages/cli/package.json");
    let node = cli["engines"]["node"].as_str().unwrap();
    compare("node", "packages/cli/package.json", &lower_bound(node));
    let vscode = package("packages/vscode/package.json");
    let editor = vscode["engines"]["vscode"].as_str().unwrap();
    compare(
        "vscode",
        "packages/vscode/package.json",
        &lower_bound(editor),
    );

    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}

fn read(path: &str) -> String {
    std::fs::read_to_string(repo().join(path)).unwrap()
}

fn package(path: &str) -> serde_json::Value {
    serde_json::from_str(&read(path)).unwrap()
}

/// A version range's lower bound as the docs write it: `>=24` is `24`, and
/// `^1.138.0` is `1.138`.
fn lower_bound(range: &str) -> String {
    let version = range.trim_start_matches(['^', '~', '>', '=']).trim();
    version.strip_suffix(".0").unwrap_or(version).to_owned()
}

/// Compares each fragment with its file in `docs/content/_generated/`, or,
/// with `ASCRIBE_BLESS`, writes it. A file there whose name starts with
/// `prefix` and that isn't one of `fragments` is left over from something
/// that no longer exists: it fails the test, and blessing removes it.
fn check_fragments(prefix: &str, fragments: &[(String, String)]) {
    let dir = repo().join("docs/content/_generated");
    let bless = std::env::var_os("ASCRIBE_BLESS").is_some();
    if bless {
        std::fs::create_dir_all(&dir).unwrap();
    }
    let mut stale = Vec::new();
    for entry in std::fs::read_dir(&dir).unwrap() {
        let name = entry.unwrap().file_name().to_string_lossy().into_owned();
        if name.starts_with(prefix) && !fragments.iter().any(|(n, _)| *n == name) {
            if bless {
                std::fs::remove_file(dir.join(&name)).unwrap();
            } else {
                stale.push(name);
            }
        }
    }
    for (name, text) in fragments {
        let path = dir.join(name);
        if bless {
            std::fs::write(&path, text).unwrap();
            continue;
        }
        let current = std::fs::read_to_string(&path).unwrap_or_default();
        if current != *text {
            stale.push(name.clone());
        }
    }
    assert!(
        stale.is_empty(),
        "out of date in docs/content/_generated/: {}. Run `{BLESS}`",
        stale.join(", ")
    );
}

/// The groups loader rules are listed in, as they're titled in the
/// diagnostics reference.
const RULE_GROUPS: &[(&str, &str)] = &[
    ("file", "The file"),
    ("project", "`[project]`"),
    ("sources", "`[sources]`"),
    ("content-types", "Content types, fields, and attributes"),
    (
        "names",
        "Dimensions, names, lifecycle states, notes, and features",
    ),
    ("phrases", "Phrases and the glossary"),
    ("widgets", "Widgets"),
    ("consumer", "The consumer, builds, and the editor"),
    ("checks", "`[checks]`"),
];

/// What each fragment starts with: what generates it, and how.
const HEADER: &str = "<!-- Generated from tests/conformance/diagnostics.toml by \
     tests/conformance/tests/docs.rs. Edit the registry, then run \
     `ASCRIBE_BLESS=1 cargo test -p ascribe-conformance --test docs`. -->\n";

/// The fragments, by file name: an index of every diagnostic, the
/// source-file diagnostics, the content model's, and the retired ones (empty
/// when there are none). The index links to entries in the other fragments
/// through the page that includes them all (SPEC §4.2).
fn render(
    registry: &DiagnosticsRegistry,
    anchors: &BTreeMap<String, String>,
) -> Vec<(String, String)> {
    let active: Vec<&Entry> = registry
        .entries
        .iter()
        .filter(|e| e.retired.is_none())
        .collect();

    // Source-file diagnostics, grouped by the construct their SPEC row names,
    // in the order each construct first appears.
    let mut constructs: Vec<(&str, Vec<(&Entry, &str)>)> = Vec::new();
    for entry in active
        .iter()
        .filter(|e| !is_model(e) && e.area.is_none())
    {
        let row = entry.row.as_deref().unwrap_or_default();
        let (construct, condition) = row.split_once(" | ").unwrap_or((row, ""));
        match constructs.iter_mut().find(|(c, _)| *c == construct) {
            Some((_, entries)) => entries.push((entry, condition)),
            None => constructs.push((construct, vec![(entry, condition)])),
        }
    }
    let all = index(active.iter().copied());
    let mut source_files = HEADER.to_owned();
    for (construct, entries) in constructs {
        let _ = write!(source_files, "\n### {construct}\n");
        for (entry, condition) in entries {
            write_entry(&mut source_files, entry, Some(condition), anchors);
        }
    }

    // Checks of the content's quality, by area, in `AREAS`' order.
    let mut content_checks = HEADER.to_owned();
    for entry in &active {
        if let Some(area) = &entry.area {
            assert!(
                AREAS.iter().any(|(name, _)| name == area),
                "{}: the area `{area}` isn't in AREAS",
                entry.code
            );
        }
    }
    for (area, title) in AREAS {
        let entries: Vec<&&Entry> = active
            .iter()
            .filter(|e| e.area.as_deref() == Some(*area))
            .collect();
        if entries.is_empty() {
            continue;
        }
        let _ = write!(content_checks, "\n### {title}\n");
        for entry in entries {
            write_entry(&mut content_checks, entry, None, anchors);
        }
    }

    let mut content_model = HEADER.to_owned();
    for (group, title) in RULE_GROUPS {
        let _ = write!(content_model, "\n### {title}\n");
        for entry in active
            .iter()
            .filter(|e| is_model(e) && model_group(e) == *group)
        {
            write_entry(&mut content_model, entry, None, anchors);
        }
    }

    let mut retired = HEADER.to_owned();
    let gone: Vec<&Entry> = registry
        .entries
        .iter()
        .filter(|e| e.retired.is_some())
        .collect();
    if !gone.is_empty() {
        retired.push_str(
            "\n## Retired\n\nNo longer reported. Their codes and names aren't reused.\n\n",
        );
        for entry in gone {
            let _ = writeln!(
                retired,
                "- {} `{}`: {}",
                entry.code,
                entry.slug,
                entry.retired.as_deref().unwrap_or_default()
            );
        }
    }

    vec![
        ("diagnostics-index.md".to_owned(), all),
        ("diagnostics-source-files.md".to_owned(), source_files),
        ("diagnostics-content-checks.md".to_owned(), content_checks),
        ("diagnostics-content-model.md".to_owned(), content_model),
        ("diagnostics-retired.md".to_owned(), retired),
    ]
}

/// The index: a table of the diagnostics, in code order, each linked to its
/// entry on the reference page.
fn index<'a>(entries: impl Iterator<Item = &'a Entry>) -> String {
    let mut out = format!(
        "{HEADER}\n| Code | Name | Severity | Level | Next step |\n|---|---|---|---|---|\n"
    );
    for entry in entries {
        let _ = writeln!(
            out,
            "| [{code}](../reference/diagnostics.md#{anchor}) | `{slug}` | {severity} | {level} | {next} |",
            code = entry.code,
            anchor = heading_anchor(entry),
            slug = entry.slug,
            severity = severity(entry.severity),
            level = level(entry.level),
            next = next(entry),
        );
    }
    out
}

/// The areas checks of the content's quality are listed in, as the
/// diagnostics reference titles them.
const AREAS: &[(&str, &str)] = &[("prose", "Prose, through Vale")];

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
    let configurable = if entry.configurable {
        " · configurable in `[checks]`"
    } else {
        ""
    };
    let _ = write!(
        out,
        "\n#### {} `{}`\n\n{} · {} level · next step: {}{configurable} · {section}\n\n",
        entry.code,
        entry.slug,
        severity(entry.severity),
        level(entry.level).to_lowercase(),
        next(entry),
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
        escape_braces(&rebase_links(entry.fix.as_deref().unwrap_or_default()))
    );
}

fn severity(severity: Severity) -> &'static str {
    match severity {
        Severity::Error => "Error",
        Severity::Warning => "Warning",
        Severity::Advice => "Advice",
    }
}

/// An entry's kind of next step, as the reference names it.
fn next(entry: &Entry) -> String {
    entry.next.map(|n| n.to_string()).unwrap_or_default()
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

/// Text whose relative links, written from `reference/` as the registry's
/// are, resolve from `_generated/` instead, where the fragment is.
fn rebase_links(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(at) = rest.find("](") {
        let (before, target) = rest.split_at(at + 2);
        out.push_str(before);
        let dest = target.split(')').next().unwrap_or_default();
        if !(dest.starts_with('#') || dest.starts_with('{') || dest.contains("://")) {
            out.push_str("../reference/");
        }
        rest = target;
    }
    out.push_str(rest);
    out
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
