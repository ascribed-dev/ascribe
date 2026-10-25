//! The addresses compared with the base's between checks (SPEC §7.10):
//! `ascribe/addresses`, which an editor's server sends once the project
//! loads and after a save, a change on disk, or an edit to
//! `ascribe.addresses`, and This page's "Also answers at", in a temporary
//! git repository. The comparison itself is `ascribe-check`'s, tested
//! there with the base as data; the bases `git` can't read are the CLI's
//! tests. This file holds what's the server's: when it compares, what it
//! keeps, and the file's buffer.

#![allow(clippy::expect_used, clippy::panic, clippy::unwrap_used)]

use std::path::Path;
use std::process::Command;
use std::sync::Arc;
use std::sync::Mutex;

use ascribe_lsp::Options;
use lsp_types::FileChangeType;
use serde_json::{Value, json};

use crate::support::{self, Client, Fixture, MODEL, Setup};

const KEYS: &str = "---\ntitle: Keys\n---\n\n# Keys\n\nRotate them yearly.\n";
const SETUP: &str = "---\ntitle: Setup\n---\n\n# Setup\n\nInstall it.\n";
const EMPTY: &str = "version = 1\n\n[moved]\n";
const MOVED: &str = "version = 1\n\n[moved]\n\"guides/keys.md\" = \"security/keys.md\"\n";

fn git(root: &Path, args: &[&str]) -> String {
    let out = Command::new("git")
        .current_dir(root)
        .args([
            "-c",
            "user.name=Test",
            "-c",
            "user.email=test@example.com",
            "-c",
            "commit.gpgsign=false",
            "-c",
            "core.autocrlf=false",
        ])
        .args(args)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "git {args:?}: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8(out.stdout).unwrap().trim().to_owned()
}

fn commit(f: &Fixture, message: &str) {
    git(&f.root(), &["add", "-A"]);
    git(&f.root(), &["commit", "-q", "-m", message]);
}

/// A project keeping its addresses, with two pages, committed on `main`.
fn repository() -> Fixture {
    let f = Fixture::new(
        MODEL,
        &[
            ("ascribe.addresses", EMPTY),
            ("docs/guides/keys.md", KEYS),
            ("docs/setup.md", SETUP),
        ],
    );
    git(&f.root(), &["init", "-q", "-b", "main"]);
    commit(&f, "First");
    f
}

/// A client whose server counts the bases it lists, in `listed`.
fn start(f: &Fixture) -> (Client, Arc<Mutex<Vec<String>>>) {
    let listed = Arc::new(Mutex::new(Vec::new()));
    let hook = listed.clone();
    let client = Client::start_with(
        &f.root(),
        Setup {
            options: Options {
                base_listed: Some(Arc::new(move |commit: &str| {
                    hook.lock().unwrap().push(commit.to_owned());
                })),
                ..Options::default()
            },
            ..Setup::default()
        },
    );
    (client, listed)
}

/// The last `ascribe/addresses` the server sent.
fn last(client: &Client) -> Value {
    client
        .notifications
        .iter()
        .rev()
        .find(|n| n.method == "ascribe/addresses")
        .map(|n| n.params.clone())
        .expect("the server says what it compared")
}

fn sent(client: &Client) -> usize {
    client
        .notifications
        .iter()
        .filter(|n| n.method == "ascribe/addresses")
        .count()
}

fn codes(params: &Value) -> Vec<String> {
    params["diagnostics"]
        .as_array()
        .unwrap()
        .iter()
        .map(|d| d["data"]["slug"].as_str().unwrap().to_owned())
        .collect()
}

/// Moves a page on disk, as the Explorer does, and tells the server.
fn move_page(client: &mut Client, f: &Fixture, from: &str, to: &str) {
    let text = std::fs::read_to_string(f.path(from)).unwrap();
    f.remove(from);
    f.write(to, &text);
    client.watched(&[
        (&f.path(from), FileChangeType::DELETED),
        (&f.path(to), FileChangeType::CREATED),
    ]);
}

fn also_answers_at(client: &mut Client, path: &Path) -> Value {
    let result = client
        .request(
            "ascribe/page",
            json!({
                "textDocument": { "uri": support::uri(path).as_str() },
                "position": { "line": 0, "character": 0 },
            }),
        )
        .response_result
        .expect("the request succeeds");
    result["about"]["alsoAnswersAt"].clone()
}

#[test]
fn the_server_lists_the_base_s_addresses_once_per_base_commit() {
    let f = repository();
    let (mut client, listed) = start(&f);
    client.settle();
    let head = git(&f.root(), &["rev-parse", "HEAD"]);
    assert_eq!(*listed.lock().unwrap(), std::slice::from_ref(&head));
    let params = last(&client);
    assert_eq!(params["base"]["requested"], "main");
    assert_eq!(params["lost"], 0);
    assert_eq!(params["diagnostics"], json!([]));

    // Changes compare again, with the base kept.
    f.write(
        "docs/setup.md",
        &SETUP.replace("Install it.", "Install it twice."),
    );
    client.watched(&[(&f.path("docs/setup.md"), FileChangeType::CHANGED)]);
    client.settle();
    let page = f.path("docs/setup.md");
    client.open(&page, 1, SETUP);
    client.save(&page);
    client.settle();
    assert_eq!(listed.lock().unwrap().len(), 1, "{listed:?}");

    // A new base commit is listed again, once.
    commit(&f, "Second");
    client.watched(&[(&f.path("docs/setup.md"), FileChangeType::CHANGED)]);
    client.settle();
    client.save(&page);
    client.settle();
    let head = git(&f.root(), &["rev-parse", "HEAD"]);
    assert_eq!(listed.lock().unwrap().last(), Some(&head));
    assert_eq!(listed.lock().unwrap().len(), 2, "{listed:?}");
    client.shutdown();
}

#[test]
fn a_page_deleted_on_disk_is_lost_with_no_check_run() {
    let f = repository();
    let (mut client, _) = start(&f);
    client.settle();
    assert_eq!(last(&client)["lost"], 0);

    f.remove("docs/setup.md");
    client.watched(&[(&f.path("docs/setup.md"), FileChangeType::DELETED)]);
    client.settle();
    let params = last(&client);
    assert_eq!(params["lost"], 1, "{params}");
    assert_eq!(codes(&params), ["address-lost"]);
    assert_eq!(
        params["uri"],
        support::uri(&f.path("ascribe.addresses")).as_str()
    );
    let lost = &params["diagnostics"][0];
    assert!(
        lost["message"].as_str().unwrap().contains("setup.md"),
        "{lost}"
    );
    // With its fixes, as `ascribe check` gives them.
    assert!(
        !lost["data"]["fixes"].as_array().unwrap().is_empty(),
        "{lost}"
    );
    // None of the pages has a diagnostic of its own about it.
    assert!(
        client
            .log
            .iter()
            .flat_map(|p| &p.diagnostics)
            .all(|d| support::slug(d) != "address-lost"),
        "{:?}",
        client.log
    );
    client.shutdown();
}

#[test]
fn a_move_s_unsaved_entry_counts_and_closing_it_unsaved_loses_the_page() {
    let f = repository();
    let (mut client, _) = start(&f);
    client.settle();
    let file = f.path("ascribe.addresses");
    // The move's edit opens the file, with the entry unsaved.
    client.open(&file, 1, MOVED);
    move_page(
        &mut client,
        &f,
        "docs/guides/keys.md",
        "docs/security/keys.md",
    );
    client.settle();
    let params = last(&client);
    assert_eq!(params["lost"], 0, "{params}");
    assert_eq!(
        also_answers_at(&mut client, &f.path("docs/security/keys.md")),
        json!([{ "address": "/guides/keys/", "entry": "guides/keys.md" }])
    );
    assert_eq!(
        also_answers_at(&mut client, &f.path("docs/setup.md")),
        json!([])
    );

    // Closed without saving, the file is the disk's again, with no entry.
    client.close(&file);
    client.settle();
    let params = last(&client);
    assert_eq!(params["lost"], 1, "{params}");
    assert_eq!(codes(&params), ["address-lost"]);
    assert_eq!(
        also_answers_at(&mut client, &f.path("docs/security/keys.md")),
        json!([])
    );
    client.shutdown();
}

#[test]
fn an_entry_left_after_the_page_moves_back_is_reported_at_the_file() {
    let f = repository();
    // A move, saved, then undone on disk: the page is back at its key.
    f.write("ascribe.addresses", MOVED);
    let (mut client, _) = start(&f);
    client.settle();
    let params = last(&client);
    assert_eq!(params["lost"], 0, "{params}");
    assert_eq!(codes(&params), ["addresses-entry-live"]);
    client.shutdown();
}

#[test]
fn a_project_without_the_file_reads_no_base() {
    let f = repository();
    f.remove("ascribe.addresses");
    let (mut client, listed) = start(&f);
    client.settle();
    f.remove("docs/setup.md");
    client.watched(&[(&f.path("docs/setup.md"), FileChangeType::DELETED)]);
    client.settle();
    assert!(listed.lock().unwrap().is_empty(), "{listed:?}");
    // Said once: nothing is compared.
    assert_eq!(sent(&client), 1);
    assert_eq!(
        last(&client),
        json!({ "uri": null, "base": null, "lost": 0, "diagnostics": [] })
    );
    client.shutdown();
}

#[test]
fn the_file_made_while_the_server_runs_is_compared_and_a_move_records_in_it() {
    let f = repository();
    f.remove("ascribe.addresses");
    commit(&f, "Without the file");
    let (mut client, _) = start(&f);
    client.settle();
    // `ascribe addresses init`, as the editor's offer runs it.
    f.write("ascribe.addresses", EMPTY);
    client.watched(&[(&f.path("ascribe.addresses"), FileChangeType::CREATED)]);
    client.settle();
    let params = last(&client);
    assert_eq!(params["lost"], 0, "{params}");
    assert_eq!(codes(&params), Vec::<String>::new());
    let answer = client
        .request(
            "workspace/willRenameFiles",
            json!({ "files": [{
                "oldUri": support::uri(&f.path("docs/guides/keys.md")).as_str(),
                "newUri": support::uri(&f.path("docs/security/keys.md")).as_str(),
            }] }),
        )
        .response_result
        .expect("a rename");
    let file = support::uri(&f.path("ascribe.addresses"));
    let edits = &answer["changes"][file.as_str()];
    assert!(
        edits.to_string().contains("guides/keys.md"),
        "no entry for the move: {answer}"
    );
    client.shutdown();
}

#[test]
fn a_project_outside_a_repository_says_the_base_can_t_be_read() {
    let f = Fixture::new(
        MODEL,
        &[("ascribe.addresses", EMPTY), ("docs/setup.md", SETUP)],
    );
    let (mut client, listed) = start(&f);
    client.settle();
    let params = last(&client);
    assert_eq!(codes(&params), ["addresses-base-unreadable"]);
    assert_eq!(params["base"], Value::Null);
    assert!(listed.lock().unwrap().is_empty());
    client.shutdown();
}
