//! `ascribe rename`, by running the binary in temporary projects: what it
//! writes, what it prints, and its exit codes. Which places a rename
//! changes is held by `ascribe-lsp`'s tests (`tests/all/rename.rs`, and
//! `model_edits.rs`, where it's compared with the editor's rename).

#![allow(clippy::expect_used, clippy::unwrap_used, clippy::panic)]

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use tempfile::TempDir;

const MODEL: &str =
    "spec = \"0.1\"\n\n[project]\ncontent-root = \"docs\"\n\n[phrases]\ncloud = \"Quill Cloud\"\n";

const KEYS: &str = "---\ntitle: Keys\n---\n\nSign in to {cloud}.\n\n## Rotate keys\n\nText.\n";

const INSTALL: &str =
    "---\ntitle: Install\n---\n\nRun it on {cloud}. See [](keys.md#rotate-keys).\n";

fn write(root: &Path, rel: &str, text: &str) {
    let path: PathBuf = rel.split('/').fold(root.to_path_buf(), |p, s| p.join(s));
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, text).unwrap();
}

fn read(root: &Path, rel: &str) -> String {
    let path: PathBuf = rel.split('/').fold(root.to_path_buf(), |p, s| p.join(s));
    fs::read_to_string(path).unwrap()
}

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

#[test]
fn a_phrase_is_renamed_in_the_model_and_every_page_and_each_file_is_named() {
    let dir = project();
    let output = ascribe(dir.path(), &["rename", "phrase:{cloud}", "hosted"]);
    assert_eq!(code_of(&output), 0, "{}", stderr(&output));
    assert_eq!(
        stdout(&output),
        "changed ascribe.toml (1 place)\nchanged docs/install.md (1 place)\nchanged docs/keys.md (1 place)\n"
    );
    assert_eq!(
        read(dir.path(), "ascribe.toml"),
        MODEL.replace("cloud =", "hosted =")
    );
    assert_eq!(
        read(dir.path(), "docs/keys.md"),
        KEYS.replace("{cloud}", "{hosted}")
    );
    let check = ascribe(dir.path(), &["check"]);
    assert_eq!(code_of(&check), 0, "{}", stdout(&check));
}

#[test]
fn a_heading_and_a_page_are_named_from_the_current_folder() {
    let dir = project();
    let docs = dir.path().join("docs");
    let output = ascribe(&docs, &["rename", "keys.md#rotate-keys", "rotate"]);
    assert_eq!(code_of(&output), 0, "{}", stderr(&output));
    assert_eq!(
        stdout(&output),
        "changed install.md (1 place)\nchanged keys.md (1 place)\n"
    );
    assert!(read(dir.path(), "docs/keys.md").contains("## Rotate keys\n@id: rotate\n"));

    let output = ascribe(&docs, &["rename", "keys.md", "guides/keys.md"]);
    assert_eq!(code_of(&output), 0, "{}", stderr(&output));
    assert_eq!(
        stdout(&output),
        "changed install.md (1 place)\nmoved keys.md to guides/keys.md\n"
    );
    assert!(!docs.join("keys.md").exists());
    assert_eq!(
        read(dir.path(), "docs/install.md"),
        INSTALL.replace("keys.md#rotate-keys", "guides/keys.md#rotate")
    );
    let check = ascribe(dir.path(), &["check"]);
    assert_eq!(code_of(&check), 0, "{}", stdout(&check));
}

#[test]
fn what_isnt_there_exits_1_and_what_cant_be_renamed_exits_2_changing_nothing() {
    let dir = project();
    for (args, code) in [
        (&["rename", "phrase:product", "name"][..], 1),
        (&["rename", "docs/nothing.md", "docs/new.md"][..], 1),
        (&["rename", "docs/keys.md#nothing", "new"][..], 1),
        (&["rename", "phrase:cloud", "Hosted"][..], 2),
        (&["rename", "docs/keys.md", "docs/install.md"][..], 2),
        (&["rename", "docs/keys.md", "elsewhere/keys.md"][..], 2),
        (&["rename", "feature:sso", "login"][..], 2),
    ] {
        let output = ascribe(dir.path(), args);
        assert_eq!(code_of(&output), code, "{args:?}: {}", stderr(&output));
        assert!(stderr(&output).starts_with("error: "), "{args:?}");
        assert_eq!(stdout(&output), "", "{args:?}");
    }
    assert_eq!(read(dir.path(), "ascribe.toml"), MODEL);
    assert_eq!(read(dir.path(), "docs/keys.md"), KEYS);
    assert_eq!(read(dir.path(), "docs/install.md"), INSTALL);
}

#[test]
fn a_page_isnt_moved_through_a_folder_that_is_a_link() {
    let dir = project();
    let outside = tempfile::tempdir().unwrap();
    crate::links::dir(outside.path(), dir.path().join("docs").join("guides"));
    let output = ascribe(
        dir.path(),
        &["rename", "docs/keys.md", "docs/guides/keys.md"],
    );
    assert_eq!(code_of(&output), 2, "{}", stdout(&output));
    assert!(stderr(&output).contains("link"), "{}", stderr(&output));
    assert_eq!(stdout(&output), "");
    assert_eq!(read(dir.path(), "docs/keys.md"), KEYS);
    assert_eq!(read(dir.path(), "docs/install.md"), INSTALL);
    assert_eq!(fs::read_dir(outside.path()).unwrap().count(), 0);
}
