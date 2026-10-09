//! The commands that answer questions (`explain`, `model`, `outline`, `link`,
//! `refs`, and `render`): their JSON, exit codes, and limits, by running the
//! binary. `tests/output.rs` has their output on `examples/quill`, which the
//! command reference shows.

#![allow(clippy::expect_used, clippy::unwrap_used, clippy::panic)]

use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use serde_json::Value;
use tempfile::TempDir;

/// A model with two builds: `cloud` leaves out what's only for
/// self-managed.
const MODEL: &str = r#"spec = "0.1"

[project]
content-root = "docs"

[types.page]
default = true

[types.page.frontmatter]
title = "string"

[dimensions.deployment]
values = ["cloud", "self-managed"]
versionless = ["cloud", "self-managed"]

[phrases]
product = "Quill"

[editor]
build = "cloud"

[builds.cloud]
variants = { deployment = "cloud" }
availability = { filter = "cloud" }

[builds.self-managed]
variants = { deployment = "self-managed" }
availability = { filter = "self-managed" }
"#;

const INSTALL: &str = "---
title: Install Quill
---

@include: _fragments/before.md

## Configure
@id: configure

See [keys](keys.md#rotate-keys) and [{product}](https://quill.dev).

## Upgrade
@available: self-managed

Upgrade {product}.
";

const BEFORE: &str = "## Before you start

You need {product}.
";

const KEYS: &str = "---
title: API keys
---

## Rotate keys

Rotate them.
";

const SERVER: &str = "---
title: Run the server
available: self-managed
---

Run it.
";

/// A project in a subfolder, `site/`, of a temporary directory.
fn project() -> TempDir {
    let dir = tempfile::tempdir().expect("a temporary directory");
    let site = dir.path().join("site");
    write(&site.join("ascribe.toml"), MODEL);
    write(&site.join("docs/install.md"), INSTALL);
    write(&site.join("docs/_fragments/before.md"), BEFORE);
    write(&site.join("docs/keys.md"), KEYS);
    write(&site.join("docs/server.md"), SERVER);
    dir
}

fn write(path: &Path, text: &str) {
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

/// Runs `ascribe` and expects it to exit with `code`.
fn run(dir: &Path, args: &[&str], code: i32) -> Output {
    let out = ascribe(dir, args);
    assert_eq!(
        out.status.code(),
        Some(code),
        "ascribe {args:?}\nstdout: {}\nstderr: {}",
        stdout(&out),
        stderr(&out)
    );
    out
}

fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

/// Runs `ascribe ... --format json`, expects `code`, and checks the
/// document's version.
fn json(dir: &Path, args: &[&str], code: i32) -> Value {
    let mut args = args.to_vec();
    args.extend(["--format", "json"]);
    let out = run(dir, &args, code);
    let value: Value = serde_json::from_slice(&out.stdout).expect("the answer is JSON");
    assert_eq!(value["schema_version"], 1, "{value}");
    assert_eq!(value["ascribe_version"], env!("CARGO_PKG_VERSION"));
    value
}

fn repository_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

#[test]
fn explain_answers_by_code_or_name() {
    let dir = tempfile::tempdir().unwrap();
    let by_code = json(dir.path(), &["explain", "asc036"], 0);
    let by_name = json(dir.path(), &["explain", "link-target-missing"], 0);
    assert_eq!(by_code, by_name);
    assert_eq!(by_code["code"], "ASC036");
    assert_eq!(by_code["severity"], "error");
    assert!(by_code["example"]["wrong"].is_string());
    assert!(
        by_code["docs"]
            .as_str()
            .unwrap()
            .ends_with("/reference/diagnostics/#asc036-link-target-missing"),
        "{by_code}"
    );
}

#[test]
fn explain_names_the_closest_for_an_unknown_code() {
    let dir = tempfile::tempdir().unwrap();
    let out = run(dir.path(), &["explain", "link-targt"], 2);
    assert!(stdout(&out).is_empty());
    assert!(
        stderr(&out).contains("ASC036 link-target-missing"),
        "{}",
        stderr(&out)
    );
}

#[test]
fn explain_lists_every_diagnostic() {
    let dir = tempfile::tempdir().unwrap();
    let text = stdout(&run(dir.path(), &["explain", "--list"], 0));
    let listed = json(dir.path(), &["explain", "--list"], 0);
    let diagnostics = listed["diagnostics"].as_array().unwrap();
    assert_eq!(text.lines().count(), diagnostics.len());
    let registry =
        fs::read_to_string(repository_root().join("tests/conformance/diagnostics.toml")).unwrap();
    let codes = registry
        .lines()
        .filter(|l| l.starts_with("code = "))
        .count();
    assert_eq!(diagnostics.len(), codes);
    assert!(text.lines().any(|l| l == "ASC036 link-target-missing"));
}

#[test]
fn model_fits_its_budget_for_quill() {
    let root = repository_root();
    let text = stdout(&run(&root, &["model", "examples/quill"], 0));
    assert!(text.chars().count() <= 4000, "{} characters", text.len());
    assert!(!text.contains("more:"), "{text}");
    assert!(text.contains("`{product}`: Quill"), "{text}");
}

#[test]
fn model_reads_every_example_content_model() {
    let root = repository_root();
    for entry in fs::read_dir(root.join("examples/content-models")).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().is_none_or(|e| e != "toml") {
            continue;
        }
        let config = path.to_str().unwrap();
        let text = stdout(&run(&root, &["--config", config, "model"], 0));
        assert!(text.starts_with("# Content model"), "{config}: {text}");
        let value = json(&root, &["--config", config, "model"], 0);
        for section in [
            "types",
            "dimensions",
            "phrases",
            "features",
            "glossary",
            "widgets",
            "builds",
        ] {
            assert!(value[section].is_array(), "{config}: no {section}");
        }
    }
}

#[test]
fn model_cuts_a_long_list_and_names_the_section() {
    let dir = tempfile::tempdir().unwrap();
    let mut model = String::from("spec = \"0.1\"\n\n[phrases]\n");
    for i in 0..400 {
        let _ = writeln!(model, "phrase-{i} = \"The value of phrase number {i}\"");
    }
    write(&dir.path().join("ascribe.toml"), &model);
    fs::create_dir(dir.path().join("docs")).unwrap();
    let text = stdout(&run(dir.path(), &["model"], 0));
    assert!(text.chars().count() <= 4000, "{} characters", text.len());
    assert!(
        text.contains("more: `ascribe model --section phrases`"),
        "{text}"
    );
    let full = stdout(&run(dir.path(), &["model", "--section", "phrases"], 0));
    assert!(full.contains("phrase-399"), "{full}");
    let one = json(dir.path(), &["model", "--section", "phrases"], 0);
    assert_eq!(one["phrases"].as_array().unwrap().len(), 400);
    assert!(one.get("types").is_none(), "{one}");
}

#[test]
fn model_with_errors_says_to_check() {
    let dir = tempfile::tempdir().unwrap();
    write(
        &dir.path().join("ascribe.toml"),
        "spec = \"0.1\"\n[phrases\n",
    );
    let out = run(dir.path(), &["model"], 2);
    assert!(stderr(&out).contains("ascribe check"), "{}", stderr(&out));
}

#[test]
fn outline_lists_headings_from_fragments() {
    let dir = project();
    let value = json(dir.path(), &["outline", "site/docs/install.md"], 0);
    assert_eq!(value["page"], "install.md");
    assert_eq!(value["file"], "docs/install.md");
    assert_eq!(value["title"], "Install Quill");
    assert_eq!(value["type"], "page");
    let headings = value["headings"].as_array().unwrap();
    let ids: Vec<&str> = headings.iter().map(|h| h["id"].as_str().unwrap()).collect();
    assert_eq!(ids, ["before-you-start", "configure", "upgrade"]);
    assert_eq!(headings[0]["fragment"], "_fragments/before.md");
    assert_eq!(headings[0]["file"], "docs/_fragments/before.md");
    assert_eq!(headings[0]["line"], 1);
    assert_eq!(headings[1]["explicit_id"], true);
    assert_eq!(headings[2]["explicit_id"], false);
}

#[test]
fn outline_with_a_build_shows_what_it_publishes() {
    let dir = project();
    let value = json(
        dir.path(),
        &["outline", "site/docs/install.md", "--build", "cloud"],
        0,
    );
    let ids: Vec<&str> = value["headings"]
        .as_array()
        .unwrap()
        .iter()
        .map(|h| h["id"].as_str().unwrap())
        .collect();
    assert_eq!(ids, ["before-you-start", "configure"]);

    let out = run(
        dir.path(),
        &["outline", "site/docs/server.md", "--build", "cloud"],
        1,
    );
    assert!(
        stderr(&out).contains("build `cloud` doesn't publish `server.md`"),
        "{}",
        stderr(&out)
    );
}

#[test]
fn outline_of_a_file_that_isnt_a_source_fails() {
    let dir = project();
    let out = run(dir.path(), &["outline", "site/docs/missing.md"], 2);
    assert!(stderr(&out).contains("missing.md"), "{}", stderr(&out));
}

#[test]
fn link_answers_from_a_page() {
    let dir = project();
    let from = "site/docs/install.md";
    let value = json(
        dir.path(),
        &["link", "keys.md#rotate-keys", "--from", from],
        0,
    );
    assert_eq!(value["exists"], true);
    assert_eq!(value["kind"], "heading");
    assert_eq!(value["title"], "Rotate keys");
    assert_eq!(value["href"], "keys.md#rotate-keys");

    let own = json(dir.path(), &["link", "#configure", "--from", from], 0);
    assert_eq!(own["href"], "#configure");

    let external = json(
        dir.path(),
        &["link", "https://quill.dev", "--from", from],
        0,
    );
    assert_eq!(external["kind"], "external");

    let missing = json(dir.path(), &["link", "key.md", "--from", from], 1);
    assert_eq!(missing["exists"], false);
    assert!(missing["href"].is_null());
    assert_eq!(missing["closest"][0]["path"], "keys.md");

    let fragment = json(
        dir.path(),
        &["link", "_fragments/before.md", "--from", from],
        1,
    );
    assert_eq!(fragment["kind"], "fragment");
    assert_eq!(fragment["closest"][0]["path"], "install.md");
}

#[test]
fn refs_finds_uses_through_includes() {
    let dir = project();
    let heading = json(dir.path(), &["refs", "site/docs/keys.md#rotate-keys"], 0);
    assert_eq!(heading["kind"], "heading");
    assert_eq!(heading["total"], 1);
    assert_eq!(heading["places"][0]["file"], "docs/install.md");
    assert_eq!(heading["places"][0]["use"], "link");

    let fragment = json(dir.path(), &["refs", "site/docs/_fragments/before.md"], 0);
    assert_eq!(fragment["kind"], "fragment");
    assert_eq!(fragment["places"][0]["use"], "include");
    assert_eq!(fragment["places"][0]["line"], 5);

    let phrase = json(
        dir.path(),
        &["refs", "phrase:{product}", "--project", "site"],
        0,
    );
    assert_eq!(phrase["total"], 3);
    assert_eq!(phrase["truncated"], false);
    assert!(phrase["next_command"].is_null());
}

#[test]
fn refs_cuts_the_list_and_says_how_to_see_it_all() {
    let dir = project();
    let args = [
        "refs",
        "phrase:product",
        "--project",
        "site",
        "--limit",
        "1",
    ];
    let value = json(dir.path(), &args, 0);
    assert_eq!(value["shown"], 1);
    assert_eq!(value["total"], 3);
    assert_eq!(value["truncated"], true);
    assert_eq!(
        value["next_command"],
        "ascribe refs phrase:product --limit 3 --project site --format json"
    );
    let text = stdout(&run(dir.path(), &args, 0));
    assert!(text.contains("Showing 1 of them"), "{text}");
}

#[test]
fn refs_to_nothing_is_exit_code_1() {
    let dir = project();
    let value = json(dir.path(), &["refs", "phrase:nope", "--project", "site"], 1);
    assert_eq!(value["exists"], false);
    run(dir.path(), &["refs", "site/docs/keys.md#nope"], 1);
    run(dir.path(), &["refs", "phrase:", "--project", "site"], 2);
}

#[test]
fn render_writes_the_page_a_build_publishes() {
    let dir = project();
    let page = "site/docs/install.md";
    let text = stdout(&run(dir.path(), &["render", page, "--build", "cloud"], 0));
    assert!(text.contains("You need Quill."), "{text}");
    assert!(!text.contains("Upgrade"), "{text}");
    assert!(!text.starts_with("---"), "{text}");
    let other = stdout(&run(
        dir.path(),
        &["render", page, "--build", "self-managed", "--frontmatter"],
        0,
    ));
    assert!(
        other.starts_with("---\ntitle: Install Quill\n---\n"),
        "{other}"
    );
    assert!(other.contains("Upgrade Quill."), "{other}");
}

#[test]
fn render_says_why_a_build_leaves_a_page_out() {
    let dir = project();
    let out = run(
        dir.path(),
        &["render", "site/docs/server.md", "--build", "cloud"],
        1,
    );
    assert!(stdout(&out).is_empty(), "{}", stdout(&out));
    assert!(stderr(&out).contains("cloud"), "{}", stderr(&out));
    let value = json(
        dir.path(),
        &["render", "site/docs/server.md", "--build", "cloud"],
        1,
    );
    assert!(value["not_published"].is_string());
    assert_eq!(value["text"], "");
}

#[test]
fn render_needs_a_build_and_a_page() {
    let dir = project();
    let out = run(dir.path(), &["render", "site/docs/keys.md"], 2);
    assert!(stderr(&out).contains("--build"), "{}", stderr(&out));
    run(
        dir.path(),
        &[
            "render",
            "site/docs/_fragments/before.md",
            "--build",
            "cloud",
        ],
        2,
    );
    run(
        dir.path(),
        &["render", "site/docs/keys.md", "--build", "nope"],
        2,
    );
}
