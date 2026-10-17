//! `ascribe.nav` in the editor: its problems as it's typed, their quick
//! fixes, the quick fix on a page its section leaves out, which edits the
//! file, and completion in it. The rules are `ascribe-check`'s.

#![allow(clippy::expect_used, clippy::panic, clippy::unwrap_used)]

use lsp_types::{Diagnostic, Position};
use serde_json::{Value, json};

use crate::support::{Client, Fixture, MODEL, slug, uri};

const PAGE: &str = "---\ntitle: Options\n---\n\nText.\n";
const NAV: &str =
    "version = 1\n\n[[sections]]\nfolder = \"Reference/\"\npages = [\"Options.md\"]\n";

fn project() -> Fixture {
    Fixture::new(
        MODEL,
        &[
            ("ascribe.nav", NAV),
            ("docs/index.md", "---\ntitle: Home\n---\n\nText.\n"),
            ("docs/Reference/Options.md", PAGE),
            (
                "docs/Reference/Defaults.md",
                "---\ntitle: Defaults\n---\n\nText.\n",
            ),
        ],
    )
}

/// The quick fixes offered on a file's diagnostic of `slug_of`.
fn actions(client: &mut Client, path: &std::path::Path, slug_of: &str) -> Vec<Value> {
    let diagnostic: Diagnostic = client
        .diagnostics(path)
        .into_iter()
        .find(|d| slug(d) == slug_of)
        .unwrap_or_else(|| panic!("no {slug_of} in {}", path.display()));
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
}

/// The text an action's edit of `path` makes of `text`.
fn applied(action: &Value, path: &std::path::Path, text: &str) -> String {
    let edits = action["edit"]["changes"][uri(path).as_str()]
        .as_array()
        .unwrap_or_else(|| panic!("no edit of {}: {action}", path.display()));
    let index = ascribe_core::LineIndex::new(text);
    let offset = |p: &Value| {
        let position: Position = serde_json::from_value(p.clone()).unwrap();
        index
            .offset(ascribe_core::LineCol {
                line: position.line,
                col: position.character,
            })
            .unwrap()
    };
    let edits: Vec<ascribe_core::TextEdit> = edits
        .iter()
        .map(|e| {
            ascribe_core::TextEdit::replace(
                offset(&e["range"]["start"])..offset(&e["range"]["end"]),
                e["newText"].as_str().unwrap(),
            )
        })
        .collect();
    ascribe_core::apply_edits(text, &edits).unwrap()
}

#[test]
fn a_page_its_section_leaves_out_is_placed_by_a_quick_fix_to_the_file() {
    let f = project();
    let mut client = Client::start(&f.root());
    client.settle();
    let page = f.path("docs/Reference/Defaults.md");
    let nav = f.path("ascribe.nav");
    assert_eq!(client.codes(&page), ["nav-page-unplaced"]);
    let actions = actions(&mut client, &page, "nav-page-unplaced");
    let place = actions
        .iter()
        .find(|a| a["title"] == "Add `Defaults.md` to `Reference/`'s pages in ascribe.nav")
        .unwrap_or_else(|| panic!("the fix: {actions:?}"));
    assert_eq!(place["isPreferred"], true, "it's safe");
    let placed = applied(place, &nav, NAV);
    assert_eq!(
        placed,
        "version = 1\n\n[[sections]]\nfolder = \"Reference/\"\npages = [\"Options.md\", \"Defaults.md\"]\n"
    );

    // The editor applies it to the file; the page has nothing left.
    client.open(&nav, 1, NAV);
    client.replace(&nav, 2, &placed);
    client.settle();
    assert!(client.codes(&page).is_empty(), "{:?}", client.codes(&page));
}

#[test]
fn the_files_problems_are_shown_as_its_typed_with_their_fixes() {
    let f = project();
    let mut client = Client::start(&f.root());
    client.settle();
    let nav = f.path("ascribe.nav");
    assert!(client.codes(&nav).is_empty());
    client.open(&nav, 1, NAV);
    let typed = NAV.replace(
        "pages = [\"Options.md\"]",
        "pages = [\"Options.md\", \"Defualts.md\", \"Options.md\"]",
    );
    client.replace(&nav, 2, &typed);
    client.settle();
    assert_eq!(client.codes(&nav), ["nav-duplicate", "nav-page-unknown"]);
    let renamed = actions(&mut client, &nav, "nav-page-unknown");
    assert_eq!(renamed[0]["title"], "Change it to `Defaults.md`");
    assert_eq!(renamed[0]["isPreferred"], false, "it's unsafe");
    let removed = actions(&mut client, &nav, "nav-duplicate");
    assert_eq!(
        applied(&removed[0], &nav, &typed),
        NAV.replace(
            "pages = [\"Options.md\"]",
            "pages = [\"Options.md\", \"Defualts.md\"]"
        )
    );

    // A file that doesn't read is shown as typed, and closing it goes back
    // to the disk's.
    client.replace(&nav, 3, "version = 2\n");
    client.settle();
    assert_eq!(client.codes(&nav), ["nav-invalid"]);
    client.close(&nav);
    client.settle();
    assert!(client.codes(&nav).is_empty());
}

#[test]
fn completion_offers_the_sections_and_their_pages_not_written_yet() {
    let f = project();
    let mut client = Client::start(&f.root());
    client.settle();
    let nav = f.path("ascribe.nav");
    let text = "version = 1\n\n[[sections]]\nfolder = \"Reference/\"\npages = [\"Options.md\", \"\"]\n\n[[sections]]\nfolder = \"\"\n";
    client.open(&nav, 1, text);
    let complete = |client: &mut Client, line: u32, character: u32| -> Vec<String> {
        let items = client
            .request(
                "textDocument/completion",
                json!({
                    "textDocument": { "uri": uri(&nav).as_str() },
                    "position": { "line": line, "character": character }
                }),
            )
            .response_result
            .expect("a result");
        items
            .as_array()
            .unwrap()
            .iter()
            .map(|i| i["label"].as_str().unwrap().to_owned())
            .collect()
    };
    // In the second string of `pages`.
    assert_eq!(complete(&mut client, 4, 24), ["Defaults.md"]);
    // In the second section's `folder`, which lists the root.
    assert!(complete(&mut client, 7, 10).is_empty(), "both are listed");
}
