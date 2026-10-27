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

use crate::links;

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
/// projects, is left out, and so is that `ascribe.addresses`, once a
/// project has it, can't be compared with a base outside a git repository.
fn reported(dir: &Path) -> Vec<(String, String)> {
    let output = ascribe(dir, &["check", "--format", "json"]);
    let report: Value = serde_json::from_slice(&output.stdout).unwrap();
    report["diagnostics"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|d| d["severity"] != "advice" && d["slug"] != "addresses-base-unreadable")
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
fn sync_check_writes_nothing_and_fails_while_the_list_is_behind() {
    let dir = project();
    let none = ascribe(dir.path(), &["permalinks", "sync", "--check"]);
    assert_eq!(code_of(&none), 1);
    assert_eq!(
        stderr(&none),
        "error: there's no ascribe.permalinks; run `ascribe permalinks sync` to make one\n"
    );
    assert!(!dir.path().join("ascribe.permalinks").exists());

    ascribe(dir.path(), &["permalinks", "sync"]);
    let current = ascribe(dir.path(), &["permalinks", "sync", "--check"]);
    assert_eq!(code_of(&current), 0, "{}", stderr(&current));
    assert_eq!(stdout(&current), "ascribe.permalinks is up to date.\n");

    let before = list_text(dir.path());
    write(
        dir.path(),
        "docs/guide.md",
        "---\ntitle: Guide\n---\n\n## Deploy\n@id {permalink=true}: deploy\n",
    );
    let behind = ascribe(dir.path(), &["permalinks", "sync", "--check"]);
    assert_eq!(code_of(&behind), 1);
    // A unified diff of the list, as `sync` would write it.
    let diff = stdout(&behind);
    assert!(
        diff.starts_with("--- a/ascribe.permalinks\n+++ b/ascribe.permalinks\n@@ "),
        "{diff}"
    );
    assert!(
        diff.lines()
            .any(|line| line.starts_with('+') && !line.starts_with("+++") && line.contains("deploy")),
        "{diff}"
    );
    assert_eq!(
        stderr(&behind),
        "error: ascribe.permalinks is out of date; run `ascribe permalinks sync` to update it\n"
    );
    assert_eq!(list_text(dir.path()), before, "--check wrote the list");

    let json = ascribe(
        dir.path(),
        &["permalinks", "sync", "--check", "--format", "json"],
    );
    assert_eq!(code_of(&json), 1);
    let json: Value = serde_json::from_slice(&json.stdout).unwrap();
    assert_eq!(json["changed"], true);
    assert_eq!(json["added"], serde_json::json!(["deploy"]));
    assert_eq!(list_text(dir.path()), before, "--check wrote the list");
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
    assert_eq!(slug, "permalink-lost");
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
        stderr(&sync).contains("kept rotate-keys, which the project no longer has"),
        "{}",
        stderr(&sync)
    );
    let found = reported(dir.path());
    assert_eq!(found.len(), 1, "{found:?}");
    assert_eq!(found[0].0, "permalink-lost");
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

    let sync = ascribe(
        dir.path(),
        &["permalinks", "sync", "--force", "--format", "json"],
    );
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
    assert_eq!(json["schema_version"], 2);
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
    let list = ["--against", "docs.permalinks"];

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

#[test]
fn retiring_a_lost_permalink_writes_ascribe_addresses_and_quiets_check() {
    let dir = project();
    ascribe(dir.path(), &["permalinks", "sync"]);
    fs::remove_file(dir.path().join("docs/keys.md")).unwrap();

    let declared = ascribe(
        dir.path(),
        &["permalinks", "retire", "install", "--reason", "Gone"],
    );
    assert_eq!(code_of(&declared), 2);
    assert!(
        stderr(&declared).contains("can't retire `install`: `install.md` still declares it"),
        "{}",
        stderr(&declared)
    );
    let placeholder = ascribe(
        dir.path(),
        &[
            "permalinks",
            "retire",
            "--force",
            "rotate-keys",
            "--reason",
            "why this address was retired",
        ],
    );
    assert_eq!(code_of(&placeholder), 2);

    let retire = ascribe(
        dir.path(),
        &[
            "permalinks",
            "retire",
            "--force",
            "rotate-keys",
            "--reason",
            "Merged into the install page",
        ],
    );
    assert_eq!(code_of(&retire), 0, "{}", stderr(&retire));
    assert_eq!(
        stdout(&retire),
        "retired rotate-keys\nwrote ascribe.addresses\nwrote ascribe.permalinks\n"
    );
    let addresses = fs::read_to_string(dir.path().join("ascribe.addresses")).unwrap();
    assert!(
        addresses.contains("[retired]\n\"go:rotate-keys\" = \"Merged into the install page\"\n"),
        "{addresses}"
    );
    assert!(
        !fs::read_to_string(dir.path().join("ascribe.toml"))
            .unwrap()
            .contains("permalinks"),
    );
    assert!(
        list_text(dir.path())
            .ends_with("[retired]\nrotate-keys = \"Merged into the install page\"\n"),
        "{}",
        list_text(dir.path())
    );
    assert_eq!(reported(dir.path()), []);
    let again = ascribe(dir.path(), &["permalinks", "sync"]);
    assert_eq!(stdout(&again), "ascribe.permalinks is up to date.\n");

    let verify = ascribe(dir.path(), &["permalinks", "verify", "rotate-keys"]);
    assert_eq!(code_of(&verify), 1);
    assert_eq!(
        stderr(&verify),
        "error: rotate-keys was retired from the docs: Merged into the install page\n"
    );
}

#[test]
fn retire_takes_a_page_as_check_names_it_and_moves_it_to_another() {
    let dir = project();
    ascribe(dir.path(), &["permalinks", "sync"]);

    // From the project's folder: the path `ascribe check` prints. `--to`
    // moves the name, and needs no reason.
    let retire = ascribe(
        dir.path(),
        &[
            "permalinks",
            "retire",
            "--force",
            "--page",
            "docs/keys.md",
            "--to",
            "docs/install.md",
        ],
    );
    assert_eq!(code_of(&retire), 0, "{}", stderr(&retire));
    assert!(
        stdout(&retire).starts_with("moved rotate-keys to install.md\n"),
        "{}",
        stdout(&retire)
    );
    let addresses = fs::read_to_string(dir.path().join("ascribe.addresses")).unwrap();
    assert!(
        addresses.contains("[moved]\n\"go:rotate-keys\" = \"install.md\"\n"),
        "{addresses}"
    );

    // From the content root, a path from there.
    let retire = ascribe(
        &dir.path().join("docs"),
        &[
            "permalinks",
            "retire",
            "--force",
            "--page",
            "install.md",
            "--reason",
            "Gone",
        ],
    );
    assert_eq!(code_of(&retire), 0, "{}", stderr(&retire));
    assert!(
        stdout(&retire).starts_with("retired install\n"),
        "{}",
        stdout(&retire)
    );

    // A page with none is named as it was given.
    write(dir.path(), "docs/other.md", "# Other\n");
    let none = ascribe(
        dir.path(),
        &[
            "permalinks",
            "retire",
            "--force",
            "--page",
            "docs/other.md",
            "--reason",
            "Gone",
        ],
    );
    assert_eq!(code_of(&none), 2);
    assert!(
        stderr(&none).contains("can't retire `docs/other.md`: it declares no permalinks"),
        "{}",
        stderr(&none)
    );
}

#[test]
fn a_name_moved_to_a_permalink_is_listed_as_an_alias_and_verified_as_found() {
    let dir = project();
    write(
        dir.path(),
        "ascribe.addresses",
        "version = 1\n\n[moved]\n\"go:key-rotation\" = \"go:rotate-keys\"\n\n[retired]\n",
    );
    let sync = ascribe(dir.path(), &["permalinks", "sync"]);
    assert_eq!(code_of(&sync), 0, "{}", stderr(&sync));
    assert!(
        list_text(dir.path()).ends_with("[aliases]\nkey-rotation = \"rotate-keys\"\n"),
        "{}",
        list_text(dir.path())
    );
    assert_eq!(reported(dir.path()), []);
    let verify = ascribe(dir.path(), &["permalinks", "verify", "key-rotation"]);
    assert_eq!(code_of(&verify), 0, "{}", stderr(&verify));
}

#[test]
fn sync_doesnt_write_the_list_through_a_link() {
    let dir = project();
    let outside = tempfile::tempdir().unwrap();
    // A link to a file that isn't there yet, so nothing is read through it.
    let target = outside.path().join("written.txt");
    if !links::file(&target, dir.path().join("ascribe.permalinks")) {
        return;
    }
    let output = ascribe(dir.path(), &["permalinks", "sync"]);
    assert_eq!(code_of(&output), 2, "{}", stderr(&output));
    assert!(stderr(&output).contains("link"), "{}", stderr(&output));
    assert!(!target.exists(), "the list was written through the link");
}

#[test]
fn retire_moves_a_name_to_an_alias_and_refuses_a_name_that_s_nowhere() {
    let dir = project();
    write(
        dir.path(),
        "ascribe.addresses",
        "version = 1\n\n[moved]\n\"go:setup\" = \"go:install\"\n\n[retired]\n",
    );
    let refused = ascribe(
        dir.path(),
        &[
            "permalinks",
            "retire",
            "--force",
            "--page",
            "docs/keys.md",
            "--to",
            "go:typo",
        ],
    );
    assert_eq!(code_of(&refused), 2);
    assert!(
        stderr(&refused).contains("`go:typo` isn't a permalink of the project"),
        "{}",
        stderr(&refused)
    );

    let moved = ascribe(
        dir.path(),
        &[
            "permalinks",
            "retire",
            "--force",
            "--page",
            "docs/keys.md",
            "--to",
            "go:setup",
        ],
    );
    assert_eq!(code_of(&moved), 0, "{}", stderr(&moved));
    let addresses = fs::read_to_string(dir.path().join("ascribe.addresses")).unwrap();
    assert!(
        addresses.contains("\"go:rotate-keys\" = \"go:setup\"\n\"go:setup\" = \"go:install\"\n"),
        "{addresses}"
    );
}
