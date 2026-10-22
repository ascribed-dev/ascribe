#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! The registry of `ascribe.toml`'s keys (`settings.toml`) agrees with the
//! loader: every key the registry has is one the loader reads, every key the
//! loader takes is one the registry has, and the default the registry gives
//! is the one the loader uses.
//!
//! Each table the registry names is written into a small model that has it,
//! with what the table needs and nothing else. `rules.rs` holds what each key
//! may be.

use ascribe_core::{FileId, Issue, Span, diagnostics};
use ascribe_model::load_str;
use ascribe_model::settings::{FIELD_TABLE, REPORTED_APART, SettingValue, settings};

/// A model with `table` in it, for `key`: the text before the table's header,
/// the header, and the keys the table needs, as `key = value` lines.
struct Host {
    before: &'static str,
    header: &'static str,
    needs: &'static [(&'static str, &'static str)],
}

const SELF_MANAGED: &str = "[dimensions.deployment]\nvalues = [\"self-managed\"]\n";
const LIVE: &str = "[dimensions.deployment]\nvalues = [\"self-managed\"]\n\
                    [releases.self-managed]\nlive = [\"3.4\"]\n";

fn host(table: &str, key: &str) -> Host {
    let (before, header, needs): (&str, &str, &[(&str, &str)]) = match table {
        "" => ("", "", &[]),
        // A type needs no keys: its `title` is implicit.
        "types.<name>" => ("", "[types.page]", &[]),
        // `values` is for a bare `enum`, and an unknown key on any other type.
        FIELD_TABLE if key == "values" => {
            ("", "[types.page.frontmatter.kind]", &[("type", "\"enum\"")])
        }
        FIELD_TABLE => (
            "",
            "[types.page.frontmatter.title]",
            &[("type", "\"string\"")],
        ),
        "dimensions.<name>" => ("", "[dimensions.deployment]", &[("values", "[\"cloud\"]")]),
        "releases" => (SELF_MANAGED, "[releases]", &[]),
        "releases.<target>" => (SELF_MANAGED, "[releases.self-managed]", &[]),
        "releases.<target>.archived.<line>" => {
            (LIVE, "[releases.self-managed.archived.\"3.2\"]", &[])
        }
        "lifecycle.<state>" => ("", "[lifecycle.beta]", &[]),
        "features.<key>" => (
            "",
            "[features.sso]",
            &[("name", "\"SSO\""), ("available", "\"ga\"")],
        ),
        "notes.<type>" => ("", "[notes.tip]", &[]),
        "phrases.<key>" => (LIVE, "[phrases.release]", &[("line", "\"self-managed\"")]),
        "widgets.<name>" => ("", "[widgets.x-y]", &[("forms", "[\"container\"]")]),
        "builds.<name>" => ("", "[builds.site]", &[]),
        "sources.<name>" => ("", "[sources.code]", &[("path", "\".\"")]),
        "checks.<check>" => ("", "[checks.page-size]", &[]),
        "from.<path>" => (
            "",
            "[from.\"../docs/ascribe.toml\"]",
            &[("take", "[\"phrases\"]")],
        ),
        "intended" => (
            "",
            "[[intended]]",
            &[("check", "\"phrase-unused\""), ("reason", "\"Kept\"")],
        ),
        "checks.vale" => ("", "[checks.vale]", &[("preset", "\"quiet\"")]),
        "permalinks.retired.<name>" => (
            "",
            "[permalinks.retired.old]",
            &[("reason", "\"Gone for good\"")],
        ),
        // A site, since `agents` defaults to `true` only with one.
        "consumer" => ("", "[consumer]", &[("site", "\"https://example.com\"")]),
        "project" | "fragments" | "versions" | "glossary" | "images" | "editor" | "checks"
        | "checks.links" | "permalinks" | "crawlers" | "listing" => {
            let header = Box::leak(format!("[{table}]").into_boxed_str());
            ("", header, &[])
        }
        other => panic!("no host for the registry's table `{other}`: add one here"),
    };
    Host {
        before,
        header,
        needs,
    }
}

/// The model `host` makes, with `line` added to its table, when there is
/// one, in place of the key of the same name the table needs. Returns the
/// text and the span of `line` in it.
fn with_line(table: &str, line: Option<(&str, &str)>) -> (String, Span) {
    let host = host(table, line.map_or("", |(k, _)| k));
    let mut text = String::new();
    if table.is_empty() {
        if line.is_none_or(|(k, _)| k != "spec") {
            text.push_str("spec = \"0.1\"\n");
        }
    } else {
        text.push_str("spec = \"0.1\"\n");
    }
    text.push_str(host.before);
    if !host.header.is_empty() {
        text.push_str(host.header);
        text.push('\n');
    }
    for (key, value) in host.needs {
        if line.is_none_or(|(k, _)| k != *key) {
            text.push_str(&format!("{key} = {value}\n"));
        }
    }
    let start = text.len();
    if let Some((key, value)) = line {
        text.push_str(&format!("{key} = {value}"));
    }
    let span = Span::new(start, text.len());
    text.push('\n');
    (text, span)
}

fn issues(text: &str) -> Vec<Issue> {
    match load_str(text, FileId::new(0)) {
        Ok(m) => m.warnings,
        Err(issues) => issues,
    }
}

fn on(issue: &Issue, span: Span) -> bool {
    let at = issue.location.span;
    at.start() >= span.start() && at.start() <= span.end()
}

fn tables() -> Vec<&'static str> {
    let mut out: Vec<&str> = Vec::new();
    for s in settings() {
        if !out.contains(&s.table.as_str()) {
            out.push(&s.table);
        }
    }
    out
}

/// A value no key takes: an array holding a table. Each key reports it,
/// as the wrong type or as what its rules make of it, which shows the loader
/// read the key.
const WRONG: &str = "[{}]";

#[test]
fn every_key_the_registry_has_is_one_the_loader_reads() {
    let mut unread = Vec::new();
    for s in settings() {
        let (text, span) = with_line(&s.table, Some((&s.key, WRONG)));
        let found = issues(&text);
        let read = found
            .iter()
            .any(|i| on(i, span) && i.slug != diagnostics::MODEL_UNKNOWN_KEY);
        if !read {
            let got: Vec<String> = found.iter().map(|i| i.slug.to_string()).collect();
            unread.push(format!("{} {}: {got:?}\n{text}", s.table, s.key));
        }
    }
    assert!(unread.is_empty(), "not read:\n{}", unread.join("\n"));
}

#[test]
fn every_key_the_loader_takes_is_one_the_registry_has() {
    let mut taken = Vec::new();
    for table in tables() {
        // A key of `[releases]` that isn't `planned-label` is a target.
        if table == "releases" {
            continue;
        }
        let apart = REPORTED_APART
            .iter()
            .filter(|(t, _)| *t == table)
            .map(|(_, k)| *k);
        let (text, span) = with_line(table, Some(("not-a-key", "1")));
        if !issues(&text)
            .iter()
            .any(|i| on(i, span) && i.slug == diagnostics::MODEL_UNKNOWN_KEY)
        {
            taken.push(format!("{table} not-a-key\n{text}"));
        }
        // A key reported apart is reported, with its own message.
        for key in apart {
            let (text, span) = with_line(table, Some((key, "1")));
            if !issues(&text).iter().any(|i| on(i, span)) {
                taken.push(format!("{table} {key}\n{text}"));
            }
        }
    }
    // An attribute takes fewer keys than a field.
    for s in settings().iter().filter(|s| s.fields_only) {
        let text = format!(
            "spec = \"0.1\"\n[images.attributes.a]\ntype = \"string\"\n{} = 1\n",
            s.key
        );
        let at = text.rfind(&s.key).unwrap();
        let span = Span::new(at, text.len());
        if !issues(&text)
            .iter()
            .any(|i| on(i, span) && i.slug == diagnostics::MODEL_UNKNOWN_KEY)
        {
            taken.push(format!("an attribute's {}\n{text}", s.key));
        }
    }
    assert!(taken.is_empty(), "taken:\n{}", taken.join("\n"));
}

#[test]
fn the_default_the_registry_gives_is_the_one_the_loader_uses() {
    let mut differ = Vec::new();
    for s in settings() {
        let Some(default) = &s.default else { continue };
        // A listing written down is a rule, which a page's record names, even
        // when it says what an absent one does.
        if s.table == "listing" && matches!(s.key.as_str(), "default" | "older-lines") {
            continue;
        }
        // What every source leaves out is applied whatever `ignore` says,
        // not in its place: `always_ignored_folders_are_the_registry_s`.
        if s.table == "sources.<name>" && s.key == "ignore" {
            continue;
        }
        let value = match default {
            SettingValue::Switch(b) => b.to_string(),
            SettingValue::Number(n) => n.to_string(),
            SettingValue::Text(t) => format!("{t:?}"),
            SettingValue::List(items) => format!("{items:?}"),
            SettingValue::EmptyTable => "{}".to_owned(),
        };
        let (absent, _) = with_line(&s.table, None);
        let (written, _) = with_line(&s.table, Some((&s.key, &value)));
        let absent = load_str(&absent, FileId::new(0));
        let written = load_str(&written, FileId::new(0));
        let (Ok(absent), Ok(written)) = (absent, written) else {
            differ.push(format!("{} {}: doesn't load", s.table, s.key));
            continue;
        };
        if format!("{absent:?}") != format!("{written:?}") {
            differ.push(format!("{} {} = {value}", s.table, s.key));
        }
    }
    assert!(differ.is_empty(), "differ:\n{}", differ.join("\n"));
}

#[test]
fn always_ignored_folders_are_the_registry_s() {
    for own in ["", "ignore = [\"private/**\"]\n"] {
        let text = format!("spec = \"0.1\"\n[sources.code]\npath = \".\"\n{own}");
        let model = load_str(&text, FileId::new(0)).unwrap();
        let source = &model.sources[0];
        for left_out in [
            "node_modules/pkg/index.js",
            "lib/node_modules/dep/a.py",
            ".git/config",
            "vendor/sub/.git/HEAD",
        ] {
            assert!(!source.reads(left_out), "{own}{left_out}");
            assert!(!source.may_use(left_out), "{own}{left_out}");
        }
        assert!(source.reads("src/app.py"), "{own}");
        assert!(source.reads("src/node_modules.py"), "{own}");
    }
}
