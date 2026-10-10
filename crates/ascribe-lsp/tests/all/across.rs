//! The content checks across the project in the editor: published on load
//! and after a save, at the levels `[checks]` sets, in `ascribe.toml` too,
//! and hidden on a file edited since they ran. The rules themselves are
//! `ascribe-check`'s, and `crates/ascribe-cli/tests/all/lsp_parity.rs` holds
//! the server to `ascribe check`.

#![allow(clippy::expect_used, clippy::panic, clippy::unwrap_used)]

use lsp_types::DiagnosticSeverity;
use serde_json::{Value, json};

use crate::support::{Client, Fixture, slug, uri};

const MODEL: &str = r#"spec = "0.1"

[project]
content-root = "docs"

[phrases]
product = "Quill"
spare = "Nothing"

[dimensions.deployment]
values = ["cloud", "self-managed"]
versionless = ["cloud", "self-managed"]

[features.sso]
name = "Single sign-on"
available = "cloud"

[glossary.terms.agent]
term = "agent"
definition = "The process that ships logs."

[glossary.terms.relay]
term = "relay"
definition = "Nothing writes this."

[checks]
page-orphan = "warning"
phrase-unused = "error"
"#;

const INDEX: &str = "---\ntitle: Home\n---\n\nSee [Alpha](a.md) and [the other Alpha](c.md).\n";
const A: &str = "---\ntitle: Alpha\n---\n\nUse {product} with the agent.\n\n@include: _used.md\n";
const B: &str = "---\ntitle: Beta\n---\n\nNothing links here.\n";
const C: &str = "---\ntitle: Alpha\n---\n\nThe same title.\n";

fn project() -> Fixture {
    Fixture::new(
        MODEL,
        &[
            ("docs/index.md", INDEX),
            ("docs/a.md", A),
            ("docs/b.md", B),
            ("docs/c.md", C),
            ("docs/_used.md", "Included.\n"),
            ("docs/_spare.md", "Nobody includes this.\n"),
        ],
    )
}

/// Each diagnostic of a file: its slug and severity.
fn found(client: &Client, path: &std::path::Path) -> Vec<(String, DiagnosticSeverity)> {
    let mut out: Vec<(String, DiagnosticSeverity)> = client
        .diagnostics(path)
        .iter()
        .map(|d| (slug(d), d.severity.expect("a severity")))
        .collect();
    out.sort_by(|a, b| a.0.cmp(&b.0));
    out
}

#[test]
fn the_checks_are_published_on_load_at_their_levels() {
    let f = project();
    let mut client = Client::start(&f.root());
    client.settle();
    let info = DiagnosticSeverity::INFORMATION;
    assert_eq!(
        found(&client, &f.path("docs/b.md")),
        [("page-orphan".to_owned(), DiagnosticSeverity::WARNING)]
    );
    assert_eq!(
        found(&client, &f.path("docs/a.md")),
        [("title-duplicate".to_owned(), info)]
    );
    assert_eq!(
        found(&client, &f.path("docs/c.md")),
        [("title-duplicate".to_owned(), info)]
    );
    assert_eq!(
        found(&client, &f.path("docs/_spare.md")),
        [("fragment-unused".to_owned(), info)]
    );
    assert!(found(&client, &f.path("docs/_used.md")).is_empty());
    assert_eq!(
        found(&client, &f.path("ascribe.toml")),
        [
            ("feature-unused".to_owned(), info),
            ("glossary-term-unused".to_owned(), info),
            ("phrase-unused".to_owned(), DiagnosticSeverity::ERROR),
        ]
    );
}

#[test]
fn an_edit_hides_them_on_its_file_and_a_save_runs_them_again() {
    let f = project();
    let mut client = Client::start(&f.root());
    client.settle();
    let index = f.path("docs/index.md");
    let c = f.path("docs/c.md");

    // Linking to the orphan changes nothing until the page is saved.
    let linked = format!("{INDEX}\nAnd [Beta](b.md).\n");
    client.open(&index, 1, INDEX);
    client.replace(&index, 2, &linked);
    client.settle();
    assert_eq!(client.codes(&f.path("docs/b.md")), ["page-orphan"]);
    f.write("docs/index.md", &linked);
    client.save(&index);
    client.settle();
    assert!(client.codes(&f.path("docs/b.md")).is_empty());

    // A file edited since the checks ran doesn't show what they found in
    // it; the other page does until the save.
    let retitled = C.replace("title: Alpha", "title: Gamma");
    client.open(&c, 1, C);
    client.replace(&c, 2, &retitled);
    client.settle();
    assert!(client.codes(&c).is_empty());
    assert_eq!(client.codes(&f.path("docs/a.md")), ["title-duplicate"]);
    f.write("docs/c.md", &retitled);
    client.save(&c);
    client.settle();
    assert!(client.codes(&f.path("docs/a.md")).is_empty());
}

#[test]
fn a_model_change_runs_them_again() {
    let f = project();
    let mut client = Client::start(&f.root());
    client.settle();
    let config = f.path("ascribe.toml");
    client.open(&config, 1, MODEL);
    let without = MODEL.replace("spare = \"Nothing\"\n", "");
    client.replace(&config, 2, &without);
    client.settle();
    assert_eq!(
        client.codes(&config),
        ["feature-unused", "glossary-term-unused"]
    );
}

#[test]
fn what_is_unused_is_what_the_inventory_counts_no_use_of() {
    let f = project();
    let mut client = Client::start(&f.root());
    client.settle();
    let inventory = client
        .request(
            "ascribe/inventory",
            json!({ "textDocument": { "uri": uri(&f.path("docs/index.md")).as_str() } }),
        )
        .response_result
        .expect("an inventory");
    let unused_entries: Vec<String> = inventory["model"]
        .as_array()
        .expect("entries")
        .iter()
        .filter(|e| ["phrase", "feature", "term"].contains(&e["kind"].as_str().unwrap()))
        .filter(|e| e["uses"] == json!(0))
        .map(|e| e["key"].as_str().unwrap().to_owned())
        .collect();
    let reported: Vec<String> = client
        .diagnostics(&f.path("ascribe.toml"))
        .iter()
        .map(|d| {
            let start = d.range.start.line as usize;
            MODEL.lines().nth(start).unwrap().to_owned()
        })
        .collect();
    assert_eq!(unused_entries.len(), reported.len(), "{reported:?}");
    for key in &unused_entries {
        assert!(
            reported.iter().any(|line| line.contains(key.as_str())),
            "`{key}` is unused, and reported: {reported:?}"
        );
    }
    let unused_fragments: Vec<&str> = inventory["fragments"]
        .as_array()
        .expect("fragments")
        .iter()
        .filter(|f| f["includedBy"].as_array().is_some_and(Vec::is_empty))
        .map(|f| f["path"].as_str().unwrap())
        .collect();
    assert_eq!(unused_fragments, ["_spare.md"]);
    assert_eq!(client.codes(&f.path("docs/_spare.md")), ["fragment-unused"]);
}

#[test]
fn a_problem_prompt_about_an_orphan_has_its_evidence() {
    let f = project();
    let mut client = Client::start(&f.root());
    client.settle();
    let b = f.path("docs/b.md");
    let diagnostic = client.diagnostics(&b).remove(0);
    let answer: Value = client
        .request(
            "ascribe/agentPrompt",
            json!({ "kind": "problem", "textDocument": { "uri": uri(&b).as_str() }, "diagnostic": diagnostic }),
        )
        .response_result
        .expect("an answer");
    let prompt = answer["prompt"].as_str().expect("a prompt");
    assert!(prompt.contains("page-orphan"), "{prompt}");
    assert!(prompt.contains("Files in the same folder"), "{prompt}");
}

/// The titles of the code actions on a published diagnostic.
fn action_titles(client: &mut Client, path: &std::path::Path, slug_of: &str) -> Vec<String> {
    let diagnostic = client
        .diagnostics(path)
        .into_iter()
        .find(|d| slug(d) == slug_of)
        .unwrap_or_else(|| panic!("no {slug_of}"));
    client
        .request(
            "textDocument/codeAction",
            json!({
                "textDocument": { "uri": uri(path).as_str() },
                "range": diagnostic.range,
                "context": { "diagnostics": [diagnostic] }
            }),
        )
        .response_result
        .expect("actions")
        .as_array()
        .cloned()
        .unwrap_or_default()
        .iter()
        .filter_map(|a| a["title"].as_str().map(str::to_owned))
        .collect()
}

#[test]
fn an_orphan_and_an_unused_phrase_can_be_marked_as_intended() {
    let f = project();
    let mut client = Client::start(&f.root());
    client.settle();
    let mark = "Mark as intended, and write why".to_owned();
    assert!(action_titles(&mut client, &f.path("docs/b.md"), "page-orphan").contains(&mark));
    assert!(action_titles(&mut client, &f.path("ascribe.toml"), "phrase-unused").contains(&mark));
    // `title-duplicate` needs writing, not acknowledging.
    assert!(!action_titles(&mut client, &f.path("docs/a.md"), "title-duplicate").contains(&mark));
}
