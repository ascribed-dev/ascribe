//! A glossary's terms in the editor (SPEC §5.4): going from a term to its
//! heading, its uses, and the hover on its heading.

#![allow(clippy::expect_used, clippy::panic, clippy::unwrap_used)]

use std::path::Path;

use ascribe_core::LineIndex;
use ascribe_lsp::Encoding;
use serde_json::{Value, json};

use crate::support::{self, Client, Fixture, uri};

const MODEL: &str = "spec = \"0.1\"\n\n[project]\ncontent-root = \"docs\"\n\n[glossary]\npages = [\"glossary.md\"]\n";

const GLOSSARY: &str = "---\ntitle: Glossary\n---\n\n## Rollout\n@id: rollout\n@term {aliases=rollouts}\n\nA plan for turning a flag on for more users.\n\n## Segment\n@term\n\nA saved group of users.\n";

const INDEX: &str = "---\ntitle: Home\n---\n\nPlan your rollouts. Read about [them](glossary.md#rollout), and [the guide](guide.md).\n";

const GUIDE: &str = "---\ntitle: Guide\n---\n\nStart a rollout for a segment.\n";

fn project() -> Fixture {
    Fixture::new(
        MODEL,
        &[
            ("docs/glossary.md", GLOSSARY),
            ("docs/index.md", INDEX),
            ("docs/guide.md", GUIDE),
        ],
    )
}

/// The position (UTF-16) of the first `needle` in `text`, plus `extra` bytes.
fn at(text: &str, needle: &str, extra: usize) -> Value {
    let i = text.find(needle).unwrap_or_else(|| panic!("no {needle:?}")) + extra;
    position(text, i)
}

fn position(text: &str, offset: usize) -> Value {
    let before = &text[..offset];
    let line = before.matches('\n').count();
    let start = before.rfind('\n').map_or(0, |i| i + 1);
    json!({ "line": line, "character": text[start..offset].encode_utf16().count() })
}

fn read(f: &Fixture, rel: &str) -> String {
    std::fs::read_to_string(f.path(rel)).expect("a file")
}

/// The text `range` covers in `text`.
fn covered(text: &str, range: &Value) -> String {
    let index = LineIndex::new(text);
    let offset = |p: &Value| {
        Encoding::Utf16
            .offset(&index, serde_json::from_value(p.clone()).unwrap())
            .unwrap()
    };
    text[offset(&range["start"])..offset(&range["end"])].to_owned()
}

fn request(client: &mut Client, method: &str, path: &Path, position: Value) -> Value {
    client
        .request(
            method,
            json!({ "textDocument": { "uri": uri(path).as_str() }, "position": position }),
        )
        .response_result
        .expect("a result")
}

#[test]
fn a_term_goes_to_its_heading() {
    let f = project();
    let index = f.path("docs/index.md");
    let mut client = Client::start(&f.root());
    let found = request(
        &mut client,
        "textDocument/definition",
        &index,
        at(INDEX, "rollouts", 3),
    );
    assert_eq!(found["uri"], uri(&f.path("docs/glossary.md")).as_str());
    assert_eq!(covered(GLOSSARY, &found["range"]), "## Rollout");
    // An author's link to the heading goes there too.
    let found = request(
        &mut client,
        "textDocument/definition",
        &index,
        at(INDEX, "[them]", 1),
    );
    assert_eq!(covered(GLOSSARY, &found["range"]), "## Rollout");
    // Prose that isn't a term goes nowhere.
    let found = request(
        &mut client,
        "textDocument/definition",
        &index,
        at(INDEX, "Plan", 1),
    );
    assert!(found.is_null(), "{found}");
}

#[test]
fn a_terms_heading_finds_its_occurrences_and_the_links_to_it() {
    let f = project();
    let glossary = f.path("docs/glossary.md");
    let mut client = Client::start(&f.root());
    let found = |client: &mut Client, position: Value| -> Vec<(String, String)> {
        let result = client
            .request(
                "textDocument/references",
                json!({
                    "textDocument": { "uri": uri(&glossary).as_str() },
                    "position": position,
                    "context": { "includeDeclaration": false },
                }),
            )
            .response_result
            .expect("references");
        result
            .as_array()
            .unwrap()
            .iter()
            .map(|l| {
                let file = l["uri"].as_str().unwrap().rsplit('/').next().unwrap();
                let text = read(&f, &format!("docs/{file}"));
                (file.to_owned(), covered(&text, &l["range"]))
            })
            .collect()
    };
    let want = vec![
        ("guide.md".to_owned(), "rollout".to_owned()),
        ("index.md".to_owned(), "rollouts".to_owned()),
        (
            "index.md".to_owned(),
            "[them](glossary.md#rollout)".to_owned(),
        ),
    ];
    assert_eq!(found(&mut client, at(GLOSSARY, "Rollout", 2)), want);
    // On its `@term`, the same.
    assert_eq!(found(&mut client, at(GLOSSARY, "@term {", 2)), want);
}

#[test]
fn a_terms_heading_shows_its_definition_and_how_many_pages_use_it() {
    let f = project();
    let glossary = f.path("docs/glossary.md");
    let mut client = Client::start(&f.root());
    let hover = request(
        &mut client,
        "textDocument/hover",
        &glossary,
        at(GLOSSARY, "Rollout", 2),
    );
    assert_eq!(
        hover["contents"]["value"],
        "**Rollout**, a glossary term\n\nA plan for turning a flag on for more users.\n\nUsed on 2 pages."
    );
    let hover = request(
        &mut client,
        "textDocument/hover",
        &glossary,
        at(GLOSSARY, "Segment", 2),
    );
    assert_eq!(
        hover["contents"]["value"],
        "**Segment**, a glossary term\n\nA saved group of users.\n\nUsed on 1 page."
    );
    // A heading that isn't a term shows nothing.
    let text = format!("{GLOSSARY}\n## See also\n\nThe guide.\n");
    client.open(&glossary, 1, &text);
    let hover = request(
        &mut client,
        "textDocument/hover",
        &glossary,
        at(&text, "See also", 2),
    );
    assert!(hover.is_null(), "{hover}");
}