//! `ascribe/edit`, over an in-memory connection: every operation's edit,
//! applied to a page, gives the expected text, in canonical form, with no new
//! diagnostics; and the errors, the inverses, and the placeholders.

#![allow(clippy::expect_used, clippy::panic, clippy::unwrap_used)]

mod support;

use std::path::PathBuf;

use ascribe_core::{FileId, LineIndex, WideEncoding, WideLineCol};
use serde_json::{Value, json};
use support::{Client, Fixture};

const MODEL: &str = r#"spec = "0.1"

[project]
content-root = "docs"

[dimensions.pm]
label = "Package manager"
values = ["npm", "pnpm", "yarn"]
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
height = "number?"

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

[widgets.quill-lab]
forms = ["line"]
primary = "identifier"
binding = "self"
attributes = { size = { type = "enum", values = ["small", "large"] }, note = "string?" }
"#;

const APP: &str = "# :snippet-start: main\nprint('hi')\n# :snippet-end:\n";

/// A project, a client, and the pages each case writes.
struct Harness {
    fixture: Fixture,
    client: Client,
    cases: usize,
}

/// What an edit did: the page after it, and the text left selected.
#[derive(Debug)]
struct Done {
    after: String,
    selected: Option<String>,
}

impl Harness {
    fn new() -> Harness {
        let fixture = Fixture::new(
            MODEL,
            &[
                (
                    "docs/guide/setup.md",
                    "---\ntitle: Setup\n---\n# Set up\n\n@include: ../_frag.md\n\n## Configure it\n",
                ),
                (
                    "docs/keys.md",
                    "---\ntitle: Keys\n---\n# Keys\n\n## Rotate keys\n",
                ),
                ("docs/_frag.md", "## From a fragment\n\nShared.\n"),
                ("docs/_loop.md", "Loops back.\n\n@include: c0.md\n"),
                ("docs/img/logo.png", "png"),
                ("code/app.py", APP),
            ],
        );
        let client = Client::start(&fixture.root());
        Harness {
            fixture,
            client,
            cases: 0,
        }
    }

    /// Runs `action` on `marked`, a page whose cursor is `‸` or whose
    /// selection is between `«` and `»`, and applies the edit. The page is
    /// written as `docs/c<n>.md` and opened.
    fn run(&mut self, marked: &str, action: &str, args: Value) -> Result<Done, String> {
        let name = format!("docs/c{}.md", self.cases);
        self.cases += 1;
        self.run_as(&name, marked, action, args)
    }

    fn run_as(
        &mut self,
        name: &str,
        marked: &str,
        action: &str,
        args: Value,
    ) -> Result<Done, String> {
        let (text, start, end) = unmark(marked);
        let path = self.fixture.path(name);
        self.fixture.write(name, &text);
        self.client.open(&path, 7, &text);
        let response = self.client.request(
            "ascribe/edit",
            json!({
                "textDocument": { "uri": support::uri(&path).as_str() },
                "range": { "start": position(&text, start), "end": position(&text, end) },
                "action": action,
                "args": args,
                "version": 7,
            }),
        );
        self.client.close(&path);
        let result = response.response_result.expect("the request succeeds");
        if let Some(error) = result.get("error") {
            return Err(error.as_str().expect("a message").to_owned());
        }
        let uri = support::uri(&path);
        let edits = result["edit"]["changes"][uri.as_str()]
            .as_array()
            .expect("edits to the page")
            .clone();
        assert_eq!(
            result["edit"]["changes"].as_object().map(|m| m.len()),
            Some(1),
            "only the page changes"
        );
        let after = apply(&text, &edits);
        let selected = match &result["select"] {
            Value::Null => None,
            range => {
                let a = offset(&after, &range["start"]);
                let b = offset(&after, &range["end"]);
                Some(after[a..b].to_owned())
            }
        };
        assert_canonical(&text, &after);
        Ok(Done { after, selected })
    }

    /// The page after `action`, which must succeed.
    fn after(&mut self, marked: &str, action: &str, args: Value) -> String {
        match self.run(marked, action, args) {
            Ok(done) => done.after,
            Err(e) => panic!("{action} failed: {e}\n{marked}"),
        }
    }

    /// The error `action` gives, which must fail.
    fn error(&mut self, marked: &str, action: &str, args: Value) -> String {
        match self.run(marked, action, args) {
            Ok(done) => panic!("{action} succeeded:\n{}", done.after),
            Err(e) => e,
        }
    }

    fn root(&self) -> PathBuf {
        self.fixture.root()
    }
}

/// The text without its markers, and the range they mark.
fn unmark(marked: &str) -> (String, usize, usize) {
    if let Some(at) = marked.find('‸') {
        let text = marked.replacen('‸', "", 1);
        return (text, at, at);
    }
    let start = marked.find('«').expect("a cursor or a selection");
    let text = marked.replacen('«', "", 1);
    let end = text.find('»').expect("the end of the selection");
    (text.replacen('»', "", 1), start, end)
}

fn position(text: &str, offset: usize) -> Value {
    let index = LineIndex::new(text);
    let p = index
        .wide_line_col(WideEncoding::Utf16, offset)
        .expect("a position");
    json!({ "line": p.line, "character": p.col })
}

fn offset(text: &str, position: &Value) -> usize {
    let index = LineIndex::new(text);
    index
        .wide_offset(
            WideEncoding::Utf16,
            WideLineCol {
                line: position["line"].as_u64().unwrap() as u32,
                col: position["character"].as_u64().unwrap() as u32,
            },
        )
        .expect("an offset")
}

fn apply(text: &str, edits: &[Value]) -> String {
    let edits: Vec<ascribe_core::TextEdit> = edits
        .iter()
        .map(|e| {
            let a = offset(text, &e["range"]["start"]);
            let b = offset(text, &e["range"]["end"]);
            ascribe_core::TextEdit::replace(
                ascribe_core::Span::new(a, b),
                e["newText"].as_str().expect("text"),
            )
        })
        .collect();
    ascribe_core::apply_edits(text, &edits).expect("edits that apply")
}

/// Formatting the page after the edit changes nothing the edit wrote: the
/// test pages are canonical before it, so nothing at all.
fn assert_canonical(before: &str, after: &str) {
    let model = ascribe_model::load_str(MODEL, FileId::new(0)).expect("a model");
    let options = ascribe_fmt::options_from_model(&model);
    if !ascribe_fmt::format(before, &options, &model).is_empty() {
        return;
    }
    let formatted = ascribe_fmt::format_source(after, &options, &model);
    assert_eq!(formatted, after, "the edit is canonical");
}

// -- Notes ------------------------------------------------------------------

#[test]
fn notes_wrap_a_paragraph_or_blocks_and_unwrap_again() {
    let mut h = Harness::new();
    let page = "# Title\n\nFirst ‸paragraph.\n\nSecond.\n";
    let wrapped = h.after(page, "wrapNote", json!({ "type": "tip" }));
    assert_eq!(
        wrapped,
        "# Title\n\n@note {type=tip}\nFirst paragraph.\n\nSecond.\n"
    );
    let back = h.after(&wrapped.replace("First", "‸First"), "unwrapNote", json!({}));
    assert_eq!(back, page.replace('‸', ""));

    let plain = h.after(page, "wrapNote", json!({}));
    assert_eq!(plain, "# Title\n\n@note\nFirst paragraph.\n\nSecond.\n");

    let blocks = "# Title\n\n«First.\n\n- a\n- b»\n\nAfter.\n";
    let wrapped = h.after(blocks, "wrapNote", json!({ "type": "warning" }));
    assert_eq!(
        wrapped,
        "# Title\n\n@note {type=warning}:\nFirst.\n\n- a\n- b\n@end\n\nAfter.\n"
    );
    let back = h.after(&wrapped.replace("- a", "- ‸a"), "unwrapNote", json!({}));
    assert_eq!(back, "# Title\n\nFirst.\n\n- a\n- b\n\nAfter.\n");
}

#[test]
fn a_note_in_a_list_item_is_indented_to_its_content() {
    let mut h = Harness::new();
    let page = "1. Install.\n\n   Then ‸check it.\n2. Done.\n";
    assert_eq!(
        h.after(page, "wrapNote", json!({ "type": "tip" })),
        "1. Install.\n\n   @note {type=tip}\n   Then check it.\n2. Done.\n"
    );
    let first = "1. ‸Install.\n2. Done.\n";
    let wrapped = h.after(first, "wrapNote", json!({}));
    assert_eq!(wrapped, "1. @note\n   Install.\n2. Done.\n");
    assert_eq!(
        h.after(
            &wrapped.replace("Install", "‸Install"),
            "unwrapNote",
            json!({})
        ),
        first.replace('‸', "")
    );
    let quoted = "> Quoted ‸text.\n";
    assert_eq!(
        h.after(quoted, "wrapNote", json!({})),
        "> @note\n> Quoted text.\n"
    );
}

#[test]
fn note_types_change_and_are_checked() {
    let mut h = Harness::new();
    let page = "@note {type=tip}: A ‸line note.\n";
    assert_eq!(
        h.after(page, "setNoteType", json!({ "type": "extra" })),
        "@note {type=extra}: A line note.\n"
    );
    assert_eq!(
        h.after(page, "setNoteType", json!({ "type": "note" })),
        "@note: A line note.\n"
    );
    assert_eq!(
        h.after(
            "@note:\nIn a ‸container.\n@end\n",
            "setNoteType",
            json!({ "type": "caution" })
        ),
        "@note {type=caution}:\nIn a container.\n@end\n"
    );
    let error = h.error(page, "setNoteType", json!({ "type": "danger" }));
    assert!(error.contains("`danger` isn't a note type"), "{error}");
    assert!(error.contains("`extra`"), "{error}");
    let error = h.error("Just ‸text.\n", "setNoteType", json!({ "type": "tip" }));
    assert_eq!(error, "Put the cursor in a note.");
}

#[test]
fn a_line_note_unwraps_to_its_text() {
    let mut h = Harness::new();
    assert_eq!(
        h.after(
            "@note {type=tip}: Careful ‸here.\n",
            "unwrapNote",
            json!({})
        ),
        "Careful here.\n"
    );
}

#[test]
fn notes_become_details() {
    let mut h = Harness::new();
    assert_eq!(
        h.after(
            "@note {type=tip}\nBound ‸text.\n",
            "noteToDetails",
            json!({ "title": "More" })
        ),
        ".More\n@details\nBound text.\n"
    );
    assert_eq!(
        h.after(
            ".Old\n@note:\nIn ‸it.\n@end\n",
            "noteToDetails",
            json!({ "title": "New" })
        ),
        ".New\n@details:\nIn it.\n@end\n"
    );
    assert_eq!(
        h.after(
            "@note: Line ‸text.\n",
            "noteToDetails",
            json!({ "title": ".NET" })
        ),
        ".\\.NET\n@details\nLine text.\n"
    );
    let error = h.error("@note: Line ‸text.\n", "noteToDetails", json!({}));
    assert_eq!(error, "`title` is missing.");
}

// -- Details ----------------------------------------------------------------

#[test]
fn details_wrap_a_block_or_blocks_and_unwrap_again() {
    let mut h = Harness::new();
    let page = "# T\n\n```sh\n‸npm i\n```\n";
    let wrapped = h.after(page, "wrapDetails", json!({ "title": "The command" }));
    assert_eq!(
        wrapped,
        "# T\n\n.The command\n@details\n```sh\nnpm i\n```\n"
    );
    assert_eq!(
        h.after(
            &wrapped.replace("npm i", "‸npm i"),
            "unwrapDetails",
            json!({})
        ),
        page.replace('‸', "")
    );
    let blocks = "«One.\n\nTwo.»\n";
    let wrapped = h.after(blocks, "wrapDetails", json!({ "title": "Both" }));
    assert_eq!(wrapped, ".Both\n@details:\nOne.\n\nTwo.\n@end\n");
    assert_eq!(
        h.after(&wrapped.replace("Two", "‸Two"), "unwrapDetails", json!({})),
        "One.\n\nTwo.\n"
    );
}

#[test]
fn a_title_line_after_a_code_block_gets_a_blank_line() {
    let mut h = Harness::new();
    let page = "```sh\nnpm i\n```\nRight ‸after.\n";
    assert_eq!(
        h.after(page, "wrapDetails", json!({ "title": "More" })),
        "```sh\nnpm i\n```\n\n.More\n@details\nRight after.\n"
    );
}

// -- Steps ------------------------------------------------------------------

#[test]
fn steps_mark_a_numbered_list_and_come_off_again() {
    let mut h = Harness::new();
    let page = "Do this:\n\n1. ‸One.\n2. Two.\n";
    let marked = h.after(page, "makeSteps", json!({}));
    assert_eq!(marked, "Do this:\n\n@steps\n1. One.\n2. Two.\n");
    assert_eq!(
        h.after(&marked.replace("One", "‸One"), "removeSteps", json!({})),
        page.replace('‸', "")
    );
    assert_eq!(
        h.error("- ‸One.\n- Two.\n", "makeSteps", json!({})),
        "Only a numbered list can be steps."
    );
    assert_eq!(
        h.error(&marked.replace("One", "‸One"), "makeSteps", json!({})),
        "The list is already steps."
    );
}

// -- Heading ids --------------------------------------------------------------

#[test]
fn headings_get_an_id() {
    let mut h = Harness::new();
    assert_eq!(
        h.after("# Getting ‸started\n\nText.\n", "addHeadingId", json!({})),
        "# Getting started\n@id: getting-started\n\nText.\n"
    );
    assert_eq!(
        h.after(
            "## Set ‸up\n",
            "addHeadingId",
            json!({ "id": "setup_v1.2" })
        ),
        "## Set up\n@id: setup_v1.2\n"
    );
    assert_eq!(
        h.error("# A\n\n## ‸B\n", "addHeadingId", json!({ "id": "a" })),
        "Another heading on the page has the id `a`."
    );
    assert_eq!(
        h.error("# A\n@id: x\n\n‸Text.\n", "addHeadingId", json!({})),
        "Put the cursor in a heading."
    );
    assert_eq!(
        h.error("# ‸A\n@id: x\n", "addHeadingId", json!({})),
        "The heading already has an `@id`."
    );
    assert!(
        h.error("# ‸A\n", "addHeadingId", json!({ "id": "two words" }))
            .contains("can't be an id")
    );
    // An id a fragment the page includes brings in.
    let error = h.error(
        "# ‸A\n\n@include: _frag.md\n",
        "addHeadingId",
        json!({ "id": "from-a-fragment" }),
    );
    assert_eq!(
        error,
        "Another heading on the page has the id `from-a-fragment`."
    );
}

// -- Inserting blocks ---------------------------------------------------------

#[test]
fn notes_are_inserted_with_placeholder_text_selected() {
    let mut h = Harness::new();
    let done = h
        .run(
            "# T\n\n‸\n\nText.\n",
            "insertNote",
            json!({ "type": "tip" }),
        )
        .unwrap();
    assert_eq!(
        done.after,
        "# T\n\n@note {type=tip}: Write the note here.\n\nText.\n"
    );
    assert_eq!(done.selected.as_deref(), Some("Write the note here."));
    let done = h
        .run(
            "Text.\n‸\nMore.\n",
            "insertNote",
            json!({ "text": "Careful." }),
        )
        .unwrap();
    assert_eq!(done.after, "Text.\n\n@note: Careful.\n\nMore.\n");
    assert_eq!(done.selected, None);
    assert_eq!(
        h.error("Some ‸text.\n", "insertNote", json!({})),
        "Put the cursor on a blank line between blocks."
    );
}

#[test]
fn steps_are_inserted() {
    let mut h = Harness::new();
    let done = h.run("‸\n", "insertSteps", json!({ "count": 2 })).unwrap();
    assert_eq!(done.after, "@steps\n1. Step 1.\n2. Step 2.\n");
    assert_eq!(done.selected.as_deref(), Some("Step 1."));
    assert_eq!(
        h.error("‸\n", "insertSteps", json!({ "count": 0 })),
        "`count` must be from 1 to 50."
    );
}

#[test]
fn variant_groups_are_inserted_in_declared_order() {
    let mut h = Harness::new();
    let done = h
        .run(
            "Install:\n‸\n",
            "insertVariantGroup",
            json!({ "dimension": "pm", "values": ["pnpm", "npm"] }),
        )
        .unwrap();
    assert_eq!(
        done.after,
        "Install:\n\n@variant {pm=npm}:\nWrite what applies to npm here.\n@variant {pm=pnpm}:\nWrite what applies to PNPM here.\n@end\n"
    );
    assert_eq!(
        done.selected.as_deref(),
        Some("Write what applies to npm here.")
    );
    let error = h.error(
        "‸\n",
        "insertVariantGroup",
        json!({ "dimension": "os", "values": ["linux"] }),
    );
    assert_eq!(
        error,
        "`os` isn't a dimension. The dimensions are `pm`, `deployment`."
    );
    let error = h.error(
        "‸\n",
        "insertVariantGroup",
        json!({ "dimension": "pm", "values": ["bun"] }),
    );
    assert_eq!(
        error,
        "`bun` isn't a value of `pm`. Its values are `npm`, `pnpm`, `yarn`."
    );
}

#[test]
fn a_variant_group_inside_a_step_is_indented() {
    let mut h = Harness::new();
    let page = "@steps\n1. Install.\n‸\n2. Run.\n";
    assert_eq!(
        h.after(
            page,
            "insertVariantGroup",
            json!({ "dimension": "deployment", "values": ["cloud", "self-managed"] })
        ),
        "@steps\n1. Install.\n\n   @variant {deployment=cloud}:\n   Write what applies to cloud here.\n   @variant {deployment=self-managed}:\n   Write what applies to self-managed here.\n   @end\n\n2. Run.\n"
    );
    // After the last step, a line indented to its content is in it.
    let last = "@steps\n1. Install.\n2. Run.\n   ‸\n";
    assert_eq!(
        h.after(last, "insertNote", json!({})),
        "@steps\n1. Install.\n2. Run.\n\n   @note: Write the note here.\n"
    );
    // Not indented, it's after the list.
    let after = "@steps\n1. Install.\n2. Run.\n‸\n";
    assert_eq!(
        h.after(after, "insertNote", json!({})),
        "@steps\n1. Install.\n2. Run.\n\n@note: Write the note here.\n"
    );
}

#[test]
fn a_variant_group_can_not_go_directly_in_an_arm() {
    let mut h = Harness::new();
    let page = "@variant {pm=npm}:\nA.\n‸\n@variant {pm=pnpm}:\nB.\n@end\n";
    let error = h.error(
        page,
        "insertVariantGroup",
        json!({ "dimension": "deployment", "values": ["cloud"] }),
    );
    assert_eq!(
        error,
        "A group of `@variant` arms can't go directly in an arm of another. Put both dimensions on one arm instead."
    );
    // In a list in an arm, it can.
    assert!(
        h.run(
            "@variant {pm=npm}:\n- A.\n‸\n- B.\n@variant {pm=pnpm}:\nB.\n@end\n",
            "insertVariantGroup",
            json!({ "dimension": "deployment", "values": ["cloud"] })
        )
        .is_ok()
    );
}

#[test]
fn arms_are_added_in_declared_order_and_removed() {
    let mut h = Harness::new();
    let page = "@variant {pm=npm}:\n‸A.\n@variant {pm=yarn}:\nC.\n@end\n";
    let done = h
        .run(page, "addVariantArm", json!({ "value": "pnpm" }))
        .unwrap();
    assert_eq!(
        done.after,
        "@variant {pm=npm}:\nA.\n@variant {pm=pnpm}:\nWrite what applies to PNPM here.\n@variant {pm=yarn}:\nC.\n@end\n"
    );
    assert_eq!(
        done.selected.as_deref(),
        Some("Write what applies to PNPM here.")
    );
    let two = "@variant {pm=npm}:\n‸A.\n@end\n";
    let done = h
        .run(two, "addVariantArm", json!({ "value": "yarn" }))
        .unwrap();
    assert_eq!(
        done.after,
        "@variant {pm=npm}:\nA.\n@variant {pm=yarn}:\nWrite what applies to yarn here.\n@end\n"
    );
    assert_eq!(
        h.error(page, "addVariantArm", json!({ "value": "yarn" })),
        "The group already has an arm for `yarn`."
    );
    // Removing the arm just added gives the page back.
    let added = done.after.replace("Write what", "‸Write what");
    assert_eq!(
        h.after(&added, "removeVariantArm", json!({})),
        two.replace('‸', "")
    );
    // A middle arm.
    assert_eq!(
        h.after(
            "@variant {pm=npm}:\nA.\n@variant {pm=pnpm}:\n‸B.\n@variant {pm=yarn}:\nC.\n@end\n",
            "removeVariantArm",
            json!({})
        ),
        "@variant {pm=npm}:\nA.\n@variant {pm=yarn}:\nC.\n@end\n"
    );
    // A group left with one arm stays a group; its only arm can't go.
    assert!(
        h.error(two, "removeVariantArm", json!({}))
            .starts_with("It's the group's only arm.")
    );
}

#[test]
fn details_includes_and_snippets_are_inserted() {
    let mut h = Harness::new();
    let done = h
        .run("‸\n", "insertDetails", json!({ "title": "More" }))
        .unwrap();
    assert_eq!(
        done.after,
        ".More\n@details:\nWrite the details here.\n@end\n"
    );
    assert_eq!(done.selected.as_deref(), Some("Write the details here."));

    assert_eq!(
        h.after("# T\n‸\n", "insertInclude", json!({ "path": "_frag.md" })),
        "# T\n\n@include: _frag.md\n"
    );
    assert_eq!(
        h.after(
            "‸\n",
            "insertInclude",
            json!({ "path": "keys.md#rotate-keys" })
        ),
        "@include: keys.md#rotate-keys\n"
    );
    assert_eq!(
        h.error("‸\n", "insertInclude", json!({ "path": "nope.md" })),
        "No page or fragment is at `nope.md`."
    );
    assert_eq!(
        h.error("‸\n", "insertInclude", json!({ "path": "keys.md#nope" })),
        "`keys.md` has no heading with the id `nope`."
    );
    let cycle = h
        .run_as(
            "docs/c0.md",
            "‸\n",
            "insertInclude",
            json!({ "path": "_loop.md" }),
        )
        .unwrap_err();
    assert!(cycle.contains("would make a cycle"), "{cycle}");

    assert_eq!(
        h.after(
            "‸\n",
            "insertSnippet",
            json!({ "address": "code:app.py#main", "title": "The app", "lang": "python" })
        ),
        "@snippet {lang=python, title=\"The app\"}: code:app.py#main\n"
    );
    assert_eq!(
        h.error(
            "‸\n",
            "insertSnippet",
            json!({ "address": "code:app.py#nope" })
        ),
        "`app.py` has no region `nope`. Its regions are `main`."
    );
    assert_eq!(
        h.error("‸\n", "insertSnippet", json!({ "address": "repo:x.py" })),
        "`repo` isn't a source. The sources are `code`."
    );
}

#[test]
fn images_are_inserted() {
    let mut h = Harness::new();
    assert_eq!(
        h.after(
            "‸\n",
            "insertImage",
            json!({ "path": "img/logo.png", "alt": "The [Quill] logo", "attributes": { "height": 40, "width": 600 } })
        ),
        "![The \\[Quill\\] logo](img/logo.png){width=600, height=40}\n"
    );
    let error = h.error(
        "‸\n",
        "insertImage",
        json!({ "path": "img/logo.png", "alt": "Logo", "attributes": { "size": 1 } }),
    );
    assert_eq!(
        error,
        "`size` isn't an image attribute. The image attributes are `width`, `height`."
    );
    let error = h.error(
        "‸\n",
        "insertImage",
        json!({ "path": "img/gone.png", "alt": "Logo" }),
    );
    assert!(
        error.starts_with("The edit would make a problem:"),
        "{error}"
    );
    let error = h.error(
        "‸\n",
        "insertImage",
        json!({ "path": "img/logo.png", "alt": "Logo", "attributes": { "width": "wide" } }),
    );
    assert!(
        error.starts_with("The edit would make a problem:"),
        "{error}"
    );
}

#[test]
fn widgets_are_inserted_in_their_declared_form() {
    let mut h = Harness::new();
    let done = h
        .run(
            "‸\n",
            "insertWidget",
            json!({ "name": "quill-compare", "attributes": { "highlight": true } }),
        )
        .unwrap();
    assert_eq!(
        done.after,
        ".Title\n@quill-compare {highlight=true}:\nWrite the content here.\n@end\n"
    );
    assert_eq!(done.selected.as_deref(), Some("Title"));
    assert_eq!(
        h.after(
            "‸\n",
            "insertWidget",
            json!({ "name": "quill-lab", "primary": "first-sync", "attributes": { "note": "Try it", "size": "small" } })
        ),
        "@quill-lab {size=small, note=\"Try it\"}: first-sync\n"
    );
    assert_eq!(
        h.after(
            "‸\n",
            "insertWidget",
            json!({ "name": "quill-aside", "primary": "An aside." })
        ),
        "@quill-aside: An aside.\n"
    );
    assert_eq!(
        h.after("‸\n", "insertWidget", json!({ "name": "quill-aside" })),
        "@quill-aside:\nWrite the content here.\n@end\n"
    );
    assert_eq!(
        h.error("‸\n", "insertWidget", json!({ "name": "quill-nope" })),
        "`quill-nope` isn't a widget. The widgets are `quill-aside`, `quill-compare`, `quill-lab`."
    );
    assert_eq!(
        h.error(
            "‸\n",
            "insertWidget",
            json!({ "name": "quill-lab", "primary": "x", "attributes": { "colour": "red" } })
        ),
        "`quill-lab` has no attribute `colour`. Its attributes are `size`, `note`."
    );
    let error = h.error(
        "‸\n",
        "insertWidget",
        json!({ "name": "quill-lab", "primary": "x", "attributes": { "size": "huge" } }),
    );
    assert!(
        error.starts_with("The edit would make a problem:"),
        "{error}"
    );
}

// -- Availability -------------------------------------------------------------

#[test]
fn availability_marks_a_section_a_block_or_a_row() {
    let mut h = Harness::new();
    assert_eq!(
        h.after(
            "# T\n\n## Stream‸ing\n\nText.\n",
            "markAvailable",
            json!({ "spec": "self-managed preview 3.4" })
        ),
        "# T\n\n## Streaming\n@available: self-managed preview 3.4\n\nText.\n"
    );
    assert_eq!(
        h.after(
            "# T\n\nIntro.\n\nOnly ‸here.\n",
            "markAvailable",
            json!({ "spec": "sso" })
        ),
        "# T\n\nIntro.\n\n@available: sso\nOnly here.\n"
    );
    let error = h.error(
        "# T\n\nFirst ‸here.\n",
        "markAvailable",
        json!({ "spec": "cloud" }),
    );
    assert!(error.starts_with("The block starts its section"), "{error}");
    let table = "| A | B |\n|---|---|\n| `stream‸` | Streams. |\n";
    assert_eq!(
        h.after(
            table,
            "markAvailable",
            json!({ "spec": "self-managed preview 3.4" })
        ),
        "| A | B |\n|---|---|\n| `stream` {available=\"self-managed preview 3.4\"} | Streams. |\n"
    );
    assert_eq!(
        h.after(table, "markAvailable", json!({ "spec": "cloud" })),
        "| A | B |\n|---|---|\n| `stream` {available=cloud} | Streams. |\n"
    );
    assert_eq!(
        h.error(
            "| A‸ | B |\n|---|---|\n| x | y |\n",
            "markAvailable",
            json!({ "spec": "cloud" })
        ),
        "A table's header row can't have availability."
    );
    let error = h.error("# ‸T\n", "markAvailable", json!({ "spec": "cloud,," }));
    assert!(
        error.starts_with("`cloud,,` isn't an availability spec"),
        "{error}"
    );
    assert!(error.contains("The features are `sso`."), "{error}");
    let error = h.error("# ‸T\n", "markAvailable", json!({ "spec": "mars" }));
    assert!(
        error.starts_with("The edit would make a problem:"),
        "{error}"
    );
}

#[test]
fn a_list_items_block_takes_availability_indented() {
    let mut h = Harness::new();
    assert_eq!(
        h.after(
            "- One.\n\n  Two ‸here.\n",
            "markAvailable",
            json!({ "spec": "cloud" })
        ),
        "- One.\n\n  @available: cloud\n  Two here.\n"
    );
}

#[test]
fn the_frontmatter_changes_only_its_own_key() {
    let mut h = Harness::new();
    let page = "---\ntitle: Install # the title\n\ndescription: How.\n---\n\n‸Text.\n";
    assert_eq!(
        h.after(
            page,
            "setPageAvailable",
            json!({ "spec": "cloud, self-managed 3.3" })
        ),
        "---\ntitle: Install # the title\n\ndescription: How.\navailable: cloud, self-managed 3.3\n---\n\nText.\n"
    );
    assert_eq!(
        h.after(
            "---\ntitle: T\navailable: \"cloud\"\ndescription: D.\n---\n‸Text.\n",
            "setPageAvailable",
            json!({ "spec": "sso" })
        ),
        "---\ntitle: T\navailable: sso\ndescription: D.\n---\nText.\n"
    );
    assert_eq!(
        h.after(
            page,
            "setPageVariant",
            json!({ "dimension": "pm", "value": "pnpm" })
        ),
        "---\ntitle: Install # the title\n\ndescription: How.\nvariant:\n  pm: pnpm\n---\n\nText.\n"
    );
    let variant = "---\ntitle: T\nvariant:\n    deployment: cloud\n# a comment\n---\n‸Text.\n";
    assert_eq!(
        h.after(
            variant,
            "setPageVariant",
            json!({ "dimension": "pm", "value": "npm" })
        ),
        "---\ntitle: T\nvariant:\n    deployment: cloud\n    pm: npm\n# a comment\n---\nText.\n"
    );
    assert_eq!(
        h.after(
            variant,
            "setPageVariant",
            json!({ "dimension": "deployment", "value": "self-managed" })
        ),
        "---\ntitle: T\nvariant:\n    deployment: self-managed\n# a comment\n---\nText.\n"
    );
    assert_eq!(
        h.after(
            "---\ntitle: T\nvariant: {deployment: cloud}\n---\n‸Text.\n",
            "setPageVariant",
            json!({ "dimension": "pm", "value": "yarn" })
        ),
        "---\ntitle: T\nvariant: {deployment: cloud, pm: yarn}\n---\nText.\n"
    );
    assert_eq!(
        h.after("‸Text.\n", "setPageAvailable", json!({ "spec": "cloud" })),
        "---\navailable: cloud\n---\nText.\n"
    );
    assert_eq!(
        h.error(
            page,
            "setPageVariant",
            json!({ "dimension": "pm", "value": "bun" })
        ),
        "`bun` isn't a value of `pm`. Its values are `npm`, `pnpm`, `yarn`."
    );
    let error = h.error(page, "setPageAvailable", json!({ "spec": "mars" }));
    assert!(
        error.starts_with("The edit would make a problem:"),
        "{error}"
    );
    let fragment = h
        .run_as(
            "docs/_part.md",
            "‸Text.\n",
            "setPageAvailable",
            json!({ "spec": "cloud" }),
        )
        .unwrap_err();
    assert!(
        fragment.starts_with("A fragment has no frontmatter keys"),
        "{fragment}"
    );
}

// -- Links, phrases, and images ---------------------------------------------

#[test]
fn selections_are_linked() {
    let mut h = Harness::new();
    assert_eq!(
        h.after(
            "See «the keys» page.\n",
            "linkSelection",
            json!({ "destination": "keys.md#rotate-keys" })
        ),
        "See [the keys](keys.md#rotate-keys) page.\n"
    );
    assert_eq!(
        h.after(
            "See *«the keys»* page.\n",
            "linkSelection",
            json!({ "destination": "https://example.com/a b" })
        ),
        "See *[the keys](<https://example.com/a b>)* page.\n"
    );
    assert_eq!(
        h.error(
            "See «the *keys» page*.\n",
            "linkSelection",
            json!({ "destination": "keys.md" })
        ),
        "The selection cuts across formatting or a link. Select plain text to link."
    );
    assert_eq!(
        h.error(
            "See «the [keys](keys.md)» page.\n",
            "linkSelection",
            json!({ "destination": "keys.md" })
        ),
        "The selection cuts across formatting or a link. Select plain text to link."
    );
    assert_eq!(
        h.error(
            "See «the keys» page.\n",
            "linkSelection",
            json!({ "destination": "gone.md" })
        ),
        "No page is at `gone.md`."
    );
    assert_eq!(
        h.error(
            "See «the keys» page.\n",
            "linkSelection",
            json!({ "destination": "_frag.md" })
        ),
        "`_frag.md` is a fragment; link to a page that includes it."
    );
    assert_eq!(
        h.error(
            "See «the keys» page.\n",
            "linkSelection",
            json!({ "destination": "keys.md#nope" })
        ),
        "`keys.md` has no heading with the id `nope`."
    );
    // An id a fragment the target includes brings in.
    assert_eq!(
        h.after(
            "See «setup».\n",
            "linkSelection",
            json!({ "destination": "guide/setup.md#from-a-fragment" })
        ),
        "See [setup](guide/setup.md#from-a-fragment).\n"
    );
}

#[test]
fn links_and_phrases_are_inserted_at_the_cursor() {
    let mut h = Harness::new();
    assert_eq!(
        h.after(
            "See ‸ for more.\n",
            "insertLink",
            json!({ "destination": "keys.md" })
        ),
        "See [](keys.md) for more.\n"
    );
    assert_eq!(
        h.error(
            "See ‸ for more.\n",
            "insertLink",
            json!({ "destination": "https://example.com" })
        ),
        "A link to a URL needs text: select the text to link instead."
    );
    assert_eq!(
        h.error(
            "See `co‸de` here.\n",
            "insertLink",
            json!({ "destination": "keys.md" })
        ),
        "The cursor is in a link, a phrase, or code; put the link outside it."
    );
    assert_eq!(
        h.after(
            "Use ‸ today.\n",
            "insertPhrase",
            json!({ "key": "product" })
        ),
        "Use {product} today.\n"
    );
    assert_eq!(
        h.after(
            "# Install‸\n@id: install\n",
            "insertPhrase",
            json!({ "key": "product" })
        ),
        "# Install{product}\n@id: install\n"
    );
    assert_eq!(
        h.error("Use ‸ today.\n", "insertPhrase", json!({ "key": "nope" })),
        "`nope` isn't a phrase. The phrases are `product`."
    );
    assert_eq!(
        h.error("‸\n", "insertPhrase", json!({ "key": "product" })),
        "Put the cursor in text for the phrase."
    );
    // A phrase in a heading without an `@id` makes its id unstable.
    let error = h.error("# Install‸\n", "insertPhrase", json!({ "key": "product" }));
    assert!(
        error.starts_with("The edit would make a problem:"),
        "{error}"
    );
}

#[test]
fn links_change_their_target_and_take_its_title() {
    let mut h = Harness::new();
    let page = "See [the ‸keys](keys.md) page.\n";
    assert_eq!(
        h.after(
            page,
            "setLinkTarget",
            json!({ "destination": "guide/setup.md" })
        ),
        "See [the keys](guide/setup.md) page.\n"
    );
    assert_eq!(
        h.after(
            "See [the ‸keys](<a b.md>) page.\n",
            "setLinkTarget",
            json!({ "destination": "keys.md" })
        ),
        "See [the keys](keys.md) page.\n"
    );
    assert_eq!(
        h.after(page, "useTargetTitle", json!({})),
        "See [](keys.md) page.\n"
    );
    assert_eq!(
        h.error("See [](keys.md‸) page.\n", "useTargetTitle", json!({})),
        "The link already takes its target's title."
    );
    assert_eq!(
        h.error(
            "See [](keys.md‸) page.\n",
            "setLinkTarget",
            json!({ "destination": "https://x.dev" })
        ),
        "A link to a URL needs text: select the text to link instead."
    );
    assert_eq!(
        h.error(
            "See [a ‸site](https://x.dev).\n",
            "useTargetTitle",
            json!({})
        ),
        "Only a link to a page takes its target's title; this one needs its text."
    );
    assert_eq!(
        h.error(
            "See [the ‸keys][k].\n\n[k]: keys.md\n",
            "setLinkTarget",
            json!({ "destination": "keys.md" })
        ),
        "Only a link written `[text](destination)` can be changed here."
    );
}

#[test]
fn images_change_their_width_and_alt() {
    let mut h = Harness::new();
    let page = "![The ‸logo](img/logo.png)\n";
    assert_eq!(
        h.after(page, "setImageWidth", json!({ "width": 600 })),
        "![The logo](img/logo.png){width=600}\n"
    );
    assert_eq!(
        h.after(
            "![The ‸logo](img/logo.png){height=40, width=300}\n",
            "setImageWidth",
            json!({ "width": "600" })
        ),
        "![The logo](img/logo.png){width=600, height=40}\n"
    );
    assert_eq!(
        h.after(page, "setImageAlt", json!({ "alt": "Quill's *logo*" })),
        "![Quill's \\*logo\\*](img/logo.png)\n"
    );
    assert_eq!(
        h.error("Not ‸an image.\n", "setImageAlt", json!({ "alt": "x" })),
        "Put the cursor in an image."
    );
}

// -- The request ---------------------------------------------------------------

#[test]
fn a_stale_version_is_an_error() {
    let mut h = Harness::new();
    let path = h.fixture.path("docs/stale.md");
    let text = "Some text.\n";
    h.fixture.write("docs/stale.md", text);
    h.client.open(&path, 3, text);
    let response = h.client.request(
        "ascribe/edit",
        json!({
            "textDocument": { "uri": support::uri(&path).as_str() },
            "range": { "start": { "line": 0, "character": 2 }, "end": { "line": 0, "character": 2 } },
            "action": "wrapNote",
            "args": {},
            "version": 2,
        }),
    );
    let result = response.response_result.unwrap();
    assert_eq!(
        result["error"],
        "The page changed after the action was chosen. Choose it again."
    );
}

#[test]
fn a_file_outside_the_project_gets_an_error() {
    let mut h = Harness::new();
    let path = h.root().join("README.md");
    let response = h.client.request(
        "ascribe/edit",
        json!({
            "textDocument": { "uri": support::uri(&path).as_str() },
            "range": { "start": { "line": 0, "character": 0 }, "end": { "line": 0, "character": 0 } },
            "action": "insertNote",
        }),
    );
    let result = response.response_result.unwrap();
    assert_eq!(result["error"], "This file isn't a page of the project.");
}

#[test]
fn every_wrap_has_an_unwrap() {
    let mut h = Harness::new();
    let pages = [
        "# T\n\nIntro.\n\nThe ‸text.\n\nAfter.\n",
        "- One.\n\n  Two ‸here.\n",
        "> Quoted ‸text.\n",
        "1. ‸Install.\n2. Run.\n",
    ];
    for page in pages {
        let original = page.replace('‸', "");
        for (wrap, unwrap, args) in [
            ("wrapNote", "unwrapNote", json!({ "type": "tip" })),
            ("wrapDetails", "unwrapDetails", json!({ "title": "More" })),
        ] {
            let wrapped = h.after(page, wrap, args);
            let at = wrapped
                .find(
                    original
                        .lines()
                        .find(|l| l.contains("text") || l.contains("here") || l.contains("Install"))
                        .unwrap()
                        .trim_start_matches(['>', ' ', '-', '1', '.']),
                )
                .unwrap();
            let mut marked = wrapped.clone();
            marked.insert(at + 1, '‸');
            assert_eq!(
                h.after(&marked, unwrap, json!({})),
                original,
                "{wrap} then {unwrap}"
            );
        }
    }
    let list = "1. ‸One.\n2. Two.\n";
    let marked = h.after(list, "makeSteps", json!({}));
    assert_eq!(
        h.after(&marked.replace("One", "O‸ne"), "removeSteps", json!({})),
        list.replace('‸', "")
    );
}
