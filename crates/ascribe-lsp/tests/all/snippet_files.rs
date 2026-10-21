//! A file a `@snippet` shows, changed on disk, against a copy of
//! `examples/tested`: the pages that show it, directly or through a
//! fragment, are checked again, and the client is told they read differently
//! so a preview of one draws it again. A file no snippet shows starts
//! nothing.

#![allow(clippy::expect_used, clippy::panic, clippy::unwrap_used)]

use std::path::Path;

use crate::support::{Client, Fixture, uri};
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
