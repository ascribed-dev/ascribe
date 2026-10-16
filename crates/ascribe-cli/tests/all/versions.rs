//! `ascribe versions`, by running the binary in temporary projects: what
//! `release` writes and refuses, in a repository and out of one, and what
//! `list` and `changes` report. The JSON's shapes are held by the schemas.

#![allow(clippy::expect_used, clippy::unwrap_used, clippy::panic)]

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use serde_json::Value;
use tempfile::TempDir;

/// One site that knows its versions: `1.0` is live, `1.1` planned, and
/// `next` points at it.
const MODEL: &str = r#"spec = "0.1"

[project]
content-root = "docs"

[dimensions.release]
label = "Product"
values = ["prod"]

[releases.prod]
live = ["1.0"]
planned = ["1.1"]

[features.next]
name = "Not released yet"
available = "prod 1.1"
"#;

/// A page marked with the feature in its frontmatter.
const NEW: &str = "---\ntitle: New\navailable: next\n---\n\nText.\n";

/// A page with the feature on a block and on two rows, beside a mark for a
/// release already out.
const PAGE: &str = "---\ntitle: Page\n---\n\n## Options\n\nIntro.\n\n@available: next\nA new paragraph.\n\n@available: prod 1.0\nAn old one.\n\n| Option | Does |\n|---|---|\n| `a` {available=next} | A. |\n| `b` {available=\"next\"} | B. |\n";

fn write(root: &Path, rel: &str, text: &str) {
    let path: PathBuf = rel.split('/').fold(root.to_path_buf(), |p, s| p.join(s));
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, text).unwrap();
}

fn read(root: &Path, rel: &str) -> String {
    let path: PathBuf = rel.split('/').fold(root.to_path_buf(), |p, s| p.join(s));
    fs::read_to_string(path).unwrap()
}

fn project(model: &str) -> TempDir {
    let dir = tempfile::tempdir().unwrap();
    write(dir.path(), "ascribe.toml", model);
    write(dir.path(), "docs/new.md", NEW);
    write(dir.path(), "docs/page.md", PAGE);
    dir
}

/// A project committed to a repository of its own.
fn repository(model: &str) -> TempDir {
    let dir = project(model);
    git(dir.path(), &["init", "-q"]);
    git(dir.path(), &["add", "-A"]);
    git(dir.path(), &["commit", "-q", "-m", "Start"]);
    dir
}

fn git(dir: &Path, args: &[&str]) {
    let out = Command::new("git")
        .current_dir(dir)
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
        .expect("run git");
    assert!(
        out.status.success(),
        "git {args:?} failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

fn ascribe(dir: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_ascribe"))
        .current_dir(dir)
        .args(args)
        .env("NO_COLOR", "1")
        // A temporary folder is in no repository, whatever the environment
        // says.
        .env_remove("GIT_DIR")
        .env_remove("GIT_WORK_TREE")
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

/// Every file of the project, by path, to show that nothing was written.
fn files(dir: &Path) -> Vec<(String, String)> {
    ["ascribe.toml", "docs/new.md", "docs/page.md"]
        .iter()
        .map(|p| ((*p).to_owned(), read(dir, p)))
        .collect()
}

#[test]
fn a_release_rewrites_the_marks_moves_the_feature_and_archives_the_old_line() {
    let dir = repository(MODEL);
    let out = ascribe(
        dir.path(),
        &["versions", "release", "prod 1.1", "--next", "1.2"],
    );
    assert_eq!(code_of(&out), 0, "{}", stderr(&out));
    let text = stdout(&out);
    assert!(text.contains("prod 1.1 is live"), "{text}");
    assert!(
        text.contains("4 marks from `next` to `prod 1.1`, in 2 files"),
        "{text}"
    );

    assert!(read(dir.path(), "docs/new.md").contains("available: prod 1.1\n"));
    let page = read(dir.path(), "docs/page.md");
    assert!(
        page.contains("@available: prod 1.1\nA new paragraph."),
        "{page}"
    );
    assert!(page.contains("@available: prod 1.0\nAn old one."), "{page}");
    assert!(page.contains("`a` {available=\"prod 1.1\"}"), "{page}");
    assert!(page.contains("`b` {available=\"prod 1.1\"}"), "{page}");

    let model = read(dir.path(), "ascribe.toml");
    assert!(model.contains("live = [\"1.1\"]"), "{model}");
    assert!(model.contains("planned = [\"1.2\"]"), "{model}");
    assert!(
        model.contains("[releases.prod.archived]\n\"1.0\" = {}"),
        "{model}"
    );
    assert!(model.contains("available = \"prod 1.2\""), "{model}");

    let check = ascribe(dir.path(), &["check"]);
    assert_eq!(code_of(&check), 0, "{}{}", stdout(&check), stderr(&check));
}

#[test]
fn a_second_release_of_the_same_line_is_refused() {
    let dir = repository(MODEL);
    let first = ascribe(
        dir.path(),
        &["versions", "release", "prod 1.1", "--next", "1.2"],
    );
    assert_eq!(code_of(&first), 0, "{}", stderr(&first));
    git(dir.path(), &["commit", "-q", "-am", "Release 1.1"]);
    let before = files(dir.path());
    let second = ascribe(
        dir.path(),
        &["versions", "release", "prod 1.1", "--next", "1.2"],
    );
    assert_eq!(code_of(&second), 2);
    assert!(
        stderr(&second).contains("live already"),
        "{}",
        stderr(&second)
    );
    assert_eq!(files(dir.path()), before);
}

#[test]
fn a_feature_with_no_line_to_move_to_stops_the_release_before_it_writes() {
    let dir = repository(MODEL);
    let before = files(dir.path());
    let out = ascribe(dir.path(), &["versions", "release", "prod 1.1"]);
    assert_eq!(code_of(&out), 2);
    assert!(stderr(&out).contains("--next"), "{}", stderr(&out));
    assert_eq!(files(dir.path()), before);
}

#[test]
fn a_versioned_build_keeps_the_old_line_live() {
    let model = format!("{MODEL}\n[builds.site]\nversions = \"prod\"\n");
    let dir = repository(&model);
    let out = ascribe(
        dir.path(),
        &["versions", "release", "prod 1.1", "--next", "1.2"],
    );
    assert_eq!(code_of(&out), 0, "{}", stderr(&out));
    let model = read(dir.path(), "ascribe.toml");
    assert!(model.contains("live = [\"1.1\", \"1.0\"]"), "{model}");
    assert!(!model.contains("archived"), "{model}");
}

#[test]
fn a_patch_nobody_planned_is_released_and_next_names_what_it_named() {
    let model = MODEL
        .replace("live = [\"1.0\"]", "live = [\"1.0.0\"]")
        .replace("planned = [\"1.1\"]", "planned = [\"1.1.0\"]")
        .replace("prod 1.1\"", "prod 1.1.0\"");
    let dir = repository(&model);
    let before = files(dir.path());
    let out = ascribe(dir.path(), &["versions", "release", "prod 1.0.1"]);
    assert_eq!(code_of(&out), 0, "{}", stderr(&out));
    let after = read(dir.path(), "ascribe.toml");
    assert!(after.contains("live = [\"1.0.1\"]"), "{after}");
    assert!(after.contains("planned = [\"1.1.0\"]"), "{after}");
    assert!(after.contains("available = \"prod 1.1.0\""), "{after}");
    assert!(after.contains("\"1.0.0\" = {}"), "{after}");
    // No mark names the patch.
    assert_eq!(files(dir.path())[1..], before[1..]);
}

#[test]
fn outside_a_repository_a_release_is_refused_unless_forced_and_a_dry_run_works() {
    let dir = project(MODEL);
    let before = files(dir.path());
    let refused = ascribe(
        dir.path(),
        &["versions", "release", "prod 1.1", "--next", "1.2"],
    );
    assert_eq!(code_of(&refused), 2);
    assert!(stderr(&refused).contains("--force"), "{}", stderr(&refused));
    assert_eq!(files(dir.path()), before);

    let dry = ascribe(
        dir.path(),
        &[
            "versions",
            "release",
            "prod 1.1",
            "--next",
            "1.2",
            "--dry-run",
        ],
    );
    assert_eq!(code_of(&dry), 0, "{}", stderr(&dry));
    assert!(stdout(&dry).contains("would be live"), "{}", stdout(&dry));
    assert_eq!(files(dir.path()), before);

    let forced = ascribe(
        dir.path(),
        &[
            "versions", "release", "prod 1.1", "--next", "1.2", "--force",
        ],
    );
    assert_eq!(code_of(&forced), 0, "{}", stderr(&forced));
    assert!(read(dir.path(), "ascribe.toml").contains("live = [\"1.1\"]"));
}

#[test]
fn a_changed_file_is_refused_by_name() {
    let dir = repository(MODEL);
    write(dir.path(), "docs/new.md", &format!("{NEW}\nMore.\n"));
    let out = ascribe(
        dir.path(),
        &["versions", "release", "prod 1.1", "--next", "1.2"],
    );
    assert_eq!(code_of(&out), 2);
    assert!(stderr(&out).contains("new.md"), "{}", stderr(&out));
    assert!(read(dir.path(), "ascribe.toml").contains("live = [\"1.0\"]"));
}

#[test]
fn the_release_json_lists_the_files_and_the_marks() {
    let dir = repository(MODEL);
    let out = ascribe(
        dir.path(),
        &[
            "versions",
            "release",
            "prod 1.1",
            "--next",
            "1.2",
            "--dry-run",
            "--format",
            "json",
        ],
    );
    assert_eq!(code_of(&out), 0, "{}", stderr(&out));
    let json: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(json["written"], false);
    assert_eq!(json["next"], "1.2");
    assert_eq!(json["next_added"], true);
    assert_eq!(json["archived"], "1.0");
    let files: Vec<&str> = json["files"]
        .as_array()
        .unwrap()
        .iter()
        .map(|f| f["file"].as_str().unwrap())
        .collect();
    assert_eq!(files, ["docs/new.md", "docs/page.md"]);
    assert_eq!(json["files"][1]["marks"].as_array().unwrap().len(), 3);
    assert_eq!(json["features"][0]["to"], "prod 1.2");
}

#[test]
fn list_gives_each_line_its_status() {
    let dir = project(MODEL);
    let out = ascribe(dir.path(), &["versions", "list", "--format", "json"]);
    assert_eq!(code_of(&out), 0, "{}", stderr(&out));
    let json: Value = serde_json::from_slice(&out.stdout).unwrap();
    let lines: Vec<(&str, &str)> = json["targets"][0]["lines"]
        .as_array()
        .unwrap()
        .iter()
        .map(|l| (l["line"].as_str().unwrap(), l["status"].as_str().unwrap()))
        .collect();
    assert_eq!(lines, [("1.1", "planned"), ("1.0", "current")]);
}

#[test]
fn changes_lists_what_one_line_has_and_the_other_doesnt() {
    let dir = project(MODEL);
    let out = ascribe(
        dir.path(),
        &["versions", "changes", "prod 1.0", "1.1", "--format", "json"],
    );
    assert_eq!(code_of(&out), 0, "{}", stderr(&out));
    let json: Value = serde_json::from_slice(&out.stdout).unwrap();
    let added: Vec<(&str, &str)> = json["added"]
        .as_array()
        .unwrap()
        .iter()
        .map(|c| (c["file"].as_str().unwrap(), c["kind"].as_str().unwrap()))
        .collect();
    assert_eq!(
        added,
        [
            ("docs/new.md", "page"),
            ("docs/page.md", "block"),
            ("docs/page.md", "row"),
            ("docs/page.md", "row"),
        ]
    );
    assert_eq!(json["removed"].as_array().unwrap().len(), 0);
}

#[test]
fn a_line_the_list_doesnt_declare_is_exit_code_2() {
    let dir = project(MODEL);
    let out = ascribe(dir.path(), &["versions", "changes", "prod 0.9", "1.1"]);
    assert_eq!(code_of(&out), 2);
    assert!(stderr(&out).contains("doesn't declare"), "{}", stderr(&out));
}

/// Two lines of `prod` live, and a build versioned by it.
const VERSIONED: &str = r#"spec = "0.1"

[project]
content-root = "docs"

[dimensions.release]
values = ["prod"]

[releases.prod]
live = ["1.0", "1.1"]
archived = { "0.9" = {} }

[builds.site]
availability = "badge"
versions = "prod"
"#;

fn archive_project(model: &str) -> TempDir {
    let dir = tempfile::tempdir().unwrap();
    write(dir.path(), "ascribe.toml", model);
    write(dir.path(), "docs/a.md", "---\ntitle: A\n---\n\nSee [B](b.md).\n");
    write(
        dir.path(),
        "docs/b.md",
        "---\ntitle: B\navailable: prod 1.1\n---\n\nNew in 1.1.\n",
    );
    dir
}

#[test]
fn archive_writes_a_line_into_the_folder_and_nothing_into_the_project() {
    let dir = archive_project(VERSIONED);
    let out = ascribe(
        dir.path(),
        &["versions", "archive", "prod 1.0", "--to", "out/1.0", "--format", "json"],
    );
    assert_eq!(code_of(&out), 0, "{}", stderr(&out));
    let json: Value = serde_json::from_str(&stdout(&out)).unwrap();
    assert_eq!(json["status"], "live");
    assert_eq!(json["build"], "site");
    assert_eq!(json["pages"], 1);
    assert_eq!(json["folder"], "out/1.0");
    assert_eq!(json["unlinked"][0]["file"], "docs/a.md");
    assert_eq!(json["unlinked"][0]["line"], "1.1");
    assert!(read(dir.path(), "out/1.0/a.html").contains("See B."));
    assert!(!dir.path().join("out/1.0/b.html").exists());
    assert_eq!(read(dir.path(), "ascribe.toml"), VERSIONED);
    // Again, into the same folder: its own archive is replaced.
    let again = ascribe(dir.path(), &["versions", "archive", "prod 1.0", "--to", "out/1.0"]);
    assert_eq!(code_of(&again), 0, "{}", stderr(&again));
}

#[test]
fn archive_says_so_when_the_line_is_current() {
    let dir = archive_project(VERSIONED);
    let out = ascribe(dir.path(), &["versions", "archive", "prod 1.1", "--to", "out"]);
    assert_eq!(code_of(&out), 0, "{}", stderr(&out));
    assert!(stdout(&out).starts_with("prod 1.1 is the current line"), "{}", stdout(&out));
}

#[test]
fn archive_refuses_a_folder_that_isnt_its_own() {
    let dir = archive_project(VERSIONED);
    write(dir.path(), "out/keep.txt", "mine");
    let out = ascribe(dir.path(), &["versions", "archive", "prod 1.0", "--to", "out"]);
    assert_eq!(code_of(&out), 2);
    assert!(stderr(&out).contains("isn't empty"), "{}", stderr(&out));
    assert_eq!(read(dir.path(), "out/keep.txt"), "mine");
}

#[test]
fn archive_refuses_an_archived_line_and_a_target_no_build_versions() {
    let dir = archive_project(VERSIONED);
    let out = ascribe(dir.path(), &["versions", "archive", "prod 0.9", "--to", "out"]);
    assert_eq!(code_of(&out), 2);
    assert!(stderr(&out).contains("is archived"), "{}", stderr(&out));
    let unversioned = VERSIONED.replace("versions = \"prod\"\n", "");
    let dir = archive_project(&unversioned);
    let out = ascribe(dir.path(), &["versions", "archive", "prod 1.0", "--to", "out"]);
    assert_eq!(code_of(&out), 2);
    assert!(stderr(&out).contains("no build is versioned by prod"), "{}", stderr(&out));
}

#[test]
fn archive_needs_build_when_two_builds_are_versioned_by_the_target() {
    let model = format!("{VERSIONED}\n[builds.other]\navailability = \"badge\"\nversions = \"prod\"\n");
    let dir = archive_project(&model);
    let out = ascribe(dir.path(), &["versions", "archive", "prod 1.0", "--to", "out"]);
    assert_eq!(code_of(&out), 2);
    assert!(stderr(&out).contains("site, other"), "{}", stderr(&out));
    let out = ascribe(
        dir.path(),
        &["versions", "archive", "prod 1.0", "--to", "out", "--build", "other"],
    );
    assert_eq!(code_of(&out), 0, "{}", stderr(&out));
}

#[test]
fn archive_refuses_a_line_whose_build_has_errors() {
    let dir = archive_project(VERSIONED);
    write(dir.path(), "docs/c.md", "---\ntitle: C\n---\n\nSee [x](missing.md).\n");
    let out = ascribe(dir.path(), &["versions", "archive", "prod 1.0", "--to", "out"]);
    assert_eq!(code_of(&out), 2);
    assert!(stderr(&out).contains("nothing was written"), "{}", stderr(&out));
    assert!(!dir.path().join("out").exists());
}
