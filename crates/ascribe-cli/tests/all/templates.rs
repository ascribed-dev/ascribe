//! `ascribe templates`, by running the binary in temporary projects: what
//! `list` shows, what `eject` says it wrote, its JSON, what it exits with
//! when it refuses, where there's no `ascribe.toml`, and `--to` from another
//! folder. What eject writes, and every case it refuses, is held by
//! `crates/ascribe-templates/tests/all/eject.rs`.

#![allow(clippy::expect_used, clippy::unwrap_used, clippy::panic)]

use std::fs;
use std::path::Path;
use std::process::{Command, Output};

use serde_json::Value;
use tempfile::TempDir;

const MODEL: &str = "spec = \"0.1\"\n\n[project]\ncontent-root = \"docs\"\n";

fn project() -> TempDir {
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join("ascribe.toml"), MODEL).unwrap();
    fs::create_dir(dir.path().join("docs")).unwrap();
    fs::write(
        dir.path().join("docs/index.md"),
        "---\ntitle: Home\n---\n\nText.\n",
    )
    .unwrap();
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
fn list_shows_each_template_with_its_summary_and_where_the_set_is_from() {
    let dir = tempfile::tempdir().unwrap();
    let output = ascribe(dir.path(), &["templates", "list"]);
    assert_eq!(code_of(&output), 0, "{}", stderr(&output));
    let text = stdout(&output);
    let set = ascribe_templates::set();
    let width = set.templates.iter().map(|t| t.name.len()).max().unwrap();
    let lines: Vec<&str> = text.lines().collect();
    assert_eq!(lines.len(), set.templates.len() + 1, "{text}");
    for (line, template) in lines.iter().zip(&set.templates) {
        assert_eq!(
            *line,
            format!("{:width$}  {}", template.name, template.summary),
            "{text}"
        );
    }
    assert!(text.contains("The Good Docs Project"), "{text}");
}

#[test]
fn eject_says_what_it_wrote_and_the_project_checks_with_the_type() {
    let dir = project();
    let output = ascribe(dir.path(), &["templates", "eject", "how-to"]);
    assert_eq!(code_of(&output), 0, "{}", stderr(&output));
    assert_eq!(
        stdout(&output),
        "wrote templates/how-to.md\n\
         wrote templates/how-to.guide.md\n\
         wrote templates/how-to.ascribe.md\n\
         wrote templates/templates.lock\n\
         wrote ascribe.toml\n\
         added [types.page] to ascribe.toml: it declared no types, so every page was of this one, and the pages outside the new types keep it\n\
         added [types.how-to] to ascribe.toml, for pages under docs/how-to/\n\
         A type's folder is only a start: to keep its pages elsewhere, change the type's `files` in ascribe.toml.\n\
         To start a page, copy templates/how-to.md into docs/how-to/, and read templates/how-to.guide.md and templates/how-to.ascribe.md first.\n"
    );
    fs::create_dir(dir.path().join("docs/how-to")).unwrap();
    fs::copy(
        dir.path().join("templates/how-to.md"),
        dir.path().join("docs/how-to/rotate-keys.md"),
    )
    .unwrap();
    let check = ascribe(dir.path(), &["check", "--format", "json"]);
    let report: Value = serde_json::from_slice(&check.stdout).unwrap();
    let errors: Vec<&Value> = report["diagnostics"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|d| d["severity"] != "advice")
        .collect();
    assert!(errors.is_empty(), "{errors:#?}");
}

#[test]
fn eject_s_json_lists_what_it_wrote_and_added() {
    let dir = project();
    let output = ascribe(dir.path(), &["templates", "eject", "--format", "json"]);
    assert_eq!(code_of(&output), 0, "{}", stderr(&output));
    let answer: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(answer["schema_version"], 1);
    let set = ascribe_templates::set();
    let written = answer["written"].as_array().unwrap();
    assert_eq!(written.len(), set.templates.len() * 3 + 2, "{written:#?}");
    assert_eq!(written.last().unwrap(), "ascribe.toml");
    let added = answer["added"].as_array().unwrap();
    assert_eq!(added.len(), set.templates.len());
    for (added, template) in added.iter().zip(&set.templates) {
        assert_eq!(added["name"], template.name.as_str());
        assert_eq!(added["folder"], format!("docs/{}", template.name).as_str());
        assert_eq!(
            added["guide"],
            serde_json::json!([
                format!("templates/{}", template.guide),
                format!("templates/{}.ascribe.md", template.name)
            ])
        );
    }
    assert_eq!(answer["refused"], Value::Array(Vec::new()));
    assert_eq!(answer["page_declared"], true);
}

#[test]
fn eject_with_no_name_gives_six_types_whose_templates_check_and_are_formatted() {
    let dir = project();
    let output = ascribe(dir.path(), &["templates", "eject"]);
    assert_eq!(code_of(&output), 0, "{}", stderr(&output));
    for template in ascribe_templates::set().templates {
        let folder = dir.path().join("docs").join(&template.name);
        fs::create_dir(&folder).unwrap();
        fs::copy(
            dir.path().join("templates").join(&template.template),
            folder.join("page.md"),
        )
        .unwrap();
    }
    let check = ascribe(dir.path(), &["check", "--format", "json"]);
    let report: Value = serde_json::from_slice(&check.stdout).unwrap();
    let errors: Vec<&Value> = report["diagnostics"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|d| d["severity"] != "advice")
        .collect();
    assert!(errors.is_empty(), "{errors:#?}");
    let fmt = ascribe(dir.path(), &["fmt", "--check"]);
    assert_eq!(code_of(&fmt), 0, "{}{}", stdout(&fmt), stderr(&fmt));
}

#[test]
fn eject_refused_writes_nothing_and_exits_2() {
    let dir = project();
    assert_eq!(code_of(&ascribe(dir.path(), &["templates", "eject"])), 0);
    let model = fs::read_to_string(dir.path().join("ascribe.toml")).unwrap();
    let output = ascribe(dir.path(), &["templates", "eject", "how-to"]);
    assert_eq!(code_of(&output), 2);
    assert_eq!(stdout(&output), "");
    let err = stderr(&output);
    assert!(
        err.starts_with("error: how-to: ascribe.toml declares [types.how-to] already\n"),
        "{err}"
    );
    assert!(err.ends_with("wrote nothing\n"), "{err}");
    assert_eq!(
        fs::read_to_string(dir.path().join("ascribe.toml")).unwrap(),
        model
    );

    let json = ascribe(dir.path(), &["templates", "eject", "--format", "json"]);
    assert_eq!(code_of(&json), 2);
    let answer: Value = serde_json::from_slice(&json.stdout).unwrap();
    assert_eq!(answer["refused"][0]["reason"], "declared");
    assert_eq!(answer["refused"][1]["reason"], "file_in_the_way");
    assert_eq!(answer["refused"][1]["path"], "templates/how-to.md");
}

#[test]
fn eject_with_no_ascribe_toml_says_how_to_write_one() {
    let dir = tempfile::tempdir().unwrap();
    let output = ascribe(dir.path(), &["templates", "eject", "how-to"]);
    assert_eq!(code_of(&output), 2);
    let err = stderr(&output);
    assert!(err.starts_with("error: no ascribe.toml found"), "{err}");
    assert!(err.contains("`spec = \"0.1\"`"), "{err}");
    assert!(fs::read_dir(dir.path()).unwrap().next().is_none());
}

#[test]
fn to_is_a_folder_from_the_current_folder() {
    let dir = project();
    let here = dir.path().join("docs");
    let output = ascribe(&here, &["templates", "eject", "--to", "../writing"]);
    assert_eq!(code_of(&output), 0, "{}", stderr(&output));
    let text = stdout(&output);
    // Paths are shown from where it ran; ascribe.toml has them from its
    // own folder.
    assert!(text.starts_with("wrote ../writing/how-to.md\n"), "{text}");
    assert!(text.contains("for pages under how-to/\n"), "{text}");
    let model = fs::read_to_string(dir.path().join("ascribe.toml")).unwrap();
    assert!(
        model.contains("template = \"writing/how-to.md\"\n"),
        "{model}"
    );

    let outside = ascribe(dir.path(), &["templates", "eject", "--to", "../elsewhere"]);
    assert_eq!(code_of(&outside), 2);
    assert!(stderr(&outside).contains("isn't inside the folder ascribe.toml is in"));
}

fn copy_dir(from: &Path, to: &Path) {
    fs::create_dir_all(to).unwrap();
    for entry in fs::read_dir(from).unwrap() {
        let entry = entry.unwrap();
        let target = to.join(entry.file_name());
        if entry.file_type().unwrap().is_dir() {
            copy_dir(&entry.path(), &target);
        } else {
            fs::copy(entry.path(), &target).unwrap();
        }
    }
}

/// The diagnostics `ascribe check` reports on `file`, a path from the
/// project's folder.
fn reported_on(dir: &Path, file: &str) -> Vec<Value> {
    let check = ascribe(dir, &["check", "--format", "json"]);
    let report: Value = serde_json::from_slice(&check.stdout).unwrap();
    report["diagnostics"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|d| d["file"].as_str().is_some_and(|f| f.ends_with(file)))
        .cloned()
        .collect()
}

const FILLED: &str = "---
title: \"Rotate keys\"
description: \"Replace the keys Quill signs releases with, without a release going unsigned.\"
---

## Overview

This guide explains how to replace the keys Quill signs releases with.

## Before you start

Before you rotate the keys, ensure:

* You can publish a release.
* You have the old key's passphrase.

## Rotate the keys

@steps
1. Run `quill keys new`.

2. Run `quill keys retire` with the old key's name.

## See also

* [Install Quill](../install.md)
";

/// The phase's goal, in a copy of `examples/getting-started`: eject the
/// how-to, copy its template to a page, and `ascribe check` gives one advice
/// for each placeholder the page still has, its two frontmatter fields among
/// them, and the placeholder that runs across blocks once. Filled in, the
/// page has nothing to report.
#[test]
fn a_page_copied_from_the_template_has_an_advice_for_each_placeholder() {
    let dir = tempfile::tempdir().unwrap();
    let example = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/getting-started");
    copy_dir(&example, dir.path());
    let eject = ascribe(dir.path(), &["templates", "eject", "how-to"]);
    assert_eq!(code_of(&eject), 0, "{}", stderr(&eject));
    let template = fs::read_to_string(dir.path().join("templates/how-to.md")).unwrap();
    fs::create_dir_all(dir.path().join("docs/how-to")).unwrap();
    fs::write(dir.path().join("docs/how-to/rotate-keys.md"), &template).unwrap();
    // Linked to, so whether it's an orphan isn't in the way.
    let install = dir.path().join("docs/install.md");
    let mut text = fs::read_to_string(&install).unwrap();
    text.push_str("\nTo change its keys, see [Rotate keys](how-to/rotate-keys.md).\n");
    fs::write(&install, text).unwrap();

    let found = reported_on(dir.path(), "how-to/rotate-keys.md");
    assert!(
        found
            .iter()
            .all(|d| d["slug"] == "template-placeholder-left"),
        "{found:#?}"
    );
    // Each placeholder opens with the one brace the template escapes, and
    // the title and the description are the template's.
    let placeholders = template.matches("\\{").count();
    assert_eq!(found.len(), placeholders + 2, "{found:#?}");
    let see_also = found
        .iter()
        .filter(|d| {
            d["message"]
                .as_str()
                .is_some_and(|m| m.contains("{Include references"))
        })
        .count();
    assert_eq!(see_also, 1, "{found:#?}");

    fs::write(dir.path().join("docs/how-to/rotate-keys.md"), FILLED).unwrap();
    let found = reported_on(dir.path(), "how-to/rotate-keys.md");
    assert!(found.is_empty(), "{found:#?}");
}
