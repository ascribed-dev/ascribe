//! The scripted scenarios of phase 15's acceptance criteria, over an in-memory
//! connection.

#![allow(clippy::expect_used, clippy::panic, clippy::unwrap_used)]

mod support;

use std::sync::mpsc;
use std::sync::{Arc, Mutex};

use lsp_types::{FileChangeType, PositionEncodingKind};
use support::{Client, Fixture, MODEL, Setup, edit, slug};
use tessera_lsp::{Exit, Options};

const PAGE: &str = "---\ntitle: Home\n---\n# Home\n\n@include: _setup.md#install\n";
const FRAGMENT: &str = "# Install\n@id: install\n\nRun it.\n";

fn project() -> Fixture {
    Fixture::new(
        MODEL,
        &[("docs/index.md", PAGE), ("docs/_setup.md", FRAGMENT)],
    )
}

#[test]
fn a_clean_project_publishes_nothing() {
    let f = project();
    let mut client = Client::start(&f.root());
    client.settle();
    assert!(client.log.is_empty(), "{:?}", client.log);
    assert_eq!(client.shutdown(), Exit::Clean);
}

#[test]
fn the_initialize_result_advertises_only_what_is_implemented() {
    let f = project();
    let client = Client::start(&f.root());
    let caps = &client.initialize_result["capabilities"];
    assert_eq!(caps["positionEncoding"], "utf-16");
    assert_eq!(caps["textDocumentSync"]["change"], 2);
    assert_eq!(caps["textDocumentSync"]["openClose"], true);
    let legend = &caps["semanticTokensProvider"]["legend"];
    assert_eq!(legend["tokenTypes"][6], "tesseraTitle");
    assert_eq!(legend["tokenModifiers"][0], "unknown");
    let mut keys: Vec<&str> = caps
        .as_object()
        .expect("an object")
        .keys()
        .map(String::as_str)
        .collect();
    keys.sort_unstable();
    assert_eq!(
        keys,
        [
            "positionEncoding",
            "semanticTokensProvider",
            "textDocumentSync"
        ]
    );
    // The file watcher is registered dynamically.
    assert!(
        client
            .requests_from_server
            .iter()
            .any(|r| r.method == "client/registerCapability")
            || client.requests_from_server.is_empty()
    );
}

// -- Acceptance: editing an open fragment updates an including page ---------

#[test]
fn editing_an_open_fragment_updates_the_including_page() {
    let f = project();
    let (page, fragment) = (f.path("docs/index.md"), f.path("docs/_setup.md"));
    let mut client = Client::start(&f.root());
    client.open(&fragment, 1, FRAGMENT);
    client.settle();
    assert!(client.codes(&page).is_empty());

    // Rename the fragment's id: the include in the unopened page is now broken.
    client.change(&fragment, 2, vec![edit((1, 5), (1, 12), "setup")]);
    client.settle();
    assert_eq!(client.codes(&page), ["include-id-missing"]);
    let published = client.publications(&page);
    assert_eq!(published.last().expect("a publication").version, None);

    // And back.
    client.change(&fragment, 3, vec![edit((1, 5), (1, 10), "install")]);
    client.settle();
    assert!(client.codes(&page).is_empty());
    assert_eq!(client.shutdown(), Exit::Clean);
}

// -- Acceptance: an unopened fragment changed on disk -----------------------

#[test]
fn changing_an_unopened_fragment_on_disk_updates_the_including_page() {
    let f = project();
    let page = f.path("docs/index.md");
    let mut client = Client::start(&f.root());
    client.settle();
    assert!(client.codes(&page).is_empty());

    f.write("docs/_setup.md", "# Install\n@id: setup\n\nRun it.\n");
    client.watched(&[(&f.path("docs/_setup.md"), FileChangeType::CHANGED)]);
    client.settle();
    assert_eq!(client.codes(&page), ["include-id-missing"]);

    f.write("docs/_setup.md", FRAGMENT);
    client.watched(&[(&f.path("docs/_setup.md"), FileChangeType::CHANGED)]);
    client.settle();
    assert!(client.codes(&page).is_empty());
}

#[test]
fn an_open_buffer_wins_over_the_file_on_disk() {
    let f = project();
    let (page, fragment) = (f.path("docs/index.md"), f.path("docs/_setup.md"));
    let mut client = Client::start(&f.root());
    client.open(&fragment, 1, "# Install\n@id: setup\n");
    client.settle();
    assert_eq!(client.codes(&page), ["include-id-missing"]);
    // The disk changes underneath the open buffer: the buffer still wins.
    f.write("docs/_setup.md", FRAGMENT);
    client.watched(&[(&fragment, FileChangeType::CHANGED)]);
    client.settle();
    assert_eq!(client.codes(&page), ["include-id-missing"]);
    // Closing the buffer reverts to the disk.
    client.close(&fragment);
    client.settle();
    assert!(client.codes(&page).is_empty());
}

// -- Acceptance: a widget declared in ascribe.toml --------------------------

const WIDGET_PAGE: &str = "---\ntitle: W\n---\n# W\n\n@my-callout: Careful now.\n";
const WIDGET_MODEL: &str = "spec = \"0.1\"\n\n[project]\ncontent-root = \"docs\"\n\n[widgets.my-callout]\nforms = [\"line\"]\nprimary = \"text\"\nbinding = \"self\"\n";

#[test]
fn declaring_a_widget_in_the_model_changes_an_unopened_file() {
    let f = Fixture::new(MODEL, &[("docs/w.md", WIDGET_PAGE)]);
    let (page, config) = (f.path("docs/w.md"), f.path("ascribe.toml"));
    let mut client = Client::start(&f.root());
    client.settle();
    assert_eq!(client.codes(&page), ["directive-unknown"]);

    // In the editor's buffer.
    client.open(&config, 1, MODEL);
    client.replace(&config, 2, WIDGET_MODEL);
    client.settle();
    assert!(
        client.codes(&page).is_empty(),
        "{:?}",
        client.diagnostics(&page)
    );
    assert!(client.codes(&config).is_empty());

    // Removing it brings the warning back.
    client.replace(&config, 3, MODEL);
    client.settle();
    assert_eq!(client.codes(&page), ["directive-unknown"]);

    // Then on disk, with the buffer closed.
    client.close(&config);
    f.write("ascribe.toml", WIDGET_MODEL);
    client.watched(&[(&config, FileChangeType::CHANGED)]);
    client.settle();
    assert!(client.codes(&page).is_empty());
}

#[test]
fn a_model_that_does_not_load_is_reported_on_ascribe_toml_and_the_last_model_stays() {
    let f = Fixture::new(WIDGET_MODEL, &[("docs/w.md", WIDGET_PAGE)]);
    let (page, config) = (f.path("docs/w.md"), f.path("ascribe.toml"));
    let mut client = Client::start(&f.root());
    client.open(&config, 1, WIDGET_MODEL);
    client.settle();
    assert!(client.codes(&page).is_empty());

    client.replace(&config, 2, "spec = \"0.1\"\n[project\n");
    client.settle();
    assert!(!client.diagnostics(&config).is_empty());
    assert_eq!(
        client.diagnostics(&config)[0].source.as_deref(),
        Some("tessera")
    );
    // The project keeps working with the last model that loaded.
    assert!(client.codes(&page).is_empty());
    client.change(&page, 1, vec![]);

    client.replace(&config, 3, WIDGET_MODEL);
    client.settle();
    assert!(
        client.diagnostics(&config).is_empty(),
        "{:?}",
        client.diagnostics(&config)
    );
}

#[test]
fn changing_the_content_root_reloads_the_project() {
    let f = Fixture::new(
        MODEL,
        &[
            ("docs/a.md", "---\ntitle: A\n---\n[x](nope.md)\n"),
            ("other/b.md", "---\ntitle: B\n---\nfine\n"),
        ],
    );
    let (a, b, config) = (
        f.path("docs/a.md"),
        f.path("other/b.md"),
        f.path("ascribe.toml"),
    );
    let mut client = Client::start(&f.root());
    client.settle();
    assert_eq!(client.codes(&a), ["link-target-missing"]);

    client.open(&config, 1, MODEL);
    client.replace(&config, 2, &MODEL.replace("docs", "other"));
    client.settle();
    // The old content root's files are no longer in the project.
    assert!(client.codes(&a).is_empty());
    assert!(client.codes(&b).is_empty());
    let publications = client.publications(&a);
    let last = publications.last().expect("a clearing publication");
    assert!(last.diagnostics.is_empty());
}

// -- Acceptance: deleting a linked file -------------------------------------

#[test]
fn deleting_a_linked_file_produces_broken_link_diagnostics() {
    let f = Fixture::new(
        MODEL,
        &[
            (
                "docs/index.md",
                "---\ntitle: Home\n---\n[Other](other.md) and ![logo](logo.png)\n",
            ),
            ("docs/other.md", "---\ntitle: Other\n---\n"),
            ("docs/logo.png", "not really a png"),
        ],
    );
    let page = f.path("docs/index.md");
    let mut client = Client::start(&f.root());
    client.settle();
    assert!(client.codes(&page).is_empty());

    f.remove("docs/other.md");
    client.watched(&[(&f.path("docs/other.md"), FileChangeType::DELETED)]);
    client.settle();
    assert_eq!(client.codes(&page), ["link-target-missing"]);

    f.remove("docs/logo.png");
    client.watched(&[(&f.path("docs/logo.png"), FileChangeType::DELETED)]);
    client.settle();
    assert_eq!(
        client.codes(&page),
        ["image-source-missing", "link-target-missing"]
    );

    // Created again: a source and an image.
    f.write("docs/other.md", "---\ntitle: Other\n---\n");
    f.write("docs/logo.png", "again");
    client.watched(&[
        (&f.path("docs/other.md"), FileChangeType::CREATED),
        (&f.path("docs/logo.png"), FileChangeType::CREATED),
    ]);
    client.settle();
    assert!(client.codes(&page).is_empty());
}

#[test]
fn deleting_a_directory_deletes_everything_under_it() {
    let f = Fixture::new(
        MODEL,
        &[
            (
                "docs/index.md",
                "---\ntitle: Home\n---\n[G](guide/a.md) ![i](guide/img/i.png)\n",
            ),
            ("docs/guide/a.md", "---\ntitle: A\n---\n"),
            ("docs/guide/img/i.png", "x"),
        ],
    );
    let page = f.path("docs/index.md");
    let mut client = Client::start(&f.root());
    client.settle();
    assert!(client.codes(&page).is_empty());

    std::fs::remove_dir_all(f.path("docs/guide")).expect("remove");
    client.watched(&[(&f.path("docs/guide"), FileChangeType::DELETED)]);
    client.settle();
    assert_eq!(
        client.codes(&page),
        ["image-source-missing", "link-target-missing"]
    );

    // A directory created with files in it (`git checkout`) is walked.
    f.write("docs/guide/a.md", "---\ntitle: A\n---\n");
    f.write("docs/guide/img/i.png", "x");
    client.watched(&[(&f.path("docs/guide"), FileChangeType::CREATED)]);
    client.settle();
    assert!(client.codes(&page).is_empty());
}

#[test]
fn a_deleted_file_has_its_diagnostics_cleared() {
    let f = Fixture::new(
        MODEL,
        &[("docs/bad.md", "---\ntitle: Bad\n---\n[x](nope.md)\n")],
    );
    let bad = f.path("docs/bad.md");
    let mut client = Client::start(&f.root());
    client.settle();
    assert_eq!(client.codes(&bad), ["link-target-missing"]);
    f.remove("docs/bad.md");
    client.watched(&[(&bad, FileChangeType::DELETED)]);
    client.settle();
    assert!(client.diagnostics(&bad).is_empty());
    assert!(client.publications(&bad).len() >= 2);
}

// -- Acceptance: a slow computation overtaken by a newer edit ---------------

#[test]
fn a_computation_overtaken_by_a_newer_edit_never_publishes() {
    let f = Fixture::new(
        MODEL,
        &[("docs/a.md", "---\ntitle: A\n---\n[x](nope.md)\n")],
    );
    let a = f.path("docs/a.md");
    // Hold the first round (the initial load, which finds the broken link)
    // until the test has sent a newer edit.
    let (arrived_tx, arrived_rx) = mpsc::channel::<()>();
    let (release_tx, release_rx) = mpsc::channel::<()>();
    let release_rx = Arc::new(Mutex::new(release_rx));
    let first = Arc::new(std::sync::atomic::AtomicBool::new(true));
    let options = Options {
        before_publish: Some(Arc::new(move |_| {
            if first.swap(false, std::sync::atomic::Ordering::SeqCst) {
                arrived_tx.send(()).expect("test is listening");
                release_rx.lock().expect("lock").recv().expect("released");
            }
        })),
        ..Options::default()
    };
    let mut client = Client::start_with(
        &f.root(),
        Setup {
            options,
            ..Setup::default()
        },
    );
    arrived_rx
        .recv_timeout(support::TIMEOUT)
        .expect("the first round computed");
    // The newer edit fixes the link. The held round is now stale.
    client.open(&a, 1, "---\ntitle: A\n---\nfixed\n");
    let _ = client.request(
        "textDocument/semanticTokens/full",
        serde_json::json!({ "textDocument": { "uri": "untitled:x" } }),
    );
    release_tx.send(()).expect("release");
    client.settle();

    for publication in client.publications(&a) {
        assert!(
            publication.diagnostics.is_empty(),
            "a stale result was published: {publication:?}"
        );
        assert_eq!(publication.version, Some(1));
    }
    assert!(!client.publications(&a).is_empty());
    assert!(client.codes(&a).is_empty());
}

// -- Acceptance: positions with multi-byte characters ------------------------

const EMOJI_PAGE: &str = "---\ntitle: E\n---\n😀 é [x](nope.md)\n";

#[test]
fn positions_count_utf16_code_units_by_default() {
    let f = Fixture::new(MODEL, &[("docs/e.md", EMOJI_PAGE)]);
    let e = f.path("docs/e.md");
    let mut client = Client::start(&f.root());
    client.settle();
    let d = &client.diagnostics(&e)[0];
    assert_eq!(slug(d), "link-target-missing");
    // `😀 é [x](` is 2 + 1 + 1 + 1 + 4 units before `nope.md` (7 chars wide).
    assert_eq!(d.range.start.line, 3);
    assert_eq!(d.range.start.character, 2 + 1 + 1 + 1 + 4);
    assert_eq!(d.range.end.character, 2 + 1 + 1 + 1 + 4 + 7);
}

#[test]
fn positions_count_bytes_when_the_client_offers_utf8() {
    let f = Fixture::new(MODEL, &[("docs/e.md", EMOJI_PAGE)]);
    let e = f.path("docs/e.md");
    let mut client = Client::start_with(
        &f.root(),
        Setup {
            encodings: Some(vec![
                PositionEncodingKind::UTF16,
                PositionEncodingKind::UTF8,
            ]),
            ..Setup::default()
        },
    );
    assert_eq!(
        client.initialize_result["capabilities"]["positionEncoding"],
        "utf-8"
    );
    client.settle();
    let d = &client.diagnostics(&e)[0];
    // 4 + 1 + 2 + 1 + 4 bytes before `nope.md`.
    assert_eq!(d.range.start.character, 4 + 1 + 2 + 1 + 4);
}

#[test]
fn incremental_edits_after_multi_byte_text_land_where_the_editor_meant() {
    let f = Fixture::new(MODEL, &[("docs/e.md", EMOJI_PAGE)]);
    let e = f.path("docs/e.md");
    let mut client = Client::start(&f.root());
    client.open(&e, 1, EMOJI_PAGE);
    client.settle();
    // Replace `nope` (UTF-16 columns 9..13 on line 3) with `e2`.
    client.change(&e, 2, vec![edit((3, 9), (3, 13), "e2")]);
    client.settle();
    let d = &client.diagnostics(&e)[0];
    assert_eq!(d.range.start.character, 9);
    assert_eq!(d.range.end.character, 9 + "e2.md".len() as u32);
    assert_eq!(
        client.publications(&e).last().expect("one").version,
        Some(2)
    );
}

#[test]
fn a_model_that_does_not_load_at_startup_is_reported_and_the_project_loads_once_it_does() {
    let f = Fixture::new(
        "spec = \"0.1\"\n[project\n",
        &[("docs/a.md", "---\ntitle: A\n---\n[x](nope.md)\n")],
    );
    let (a, config) = (f.path("docs/a.md"), f.path("ascribe.toml"));
    let mut client = Client::start(&f.root());
    client.settle();
    assert!(!client.diagnostics(&config).is_empty());
    assert!(client.diagnostics(&a).is_empty());

    f.write("ascribe.toml", MODEL);
    client.watched(&[(&config, FileChangeType::CHANGED)]);
    client.settle();
    assert!(client.diagnostics(&config).is_empty());
    assert_eq!(client.codes(&a), ["link-target-missing"]);
}

// -- Files and assets ---------------------------------------------------------

#[test]
fn a_created_page_is_picked_up_and_a_renamed_one_moves() {
    let f = Fixture::new(
        MODEL,
        &[("docs/index.md", "---\ntitle: Home\n---\n[N](new.md)\n")],
    );
    let page = f.path("docs/index.md");
    let mut client = Client::start(&f.root());
    client.settle();
    assert_eq!(client.codes(&page), ["link-target-missing"]);
    f.write("docs/new.md", "---\ntitle: New\n---\n");
    client.watched(&[(&f.path("docs/new.md"), FileChangeType::CREATED)]);
    client.settle();
    assert!(client.codes(&page).is_empty());
    // A rename arrives as a deletion and a creation.
    std::fs::rename(f.path("docs/new.md"), f.path("docs/renamed.md")).expect("rename");
    client.watched(&[
        (&f.path("docs/new.md"), FileChangeType::DELETED),
        (&f.path("docs/renamed.md"), FileChangeType::CREATED),
    ]);
    client.settle();
    assert_eq!(client.codes(&page), ["link-target-missing"]);
}

#[test]
fn the_file_level_checks_see_the_layered_files_like_the_source_index() {
    // An image that appears on disk is seen by the checks that report a missing
    // image (file level) and by the source index (page level) alike.
    let f = Fixture::new(
        MODEL,
        &[("docs/index.md", "---\ntitle: Home\n---\n![alt](pic.png)\n")],
    );
    let page = f.path("docs/index.md");
    let mut client = Client::start(&f.root());
    client.settle();
    assert_eq!(client.codes(&page), ["image-source-missing"]);
    f.write("docs/pic.png", "x");
    client.watched(&[(&f.path("docs/pic.png"), FileChangeType::CREATED)]);
    client.settle();
    assert!(client.codes(&page).is_empty());
}

#[test]
fn without_a_project_the_server_stays_quiet_and_picks_one_up_when_it_appears() {
    let dir = tempfile::tempdir().expect("dir");
    let root = dir.path().canonicalize().expect("real");
    let mut client = Client::start(&root);
    client.settle();
    assert!(client.log.is_empty());
    std::fs::create_dir_all(root.join("docs")).expect("mkdir");
    std::fs::write(root.join("docs/a.md"), "---\ntitle: A\n---\n[x](nope.md)\n").expect("write");
    std::fs::write(root.join("ascribe.toml"), MODEL).expect("write");
    client.watched(&[(&root.join("ascribe.toml"), FileChangeType::CREATED)]);
    client.settle();
    assert_eq!(
        client.codes(&root.join("docs/a.md")),
        ["link-target-missing"]
    );
}

// -- Robustness ---------------------------------------------------------------

#[test]
fn a_panic_while_handling_a_request_does_not_take_the_server_down() {
    let f = project();
    let options = Options {
        before_request: Some(Arc::new(|method| {
            if method == "textDocument/semanticTokens/range" {
                panic!("boom");
            }
        })),
        ..Options::default()
    };
    let mut client = Client::start_with(
        &f.root(),
        Setup {
            options,
            ..Setup::default()
        },
    );
    let response = client.request(
        "textDocument/semanticTokens/range",
        serde_json::json!({
            "textDocument": { "uri": support::uri(&f.path("docs/index.md")).as_str() },
            "range": { "start": { "line": 0, "character": 0 }, "end": { "line": 1, "character": 0 } },
        }),
    );
    let error = response.response_result.expect_err("an error response");
    assert_eq!(error.code, lsp_server::ErrorCode::InternalError as i32);
    // Still alive and still following changes.
    let page = f.path("docs/index.md");
    f.write("docs/_setup.md", "# Install\n@id: setup\n");
    client.watched(&[(&f.path("docs/_setup.md"), FileChangeType::CHANGED)]);
    client.settle();
    assert_eq!(client.codes(&page), ["include-id-missing"]);
    assert_eq!(client.shutdown(), Exit::Clean);
}

#[test]
fn a_panic_while_computing_diagnostics_does_not_take_the_server_down() {
    let f = project();
    let fired = Arc::new(std::sync::atomic::AtomicBool::new(false));
    let hook_fired = fired.clone();
    let options = Options {
        before_publish: Some(Arc::new(move |info| {
            if info.files.iter().any(|p| p == "index.md")
                && !hook_fired.swap(true, std::sync::atomic::Ordering::SeqCst)
            {
                panic!("boom");
            }
        })),
        ..Options::default()
    };
    let mut client = Client::start_with(
        &f.root(),
        Setup {
            options,
            ..Setup::default()
        },
    );
    client.settle();
    assert!(fired.load(std::sync::atomic::Ordering::SeqCst));
    let page = f.path("docs/index.md");
    f.write("docs/_setup.md", "# Install\n@id: setup\n");
    client.watched(&[(&f.path("docs/_setup.md"), FileChangeType::CHANGED)]);
    client.settle();
    assert_eq!(client.codes(&page), ["include-id-missing"]);
}

#[test]
fn exit_without_shutdown_is_an_abrupt_exit() {
    let f = project();
    let mut client = Client::start(&f.root());
    client.settle();
    assert_eq!(client.exit_without_shutdown(), Exit::Abrupt);
}
