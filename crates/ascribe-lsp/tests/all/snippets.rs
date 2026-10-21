//! A `@snippet`'s address in the editor: Go to Definition opens its code at
//! the region's start tag, or at the file's first line, and the hover shows
//! the code as the page does.

#![allow(clippy::expect_used, clippy::panic, clippy::unwrap_used)]

use std::path::Path;

use crate::support::{Client, Fixture, MODEL, uri};
use serde_json::{Value, json};

const PAGE: &str = "docs/index.md";

const CODE: &str = "// The greeting.\nimport { log } from \"./log\";\n\n// :snippet-start: greet\nfunction greet(name: string): string {\n  const secret = 1; // :remove:\n  return `Hello, ${name}!`;\n}\n// :snippet-end:\n";

/// A project whose source `code` is the folder `code/`, with `CODE` in
/// `code/greet.ts`, and a page that's `page`.
fn project(page: &str) -> Fixture {
    let model = format!("{MODEL}\n[sources.code]\npath = \"code\"\n");
    Fixture::new(
        &model,
        &[
            ("code/greet.ts", CODE),
            (PAGE, &format!("---\ntitle: Greet\n---\n\n{page}\n")),
        ],
    )
}

fn start(folder: &Path) -> Client {
    let mut client = Client::start(folder);
    client.settle();
    client
}

/// The LSP position of the first `needle` in `text`, plus `extra` bytes.
fn at(text: &str, needle: &str, extra: usize) -> Value {
    let i = text.find(needle).unwrap_or_else(|| panic!("no {needle:?}")) + extra;
    let before = &text[..i];
    let line = before.matches('\n').count();
    let start = before.rfind('\n').map_or(0, |n| n + 1);
    json!({ "line": line, "character": text[start..i].chars().map(char::len_utf16).sum::<usize>() })
}

fn request(client: &mut Client, method: &str, page: &Path, pos: Value) -> Value {
    client
        .request(
            method,
            json!({ "textDocument": { "uri": uri(page).as_str() }, "position": pos }),
        )
        .response_result
        .expect("a result")
}

/// Go to Definition on the page's address, `address`.
fn definition(client: &mut Client, page: &Path, address: &str) -> Value {
    let text = std::fs::read_to_string(page).expect("the page");
    request(
        client,
        "textDocument/definition",
        page,
        at(&text, address, 3),
    )
}

fn hover(client: &mut Client, page: &Path, address: &str) -> String {
    let text = std::fs::read_to_string(page).expect("the page");
    let r = request(client, "textDocument/hover", page, at(&text, address, 3));
    r["contents"]["value"]
        .as_str()
        .unwrap_or_else(|| panic!("a hover: {r}"))
        .to_owned()
}

/// The location's file and its range's start line and character.
fn place(location: &Value) -> (String, u64, u64) {
    let location = location
        .as_array()
        .and_then(|a| a.first())
        .unwrap_or(location);
    let start = &location["range"]["start"];
    (
        location["uri"].as_str().expect("a uri").to_owned(),
        start["line"].as_u64().expect("a line"),
        start["character"].as_u64().expect("a character"),
    )
}

#[test]
fn definition_opens_the_region_at_its_start_tag() {
    let f = project("@snippet: code:greet.ts#greet");
    let mut client = start(&f.root());
    let found = definition(&mut client, &f.path(PAGE), "code:greet.ts#greet");
    let file = uri(&f.path("code/greet.ts"));
    assert_eq!(place(&found), (file.as_str().to_owned(), 3, 0), "{found}");
    // Elsewhere on the line, it's a directive line like any other.
    let text = std::fs::read_to_string(f.path(PAGE)).unwrap();
    let elsewhere = request(
        &mut client,
        "textDocument/definition",
        &f.path(PAGE),
        at(&text, "@snippet", 2),
    );
    assert!(!elsewhere.to_string().contains("greet.ts"), "{elsewhere}");
}

#[test]
fn definition_opens_a_whole_file_at_its_first_line() {
    let f = project("@snippet: code:greet.ts");
    let mut client = start(&f.root());
    let found = definition(&mut client, &f.path(PAGE), "code:greet.ts");
    let file = uri(&f.path("code/greet.ts"));
    assert_eq!(place(&found), (file.as_str().to_owned(), 0, 0), "{found}");
}

#[test]
fn definition_opens_a_file_outside_the_projects_folder() {
    // The docs in docs/, with the repository around them as their source.
    let f = Fixture::new("spec = \"0.1\"\n", &[]);
    std::fs::create_dir_all(f.path(".git")).unwrap();
    f.write(
        "docs/ascribe.toml",
        &format!("{MODEL}\n[sources.code]\npath = \"..\"\ninclude = [\"code/**\"]\n"),
    );
    f.write(
        "docs/docs/index.md",
        "---\ntitle: Greet\n---\n\n@snippet: code:code/greet.ts#greet\n",
    );
    f.write("code/greet.ts", CODE);
    let mut client = start(&f.path("docs"));
    let page = f.path("docs/docs/index.md");
    let found = definition(&mut client, &page, "code:code/greet.ts#greet");
    let file = uri(&f.path("code/greet.ts"));
    assert_eq!(place(&found), (file.as_str().to_owned(), 3, 0), "{found}");
}

#[test]
fn definition_on_a_missing_region_goes_nowhere() {
    let f = project("@snippet: code:greet.ts#nothing\n\n@snippet: code:gone.ts");
    let mut client = start(&f.root());
    for address in ["code:greet.ts#nothing", "code:gone.ts"] {
        let found = definition(&mut client, &f.path(PAGE), address);
        assert!(found.is_null(), "{address}: {found}");
    }
}

#[test]
fn the_hover_shows_the_code_with_remove_applied() {
    let f = project("@snippet: code:greet.ts#greet");
    let mut client = start(&f.root());
    let text = hover(&mut client, &f.path(PAGE), "code:greet.ts#greet");
    // After what it said before.
    assert!(text.contains(", region `greet`, lines 5 to 8"), "{text}");
    assert!(
        text.ends_with(
            "\n\n```ts\nfunction greet(name: string): string {\n  return `Hello, ${name}!`;\n}\n```"
        ),
        "{text}"
    );
    assert!(!text.contains("secret"), "{text}");
}

#[test]
fn the_hover_cuts_long_code() {
    let long: String = (1..=54).map(|n| format!("line {n}\n")).collect();
    let f = project("@snippet {lang=text}: code:long.txt");
    f.write("code/long.txt", &long);
    let mut client = start(&f.root());
    let text = hover(&mut client, &f.path(PAGE), "code:long.txt");
    let shown: String = (1..=20).map(|n| format!("line {n}\n")).collect();
    assert!(
        text.ends_with(&format!("\n\n```text\n{shown}```\n\nand 34 more lines")),
        "{text}"
    );
    assert!(!text.contains("line 21"), "{text}");
}
