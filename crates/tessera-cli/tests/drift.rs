//! `ascribe drift`: the pages whose examples changed, by running the binary
//! in a temporary repository.

#![allow(clippy::expect_used, clippy::unwrap_used)]

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use tempfile::TempDir;

const MODEL: &str = r#"spec = "0.1"

[project]
content-root = "docs"

[consumer]
site = "https://docs.example.com"

[sources.code]
path = ".."
include = ["service/**"]

[builds.site]
"#;

fn code(main: &str) -> String {
    format!("import os\n\n# :snippet-start: main\n{main}# :snippet-end:\nrun()\n")
}

const RUN: &str = "# Run\n\nConnect first:\n\n@snippet: code:service/app.py#main\n";
const DEPLOY: &str = "# Deploy\n\n@snippet: code:service/app.py#main\n\nThen deploy.\n";

/// A repository whose project is in `site/`, its code in `service/`,
/// committed on `main`, with a `feature` branch checked out.
fn repo() -> TempDir {
    let dir = tempfile::tempdir().expect("a temporary directory");
    git(dir.path(), &["init", "-q", "-b", "main"]);
    write(dir.path(), "site/ascribe.toml", MODEL);
    write(dir.path(), "site/docs/run.md", RUN);
    write(dir.path(), "site/docs/deploy.md", DEPLOY);
    write(dir.path(), "site/docs/about.md", "# About\n");
    write(dir.path(), "service/app.py", &code("connect()\n"));
    commit(dir.path(), "first");
    git(dir.path(), &["checkout", "-q", "-b", "feature"]);
    dir
}

/// The code changes, and so does `deploy.md`'s own text.
fn change(dir: &TempDir) {
    write(
        dir.path(),
        "service/app.py",
        &code("client = connect(retries=3)\nclient.open()\n"),
    );
    write(
        dir.path(),
        "site/docs/deploy.md",
        &DEPLOY.replace("Then deploy.", "Then deploy it."),
    );
    commit(dir.path(), "change");
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

fn commit(dir: &Path, message: &str) {
    git(dir, &["add", "-A"]);
    git(dir, &["commit", "-q", "-m", message]);
}

fn write(root: &Path, rel: &str, text: &str) {
    let path: PathBuf = rel.split('/').fold(root.to_path_buf(), |p, s| p.join(s));
    fs::create_dir_all(path.parent().expect("a parent")).expect("create directories");
    fs::write(path, text).expect("write a file");
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

fn site(dir: &TempDir) -> PathBuf {
    dir.path().join("site")
}

#[test]
fn both_groups_in_text() {
    let dir = repo();
    change(&dir);
    let out = ascribe(&site(&dir), &["drift"]);
    assert_eq!(code_of(&out), 0, "{}", stderr(&out));
    let text = stdout(&out);
    assert!(text.starts_with("compared with main ("), "{text}");
    assert!(
        text.ends_with(
            "\n\nExamples that changed. The page shows the new code; check the words around it:\n  run.md\n    code:service/app.py#main (+2 \u{2212}1)\n\nExamples that changed along with the page:\n  deploy.md\n    code:service/app.py#main (+2 \u{2212}1)\n"
        ),
        "{text}"
    );
    // The first group exits 1 with `--exit-code`.
    let out = ascribe(&site(&dir), &["drift", "--exit-code"]);
    assert_eq!(code_of(&out), 1, "{}", stderr(&out));
}

#[test]
fn nothing_changed() {
    let dir = repo();
    // An edit outside the region isn't a change to the example.
    write(
        dir.path(),
        "service/app.py",
        &format!("{}print('done')\n", code("connect()\n")),
    );
    let out = ascribe(&site(&dir), &["drift", "--exit-code"]);
    assert_eq!(code_of(&out), 0, "{}", stderr(&out));
    assert!(
        stdout(&out).ends_with(")\n\nNo examples changed.\n"),
        "{}",
        stdout(&out)
    );
    let out = ascribe(&site(&dir), &["drift", "--format", "summary"]);
    assert_eq!(code_of(&out), 0, "{}", stderr(&out));
    assert_eq!(stdout(&out), "");
}

#[test]
fn only_the_page_changed_with_its_example_exits_0() {
    let dir = repo();
    change(&dir);
    write(
        dir.path(),
        "site/docs/run.md",
        &RUN.replace("first", "first of all"),
    );
    commit(dir.path(), "and the run page");
    let out = ascribe(&site(&dir), &["drift", "--exit-code"]);
    assert_eq!(code_of(&out), 0, "{}", stderr(&out));
    assert!(
        !stdout(&out).contains("check the words"),
        "{}",
        stdout(&out)
    );
}

#[test]
fn json_output() {
    let dir = repo();
    change(&dir);
    let out = ascribe(&site(&dir), &["drift", "--format", "json"]);
    assert_eq!(code_of(&out), 0, "{}", stderr(&out));
    let json: serde_json::Value = serde_json::from_str(&stdout(&out)).expect("JSON");
    assert_eq!(json["schema_version"], 1);
    assert_eq!(json["base"]["requested"], "main");
    assert_eq!(json["repository"]["project_prefix"], "site/");
    let pages = json["pages"].as_array().expect("pages");
    assert_eq!(pages.len(), 2);
    assert_eq!(pages[0]["path"], "deploy.md");
    assert_eq!(pages[0]["page_changed"], true);
    assert_eq!(pages[1]["path"], "run.md");
    assert_eq!(pages[1]["route"], "/run/");
    assert_eq!(pages[1]["page_changed"], false);
    assert_eq!(pages[1]["builds"], serde_json::json!(["site"]));
    assert_eq!(
        pages[1]["examples"],
        serde_json::json!([{
            "address": "code:service/app.py#main",
            "source": "code",
            "file": "service/app.py",
            "was_file": null,
            "added": 2,
            "removed": 1,
        }])
    );
}

#[test]
fn summary_output() {
    let dir = repo();
    change(&dir);
    let out = ascribe(&site(&dir), &["drift", "--format", "summary"]);
    assert_eq!(code_of(&out), 0, "{}", stderr(&out));
    let text = stdout(&out);
    assert!(
        text.starts_with("### Examples that changed\n\nCompared with `main` (`"),
        "{text}"
    );
    assert!(
        text.ends_with(
            "\n\nThe page shows the new code; check the words around it:\n\n- [run.md](https://docs.example.com/run/)\n  - `code:service/app.py#main` (+2 \u{2212}1)\n\nChanged along with the page:\n\n- [deploy.md](https://docs.example.com/deploy/)\n  - `code:service/app.py#main` (+2 \u{2212}1)\n"
        ),
        "{text}"
    );
}

#[test]
fn a_shallow_clone_exits_2_and_says_to_fetch_more() {
    let upstream = repo();
    change(&upstream);
    git(upstream.path(), &["checkout", "-q", "main"]);
    write(upstream.path(), "site/docs/later.md", "# Later\n");
    commit(upstream.path(), "on main");

    // What actions/checkout does by default: one commit of the branch, then
    // one of the base.
    let clone = tempfile::tempdir().unwrap();
    let path = upstream.path().display().to_string().replace('\\', "/");
    let url = if path.starts_with('/') {
        format!("file://{path}")
    } else {
        format!("file:///{path}")
    };
    git(
        clone.path(),
        &[
            "clone", "-q", "--depth", "1", "--branch", "feature", &url, ".",
        ],
    );
    git(
        clone.path(),
        &[
            "fetch",
            "-q",
            "--depth",
            "1",
            "origin",
            "main:refs/remotes/origin/main",
        ],
    );
    let out = ascribe(&clone.path().join("site"), &["drift"]);
    assert_eq!(code_of(&out), 2, "{}", stdout(&out));
    let message = stderr(&out);
    assert!(message.contains("fetch-depth: 0"), "{message}");
    assert!(!message.contains("--base-exact"), "{message}");
    // `check` doesn't read history: the shallow clone gets what the full
    // repository gets.
    git(upstream.path(), &["checkout", "-q", "feature"]);
    let shallow = ascribe(&clone.path().join("site"), &["check"]);
    let full = ascribe(&site(&upstream), &["check"]);
    assert_eq!(code_of(&shallow), code_of(&full));
    assert_eq!(stdout(&shallow), stdout(&full));
}

#[test]
fn not_a_repository_exits_2() {
    let dir = tempfile::tempdir().unwrap();
    write(dir.path(), "ascribe.toml", MODEL);
    write(dir.path(), "docs/index.md", "# Home\n");
    let out = ascribe(dir.path(), &["drift"]);
    assert_eq!(code_of(&out), 2);
    assert!(stderr(&out).starts_with("error: "), "{}", stderr(&out));
}

#[test]
fn an_unknown_build_exits_2() {
    let dir = repo();
    let out = ascribe(&site(&dir), &["drift", "--build", "nope"]);
    assert_eq!(code_of(&out), 2);
    assert!(stderr(&out).contains("no build `nope`"), "{}", stderr(&out));
}

#[test]
fn a_relative_config_whose_source_is_its_parent() {
    let dir = repo();
    change(&dir);
    // As the CI recipe runs it: from the repository's root, with the
    // project's folder as `--config`, and a source at `..`.
    let out = ascribe(dir.path(), &["drift", "--config", "site"]);
    assert_eq!(code_of(&out), 0, "{}", stderr(&out));
    assert!(stdout(&out).contains("  run.md\n"), "{}", stdout(&out));
    let out = ascribe(
        dir.path(),
        &["check", "--config", "site", "--format", "json"],
    );
    assert!(
        !stdout(&out).contains("model-source-outside-repository"),
        "{}",
        stdout(&out)
    );
}
