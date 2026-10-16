//! What's known about a page and the heading at the cursor
//! (`ascribe/page`), over an in-memory connection.

#![allow(clippy::expect_used, clippy::panic, clippy::unwrap_used)]

use crate::support;

use crate::support::{Client, Fixture};
use serde_json::{Value, json};

const MODEL: &str = r#"spec = "0.1"

[project]
content-root = "docs"

[phrases]
product = "Quill"

[types.page.frontmatter]
title = "string"

[builds.site]
"#;

const INDEX: &str =
    "---\ntitle: Home\n---\n\n# Home\n\nSee [setup](guide.md#setup) and [the guide](guide.md).\n";
const GUIDE: &str = "---\ntitle: Guide\ntype: page\n---\n\nIntro.\n\n@include: _setup.md\n\n## Use {product}\n\nJump to [install](#install).\n";
const SETUP: &str = "## Setup\n\n### Install\n\nRun it.\n";
const OTHER: &str = "---\ntitle: Other\n---\n\n@include: _setup.md#install\n";

fn project(model: &str) -> Fixture {
    Fixture::new(
        model,
        &[
            ("docs/index.md", INDEX),
            ("docs/guide.md", GUIDE),
            ("docs/_setup.md", SETUP),
            ("docs/other.md", OTHER),
            ("docs/notes.txt", "Not a page.\n"),
        ],
    )
}

fn started(f: &Fixture) -> Client {
    let mut client = Client::start(&f.root());
    client.settle();
    client
}

/// The position of the start of the first `needle` in `text`.
fn at(text: &str, needle: &str) -> Value {
    let i = text.find(needle).unwrap_or_else(|| panic!("no {needle:?}"));
    let before = &text[..i];
    let line = before.matches('\n').count();
    let start = before.rfind('\n').map_or(0, |i| i + 1);
    json!({ "line": line, "character": text[start..i].encode_utf16().count() })
}

fn page(client: &mut Client, f: &Fixture, file: &str, position: Value) -> Value {
    client
        .request(
            "ascribe/page",
            json!({
                "textDocument": { "uri": support::uri(&f.path(&format!("docs/{file}"))).as_str() },
                "position": position,
            }),
        )
        .response_result
        .expect("the request succeeds")
}

/// A list of places as each one's content path and line text.
fn places(list: &Value) -> Vec<(String, String)> {
    list.as_array()
        .expect("a list")
        .iter()
        .map(|p| {
            (
                p["path"].as_str().unwrap().to_owned(),
                p["text"].as_str().unwrap().to_owned(),
            )
        })
        .collect()
}

fn pairs(list: &[(&str, &str)]) -> Vec<(String, String)> {
    list.iter()
        .map(|(a, b)| ((*a).to_owned(), (*b).to_owned()))
        .collect()
}

#[test]
fn a_page_outside_any_heading_says_what_it_is_and_what_links_to_it() {
    let f = project(MODEL);
    let mut client = started(&f);
    let answer = page(&mut client, &f, "guide.md", at(GUIDE, "Intro"));
    assert_eq!(
        answer["about"],
        json!({
            "path": "guide.md",
            "fragment": false,
            "title": "Guide",
            "type": "page",
            "address": "/guide/",
            "markdownAddress": null,
        })
    );
    assert_eq!(answer["heading"], Value::Null);
    assert_eq!(
        places(&answer["linkedFrom"]),
        pairs(&[
            (
                "index.md",
                "See [setup](guide.md#setup) and [the guide](guide.md)."
            ),
            (
                "index.md",
                "See [setup](guide.md#setup) and [the guide](guide.md)."
            ),
        ])
    );
    assert_eq!(places(&answer["includedBy"]), pairs(&[]));
    let first = &answer["linkedFrom"][0];
    assert_eq!(
        first["uri"].as_str(),
        Some(support::uri(&f.path("docs/index.md")).as_str())
    );
    assert_eq!(
        first["range"],
        json!({ "start": { "line": 6, "character": 4 }, "end": { "line": 6, "character": 27 } })
    );
}

#[test]
fn the_cursor_anywhere_in_a_section_narrows_to_its_heading() {
    let f = project(MODEL);
    let mut client = started(&f);
    let answer = page(&mut client, &f, "guide.md", at(GUIDE, "Jump"));
    // The heading's text has the phrase's value.
    assert_eq!(answer["heading"]["id"], "use-quill");
    assert_eq!(answer["heading"]["text"], "Use Quill");
    assert_eq!(
        answer["heading"]["range"],
        json!({ "start": { "line": 9, "character": 0 }, "end": { "line": 9, "character": 16 } })
    );
    assert_eq!(places(&answer["linkedFrom"]), pairs(&[]));
    // The rest doesn't depend on the cursor.
    assert_eq!(answer["about"]["title"], "Guide");
}

#[test]
fn a_fragment_has_no_address_and_its_headings_are_linked_through_its_includers() {
    let f = project(MODEL);
    let mut client = started(&f);
    let answer = page(&mut client, &f, "_setup.md", at(SETUP, "Run it"));
    assert_eq!(
        answer["about"],
        json!({
            "path": "_setup.md",
            "fragment": true,
            "title": null,
            "type": null,
            "address": null,
            "markdownAddress": null,
        })
    );
    // The innermost section: "Install", inside "Setup".
    assert_eq!(answer["heading"]["id"], "install");
    assert_eq!(
        places(&answer["linkedFrom"]),
        pairs(&[("guide.md", "Jump to [install](#install).")])
    );
    assert_eq!(
        places(&answer["includedBy"]),
        pairs(&[("other.md", "@include: _setup.md#install")])
    );
    // At the start of the file, before any heading: the whole fragment.
    let whole = page(
        &mut client,
        &f,
        "_setup.md",
        json!({ "line": 0, "character": 0 }),
    );
    assert_eq!(whole["heading"]["id"], "setup");
    let before = page(
        &mut client,
        &f,
        "other.md",
        json!({ "line": 0, "character": 0 }),
    );
    assert_eq!(before["heading"], Value::Null);
}

#[test]
fn a_file_that_isnt_a_page_gets_an_empty_answer() {
    let f = project(MODEL);
    let mut client = started(&f);
    let empty = json!({ "about": null, "heading": null, "linkedFrom": [], "includedBy": [] });
    assert_eq!(
        page(
            &mut client,
            &f,
            "notes.txt",
            json!({ "line": 0, "character": 0 })
        ),
        empty
    );
    let model = client
        .request(
            "ascribe/page",
            json!({
                "textDocument": { "uri": support::uri(&f.path("ascribe.toml")).as_str() },
                "position": { "line": 0, "character": 0 },
            }),
        )
        .response_result
        .expect("the request succeeds");
    assert_eq!(model, empty);
}

#[test]
fn an_unsaved_edit_is_counted_before_its_saved() {
    let f = project(MODEL);
    let mut client = started(&f);
    let path = f.path("docs/index.md");
    client.open(&path, 1, INDEX);
    client.replace(
        &path,
        2,
        &format!("{INDEX}\nAnd [again](guide.md#use-quill).\n"),
    );
    client.settle();
    let answer = page(&mut client, &f, "guide.md", at(GUIDE, "Jump"));
    assert_eq!(
        places(&answer["linkedFrom"]),
        pairs(&[("index.md", "And [again](guide.md#use-quill).")])
    );
}

#[test]
fn a_project_that_publishes_markdown_versions_gives_their_address() {
    let f = project(&MODEL.replace(
        "[builds.site]",
        "[consumer]\nsite = \"https://docs.example.com\"\nagents = true\n\n[builds.site]",
    ));
    let mut client = started(&f);
    let answer = page(
        &mut client,
        &f,
        "guide.md",
        json!({ "line": 0, "character": 0 }),
    );
    assert_eq!(answer["about"]["address"], "/guide/");
    assert_eq!(answer["about"]["markdownAddress"], "/guide.md");
}
