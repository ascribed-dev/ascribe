//! `[from]` in the editor (#813): the hover on a name in a `take` list says
//! where it's written, Go to Definition opens it there, and the file it's
//! taken from is followed on disk as `ascribe.toml` is. An edit of a taken
//! key is made in the file that holds it (#814): a rename, and Project
//! Settings' `setSetting`.

#![allow(clippy::expect_used, clippy::panic, clippy::unwrap_used)]

use std::path::Path;

use ascribe_core::{LineIndex, RelPath, WideEncoding, WideLineCol};
use lsp_types::{FileChangeType, Position};
use serde_json::{Value, json};

use crate::support::{Client, Fixture, MODEL, uri};

/// The other project's file, beside this one's content: its `[phrases]` on
/// line 3.
const SHARED: &str = "spec = \"0.1\"\n\n[phrases]\nproduct = \"Quill\"\n\n[consumer]\n# The address.\nsite = \"https://a.example\"\n";

fn model() -> String {
    format!(
        "{MODEL}\n[from.\"shared/ascribe.toml\"]\ntake = [\"phrases.product\", \"consumer.site\"]\n\n[consumer]\nbase-path = \"/\"\n"
    )
}

fn project() -> Fixture {
    Fixture::new(
        &model(),
        &[
            ("shared/ascribe.toml", SHARED),
            (
                "docs/index.md",
                "---\ntitle: Home\n---\n\n{product} writes docs.\n",
            ),
        ],
    )
}

/// Where `phrases.product` is in the `take` list of `text`: inside the name.
fn on_the_name(text: &str) -> Position {
    let line = text
        .lines()
        .position(|l| l.starts_with("take = "))
        .expect("a take list");
    Position::new(u32::try_from(line).unwrap(), 12)
}

fn ask(client: &mut Client, method: &str, config: &Path, at: Position) -> Value {
    client
        .request(
            method,
            json!({ "textDocument": { "uri": uri(config).as_str() }, "position": at }),
        )
        .response_result
        .expect("an answer")
}

#[test]
fn the_hover_on_a_taken_name_says_where_its_written() {
    let f = project();
    let config = f.path("ascribe.toml");
    let text = model();
    let mut client = Client::start(&f.root());
    client.open(&config, 1, &text);
    client.settle();

    let hover = ask(
        &mut client,
        "textDocument/hover",
        &config,
        on_the_name(&text),
    );
    let value = hover["contents"]["value"].as_str().expect("a hover");
    assert!(
        value.contains("taken from `shared/ascribe.toml`, where it's written, at line 4"),
        "{value}"
    );
}

#[test]
fn go_to_definition_on_a_taken_name_opens_the_file_its_taken_from() {
    let f = project();
    let config = f.path("ascribe.toml");
    let text = model();
    let mut client = Client::start(&f.root());
    client.open(&config, 1, &text);
    client.settle();

    let location = ask(
        &mut client,
        "textDocument/definition",
        &config,
        on_the_name(&text),
    );
    assert_eq!(
        location["uri"].as_str(),
        Some(uri(&f.path("shared/ascribe.toml")).as_str())
    );
    // `product`, at the start of the file's fourth line.
    assert_eq!(
        location["range"]["start"],
        json!({ "line": 3, "character": 0 })
    );
}

#[test]
fn the_file_its_taken_from_is_followed_on_disk() {
    let f = project();
    let config = f.path("ascribe.toml");
    let page = f.path("docs/index.md");
    let mut client = Client::start(&f.root());
    client.open(
        &page,
        1,
        "---\ntitle: Home\n---\n\n{product} writes docs.\n",
    );
    client.settle();
    assert!(
        client.codes(&config).is_empty(),
        "{:?}",
        client.codes(&config)
    );

    // The other file loses the phrase: the name in `take` names nothing.
    f.write(
        "shared/ascribe.toml",
        &SHARED.replace("product = \"Quill\"\n", ""),
    );
    client.watched(&[(&f.path("shared/ascribe.toml"), FileChangeType::CHANGED)]);
    client.settle();
    assert_eq!(client.codes(&config), vec!["model-from-take".to_owned()]);

    // And has it again.
    f.write("shared/ascribe.toml", SHARED);
    client.watched(&[(&f.path("shared/ascribe.toml"), FileChangeType::CHANGED)]);
    client.settle();
    assert!(
        client.codes(&config).is_empty(),
        "{:?}",
        client.codes(&config)
    );
}

/// The text on disk of the project's file at `u`, a URI.
fn on_disk(f: &Fixture, u: &str) -> String {
    let path = ["ascribe.toml", "shared/ascribe.toml", "docs/index.md"]
        .iter()
        .map(|p| f.path(p))
        .find(|p| uri(p).as_str() == u)
        .unwrap_or_else(|| panic!("{u} isn't one of the project's files"));
    std::fs::read_to_string(path).unwrap()
}

/// The texts a workspace edit leaves, by URI, each applied to the text
/// `before` gives for it.
fn applied(edit: &Value, before: impl Fn(&str) -> String) -> Vec<(String, String)> {
    let changes = edit["changes"].as_object().expect("changes");
    let mut out: Vec<(String, String)> = changes
        .iter()
        .filter(|(_, edits)| !edits.as_array().unwrap().is_empty())
        .map(|(uri, edits)| {
            let text = before(uri);
            let index = LineIndex::new(&text);
            let offset = |p: &Value| {
                index
                    .wide_offset(
                        WideEncoding::Utf16,
                        WideLineCol {
                            line: p["line"].as_u64().unwrap() as u32,
                            col: p["character"].as_u64().unwrap() as u32,
                        },
                    )
                    .expect("an offset")
            };
            let edits: Vec<ascribe_core::TextEdit> = edits
                .as_array()
                .unwrap()
                .iter()
                .map(|e| {
                    ascribe_core::TextEdit::replace(
                        ascribe_core::Span::new(
                            offset(&e["range"]["start"]),
                            offset(&e["range"]["end"]),
                        ),
                        e["newText"].as_str().unwrap(),
                    )
                })
                .collect();
            (
                uri.clone(),
                ascribe_core::apply_edits(&text, &edits).expect("edits that apply"),
            )
        })
        .collect();
    out.sort();
    out
}

#[test]
fn renaming_a_taken_phrase_renames_it_where_its_written_and_in_take() {
    let f = project();
    let config = f.path("ascribe.toml");
    let shared = f.path("shared/ascribe.toml");
    let page = f.path("docs/index.md");
    let mut client = Client::start(&f.root());
    let text = "---\ntitle: Home\n---\n\n{product} writes docs.\n";
    client.open(&page, 1, text);
    client.settle();

    let edit = client
        .request(
            "textDocument/rename",
            json!({
                "textDocument": { "uri": uri(&page).as_str() },
                "position": { "line": 4, "character": 3 },
                "newName": "name",
            }),
        )
        .response_result
        .expect("an edit");
    let after = applied(&edit, |u| {
        if u == uri(&page).as_str() {
            text.to_owned()
        } else {
            on_disk(&f, u)
        }
    });
    let of = |path: &Path| {
        after
            .iter()
            .find(|(u, _)| u == uri(path).as_str())
            .map(|(_, t)| t.as_str())
            .unwrap_or_else(|| panic!("{} didn't change: {edit}", path.display()))
    };
    assert_eq!(of(&page), "---\ntitle: Home\n---\n\n{name} writes docs.\n");
    assert_eq!(of(&shared), SHARED.replace("product = ", "name = "));
    assert_eq!(
        of(&config),
        model().replace("\"phrases.product\"", "\"phrases.name\"")
    );
}

/// Sets `[consumer] key` to `value` through Project Settings, as shown from
/// the disk.
fn set(client: &mut Client, config: &Path, key: &str, value: Value) -> Value {
    client
        .request(
            "ascribe/edit",
            json!({
                "textDocument": { "uri": uri(config).as_str() },
                "range": { "start": { "line": 0, "character": 0 }, "end": { "line": 0, "character": 0 } },
                "action": "setSetting",
                "args": { "table": "consumer", "key": key, "value": value, "shown": null },
            }),
        )
        .response_result
        .expect("an answer")
}

#[test]
fn project_settings_shows_and_sets_a_taken_setting_where_its_written() {
    let f = project();
    let config = f.path("ascribe.toml");
    let shared = f.path("shared/ascribe.toml");
    let mut client = Client::start(&f.root());
    client.settle();

    let answer = client
        .request(
            "ascribe/settings",
            json!({ "textDocument": { "uri": uri(&config).as_str() } }),
        )
        .response_result
        .expect("an answer");
    let site = answer["settings"]
        .as_array()
        .unwrap()
        .iter()
        .find(|s| s["table"] == "consumer" && s["key"] == "site")
        .expect("[consumer] site");
    assert_eq!(site["value"], "https://a.example", "{site}");
    assert_eq!(site["set"], Value::Null, "it isn't set in this file");

    let read = |u: &str| on_disk(&f, u);
    let answer = set(&mut client, &config, "site", json!("https://b.example"));
    let after = applied(&answer["edit"], read);
    assert_eq!(
        after,
        vec![(
            uri(&shared).as_str().to_owned(),
            SHARED.replace("https://a.example", "https://b.example")
        )],
        "{answer}"
    );

    // A key of the same table that isn't taken is this file's.
    let answer = set(&mut client, &config, "base-path", json!("/docs/"));
    let after = applied(&answer["edit"], read);
    assert_eq!(
        after,
        vec![(
            uri(&config).as_str().to_owned(),
            model().replace("base-path = \"/\"", "base-path = \"/docs/\"")
        )]
    );
}

/// A project that takes a dimension and the glossary from `shared`.
fn taking_tables() -> Fixture {
    Fixture::new(
        &format!(
            "{MODEL}\n[from.\"shared/ascribe.toml\"]\ntake = [\"dimensions.pm\", \"glossary\"]\n"
        ),
        &[
            (
                "shared/ascribe.toml",
                "spec = \"0.1\"\n\n[dimensions.pm]\nvalues = [\"npm\", \"yarn\"]\n\n[glossary]\npages = [\"terms.md\"]\n",
            ),
            ("docs/index.md", "---\ntitle: Home\n---\n\nWelcome.\n"),
            (
                "docs/terms.md",
                "---\ntitle: Terms\n---\n\n## Widget\n@term\n\nA thing.\n",
            ),
        ],
    )
}

#[test]
fn renaming_a_taken_value_or_a_page_a_taken_key_names_edits_where_its_written() {
    use ascribe_lsp::{Renaming, rename};
    let f = taking_tables();
    let config = f.path("ascribe.toml");
    let shared = f.path("shared/ascribe.toml");
    let model = std::fs::read_to_string(&config).unwrap();

    let renamed = rename(
        &config,
        &Renaming::Value {
            value: "yarn".to_owned(),
            new: "pnpm".to_owned(),
        },
    )
    .expect("renamed");
    let files: Vec<_> = renamed.files.iter().map(|r| r.path.clone()).collect();
    assert_eq!(files, vec![shared.clone()]);
    assert!(
        renamed.files[0]
            .text
            .contains("values = [\"npm\", \"pnpm\"]"),
        "{}",
        renamed.files[0].text
    );

    let renamed = rename(
        &config,
        &Renaming::File {
            from: RelPath::parse("terms.md").unwrap(),
            to: RelPath::parse("glossary.md").unwrap(),
        },
    )
    .expect("renamed");
    let shared_after = renamed
        .files
        .iter()
        .find(|r| r.path == shared)
        .expect("the glossary's pages follow the page");
    assert!(
        shared_after.text.contains("pages = [\"glossary.md\"]"),
        "{}",
        shared_after.text
    );
    assert!(renamed.files.iter().all(|r| r.path != config));
    assert_eq!(std::fs::read_to_string(&config).unwrap(), model);
}

/// A project that takes a phrase and a dimension from `shared`, a project
/// of its own whose pages use them too.
fn taking_from_a_project() -> Fixture {
    Fixture::new(
        &format!(
            "{MODEL}\n[from.\"shared/ascribe.toml\"]\ntake = [\"phrases.product\", \"dimensions.pm\"]\n"
        ),
        &[
            (
                "shared/ascribe.toml",
                &format!(
                    "{MODEL}\n[phrases]\nproduct = \"Quill\"\n\n[dimensions.pm]\nvalues = [\"npm\", \"yarn\"]\n"
                ),
            ),
            (
                "shared/docs/index.md",
                "---\ntitle: Shared\n---\n\n{product} installs.\n\n@variant {pm=yarn}:\nWith yarn.\n@end\n",
            ),
            (
                "docs/index.md",
                "---\ntitle: Home\n---\n\n{product} writes docs.\n",
            ),
        ],
    )
}

#[test]
fn renaming_a_taken_key_renames_it_in_the_pages_of_the_project_that_writes_it() {
    use ascribe_lsp::{Renaming, rename};
    let f = taking_from_a_project();
    let config = f.path("ascribe.toml");
    let shared = f.path("shared/ascribe.toml");
    let shared_page = f.path("shared/docs/index.md");
    let page = f.path("docs/index.md");
    let text_of = |renamed: &ascribe_lsp::Renamed, path: &Path| {
        renamed
            .files
            .iter()
            .find(|r| r.path == path)
            .unwrap_or_else(|| panic!("{} didn't change: {renamed:?}", path.display()))
            .text
            .clone()
    };

    let renamed = rename(
        &config,
        &Renaming::Phrase {
            key: "product".to_owned(),
            new: "name".to_owned(),
        },
    )
    .expect("renamed");
    let files: Vec<_> = renamed.files.iter().map(|r| r.path.clone()).collect();
    assert_eq!(
        files,
        vec![
            config.clone(),
            page.clone(),
            shared.clone(),
            shared_page.clone()
        ]
    );
    assert!(text_of(&renamed, &config).contains("\"phrases.name\""));
    assert!(text_of(&renamed, &page).contains("{name} writes docs."));
    assert!(text_of(&renamed, &shared).contains("\nname = \"Quill\""));
    assert!(text_of(&renamed, &shared_page).contains("{name} installs."));

    let renamed = rename(
        &config,
        &Renaming::Value {
            value: "yarn".to_owned(),
            new: "pnpm".to_owned(),
        },
    )
    .expect("renamed");
    let files: Vec<_> = renamed.files.iter().map(|r| r.path.clone()).collect();
    assert_eq!(files, vec![shared.clone(), shared_page.clone()]);
    assert!(text_of(&renamed, &shared).contains("values = [\"npm\", \"pnpm\"]"));
    assert!(text_of(&renamed, &shared_page).contains("@variant {pm=pnpm}:"));

    // A new name the other project has already is refused, naming the file.
    f.write(
        "shared/ascribe.toml",
        &format!(
            "{MODEL}\n[phrases]\nproduct = \"Quill\"\nname = \"Q\"\n\n[dimensions.pm]\nvalues = [\"npm\", \"yarn\"]\n"
        ),
    );
    let refused = rename(
        &config,
        &Renaming::Phrase {
            key: "product".to_owned(),
            new: "name".to_owned(),
        },
    )
    .expect_err("refused");
    assert!(
        refused.to_string().contains("`shared/ascribe.toml`"),
        "{refused}"
    );
}

#[test]
fn the_editors_rename_of_a_taken_phrase_renames_it_in_the_other_projects_pages() {
    let f = taking_from_a_project();
    let shared_page = f.path("shared/docs/index.md");
    let page = f.path("docs/index.md");
    let mut client = Client::start(&f.root());
    let text = "---\ntitle: Home\n---\n\n{product} writes docs.\n";
    client.open(&page, 1, text);
    client.settle();

    let edit = client
        .request(
            "textDocument/rename",
            json!({
                "textDocument": { "uri": uri(&page).as_str() },
                "position": { "line": 4, "character": 3 },
                "newName": "name",
            }),
        )
        .response_result
        .expect("an edit");
    let after = applied(&edit, |u| {
        let path = [
            "ascribe.toml",
            "shared/ascribe.toml",
            "docs/index.md",
            "shared/docs/index.md",
        ]
        .iter()
        .map(|p| f.path(p))
        .find(|p| uri(p).as_str() == u)
        .unwrap_or_else(|| panic!("{u} isn't one of the projects' files"));
        std::fs::read_to_string(path).unwrap()
    });
    let shared_after = after
        .iter()
        .find(|(u, _)| u == uri(&shared_page).as_str())
        .unwrap_or_else(|| panic!("the other project's page didn't change: {edit}"));
    assert!(
        shared_after.1.contains("{name} installs."),
        "{}",
        shared_after.1
    );
    assert_eq!(after.len(), 4, "{edit}");
}
