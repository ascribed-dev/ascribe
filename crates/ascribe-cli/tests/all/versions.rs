//! `ascribe versions`, by running the binary in temporary projects: what
//! `release` writes and refuses, in a repository and out of one, and what
//! `list` and `changes` report. The JSON's shapes are held by the schemas.

#![allow(clippy::expect_used, clippy::unwrap_used, clippy::panic)]

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use serde_json::Value;
use tempfile::TempDir;

/// One site that knows its versions: `1.0` is live, and `1.1` planned,
/// which `planned` names.
const MODEL: &str = r#"spec = "0.1"

[project]
content-root = "docs"

[dimensions.release]
label = "Product"
values = ["prod"]

[releases.prod]
live = ["1.0"]
planned = ["1.1"]
"#;

/// A page marked as planned in its frontmatter.
const NEW: &str = "---\ntitle: New\navailable: prod planned\n---\n\nText.\n";

/// A page with `planned` on a block and on two rows, one with a state,
/// beside a mark for a release already out.
const PAGE: &str = "---\ntitle: Page\n---\n\n## Options\n\nIntro.\n\n@available: prod planned\nA new paragraph.\n\n@available: prod 1.0\nAn old one.\n\n| Option | Does |\n|---|---|\n| `a` {available=\"prod planned\"} | A. |\n| `b` {available=\"prod beta planned\"} | B. |\n";

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
fn a_release_rewrites_the_planned_marks_and_archives_the_old_line() {
    let dir = repository(MODEL);
    let out = ascribe(
        dir.path(),
        &["versions", "release", "prod 1.1", "--next", "1.2"],
    );
    assert_eq!(code_of(&out), 0, "{}", stderr(&out));
    let text = stdout(&out);
    assert!(text.contains("prod 1.1 is live"), "{text}");
    assert!(
        text.contains("3 marks from `prod planned` to `prod 1.1`, in 2 files"),
        "{text}"
    );
    assert!(
        text.contains("1 mark from `prod beta planned` to `prod beta 1.1`, in 1 file"),
        "{text}"
    );
    assert!(text.contains("prod 1.2 is planned"), "{text}");

    assert!(read(dir.path(), "docs/new.md").contains("available: prod 1.1\n"));
    let page = read(dir.path(), "docs/page.md");
    assert!(
        page.contains("@available: prod 1.1\nA new paragraph."),
        "{page}"
    );
    // 1.0 is archived, so its mark says nothing and goes.
    assert!(page.contains("A new paragraph.\n\nAn old one."), "{page}");
    assert!(page.contains("`a` {available=\"prod 1.1\"}"), "{page}");
    assert!(page.contains("`b` {available=\"prod beta 1.1\"}"), "{page}");

    let model = read(dir.path(), "ascribe.toml");
    assert!(model.contains("live = [\"1.1\"]"), "{model}");
    assert!(model.contains("planned = [\"1.2\"]"), "{model}");
    assert!(
        model.contains("[releases.prod.archived]\n\"1.0\" = {}"),
        "{model}"
    );

    let check = ascribe(dir.path(), &["check"]);
    assert_eq!(code_of(&check), 0, "{}{}", stdout(&check), stderr(&check));
}

/// A page with marks naming the live line, 1.0, and an older one, 0.9,
/// beside a feature marked 1.0.
const OLD_MARKS: &str = "---\ntitle: Old\n---\n\n@available: prod 1.0\nGone.\n\n@available: prod beta 1.0\nStill beta.\n\n@available: prod 0.9\nOlder.\n\n@available: prod planned\nNew.\n\n| Option | Does |\n|---|---|\n| `a` {available=\"prod 1.0.2\"} | A. |\n| `b` | B. |\n";

#[test]
fn a_release_removes_the_marks_of_the_line_it_archives_and_leaves_the_others() {
    let model = format!("{MODEL}\n[features.thing]\nname = \"Thing\"\navailable = \"prod 1.0\"\n");
    let dir = project(&model);
    write(dir.path(), "docs/old.md", OLD_MARKS);
    git(dir.path(), &["init", "-q"]);
    git(dir.path(), &["add", "-A"]);
    git(dir.path(), &["commit", "-q", "-m", "Start"]);

    let before: Vec<String> = ["ascribe.toml", "docs/old.md"]
        .iter()
        .map(|p| read(dir.path(), p))
        .collect();
    let dry = ascribe(
        dir.path(),
        &["versions", "release", "prod 1.1", "--dry-run"],
    );
    assert_eq!(code_of(&dry), 0, "{}", stderr(&dry));
    let text = stdout(&dry);
    assert!(
        text.contains(
            "With prod 1.0 archived, 5 marks say nothing: would remove 3, would rewrite 2."
        ),
        "{text}"
    );
    assert!(
        text.contains("docs/old.md:5: `prod 1.0`, removed"),
        "{text}"
    );
    assert!(
        text.contains("docs/old.md:8: `prod beta 1.0` to `prod beta`"),
        "{text}"
    );
    assert!(
        text.contains("feature thing: `prod 1.0` to `prod`"),
        "{text}"
    );
    assert!(!text.contains("prod 0.9"), "{text}");
    let after: Vec<String> = ["ascribe.toml", "docs/old.md"]
        .iter()
        .map(|p| read(dir.path(), p))
        .collect();
    assert_eq!(before, after, "a dry run writes nothing");

    let json = ascribe(
        dir.path(),
        &[
            "versions",
            "release",
            "prod 1.1",
            "--dry-run",
            "--format",
            "json",
        ],
    );
    let json: Value = serde_json::from_slice(&json.stdout).unwrap();
    let cleared = &json["cleared"];
    assert_eq!(cleared.as_array().unwrap().len(), 2, "{json}");
    assert_eq!(cleared[0]["file"], "docs/old.md");
    assert_eq!(cleared[0]["marks"][0]["from"], "prod 1.0");
    assert_eq!(cleared[0]["marks"][0]["to"], Value::Null);
    assert_eq!(cleared[0]["marks"][1]["to"], "prod beta");
    assert_eq!(json["cleared_features"][0]["key"], "thing");
    assert_eq!(json["cleared_features"][0]["to"], "prod");

    let out = ascribe(dir.path(), &["versions", "release", "prod 1.1"]);
    assert_eq!(code_of(&out), 0, "{}", stderr(&out));
    let old = read(dir.path(), "docs/old.md");
    assert_eq!(
        old,
        "---\ntitle: Old\n---\n\nGone.\n\n@available: prod beta\nStill beta.\n\n@available: prod 0.9\nOlder.\n\n@available: prod 1.1\nNew.\n\n| Option | Does |\n|---|---|\n| `a` | A. |\n| `b` | B. |\n"
    );
    assert!(
        read(dir.path(), "ascribe.toml").contains("available = \"prod\"\n"),
        "{}",
        read(dir.path(), "ascribe.toml")
    );

    // What's left of 1.0 says nothing to `ascribe check`; the mark for 0.9,
    // which said nothing before the release, is still reported.
    let check = ascribe(dir.path(), &["check", "--format", "json"]);
    let report: Value = serde_json::from_slice(&check.stdout).unwrap();
    let same: Vec<String> = report["diagnostics"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|d| d["slug"] == "available-same-in-every-line")
        .map(|d| d["message"].as_str().unwrap().to_owned())
        .collect();
    assert_eq!(same.len(), 1, "{same:?}");
    assert!(same[0].contains("`prod 0.9`"), "{same:?}");
}

/// A repository of two projects: `docs`, whose `ascribe.toml` writes the
/// lines, and `site`, which takes them with `[from]`.
fn two_projects() -> TempDir {
    let dir = tempfile::tempdir().unwrap();
    write(dir.path(), "docs/ascribe.toml", MODEL);
    write(dir.path(), "docs/docs/new.md", NEW);
    write(dir.path(), "docs/docs/page.md", PAGE);
    write(
        dir.path(),
        "site/ascribe.toml",
        "spec = \"0.1\"\n\n[project]\ncontent-root = \"pages\"\n\n[from.\"../docs/ascribe.toml\"]\ntake = [\"dimensions.release\", \"releases\"]\n",
    );
    write(
        dir.path(),
        "site/pages/index.md",
        "---\ntitle: Home\n---\n\nWelcome.\n",
    );
    git(dir.path(), &["init", "-q"]);
    git(dir.path(), &["add", "-A"]);
    git(dir.path(), &["commit", "-q", "-m", "Start"]);
    dir
}

#[test]
fn a_release_in_a_project_that_takes_the_lines_is_made_where_theyre_written() {
    let dir = two_projects();
    let site = dir.path().join("site");
    let before = read(dir.path(), "site/ascribe.toml");
    let out = ascribe(&site, &["versions", "release", "prod 1.1", "--next", "1.2"]);
    assert_eq!(code_of(&out), 0, "{}", stderr(&out));
    let text = stdout(&out);
    assert!(
        text.contains("The lines of prod are written in ../docs/ascribe.toml, which [from] takes them from: this changes that project."),
        "{text}"
    );
    assert!(text.contains("prod 1.1 is live"), "{text}");

    // The lines move in the file that writes them, and that project's marks
    // with them; this project's file is as it was.
    let model = read(dir.path(), "docs/ascribe.toml");
    assert!(model.contains("live = [\"1.1\"]"), "{model}");
    assert!(model.contains("planned = [\"1.2\"]"), "{model}");
    assert!(read(dir.path(), "docs/docs/new.md").contains("available: prod 1.1\n"));
    assert_eq!(read(dir.path(), "site/ascribe.toml"), before);

    // Both projects have the line live, from the one release.
    for project in ["docs", "site"] {
        let list = ascribe(&dir.path().join(project), &["versions", "list"]);
        assert_eq!(code_of(&list), 0, "{}", stderr(&list));
        let listed = stdout(&list);
        assert!(
            listed.contains("1.1") && listed.contains("current"),
            "{project}: {listed}"
        );
        let check = ascribe(&dir.path().join(project), &["check"]);
        assert_eq!(
            code_of(&check),
            0,
            "{project}: {}{}",
            stdout(&check),
            stderr(&check)
        );
    }
}

#[test]
fn the_release_json_names_the_file_the_lines_are_written_in() {
    let dir = two_projects();
    let out = ascribe(
        &dir.path().join("site"),
        &[
            "versions",
            "release",
            "prod 1.1",
            "--dry-run",
            "--format",
            "json",
        ],
    );
    assert_eq!(code_of(&out), 0, "{}", stderr(&out));
    let json: Value = serde_json::from_str(&stdout(&out)).unwrap();
    assert_eq!(json["from"], "../docs/ascribe.toml");
    assert_eq!(json["files"][0]["file"], "docs/new.md");
    // Run where the lines are written, it's the project's own.
    let out = ascribe(
        &dir.path().join("docs"),
        &[
            "versions",
            "release",
            "prod 1.1",
            "--dry-run",
            "--format",
            "json",
        ],
    );
    let json: Value = serde_json::from_str(&stdout(&out)).unwrap();
    assert_eq!(json["from"], Value::Null);
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
fn a_release_with_no_next_line_leaves_nothing_planned() {
    let dir = repository(MODEL);
    let out = ascribe(dir.path(), &["versions", "release", "prod 1.1"]);
    assert_eq!(code_of(&out), 0, "{}", stderr(&out));
    let model = read(dir.path(), "ascribe.toml");
    assert!(model.contains("planned = []"), "{model}");
    assert!(read(dir.path(), "docs/new.md").contains("available: prod 1.1\n"));
    let check = ascribe(dir.path(), &["check"]);
    assert_eq!(code_of(&check), 0, "{}{}", stdout(&check), stderr(&check));
}

#[test]
fn planned_with_no_planned_line_is_an_error_at_the_mark() {
    let dir = project(&MODEL.replace("planned = [\"1.1\"]\n", ""));
    let out = ascribe(dir.path(), &["check", "--format", "json"]);
    assert_eq!(code_of(&out), 1, "{}{}", stdout(&out), stderr(&out));
    let text = stdout(&out);
    assert_eq!(text.matches("available-planned-line").count(), 4, "{text}");
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
fn a_patch_nobody_planned_is_released_and_planned_names_what_it_named() {
    let model = MODEL
        .replace("live = [\"1.0\"]", "live = [\"1.0.0\"]")
        .replace("planned = [\"1.1\"]", "planned = [\"1.1.0\"]");
    let dir = repository(&model);
    let before = files(dir.path());
    let out = ascribe(dir.path(), &["versions", "release", "prod 1.0.1"]);
    assert_eq!(code_of(&out), 0, "{}", stderr(&out));
    let after = read(dir.path(), "ascribe.toml");
    assert!(after.contains("live = [\"1.0.1\"]"), "{after}");
    assert!(after.contains("planned = [\"1.1.0\"]"), "{after}");
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
    assert_eq!(json["schema_version"], 2);
    assert_eq!(json["files"][0]["marks"][0]["from"], "prod planned");
    assert_eq!(json["files"][0]["marks"][0]["to"], "prod 1.1");
    assert!(json.get("features").is_none());
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
    write(
        dir.path(),
        "docs/a.md",
        "---\ntitle: A\n---\n\nSee [B](b.md).\n",
    );
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
        &[
            "versions", "archive", "prod 1.0", "--to", "out/1.0", "--format", "json",
        ],
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
    let again = ascribe(
        dir.path(),
        &["versions", "archive", "prod 1.0", "--to", "out/1.0"],
    );
    assert_eq!(code_of(&again), 0, "{}", stderr(&again));
}

#[test]
fn archive_says_so_when_the_line_is_current() {
    let dir = archive_project(VERSIONED);
    let out = ascribe(
        dir.path(),
        &["versions", "archive", "prod 1.1", "--to", "out"],
    );
    assert_eq!(code_of(&out), 0, "{}", stderr(&out));
    assert!(
        stdout(&out).starts_with("prod 1.1 is the current line"),
        "{}",
        stdout(&out)
    );
}

#[test]
fn archive_refuses_a_folder_that_isnt_its_own() {
    let dir = archive_project(VERSIONED);
    write(dir.path(), "out/keep.txt", "mine");
    let out = ascribe(
        dir.path(),
        &["versions", "archive", "prod 1.0", "--to", "out"],
    );
    assert_eq!(code_of(&out), 2);
    assert!(stderr(&out).contains("isn't empty"), "{}", stderr(&out));
    assert_eq!(read(dir.path(), "out/keep.txt"), "mine");
}

#[test]
fn archive_refuses_an_archived_line_and_a_target_no_build_versions() {
    let dir = archive_project(VERSIONED);
    let out = ascribe(
        dir.path(),
        &["versions", "archive", "prod 0.9", "--to", "out"],
    );
    assert_eq!(code_of(&out), 2);
    assert!(stderr(&out).contains("is archived"), "{}", stderr(&out));
    let unversioned = VERSIONED.replace("versions = \"prod\"\n", "");
    let dir = archive_project(&unversioned);
    let out = ascribe(
        dir.path(),
        &["versions", "archive", "prod 1.0", "--to", "out"],
    );
    assert_eq!(code_of(&out), 2);
    assert!(
        stderr(&out).contains("no build is versioned by prod"),
        "{}",
        stderr(&out)
    );
}

#[test]
fn archive_needs_build_when_two_builds_are_versioned_by_the_target() {
    let model =
        format!("{VERSIONED}\n[builds.other]\navailability = \"badge\"\nversions = \"prod\"\n");
    let dir = archive_project(&model);
    let out = ascribe(
        dir.path(),
        &["versions", "archive", "prod 1.0", "--to", "out"],
    );
    assert_eq!(code_of(&out), 2);
    assert!(stderr(&out).contains("site, other"), "{}", stderr(&out));
    let out = ascribe(
        dir.path(),
        &[
            "versions", "archive", "prod 1.0", "--to", "out", "--build", "other",
        ],
    );
    assert_eq!(code_of(&out), 0, "{}", stderr(&out));
}

#[test]
fn archive_refuses_a_line_whose_build_has_errors() {
    let dir = archive_project(VERSIONED);
    write(
        dir.path(),
        "docs/c.md",
        "---\ntitle: C\n---\n\nSee [x](missing.md).\n",
    );
    let out = ascribe(
        dir.path(),
        &["versions", "archive", "prod 1.0", "--to", "out"],
    );
    assert_eq!(code_of(&out), 2);
    assert!(
        stderr(&out).contains("nothing was written"),
        "{}",
        stderr(&out)
    );
    assert!(!dir.path().join("out").exists());
}

/// Three lines, and a page rewritten for each: `b.md` replaces `a.md` in
/// 1.1, and `c.md` replaces `b.md` in 1.2.
const REWRITTEN: &str = r#"spec = "0.1"

[project]
content-root = "docs"

[dimensions.release]
label = "Product"
values = ["prod"]

[releases.prod]
live = ["1.0", "1.1", "1.2"]
"#;

fn rewritten() -> TempDir {
    let dir = tempfile::tempdir().unwrap();
    write(dir.path(), "ascribe.toml", REWRITTEN);
    write(
        dir.path(),
        "docs/a.md",
        "---\ntitle: A\navailable: prod (ga 1.0, removed 1.1)\n---\n\nText.\n",
    );
    write(
        dir.path(),
        "docs/b.md",
        "---\ntitle: B\navailable: prod (ga 1.1, removed 1.2)\nreplaces: a.md\n---\n\nText.\n",
    );
    write(
        dir.path(),
        "docs/c.md",
        "---\ntitle: C\navailable: prod 1.2\nreplaces: b.md\n---\n\nText.\n",
    );
    dir
}

/// Each entry of `changes`' JSON list `key`, as its file and the file it's
/// in place of.
fn entries(dir: &Path, from: &str, to: &str, key: &str) -> Vec<(String, Option<String>)> {
    let out = ascribe(dir, &["versions", "changes", from, to, "--format", "json"]);
    assert_eq!(code_of(&out), 0, "{}", stderr(&out));
    let json: Value = serde_json::from_slice(&out.stdout).unwrap();
    json[key]
        .as_array()
        .unwrap()
        .iter()
        .map(|c| {
            (
                c["file"].as_str().unwrap().to_owned(),
                c["in_place_of"]["file"].as_str().map(str::to_owned),
            )
        })
        .collect()
}

#[test]
fn a_page_added_in_place_of_one_removed_is_one_change() {
    let dir = rewritten();
    assert_eq!(
        entries(dir.path(), "prod 1.0", "1.1", "added"),
        [("docs/b.md".to_owned(), Some("docs/a.md".to_owned()))]
    );
    assert!(entries(dir.path(), "prod 1.0", "1.1", "removed").is_empty());
}

#[test]
fn going_back_a_line_the_replaced_page_is_in_place_of_its_replacement() {
    let dir = rewritten();
    assert_eq!(
        entries(dir.path(), "prod 1.1", "1.0", "added"),
        [("docs/a.md".to_owned(), Some("docs/b.md".to_owned()))]
    );
    assert!(entries(dir.path(), "prod 1.1", "1.0", "removed").is_empty());
}

#[test]
fn a_chain_of_replaces_pairs_pages_two_lines_apart() {
    let dir = rewritten();
    assert_eq!(
        entries(dir.path(), "prod 1.0", "1.2", "added"),
        [("docs/c.md".to_owned(), Some("docs/a.md".to_owned()))]
    );
    assert!(entries(dir.path(), "prod 1.0", "1.2", "removed").is_empty());
}

#[test]
fn the_text_and_the_notes_name_the_page_replaced() {
    let dir = rewritten();
    let text = ascribe(dir.path(), &["versions", "changes", "prod 1.0", "1.1"]);
    assert_eq!(
        stdout(&text),
        "In 1.1 and not 1.0:\n  docs/b.md:3: the page \"B\", in place of \"A\" at docs/a.md:3\n"
    );
    let notes = ascribe(
        dir.path(),
        &[
            "versions", "changes", "prod 1.0", "1.1", "--format", "markdown",
        ],
    );
    let notes = stdout(&notes);
    assert!(
        notes.contains("- New: the page, in place of \"A\" in `docs/a.md`"),
        "{notes}"
    );
    assert!(!notes.contains("Removed"), "{notes}");
}

/// A project with `REWRITTEN`'s lines and these pages, each `(file, title,
/// available, replaces)`.
fn pages(list: &[(&str, &str, &str, Option<&str>)]) -> TempDir {
    let dir = tempfile::tempdir().unwrap();
    write(dir.path(), "ascribe.toml", REWRITTEN);
    for (file, title, available, replaces) in list {
        let replaces = replaces.map_or(String::new(), |r| format!("replaces: {r}\n"));
        write(
            dir.path(),
            &format!("docs/{file}"),
            &format!("---\ntitle: {title}\navailable: {available}\n{replaces}---\n\nText.\n"),
        );
    }
    dir
}

#[test]
fn a_page_in_a_line_stops_the_chain() {
    // `b` took `a`'s place and is in both lines, so `c`, replacing `b`,
    // isn't in place of `a`.
    let dir = pages(&[
        ("a.md", "A", "prod (ga 1.0, removed 1.1)", None),
        ("b.md", "B", "prod 1.0", Some("a.md")),
        ("c.md", "C", "prod 1.1", Some("b.md")),
    ]);
    assert_eq!(
        entries(dir.path(), "prod 1.0", "1.1", "added"),
        [("docs/c.md".to_owned(), None)]
    );
    assert_eq!(
        entries(dir.path(), "prod 1.0", "1.1", "removed"),
        [("docs/a.md".to_owned(), None)]
    );
}

#[test]
fn a_page_added_too_stops_the_chain() {
    // `y` replaces `z` itself; `x`, replacing `y`, doesn't reach `z`
    // through it, though it comes first.
    let dir = pages(&[
        ("x.md", "X", "prod 1.1", Some("y.md")),
        ("y.md", "Y", "prod 1.1", Some("z.md")),
        ("z.md", "Z", "prod (ga 1.0, removed 1.1)", None),
    ]);
    assert_eq!(
        entries(dir.path(), "prod 1.0", "1.1", "added"),
        [
            ("docs/x.md".to_owned(), None),
            ("docs/y.md".to_owned(), Some("docs/z.md".to_owned())),
        ]
    );
    assert!(entries(dir.path(), "prod 1.0", "1.1", "removed").is_empty());
}

/// `prod` with 1.1 current and 1.0 live, or with 1.0 live and 1.1 planned,
/// and a link to a heading 1.1 removes: an error in 1.1 alone.
fn removed_heading(lines: &str) -> TempDir {
    let dir = tempfile::tempdir().unwrap();
    let model = format!(
        "spec = \"0.1\"\n\n[project]\ncontent-root = \"docs\"\n\n[dimensions.release]\nvalues = [\"prod\"]\n\n[releases.prod]\n{lines}\n\n[builds.site]\navailability = \"badge\"\nversions = \"prod\"\n"
    );
    write(dir.path(), "ascribe.toml", &model);
    write(
        dir.path(),
        "docs/a.md",
        "---\ntitle: A\n---\n\nSee [old](b.md#old).\n",
    );
    write(
        dir.path(),
        "docs/b.md",
        "---\ntitle: B\n---\n\nText.\n\n## Old\n@available: prod (ga 1.0, removed 1.1)\n\nOld.\n",
    );
    dir
}

#[test]
fn archive_checks_the_line_it_archives_and_no_other() {
    // The error is in the current line, 1.1; 1.0 has none.
    let dir = removed_heading("live = [\"1.1\", \"1.0\"]");
    let out = ascribe(
        dir.path(),
        &["versions", "archive", "prod 1.0", "--to", "out"],
    );
    assert_eq!(code_of(&out), 0, "{}", stderr(&out));
    let out = ascribe(
        dir.path(),
        &["versions", "archive", "prod 1.1", "--to", "out2"],
    );
    assert_eq!(code_of(&out), 2);
    assert!(stderr(&out).contains("in build `site`"), "{}", stderr(&out));
    // A planned line's errors, which `ascribe check` doesn't see.
    let dir = removed_heading("live = [\"1.0\"]\nplanned = [\"1.1\"]");
    let check = ascribe(dir.path(), &["check"]);
    assert_eq!(code_of(&check), 0, "{}", stderr(&check));
    let out = ascribe(
        dir.path(),
        &["versions", "archive", "prod 1.1", "--to", "out"],
    );
    assert_eq!(code_of(&out), 2);
    assert!(
        stderr(&out).contains("in build `site@1.1`"),
        "{}",
        stderr(&out)
    );
    assert!(!dir.path().join("out").exists());
}

// -- `versions retire` ------------------------------------------------------

/// `prod` 1.0 and 1.1 live, beside `cloud`, which has no versions, and one
/// build versioned by `prod`.
const RETIRING: &str = r#"spec = "0.1"

[project]
content-root = "docs"

[dimensions.deployment]
values = ["cloud", "prod"]
versionless = ["cloud"]

[releases.prod]
live = ["1.0", "1.1"]
ends = { "1.0" = 2026-06-30 }

[builds.site]
availability = "badge"
versions = "prod"
"#;

/// A page only `prod` has, with a block every line has, a block, a
/// section, and a row no live line has, and a mark that says nothing once
/// its version goes; and a page every line has, about both targets.
const PROD: &str = "---\ntitle: Prod\navailable: prod\n---\n\nIntro.\n\n@available: prod 1.0\nSince 1.0, which says nothing now.\n\n@available: prod (ga 0.9, removed 1.1)\nGone in 1.1.\n\n## Old settings\n@available: prod (ga 0.8, removed 1.1)\n\nThey went.\n\n## Options\n\n| Option | Does |\n|---|---|\n| `a` {available=\"prod (ga 0.9, removed 1.1)\"} | A. |\n| `b` | B. |\n";

const BOTH: &str = "---\ntitle: Both\navailable: cloud, prod 1.0\n---\n\nFor everyone.\n\n@available: prod deprecated 1.1\nOld, still here.\n";

fn retiring_repository() -> TempDir {
    let dir = tempfile::tempdir().unwrap();
    write(dir.path(), "ascribe.toml", RETIRING);
    write(dir.path(), "docs/prod.md", PROD);
    write(dir.path(), "docs/both.md", BOTH);
    git(dir.path(), &["init", "-q"]);
    git(dir.path(), &["add", "-A"]);
    git(dir.path(), &["commit", "-q", "-m", "Start"]);
    dir
}

fn retire(dir: &Path, extra: &[&str]) -> Output {
    let mut args = vec![
        "versions",
        "retire",
        "prod 1.0",
        "--url",
        "https://example.com/1.0/",
    ];
    args.extend_from_slice(extra);
    Command::new(env!("CARGO_BIN_EXE_ascribe"))
        .current_dir(dir)
        .args(&args)
        .env("NO_COLOR", "1")
        .env("ASCRIBE_TODAY", "2026-10-16")
        .env_remove("GIT_DIR")
        .env_remove("GIT_WORK_TREE")
        .output()
        .expect("run ascribe")
}

#[test]
fn a_retirement_in_a_project_that_takes_the_lines_is_made_where_theyre_written() {
    let dir = tempfile::tempdir().unwrap();
    write(dir.path(), "docs/ascribe.toml", RETIRING);
    write(dir.path(), "docs/docs/prod.md", PROD);
    write(dir.path(), "docs/docs/both.md", BOTH);
    let site = "spec = \"0.1\"\n\n[project]\ncontent-root = \"pages\"\n\n[from.\"../docs/ascribe.toml\"]\ntake = [\"dimensions.deployment\", \"releases\"]\n";
    write(dir.path(), "site/ascribe.toml", site);
    write(
        dir.path(),
        "site/pages/index.md",
        "---\ntitle: Home\n---\n\nWelcome.\n",
    );
    git(dir.path(), &["init", "-q"]);
    git(dir.path(), &["add", "-A"]);
    git(dir.path(), &["commit", "-q", "-m", "Start"]);

    let out = retire(&dir.path().join("site"), &["--force"]);
    assert_eq!(code_of(&out), 0, "{}", stderr(&out));
    assert!(
        stdout(&out).contains("The lines of prod are written in ../docs/ascribe.toml"),
        "{}",
        stdout(&out)
    );
    let model = read(dir.path(), "docs/ascribe.toml");
    assert!(model.contains("live = [\"1.1\"]"), "{model}");
    assert!(
        model.contains("\"1.0\" = { url = \"https://example.com/1.0/\""),
        "{model}"
    );
    assert_eq!(read(dir.path(), "site/ascribe.toml"), site);
}

#[test]
fn retire_without_an_address_names_the_archive_command() {
    let dir = retiring_repository();
    let out = ascribe(dir.path(), &["versions", "retire", "prod 1.0"]);
    assert_eq!(code_of(&out), 2, "{}", stderr(&out));
    assert!(
        stderr(&out).contains("ascribe versions archive \"prod 1.0\""),
        "{}",
        stderr(&out)
    );
    assert_eq!(read(dir.path(), "ascribe.toml"), RETIRING);
}

#[test]
fn retire_where_no_build_is_versioned_says_a_release_keeps_the_list() {
    let docs = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs");
    let list = ascribe(&docs, &["versions", "list", "--format", "json"]);
    let list: Value = serde_json::from_slice(&list.stdout).unwrap();
    let current = list["targets"][0]["lines"]
        .as_array()
        .unwrap()
        .iter()
        .find(|l| l["status"] == "current")
        .unwrap()["line"]
        .as_str()
        .unwrap()
        .to_owned();
    let line = format!("ascribe {current}");
    let out = ascribe(
        &docs,
        &[
            "versions",
            "retire",
            &line,
            "--url",
            "https://example.com/",
            "--dry-run",
        ],
    );
    assert_eq!(code_of(&out), 2, "{}", stderr(&out));
    assert!(stderr(&out).contains("no build is versioned by `ascribe`"));
    assert!(stderr(&out).contains("`ascribe versions release` keeps its list"));
}

#[test]
fn retire_refuses_the_current_line_and_a_line_that_isnt_the_oldest() {
    let dir = retiring_repository();
    let out = ascribe(
        dir.path(),
        &["versions", "retire", "prod 1.1", "--url", "https://x/"],
    );
    assert_eq!(code_of(&out), 2);
    assert!(
        stderr(&out).contains("is the current line"),
        "{}",
        stderr(&out)
    );
    let out = ascribe(
        dir.path(),
        &["versions", "retire", "prod 0.9", "--url", "https://x/"],
    );
    assert_eq!(code_of(&out), 2);
    assert!(stderr(&out).contains("isn't declared"), "{}", stderr(&out));
}

#[test]
fn retire_archives_the_line_and_rewrites_the_marks_without_deleting_anything() {
    let dir = retiring_repository();
    let out = retire(dir.path(), &[]);
    assert_eq!(code_of(&out), 0, "{}", stderr(&out));
    let model = read(dir.path(), "ascribe.toml");
    assert!(model.contains("live = [\"1.1\"]"), "{model}");
    // `ends` gives the day, and leaves.
    assert!(
        model.contains("\"1.0\" = { url = \"https://example.com/1.0/\", ended = 2026-06-30 }"),
        "{model}"
    );
    assert!(!model.contains("ends"), "{model}");
    let prod = read(dir.path(), "docs/prod.md");
    assert!(prod.contains("@available: prod\nSince 1.0"), "{prod}");
    // Content in no live line is listed, not deleted.
    assert!(prod.contains("Gone in 1.1."), "{prod}");
    assert!(prod.contains("## Old settings"), "{prod}");
    assert!(prod.contains("| `a` {available="), "{prod}");
    let both = read(dir.path(), "docs/both.md");
    assert!(both.contains("available: cloud, prod\n"), "{both}");
    assert!(both.contains("@available: prod deprecated\n"), "{both}");
    let text = stdout(&out);
    assert!(
        text.contains("prod 1.0 is archived, at https://example.com/1.0/."),
        "{text}"
    );
    assert!(text.contains("No live line's output changed."), "{text}");
    assert!(
        text.contains("1 section, 1 block and 1 row are in no live line."),
        "{text}"
    );
    assert!(
        text.contains("1 mark says nothing once rewritten"),
        "{text}"
    );

    // Run twice, the second run refuses: the line isn't live.
    let again = retire(dir.path(), &[]);
    assert_eq!(code_of(&again), 2);
    assert!(
        stderr(&again).contains("isn't live: it's archived already"),
        "{}",
        stderr(&again)
    );
}

#[test]
fn retire_records_the_lines_addresses_in_the_addresses_file_and_the_build_redirects_them() {
    let dir = retiring_repository();
    // A page only 1.0 has, which goes to the archive.
    write(
        dir.path(),
        "docs/old.md",
        "---\ntitle: Old\navailable: prod (ga 0.9, removed 1.1)\n---\n\nGone in 1.1.\n",
    );
    let init = ascribe(dir.path(), &["addresses", "init"]);
    assert_eq!(code_of(&init), 0, "{}", stderr(&init));
    git(dir.path(), &["add", "-A"]);
    git(dir.path(), &["commit", "-q", "-m", "Keep the addresses"]);

    let out = retire(dir.path(), &["--force"]);
    assert_eq!(code_of(&out), 0, "{}", stderr(&out));
    let text = stdout(&out);
    assert!(
        text.contains("Recorded 2 addresses in ascribe.addresses, each to its page in the current line, so the build redirects them."),
        "{text}"
    );
    assert!(
        text.contains("The one that goes to the archive is your host's to redirect."),
        "{text}"
    );
    let file = read(dir.path(), "ascribe.addresses");
    assert!(
        file.contains("[moved]\n\"/1.0/both/\" = \"both.md\"\n\"/1.0/prod/\" = \"prod.md\"\n"),
        "{file}"
    );
    assert!(!file.contains("old"), "{file}");
    git(dir.path(), &["commit", "-q", "-am", "Retire 1.0"]);

    // The line's build is gone on purpose: nothing is lost.
    let check = ascribe(dir.path(), &["check", "--base", "HEAD~1"]);
    let report = format!("{}{}", stdout(&check), stderr(&check));
    assert!(!report.contains("ASC258"), "{report}");
    assert!(!report.contains("ASC255"), "{report}");
    let list = ascribe(dir.path(), &["addresses", "list"]);
    let listed = stdout(&list);
    assert!(listed.contains("/1.0/both/ -> /both/ (301"), "{listed}");
    assert!(listed.contains("/1.0/prod/ -> /prod/ (301"), "{listed}");
}

#[test]
fn retire_needs_no_force_when_another_build_changes_only_the_targets_part_of_its_availability_lines()
 {
    let dir = retiring_repository();
    // `both.md` is `cloud, prod 1.0`: its line in the cloud build loses
    // prod's `1.0+`, and nothing else.
    let model = format!("{RETIRING}\n[builds.cloud]\navailability = {{ filter = \"cloud\" }}\n");
    write(dir.path(), "ascribe.toml", &model);
    git(dir.path(), &["commit", "-q", "-am", "A cloud build"]);
    let out = retire(dir.path(), &["--dry-run"]);
    assert_eq!(code_of(&out), 0, "{}", stderr(&out));
    let text = stdout(&out);
    assert!(
        text.contains("Expected: builds that aren't lines of prod change only prod's part of their availability lines:\n  cloud's plain output changes both.md\n  cloud's site output changes both.md\n"),
        "{text}"
    );
    assert!(!text.contains("--force"), "{text}");
    let out = retire(dir.path(), &["--format", "json"]);
    assert_eq!(code_of(&out), 0, "{}", stderr(&out));
    let json: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(json["outputs"], serde_json::json!([]));
    assert_eq!(json["expected"][0]["build"], "cloud");
    assert_eq!(
        json["expected"][0]["changed"],
        serde_json::json!(["both.md"])
    );
    assert!(read(dir.path(), "docs/both.md").contains("available: cloud, prod\n"));
}

#[test]
fn retire_still_needs_force_when_another_build_changes_more_than_the_targets_part() {
    let dir = retiring_repository();
    // A filter build for 1.0 also gains the block `prod deprecated 1.1`
    // no longer limits to 1.1.
    let model = format!("{RETIRING}\n[builds.old]\navailability = {{ filter = \"prod 1.0\" }}\n");
    write(dir.path(), "ascribe.toml", &model);
    git(dir.path(), &["commit", "-q", "-am", "A build for 1.0"]);
    let out = retire(dir.path(), &["--dry-run"]);
    assert_eq!(code_of(&out), 0, "{}", stderr(&out));
    let text = stdout(&out);
    assert!(
        text.contains("Builds that aren't lines of prod change what they show:\n  old's plain output changes both.md\n  old's site output changes both.md\nRewriting them needs --force.\n"),
        "{text}"
    );
    let out = retire(dir.path(), &[]);
    assert_eq!(code_of(&out), 2, "{}", stdout(&out));
    assert!(stderr(&out).contains("pass --force"), "{}", stderr(&out));
    assert!(read(dir.path(), "docs/both.md").contains("available: cloud, prod 1.0\n"));
}

#[test]
fn without_the_addresses_file_retire_records_nothing() {
    let dir = retiring_repository();
    let out = retire(dir.path(), &["--format", "json"]);
    assert_eq!(code_of(&out), 0, "{}", stderr(&out));
    let json: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(json["recorded"], false);
    assert_eq!(json["redirects"].as_array().map(Vec::len), Some(2));
    assert!(!dir.path().join("ascribe.addresses").exists());
}

#[test]
fn without_ends_the_line_ended_today() {
    let dir = retiring_repository();
    let model = RETIRING.replace("ends = { \"1.0\" = 2026-06-30 }\n", "");
    write(dir.path(), "ascribe.toml", &model);
    git(dir.path(), &["commit", "-q", "-am", "No end"]);
    let out = retire(dir.path(), &[]);
    assert_eq!(code_of(&out), 0, "{}", stderr(&out));
    assert!(read(dir.path(), "ascribe.toml").contains("ended = 2026-10-16"));
}

#[test]
fn retire_lists_and_removes_what_no_line_has_and_the_marks_that_say_nothing() {
    let dir = retiring_repository();
    let out = retire(dir.path(), &["--format", "json"]);
    assert_eq!(code_of(&out), 0, "{}", stderr(&out));
    let json: Value = serde_json::from_slice(&out.stdout).unwrap();
    let unused: Vec<(String, String)> = json["unused"]
        .as_array()
        .unwrap()
        .iter()
        .map(|u| {
            (
                u["kind"].as_str().unwrap().to_owned(),
                u["text"].as_str().unwrap().to_owned(),
            )
        })
        .collect();
    assert_eq!(
        unused,
        [
            ("mark".to_owned(), "@available: prod".to_owned()),
            ("block".to_owned(), "Gone in 1.1.".to_owned()),
            ("section".to_owned(), "Old settings".to_owned()),
            (
                "row".to_owned(),
                "| `a` {available=\"prod (ga 0.9, removed 1.1)\"} | A. |".to_owned()
            ),
        ]
    );
    git(dir.path(), &["commit", "-q", "-am", "Retire 1.0"]);

    let shown = ascribe(
        dir.path(),
        &["versions", "retire", "prod 1.0", "--show-unused"],
    );
    assert_eq!(code_of(&shown), 0, "{}", stderr(&shown));
    assert!(
        stdout(&shown).contains("docs/prod.md:15: the section \"Old settings\""),
        "{}",
        stdout(&shown)
    );
    assert_eq!(read(dir.path(), "docs/prod.md").len(), json_len(dir.path()));

    let removed = ascribe(
        dir.path(),
        &["versions", "retire", "prod 1.0", "--remove-unused"],
    );
    assert_eq!(code_of(&removed), 0, "{}", stderr(&removed));
    assert_eq!(
        read(dir.path(), "docs/prod.md"),
        "---\ntitle: Prod\navailable: prod\n---\n\nIntro.\n\nSince 1.0, which says nothing now.\n\n## Options\n\n| Option | Does |\n|---|---|\n| `b` | B. |\n"
    );
    let check = ascribe(dir.path(), &["check"]);
    assert_eq!(code_of(&check), 0, "{}", stdout(&check));
}

#[test]
fn the_commands_a_dry_run_names_work() {
    let dir = retiring_repository();
    let dry = retire(dir.path(), &["--dry-run"]);
    assert_eq!(code_of(&dry), 0, "{}", stderr(&dry));
    let text = stdout(&dry);
    let review: Vec<&str> = text
        .lines()
        .filter_map(|l| l.split_once("Review each: ").map(|(_, c)| c))
        .collect();
    assert_eq!(review.len(), 2, "{text}");
    for command in review {
        let listed = run_named(dir.path(), command);
        assert_eq!(code_of(&listed), 0, "{command}: {}", stderr(&listed));
        let delete = stdout(&listed);
        let delete = delete
            .lines()
            .find_map(|l| l.split_once(" with: ").map(|(_, c)| c.to_owned()))
            .unwrap_or_else(|| panic!("{command}: {delete}"));
        assert_eq!(
            delete,
            "ascribe versions retire \"prod 1.0\" --url https://example.com/1.0/ --remove-unused"
        );
    }
    assert_eq!(read(dir.path(), "ascribe.toml"), RETIRING);
}

/// Runs a command the text names, `ascribe` and a quoted line among its
/// words, in `dir`.
fn run_named(dir: &Path, command: &str) -> Output {
    let (before, rest) = command.split_once('"').unwrap();
    let (line, after) = rest.split_once('"').unwrap();
    let mut args: Vec<&str> = before.split_whitespace().skip(1).collect();
    args.push(line);
    args.extend(after.split_whitespace());
    ascribe(dir, &args)
}

/// The length of `docs/prod.md` as committed, to show `--show-unused`
/// wrote nothing.
fn json_len(dir: &Path) -> usize {
    let out = Command::new("git")
        .current_dir(dir)
        .args(["show", "HEAD:docs/prod.md"])
        .output()
        .expect("run git");
    out.stdout.len()
}

/// Every file under `dir`, by its path from it, with its bytes.
fn tree(dir: &Path) -> Vec<(String, Vec<u8>)> {
    fn walk(root: &Path, dir: &Path, out: &mut Vec<(String, Vec<u8>)>) {
        for entry in fs::read_dir(dir).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                walk(root, &path, out);
            } else {
                let rel = path
                    .strip_prefix(root)
                    .unwrap()
                    .to_string_lossy()
                    .replace('\\', "/");
                out.push((rel, fs::read(&path).unwrap()));
            }
        }
    }
    let mut out = Vec::new();
    walk(dir, dir, &mut out);
    out.sort();
    out
}

fn copy(from: &Path, to: &Path) {
    for (rel, bytes) in tree(from) {
        if rel.starts_with(".ascribe/") {
            continue;
        }
        let path: PathBuf = rel.split('/').fold(to.to_path_buf(), |p, s| p.join(s));
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, bytes).unwrap();
    }
}

/// Retiring 3.4 in a copy of `examples/quill` leaves 3.5's output byte for
/// byte what moving the line in the list alone gives, and lists a redirect
/// for every page 3.4 published. The JSON output records each mark as
/// written, so it's the site and plain outputs that are compared, as
/// `release` compares them.
#[test]
fn retiring_quills_older_line_changes_no_remaining_lines_output() {
    let quill = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/quill");
    let listed = tempfile::tempdir().unwrap();
    copy(&quill, listed.path());
    let built = ascribe(listed.path(), &["build"]);
    assert_eq!(code_of(&built), 0, "{}", stderr(&built));
    let versions: Value = serde_json::from_str(&read(
        listed.path(),
        ".ascribe/build/_ascribe/versions.json",
    ))
    .unwrap();
    let mut published: Vec<String> = versions["builds"]["site"]["pages"]
        .as_object()
        .unwrap()
        .values()
        .filter_map(|p| p["lines"]["3.4"].as_str().map(str::to_owned))
        .collect();
    published.sort();

    // The list alone.
    let model = read(listed.path(), "ascribe.toml").replace(
        "live = [\"3.4\", \"3.5\"]",
        "live = [\"3.5\"]\n\n[releases.self-managed.archived]\n\"3.4\" = { url = \"https://example.com/3.4/\", ended = 2026-10-16 }",
    );
    write(listed.path(), "ascribe.toml", &model);
    let built = ascribe(listed.path(), &["build"]);
    assert_eq!(code_of(&built), 0, "{}", stderr(&built));
    // The retired line's output is gone.
    assert!(!listed.path().join(".ascribe/build/site@3.4").exists());

    // The command.
    let retired = tempfile::tempdir().unwrap();
    copy(&quill, retired.path());
    let out = Command::new(env!("CARGO_BIN_EXE_ascribe"))
        .current_dir(retired.path())
        .args([
            "versions",
            "retire",
            "self-managed 3.4",
            "--url",
            "https://example.com/3.4/",
            "--force",
            "--format",
            "json",
        ])
        .env("ASCRIBE_TODAY", "2026-10-16")
        .output()
        .unwrap();
    assert_eq!(code_of(&out), 0, "{}", stderr(&out));
    assert_eq!(read(retired.path(), "ascribe.toml"), model);
    let json: Value = serde_json::from_slice(&out.stdout).unwrap();
    let mut from: Vec<String> = json["redirects"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| r["from"].as_str().unwrap().to_owned())
        .collect();
    from.sort();
    assert_eq!(from, published);
    assert!(
        json["redirects"]
            .as_array()
            .unwrap()
            .iter()
            .any(|r| r["from"] == "/3.4/configure-legacy/" && r["to"] == "/configure/")
    );
    let built = ascribe(retired.path(), &["build"]);
    assert_eq!(code_of(&built), 0, "{}", stderr(&built));
    for folder in ["site/site", "site/plain", "_ascribe"] {
        let rel = format!(".ascribe/build/{folder}");
        assert!(
            tree(&listed.path().join(&rel)) == tree(&retired.path().join(&rel)),
            "{folder} differs"
        );
    }
}
