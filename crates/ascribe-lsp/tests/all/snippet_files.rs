//! A file a `@snippet` shows, changed on disk, against a copy of
//! `examples/tested`: the pages that show it, directly or through a
//! fragment, are checked again, and the client is told they read differently
//! so a preview of one draws it again. A file no snippet shows starts
//! nothing. A problem with its tags is shown on it too, once, while a page
//! shows it.

#![allow(clippy::expect_used, clippy::panic, clippy::unwrap_used)]

use std::path::Path;

use crate::support::{Client, Fixture, slug, uri};
use lsp_types::FileChangeType;
use serde_json::json;

const CODE: &str = "code/examples/greet.ts";

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

/// A copy of `examples/tested`, with a fragment that shows the same region
/// and a page that includes it.
fn tested() -> Fixture {
    let source = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/tested");
    let f = Fixture::new("", &[]);
    copy_dir(&source, &f.root());
    f.write(
        "docs/_greet.md",
        "## Greet\n\n@snippet: code:examples/greet.ts#greet\n",
    );
    f.write(
        "docs/again.md",
        "---\ntitle: Again\n---\n\n@include: _greet.md\n",
    );
    f
}

fn start(f: &Fixture) -> Client {
    let mut client = Client::start(&f.root());
    client.settle();
    client
}

fn read(f: &Fixture, rel: &str) -> String {
    std::fs::read_to_string(f.path(rel)).expect("a file")
}

/// Whether a file's diagnostics include `slug`.
fn has(client: &Client, path: &Path, slug: &str) -> bool {
    client.codes(path).iter().any(|code| code == slug)
}

/// The pages each `ascribe/preview/changed` since `from` named, by file name.
fn changed_since(client: &Client, from: usize) -> Vec<Vec<String>> {
    client.notifications[from..]
        .iter()
        .filter(|n| n.method == "ascribe/preview/changed")
        .map(|n| {
            n.params["pages"]
                .as_array()
                .unwrap()
                .iter()
                .map(|uri| uri.as_str().unwrap().rsplit('/').next().unwrap().to_owned())
                .collect()
        })
        .collect()
}

fn preview_html(client: &mut Client, page: &Path) -> String {
    let result = client
        .request(
            "ascribe/preview",
            json!({ "textDocument": { "uri": uri(page).as_str() } }),
        )
        .response_result
        .expect("the preview");
    result["page"]["html"]
        .as_str()
        .unwrap_or_default()
        .to_owned()
}

#[test]
fn the_problems_follow_an_edit_to_a_regions_tags_on_disk_in_every_page_that_shows_it() {
    let f = tested();
    let page = f.path("docs/index.md");
    let fragment = f.path("docs/_greet.md");
    let code = f.path(CODE);
    let mut client = start(&f);
    for file in [&page, &fragment] {
        assert!(!has(&client, file, "snippet-region-missing"), "{file:?}");
    }

    // The region's tags removed.
    let text = read(&f, CODE);
    let untagged: String = text
        .lines()
        .filter(|line| !line.contains(":snippet-"))
        .map(|line| format!("{line}\n"))
        .collect();
    let before = client.notifications.len();
    f.write(CODE, &untagged);
    client.watched(&[(&code, FileChangeType::CHANGED)]);
    client.settle();
    for file in [&page, &fragment] {
        assert!(
            has(&client, file, "snippet-region-missing"),
            "{file:?}: {:?}",
            client.codes(file)
        );
    }
    // The pages that show it, the one through the fragment included.
    assert_eq!(
        changed_since(&client, before),
        [["again.md".to_owned(), "index.md".to_owned()]]
    );

    // And back.
    f.write(CODE, &text);
    client.watched(&[(&code, FileChangeType::CHANGED)]);
    client.settle();
    for file in [&page, &fragment] {
        assert!(!has(&client, file, "snippet-region-missing"), "{file:?}");
    }
}

#[test]
fn the_preview_shows_the_code_as_it_is_on_disk() {
    let f = tested();
    let page = f.path("docs/again.md");
    let code = f.path(CODE);
    let mut client = start(&f);
    assert!(preview_html(&mut client, &page).contains("Hello, "));

    f.write(CODE, &read(&f, CODE).replace("Hello, ", "Howdy, "));
    client.watched(&[(&code, FileChangeType::CHANGED)]);
    client.settle();
    let html = preview_html(&mut client, &page);
    assert!(
        html.contains("Howdy, ") && !html.contains("Hello, "),
        "{html}"
    );
}

#[test]
fn a_deleted_file_is_missing_and_one_created_again_is_shown() {
    let f = tested();
    let page = f.path("docs/index.md");
    let code = f.path(CODE);
    let mut client = start(&f);
    let text = read(&f, CODE);

    f.remove(CODE);
    client.watched(&[(&code, FileChangeType::DELETED)]);
    client.settle();
    assert!(has(&client, &page, "snippet-file-missing"));

    f.write(CODE, &text);
    client.watched(&[(&code, FileChangeType::CREATED)]);
    client.settle();
    assert!(!has(&client, &page, "snippet-file-missing"));
}

#[test]
fn deleting_the_folder_the_code_is_in_checks_its_pages_again() {
    let f = tested();
    let page = f.path("docs/index.md");
    let fragment = f.path("docs/_greet.md");
    let folder = f.path("code/examples");
    let mut client = start(&f);
    let saved = f.root().with_extension("saved");
    copy_dir(&folder, &saved);
    let before = client.notifications.len();

    // A branch switch, or `rm -rf`: the watcher reports the folder alone.
    std::fs::remove_dir_all(&folder).expect("rm -rf");
    client.watched(&[(&folder, FileChangeType::DELETED)]);
    client.settle();
    for file in [&page, &fragment] {
        assert!(
            has(&client, file, "snippet-file-missing"),
            "{file:?}: {:?}",
            client.codes(file)
        );
    }
    assert_eq!(
        changed_since(&client, before),
        [["again.md".to_owned(), "index.md".to_owned()]]
    );

    // And back, reported as the folder created.
    copy_dir(&saved, &folder);
    std::fs::remove_dir_all(&saved).expect("rm -rf");
    client.watched(&[(&folder, FileChangeType::CREATED)]);
    client.settle();
    for file in [&page, &fragment] {
        assert!(!has(&client, file, "snippet-file-missing"), "{file:?}");
    }
}

#[test]
fn a_burst_of_changes_checks_each_page_once() {
    let f = tested();
    let page = f.path("docs/index.md");
    let mut client = start(&f);
    let published = client.publications(&page).len();
    let before = client.notifications.len();

    // Both files the page shows change, as a branch switch changes them,
    // each reported more than once.
    f.write(CODE, "// no region\n");
    f.write("code/examples/greet.out.txt", "Hi!\n");
    let output = f.path("code/examples/greet.out.txt");
    let code = f.path(CODE);
    client.watched(&[
        (&code, FileChangeType::CHANGED),
        (&output, FileChangeType::CHANGED),
        (&code, FileChangeType::CHANGED),
    ]);
    client.settle();
    assert_eq!(client.publications(&page).len(), published + 1);
    assert!(has(&client, &page, "snippet-region-missing"));
    assert_eq!(
        changed_since(&client, before),
        [["again.md".to_owned(), "index.md".to_owned()]]
    );
}

#[test]
fn a_file_no_snippet_shows_starts_nothing() {
    let f = tested();
    let page = f.path("docs/index.md");
    let mut client = start(&f);
    let published = client.publications(&page).len();
    let before = client.notifications.len();

    let test = "code/test/greet.test.mjs";
    f.write(test, &format!("{}\n// changed\n", read(&f, test)));
    client.watched(&[(&f.path(test), FileChangeType::CHANGED)]);
    client.settle();
    assert_eq!(client.publications(&page).len(), published);
    assert!(changed_since(&client, before).is_empty());
}

/// `greet.ts` with its region's `:snippet-end:` gone, on disk and reported:
/// both pages that show it, one through a fragment, have `snippet-tags`.
fn unclose(f: &Fixture, client: &mut Client) -> String {
    let text = read(f, CODE);
    let unclosed: String = text
        .lines()
        .filter(|line| !line.contains(":snippet-end:"))
        .map(|line| format!("{line}\n"))
        .collect();
    f.write(CODE, &unclosed);
    client.watched(&[(&f.path(CODE), FileChangeType::CHANGED)]);
    client.settle();
    text
}

#[test]
fn a_snippet_tags_problem_is_shown_once_on_the_code_file_at_the_tag() {
    let f = tested();
    let page = f.path("docs/index.md");
    let fragment = f.path("docs/_greet.md");
    let code = f.path(CODE);
    let mut client = start(&f);
    assert!(client.diagnostics(&code).is_empty());

    unclose(&f, &mut client);
    let on_page = client
        .diagnostics(&page)
        .into_iter()
        .find(|d| slug(d) == "snippet-tags")
        .expect("the page's own");
    let on_code = client.diagnostics(&code);
    // Once, though two files show it.
    assert_eq!(on_code.len(), 1, "{on_code:#?}");
    let d = &on_code[0];
    assert_eq!(slug(d), "snippet-tags");
    // `:snippet-start: greet`, the tag never closed, is on the first line.
    assert_eq!(d.range.start.line, 0);
    assert_eq!(d.message, on_page.message);
    assert_eq!(d.severity, on_page.severity);
    // It names each `@snippet` that shows the file.
    let shown_by: Vec<String> = d
        .related_information
        .iter()
        .flatten()
        .map(|r| {
            r.location
                .uri
                .as_str()
                .rsplit('/')
                .next()
                .unwrap()
                .to_owned()
        })
        .collect();
    assert_eq!(shown_by, ["_greet.md", "index.md"], "{d:#?}");
    // The pages keep their own.
    for file in [&page, &fragment] {
        assert!(has(&client, file, "snippet-tags"), "{file:?}");
    }
}

#[test]
fn fixing_the_tags_on_disk_clears_the_code_files_problem_and_the_pages() {
    let f = tested();
    let page = f.path("docs/index.md");
    let code = f.path(CODE);
    let mut client = start(&f);
    let text = unclose(&f, &mut client);
    assert!(has(&client, &code, "snippet-tags"));

    f.write(CODE, &text);
    client.watched(&[(&code, FileChangeType::CHANGED)]);
    client.settle();
    assert!(client.diagnostics(&code).is_empty());
    assert!(!has(&client, &page, "snippet-tags"));
}

#[test]
fn removing_the_last_snippet_that_shows_the_file_clears_its_problem() {
    let f = tested();
    let page = f.path("docs/index.md");
    let fragment = f.path("docs/_greet.md");
    let code = f.path(CODE);
    let mut client = start(&f);
    unclose(&f, &mut client);
    assert!(has(&client, &code, "snippet-tags"));

    let without = |path: &Path| -> String {
        std::fs::read_to_string(path)
            .unwrap()
            .lines()
            .filter(|line| !line.contains("code:examples/greet.ts"))
            .map(|line| format!("{line}\n"))
            .collect()
    };
    // One goes: the other still shows it.
    let text = without(&fragment);
    client.open(&fragment, 1, &read(&f, "docs/_greet.md"));
    client.replace(&fragment, 2, &text);
    client.settle();
    assert_eq!(client.diagnostics(&code).len(), 1);
    let related = client.diagnostics(&code)[0]
        .related_information
        .clone()
        .unwrap_or_default();
    assert_eq!(related.len(), 1, "{related:#?}");

    // The last goes.
    let text = without(&page);
    client.open(&page, 1, &read(&f, "docs/index.md"));
    client.replace(&page, 2, &text);
    client.settle();
    assert!(client.diagnostics(&code).is_empty());
}
