//! `ascribe baseline write` and `prune`, and `ascribe check`, `build`, and
//! `report` reading `ascribe.baseline`, by running the binary: what each writes, its JSON,
//! its exit codes, and a baseline that can't be read. What the baseline
//! holds back, and its stale entries, are conformance cases
//! (`tests/conformance/cases/checks/baseline/`); the file's syntax is
//! `ascribe_check::baseline`'s unit tests.

#![allow(clippy::expect_used, clippy::unwrap_used)]

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use tempfile::TempDir;

const MODEL: &str = "spec = \"0.1\"\n\n[project]\ncontent-root = \"docs\"\n";

/// A long `nav-label` is advice: `nav-label-long`.
fn page(title: &str, label: &str) -> String {
    format!("---\ntitle: {title}\nnav-label: {label}\n---\n\nText.\n")
}

/// A project with two pages whose `nav-label`s are advice, and a home page
/// that links to both.
fn project(model: &str) -> TempDir {
    let dir = tempfile::tempdir().expect("a temporary directory");
    write(dir.path(), "ascribe.toml", model);
    write(
        dir.path(),
        "docs/index.md",
        "---\ntitle: Home\n---\n\nSee [sync](sync.md) and [keys](<guides/the keys.md>).\n",
    );
    write(
        dir.path(),
        "docs/sync.md",
        &page("Sync", "Synchronize two copies"),
    );
    write(
        dir.path(),
        "docs/guides/the keys.md",
        &page("Keys", "All about the API keys"),
    );
    dir
}

fn write(root: &Path, rel: &str, text: &str) {
    let path: PathBuf = rel.split('/').fold(root.to_path_buf(), |p, s| p.join(s));
    fs::create_dir_all(path.parent().expect("a parent")).expect("create directories");
    fs::write(path, text).expect("write a file");
}

fn read(root: &Path, rel: &str) -> String {
    fs::read_to_string(root.join(rel)).expect("read a file")
}

fn ascribe(dir: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_ascribe"))
        .current_dir(dir)
        .args(args)
        .env("NO_COLOR", "1")
        .output()
        .expect("run ascribe")
}

fn code_of(output: &Output) -> i32 {
    output.status.code().expect("exited with a code")
}

fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

fn json(output: &Output) -> serde_json::Value {
    serde_json::from_slice(&output.stdout).expect("the command's JSON")
}

#[test]
fn write_makes_the_baseline_and_check_holds_back_what_it_lists() {
    let dir = project(MODEL);
    let out = ascribe(dir.path(), &["baseline", "write"]);
    assert_eq!(code_of(&out), 0, "{}", stderr(&out));
    assert_eq!(
        stdout(&out),
        "wrote ascribe.baseline: 2 problems in 2 files, 2 added; commit it with the docs\n"
    );
    let text = read(dir.path(), "ascribe.baseline");
    let lines: Vec<&str> = text.lines().collect();
    assert_eq!(lines[0], "# ascribe baseline 1");
    assert_eq!(lines[2], "# check fingerprint order file");
    // Sorted by file: a path with a space is written as it is.
    assert!(
        lines[3].starts_with("nav-label-long ") && lines[3].ends_with(" 0 docs/guides/the keys.md"),
        "{text}"
    );
    assert!(lines[4].ends_with(" 0 docs/sync.md"), "{text}");
    assert_eq!(lines.len(), 5, "{text}");

    let out = ascribe(dir.path(), &["check", "--format", "concise"]);
    assert_eq!(code_of(&out), 0, "{}", stderr(&out));
    assert_eq!(
        stdout(&out),
        "checked 3 files for every build, 0 with problems: 0 errors, 0 warnings, 0 advice, 2 in the baseline\n"
    );
    let report = json(&ascribe(dir.path(), &["check", "--format", "json"]));
    assert_eq!(report["diagnostics"], serde_json::json!([]));
    assert_eq!(report["summary"]["baselined"], 2);
    assert_eq!(report["summary"]["advice"], 0);
}

#[test]
fn write_run_twice_gives_the_same_file_byte_for_byte() {
    let dir = project(MODEL);
    assert_eq!(code_of(&ascribe(dir.path(), &["baseline", "write"])), 0);
    let first = fs::read(dir.path().join("ascribe.baseline")).unwrap();
    let out = ascribe(dir.path(), &["baseline", "write", "--force"]);
    assert_eq!(code_of(&out), 0, "{}", stderr(&out));
    assert_eq!(
        stdout(&out),
        "ascribe.baseline is up to date: 2 problems in 2 files\n"
    );
    assert_eq!(
        fs::read(dir.path().join("ascribe.baseline")).unwrap(),
        first
    );
}

#[test]
fn write_keeps_what_it_had_adds_only_advice_and_says_what_it_skipped() {
    let dir = project(MODEL);
    assert_eq!(code_of(&ascribe(dir.path(), &["baseline", "write"])), 0);
    // One problem is now a warning, and a third page brings new advice.
    write(
        dir.path(),
        "ascribe.toml",
        &format!("{MODEL}\n[checks]\npage-orphan = \"warning\"\n"),
    );
    write(
        dir.path(),
        "docs/new.md",
        &page("New", "A label that's long"),
    );
    let out = ascribe(
        dir.path(),
        &["baseline", "write", "--force", "--format", "json"],
    );
    assert_eq!(code_of(&out), 0, "{}", stderr(&out));
    let written = json(&out);
    assert_eq!(written["schema_version"], 1);
    assert_eq!(written["file"], "ascribe.baseline");
    assert_eq!(written["created"], false);
    assert_eq!(written["changed"], true);
    assert_eq!(written["entries"], 3);
    assert_eq!(written["files"], 3);
    assert_eq!(written["added"], 1);
    assert_eq!(
        written["skipped"],
        serde_json::json!({"errors": 0, "warnings": 1})
    );
    let text = read(dir.path(), "ascribe.baseline");
    assert!(!text.contains("page-orphan"), "{text}");

    let out = ascribe(dir.path(), &["baseline", "write", "--force"]);
    assert!(
        stdout(&out).contains(
            "skipped 1 warning, since a baseline holds advice only; to put one off, lower its check to \"advice\" in [checks] in ascribe.toml, then run `ascribe baseline write` again"
        ),
        "{}",
        stdout(&out)
    );
    // The warning isn't held back.
    let report = json(&ascribe(dir.path(), &["check", "--format", "json"]));
    assert_eq!(report["summary"]["warnings"], 1);
    assert_eq!(report["summary"]["baselined"], 3);
}

#[test]
fn prune_removes_the_entries_that_match_nothing() {
    let dir = project(MODEL);
    assert_eq!(code_of(&ascribe(dir.path(), &["baseline", "write"])), 0);
    write(dir.path(), "docs/sync.md", &page("Sync", "Sync"));

    // Until it's pruned, the stale entry is reported, as advice.
    let out = ascribe(dir.path(), &["check", "--format", "concise"]);
    assert_eq!(code_of(&out), 0, "{}", stderr(&out));
    assert!(
        stdout(&out).starts_with("ascribe.baseline:5: [ASC243] ascribe.baseline holds back `nav-label-long` in `docs/sync.md`, and nothing there matches it any more; run `ascribe baseline prune`\n"),
        "{}",
        stdout(&out)
    );

    let out = ascribe(dir.path(), &["baseline", "prune", "--force"]);
    assert_eq!(code_of(&out), 0, "{}", stderr(&out));
    assert_eq!(
        stdout(&out),
        "removed 1 entry that matched nothing from ascribe.baseline: 1 problem in 1 file left; commit it with the docs\n"
    );
    assert!(!read(dir.path(), "ascribe.baseline").contains("docs/sync.md"));

    let out = ascribe(
        dir.path(),
        &["baseline", "prune", "--force", "--format", "json"],
    );
    assert_eq!(code_of(&out), 0, "{}", stderr(&out));
    let pruned = json(&out);
    assert_eq!(pruned["schema_version"], 1);
    assert_eq!(pruned["found"], true);
    assert_eq!(pruned["changed"], false);
    assert_eq!(pruned["entries"], 1);
    assert_eq!(pruned["removed"], 0);

    let out = ascribe(dir.path(), &["check", "--format", "concise"]);
    assert_eq!(
        stdout(&out),
        "checked 3 files for every build, 0 with problems: 0 errors, 0 warnings, 0 advice, 1 in the baseline\n"
    );
}

#[test]
fn prune_without_a_baseline_writes_nothing() {
    let dir = project(MODEL);
    let out = ascribe(
        dir.path(),
        &["baseline", "prune", "--force", "--format", "json"],
    );
    assert_eq!(code_of(&out), 0, "{}", stderr(&out));
    assert_eq!(json(&out)["found"], false);
    assert!(!dir.path().join("ascribe.baseline").exists());
}

#[test]
fn no_baseline_reports_everything() {
    let dir = project(MODEL);
    assert_eq!(code_of(&ascribe(dir.path(), &["baseline", "write"])), 0);
    let out = ascribe(dir.path(), &["check", "--no-baseline", "--format", "json"]);
    assert_eq!(code_of(&out), 0, "{}", stderr(&out));
    let report = json(&out);
    assert_eq!(report["summary"]["advice"], 2);
    assert!(report["summary"].get("baselined").is_none(), "{report}");
}

#[test]
fn a_baseline_that_can_t_be_read_stops_every_command_but_no_baseline() {
    let dir = project(MODEL);
    write(
        dir.path(),
        "ascribe.baseline",
        "# ascribe baseline 1\n\nnav-label-long 0123456789abcdef docs/sync.md\n",
    );
    let line =
        "ascribe.baseline:3: an entry is `check fingerprint order file`, separated by one space";

    let out = ascribe(dir.path(), &["check"]);
    assert_eq!(code_of(&out), 2);
    assert!(
        stderr(&out).starts_with(&format!("error: {line}")),
        "{}",
        stderr(&out)
    );
    assert!(
        stderr(&out).contains("pass --no-baseline"),
        "{}",
        stderr(&out)
    );

    let out = ascribe(dir.path(), &["check", "--format", "json"]);
    assert_eq!(code_of(&out), 2);
    let error = json(&out)["error"].as_str().unwrap_or_default().to_owned();
    assert!(error.starts_with(line), "{error}");

    for verb in ["write", "prune"] {
        let out = ascribe(dir.path(), &["baseline", verb]);
        assert_eq!(code_of(&out), 2, "{verb}");
        assert!(
            stderr(&out).starts_with(&format!("error: {line}")),
            "{verb}: {}",
            stderr(&out)
        );
    }

    let out = ascribe(
        dir.path(),
        &["check", "--no-baseline", "--format", "concise"],
    );
    assert_eq!(code_of(&out), 0, "{}", stderr(&out));
}

#[test]
fn a_check_of_some_files_or_builds_reports_no_stale_entry() {
    let dir = project(MODEL);
    assert_eq!(code_of(&ascribe(dir.path(), &["baseline", "write"])), 0);
    write(dir.path(), "docs/sync.md", &page("Sync", "Sync"));
    let out = ascribe(
        dir.path(),
        &["check", "docs/sync.md", "--format", "concise"],
    );
    assert_eq!(code_of(&out), 0, "{}", stderr(&out));
    assert_eq!(
        stdout(&out),
        "checked 3 files for every build, reported on 1 file in the paths, 0 with problems: 0 errors, 0 warnings, 0 advice\n"
    );
    let out = ascribe(
        dir.path(),
        &["check", "--editor-build", "--format", "concise"],
    );
    assert!(!stdout(&out).contains("ASC243"), "{}", stdout(&out));
}

#[test]
fn build_holds_back_what_the_baseline_lists_as_check_does() {
    let dir = project(MODEL);
    assert_eq!(code_of(&ascribe(dir.path(), &["baseline", "write"])), 0);
    let out = ascribe(dir.path(), &["build", "--emit", "json"]);
    assert_eq!(code_of(&out), 0, "{}", stderr(&out));
    assert_eq!(
        stdout(&out),
        "checked 3 files in every build: no problems, 2 in the baseline\n"
    );
    let report = json(&ascribe(
        dir.path(),
        &["build", "--emit", "json", "--format", "json"],
    ));
    let check = json(&ascribe(dir.path(), &["check", "--format", "json"]));
    assert_eq!(report["diagnostics"], check["diagnostics"]);
    assert_eq!(report["summary"], check["summary"]);
    assert_eq!(report["summary"]["baselined"], 2);
}

#[test]
fn build_no_baseline_reports_everything() {
    let dir = project(MODEL);
    assert_eq!(code_of(&ascribe(dir.path(), &["baseline", "write"])), 0);
    let out = ascribe(dir.path(), &["build", "--emit", "json", "--no-baseline"]);
    assert_eq!(code_of(&out), 0, "{}", stderr(&out));
    assert_eq!(
        stdout(&out),
        "checked 3 files in every build: 0 errors, 0 warnings, 2 advice: ascribe check --no-baseline for each\n"
    );
    let report = json(&ascribe(
        dir.path(),
        &[
            "build",
            "--emit",
            "json",
            "--format",
            "json",
            "--no-baseline",
        ],
    ));
    assert_eq!(report["summary"]["advice"], 2);
    assert_eq!(report["summary"].get("baselined"), None);
}

#[test]
fn build_without_a_baseline_reports_everything() {
    let dir = project(MODEL);
    let out = ascribe(dir.path(), &["build", "--emit", "json"]);
    assert_eq!(code_of(&out), 0, "{}", stderr(&out));
    assert_eq!(
        stdout(&out),
        "checked 3 files in every build: 0 errors, 0 warnings, 2 advice: ascribe check for each\n"
    );
}

#[test]
fn report_holds_back_what_the_baseline_lists_and_counts_it() {
    let dir = project(MODEL);
    let before = json(&ascribe(
        dir.path(),
        &["report", "problems", "--format", "json", "--no-history"],
    ));
    assert_eq!(before["problems"]["summary"]["advice"], 2);
    assert_eq!(before["problems"].get("baselined"), None);

    assert_eq!(code_of(&ascribe(dir.path(), &["baseline", "write"])), 0);
    let out = ascribe(dir.path(), &["report", "problems", "--no-history"]);
    assert_eq!(code_of(&out), 0, "{}", stderr(&out));
    assert!(
        stdout(&out).contains("0 errors, 0 warnings, 0 advice, and 2 in the baseline."),
        "{}",
        stdout(&out)
    );
    let after = json(&ascribe(
        dir.path(),
        &["report", "problems", "--format", "json", "--no-history"],
    ));
    assert_eq!(after["problems"]["summary"]["advice"], 0);
    assert_eq!(after["problems"]["baselined"], 2);
    // `--exit-code` doesn't fail on what the baseline holds back.
    let out = ascribe(
        dir.path(),
        &["report", "problems", "--no-history", "--exit-code"],
    );
    assert_eq!(code_of(&out), 0, "{}", stderr(&out));
}

#[test]
fn report_no_baseline_reports_everything() {
    let dir = project(MODEL);
    assert_eq!(code_of(&ascribe(dir.path(), &["baseline", "write"])), 0);
    let report = json(&ascribe(
        dir.path(),
        &[
            "report",
            "problems",
            "--format",
            "json",
            "--no-history",
            "--no-baseline",
        ],
    ));
    assert_eq!(report["problems"]["summary"]["advice"], 2);
    assert_eq!(report["problems"].get("baselined"), None);
    assert_eq!(
        report["problems"]["list_command"],
        "ascribe check --no-baseline --format json"
    );
}

#[test]
fn a_baseline_that_cant_be_read_stops_build_and_report() {
    let dir = project(MODEL);
    write(
        dir.path(),
        "ascribe.baseline",
        "# ascribe baseline 1\nnot an entry\n",
    );
    for args in [
        &["build", "--emit", "json"][..],
        &["report", "problems", "--no-history"][..],
    ] {
        let out = ascribe(dir.path(), args);
        assert_eq!(code_of(&out), 2, "{args:?}: {}", stdout(&out));
        assert!(
            stderr(&out).starts_with("error: ascribe.baseline:2: ")
                && stderr(&out).contains("pass --no-baseline"),
            "{args:?}: {}",
            stderr(&out)
        );
        let mut without = args.to_vec();
        without.push("--no-baseline");
        let out = ascribe(dir.path(), &without);
        assert_eq!(code_of(&out), 0, "{without:?}: {}", stderr(&out));
    }
}
