//! The lines of a versioned build in the editor: the builds a client can
//! choose, the build lens and the preview of a line, the diagnostics of every
//! line, and the hovers on a mark's version and on `replaces`. Over a copy of
//! `examples/quill`, whose `site` build has the lines 3.5 and 3.4.

#![allow(clippy::expect_used, clippy::panic, clippy::unwrap_used)]

use std::path::Path;

use crate::support::{self, Client, Fixture, MODEL};
use serde_json::{Value, json};

fn copy_dir(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).expect("mkdir");
    for entry in std::fs::read_dir(from).expect("read_dir").flatten() {
        let target = to.join(entry.file_name());
        if entry.path().is_dir() {
            copy_dir(&entry.path(), &target);
        } else {
            std::fs::copy(entry.path(), target).expect("copy");
        }
    }
}

fn quill() -> Fixture {
    let source = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/quill");
    let f = Fixture::new(MODEL, &[]);
    copy_dir(&source, &f.root());
    f
}

fn doc(path: &Path) -> Value {
    json!({ "uri": support::uri(path).as_str() })
}

fn request(client: &mut Client, method: &str, params: Value) -> Value {
    client
        .request(method, params)
        .response_result
        .expect("the request succeeds")
}

fn hover_text(client: &mut Client, path: &Path, line: u32, character: u32) -> String {
    let hover = request(
        client,
        "textDocument/hover",
        json!({ "textDocument": doc(path), "position": { "line": line, "character": character } }),
    );
    hover["contents"]["value"]
        .as_str()
        .expect("a hover")
        .to_owned()
}

/// Where `needle` first is in `text`, as a line and character.
fn position_of(text: &str, needle: &str) -> (u32, u32) {
    let at = text.find(needle).expect("the text is there");
    let line = text[..at].matches('\n').count();
    let start = text[..at].rfind('\n').map_or(0, |i| i + 1);
    (line as u32, (at - start) as u32)
}

#[test]
fn a_versioned_build_offers_its_lines_the_current_one_first() {
    let f = quill();
    let mut client = Client::start(&f.root());
    let page = f.path("docs/keys.md");
    let targets = request(
        &mut client,
        "ascribe/targets",
        json!({ "textDocument": doc(&page), "kinds": ["builds"] }),
    );
    let site = &targets["builds"][0];
    assert_eq!(site["name"], "site");
    assert_eq!(
        site["lines"],
        json!([
            { "name": "site", "line": "3.5", "current": true },
            { "name": "site@3.4", "line": "3.4", "current": false },
        ])
    );
    // A build that isn't versioned has no lines.
    assert!(targets["builds"][1].get("lines").is_none());
    // The preview's picker offers the same.
    let preview = request(
        &mut client,
        "ascribe/preview",
        json!({ "textDocument": doc(&page) }),
    );
    assert_eq!(preview["builds"][0]["lines"], site["lines"]);
    client.shutdown();
}

#[test]
fn the_lens_dims_a_page_new_in_the_current_line_whole_in_an_older_one() {
    let f = quill();
    let mut client = Client::start(&f.root());
    let page = f.path("docs/webhooks.md");
    let older = request(
        &mut client,
        "ascribe/buildView",
        json!({ "textDocument": doc(&page), "build": "site@3.4" }),
    );
    assert_eq!(older["build"], "site@3.4");
    assert_eq!(older["pageIncluded"], false);
    let current = request(
        &mut client,
        "ascribe/buildView",
        json!({ "textDocument": doc(&page), "build": "site" }),
    );
    assert_eq!(current["pageIncluded"], true);
    client.shutdown();
}

#[test]
fn the_preview_of_an_older_line_is_under_its_folder_with_its_links_out_marked() {
    let f = quill();
    let mut client = Client::start(&f.root());
    let page = f.path("docs/upgrade.md");
    let preview = request(
        &mut client,
        "ascribe/preview",
        json!({ "textDocument": doc(&page), "build": "site@3.4" }),
    );
    assert_eq!(preview["build"], "site@3.4");
    assert_eq!(preview["page"]["route"], "/3.4/upgrade/");
    let html = preview["page"]["html"].as_str().expect("html");
    assert!(html.contains("data-ascribe-release-line=\"3.5\""), "{html}");
    client.shutdown();
}

#[test]
fn a_problem_in_one_line_says_which() {
    let f = quill();
    // A broken link in a section only 3.4 has.
    let text = std::fs::read_to_string(f.path("docs/upgrade.md")).expect("the page");
    f.write(
        "docs/upgrade.md",
        &format!(
            "{text}\n## Before 3.5\n@available: self-managed (ga 3.4, removed 3.5)\n\nSee [keys](keys.md#nope).\n"
        ),
    );
    let mut client = Client::start(&f.root());
    let page = f.path("docs/upgrade.md");
    let text = std::fs::read_to_string(&page).expect("the page");
    client.open(&page, 1, &text);
    client.settle();
    let messages: Vec<String> = client
        .diagnostics(&page)
        .iter()
        .map(|d| d.message.clone())
        .collect();
    let only = messages
        .iter()
        .filter(|m| m.contains("only in build `site@3.4`"))
        .count();
    assert_eq!(only, 1, "{messages:#?}");
    client.shutdown();
}

#[test]
fn a_hover_on_a_marks_version_says_its_line() {
    let f = quill();
    let mut client = Client::start(&f.root());
    let page = f.path("docs/keys.md");
    let text = std::fs::read_to_string(&page).expect("the page");
    let (line, character) = position_of(&text, "self-managed 3.5");
    let hover = hover_text(&mut client, &page, line, character + 2);
    assert!(
        hover.contains("`self-managed 3.5`: in line 3.5, the current line"),
        "{hover}"
    );
    client.shutdown();
}

#[test]
fn a_hover_on_replaces_says_which_page_each_line_publishes() {
    let f = quill();
    let mut client = Client::start(&f.root());
    let page = f.path("docs/configure.md");
    let text = std::fs::read_to_string(&page).expect("the page");
    let (line, character) = position_of(&text, "configure-legacy.md");
    let hover = hover_text(&mut client, &page, line, character + 1);
    assert!(
        hover.starts_with("**Replaces** `configure-legacy.md`"),
        "{hover}"
    );
    assert!(hover.contains("- 3.5 (`site`): this page"), "{hover}");
    assert!(
        hover.contains("- 3.4 (`site@3.4`): `configure-legacy.md`"),
        "{hover}"
    );
    client.shutdown();
}
