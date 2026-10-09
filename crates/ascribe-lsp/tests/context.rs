//! `ascribe/context` and `ascribe/targets`, over an in-memory connection.

#![allow(clippy::expect_used, clippy::panic, clippy::unwrap_used)]

mod support;

use std::path::Path;

use serde_json::{Value, json};
use support::{Client, Fixture, MODEL};

const MODEL_FULL: &str = r#"spec = "0.1"

[project]
content-root = "docs"

[dimensions.pm]
label = "Package manager"
values = ["npm", "pnpm"]
labels = { pnpm = "PNPM" }

[dimensions.deployment]
values = ["cloud", "self-managed"]
versionless = ["cloud"]

[phrases]
product = "Quill"

[features.sso]
name = "Single sign-on"
available = "cloud"

[notes.extra]
label = "Extra"

[images.attributes]
width = "number?"

[sources.code]
path = "code"
include = ["**/*.py"]

[widgets.quill-aside]
description = "An aside."
forms = ["line", "container"]
primary = "text?"
binding = "block"

[widgets.quill-compare]
forms = ["container"]
title = "required"
groupable = true
attributes = { highlight = { type = "boolean", default = false } }

[builds.site]
variants = "switch"
availability = "badge"

[builds.cloud]
variants = { deployment = "cloud" }
availability = { filter = "cloud" }

[editor]
build = "cloud"
"#;

const PAGE: &str = "---
title: Install
available: cloud
variant:
  pm: npm
---

# Install {product}

Intro with a [link to the guide](guide/setup.md) and ![The logo](img/logo.png){width=600} and {product}.

## Steps
@available: cloud

@steps
1. First step.

   @variant {pm=npm}:
   Run npm.
   @variant {pm=pnpm}:
   Run pnpm.
   @end
2. Second.

@note {type=tip}
A bound paragraph.

@note: A line note.

@note {type=extra}:
Inside a container.

@end

.More
@details:
Hidden.
@end

@include: _frag.md

@snippet: code:app.py#main

| A | B |
|---|---|
| x {available=cloud} | y |

```js
const a = 1;

const b = 2;
```

Para one.

Para two.

@quill-aside: An aside.

.Before
@quill-compare {highlight=true}:
Old.
.After
@quill-compare:
New.
@end
";

const APP: &str = "# :snippet-start: main\nprint('hi')\n# :snippet-end:\n";

fn project() -> Fixture {
    Fixture::new(
        MODEL_FULL,
        &[
            ("docs/index.md", PAGE),
            (
                "docs/guide/setup.md",
                "---\ntitle: Setup\n---\n# Set up\n\n@include: ../_frag.md\n\n## Configure it\n",
            ),
            ("docs/_frag.md", "## From a fragment\n\nShared.\n"),
            ("docs/_plain.md", "Just text.\n"),
            ("docs/img/logo.png", "png"),
            ("docs/img/.hidden/x.png", "png"),
            ("docs/img/notes.txt", "text"),
            ("docs/node_modules/pkg/y.png", "png"),
            ("code/app.py", APP),
            ("code/README.md", "not included"),
        ],
    )
}

/// The position of the first `needle` in `text`, plus `delta` characters,
/// as a line and a UTF-16 column.
fn pos(text: &str, needle: &str, delta: usize) -> (u32, u32) {
    let at = text
        .find(needle)
        .unwrap_or_else(|| panic!("{needle:?} in the text"));
    let at = text[at..]
        .char_indices()
        .nth(delta)
        .map_or(text.len(), |(i, _)| at + i);
    let before = &text[..at];
    let line = before.matches('\n').count();
    let column = before[before.rfind('\n').map_or(0, |i| i + 1)..]
        .encode_utf16()
        .count();
    (line as u32, column as u32)
}

fn range(a: (u32, u32), b: (u32, u32)) -> Value {
    json!({
        "start": { "line": a.0, "character": a.1 },
        "end": { "line": b.0, "character": b.1 },
    })
}

fn context(client: &mut Client, path: &Path, a: (u32, u32), b: (u32, u32)) -> Value {
    client
        .request(
            "ascribe/context",
            json!({
                "textDocument": { "uri": support::uri(path).as_str() },
                "range": range(a, b),
            }),
        )
        .response_result
        .expect("the request succeeds")
}

fn targets(client: &mut Client, path: &Path, kinds: &[&str]) -> Value {
    client
        .request(
            "ascribe/targets",
            json!({
                "textDocument": { "uri": support::uri(path).as_str() },
                "kinds": kinds,
            }),
        )
        .response_result
        .expect("the request succeeds")
}

/// The kinds in `at`, innermost first.
fn kinds(result: &Value) -> Vec<String> {
    result["at"]
        .as_array()
        .expect("at")
        .iter()
        .map(|n| n["kind"].as_str().expect("a kind").to_owned())
        .collect()
}

/// The node of `kind` in `at`.
fn node<'a>(result: &'a Value, kind: &str) -> &'a Value {
    result["at"]
        .as_array()
        .expect("at")
        .iter()
        .find(|n| n["kind"] == kind)
        .unwrap_or_else(|| panic!("a {kind} in {result}"))
}

fn started(f: &Fixture) -> (Client, std::path::PathBuf) {
    let page = f.path("docs/index.md");
    let mut client = Client::start(&f.root());
    client.settle();
    (client, page)
}

fn at(client: &mut Client, page: &Path, needle: &str, delta: usize) -> Value {
    let p = pos(PAGE, needle, delta);
    context(client, page, p, p)
}

#[test]
fn a_cursor_in_a_variant_arm_in_a_list_item_in_steps_has_the_whole_chain() {
    let f = project();
    let (mut client, page) = started(&f);
    let result = at(&mut client, &page, "Run pnpm.", 2);
    assert_eq!(
        kinds(&result),
        [
            "paragraph",
            "variantGroup",
            "listItem",
            "list",
            "steps",
            "section",
            "section"
        ]
    );
    let group = node(&result, "variantGroup");
    assert_eq!(group["dimension"], "pm");
    assert_eq!(group["arm"], 1);
    let arms = group["arms"].as_array().unwrap();
    assert_eq!(arms[0]["value"], "npm");
    assert_eq!(arms[1]["value"], "pnpm");
    let (line, _) = pos(PAGE, "   @variant {pm=pnpm}:", 0);
    assert_eq!(arms[1]["range"]["start"]["line"], line);
    let list = node(&result, "list");
    assert_eq!(list["ordered"], true);
    assert_eq!(list["steps"], true);
    // The inner section is `## Steps`; the outer, `# Install {product}`.
    let sections: Vec<&Value> = result["at"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|n| n["kind"] == "section")
        .collect();
    assert_eq!(sections[0]["headingId"], "steps");
    assert_eq!(sections[1]["headingId"], "install-quill");
    assert_eq!(result["project"]["editorBuild"], "cloud");
    // The root as the server spells it: on Windows, a drive letter's case
    // may differ from the test's.
    let root = result["project"]["root"].as_str().expect("a root");
    assert_eq!(Path::new(root).file_name(), f.root().file_name());
    assert_eq!(result["selection"], Value::Null);
    assert_eq!(result["insertable"], false);
}

#[test]
fn each_node_kind_is_reported() {
    let f = project();
    let (mut client, page) = started(&f);

    let r = at(&mut client, &page, "Install {product}", 2);
    assert_eq!(kinds(&r), ["heading", "section"]);
    let heading = node(&r, "heading");
    assert_eq!(heading["level"], 1);
    assert_eq!(heading["id"], "install-quill");
    assert_eq!(heading["explicitId"], false);

    let r = at(&mut client, &page, "{product}.", 1);
    assert_eq!(kinds(&r), ["phrase", "paragraph", "section"]);
    assert_eq!(node(&r, "phrase")["key"], "product");
    assert_eq!(node(&r, "phrase")["declared"], true);
    assert_eq!(r["token"]["kind"], "phrase");

    let r = at(&mut client, &page, "link to the guide", 3);
    assert_eq!(kinds(&r), ["link", "paragraph", "section"]);
    assert_eq!(node(&r, "link")["destination"], "guide/setup.md");
    assert_eq!(node(&r, "link")["textEmpty"], false);
    assert_eq!(r["token"]["kind"], "link");

    let r = at(&mut client, &page, "The logo", 1);
    let image = node(&r, "image");
    assert_eq!(image["src"], "img/logo.png");
    assert_eq!(image["alt"], "The logo");
    assert_eq!(
        image["attributes"],
        json!([{ "key": "width", "value": "600" }])
    );
    assert_eq!(r["token"]["kind"], "image");
    assert_eq!(r["token"]["src"], "img/logo.png");

    let r = at(&mut client, &page, "@available: cloud", 13);
    assert_eq!(kinds(&r), ["availability", "section", "section"]);
    assert_eq!(node(&r, "availability")["spec"], "cloud");

    let r = at(&mut client, &page, "A bound paragraph", 0);
    assert_eq!(kinds(&r), ["paragraph", "note", "section", "section"]);
    let note = node(&r, "note");
    assert_eq!(note["type"], "tip");
    assert_eq!(note["form"], "block");
    let (line, _) = pos(PAGE, "@note {type=tip}", 0);
    assert_eq!(note["range"]["start"]["line"], line);

    // On the directive's name: the token is the name.
    let r = at(&mut client, &page, "@note {type=tip}", 2);
    assert_eq!(node(&r, "note")["form"], "block");
    assert_eq!(
        r["token"],
        json!({
            "kind": "directiveName",
            "range": range(pos(PAGE, "@note {type=tip}", 0), pos(PAGE, "@note {type=tip}", 5)),
            "name": "note",
        })
    );
    // On an attribute's value: the token is the attribute.
    let r = at(&mut client, &page, "type=tip", 6);
    assert_eq!(r["token"]["kind"], "attribute");
    assert_eq!(r["token"]["directive"], "note");
    assert_eq!(r["token"]["key"], "type");
    assert_eq!(r["token"]["value"], "tip");

    let r = at(&mut client, &page, "A line note", 0);
    assert_eq!(node(&r, "note")["form"], "line");
    assert_eq!(node(&r, "note")["type"], "note");

    let r = at(&mut client, &page, "Inside a container", 0);
    assert_eq!(kinds(&r), ["paragraph", "note", "section", "section"]);
    assert_eq!(node(&r, "note")["form"], "container");
    assert_eq!(node(&r, "note")["type"], "extra");

    let r = at(&mut client, &page, "Hidden.", 0);
    assert_eq!(kinds(&r), ["paragraph", "details", "section", "section"]);
    assert_eq!(node(&r, "details")["title"], "More");
    assert_eq!(node(&r, "details")["form"], "container");
    let (line, _) = pos(PAGE, ".More", 0);
    assert_eq!(node(&r, "details")["range"]["start"]["line"], line);

    let r = at(&mut client, &page, "_frag.md", 1);
    let include = node(&r, "include");
    assert_eq!(include["path"], "_frag.md");
    assert_eq!(include["section"], Value::Null);
    assert_eq!(r["token"]["kind"], "include");
    assert_eq!(r["token"]["path"], "_frag.md");

    let r = at(&mut client, &page, "code:app.py#main", 1);
    assert_eq!(node(&r, "snippet")["address"], "code:app.py#main");

    let r = at(&mut client, &page, "x {available=cloud}", 0);
    assert_eq!(kinds(&r), ["tableRow", "table", "section", "section"]);
    assert_eq!(node(&r, "tableRow")["header"], false);
    assert_eq!(node(&r, "tableRow")["available"], "cloud");
    let r = at(&mut client, &page, "| A | B |", 2);
    assert_eq!(node(&r, "tableRow")["header"], true);
    assert_eq!(node(&r, "tableRow")["available"], Value::Null);

    let r = at(&mut client, &page, "const a", 2);
    assert_eq!(kinds(&r), ["codeBlock", "section", "section"]);
    assert_eq!(node(&r, "codeBlock")["info"], "js");
    assert_eq!(node(&r, "codeBlock")["fenced"], true);

    let r = at(&mut client, &page, "An aside.", 1);
    let widget = node(&r, "widget");
    assert_eq!(widget["name"], "quill-aside");
    assert_eq!(widget["form"], "line");

    let r = at(&mut client, &page, "Old.", 0);
    assert_eq!(kinds(&r), ["paragraph", "widget", "section", "section"]);
    let widget = node(&r, "widget");
    assert_eq!(widget["name"], "quill-compare");
    assert_eq!(widget["form"], "group");
    assert_eq!(
        widget["attributes"],
        json!([{ "key": "highlight", "value": "true" }])
    );

    let r = at(&mut client, &page, "pm: npm", 5);
    assert_eq!(kinds(&r), ["frontmatter"]);
    let frontmatter = node(&r, "frontmatter");
    assert_eq!(frontmatter["available"]["value"], "cloud");
    assert_eq!(
        frontmatter["available"]["range"],
        range(
            pos(PAGE, "cloud\nvariant", 0),
            pos(PAGE, "cloud\nvariant", 5)
        )
    );
    assert_eq!(frontmatter["variant"]["value"], "\n  pm: npm");
    assert_eq!(
        frontmatter["variant"]["range"]["end"],
        range(pos(PAGE, "npm\n---", 3), pos(PAGE, "npm\n---", 3))["end"]
    );
    assert_eq!(r["insertable"], false);

    assert_eq!(client.shutdown(), ascribe_lsp::Exit::Clean);
}

#[test]
fn a_list_bound_by_no_steps_says_so_and_an_explicit_id_is_marked() {
    let text = "# Top\n@id: top\n\n- one\n- two\n";
    let f = Fixture::new(MODEL, &[("docs/a.md", text)]);
    let page = f.path("docs/a.md");
    let mut client = Client::start(&f.root());
    client.settle();
    let r = context(&mut client, &page, pos(text, "two", 0), pos(text, "two", 0));
    assert_eq!(kinds(&r), ["paragraph", "listItem", "list", "section"]);
    assert_eq!(node(&r, "list")["steps"], false);
    assert_eq!(node(&r, "list")["ordered"], false);
    let r = context(&mut client, &page, (0, 3), (0, 3));
    assert_eq!(node(&r, "heading")["id"], "top");
    assert_eq!(node(&r, "heading")["explicitId"], true);
}

#[test]
fn selections_are_prose_blocks_code_or_mixed() {
    let f = project();
    let (mut client, page) = started(&f);
    let select = |client: &mut Client, from: (&str, usize), to: (&str, usize)| {
        context(
            client,
            &page,
            pos(PAGE, from.0, from.1),
            pos(PAGE, to.0, to.1),
        )
    };

    // Inside one paragraph.
    let r = select(&mut client, ("Para one.", 0), ("Para one.", 4));
    assert_eq!(
        r["selection"],
        json!({ "kind": "prose", "text": "Para", "inline": true })
    );
    // Across two whole paragraphs.
    let r = select(&mut client, ("Para one.", 0), ("Para two.", 9));
    assert_eq!(r["selection"]["kind"], "blocks");
    assert_eq!(r["selection"]["inline"], false);
    assert_eq!(r["selection"]["text"], "Para one.\n\nPara two.");
    // Across parts of two paragraphs.
    let r = select(&mut client, ("Para one.", 5), ("Para two.", 4));
    assert_eq!(r["selection"]["kind"], "mixed");
    assert_eq!(r["selection"]["inline"], false);
    // A directive's own lines with the blocks it holds aren't whole blocks:
    // an arm with its opener,
    let r = select(&mut client, ("@variant {pm=npm}", 0), ("Run npm.", 8));
    assert_eq!(r["selection"]["kind"], "mixed");
    // both arms of a group without its `@end`,
    let r = select(&mut client, ("@variant {pm=npm}", 0), ("Run pnpm.", 9));
    assert_eq!(r["selection"]["kind"], "mixed");
    // part of a container's opener through its content.
    let r = select(
        &mut client,
        ("@note {type=extra}:", 6),
        ("Inside a container.", 19),
    );
    assert_eq!(r["selection"]["kind"], "mixed");
    // The blocks inside an arm or a container are blocks.
    let r = select(&mut client, ("Run npm.", 0), ("Run npm.", 8));
    assert_eq!(r["selection"]["kind"], "prose");
    let r = select(
        &mut client,
        ("Inside a container.", 0),
        ("Inside a container.", 19),
    );
    assert_eq!(r["selection"]["kind"], "prose");
    // A whole group or container is a block.
    let r = select(
        &mut client,
        ("@variant {pm=npm}", 0),
        ("Run pnpm.\n   @end", 18),
    );
    assert_eq!(r["selection"]["kind"], "blocks");
    let r = select(
        &mut client,
        ("@note {type=extra}:", 0),
        ("container.\n\n@end", 16),
    );
    assert_eq!(r["selection"]["kind"], "blocks");
    // Some of a list's items aren't blocks; the whole list is.
    let r = select(&mut client, ("1. First", 0), ("2. Second.", 5));
    assert_eq!(r["selection"]["kind"], "mixed");
    let r = select(&mut client, ("1. First", 0), ("2. Second.", 10));
    assert_eq!(r["selection"]["kind"], "blocks");
    let r = select(&mut client, ("1. First", 0), ("1. First step.", 14));
    assert_eq!(r["selection"]["kind"], "mixed");
    // From a paragraph into the middle of a code block.
    let r = select(&mut client, ("A line note", 2), ("const a", 5));
    assert_eq!(r["selection"]["kind"], "mixed");
    // Inside a code block.
    let r = select(&mut client, ("const a", 0), ("const b", 5));
    assert_eq!(r["selection"]["kind"], "code");
    // A whole code block is a block.
    let r = select(&mut client, ("```js", 0), ("2;\n```", 6));
    assert_eq!(r["selection"]["kind"], "blocks");
    // Inside one heading's text.
    let r = select(
        &mut client,
        ("Install {product}", 0),
        ("Install {product}", 7),
    );
    assert_eq!(r["selection"]["kind"], "prose");
    assert_eq!(r["selection"]["inline"], true);
    // Part of a table.
    let r = select(&mut client, ("| A | B |", 2), ("| A | B |", 6));
    assert_eq!(r["selection"]["kind"], "other");
}

#[test]
fn a_blank_line_between_blocks_is_insertable_and_one_in_code_is_not() {
    let f = project();
    let (mut client, page) = started(&f);
    let r = at(&mut client, &page, "\nPara two.", 0);
    assert_eq!(r["insertable"], true);
    assert_eq!(kinds(&r), ["section", "section"]);
    let r = at(&mut client, &page, "\nconst b", 0);
    assert_eq!(r["insertable"], false);
    assert_eq!(kinds(&r)[0], "codeBlock");
    // Not on a line with text.
    let r = at(&mut client, &page, "Para two.", 0);
    assert_eq!(r["insertable"], false);
}

#[test]
fn unsaved_edits_are_answered() {
    let f = project();
    let (mut client, page) = started(&f);
    let text = "# Draft\n\n@note {type=warning}: Unsaved.\n";
    client.open(&page, 1, PAGE);
    client.replace(&page, 2, text);
    let r = context(
        &mut client,
        &page,
        pos(text, "Unsaved", 1),
        pos(text, "Unsaved", 1),
    );
    assert_eq!(kinds(&r), ["note", "section"]);
    assert_eq!(node(&r, "note")["type"], "warning");
    assert_eq!(node(&r, "section")["headingId"], "draft");
}

#[test]
fn positions_are_utf16_on_a_line_with_non_ascii_text() {
    let text = "Ünïcödé 😀 then {product} here.\n";
    let f = Fixture::new(
        &format!("{MODEL}\n[phrases]\nproduct = \"Quill\"\n"),
        &[("docs/a.md", text)],
    );
    let page = f.path("docs/a.md");
    let mut client = Client::start(&f.root());
    client.settle();
    let p = pos(text, "{product}", 2);
    // The emoji is two UTF-16 units.
    assert_eq!(p, (0, 18));
    let r = context(&mut client, &page, p, p);
    assert_eq!(
        node(&r, "phrase")["range"],
        range(pos(text, "{product}", 0), pos(text, "{product}", 9))
    );
    assert_eq!(node(&r, "phrase")["range"]["start"]["character"], 16);
}

#[test]
fn a_document_outside_the_project_gets_an_empty_answer() {
    let f = Fixture::new(
        MODEL,
        &[
            ("docs/a.md", "Text.\n"),
            ("docs/nested/ascribe.toml", MODEL),
            ("docs/nested/docs/b.md", "# Nested\n\nText.\n"),
        ],
    );
    let nested = f.path("docs/nested/docs/b.md");
    let mut client = Client::start(&f.root());
    client.settle();
    // The outer project's server doesn't answer for the nested project's
    // page.
    let r = context(&mut client, &nested, (0, 2), (0, 2));
    assert_eq!(
        r,
        json!({ "project": null, "at": [], "selection": null, "token": null, "insertable": false })
    );
    assert_eq!(targets(&mut client, &nested, &["pages"]), json!({}));
    let outside = f.path("notes.md");
    assert_eq!(
        context(&mut client, &outside, (0, 0), (0, 0))["at"],
        json!([])
    );
    assert_eq!(client.shutdown(), ascribe_lsp::Exit::Clean);

    // The nested project's own server does.
    let mut client = Client::start(&f.path("docs/nested"));
    client.settle();
    let r = context(&mut client, &nested, (0, 2), (0, 2));
    assert_eq!(kinds(&r), ["heading", "section"]);
    let pages = targets(&mut client, &nested, &["pages"]);
    assert_eq!(pages["pages"][0]["path"], "b.md");
}

#[test]
fn targets_list_only_the_kinds_asked_for() {
    let f = project();
    let (mut client, page) = started(&f);
    let r = targets(&mut client, &page, &["builds", "notes"]);
    let keys: Vec<&String> = r.as_object().unwrap().keys().collect();
    assert_eq!(keys, ["builds", "modelUri", "notes"]);
    assert_eq!(
        r["builds"],
        json!([
            { "name": "site", "editor": false },
            { "name": "cloud", "editor": true },
        ])
    );
    let notes = r["notes"].as_array().unwrap();
    assert_eq!(notes[0], json!({ "type": "note", "label": "Note" }));
    assert_eq!(
        notes.last().unwrap(),
        &json!({ "type": "extra", "label": "Extra" })
    );
    let uri = r["modelUri"].as_str().expect("a URI");
    assert!(
        uri.starts_with("file:///") && uri.ends_with("/ascribe.toml"),
        "{uri}"
    );
}

#[test]
fn targets_write_paths_from_the_requesting_page() {
    let f = project();
    let (mut client, _) = started(&f);
    let from = f.path("docs/guide/setup.md");
    let r = targets(
        &mut client,
        &from,
        &["pages", "headings", "fragments", "images"],
    );
    assert_eq!(
        r["pages"],
        json!([
            { "path": "guide/setup.md", "title": "Setup", "type": "page", "link": "setup.md" },
            { "path": "index.md", "title": "Install", "type": "page", "link": "../index.md" },
        ])
    );
    let headings = r["headings"].as_array().unwrap();
    // A page's own headings link with `#id` alone, a fragment's included
    // ones as the page's.
    assert_eq!(
        headings[0],
        json!({ "page": "guide/setup.md", "text": "Set up", "id": "set-up", "level": 1, "link": "#set-up" })
    );
    assert_eq!(
        headings[1],
        json!({ "page": "guide/setup.md", "text": "From a fragment", "id": "from-a-fragment", "level": 2, "link": "#from-a-fragment" })
    );
    assert!(headings.contains(&json!({
        "page": "index.md", "text": "Steps", "id": "steps", "level": 2, "link": "../index.md#steps"
    })));
    assert_eq!(
        r["fragments"],
        json!([
            { "path": "_frag.md", "include": "../_frag.md", "startsWithHeading": true },
            { "path": "_plain.md", "include": "../_plain.md", "startsWithHeading": false },
        ])
    );
    // Only images, and none in a hidden folder or `node_modules`.
    assert_eq!(
        r["images"],
        json!([{ "path": "docs/img/logo.png", "link": "../img/logo.png" }])
    );
}

#[test]
fn targets_list_the_content_model() {
    let f = project();
    let (mut client, page) = started(&f);
    let r = targets(
        &mut client,
        &page,
        &["phrases", "dimensions", "widgets", "features", "snippets"],
    );
    let model = std::fs::read_to_string(f.path("ascribe.toml")).unwrap();
    let span = |needle: &str, len: usize| range(pos(&model, needle, 0), pos(&model, needle, len));
    assert_eq!(
        r["phrases"],
        json!([{ "key": "product", "value": "Quill", "range": span("product =", 7) }])
    );
    let pm = &r["dimensions"][0];
    assert_eq!(pm["name"], "pm");
    assert_eq!(pm["label"], "Package manager");
    assert_eq!(
        pm["values"],
        json!([
            { "value": "npm", "label": "npm", "versionless": false, "range": span("npm\", \"pnpm", 3) },
            { "value": "pnpm", "label": "PNPM", "versionless": false, "range": span("pnpm\"]", 4) },
        ])
    );
    assert_eq!(r["dimensions"][1]["values"][0]["versionless"], true);
    assert_eq!(
        r["features"],
        json!([{
            "key": "sso",
            "name": "Single sign-on",
            "availability": "cloud",
            "range": span("[features.sso]", 14),
        }])
    );
    assert_eq!(
        r["widgets"][0],
        json!({
            "name": "quill-aside",
            "description": "An aside.",
            "line": true,
            "container": true,
            "groupable": false,
            "primary": "text",
            "binding": "block",
            "attributes": [],
        })
    );
    assert_eq!(
        r["widgets"][1]["attributes"],
        json!([{
            "key": "highlight",
            "type": "boolean",
            "values": [],
            "required": false,
            "default": "false",
            "description": null,
        }])
    );
    assert_eq!(
        r["snippets"],
        json!([{
            "name": "code",
            "files": [{
                "path": "app.py",
                "address": "code:app.py",
                "regions": [{ "name": "main", "address": "code:app.py#main" }],
            }],
        }])
    );
}

#[test]
fn targets_include_unsaved_edits() {
    let f = project();
    let (mut client, page) = started(&f);
    client.open(&page, 1, PAGE);
    client.replace(&page, 2, "---\ntitle: Renamed\n---\n# New heading\n");
    let r = targets(&mut client, &page, &["pages", "headings"]);
    assert!(r["pages"].as_array().unwrap().contains(
        &json!({ "path": "index.md", "title": "Renamed", "type": "page", "link": "index.md" })
    ));
    assert!(
        r["headings"]
            .as_array()
            .unwrap()
            .iter()
            .any(|h| h["id"] == "new-heading" && h["link"] == "#new-heading")
    );
}
