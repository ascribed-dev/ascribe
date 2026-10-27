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
spare = "Spare parts"

[dimensions.deployment]
values = ["cloud", "self-managed"]
versionless = ["cloud", "self-managed"]

[features.sso]
name = "Single sign-on"
available = "cloud"

[glossary]
pages = ["glossary.md"]

[checks]
page-orphan = "warning"
phrase-unused = "error"
"#;

const INDEX: &str = "---\ntitle: Home\n---\n\nSee [Alpha](a.md) and [the other Alpha](c.md), and [the glossary](glossary.md).\n";
const GLOSSARY: &str = "---\ntitle: Glossary\n---\n\n## agent\n@term\n\nThe process that ships logs.\n\n## relay\n@term\n\nNothing writes this.\n";
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
            ("docs/glossary.md", GLOSSARY),
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
            ("phrase-unused".to_owned(), DiagnosticSeverity::ERROR),
        ]
    );
    assert_eq!(
        found(&client, &f.path("docs/glossary.md")),
        [("glossary-term-unused".to_owned(), info)]
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
    let without = MODEL.replace("spare = \"Spare parts\"\n", "");
    client.replace(&config, 2, &without);
    client.settle();
    assert_eq!(client.codes(&config), ["feature-unused"]);
}

#[test]
fn a_file_the_project_doesnt_read_doesnt_run_them_again_and_a_page_on_disk_does() {
    let f = project();
    let mut client = Client::start(&f.root());
    client.settle();
    let (a, b, c) = (
        f.path("docs/a.md"),
        f.path("docs/b.md"),
        f.path("docs/c.md"),
    );
    // Retitled in the editor, not saved: the other page keeps the
    // duplicate title the checks found until they run again.
    client.open(&c, 1, C);
    client.replace(&c, 2, &C.replace("title: Alpha", "title: Gamma"));
    client.settle();
    assert_eq!(client.codes(&a), ["title-duplicate"]);

    // A build's output, a cache, and an image nothing shows, written, and a
    // file that was never there deleted: what the checks find can't change.
    f.write("dist/out.js", "x\n");
    f.write(".ascribe/build/index.html", "<p></p>\n");
    f.write("docs/new.png", "");
    client.watched(&[
        (&f.path("dist/out.js"), lsp_types::FileChangeType::CHANGED),
        (
            &f.path(".ascribe/build/index.html"),
            lsp_types::FileChangeType::CREATED,
        ),
        (&f.path("docs/new.png"), lsp_types::FileChangeType::CREATED),
        (&f.path("dist/gone.js"), lsp_types::FileChangeType::DELETED),
    ]);
    client.settle();
    assert_eq!(client.codes(&a), ["title-duplicate"]);

    // A page deleted on disk changes the project: they run again, with the
    // retitled page as the editor has it.
    f.remove("docs/b.md");
    client.watched(&[(&b, lsp_types::FileChangeType::DELETED)]);
    client.settle();
    assert!(client.codes(&a).is_empty());
}

#[test]
fn a_model_that_stops_loading_clears_them_and_they_come_back_when_it_loads() {
    let f = project();
    let mut client = Client::start(&f.root());
    client.settle();
    let (config, b) = (f.path("ascribe.toml"), f.path("docs/b.md"));
    let before = found(&client, &config);
    client.open(&config, 1, MODEL);
    client.replace(&config, 2, &format!("{MODEL}unknown-key = 1\n"));
    client.settle();
    assert_eq!(client.codes(&config), ["model-unknown-key"]);
    assert!(client.codes(&b).is_empty());
    assert!(client.codes(&f.path("docs/glossary.md")).is_empty());

    // The same model as before: nothing in it changed, yet every check
    // runs again.
    client.replace(&config, 3, MODEL);
    client.settle();
    assert_eq!(found(&client, &config), before);
    assert_eq!(client.codes(&b), ["page-orphan"]);
    assert_eq!(
        client.codes(&f.path("docs/glossary.md")),
        ["glossary-term-unused"]
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
    // Phrases and features are reported in `ascribe.toml`, terms at their
    // headings on the glossary page.
    for (kinds, file, text) in [
        (&["phrase", "feature"][..], "ascribe.toml", MODEL),
        (&["term"][..], "docs/glossary.md", GLOSSARY),
    ] {
        let unused_entries: Vec<String> = inventory["model"]
            .as_array()
            .expect("entries")
            .iter()
            .filter(|e| kinds.contains(&e["kind"].as_str().unwrap()))
            .filter(|e| e["uses"] == json!(0))
            .map(|e| e["key"].as_str().unwrap().to_owned())
            .collect();
        let reported: Vec<String> = client
            .diagnostics(&f.path(file))
            .iter()
            .map(|d| {
                let start = d.range.start.line as usize;
                text.lines().nth(start).unwrap().to_owned()
            })
            .collect();
        assert_eq!(unused_entries.len(), reported.len(), "{reported:?}");
        for key in &unused_entries {
            assert!(
                reported.iter().any(|line| line.contains(key.as_str())),
                "`{key}` is unused, and reported: {reported:?}"
            );
        }
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

#[test]
fn a_list_of_permalinks_thats_behind_is_brought_up_to_date_by_a_quick_fix() {
    let page = "---\ntitle: Keys\n---\n\n## Rotate keys\n@id {permalink=true}: rotate-keys\n\
                @summary: Replace a key before it expires.\n";
    let header = "# Written by Ascribe. Don't edit it: run `ascribe permalinks sync`.\n";
    let f = Fixture::new(
        crate::support::MODEL,
        &[
            ("docs/keys.md", page),
            (
                "ascribe.permalinks",
                &format!("{header}version = 1\n\n[permalinks]\n"),
            ),
        ],
    );
    let mut client = Client::start(&f.root());
    client.settle();
    let keys = f.path("docs/keys.md");
    let diagnostic = client
        .diagnostics(&keys)
        .into_iter()
        .find(|d| slug(d) == "permalinks-list-behind")
        .expect("permalinks-list-behind");
    let actions = client
        .request(
            "textDocument/codeAction",
            json!({
                "textDocument": { "uri": uri(&keys).as_str() },
                "range": diagnostic.range,
                "context": { "diagnostics": [diagnostic] }
            }),
        )
        .response_result
        .expect("actions");
    let sync = actions
        .as_array()
        .into_iter()
        .flatten()
        .find(|a| a["title"] == "Bring ascribe.permalinks up to date")
        .expect("the quick fix");
    assert_eq!(sync["isPreferred"], true);
    let list = uri(&f.path("ascribe.permalinks"));
    let edits = &sync["edit"]["changes"][list.as_str()];
    assert_eq!(
        edits[0]["newText"],
        format!("{header}version = 1\n\n[permalinks]\nrotate-keys = \"keys.md#rotate-keys\"\n")
    );

    // Once the list has it, the warning goes when the editor hears of it.
    f.write(
        "ascribe.permalinks",
        edits[0]["newText"].as_str().expect("text"),
    );
    client.watched(&[(
        &f.path("ascribe.permalinks"),
        lsp_types::FileChangeType::CHANGED,
    )]);
    client.settle();
    assert!(client.codes(&keys).is_empty(), "{:?}", client.codes(&keys));
}

#[test]
fn a_lost_permalink_is_reported_on_the_list_and_retired_by_a_quick_fix() {
    let header = "# Written by Ascribe. Don't edit it: run `ascribe permalinks sync`.\n";
    let f = Fixture::new(
        crate::support::MODEL,
        &[
            ("docs/keys.md", "---\ntitle: Keys\n---\n\nText.\n"),
            (
                "ascribe.permalinks",
                &format!(
                    "{header}version = 1\n\n[permalinks]\nrotate-keys = \"keys.md#rotate-keys\"\n"
                ),
            ),
        ],
    );
    let mut client = Client::start(&f.root());
    client.settle();
    let list = f.path("ascribe.permalinks");
    let diagnostic = client
        .diagnostics(&list)
        .into_iter()
        .find(|d| slug(d) == "permalink-lost")
        .expect("permalink-lost, on the list");
    assert_eq!(diagnostic.range.start.line, 4);
    let actions = client
        .request(
            "textDocument/codeAction",
            json!({
                "textDocument": { "uri": uri(&list).as_str() },
                "range": diagnostic.range,
                "context": { "diagnostics": [diagnostic] }
            }),
        )
        .response_result
        .expect("actions");
    let retire = actions
        .as_array()
        .into_iter()
        .flatten()
        .find(|a| a["title"] == "Retire this permalink, and write why")
        .expect("the quick fix");
    // It needs a person's sentence, so it isn't applied unasked.
    assert_eq!(retire["isPreferred"], false);
    // The project has no ascribe.addresses yet: the fix creates it.
    let addresses = uri(&f.path("ascribe.addresses"));
    let changes = retire["edit"]["documentChanges"]
        .as_array()
        .expect("document changes");
    assert_eq!(changes[0]["kind"], "create");
    assert_eq!(changes[0]["uri"], addresses.as_str());
    let text = changes[1]["edits"][0]["newText"]
        .as_str()
        .expect("the file's text")
        .to_owned();
    let entry = "\"go:rotate-keys\" = \"why this address was retired\"";
    assert!(text.contains(&format!("[retired]\n{entry}\n")), "{text}");
    // And it opens ascribe.addresses with the placeholder selected.
    let command = &retire["command"];
    assert_eq!(command["command"], "ascribe.openFile");
    assert_eq!(command["arguments"][0], addresses.as_str());
    let selection = &command["arguments"][1];
    let (line, row) = text
        .lines()
        .enumerate()
        .find(|(_, l)| *l == entry)
        .expect("the entry");
    let column = row.find("why").expect("the placeholder");
    assert_eq!(
        selection["start"],
        json!({ "line": line, "character": column })
    );
    assert_eq!(
        selection["end"],
        json!({ "line": line, "character": column + "why this address was retired".len() })
    );

    // Taken, the placeholder quiets nothing: the file doesn't read until
    // it's replaced. With the reason written, the permalink isn't lost.
    let addresses_path = f.path("ascribe.addresses");
    f.write("ascribe.addresses", &text);
    client.watched(&[(&addresses_path, lsp_types::FileChangeType::CREATED)]);
    client.settle();
    assert!(
        client
            .codes(&list)
            .iter()
            .any(|c| c.contains("permalink-lost")),
        "{:?}",
        client.codes(&list)
    );
    f.write(
        "ascribe.addresses",
        &text.replace("why this address was retired", "Merged into Keys"),
    );
    client.watched(&[(&addresses_path, lsp_types::FileChangeType::CHANGED)]);
    client.settle();
    assert!(client.codes(&list).is_empty(), "{:?}", client.codes(&list));
    // The list is behind on the retired name now, and the same quick fix as
    // for a declared name records it.
    let behind = client
        .diagnostics(&addresses_path)
        .into_iter()
        .find(|d| slug(d) == "permalinks-list-behind")
        .expect("permalinks-list-behind, on the entry");
    let actions = client
        .request(
            "textDocument/codeAction",
            json!({
                "textDocument": { "uri": addresses.as_str() },
                "range": behind.range,
                "context": { "diagnostics": [behind] }
            }),
        )
        .response_result
        .expect("actions");
    let sync = actions
        .as_array()
        .into_iter()
        .flatten()
        .find(|a| a["title"] == "Bring ascribe.permalinks up to date")
        .expect("the quick fix");
    assert_eq!(
        sync["edit"]["changes"][uri(&list).as_str()][0]["newText"],
        format!(
            "{header}version = 1\n\n[permalinks]\n\n[retired]\nrotate-keys = \"Merged into Keys\"\n"
        )
    );
}
