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

#[test]
fn this_page_lists_a_memo_again_as_its_file_changes() {
    let section = "---\nabout: guide.md#install\n---\n\nWhy the section.\n";
    let (fixture, mut client) = project(&[("memos/section.md", section)]);
    let ask = |client: &mut Client, needle: &str| {
        let page = client
            .request(
                "ascribe/page",
                json!({ "textDocument": doc(&fixture, "docs/guide.md"), "position": at(GUIDE, needle) }),
            )
            .response_result
            .expect("a result");
        page["memos"]
            .as_array()
            .unwrap()
            .iter()
            .map(|memo| memo["firstLine"].as_str().unwrap().to_owned())
            .collect::<Vec<_>>()
    };
    assert_eq!(ask(&mut client, "Run the"), ["Why the section."]);
    // Opened and edited, before it's saved: the page's snapshot is the same.
    let memo = fixture.path("memos/section.md");
    client.open(&memo, 1, section);
    client.replace(&memo, 2, "---\nabout: guide.md\n---\n\nWhy the page.\n");
    client.settle();
    assert!(ask(&mut client, "Run the").is_empty());
    assert_eq!(ask(&mut client, "The guide"), ["Why the page."]);
    // Closed, it's what's on disk again.
    client.close(&memo);
    client.settle();
    assert_eq!(ask(&mut client, "Run the"), ["Why the section."]);
    // Written on disk.
    fixture.write(
        "memos/section.md",
        "---\nabout: guide.md#install\n---\n\nWhy now.\n",
    );
    client.watched(&[(&memo, FileChangeType::CHANGED)]);
    client.settle();
    assert_eq!(ask(&mut client, "Run the"), ["Why now."]);
}

#[test]
fn a_memo_about_a_term_is_on_its_heading_in_the_lens_and_this_page() {
    const GLOSSARY: &str = "---\ntitle: Glossary\n---\n\nThe words the docs use.\n\n## Region\n@id: area\n@term\n\nWhere a workspace runs.\n";
    let model = format!("{MODEL}\n[glossary]\npages = [\"glossary.md\"]\n");
    let fixture = Fixture::new(
        &model,
        &[
            ("docs/glossary.md", GLOSSARY),
            (
                "memos/term.md",
                "---\nabout: term:area\n---\n\nWhy the term.\n",
            ),
            (
                "memos/quoted.md",
                "---\nabout: term:area\nquote: Where a workspace runs\n---\n\nWhy the definition.\n",
            ),
        ],
    );
    let mut client = Client::start(&fixture.root());
    client.settle();
    let lenses = client
        .request(
            "textDocument/codeLens",
            json!({ "textDocument": doc(&fixture, "docs/glossary.md") }),
        )
        .response_result
        .expect("a result");
    assert_eq!(
        titles(lenses.as_array().unwrap()),
        [(6, "1 memo".to_owned()), (10, "1 memo".to_owned())]
    );
    let page = client
        .request(
            "ascribe/page",
            json!({ "textDocument": doc(&fixture, "docs/glossary.md"), "position": at(GLOSSARY, "Where") }),
        )
        .response_result
        .expect("a result");
    let paths: Vec<&str> = page["memos"]
        .as_array()
        .unwrap()
        .iter()
        .map(|m| m["path"].as_str().unwrap())
        .collect();
    assert_eq!(paths, ["memos/quoted.md", "memos/term.md"]);
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
fn a_quote_yaml_would_read_as_a_number_or_null_is_written_as_text() {
    let page =
        "---\ntitle: Guide\n---\n\n## Values\n@id: values\n\n```json\nnull\n```\n\n1.0\n\ntrue\n";
    let fixture = Fixture::new(MODEL, &[("docs/guide.md", page)]);
    let mut client = Client::start(&fixture.root());
    client.settle();
    for (needle, quote) in [
        ("null\n```", "\"null\""),
        ("1.0", "\"1.0\""),
        ("true", "\"true\""),
    ] {
        let cursor = at(page, needle);
        let (_, text) = created(&add(
            &mut client,
            &fixture,
            "addMemo",
            json!({ "start": cursor, "end": cursor }),
        ));
        assert!(text.contains(&format!("\nquote: {quote}\n")), "{text}");
        // And it's a memo file ascribe check reads as one, placed.
        let memo = ascribe_resolve::MemoFile::parse(ascribe_core::RelPath::root(), text.clone());
        assert!(memo.problems.is_empty(), "{text}: {:?}", memo.problems);
    }
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

/// The new text of each edit a rename's answer makes to a memo file, with
/// the character it starts at.
fn memo_edits(result: &Value, fixture: &Fixture, rel: &str) -> Vec<(u64, String)> {
    let uri = support::uri(&fixture.path(rel));
    result["changes"][uri.as_str()]
        .as_array()
        .map(|edits| {
            edits
                .iter()
                .map(|e| {
                    (
                        e["range"]["start"]["character"].as_u64().unwrap(),
                        e["newText"].as_str().unwrap().to_owned(),
                    )
                })
                .collect()
        })
        .unwrap_or_default()
}

const RENAMED: &[(&str, &str)] = &[
    (
        "memos/section.md",
        "---\nabout: guide.md#install\n---\n\nWhy the section.\n",
    ),
    (
        ".ascribe/memos/mine.md",
        "---\nabout: \"guide.md#install\"\nquote: Run the installer\n---\n\nMine.\n",
    ),
    (
        "memos/page.md",
        "---\nabout: guide.md\n---\n\nWhy the page.\n",
    ),
    (
        "memos/upgrade.md",
        "---\nabout: guide.md#upgrade\n---\n\nWhy upgrade.\n",
    ),
    (
        "memos/phrase.md",
        "---\nabout: phrase:product\n---\n\nWhy.\n",
    ),
];

#[test]
fn renaming_an_id_rewrites_the_about_of_the_memos_in_both_folders() {
    let (fixture, mut client) = project(RENAMED);
    let page = fixture.path("docs/guide.md");
    client.open(&page, 1, GUIDE);
    let result = client
        .request(
            "textDocument/rename",
            json!({
                "textDocument": doc(&fixture, "docs/guide.md"),
                "position": at(GUIDE, "install\n\nRun"),
                "newName": "setup",
            }),
        )
        .response_result
        .expect("a rename");
    assert_eq!(
        memo_edits(&result, &fixture, "memos/section.md"),
        [(7, "guide.md#setup".to_owned())]
    );
    // Inside its quotes, which stay.
    assert_eq!(
        memo_edits(&result, &fixture, ".ascribe/memos/mine.md"),
        [(8, "guide.md#setup".to_owned())]
    );
    for untouched in ["memos/page.md", "memos/upgrade.md", "memos/phrase.md"] {
        assert!(
            memo_edits(&result, &fixture, untouched).is_empty(),
            "{untouched}"
        );
    }
    // An id made from the heading's text follows the text.
    let result = client
        .request(
            "textDocument/rename",
            json!({
                "textDocument": doc(&fixture, "docs/guide.md"),
                "position": at(GUIDE, "Upgrade\n\nUpgrade"),
                "newName": "Upgrading",
            }),
        )
        .response_result
        .expect("a rename");
    assert_eq!(
        memo_edits(&result, &fixture, "memos/upgrade.md"),
        [(7, "guide.md#upgrading".to_owned())]
    );
}

#[test]
fn moving_a_page_rewrites_the_about_of_the_memos_keeping_the_id() {
    let (fixture, mut client) = project(RENAMED);
    let result = client
        .request(
            "workspace/willRenameFiles",
            json!({ "files": [{
                "oldUri": support::uri(&fixture.path("docs/guide.md")).as_str(),
                "newUri": support::uri(&fixture.path("docs/guides/guide.md")).as_str(),
            }] }),
        )
        .response_result
        .expect("a rename");
    for (memo, at, about) in [
        ("memos/section.md", 7, "guides/guide.md#install"),
        (".ascribe/memos/mine.md", 8, "guides/guide.md#install"),
        ("memos/page.md", 7, "guides/guide.md"),
        ("memos/upgrade.md", 7, "guides/guide.md#upgrade"),
    ] {
        assert_eq!(
            memo_edits(&result, &fixture, memo),
            [(at, about.to_owned())],
            "{memo}"
        );
    }
    assert!(memo_edits(&result, &fixture, "memos/phrase.md").is_empty());
}

#[test]
fn renaming_a_term_a_phrase_or_a_fragments_heading_rewrites_the_memos_about_it() {
    const GLOSSARY: &str = "---\ntitle: Glossary\n---\n\nUse {product}.\n\n## Region\n@term\n\nWhere a workspace runs.\n";
    const PAGE: &str = "---\ntitle: Page\n---\n\n## Page\n\n@include: _part.md\n";
    const PART: &str = "## Part\n@id: part\n\nText.\n";
    let model = format!("{MODEL}\n[glossary]\npages = [\"glossary.md\"]\n");
    let fixture = Fixture::new(
        &model,
        &[
            ("docs/glossary.md", GLOSSARY),
            ("docs/page.md", PAGE),
            ("docs/_part.md", PART),
            ("memos/term.md", "---\nabout: term:region\n---\n\nWhy.\n"),
            (
                "memos/phrase.md",
                "---\nabout: phrase:product\n---\n\nWhy.\n",
            ),
            (
                "memos/part.md",
                "---\nabout:\n  page.md#part\n---\n\nWhy.\n",
            ),
        ],
    );
    let mut client = Client::start(&fixture.root());
    client.settle();
    let mut rename = |rel: &str, text: &str, needle: &str, new: &str| {
        client.open(&fixture.path(rel), 1, text);
        client
            .request(
                "textDocument/rename",
                json!({
                    "textDocument": doc(&fixture, rel),
                    "position": at(text, needle),
                    "newName": new,
                }),
            )
            .response_result
            .expect("a rename")
    };
    // Rewording a term's heading with no `@id` changes its id.
    let result = rename("docs/glossary.md", GLOSSARY, "Region", "Area");
    assert_eq!(
        memo_edits(&result, &fixture, "memos/term.md"),
        [(7, "term:area".to_owned())]
    );
    let result = rename("docs/glossary.md", GLOSSARY, "product}", "brand");
    assert_eq!(
        memo_edits(&result, &fixture, "memos/phrase.md"),
        [(7, "phrase:brand".to_owned())]
    );
    // On the line under `about:`.
    let result = rename("docs/_part.md", PART, "part\n", "piece");
    assert_eq!(
        memo_edits(&result, &fixture, "memos/part.md"),
        [(2, "page.md#piece".to_owned())]
    );
}
