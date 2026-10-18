//! A glossary's terms in the editor (SPEC §5.4): going from a term to its
//! heading, its uses, completing `@term`, the hover on its heading, and the
//! quick fixes of a glossary page's problems.

#![allow(clippy::expect_used, clippy::panic, clippy::unwrap_used)]

use std::path::Path;

use ascribe_core::{LineIndex, Span, TextEdit, apply_edits};
use ascribe_lsp::Encoding;
use lsp_types::FileChangeType;
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

/// Applies each file's edits in a workspace edit to the files on disk, and
/// tells the server they changed.
fn apply(client: &mut Client, f: &Fixture, edit: &Value) {
    let mut changed = Vec::new();
    for (uri_text, edits) in edit["changes"].as_object().expect("changes") {
        let path = f
            .root()
            .join(
                uri_text
                    .rsplit_once(&*format!("{}/", uri(&f.root()).as_str()))
                    .map_or_else(|| panic!("{uri_text} outside the project"), |(_, r)| r),
            )
            .to_path_buf();
        let text = std::fs::read_to_string(&path).expect("the file");
        let index = LineIndex::new(&text);
        let edits: Vec<TextEdit> = edits
            .as_array()
            .unwrap()
            .iter()
            .map(|e| {
                let offset = |p: &Value| {
                    Encoding::Utf16
                        .offset(&index, serde_json::from_value(p.clone()).unwrap())
                        .unwrap()
                };
                TextEdit::replace(
                    Span::new(offset(&e["range"]["start"]), offset(&e["range"]["end"])),
                    e["newText"].as_str().unwrap(),
                )
            })
            .collect();
        std::fs::write(&path, apply_edits(&text, &edits).unwrap()).unwrap();
        changed.push(path);
    }
    let events: Vec<_> = changed
        .iter()
        .map(|p| (p.as_path(), FileChangeType::CHANGED))
        .collect();
    client.watched(&events);
    client.settle();
}

/// The quick fixes the server offers for the diagnostic `slug` on `path`.
fn fixes(client: &mut Client, path: &Path, slug: &str) -> Vec<Value> {
    let diagnostic = client
        .diagnostics(path)
        .into_iter()
        .find(|d| support::slug(d) == slug)
        .unwrap_or_else(|| panic!("no {slug} on {}", path.display()));
    let diagnostic = serde_json::to_value(diagnostic).unwrap();
    client
        .request(
            "textDocument/codeAction",
            json!({
                "textDocument": { "uri": uri(path).as_str() },
                "range": diagnostic["range"].clone(),
                "context": { "diagnostics": [diagnostic] },
            }),
        )
        .response_result
        .expect("actions")
        .as_array()
        .cloned()
        .unwrap_or_default()
}

/// Every diagnostic's slug the server publishes, across the project, in
/// order.
fn slugs(client: &Client) -> Vec<String> {
    let mut all: Vec<String> = client
        .latest
        .values()
        .flat_map(|p| p.diagnostics.iter().map(support::slug))
        .collect();
    all.sort();
    all
}

/// Asserts that the diagnostics now are `before` without one `fixed`.
fn left_behind_none(client: &Client, mut before: Vec<String>, fixed: Option<&str>) {
    if let Some(fixed) = fixed {
        let at = before
            .iter()
            .position(|s| s == fixed)
            .expect("it was there");
        before.remove(at);
    }
    assert_eq!(slugs(client), before);
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

/// The labels completion offers in `marked` (with `$0` at the cursor), in
/// place of `path`'s text.
fn completed(client: &mut Client, path: &Path, marked: &str) -> Vec<String> {
    let i = marked.find("$0").expect("a cursor");
    let text = marked.replacen("$0", "", 1);
    client.open(path, 100, &text);
    let list = request(client, "textDocument/completion", path, position(&text, i));
    list["items"]
        .as_array()
        .cloned()
        .unwrap_or_default()
        .iter()
        .map(|i| i["label"].as_str().unwrap().to_owned())
        .collect()
}

#[test]
fn term_completes_at_the_top_of_a_section_on_a_glossary_page() {
    let f = project();
    let glossary = f.path("docs/glossary.md");
    let mut client = Client::start(&f.root());
    let term = "term".to_owned();
    let page = |cursor: &str| format!("{GLOSSARY}\n## Region\n{cursor}\n");
    assert!(completed(&mut client, &glossary, &page("@te$0")).contains(&term));
    // Under the heading's other directives.
    assert!(completed(&mut client, &glossary, &page("@id: region\n@te$0")).contains(&term));
    // Not below the section's first block, nor on a heading that's a term.
    assert!(!completed(&mut client, &glossary, &page("\nA place.\n\n@te$0")).contains(&term));
    assert!(!completed(&mut client, &glossary, &page("@term\n@te$0")).contains(&term));
    // Its attributes, and their values.
    assert_eq!(
        completed(&mut client, &glossary, &page("@term {$0}")),
        ["aliases", "match", "case-sensitive"]
    );
    assert_eq!(
        completed(&mut client, &glossary, &page("@term {match=$0}")),
        ["first", "every", "marked"]
    );
    // Not on a page that isn't a glossary page.
    let index = f.path("docs/index.md");
    let names = completed(&mut client, &index, &format!("{INDEX}\n## Region\n@te$0\n"));
    assert!(names.contains(&"id".to_owned()), "{names:?}");
    assert!(!names.contains(&term), "{names:?}");
}

#[test]
fn a_term_off_a_glossary_page_is_fixed_by_listing_the_page() {
    let page = "---\ntitle: Terms\n---\n\n# Terms\n\n## Flag\n@term\n\nA switch.\n";
    let f = Fixture::new(
        MODEL,
        &[("docs/glossary.md", GLOSSARY), ("docs/terms.md", page)],
    );
    let terms = f.path("docs/terms.md");
    let mut client = Client::start(&f.root());
    client.settle();
    let before = slugs(&client);
    let found = fixes(&mut client, &terms, "term-off-glossary-page");
    let fix = found
        .iter()
        .find(|a| a["title"] == "List `terms.md` in `[glossary] pages`")
        .unwrap_or_else(|| panic!("{found:?}"));
    apply(&mut client, &f, &fix["edit"]);
    assert!(read(&f, "ascribe.toml").contains("pages = [\"glossary.md\", \"terms.md\"]"));
    left_behind_none(&client, before, Some("term-off-glossary-page"));
}

#[test]
fn a_missing_definition_is_fixed_with_a_paragraph_to_write_it_in() {
    let glossary = GLOSSARY.replace("@term\n\nA saved group of users.\n", "@term\n- A list.\n");
    let f = Fixture::new(MODEL, &[("docs/glossary.md", &glossary)]);
    let path = f.path("docs/glossary.md");
    let mut client = Client::start(&f.root());
    client.settle();
    let before = slugs(&client);
    let found = fixes(&mut client, &path, "glossary-definition-missing");
    let fix = found
        .iter()
        .find(|a| a["title"] == "Write a definition of Segment")
        .unwrap_or_else(|| panic!("{found:?}"));
    apply(&mut client, &f, &fix["edit"]);
    let after = read(&f, "docs/glossary.md");
    assert!(
        after.ends_with(
            "## Segment\n@term\n\nWhat Segment means, in a sentence or two.\n\n- A list.\n"
        ),
        "{after}"
    );
    // The paragraph is selected, for the author to type over.
    let command = &fix["command"];
    assert_eq!(command["command"], "ascribe.selectRange");
    assert_eq!(command["arguments"][0], uri(&path).as_str());
    assert_eq!(
        covered(&after, &command["arguments"][1]),
        "What Segment means, in a sentence or two."
    );
    left_behind_none(&client, before, Some("glossary-definition-missing"));
}

#[test]
fn a_term_without_an_id_is_given_a_stable_one() {
    let f = project();
    let glossary = f.path("docs/glossary.md");
    let mut client = Client::start(&f.root());
    client.settle();
    let before = slugs(&client);
    let i = GLOSSARY.find("Segment").unwrap();
    let range = json!({ "start": position(GLOSSARY, i), "end": position(GLOSSARY, i) });
    let actions = client
        .request(
            "textDocument/codeAction",
            json!({
                "textDocument": { "uri": uri(&glossary).as_str() },
                "range": range,
                "context": { "diagnostics": [] },
            }),
        )
        .response_result
        .expect("actions");
    let add = actions
        .as_array()
        .unwrap()
        .iter()
        .find(|a| a["title"] == "Add a stable @id for this heading")
        .expect("the action");
    apply(&mut client, &f, &add["edit"]);
    assert!(read(&f, "docs/glossary.md").contains("## Segment\n@id: segment\n@term\n"));
    left_behind_none(&client, before, None);
}