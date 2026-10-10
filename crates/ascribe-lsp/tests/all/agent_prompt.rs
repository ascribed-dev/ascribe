//! `ascribe/agentPrompt`, over an in-memory connection.

#![allow(clippy::expect_used, clippy::panic, clippy::unwrap_used)]

use crate::support;

use crate::support::{Client, Fixture};
use serde_json::{Value, json};

const MODEL: &str = r#"spec = "0.1"

[project]
content-root = "docs"

[dimensions.pm]
values = ["npm", "pnpm"]

[builds.site]
variants = "switch"

[builds.npm]
variants = { pm = "npm" }

[editor]
build = "site"
"#;

const BROKEN: &str = "---\ntitle: Install\n---\n\nRead [the overview](overview.md).\n";
const CLEAN: &str = "---\ntitle: Install\n---\n\nAll good.\n";

fn ask(client: &mut Client, params: Value) -> Value {
    client
        .request("ascribe/agentPrompt", params)
        .response_result
        .expect("the request succeeds")
}

fn prompt(answer: &Value) -> &str {
    answer["prompt"].as_str().expect("a prompt")
}

#[test]
fn a_problems_prompt_names_its_place_and_the_editors_build() {
    let fixture = Fixture::new(MODEL, &[("docs/install.md", BROKEN)]);
    let path = fixture.path("docs/install.md");
    let mut client = Client::start(&fixture.root());
    client.open(&path, 1, BROKEN);
    client.settle();
    let diagnostic = client.diagnostics(&path).remove(0);
    let uri = support::uri(&path);
    let answer = ask(
        &mut client,
        json!({ "kind": "problem", "textDocument": { "uri": uri.as_str() }, "diagnostic": diagnostic }),
    );
    let text = prompt(&answer);
    assert!(
        text.starts_with("Fix this problem in `docs/install.md`.\n\nWhere: docs/install.md:5\nBuild: `site`, the editor's. `ascribe check` checks every build.\n"),
        "{text}"
    );
    assert!(
        text.contains("Message: `overview.md` doesn't exist\n"),
        "{text}"
    );
    assert!(!text.contains("unsaved"), "{text}");
}

#[test]
fn a_prompt_about_unsaved_text_says_to_save_it() {
    let fixture = Fixture::new(MODEL, &[("docs/install.md", CLEAN)]);
    let path = fixture.path("docs/install.md");
    let uri = support::uri(&path);
    let mut client = Client::start(&fixture.root());
    client.open(&path, 1, CLEAN);
    client.replace(&path, 2, BROKEN);
    client.settle();
    let answer = ask(
        &mut client,
        json!({ "kind": "file", "textDocument": { "uri": uri.as_str() }, "unsaved": [uri.as_str()] }),
    );
    let text = prompt(&answer);
    assert!(
        text.contains(
            "Where: docs/install.md\nThe file has unsaved changes; save it before you start.\n"
        ),
        "{text}"
    );
    assert!(
        text.contains("- 5: [ASC036] `overview.md` doesn't exist\n"),
        "{text}"
    );
}

#[test]
fn no_problem_no_prompt() {
    let fixture = Fixture::new(MODEL, &[("docs/install.md", CLEAN)]);
    let path = fixture.path("docs/install.md");
    let uri = support::uri(&path);
    let mut client = Client::start(&fixture.root());
    client.open(&path, 1, CLEAN);
    client.settle();
    for kind in ["file", "project"] {
        let answer = ask(
            &mut client,
            json!({ "kind": kind, "textDocument": { "uri": uri.as_str() } }),
        );
        assert_eq!(answer, Value::Null, "{kind}");
    }
}

#[test]
fn a_problem_no_longer_reported_has_no_prompt() {
    let fixture = Fixture::new(MODEL, &[("docs/install.md", BROKEN)]);
    let path = fixture.path("docs/install.md");
    let uri = support::uri(&path);
    let mut client = Client::start(&fixture.root());
    client.open(&path, 1, BROKEN);
    client.settle();
    let diagnostic = client.diagnostics(&path).remove(0);
    client.replace(&path, 2, CLEAN);
    client.settle();
    let answer = ask(
        &mut client,
        json!({ "kind": "problem", "textDocument": { "uri": uri.as_str() }, "diagnostic": diagnostic }),
    );
    assert_eq!(answer, Value::Null);
}

#[test]
fn the_projects_prompt_needs_no_file() {
    let fixture = Fixture::new(
        MODEL,
        &[("docs/install.md", BROKEN), ("docs/upgrade.md", BROKEN)],
    );
    let mut client = Client::start(&fixture.root());
    client.settle();
    let answer = ask(&mut client, json!({ "kind": "project" }));
    let text = prompt(&answer);
    assert!(
        text.starts_with("Fix the 2 problems `ascribe check` reports in this project.\n"),
        "{text}"
    );
    assert!(
        text.contains("- docs/install.md: 1 error\n- docs/upgrade.md: 1 error\n"),
        "{text}"
    );
}
