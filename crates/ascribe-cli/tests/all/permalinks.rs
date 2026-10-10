//! `ascribe permalinks`, and `ascribe check` over a project that commits
//! its list, by running the binary in temporary projects: what a person
//! sees when a permalink is added, moved, or removed, and what a product's
//! CI sees from `verify` with nothing but a copy of the list. What the
//! checks report on each case is held by the conformance cases in
//! `tests/conformance/cases/permalinks/`; the JSON's shapes by the schemas.

#![allow(clippy::expect_used, clippy::unwrap_used, clippy::panic)]

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use serde_json::Value;
use tempfile::TempDir;

const MODEL: &str = "spec = \"0.1\"\n\n[project]\ncontent-root = \"docs\"\n";

const KEYS: &str =
    "---\ntitle: Keys\n---\n\n## Rotate keys\n@id {permalink=true}: rotate-keys\n\nText.\n";

const INSTALL: &str = "---\ntitle: Install\npermalink: install\n---\n\nText.\n";

fn write(root: &Path, rel: &str, text: &str) {
    let path: PathBuf = rel.split('/').fold(root.to_path_buf(), |p, s| p.join(s));
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, text).unwrap();
}

/// A project with a page's own permalink and a heading's.
fn project() -> TempDir {
    let dir = tempfile::tempdir().unwrap();
    write(dir.path(), "ascribe.toml", MODEL);
    write(dir.path(), "docs/keys.md", KEYS);
    write(dir.path(), "docs/install.md", INSTALL);
    dir
}

fn ascribe(dir: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_ascribe"))
        .current_dir(dir)
        .args(args)
        .env("NO_COLOR", "1")
        .output()
        .expect("run ascribe")
}

fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

fn code_of(output: &Output) -> i32 {
    output.status.code().expect("exited with a code")
}

fn list_text(dir: &Path) -> String {
    fs::read_to_string(dir.join("ascribe.permalinks")).unwrap()
}

/// The errors and warnings `ascribe check --format json` reports, by slug,
/// with each message. Advice, such as `page-orphan` on these small
/// projects, is left out.
fn reported(dir: &Path) -> Vec<(String, String)> {
    let output = ascribe(dir, &["check", "--format", "json"]);
    let report: Value = serde_json::from_slice(&output.stdout).unwrap();
    report["diagnostics"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|d| d["severity"] != "advice")
        .map(|d| {
            (
                d["slug"].as_str().unwrap().to_owned(),
                d["message"].as_str().unwrap().to_owned(),
            )
        })
        .collect()
}

#[test]
fn a_project_without_the_list_is_told_nothing_about_its_permalinks() {
    let dir = project();
    let check = ascribe(dir.path(), &["check", "--deny-warnings"]);
    assert_eq!(code_of(&check), 0, "{}", stdout(&check));
    let list = ascribe(dir.path(), &["permalinks", "list"]);
    assert_eq!(code_of(&list), 0);
    let text = stdout(&list);
    assert!(text.contains("docs/install.md:3: install"), "{text}");
    assert!(text.contains("docs/keys.md:6: rotate-keys"), "{text}");
    assert!(text.contains("No ascribe.permalinks"), "{text}");
    assert!(!dir.path().join("ascribe.permalinks").exists());
}

#[test]
fn sync_makes_the_list_and_check_then_passes() {
    let dir = project();
    let sync = ascribe(dir.path(), &["permalinks", "sync"]);
    assert_eq!(code_of(&sync), 0, "{}", stderr(&sync));
    assert!(
        stdout(&sync).contains("with 2 permalinks"),
        "{}",
        stdout(&sync)
    );
    assert_eq!(
        list_text(dir.path()),
        "# Written by Ascribe. Don't edit it: run `ascribe permalinks sync`.\n\
         version = 1\n\
         \n\
         [permalinks]\n\
         install = \"install.md\"\n\
         rotate-keys = \"keys.md#rotate-keys\"\n"
    );
    let again = ascribe(dir.path(), &["permalinks", "sync"]);
    assert_eq!(stdout(&again), "ascribe.permalinks is up to date.\n");
    let check = ascribe(dir.path(), &["check", "--deny-warnings"]);
    assert_eq!(code_of(&check), 0, "{}", stdout(&check));
}

#[test]
fn deleting_a_marked_heading_warns_naming_it_and_what_the_check_cant_see() {
    let dir = project();
    ascribe(dir.path(), &["permalinks", "sync"]);
    write(
        dir.path(),
        "docs/keys.md",
        "---\ntitle: Keys\n---\n\nText.\n",
    );
    let found = reported(dir.path());
    assert_eq!(found.len(), 1, "{found:?}");
    let (slug, message) = &found[0];
    assert_eq!(slug, "permalink-removed");
    assert!(
        message.contains("`rotate-keys` was a permalink"),
        "{message}"
    );
    assert!(
        message.contains("Links to it from outside these docs, such as from a product, aren't found by this check"),
        "{message}"
    );
    // A warning, so it fails only with --deny-warnings: what stops a
    // removal reaching main unnoticed.
    assert_eq!(code_of(&ascribe(dir.path(), &["check"])), 0);
    assert_eq!(
        code_of(&ascribe(dir.path(), &["check", "--deny-warnings"])),
        1
    );
}

#[test]
fn sync_keeps_a_removed_name_and_says_so() {
    let dir = project();
    ascribe(dir.path(), &["permalinks", "sync"]);
    let before = list_text(dir.path());
    fs::remove_file(dir.path().join("docs/keys.md")).unwrap();
    let sync = ascribe(dir.path(), &["permalinks", "sync"]);
    assert_eq!(code_of(&sync), 0);
    assert_eq!(list_text(dir.path()), before, "sync dropped a name");
    assert!(
        stderr(&sync).contains("kept rotate-keys, which the content no longer declares"),
        "{}",
        stderr(&sync)
    );
    let found = reported(dir.path());
    assert_eq!(found.len(), 1, "{found:?}");
    assert_eq!(found[0].0, "permalink-removed");
}

#[test]
fn moving_a_heading_and_syncing_changes_one_line_and_warns_about_nothing() {
    let dir = project();
    ascribe(dir.path(), &["permalinks", "sync"]);
    let before = list_text(dir.path());
    write(
        dir.path(),
        "docs/keys.md",
        "---\ntitle: Keys\n---\n\nText.\n",
    );
    write(
        dir.path(),
        "docs/security.md",
        "---\ntitle: Security\n---\n\n## Rotate keys\n@id {permalink=true}: rotate-keys\n",
    );
    // Moving isn't removing: the list is behind, at the heading's new place.
    let found = reported(dir.path());
    assert_eq!(found.len(), 1, "{found:?}");
    assert_eq!(found[0].0, "permalinks-list-behind");
    assert!(found[0].1.contains("keys.md#rotate-keys"), "{}", found[0].1);

    let sync = ascribe(dir.path(), &["permalinks", "sync", "--format", "json"]);
    assert_eq!(code_of(&sync), 0);
    let json: Value = serde_json::from_slice(&sync.stdout).unwrap();
    assert_eq!(json["changed"], true);
    assert_eq!(json["moved"][0]["name"], "rotate-keys");
    assert_eq!(json["moved"][0]["from"], "keys.md#rotate-keys");
    assert_eq!(json["moved"][0]["to"], "security.md#rotate-keys");
    let after = list_text(dir.path());
    let changed: Vec<(&str, &str)> = before
        .lines()
        .zip(after.lines())
        .filter(|(a, b)| a != b)
        .collect();
    assert_eq!(
        changed,
        [(
            "rotate-keys = \"keys.md#rotate-keys\"",
            "rotate-keys = \"security.md#rotate-keys\""
        )]
    );
    assert_eq!(before.lines().count(), after.lines().count());
    assert!(reported(dir.path()).is_empty());
}

#[test]
fn list_says_which_permalinks_the_list_has() {
    let dir = project();
    ascribe(dir.path(), &["permalinks", "sync"]);
    write(
        dir.path(),
        "docs/install.md",
        "---\ntitle: Install\n---\n\nText.\n",
    );
    write(
        dir.path(),
        "docs/keys.md",
        &format!("{KEYS}\n## Revoke keys\n@id {{permalink=true}}: revoke-keys\n"),
    );
    let list = ascribe(dir.path(), &["permalinks", "list", "--format", "json"]);
    assert_eq!(code_of(&list), 0);
    let json: Value = serde_json::from_slice(&list.stdout).unwrap();
    assert_eq!(json["schema_version"], 1);
    assert_eq!(json["has_list"], true);
    let states: Vec<(&str, &str)> = json["permalinks"]
        .as_array()
        .unwrap()
        .iter()
        .map(|p| (p["name"].as_str().unwrap(), p["state"].as_str().unwrap()))
        .collect();
    assert_eq!(
        states,
        [("rotate-keys", "listed"), ("revoke-keys", "not_listed")]
    );
    assert_eq!(json["removed"][0]["name"], "install");
    assert_eq!(json["removed"][0]["place"], "install.md");
}

#[test]
fn verify_needs_only_the_list_and_fails_on_a_name_it_doesnt_have() {
    let docs = project();
    ascribe(docs.path(), &["permalinks", "sync"]);
    // A product's repository: a copy of the list, and no ascribe.toml.
    let product = tempfile::tempdir().unwrap();
    write(product.path(), "docs.permalinks", &list_text(docs.path()));
    write(
        product.path(),
        "links.txt",
        "# The names the product links to.\ninstall\n\nrotate-keys\n",
    );
    let list = ["--list", "docs.permalinks"];

    let pass = ascribe(
        product.path(),
        &[&["permalinks", "verify", "--from", "links.txt"], &list[..]].concat(),
    );
    assert_eq!(code_of(&pass), 0, "{}", stderr(&pass));
    assert_eq!(
        stdout(&pass),
        "2 names are permalinks in docs.permalinks.\n"
    );

    let fail = ascribe(
        product.path(),
        &[&["permalinks", "verify", "install", "nope"], &list[..]].concat(),
    );
    assert_eq!(code_of(&fail), 1);
    assert_eq!(
        stderr(&fail),
        "error: nope isn't a permalink in docs.permalinks\n"
    );

    let json = ascribe(
        product.path(),
        &[
            &[
                "permalinks",
                "verify",
                "nope",
                "install",
                "nope",
                "--format",
                "json",
            ],
            &list[..],
        ]
        .concat(),
    );
    assert_eq!(code_of(&json), 1);
    let json: Value = serde_json::from_slice(&json.stdout).unwrap();
    assert_eq!(json["list"], "docs.permalinks");
    assert_eq!(json["found"], serde_json::json!(["install"]));
    assert_eq!(json["missing"], serde_json::json!(["nope"]));

    // In the docs, the list beside ascribe.toml.
    let beside = ascribe(docs.path(), &["permalinks", "verify", "rotate-keys"]);
    assert_eq!(code_of(&beside), 0, "{}", stderr(&beside));
}

#[test]
fn verify_cant_run_without_a_list_or_with_one_it_cant_read() {
    let dir = project();
    let missing = ascribe(dir.path(), &["permalinks", "verify", "install"]);
    assert_eq!(code_of(&missing), 2);
    assert!(
        stderr(&missing).contains("ascribe.permalinks doesn't exist"),
        "{}",
        stderr(&missing)
    );

    write(dir.path(), "ascribe.permalinks", "version = 9\n");
    let newer = ascribe(dir.path(), &["permalinks", "verify", "install"]);
    assert_eq!(code_of(&newer), 2);
    assert!(
        stderr(&newer).contains("reads version 1"),
        "{}",
        stderr(&newer)
    );
    // So does sync, which won't write over a list it can't read.
    let sync = ascribe(dir.path(), &["permalinks", "sync"]);
    assert_eq!(code_of(&sync), 2);
    assert_eq!(list_text(dir.path()), "version = 9\n");
}
