//! The editor's Pages view as the server serves it: the navigation in
//! `ascribe/inventory`, following `ascribe.nav` as it's typed, saved, and
//! closed; the `ascribe/edit` operations its drags, Place Here, and Rename
//! Label… ask for, each keeping the file's comments; and what a page or a
//! folder moving changes in the file.

#![allow(clippy::expect_used, clippy::panic, clippy::unwrap_used)]

use std::path::Path;

use lsp_types::{FileChangeType, Position};
use serde_json::{Value, json};

use crate::support::{Client, Fixture, MODEL, uri};

fn page(title: &str) -> String {
    format!("---\ntitle: {title}\n---\n\nText.\n")
}

/// A small site: a home page, Guides with one page, and Reference with
/// three, and a nested section in Guides whose page sits beside it.
fn project(nav: Option<&str>) -> Fixture {
    let mut files = vec![
        ("docs/index.md", page("Home")),
        ("docs/Guides/My Setup.md", page("My setup")),
        ("docs/Guides/Deep.md", page("Deep")),
        ("docs/Guides/Deep/Part.md", page("Part")),
        ("docs/Reference/Options.md", page("Options")),
        ("docs/Reference/Defaults.md", page("Defaults")),
        ("docs/Reference/Limits.md", page("Limits")),
        ("docs/_fragments/note.md", "Included.\n".to_owned()),
    ];
    if let Some(nav) = nav {
        files.push(("ascribe.nav", nav.to_owned()));
    }
    let files: Vec<(&str, &str)> = files.iter().map(|(p, t)| (*p, t.as_str())).collect();
    Fixture::new(MODEL, &files)
}

fn inventory(client: &mut Client, f: &Fixture) -> Value {
    client
        .request(
            "ascribe/inventory",
            json!({ "textDocument": { "uri": uri(&f.path("ascribe.toml")).as_str() } }),
        )
        .response_result
        .expect("an inventory")
}

/// The navigation as lines: each section's label and folder, then its pages
/// in order, a nested section where it sits, marked when not placed.
fn outline(inventory: &Value) -> Vec<String> {
    let mut out = Vec::new();
    lines(&inventory["navigation"], 0, &mut out);
    out
}

fn lines(section: &Value, depth: usize, out: &mut Vec<String>) {
    let pad = "  ".repeat(depth);
    out.push(format!(
        "{pad}{} [{}]",
        section["label"].as_str().unwrap(),
        section["folder"].as_str().unwrap()
    ));
    let nested = section["sections"].as_array().unwrap();
    for page in section["pages"].as_array().unwrap() {
        let path = page["path"].as_str().unwrap();
        let mark = if page["placed"] == true {
            ""
        } else {
            " (not placed)"
        };
        out.push(format!("{pad}  {path}{mark}"));
        for s in nested.iter().filter(|s| s["page"] == page["path"]) {
            lines(s, depth + 1, out);
        }
    }
    let paths: Vec<&Value> = section["pages"]
        .as_array()
        .unwrap()
        .iter()
        .map(|p| &p["path"])
        .collect();
    for s in nested.iter().filter(|s| !paths.contains(&&s["page"])) {
        lines(s, depth + 1, out);
    }
}

fn folders(inventory: &Value) -> Vec<String> {
    inventory["navigation"]["sections"]
        .as_array()
        .unwrap()
        .iter()
        .map(|s| s["folder"].as_str().unwrap().to_owned())
        .collect()
}

const REFERENCE_FIRST: &str = "version = 1\n\n[[sections]]\nfolder = \"Reference/\"\n";

#[test]
fn the_inventory_has_the_navigation_in_order() {
    let f = project(None);
    let mut client = Client::start(&f.root());
    client.settle();
    assert_eq!(
        outline(&inventory(&mut client, &f)),
        [
            "Start []",
            "  Guides [Guides/]",
            "    Guides/Deep.md",
            "    Deep [Guides/Deep/]",
            "      Guides/Deep/Part.md",
            "    Guides/My Setup.md",
            "  Reference [Reference/]",
            "    Reference/Defaults.md",
            "    Reference/Limits.md",
            "    Reference/Options.md",
        ]
    );
}

#[test]
fn the_navigation_follows_ascribe_nav_as_its_typed_closed_and_saved() {
    let f = project(None);
    let mut client = Client::start(&f.root());
    client.settle();
    let nav = f.path("ascribe.nav");
    assert_eq!(
        folders(&inventory(&mut client, &f)),
        ["Guides/", "Reference/"]
    );

    // Typed in the editor, before it's saved.
    client.open(&nav, 1, "version = 1\n");
    client.replace(&nav, 2, REFERENCE_FIRST);
    client.settle();
    assert_eq!(
        folders(&inventory(&mut client, &f)),
        ["Reference/", "Guides/"]
    );

    // Closed without saving: the disk has none.
    client.close(&nav);
    client.settle();
    assert_eq!(
        folders(&inventory(&mut client, &f)),
        ["Guides/", "Reference/"]
    );

    // Written on disk, as a save or a drag's edit saved is.
    f.write("ascribe.nav", REFERENCE_FIRST);
    client.watched(&[(&nav, FileChangeType::CREATED)]);
    client.settle();
    assert_eq!(
        folders(&inventory(&mut client, &f)),
        ["Reference/", "Guides/"]
    );

    // A page the list leaves out is marked where the default puts it.
    f.write(
        "ascribe.nav",
        "version = 1\n[[sections]]\nfolder = \"Reference/\"\npages = [\"Options.md\"]\nlabel = \"API\"\n",
    );
    client.watched(&[(&nav, FileChangeType::CHANGED)]);
    client.settle();
    let inventory = inventory(&mut client, &f);
    let reference = &inventory["navigation"]["sections"][0];
    assert_eq!(reference["label"], "API");
    assert_eq!(reference["labelSet"], "API");
    assert_eq!(reference["listsPages"], true);
    assert_eq!(
        outline(&inventory)[1..5],
        [
            "  API [Reference/]",
            "    Reference/Options.md",
            "    Reference/Defaults.md (not placed)",
            "    Reference/Limits.md (not placed)",
        ]
    );
}

#[test]
fn a_page_s_label_is_its_nav_label_and_rename_label_sets_it() {
    let f = project(None);
    let mut client = Client::start(&f.root());
    client.settle();
    let entry = |inventory: &Value, path: &str| {
        inventory["pages"]
            .as_array()
            .unwrap()
            .iter()
            .find(|p| p["path"] == path)
            .cloned()
            .unwrap_or_else(|| panic!("no page {path}"))
    };
    let setup = entry(&inventory(&mut client, &f), "Guides/My Setup.md");
    assert_eq!(setup["label"], "My setup");
    assert_eq!(setup["navLabel"], Value::Null);

    // Asked through the page, which isn't open: a name with a space, as
    // the view joins it to the content root.
    let path = f.path("docs/Guides/My Setup.md");
    let text = page("My setup");
    let set = |client: &mut Client, label: Value| {
        let at = json!({ "line": 0, "character": 0 });
        client
            .request(
                "ascribe/edit",
                json!({
                    "textDocument": { "uri": uri(&path).as_str() },
                    "range": { "start": at, "end": at },
                    "action": "setPageLabel",
                    "args": { "label": label },
                }),
            )
            .response_result
            .expect("an answer")
    };
    let after = applied(&set(&mut client, json!("Setup")), &path, &text);
    assert_eq!(
        after,
        "---\ntitle: My setup\nnav-label: Setup\n---\n\nText.\n"
    );
    client.open(&path, 1, &after);
    client.settle();
    let setup = entry(&inventory(&mut client, &f), "Guides/My Setup.md");
    assert_eq!(setup["label"], "Setup");
    assert_eq!(setup["navLabel"], "Setup");
    assert_eq!(setup["title"], "My setup");

    // Empty takes it out, and the title is its label again.
    let cleared = applied(&set(&mut client, json!("")), &path, &after);
    assert_eq!(cleared, text);
    client.replace(&path, 2, &cleared);
    client.settle();
    let setup = entry(&inventory(&mut client, &f), "Guides/My Setup.md");
    assert_eq!(setup["label"], "My setup");
    assert_eq!(setup["navLabel"], Value::Null);
}

/// The edit `ascribe/edit` answers `action` with.
fn edit(client: &mut Client, f: &Fixture, action: &str, args: Value) -> Value {
    let at = json!({ "line": 0, "character": 0 });
    client
        .request(
            "ascribe/edit",
            json!({
                "textDocument": { "uri": uri(&f.path("ascribe.toml")).as_str() },
                "range": { "start": at, "end": at },
                "action": action,
                "args": args,
            }),
        )
        .response_result
        .expect("an answer")
}

/// The text the edit's change of `path` makes of `text`.
fn applied(edit: &Value, path: &Path, text: &str) -> String {
    let edits = edit["edit"]["changes"][uri(path).as_str()]
        .as_array()
        .unwrap_or_else(|| panic!("no edit of {}: {edit}", path.display()));
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

/// Asks for `action` with the file as `text`, and gives the file after,
/// once the server has it as the editor would.
fn step(
    client: &mut Client,
    f: &Fixture,
    version: i32,
    text: &str,
    action: &str,
    args: Value,
) -> String {
    let answer = edit(client, f, action, args);
    let after = applied(&answer, &f.path("ascribe.nav"), text);
    client.replace(&f.path("ascribe.nav"), version, &after);
    client.settle();
    after
}

#[test]
fn the_first_drop_creates_the_file_with_the_sections_listed() {
    let f = project(None);
    let mut client = Client::start(&f.root());
    client.settle();
    let answer = edit(
        &mut client,
        &f,
        "moveSection",
        json!({ "folder": "Reference/", "before": "Guides/" }),
    );
    let changes = answer["edit"]["documentChanges"]
        .as_array()
        .unwrap_or_else(|| panic!("a new file: {answer}"));
    assert_eq!(changes[0]["kind"], "create");
    assert_eq!(changes[0]["uri"], uri(&f.path("ascribe.nav")).as_str());
    assert_eq!(changes[0]["options"]["overwrite"], false);
    assert_eq!(
        changes[1]["edits"][0]["newText"],
        "version = 1\n\n[[sections]]\nfolder = \"Reference/\"\n\n[[sections]]\nfolder = \"Guides/\"\n"
    );
}

#[test]
fn a_top_section_beside_its_page_moves_among_the_top_sections() {
    // `Reference.md` beside `Reference/` doesn't place it among the root's
    // pages: it's ordered by `[[sections]]` as any top section is (decision
    // #610). Below the root, `Guides/Deep.md` still places `Guides/Deep/`.
    let files = [
        ("docs/index.md", page("Home")),
        ("docs/Reference.md", page("Reference")),
        ("docs/Guides/My Setup.md", page("My setup")),
        ("docs/Guides/Deep.md", page("Deep")),
        ("docs/Guides/Deep/Part.md", page("Part")),
        ("docs/Reference/Options.md", page("Options")),
    ];
    let files: Vec<(&str, &str)> = files.iter().map(|(p, t)| (*p, t.as_str())).collect();
    let f = Fixture::new(MODEL, &files);
    let mut client = Client::start(&f.root());
    client.settle();
    assert_eq!(
        folders(&inventory(&mut client, &f)),
        ["Guides/", "Reference/"]
    );
    let answer = edit(
        &mut client,
        &f,
        "moveSection",
        json!({ "folder": "Reference/", "before": "Guides/" }),
    );
    let changes = answer["edit"]["documentChanges"]
        .as_array()
        .unwrap_or_else(|| panic!("a new file: {answer}"));
    assert_eq!(
        changes[1]["edits"][0]["newText"],
        "version = 1\n\n[[sections]]\nfolder = \"Reference/\"\n\n[[sections]]\nfolder = \"Guides/\"\n"
    );
    let refused = edit(
        &mut client,
        &f,
        "moveSection",
        json!({ "folder": "Guides/Deep/" }),
    );
    assert!(
        refused["error"]
            .as_str()
            .is_some_and(|e| e.contains("move that page among `Guides/`'s pages")),
        "{refused}"
    );
}

#[test]
fn each_drop_keeps_the_files_comments_and_gives_the_order_asked_for() {
    let text = "# The site's order.\nversion = 1\n\n# The guides.\n[[sections]]\nfolder = \"Guides/\" # first\n";
    let f = project(Some(text));
    let mut client = Client::start(&f.root());
    let nav = f.path("ascribe.nav");
    client.open(&nav, 1, text);
    client.settle();

    // Reference, which the file leaves out, before Guides: it's listed
    // first, its comment and Guides' kept.
    let text = step(
        &mut client,
        &f,
        2,
        text,
        "moveSection",
        json!({ "folder": "Reference/", "before": "Guides/" }),
    );
    assert_eq!(
        text,
        "# The site's order.\nversion = 1\n\n[[sections]]\nfolder = \"Reference/\"\n\n# The guides.\n[[sections]]\nfolder = \"Guides/\" # first\n"
    );
    assert_eq!(
        folders(&inventory(&mut client, &f)),
        ["Reference/", "Guides/"]
    );

    // Limits after Options: Reference's pages are written as they stood,
    // then Limits moves.
    let text = step(
        &mut client,
        &f,
        3,
        &text,
        "movePage",
        json!({ "page": "Reference/Limits.md" }),
    );
    assert!(
        text.contains(
            "folder = \"Reference/\"\npages = [\"Defaults.md\", \"Options.md\", \"Limits.md\"]\n"
        ),
        "{text}"
    );
    // Options first.
    let text = step(
        &mut client,
        &f,
        4,
        &text,
        "movePage",
        json!({ "page": "Reference/Options.md", "before": "Reference/Defaults.md" }),
    );
    assert!(
        text.contains("pages = [\"Options.md\", \"Defaults.md\", \"Limits.md\"]\n"),
        "{text}"
    );
    let inventory = inventory(&mut client, &f);
    assert_eq!(
        outline(&inventory)[1..5],
        [
            "  Reference [Reference/]",
            "    Reference/Options.md",
            "    Reference/Defaults.md",
            "    Reference/Limits.md",
        ]
    );

    // Its label, set and then taken out.
    let labeled = step(
        &mut client,
        &f,
        5,
        &text,
        "setSectionLabel",
        json!({ "folder": "Reference/", "label": "API" }),
    );
    assert!(
        labeled.contains("folder = \"Reference/\"\nlabel = \"API\"\npages = ["),
        "{labeled}"
    );
    let text = step(
        &mut client,
        &f,
        6,
        &labeled,
        "setSectionLabel",
        json!({ "folder": "Reference/", "label": null }),
    );
    assert!(!text.contains("label"), "{text}");
    assert!(text.starts_with("# The site's order.\n"), "{text}");
    assert!(text.contains("# The guides.\n[[sections]]\nfolder = \"Guides/\" # first\n"));
    // What the drags wrote reads, and names nothing that isn't there.
    assert!(client.codes(&nav).is_empty(), "{:?}", client.codes(&nav));
}

#[test]
fn a_page_its_section_leaves_out_is_placed_and_a_section_moves_after_its_siblings() {
    let text = "version = 1\n\n[[sections]]\nfolder = \"Reference/\"\npages = [\n  \"Options.md\", # the most read\n]\n";
    let f = project(Some(text));
    let mut client = Client::start(&f.root());
    let nav = f.path("ascribe.nav");
    client.open(&nav, 1, text);
    client.settle();
    let text = step(
        &mut client,
        &f,
        2,
        text,
        "placePage",
        json!({ "page": "Reference/Limits.md" }),
    );
    assert_eq!(
        text,
        "version = 1\n\n[[sections]]\nfolder = \"Reference/\"\npages = [\n  \"Options.md\", # the most read\n  \"Limits.md\",\n]\n"
    );
    let refused = edit(
        &mut client,
        &f,
        "placePage",
        json!({ "page": "Reference/Limits.md" }),
    );
    assert_eq!(
        refused["error"],
        "`Limits.md` is in `Reference/`'s pages already."
    );

    // At the end: the page it leaves out is placed before it.
    let text = step(
        &mut client,
        &f,
        3,
        &text,
        "movePage",
        json!({ "page": "Reference/Options.md" }),
    );
    assert_eq!(
        text,
        "version = 1\n\n[[sections]]\nfolder = \"Reference/\"\npages = [\n  \"Limits.md\",\n  \"Defaults.md\",\n  \"Options.md\", # the most read\n]\n"
    );
    assert_eq!(
        outline(&inventory(&mut client, &f))[1..5],
        [
            "  Reference [Reference/]",
            "    Reference/Limits.md",
            "    Reference/Defaults.md",
            "    Reference/Options.md",
        ]
    );

    // Reference after Guides, which the file then lists too.
    let text = step(
        &mut client,
        &f,
        4,
        &text,
        "moveSection",
        json!({ "folder": "Reference/" }),
    );
    assert_eq!(
        text,
        "version = 1\n\n[[sections]]\nfolder = \"Guides/\"\n\n[[sections]]\nfolder = \"Reference/\"\npages = [\n  \"Limits.md\",\n  \"Defaults.md\",\n  \"Options.md\", # the most read\n]\n"
    );
    assert_eq!(
        folders(&inventory(&mut client, &f)),
        ["Guides/", "Reference/"]
    );
    assert!(client.codes(&nav).is_empty(), "{:?}", client.codes(&nav));

    // A page can't go before another section's.
    let refused = edit(
        &mut client,
        &f,
        "movePage",
        json!({ "page": "Reference/Options.md", "before": "Guides/My Setup.md" }),
    );
    assert!(
        refused["error"]
            .as_str()
            .is_some_and(|e| e.contains("moving its file")),
        "{refused}"
    );
}

/// The edit `workspace/willRenameFiles` answers for `from` moving to `to`,
/// applied to `ascribe.nav`'s `text`.
fn renamed(client: &mut Client, f: &Fixture, from: &str, to: &str, text: &str) -> String {
    let answer = client
        .request(
            "workspace/willRenameFiles",
            json!({ "files": [{
                "oldUri": uri(&f.path(from)).as_str(),
                "newUri": uri(&f.path(to)).as_str(),
            }] }),
        )
        .response_result
        .expect("a rename");
    applied(&json!({ "edit": answer }), &f.path("ascribe.nav"), text)
}

#[test]
fn a_page_or_a_folder_that_moves_keeps_ascribe_nav_right() {
    let text = "version = 1\n\n[[sections]]\nfolder = \"Guides/\"\npages = [\"My Setup.md\", \"Deep.md\"]\n\n[[sections]]\nfolder = \"Guides/Deep/\" # nested\n\n[[sections]]\nfolder = \"Reference/\"\npages = [\"Options.md\", \"Defaults.md\", \"Limits.md\"]\n";
    let f = project(Some(text));
    let mut client = Client::start(&f.root());
    client.settle();

    // A page renamed in its folder keeps its place.
    assert_eq!(
        renamed(
            &mut client,
            &f,
            "docs/Reference/Options.md",
            "docs/Reference/Settings.md",
            text
        ),
        text.replace("\"Options.md\"", "\"Settings.md\"")
    );
    // A folder renamed: its section and its nested section's follow.
    assert_eq!(
        renamed(&mut client, &f, "docs/Guides", "docs/How-to", text),
        text.replace("\"Guides/", "\"How-to/")
    );
    // A page moved to another section leaves its old section's list; the
    // new one places it by its default.
    assert_eq!(
        renamed(
            &mut client,
            &f,
            "docs/Reference/Defaults.md",
            "docs/Guides/Defaults.md",
            text
        ),
        text.replace("\"Options.md\", \"Defaults.md\", ", "\"Options.md\", ")
    );
}

#[test]
fn an_unlisted_page_its_section_leaves_out_is_marked_and_place_here_refuses_it() {
    let text = "version = 1\n\n[[sections]]\nfolder = \"Reference/\"\npages = [\"Options.md\"]\n";
    let f = project(Some(text));
    f.write(
        "docs/Reference/Secret.md",
        "---\ntitle: Secret\nlisting: unlisted\n---\n\nText.\n",
    );
    let mut client = Client::start(&f.root());
    let inventory = inventory(&mut client, &f);
    let reference = inventory["navigation"]["sections"]
        .as_array()
        .unwrap()
        .iter()
        .find(|s| s["folder"] == "Reference/")
        .unwrap();
    let entry = |path: &str| {
        reference["pages"]
            .as_array()
            .unwrap()
            .iter()
            .find(|p| p["path"] == path)
            .unwrap()
            .clone()
    };
    assert_eq!(
        entry("Reference/Secret.md"),
        json!({ "path": "Reference/Secret.md", "placed": true, "unlisted": true })
    );
    assert_eq!(
        entry("Reference/Limits.md"),
        json!({ "path": "Reference/Limits.md", "placed": false, "unlisted": false })
    );
    let refused = edit(
        &mut client,
        &f,
        "placePage",
        json!({ "page": "Reference/Secret.md" }),
    );
    assert_eq!(
        refused["error"],
        "`Secret.md` is `unlisted`, so no sidebar shows it and it has no place to take."
    );
}
