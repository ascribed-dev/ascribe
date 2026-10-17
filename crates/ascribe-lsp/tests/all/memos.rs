//! Memo files in the editor (SPEC §7.7), over an in-memory connection: the
//! lens on what memos are about, their problems on the memo file as it
//! changes, completion of `about` and its link, This page's memos, and
//! `addMemo`'s edit.

#![allow(clippy::expect_used, clippy::panic, clippy::unwrap_used)]

use crate::support::{self, Client, Fixture};
use lsp_types::FileChangeType;
use serde_json::{Value, json};

const MODEL: &str = r#"spec = "0.1"

[project]
content-root = "docs"

[phrases]
product = "Quill"
"#;

const GUIDE: &str = "---\ntitle: Guide\n---\n\nThe guide covers setup.\n\n## Install\n@id: install\n\nRun the installer\nas an administrator.\n\n- Restart the agent.\n\n## Upgrade\n\nUpgrade it.\n";

fn project(memos: &[(&str, &str)]) -> (Fixture, Client) {
    let mut files = vec![("docs/guide.md", GUIDE)];
    files.extend_from_slice(memos);
    let fixture = Fixture::new(MODEL, &files);
    let mut client = Client::start(&fixture.root());
    client.settle();
    (fixture, client)
}

fn doc(fixture: &Fixture, rel: &str) -> Value {
    json!({ "uri": support::uri(&fixture.path(rel)).as_str() })
}

/// The position of the start of the first `needle` in `text`.
fn at(text: &str, needle: &str) -> Value {
    let i = text.find(needle).unwrap_or_else(|| panic!("no {needle:?}"));
    let before = &text[..i];
    let line = before.matches('\n').count();
    let start = before.rfind('\n').map_or(0, |i| i + 1);
    json!({ "line": line, "character": text[start..i].encode_utf16().count() })
}

fn lenses(client: &mut Client, fixture: &Fixture) -> Vec<Value> {
    client
        .request(
            "textDocument/codeLens",
            json!({ "textDocument": doc(fixture, "docs/guide.md") }),
        )
        .response_result
        .expect("a result")
        .as_array()
        .cloned()
        .unwrap_or_default()
}

/// Each lens as its line and title.
fn titles(lenses: &[Value]) -> Vec<(u64, String)> {
    lenses
        .iter()
        .map(|l| {
            (
                l["range"]["start"]["line"].as_u64().unwrap(),
                l["command"]["title"].as_str().unwrap().to_owned(),
            )
        })
        .collect()
}

#[test]
fn a_lens_counts_the_memos_on_a_heading_a_block_and_the_page() {
    let (fixture, mut client) = project(&[
        (
            "memos/section.md",
            "---\nabout: guide.md#install\n---\n\nWhy the section.\n",
        ),
        (
            "memos/block.md",
            "---\nabout: guide.md#install\nquote: Run the installer as\n---\n\nWhy the block.\n",
        ),
        (
            "memos/gone.md",
            "---\nabout: guide.md#install\nquote: Not in the page\n---\n\nOutdated.\n",
        ),
        (
            "memos/page.md",
            "---\nabout: guide.md\n---\n\nWhy the page.\n",
        ),
        (
            ".ascribe/memos/mine.md",
            "---\nabout: guide.md\n---\n\nMine.\n",
        ),
        (
            "memos/other.md",
            "---\nabout: nowhere.md\n---\n\nDetached.\n",
        ),
    ]);
    let lenses = lenses(&mut client, &fixture);
    assert_eq!(
        titles(&lenses),
        [
            (0, "2 memos".to_owned()),
            (6, "2 memos, 1 outdated".to_owned()),
            (9, "1 memo".to_owned()),
        ]
    );
    // One memo opens it, at its text.
    let one = &lenses[2]["command"];
    assert_eq!(one["command"], "ascribe.openFile");
    assert_eq!(
        one["arguments"][0],
        support::uri(&fixture.path("memos/block.md")).as_str()
    );
    assert_eq!(
        one["arguments"][1]["start"],
        json!({ "line": 5, "character": 0 })
    );
    // Several are listed to pick one, the committed ones first.
    let several = &lenses[0]["command"];
    assert_eq!(several["command"], "ascribe.showMemos");
    let listed: Vec<(&str, bool, bool)> = several["arguments"][0]
        .as_array()
        .unwrap()
        .iter()
        .map(|m| {
            (
                m["path"].as_str().unwrap(),
                m["personal"].as_bool().unwrap(),
                m["outdated"].as_bool().unwrap(),
            )
        })
        .collect();
    assert_eq!(
        listed,
        [
            ("memos/page.md", false, false),
            (".ascribe/memos/mine.md", true, false)
        ]
    );
    assert_eq!(several["arguments"][0][0]["firstLine"], "Why the page.");
}

#[test]
fn a_memo_files_problems_follow_its_text_and_a_personal_memo_has_them_too() {
    let (fixture, mut client) = project(&[
        ("memos/a.md", "---\nabout: guide.md#nope\n---\n\nWhy.\n"),
        (
            ".ascribe/memos/b.md",
            "---\nabout: guide.md\nauthor: me\n---\n\nMine.\n",
        ),
    ]);
    let team = fixture.path("memos/a.md");
    let personal = fixture.path(".ascribe/memos/b.md");
    assert_eq!(client.codes(&team), ["memo-detached"]);
    assert_eq!(client.codes(&personal), ["memo-invalid"]);
    // Fixed in the editor, before it's saved.
    client.open(&team, 1, "---\nabout: guide.md#nope\n---\n\nWhy.\n");
    client.replace(&team, 2, "---\nabout: guide.md#install\n---\n\nWhy.\n");
    client.settle();
    assert!(client.codes(&team).is_empty());
    // A new memo file on disk is checked.
    fixture.write("memos/c.md", "---\nabout: phrase:produkt\n---\n\nWhy.\n");
    client.watched(&[(&fixture.path("memos/c.md"), FileChangeType::CREATED)]);
    client.settle();
    assert_eq!(client.codes(&fixture.path("memos/c.md")), ["memo-detached"]);
}

#[test]
fn about_completes_targets_and_links_to_what_it_names() {
    let text = "---\nabout: guide.md#install\n---\n\nWhy.\n";
    let (fixture, mut client) = project(&[("memos/a.md", text)]);
    let memo = fixture.path("memos/a.md");
    client.open(&memo, 1, text);
    let items = client
        .request(
            "textDocument/completion",
            json!({ "textDocument": doc(&fixture, "memos/a.md"), "position": at(text, "guide.md#") }),
        )
        .response_result
        .expect("a result");
    let labels: Vec<&str> = items
        .as_array()
        .unwrap()
        .iter()
        .map(|i| i["label"].as_str().unwrap())
        .collect();
    for want in [
        "guide.md",
        "guide.md#install",
        "guide.md#upgrade",
        "phrase:product",
    ] {
        assert!(labels.contains(&want), "{want} in {labels:?}");
    }
    // Nothing is offered in the memo's text.
    let none = client
        .request(
            "textDocument/completion",
            json!({ "textDocument": doc(&fixture, "memos/a.md"), "position": at(text, "Why") }),
        )
        .response_result
        .expect("a result");
    assert_eq!(none, json!([]));
    let links = client
        .request(
            "textDocument/documentLink",
            json!({ "textDocument": doc(&fixture, "memos/a.md") }),
        )
        .response_result
        .expect("a result");
    let target = links[0]["target"].as_str().unwrap();
    assert!(target.ends_with("docs/guide.md#L7"), "{target}");
    assert_eq!(links[0]["range"]["start"], at(text, "guide.md#"));
}

#[test]
fn this_page_lists_the_memos_about_the_heading_at_the_cursor() {
    let (fixture, mut client) = project(&[
        (
            "memos/section.md",
            "---\nabout: guide.md#install\n---\n\nWhy the section.\n",
        ),
        (
            "memos/page.md",
            "---\nabout: guide.md\n---\n\nWhy the page.\n",
        ),
    ]);
    let ask = |client: &mut Client, needle: &str| {
        client
            .request(
                "ascribe/page",
                json!({ "textDocument": doc(&fixture, "docs/guide.md"), "position": at(GUIDE, needle) }),
            )
            .response_result
            .expect("a result")
    };
    let page = ask(&mut client, "Run the");
    assert_eq!(page["memos"][0]["path"], "memos/section.md");
    assert_eq!(page["memos"][0]["firstLine"], "Why the section.");
    assert_eq!(page["memos"][0]["state"], "placed");
    assert_eq!(page["memos"].as_array().unwrap().len(), 1);
    let top = ask(&mut client, "The guide");
    assert_eq!(top["memos"][0]["path"], "memos/page.md");
}

fn add(client: &mut Client, fixture: &Fixture, action: &str, range: Value) -> Value {
    client
        .request(
            "ascribe/edit",
            json!({
                "textDocument": doc(fixture, "docs/guide.md"),
                "range": range,
                "action": action,
            }),
        )
        .response_result
        .expect("a result")
}

/// The file an edit creates, and the text it writes.
fn created(result: &Value) -> (String, String) {
    let changes = result["edit"]["documentChanges"]
        .as_array()
        .expect("changes");
    assert_eq!(changes[0]["kind"], "create");
    assert_eq!(changes[0]["options"]["overwrite"], false);
    (
        changes[0]["uri"].as_str().unwrap().to_owned(),
        changes[1]["edits"][0]["newText"]
            .as_str()
            .unwrap()
            .to_owned(),
    )
}

#[test]
fn add_memo_writes_the_section_and_the_block_at_the_cursor() {
    let (fixture, mut client) = project(&[("memos/run-the-installer.md", "Taken.\n")]);
    let cursor = at(GUIDE, "the installer");
    let result = add(
        &mut client,
        &fixture,
        "addMemo",
        json!({ "start": cursor, "end": cursor }),
    );
    let (uri, text) = created(&result);
    assert!(uri.ends_with("/memos/run-the-installer-2.md"), "{uri}");
    assert_eq!(
        text,
        "---\nabout: guide.md#install\nquote: \"Run the installer\"\n---\n\n"
    );
    assert_eq!(
        result["select"]["start"],
        json!({ "line": 5, "character": 0 })
    );
    // A selection is the quote; a personal memo goes in .ascribe/memos/.
    let result = add(
        &mut client,
        &fixture,
        "addPersonalMemo",
        json!({ "start": at(GUIDE, "an administrator"), "end": at(GUIDE, ".\n\n- Restart") }),
    );
    let (uri, text) = created(&result);
    assert!(
        uri.ends_with("/.ascribe/memos/an-administrator.md"),
        "{uri}"
    );
    assert!(text.contains("quote: \"an administrator\"\n"), "{text}");
    // Before the first heading, the memo is about the page; on a heading,
    // its section, with no quote.
    let top = at(GUIDE, "The guide");
    let (_, text) = created(&add(
        &mut client,
        &fixture,
        "addMemo",
        json!({ "start": top, "end": top }),
    ));
    assert!(text.starts_with("---\nabout: guide.md\nquote:"), "{text}");
    let heading = at(GUIDE, "Upgrade\n\nUpgrade");
    let (uri, text) = created(&add(
        &mut client,
        &fixture,
        "addMemo",
        json!({ "start": heading, "end": heading }),
    ));
    assert!(uri.ends_with("/memos/upgrade.md"), "{uri}");
    assert_eq!(text, "---\nabout: guide.md#upgrade\n---\n\n");
}

#[test]
fn a_team_memo_isnt_added_when_memos_is_in_the_content_root() {
    let fixture = Fixture::new(
        "spec = \"0.1\"\n\n[project]\ncontent-root = \".\"\noutput-dir = \"../build\"\n",
        &[("guide.md", "---\ntitle: Guide\n---\n\nText.\n")],
    );
    let mut client = Client::start(&fixture.root());
    client.settle();
    let cursor = json!({ "line": 4, "character": 0 });
    let result = client
        .request(
            "ascribe/edit",
            json!({
                "textDocument": doc(&fixture, "guide.md"),
                "range": { "start": cursor, "end": cursor },
                "action": "addMemo",
            }),
        )
        .response_result
        .expect("a result");
    assert!(
        result["error"].as_str().unwrap().contains("content root"),
        "{result}"
    );
}
