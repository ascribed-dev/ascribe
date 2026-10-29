//! `ascribe new`, by running the binary in a temporary project with the
//! how-to template: the page it writes and what it says, from another
//! folder, with no title, with `--dry-run` and `--format json`, and what it
//! refuses. Where a page goes and its text are `ascribe_templates::start`'s,
//! held by `crates/ascribe-templates/tests/all/start.rs`.

#![allow(clippy::expect_used, clippy::unwrap_used, clippy::panic)]

use std::fs;
use std::path::Path;
use std::process::{Command, Output};

use serde_json::Value;
use tempfile::TempDir;

/// A project started with `ascribe init --templates`.
fn project() -> TempDir {
    let dir = tempfile::tempdir().unwrap();
    let out = ascribe(dir.path(), &["init", "--templates"]);
    assert_eq!(code_of(&out), 0, "{}", stderr(&out));
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

fn template(dir: &Path) -> String {
    fs::read_to_string(dir.join("templates/how-to.md")).unwrap()
}

/// The how-to's template as a page starts from it: titled `title`, or with
/// the template's own, and a draft.
fn started(dir: &Path, title: Option<&str>) -> String {
    let text = template(dir);
    let text = match title {
        Some(title) => text.replacen("title: \"Title\"", &format!("title: \"{title}\""), 1),
        None => text,
    };
    text.replacen("\n---\n", "\ndraft: true\n---\n", 1)
}

#[test]
fn a_page_in_a_folder_is_named_for_its_title_and_written_from_the_template() {
    let dir = project();
    let out = ascribe(
        dir.path(),
        &["new", "how-to", "docs/guides", "--title", "Rotate keys"],
    );
    assert_eq!(code_of(&out), 0, "{}", stderr(&out));
    let page = fs::read_to_string(dir.path().join("docs/guides/rotate-keys.md")).unwrap();
    assert_eq!(page, started(dir.path(), Some("Rotate keys")));
    let text = stdout(&out);
    assert!(
        text.starts_with(
            "wrote docs/guides/rotate-keys.md, a page of type `how-to`, from templates/how-to.md\n\
             Read its guide before you write: templates/how-to.guide.md, templates/how-to.ascribe.md\n\
             It has "
        ),
        "{text}"
    );
    assert!(
        text.ends_with(
            " placeholders left to fill in, which `ascribe check docs/guides/rotate-keys.md` lists\n\
             It's a draft, with `draft: true`, which no build publishes: take the line out to publish it\n"
        ),
        "{text}"
    );
    // What it counts is what the check reports: advice, in a draft.
    let check = ascribe(
        dir.path(),
        &["check", "docs/guides/rotate-keys.md", "--format", "json"],
    );
    let found: Value = serde_json::from_str(&stdout(&check)).unwrap();
    let left = found["diagnostics"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|d| d["code"] == "ASC195")
        .count();
    assert!(
        text.contains(&format!("It has {left} placeholders")),
        "{text}"
    );
}

#[test]
fn it_never_writes_over_a_file() {
    let dir = project();
    let page = dir.path().join("docs/rotate.md");
    fs::write(&page, "---\ntitle: Mine\n---\n").unwrap();
    let out = ascribe(dir.path(), &["new", "how-to", "docs/rotate.md"]);
    assert_eq!(code_of(&out), 2);
    assert_eq!(
        stderr(&out),
        "error: `docs/rotate.md` is there already, and `new` never writes over a file\n"
    );
    assert_eq!(
        fs::read_to_string(&page).unwrap(),
        "---\ntitle: Mine\n---\n"
    );
}

#[test]
fn without_a_title_the_page_keeps_the_templates_and_needs_its_own_path() {
    let dir = project();
    let out = ascribe(dir.path(), &["new", "how-to", "docs/rotate.md"]);
    assert_eq!(code_of(&out), 0, "{}", stderr(&out));
    assert_eq!(
        fs::read_to_string(dir.path().join("docs/rotate.md")).unwrap(),
        started(dir.path(), None)
    );
    let out = ascribe(dir.path(), &["new", "how-to", "docs/guides"]);
    assert_eq!(code_of(&out), 2);
    assert_eq!(
        stderr(&out),
        "error: `docs/guides` is a folder, where a page is named for its title: give --title, or a page's path ending in .md\n"
    );
    assert!(!dir.path().join("docs/guides").exists());
}

#[test]
fn paths_are_from_the_current_folder() {
    let dir = project();
    fs::create_dir_all(dir.path().join("docs/guides")).unwrap();
    let here = dir.path().join("docs/guides");
    let out = ascribe(&here, &["new", "how-to", ".", "--title", "Rotate keys"]);
    assert_eq!(code_of(&out), 0, "{}", stderr(&out));
    assert!(
        stdout(&out).starts_with(
            "wrote rotate-keys.md, a page of type `how-to`, from ../../templates/how-to.md\n"
        ),
        "{}",
        stdout(&out)
    );
    assert!(here.join("rotate-keys.md").is_file());
    // Outside the content root, it refuses.
    let out = ascribe(dir.path(), &["new", "how-to", "rotate.md"]);
    assert_eq!(code_of(&out), 2);
    assert_eq!(
        stderr(&out),
        "error: `rotate.md` isn't in the content root, `docs`: a page goes there\n"
    );
}

#[test]
fn a_dry_run_prints_the_page_and_writes_nothing() {
    let dir = project();
    let out = ascribe(
        dir.path(),
        &[
            "new",
            "how-to",
            "docs/rotate.md",
            "--title",
            "Rotate keys",
            "--dry-run",
        ],
    );
    assert_eq!(code_of(&out), 0, "{}", stderr(&out));
    let text = stdout(&out);
    assert!(
        text.starts_with(
            "would write docs/rotate.md, a page of type `how-to`, from templates/how-to.md\n"
        ),
        "{text}"
    );
    assert!(text.ends_with(&format!("\n\n{}", started(dir.path(), Some("Rotate keys")))));
    assert!(!dir.path().join("docs/rotate.md").exists());
}

#[test]
fn json_says_what_was_written() {
    let dir = project();
    let out = ascribe(
        dir.path(),
        &[
            "new",
            "page",
            "docs/about.md",
            "--title",
            "About",
            "--format",
            "json",
        ],
    );
    assert_eq!(code_of(&out), 0, "{}", stderr(&out));
    let answer: Value = serde_json::from_str(&stdout(&out)).unwrap();
    assert_eq!(answer["schema_version"], 1);
    assert_eq!(answer["path"], "docs/about.md");
    assert_eq!(answer["type"], "page");
    assert_eq!(answer["title"], "About");
    // `page` has no template: the page is its frontmatter.
    assert_eq!(answer["template"], Value::Null);
    assert_eq!(answer["placeholders"], 0);
    assert_eq!(answer["written"], true);
    let text = fs::read_to_string(dir.path().join("docs/about.md")).unwrap();
    assert_eq!(answer["text"], text.as_str());
    assert_eq!(
        text,
        "---\ntype: page\ntitle: \"About\"\ndescription:\ndraft: true\n---\n"
    );
}

#[test]
fn a_type_the_model_doesnt_have_is_refused() {
    let dir = project();
    let out = ascribe(dir.path(), &["new", "recipe", "docs/a.md"]);
    assert_eq!(code_of(&out), 2);
    assert!(
        stderr(&out).starts_with(
            "error: the content model has no page type `recipe`; its types are `page`, `how-to`"
        ),
        "{}",
        stderr(&out)
    );
}
