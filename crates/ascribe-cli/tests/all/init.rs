//! `ascribe init`, by running the binary in temporary folders: what it says
//! it wrote, its JSON, the project it starts checking clean with a page that
//! names its type, a folder of pages it adopts, and what it exits with when
//! it refuses. What it writes,
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
         wrote ascribe.addresses, where a page's move is recorded, so the addresses it published keep working\n\
         made docs/, the folder the pages go in\n\
         Next, write a first page, docs/index.md, and run `ascribe check`\n\
         Then, to publish the pages as a site with Astro, read https://ascribed-dev.com/go/astro-guide/\n"
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
        text.contains("wrote ascribe.toml\nwrote ascribe.addresses, where a page's move is recorded, so the addresses it published keep working\nmade docs/, the folder the pages go in\nadded [types."),
        "{text}"
    );
    assert!(
        text.ends_with("read https://ascribed-dev.com/go/astro-guide/\n"),
        "{text}"
    );
    let check = ascribe(dir.path(), &["check"]);
    assert_eq!(code_of(&check), 0, "{}{}", stdout(&check), stderr(&check));
    assert_eq!(stdout(&check), "No pages in docs/ yet.\n");

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

/// A check of the project `ascribe init` starts, before its first page,
/// says there are none, naming the content root as `ascribe.toml` does
/// from wherever it's run, and exits 0; once the page
/// is written, it's counted.
#[test]
fn check_before_the_first_page_says_there_are_none() {
    let dir = tempfile::tempdir().unwrap();
    let output = ascribe(dir.path(), &["init"]);
    assert_eq!(code_of(&output), 0, "{}", stderr(&output));
    for (from, said) in [
        (dir.path().to_path_buf(), "No pages in docs/ yet.\n"),
        (dir.path().join("docs"), "No pages in docs/ yet.\n"),
    ] {
        let check = ascribe(&from, &["check"]);
        assert_eq!(code_of(&check), 0, "{}", stderr(&check));
        assert_eq!(stdout(&check), said);
    }
    let concise = ascribe(dir.path(), &["check", "--format", "concise"]);
    assert_eq!(code_of(&concise), 0, "{}", stderr(&concise));
    assert_eq!(stdout(&concise), "No pages in docs/ yet.\n");

    fs::write(
        dir.path().join("docs/index.md"),
        "---\ntype: page\ntitle: Handbook\ndescription: How the team works.\n---\n\nWelcome.\n",
    )
    .unwrap();
    let check = ascribe(dir.path(), &["check"]);
    assert_eq!(code_of(&check), 0, "{}", stderr(&check));
    assert!(
        stdout(&check).starts_with("checked 1 file"),
        "{}",
        stdout(&check)
    );
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
    assert_eq!(
        answer["written"],
        serde_json::json!(["ascribe.toml", "ascribe.addresses"])
    );
    assert_eq!(answer["content_root"], "docs");
    assert_eq!(answer["content_root_made"], true);
    assert_eq!(answer["pages"], 0);
    assert_eq!(answer["templates"], Value::Null);
    assert_eq!(answer["next"], "https://ascribed-dev.com/go/astro-guide/");
    assert!(dir.path().join("site/ascribe.toml").is_file());
}

/// In a folder of pages, `init` writes an `ascribe.toml` that declares no
/// types, checks the pages, and exits 0 though they have problems: the
/// project is there. The output of `examples/existing-docs` is held by
/// `output.rs`.
#[test]
fn init_in_a_folder_of_pages_adopts_them_and_checks_them() {
    let dir = tempfile::tempdir().unwrap();
    fs::create_dir_all(dir.path().join("site/handbook")).unwrap();
    fs::write(
        dir.path().join("site/handbook/index.md"),
        "# Handbook\n\nHi.\n",
    )
    .unwrap();
    let output = ascribe(dir.path(), &["init", "site", "--content-root", "handbook"]);
    assert_eq!(code_of(&output), 0, "{}", stderr(&output));
    let text = stdout(&output);
    assert!(
        text.starts_with(
            "wrote site/ascribe.toml\nwrote site/ascribe.addresses, where a page's move is recorded, so the addresses it published keep working\nfound 1 page in site/handbook/; with no page types declared in ascribe.toml, no page needs `type:`\n1 page has no `title`; `ascribe check --fix` takes the heading it opens with as its title\n"
        ),
        "{text}"
    );
    assert!(
        text.contains("\nNext, run `ascribe check --fix`, which fixes 1 of them\n"),
        "{text}"
    );
    let model = fs::read_to_string(dir.path().join("site/ascribe.toml")).unwrap();
    assert!(model.contains("content-root = \"handbook\"\n"), "{model}");
    assert!(!model.contains("[types"), "{model}");

    let fix = ascribe(&dir.path().join("site"), &["check", "--fix"]);
    assert_eq!(code_of(&fix), 0, "{}", stdout(&fix));
    assert_eq!(
        fs::read_to_string(dir.path().join("site/handbook/index.md")).unwrap(),
        "---\ntitle: Handbook\n---\n\nHi.\n"
    );

    // Its JSON counts the pages.
    let dir = tempfile::tempdir().unwrap();
    fs::create_dir(dir.path().join("docs")).unwrap();
    fs::write(dir.path().join("docs/a.md"), "A.\n").unwrap();
    let output = ascribe(dir.path(), &["init", "--format", "json"]);
    assert_eq!(code_of(&output), 0, "{}", stderr(&output));
    let answer: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(answer["pages"], 1);
    assert_eq!(answer["content_root_made"], false);
}

#[test]
fn init_refuses_the_templates_in_a_folder_of_pages_and_a_content_root_outside() {
    let dir = tempfile::tempdir().unwrap();
    fs::create_dir(dir.path().join("docs")).unwrap();
    fs::write(dir.path().join("docs/a.md"), "A.\n").unwrap();
    let output = ascribe(dir.path(), &["init", "--templates"]);
    assert_eq!(code_of(&output), 2);
    assert_eq!(stdout(&output), "");
    assert!(
        stderr(&output).starts_with(
            "error: the templates declare page types, which every page in `docs/` (1 found)"
        ),
        "{}",
        stderr(&output)
    );
    let output = ascribe(dir.path(), &["init", "--content-root", "."]);
    assert_eq!(code_of(&output), 2);
    assert!(
        stderr(&output).contains("`.` isn't a folder inside it"),
        "{}",
        stderr(&output)
    );
    assert!(!dir.path().join("ascribe.toml").exists());
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
    assert!(!dir.path().join("docs/ascribe.addresses").exists());
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
    assert!(!dir.path().join("ascribe.addresses").exists());
    assert!(!dir.path().join("docs").exists());
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

/// A page renamed with `git mv` in a project `init` started, with no step
/// of its own to keep the addresses, loses the address it published:
/// `address-lost`, whose fix records the move.
#[test]
fn a_page_renamed_in_a_project_init_started_is_an_address_lost() {
    let dir = tempfile::tempdir().unwrap();
    git(dir.path(), &["init", "-q", "-b", "main"]);
    let output = ascribe(dir.path(), &["init"]);
    assert_eq!(code_of(&output), 0, "{}", stderr(&output));
    fs::create_dir(dir.path().join("docs/guides")).unwrap();
    fs::write(
        dir.path().join("docs/guides/configure.md"),
        "---\ntype: page\ntitle: Configure\ndescription: How to set it up.\n---\n\nSet it up.\n",
    )
    .unwrap();
    git(dir.path(), &["add", "-A"]);
    git(dir.path(), &["commit", "-q", "-m", "first"]);
    git(dir.path(), &["switch", "-q", "-c", "rename"]);
    git(
        dir.path(),
        &["mv", "docs/guides/configure.md", "docs/guides/setup.md"],
    );

    let check = ascribe(dir.path(), &["check", "--base", "main", "--format", "json"]);
    let report: Value = serde_json::from_slice(&check.stdout).unwrap();
    let lost: Vec<&Value> = report["diagnostics"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|d| d["slug"] == "address-lost")
        .collect();
    assert_eq!(lost.len(), 1, "{report:#}");
    assert!(
        lost[0]["message"]
            .as_str()
            .unwrap()
            .contains("/guides/configure/"),
        "{:#}",
        lost[0]
    );
    let fix = &lost[0]["fixes"][0];
    assert_eq!(
        fix["title"],
        "git sees it renamed to guides/setup.md: record git's renames, as `ascribe addresses move --from-git` does"
    );
    assert_eq!(fix["applicability"], "safe");

    let fixed = ascribe(dir.path(), &["check", "--base", "main", "--fix"]);
    let text = format!("{}{}", stdout(&fixed), stderr(&fixed));
    assert!(!text.contains("address-lost"), "{text}");
    let recorded = fs::read_to_string(dir.path().join("ascribe.addresses")).unwrap();
    assert!(
        recorded.contains("\"guides/configure.md\" = \"guides/setup.md\""),
        "{recorded}"
    );
}

/// Before the first commit, nothing can have been published, so a check
/// of a project `init` started compares no addresses and doesn't say so,
/// in a repository or outside one (decision #1016); a base named is still
/// one that can't be read.
#[test]
fn before_the_first_commit_the_addresses_compare_nothing_and_say_nothing() {
    let outside = tempfile::tempdir().unwrap();
    let unborn = tempfile::tempdir().unwrap();
    git(unborn.path(), &["init", "-q", "-b", "main"]);
    for dir in [outside.path(), unborn.path()] {
        let output = ascribe(dir, &["init"]);
        assert_eq!(code_of(&output), 0, "{}", stderr(&output));
        let check = ascribe(dir, &["check"]);
        assert_eq!(code_of(&check), 0, "{}", stderr(&check));
        assert_eq!(stdout(&check), "No pages in docs/ yet.\n");
        let named = ascribe(dir, &["check", "--base", "main", "--format", "concise"]);
        assert!(
            stdout(&named).contains("addresses weren't compared with the base"),
            "{}",
            stdout(&named)
        );
    }
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
