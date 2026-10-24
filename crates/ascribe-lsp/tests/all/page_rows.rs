//! The rows of This page's About and Health, and what a page includes
//! (`ascribe/page`), over an in-memory connection: each row for a page that
//! has it, each left out for a page that doesn't, a page of Ascribe's own
//! docs, and the tested examples of `examples/tested`. That Size and
//! Listing agree with `page-size` and `pages.json` for a built project is
//! `crates/ascribe-cli/tests/all/page_parity.rs`'s.

#![allow(clippy::expect_used, clippy::panic, clippy::unwrap_used)]

use std::path::Path;
use std::sync::Arc;

use ascribe_lsp::Options;
use serde_json::{Value, json};

use crate::support::{self, Client, Fixture, Setup};

const MODEL: &str = r#"spec = "0.1"

[project]
content-root = "docs"

[types.guide]
template = "templates/guide.md"

[types.guide.frontmatter]
title = "string"
description = { type = "string?", role = "description" }
review-by = { type = "date?", role = "review-date" }

[dimensions.deployment]
label = "Deployment"
values = ["cloud", "self-managed"]
versionless = ["cloud"]

[consumer]
site = "https://docs.example.com"
agents = true

[listing.folders]
"internal/" = "unlisted"

[builds.site]
variants = "switch"
availability = "badge"

[builds.cloud]
variants = { deployment = "cloud" }
availability = { filter = "cloud" }

[checks]
page-orphan = "off"
fragment-unused = "off"

[checks.page-size]
limit = 100
"#;

const TEMPLATE: &str = "---\ntype: guide\ntitle: \"Title\"\ndescription: \"{One sentence.}\"\n---\n\nExplain how to \\{the task}.\n";

const ROTATE: &str = "---\ntype: guide\ntitle: Rotate keys\ndescription: \"{One sentence.}\"\nreview-by: 2026-10-03\navailable: self-managed\npermalink: rotate-keys\n---\n\nExplain how to \\{the task}.\n\n## Rotate now\n@id {permalink=true}: rotate-now\n\n@include: ../_steps.md\n\nDone.\n";

const PLAIN: &str = "---\ntype: guide\ntitle: Plain\n---\n\nNothing more to say.\n";

const STEPS: &str = "1. Do it.\n";

fn project() -> Fixture {
    Fixture::new(
        MODEL,
        &[
            ("templates/guide.md", TEMPLATE),
            (
                "docs/index.md",
                "---\ntype: guide\ntitle: Home\ndescription: The start.\n---\n\nSee [rotating](internal/rotate.md).\n",
            ),
            ("docs/internal/rotate.md", ROTATE),
            ("docs/plain.md", PLAIN),
            ("docs/_steps.md", STEPS),
        ],
    )
}

/// A client whose day is 17 October 2026.
fn started(root: &Path) -> Client {
    let mut client = Client::start_with(
        root,
        Setup {
            options: Options {
                today: Some(Arc::new(|| ascribe_core::Date::new(2026, 10, 17))),
                ..Options::default()
            },
            ..Setup::default()
        },
    );
    client.settle();
    client
}

fn page(client: &mut Client, path: &Path) -> Value {
    client
        .request(
            "ascribe/page",
            json!({
                "textDocument": { "uri": support::uri(path).as_str() },
                "position": { "line": 0, "character": 0 },
            }),
        )
        .response_result
        .expect("the request succeeds")
}

fn line(n: u32, from: u32, to: u32) -> Value {
    json!({ "start": { "line": n, "character": from }, "end": { "line": n, "character": to } })
}

#[test]
fn about_says_whether_the_page_is_described_due_published_listed_and_unfinished() {
    let f = project();
    let mut client = started(&f.root());
    let about = page(&mut client, &f.path("docs/internal/rotate.md"))["about"].clone();
    assert_eq!(
        about["description"],
        json!({ "field": "description", "length": 15, "limit": 300, "reported": false, "range": line(3, 0, 30) })
    );
    assert_eq!(
        about["review"],
        json!({ "field": "review-by", "date": "2026-10-03", "days": -14, "overdue": true, "reported": true, "range": line(4, 0, 21) })
    );
    assert_eq!(about["availability"], "self-managed");
    assert_eq!(
        about["builds"],
        json!([
            { "name": "site", "leftOut": null },
            { "name": "cloud", "leftOut": "availability" },
        ])
    );
    let model = support::uri(&f.path("ascribe.toml"));
    let rule = MODEL
        .lines()
        .position(|l| l.starts_with("\"internal/\""))
        .unwrap() as u32;
    assert_eq!(
        about["listing"],
        json!({
            "build": "site",
            "listing": "unlisted",
            "from": "folder",
            "rule": "\"internal/\" = \"unlisted\"",
            "uri": model.as_str(),
            "range": line(rule, 0, 24),
        })
    );
    assert_eq!(
        about["permalinks"],
        json!([
            // The page's own leads to its opening, whose summary is the
            // page's description; the heading's section has none.
            { "name": "rotate-keys", "heading": null, "summary": "{One sentence.}", "range": line(6, 0, 22) },
            { "name": "rotate-now", "heading": "Rotate now", "summary": null, "range": line(12, 0, 32) },
        ])
    );
    // The template's description, and its placeholder in the body.
    assert_eq!(about["template"], json!({ "type": "guide", "left": 2 }));
}

#[test]
fn a_permalinks_summary_is_the_one_help_json_writes() {
    // The heading's summary comes from the fragment the section includes,
    // and is written as the plain output writes it, its emphasis kept.
    let f = project();
    f.write(
        "docs/keys.md",
        "---\ntype: guide\ntitle: Keys\ndescription: Keys.\n---\n\n## Rotate now\n\
         @id {permalink=true}: rotate-now\n\n@include: _summary.md\n\nDone.\n",
    );
    f.write(
        "docs/_summary.md",
        "@summary\nReplace a key *before* it expires.\n",
    );
    let mut client = started(&f.root());
    let about = page(&mut client, &f.path("docs/keys.md"))["about"].clone();
    assert_eq!(
        about["permalinks"][0]["summary"], "Replace a key *before* it expires.",
        "{about}"
    );
}

#[test]
fn review_warns_where_review_overdue_reports_the_page() {
    let f = project();
    let mut client = started(&f.root());
    let path = f.path("docs/internal/rotate.md");
    assert_eq!(
        page(&mut client, &path)["about"]["review"]["reported"],
        true
    );
    assert!(
        client.codes(&path).contains(&"review-overdue".to_owned()),
        "{:?}",
        client.codes(&path)
    );
}

#[test]
fn a_row_with_nothing_to_say_is_left_out() {
    let f = project();
    let mut client = started(&f.root());
    let about = page(&mut client, &f.path("docs/plain.md"))["about"].clone();
    // A description is shown missing: `page-description-missing` reports it.
    assert_eq!(about["description"]["length"], Value::Null);
    assert_eq!(about["review"], Value::Null);
    assert_eq!(about["availability"], Value::Null);
    assert_eq!(about["permalinks"], json!([]));
    // Its type has a template, and nothing of it is left on the page.
    assert_eq!(about["template"], Value::Null);
    assert_eq!(
        about["listing"],
        json!({ "build": "site", "listing": "listed", "from": "default", "rule": null, "uri": null, "range": null })
    );
}

#[test]
fn health_has_the_size_of_the_largest_build_and_includes_lists_the_fragments() {
    let f = project();
    let mut client = started(&f.root());
    let answer = page(&mut client, &f.path("docs/internal/rotate.md"));
    let size = &answer["health"]["size"];
    // The cloud build leaves the page out, so its size is the site's.
    assert_eq!(size["build"], "site");
    assert_eq!(size["limit"], 100);
    assert!(size["characters"].as_u64().unwrap() > 0, "{size}");
    assert_eq!(answer["health"]["examples"], json!([]));
    assert_eq!(
        answer["includes"],
        json!([{
            "kind": "include",
            "written": "../_steps.md",
            "path": "_steps.md",
            "uri": support::uri(&f.path("docs/_steps.md")).as_str(),
            "range": line(14, 0, 22),
        }])
    );
}

#[test]
fn a_fragment_has_no_builds_listing_or_size() {
    let f = project();
    let mut client = started(&f.root());
    let answer = page(&mut client, &f.path("docs/_steps.md"));
    assert_eq!(answer["about"]["builds"], json!([]));
    assert_eq!(answer["about"]["listing"], Value::Null);
    assert_eq!(answer["about"]["description"], Value::Null);
    assert_eq!(answer["health"]["size"], Value::Null);
}

#[test]
fn the_inventory_lists_the_builds_that_publish_each_page() {
    let f = project();
    let mut client = started(&f.root());
    let inventory = client
        .request(
            "ascribe/inventory",
            json!({ "textDocument": { "uri": support::uri(&f.path("ascribe.toml")).as_str() } }),
        )
        .response_result
        .expect("the request succeeds");
    let builds = |path: &str| {
        inventory["pages"]
            .as_array()
            .unwrap()
            .iter()
            .find(|p| p["path"] == path)
            .map(|p| p["builds"].clone())
    };
    assert_eq!(builds("internal/rotate.md"), Some(json!(["site"])));
    assert_eq!(builds("plain.md"), Some(json!(["site", "cloud"])));
    // The same set This page's Builds row says publishes it.
    let about = page(&mut client, &f.path("docs/internal/rotate.md"))["about"].clone();
    let published: Vec<Value> = about["builds"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|b| b["leftOut"].is_null())
        .map(|b| b["name"].clone())
        .collect();
    assert_eq!(Some(Value::Array(published)), builds("internal/rotate.md"));
}

#[test]
fn an_edit_to_the_page_is_in_the_rows_before_its_saved() {
    let f = project();
    let mut client = started(&f.root());
    let path = f.path("docs/plain.md");
    assert_eq!(page(&mut client, &path)["about"]["review"], Value::Null);
    client.open(&path, 1, PLAIN);
    client.replace(
        &path,
        2,
        &PLAIN.replace("title: Plain\n", "title: Plain\nreview-by: 2026-10-29\n"),
    );
    client.settle();
    let review = &page(&mut client, &path)["about"]["review"];
    assert_eq!(review["days"], 12);
    assert_eq!(review["overdue"], false);
}

fn copy_dir(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).expect("mkdir");
    for entry in std::fs::read_dir(from).expect("read_dir").flatten() {
        let target = to.join(entry.file_name());
        if entry.path().is_dir() {
            copy_dir(&entry.path(), &target);
        } else {
            std::fs::copy(entry.path(), target).expect("copy");
        }
    }
}

#[test]
fn health_lists_each_tested_example_and_includes_its_file() {
    let source = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/tested");
    let f = Fixture::new("", &[]);
    copy_dir(&source, &f.root());
    // A heading over the example, which its row names.
    let index = f.path("docs/index.md");
    let text = std::fs::read_to_string(&index).unwrap();
    std::fs::write(
        &index,
        text.replace("The example takes", "## Greet\n\nThe example takes"),
    )
    .unwrap();
    let mut client = started(&f.root());
    let answer = page(&mut client, &index);
    let examples = answer["health"]["examples"].as_array().unwrap();
    let greet = examples
        .iter()
        .find(|e| {
            e["address"]
                .as_str()
                .unwrap()
                .starts_with("code:examples/greet.ts")
        })
        .unwrap_or_else(|| panic!("greet.ts: {examples:?}"));
    assert_eq!(greet["state"], "tested");
    assert_eq!(greet["heading"], "Greet");
    assert!(
        greet["tested"].as_str().unwrap().contains("Node"),
        "{greet}"
    );
    let file = support::uri(&f.path("code/examples/greet.ts"));
    let shown = answer["includes"]
        .as_array()
        .unwrap()
        .iter()
        .find(|i| i["kind"] == "snippet" && i["uri"] == file.as_str())
        .unwrap_or_else(|| panic!("greet.ts: {}", answer["includes"]));
    assert_eq!(shown["path"], "code/examples/greet.ts");
    // Its observations hashing it as something else make it stale, and
    // the rows follow the change on disk.
    let mut changed = Vec::new();
    for name in ["default", "node-22"] {
        let observation = f.path(&format!("code/examples.observations/{name}.toml"));
        let text = std::fs::read_to_string(&observation).unwrap();
        let edited: String = text
            .lines()
            .map(|line| match line.starts_with("\"examples/greet.ts\"") {
                true => format!("\"examples/greet.ts\" = \"sha256-lf:{}\"\n", "0".repeat(64)),
                false => format!("{line}\n"),
            })
            .collect();
        std::fs::write(&observation, edited).unwrap();
        changed.push(observation);
    }
    let events: Vec<_> = changed
        .iter()
        .map(|p| (p.as_path(), lsp_types::FileChangeType::CHANGED))
        .collect();
    client.watched(&events);
    client.settle();
    let again = page(&mut client, &f.path("docs/index.md"));
    let greet = again["health"]["examples"]
        .as_array()
        .unwrap()
        .iter()
        .find(|e| {
            e["address"]
                .as_str()
                .unwrap()
                .starts_with("code:examples/greet.ts")
        })
        .cloned()
        .unwrap();
    assert_eq!(greet["state"], "stale");
}

#[test]
fn a_page_of_ascribes_docs_has_its_rows() {
    let docs = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs");
    let docs = support::real_path(&docs);
    let mut client = started(&docs);
    let path = docs.join("content/editor/index.md");
    let about = page(&mut client, &path)["about"].clone();
    let description = &about["description"];
    assert_eq!(description["field"], "description");
    assert_eq!(description["limit"], 300);
    assert!(description["length"].as_u64().unwrap() > 0, "{description}");
    assert_eq!(
        about["builds"],
        json!([{ "name": "site", "leftOut": null }])
    );
    assert_eq!(about["listing"]["listing"], "listed");
    assert_eq!(about["listing"]["from"], "default");
    let names: Vec<&str> = about["permalinks"]
        .as_array()
        .unwrap()
        .iter()
        .map(|p| p["name"].as_str().unwrap())
        .collect();
    assert!(names.contains(&"editor-guide"), "{names:?}");
    let answer = page(&mut client, &path);
    let size = &answer["health"]["size"];
    assert_eq!(size["build"], "site");
    assert_eq!(size["limit"], 50_000);
    assert!(size["characters"].as_u64().unwrap() > 1_000, "{size}");
}
