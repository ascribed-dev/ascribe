//! `[from]` in the editor (#813): the hover on a name in a `take` list says
//! where it's written, Go to Definition opens it there, and the file it's
//! taken from is followed on disk as `ascribe.toml` is.

#![allow(clippy::expect_used, clippy::panic, clippy::unwrap_used)]

use std::path::Path;

use lsp_types::{FileChangeType, Position};
use serde_json::{Value, json};

use crate::support::{Client, Fixture, MODEL, uri};

/// The other project's file, beside this one's content: its `[phrases]` on
/// line 3.
const SHARED: &str = "spec = \"0.1\"\n\n[phrases]\nproduct = \"Quill\"\n";

fn model() -> String {
    format!("{MODEL}\n[from.\"shared/ascribe.toml\"]\ntake = [\"phrases.product\"]\n")
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
    f.write("shared/ascribe.toml", "spec = \"0.1\"\n");
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
