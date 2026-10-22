//! `ascribe.baseline` in the editor (#790): the advice it lists isn't
//! published, the hover on its line says it's held back, an entry that
//! matches nothing is published on the file, the file is followed on disk
//! and as it's typed, and a held finding's quick fix moves it to an
//! `@intended`.

#![allow(clippy::expect_used, clippy::panic, clippy::unwrap_used)]

use std::path::Path;

use ascribe_check::identity::fingerprint;
use ascribe_core::{LineCol, LineIndex, Span};
use lsp_types::{Diagnostic, FileChangeType, Position};
use serde_json::{Value, json};

use crate::support::{Client, Fixture, MODEL, slug, uri};

const PAGE: &str = "---\ntitle: Home\n---\n\n<!-- Ask Sam about the prices. -->\n\nText.\n\n<!-- Check the table. -->\n";

const HEADER: &str = "# ascribe baseline 1\n# check fingerprint order file\n";

fn project() -> Fixture {
    Fixture::new(MODEL, &[("docs/index.md", PAGE)])
}

fn offset(text: &str, at: Position) -> usize {
    LineIndex::new(text)
        .offset(LineCol {
            line: at.line,
            col: at.character,
        })
        .unwrap()
}

/// The baseline's line for `d`, a diagnostic published for `docs/index.md`
/// over `text`: what `ascribe baseline write` writes for it.
fn entry(d: &Diagnostic, text: &str) -> String {
    let span = Span::new(offset(text, d.range.start), offset(text, d.range.end));
    format!("{} {} 0 docs/index.md\n", slug(d), fingerprint(text, span))
}

/// The page's two comments, in order, as published with no baseline.
fn comments(client: &Client, page: &Path) -> Vec<Diagnostic> {
    let mut found: Vec<Diagnostic> = client
        .diagnostics(page)
        .into_iter()
        .filter(|d| slug(d) == "html-comment")
        .collect();
    found.sort_by_key(|d| d.range.start.line);
    assert_eq!(found.len(), 2, "{found:?}");
    found
}

fn hover(client: &mut Client, page: &Path, line: u32) -> Option<String> {
    let result = client
        .request(
            "textDocument/hover",
            json!({
                "textDocument": { "uri": uri(page).as_str() },
                "position": { "line": line, "character": 3 },
            }),
        )
        .response_result
        .expect("a hover");
    result["contents"]["value"].as_str().map(str::to_owned)
}

/// The quick fixes asked for at `line` of the page, with no diagnostic
/// there to send.
fn actions_at(client: &mut Client, page: &Path, line: u32) -> Vec<Value> {
    client
        .request(
            "textDocument/codeAction",
            json!({
                "textDocument": { "uri": uri(page).as_str() },
                "range": { "start": { "line": line, "character": 0 }, "end": { "line": line, "character": 0 } },
                "context": { "diagnostics": [] }
            }),
        )
        .response_result
        .expect("actions")
        .as_array()
        .cloned()
        .unwrap_or_default()
}

/// The text an action's edit of `path` makes of `text`.
fn applied(action: &Value, path: &Path, text: &str) -> String {
    let edits = action["edit"]["changes"][uri(path).as_str()]
        .as_array()
        .unwrap_or_else(|| panic!("no edit of {}: {action}", path.display()));
    let edits: Vec<ascribe_core::TextEdit> = edits
        .iter()
        .map(|e| {
            let at = |p: &Value| offset(text, serde_json::from_value(p.clone()).unwrap());
            ascribe_core::TextEdit::replace(
                at(&e["range"]["start"])..at(&e["range"]["end"]),
                e["newText"].as_str().unwrap(),
            )
        })
        .collect();
    ascribe_core::apply_edits(text, &edits).unwrap()
}

#[test]
fn what_the_baseline_lists_isnt_published_and_the_hover_says_so() {
    let f = project();
    let mut client = Client::start(&f.root());
    client.settle();
    let page = f.path("docs/index.md");
    let baseline = f.path("ascribe.baseline");
    let [first, second] = <[Diagnostic; 2]>::try_from(comments(&client, &page)).unwrap();

    // `ascribe baseline write` writes the file on disk, and the watcher
    // says so.
    f.write(
        "ascribe.baseline",
        &format!("{HEADER}{}", entry(&first, PAGE)),
    );
    client.watched(&[(&baseline, FileChangeType::CREATED)]);
    client.settle();
    let left = client.diagnostics(&page);
    assert_eq!(left.len(), 1, "{left:?}");
    assert_eq!(left[0].range, second.range, "the one it doesn't list");
    assert!(
        client.diagnostics(&baseline).is_empty(),
        "its entry matches"
    );

    // It isn't a diagnostic, so the hover on its line says it's held back.
    let said = hover(&mut client, &page, first.range.start.line).expect("a hover");
    assert!(
        said.contains("`ascribe.baseline` holds back this advice") && said.contains(&first.message),
        "{said}"
    );
    assert_eq!(hover(&mut client, &page, second.range.start.line), None);

    // Pruned on disk, it holds nothing back.
    f.write("ascribe.baseline", HEADER);
    client.watched(&[(&baseline, FileChangeType::CHANGED)]);
    client.settle();
    assert_eq!(client.diagnostics(&page).len(), 2);
    assert_eq!(hover(&mut client, &page, first.range.start.line), None);
}

#[test]
fn an_entry_that_matches_nothing_is_published_on_the_file_as_its_typed() {
    let f = project();
    let page = f.path("docs/index.md");
    let baseline = f.path("ascribe.baseline");
    let stale = "html-comment 0123456789abcdef 0 docs/index.md\n";
    f.write("ascribe.baseline", &format!("{HEADER}{stale}"));
    let mut client = Client::start(&f.root());
    client.settle();
    assert_eq!(client.codes(&baseline), ["baseline-entry-unmatched"]);
    let at = &client.diagnostics(&baseline)[0];
    assert_eq!(at.range.start.line, 2, "at the entry's line");
    assert_eq!(client.diagnostics(&page).len(), 2);

    // Typed in the editor: an entry for the first comment holds it back,
    // and isn't stale.
    let [first, _] = <[Diagnostic; 2]>::try_from(comments(&client, &page)).unwrap();
    let typed = format!("{HEADER}{}", entry(&first, PAGE));
    client.open(&baseline, 1, &format!("{HEADER}{stale}"));
    client.replace(&baseline, 2, &typed);
    client.settle();
    assert!(client.codes(&baseline).is_empty());
    assert_eq!(client.diagnostics(&page).len(), 1);

    // A line that doesn't parse is an error at it, and nothing is held
    // back until it's fixed.
    client.replace(
        &baseline,
        3,
        &format!("{HEADER}html-comment 0 docs/index.md\n"),
    );
    client.settle();
    let wrong = client.diagnostics(&baseline);
    assert_eq!(wrong.len(), 1, "{wrong:?}");
    assert_eq!(wrong[0].range.start.line, 2);
    assert!(
        wrong[0].message.contains("fewer fields"),
        "{}",
        wrong[0].message
    );
    assert_eq!(client.diagnostics(&page).len(), 2);

    // Closed, it's the disk's again.
    client.close(&baseline);
    client.settle();
    assert_eq!(client.codes(&baseline), ["baseline-entry-unmatched"]);
}

#[test]
fn a_held_findings_quick_fix_moves_it_to_an_intended_at_its_block() {
    let f = project();
    let page = f.path("docs/index.md");
    let baseline = f.path("ascribe.baseline");
    let mut client = Client::start(&f.root());
    client.settle();
    let [first, second] = <[Diagnostic; 2]>::try_from(comments(&client, &page)).unwrap();
    let listed = format!("{HEADER}{}{}", entry(&first, PAGE), entry(&second, PAGE));
    f.write("ascribe.baseline", &listed);
    client.watched(&[(&baseline, FileChangeType::CREATED)]);
    client.settle();
    assert!(client.diagnostics(&page).is_empty());

    let actions = actions_at(&mut client, &page, first.range.start.line);
    let titles: Vec<&str> = actions.iter().filter_map(|a| a["title"].as_str()).collect();
    assert_eq!(
        titles,
        ["Move out of the baseline, mark as intended, and write why"]
    );
    let moved = &actions[0];
    let fixed = applied(moved, &page, PAGE);
    assert_eq!(
        fixed,
        PAGE.replace(
            "<!-- Ask Sam",
            "@intended {check=html-comment}: why this is intended\n<!-- Ask Sam"
        ),
    );
    let pruned = applied(moved, &baseline, &listed);
    assert_eq!(pruned, format!("{HEADER}{}", entry(&second, PAGE)));

    // Applied, the finding is acknowledged at its block, out of the
    // baseline, and nothing is stale.
    f.write("docs/index.md", &fixed);
    f.write("ascribe.baseline", &pruned);
    client.watched(&[
        (&page, FileChangeType::CHANGED),
        (&baseline, FileChangeType::CHANGED),
    ]);
    client.settle();
    let codes: Vec<String> = client.diagnostics(&page).iter().map(slug).collect();
    assert!(!codes.iter().any(|c| c == "html-comment"), "{codes:?}");
    assert!(client.diagnostics(&baseline).is_empty());
}

#[test]
fn an_entry_for_a_check_the_editor_doesnt_run_isnt_stale() {
    let f = Fixture::new(
        MODEL,
        &[("docs/index.md", PAGE), ("docs/images/unused.png", "png")],
    );
    // What `ascribe baseline write` writes: every advice of the whole check,
    // `image-unused` among them, which the editor doesn't run.
    let project = ascribe_check::Project::load(&f.path("ascribe.toml")).unwrap();
    let found = ascribe_check::diagnose(&project, &[]).unwrap();
    let written = ascribe_check::baseline::write(&project, None, &found.diagnostics);
    assert!(written.text.contains("image-unused "), "{}", written.text);
    f.write("ascribe.baseline", &written.text);
    let mut client = Client::start(&f.root());
    client.settle();
    let baseline = f.path("ascribe.baseline");
    assert!(
        client.diagnostics(&baseline).is_empty(),
        "{:?}",
        client.diagnostics(&baseline)
    );
    assert!(client.diagnostics(&f.path("docs/index.md")).is_empty());
}
