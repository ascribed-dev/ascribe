//! Starting a page from the editor, over an in-memory connection: the types
//! `ascribe/targets` lists for a folder, `newPage`'s edit, which creates the
//! page with its text, and the places `nextPlaceholder` steps through.

#![allow(clippy::expect_used, clippy::panic, clippy::unwrap_used)]

use crate::support::{self, Client, Fixture};
use serde_json::{Value, json};

const MODEL: &str = r#"spec = "0.1"

[project]
content-root = "docs"

[types.page.frontmatter]
title = "string"
owner = "string"

[types.how-to]
template = "templates/how-to.md"
purpose = "One task, step by step."

[types.how-to.frontmatter]
title = "string"
description = "string"
"#;

/// The how-to template of the set, as eject writes it.
fn how_to() -> &'static str {
    ascribe_templates::FILES
        .iter()
        .find(|f| f.name == "how-to.md")
        .expect("the set has a how-to")
        .text
}

fn project() -> (Fixture, Client) {
    let fixture = Fixture::new(
        MODEL,
        &[
            ("templates/how-to.md", how_to()),
            ("docs/index.md", "---\ntype: page\ntitle: Home\nowner: me\n---\nHome.\n"),
            (
                "docs/how-to/index.md",
                "---\ntype: how-to\ntitle: How-tos\ndescription: All of them.\n---\n",
            ),
        ],
    );
    let client = Client::start(&fixture.root());
    (fixture, client)
}

fn model_uri(fixture: &Fixture) -> String {
    support::uri(&fixture.path("ascribe.toml"))
        .as_str()
        .to_owned()
}

fn folder_uri(fixture: &Fixture, rel: &str) -> String {
    support::uri(&fixture.path(rel)).as_str().to_owned()
}

/// `ascribe/edit`'s answer for `action` asked through `uri`.
fn edit(client: &mut Client, uri: &str, at: (u32, u32), action: &str, args: Value) -> Value {
    let position = json!({ "line": at.0, "character": at.1 });
    client
        .request(
            "ascribe/edit",
            json!({
                "textDocument": { "uri": uri },
                "range": { "start": position, "end": position },
                "action": action,
                "args": args,
            }),
        )
        .response_result
        .expect("the request succeeds")
}

/// The text `range` covers in `text`, its columns counted in UTF-16.
fn covered(text: &str, range: &Value) -> String {
    let offset = |at: &Value| {
        let line = at["line"].as_u64().unwrap() as usize;
        let character = at["character"].as_u64().unwrap() as usize;
        let start: usize = text.split_inclusive('\n').take(line).map(str::len).sum();
        let rest = &text[start..];
        let mut units = 0;
        for (i, c) in rest.char_indices() {
            if units >= character {
                return start + i;
            }
            units += c.len_utf16();
        }
        text.len()
    };
    text[offset(&range["start"])..offset(&range["end"])].to_owned()
}

#[test]
fn a_folder_lists_the_types_a_page_started_there_would_be_of() {
    let (fixture, mut client) = project();
    let types = |client: &mut Client, folder: Option<String>| {
        let result = client
            .request(
                "ascribe/targets",
                json!({
                    "textDocument": { "uri": model_uri(&fixture) },
                    "kinds": ["types"],
                    "folder": folder,
                }),
            )
            .response_result
            .expect("the request succeeds");
        result["types"].clone()
    };
    let every = json!([
        { "name": "page", "purpose": null, "template": false },
        { "name": "how-to", "purpose": "One task, step by step.", "template": true },
    ]);
    assert_eq!(types(&mut client, None), every);
    // A page names its type, so any folder of pages takes every type.
    assert_eq!(
        types(&mut client, Some(folder_uri(&fixture, "docs/how-to"))),
        every
    );
    assert_eq!(types(&mut client, Some(folder_uri(&fixture, "docs"))), every);
    let fragments = types(&mut client, Some(folder_uri(&fixture, "docs/_parts")));
    assert_eq!(fragments, json!([]));
    let outside = types(&mut client, Some(folder_uri(&fixture, "templates")));
    assert_eq!(outside, json!([]));
}

#[test]
fn a_new_page_is_created_with_its_text_and_the_first_placeholder_selected() {
    let (fixture, mut client) = project();
    let result = edit(
        &mut client,
        &model_uri(&fixture),
        (0, 0),
        "newPage",
        json!({
            "type": "how-to",
            "title": "Rotate keys",
            "folder": folder_uri(&fixture, "docs/how-to"),
        }),
    );
    let page = folder_uri(&fixture, "docs/how-to/rotate-keys.md");
    let changes = &result["edit"]["documentChanges"];
    assert_eq!(changes[0]["kind"], "create");
    assert_eq!(changes[0]["uri"], page.as_str());
    assert_eq!(changes[0]["options"]["overwrite"], false);
    assert_eq!(changes[1]["textDocument"]["uri"], page.as_str());
    let edits = changes[1]["edits"].as_array().expect("the page's edits");
    assert_eq!(edits.len(), 1);
    let text = edits[0]["newText"].as_str().expect("the page's text");
    assert!(
        text.starts_with("---\ntype: how-to\ntitle: \"Rotate keys\"\n"),
        "{text}"
    );
    assert_eq!(
        text.replacen("title: \"Rotate keys\"", "title: \"Title\"", 1),
        how_to(),
        "only the title changes"
    );
    assert_eq!(
        covered(text, &result["select"]),
        "{One sentence that says what a reader can do once they've followed this page.}"
    );
    assert!(result["edit"].get("changes").is_none());
    // Nothing is written: the edit is the client's to apply.
    assert!(!fixture.path("docs/how-to/rotate-keys.md").exists());
}

#[test]
fn without_a_folder_the_page_goes_in_the_content_root() {
    let (fixture, mut client) = project();
    let result = edit(
        &mut client,
        &model_uri(&fixture),
        (0, 0),
        "newPage",
        json!({ "type": "how-to", "title": "Rotate keys" }),
    );
    assert_eq!(
        result["edit"]["documentChanges"][0]["uri"],
        folder_uri(&fixture, "docs/rotate-keys.md").as_str()
    );
}

#[test]
fn a_type_without_a_template_starts_with_its_frontmatter_and_nothing_selected() {
    let (fixture, mut client) = project();
    let result = edit(
        &mut client,
        &model_uri(&fixture),
        (0, 0),
        "newPage",
        json!({ "type": "page", "title": "Notes", "folder": folder_uri(&fixture, "docs") }),
    );
    let changes = &result["edit"]["documentChanges"];
    assert_eq!(
        changes[0]["uri"],
        folder_uri(&fixture, "docs/notes.md").as_str()
    );
    assert_eq!(
        changes[1]["edits"][0]["newText"],
        "---\ntype: page\ntitle: \"Notes\"\nowner:\n---\n"
    );
    assert_eq!(result["select"], Value::Null);
}

#[test]
fn a_page_is_never_started_over_a_file_or_in_a_fragment_s_folder() {
    let (fixture, mut client) = project();
    let model = model_uri(&fixture);
    let error = |client: &mut Client, args: Value| {
        let result = edit(client, &model, (0, 0), "newPage", args);
        result["error"].as_str().expect("an error").to_owned()
    };
    let taken = error(
        &mut client,
        json!({ "type": "how-to", "title": "Index", "folder": folder_uri(&fixture, "docs/how-to") }),
    );
    assert!(
        taken.contains("`how-to/index.md` is there already"),
        "{taken}"
    );
    assert!(taken.starts_with('`') && taken.ends_with('.'), "{taken}");
    let elsewhere = error(
        &mut client,
        json!({ "type": "how-to", "title": "Keys", "folder": folder_uri(&fixture, "docs/_parts") }),
    );
    assert!(
        elsewhere.contains("wouldn't be a page of type `how-to`: it's a fragment's path"),
        "{elsewhere}"
    );
    let outside = error(
        &mut client,
        json!({ "type": "how-to", "title": "Keys", "folder": folder_uri(&fixture, "templates") }),
    );
    assert_eq!(outside, "The folder isn't in the project's content root.");
    let unknown = error(&mut client, json!({ "type": "guide", "title": "Keys" }));
    assert!(
        unknown.starts_with("The content model has no page type `guide`"),
        "{unknown}"
    );
    let untitled = error(&mut client, json!({ "type": "how-to", "title": " " }));
    assert_eq!(untitled, "`title` is missing.");
}

#[test]
fn next_placeholder_steps_through_what_the_template_left_and_comes_round() {
    let (fixture, mut client) = project();
    let text = how_to().replacen("title: \"Title\"", "title: \"Rotate keys\"", 1);
    let path = fixture.path("docs/how-to/rotate-keys.md");
    fixture.write("docs/how-to/rotate-keys.md", &text);
    client.open(&path, 1, &text);
    let uri = support::uri(&path).as_str().to_owned();
    let mut at = (0, 0);
    let mut seen = Vec::new();
    loop {
        let result = edit(&mut client, &uri, at, "nextPlaceholder", json!({}));
        assert_eq!(result["edit"]["changes"], json!({}), "nothing changes");
        let select = &result["select"];
        let place = covered(&text, select);
        if seen.first() == Some(&place) {
            break;
        }
        assert!(seen.len() < 40, "it never came round: {seen:#?}");
        seen.push(place);
        let end = &select["end"];
        at = (
            end["line"].as_u64().unwrap() as u32,
            end["character"].as_u64().unwrap() as u32,
        );
    }
    assert_eq!(
        seen[0],
        "{One sentence that says what a reader can do once they've followed this page.}"
    );
    assert_eq!(seen[1], "\\{insert a brief description of the task}");
    // The one that runs across a paragraph and a list is one place, whole.
    let last = seen.last().expect("places");
    assert!(last.starts_with("\\{Include references"), "{last}");
    assert!(last.ends_with("* Troubleshooting link}"), "{last}");
    // The same placeholder twice is a place each time.
    let twice = seen
        .iter()
        .filter(|p| *p == "\\{insert a brief description of the task}")
        .count();
    assert_eq!(twice, 2);
}

#[test]
fn next_placeholder_says_when_there_is_nothing_to_fill_in() {
    let (fixture, mut client) = project();
    let page = |client: &mut Client, rel: &str, text: &str| {
        let path = fixture.path(rel);
        fixture.write(rel, text);
        client.open(&path, 1, text);
        let uri = support::uri(&path).as_str().to_owned();
        let result = edit(client, &uri, (0, 0), "nextPlaceholder", json!({}));
        result["error"].as_str().expect("an error").to_owned()
    };
    assert_eq!(
        page(
            &mut client,
            "docs/how-to/done.md",
            "---\ntype: how-to\ntitle: Done\ndescription: All done.\n---\n\nFilled in.\n"
        ),
        "Nothing the template left to fill in is on the page."
    );
    assert_eq!(
        page(
            &mut client,
            "docs/notes.md",
            "---\ntype: page\ntitle: Notes\nowner: me\n---\n"
        ),
        "This page's type has no template, so nothing on it is left to fill in."
    );
}
