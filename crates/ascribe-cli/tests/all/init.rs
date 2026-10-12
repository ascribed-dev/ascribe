//! `ascribe init`, by running the binary in temporary folders: what it says
//! it wrote, its JSON, the project it starts checking clean with a page of
//! the default type, and what it exits with when it refuses. What it writes,
//! and each case it refuses, is held by
//! `crates/ascribe-templates/tests/all/init.rs`.

#![allow(clippy::expect_used, clippy::unwrap_used, clippy::panic)]

use std::fs;
use std::path::Path;
use std::process::{Command, Output};

use serde_json::Value;

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
fn init_says_what_it_wrote_and_where_to_go_next() {
    let dir = tempfile::tempdir().unwrap();
    let output = ascribe(dir.path(), &["init"]);
    assert_eq!(code_of(&output), 0, "{}", stderr(&output));
    assert_eq!(
        stdout(&output),
        "wrote ascribe.toml\n\
         made docs/, the folder the pages go in\n\
         Next, to publish the pages as a site with Astro, read https://ascribed-dev.com/go/astro-guide/\n"
    );
}

/// The phase's goal: in an empty folder, `ascribe init --templates`, then
/// `ascribe check`, which is clean; and a page is of the type it names.
#[test]
fn init_with_the_templates_checks_clean_and_a_page_is_of_the_type_it_names() {
    let dir = tempfile::tempdir().unwrap();
    let output = ascribe(dir.path(), &["init", "--templates"]);
    assert_eq!(code_of(&output), 0, "{}", stderr(&output));
    let text = stdout(&output);
    assert!(text.starts_with("wrote templates/"), "{text}");
    assert!(
        text.contains("wrote ascribe.toml\nmade docs/, the folder the pages go in\nadded [types."),
        "{text}"
    );
    assert!(
        text.ends_with("read https://ascribed-dev.com/go/astro-guide/\n"),
        "{text}"
    );
    let check = ascribe(dir.path(), &["check"]);
    assert_eq!(code_of(&check), 0, "{}{}", stdout(&check), stderr(&check));
    assert!(
        stdout(&check).contains("0 errors, 0 warnings, 0 advice"),
        "{}",
        stdout(&check)
    );

    fs::write(
        dir.path().join("docs/index.md"),
        "---\ntype: page\ntitle: Handbook\ndescription: How the team works.\n---\n\nWelcome.\n",
    )
    .unwrap();
    let check = ascribe(dir.path(), &["check", "--format", "json"]);
    assert_eq!(code_of(&check), 0, "{}", stdout(&check));
    let report: Value = serde_json::from_slice(&check.stdout).unwrap();
    assert_eq!(report["diagnostics"], Value::Array(Vec::new()));
    let outline = ascribe(
        dir.path(),
        &["outline", "docs/index.md", "--format", "json"],
    );
    assert_eq!(code_of(&outline), 0, "{}", stderr(&outline));
    let outline: Value = serde_json::from_slice(&outline.stdout).unwrap();
    assert_eq!(outline["type"], "page", "{outline:#}");
}

#[test]
fn init_s_json_lists_what_it_wrote() {
    let dir = tempfile::tempdir().unwrap();
    let output = ascribe(dir.path(), &["init", "site", "--format", "json"]);
    assert_eq!(code_of(&output), 0, "{}", stderr(&output));
    let answer: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(
        answer["schema_version"],
        ascribe_templates::init::SCHEMA_VERSION
    );
    assert_eq!(answer["written"], serde_json::json!(["ascribe.toml"]));
    assert_eq!(answer["content_root"], "docs");
    assert_eq!(answer["content_root_made"], true);
    assert_eq!(answer["templates"], Value::Null);
    assert_eq!(answer["next"], "https://ascribed-dev.com/go/astro-guide/");
    assert!(dir.path().join("site/ascribe.toml").is_file());
}

#[test]
fn init_in_a_project_names_it_and_writes_nothing() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join("ascribe.toml"), "spec = \"0.1\"\n").unwrap();
    fs::create_dir(dir.path().join("docs")).unwrap();
    let output = ascribe(&dir.path().join("docs"), &["init", "--templates"]);
    assert_eq!(code_of(&output), 2);
    assert_eq!(stdout(&output), "");
    assert_eq!(
        stderr(&output),
        "error: there's a project here already, `../ascribe.toml`; to add the templates to it, run `ascribe templates eject` there\n"
    );
    assert!(!dir.path().join("templates").exists());
    assert!(!dir.path().join("docs/ascribe.toml").exists());
}

#[test]
fn init_with_a_template_in_the_way_writes_nothing_and_exits_2() {
    let dir = tempfile::tempdir().unwrap();
    fs::create_dir(dir.path().join("templates")).unwrap();
    fs::write(dir.path().join("templates/how-to.md"), "Mine.\n").unwrap();
    let output = ascribe(dir.path(), &["init", "--templates"]);
    assert_eq!(code_of(&output), 2);
    assert_eq!(stdout(&output), "");
    let err = stderr(&output);
    assert!(
        err.starts_with("error: how-to: `templates/how-to.md` is there already"),
        "{err}"
    );
    assert!(err.ends_with("wrote nothing\n"), "{err}");
    assert!(!dir.path().join("ascribe.toml").exists());
    assert!(!dir.path().join("docs").exists());
}

#[test]
fn init_takes_no_config() {
    let dir = tempfile::tempdir().unwrap();
    let output = ascribe(dir.path(), &["init", "--config", "."]);
    assert_eq!(code_of(&output), 2);
    assert!(
        stderr(&output).contains("doesn't take --config"),
        "{}",
        stderr(&output)
    );
    assert!(fs::read_dir(dir.path()).unwrap().next().is_none());
}

/// Getting started's project is made by the command: its `ascribe.toml`,
/// without the tags that mark the page's two snippets, is what `ascribe
/// init` writes, then what the page adds at its end.
#[test]
fn getting_started_s_ascribe_toml_starts_with_what_init_writes() {
    let dir = tempfile::tempdir().unwrap();
    let output = ascribe(dir.path(), &["init"]);
    assert_eq!(code_of(&output), 0, "{}", stderr(&output));
    let written = fs::read_to_string(dir.path().join("ascribe.toml")).unwrap();
    let example =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/getting-started/ascribe.toml");
    let example = fs::read_to_string(example).unwrap();
    let untagged: String = example
        .lines()
        .filter(|line| !line.starts_with("# :snippet-"))
        .map(|line| format!("{line}\n"))
        .collect();
    assert!(
        untagged.starts_with(&written),
        "examples/getting-started/ascribe.toml no longer starts with what `ascribe init` writes:\n{written}"
    );
}
