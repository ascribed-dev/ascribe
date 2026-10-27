//! The thread of OpenAPI's phase 1, by running the binary on a copy of
//! `examples/astro-site`: its guide links to `getLoom` in `[apis.loom]`'s
//! spec, `ascribe check` passes, `ascribe build` writes the link as Redoc's
//! address of the operation, and once the spec loses `getLoom`, `check`
//! fails with `link-operation-missing`.

#![allow(clippy::expect_used, clippy::panic)]

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

/// The example, as its repository has it.
fn example() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/astro-site")
}

fn copy(from: &Path, to: &Path) {
    fs::create_dir_all(to).expect("create a directory");
    for entry in fs::read_dir(from).expect("read a directory") {
        let entry = entry.expect("an entry");
        let target = to.join(entry.file_name());
        if entry.file_type().expect("a type").is_dir() {
            copy(&entry.path(), &target);
        } else {
            fs::copy(entry.path(), target).expect("copy a file");
        }
    }
}

fn ascribe(dir: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_ascribe"))
        .current_dir(dir)
        .args(args)
        .env("NO_COLOR", "1")
        .output()
        .expect("run ascribe")
}

fn text(output: &Output) -> String {
    format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

#[test]
fn openapi_a_guide_s_link_to_an_operation_is_checked_and_written_as_redoc_s_address() {
    let dir = tempfile::tempdir().expect("a temporary directory");
    let root = dir.path();
    fs::copy(example().join("ascribe.toml"), root.join("ascribe.toml")).expect("the model");
    copy(&example().join("api"), &root.join("api"));
    copy(&example().join("content"), &root.join("content"));

    let out = ascribe(root, &["check"]);
    assert_eq!(out.status.code(), Some(0), "{}", text(&out));

    let out = ascribe(root, &["build", "--emit", "site"]);
    assert_eq!(out.status.code(), Some(0), "{}", text(&out));
    let guide =
        fs::read_to_string(root.join(".ascribe/build/site/site/Guides/Teams/Share a loom.md"))
            .expect("the guide, built");
    assert!(
        guide.contains("[Get a loom](/docs/api#operation/getLoom)"),
        "{guide}"
    );
    assert!(
        guide.contains("[List invoices](/docs/api#operation/listInvoices)"),
        "{guide}"
    );

    let spec = root.join("api/openapi.yaml");
    let text_of_spec = fs::read_to_string(&spec).expect("the spec");
    fs::write(
        &spec,
        text_of_spec.replace("operationId: getLoom", "operationId: fetchLoom"),
    )
    .expect("the spec, without getLoom");
    let out = ascribe(root, &["check"]);
    let said = text(&out);
    assert_eq!(out.status.code(), Some(1), "{said}");
    assert!(said.contains("link-operation-missing"), "{said}");
    assert!(
        said.contains("`api/openapi.yaml` has no operation `getLoom`"),
        "{said}"
    );
}

// -- An id renamed since the base ----------------------------------------------

const WIDGETS_MODEL: &str = r#"spec = "0.1"

[project]
content-root = "content"

[apis.public]
spec = "api/openapi.yaml"
reference = { renderer = "redoc", at = "/api/" }

[checks]
api-operation-unreferenced = "off"
"#;

/// A spec with `createWidget` at `POST /widgets`.
const WIDGETS: &str = "openapi: 3.1.0\n\
info: { title: Widgets, version: \"1\" }\n\
paths:\n  /widgets:\n    post:\n      operationId: createWidget\n      summary: Create a widget\n\
  /widgets/{id}:\n    get:\n      operationId: getWidget\n      summary: Get a widget\n";

const GUIDE: &str = "---\ntitle: Widgets\n---\n\nMake one with [Create a widget](../api/openapi.yaml#createWidget).\n";

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
            "protocol.file.allow=always",
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

/// A repository with the widgets' guide and spec committed on `main`, and a
/// branch `regenerated` where the spec is `spec`, uncommitted.
fn widgets(spec: &str) -> tempfile::TempDir {
    let dir = tempfile::tempdir().expect("a temporary directory");
    let root = dir.path();
    git(root, &["init", "-q", "-b", "main"]);
    fs::write(root.join("ascribe.toml"), WIDGETS_MODEL).expect("the model");
    fs::create_dir_all(root.join("api")).expect("the spec's folder");
    fs::write(root.join("api/openapi.yaml"), WIDGETS).expect("the spec");
    fs::create_dir_all(root.join("content")).expect("the content");
    fs::write(root.join("content/index.md"), GUIDE).expect("the guide");
    git(root, &["add", "-A"]);
    git(root, &["commit", "-q", "-m", "first"]);
    git(root, &["switch", "-q", "-c", "regenerated"]);
    fs::write(root.join("api/openapi.yaml"), spec).expect("the spec, regenerated");
    dir
}

#[test]
fn openapi_a_renamed_id_is_offered_the_operation_at_its_method_and_path_in_the_base() {
    let dir = widgets(&WIDGETS.replace("createWidget", "widgets_create"));
    let root = dir.path();

    let out = ascribe(root, &["check"]);
    let said = text(&out);
    assert_eq!(out.status.code(), Some(1), "{said}");
    assert!(
        said.contains(
            "`api/openapi.yaml` has no operation `createWidget`; `createWidget` is now `widgets_create` (POST /widgets)"
        ),
        "{said}"
    );

    let out = ascribe(
        root,
        &[
            "link",
            "../api/openapi.yaml#createWidget",
            "--from",
            "content/index.md",
            "--format",
            "json",
        ],
    );
    let answer: serde_json::Value = serde_json::from_slice(&out.stdout).expect("JSON");
    assert_eq!(
        answer["closest"][0]["href"], "../api/openapi.yaml#widgets_create",
        "{answer}"
    );
    assert!(
        answer["problem"]
            .as_str()
            .is_some_and(|p| p.ends_with("`createWidget` is now `widgets_create` (POST /widgets)")),
        "{answer}"
    );

    let out = ascribe(root, &["check", "--fix"]);
    assert_eq!(out.status.code(), Some(0), "{}", text(&out));
    let guide = fs::read_to_string(root.join("content/index.md")).expect("the guide");
    assert!(
        guide.contains("(../api/openapi.yaml#widgets_create)"),
        "{guide}"
    );
}

#[test]
fn openapi_a_renamed_id_whose_path_moved_is_offered_the_closest() {
    let dir = widgets(
        &WIDGETS
            .replace("createWidget", "widgets_create")
            .replace("  /widgets:\n", "  /v2/widgets:\n"),
    );
    let root = dir.path();
    let out = ascribe(root, &["check", "--format", "json"]);
    let report: serde_json::Value = serde_json::from_slice(&out.stdout).expect("JSON");
    let said = report.to_string();
    assert!(
        said.contains("`api/openapi.yaml` has no operation `createWidget`; the closest is `"),
        "{said}"
    );
    assert!(!said.contains("is now"), "{said}");

    // Nothing is safe to apply.
    let out = ascribe(root, &["check", "--fix"]);
    assert_eq!(out.status.code(), Some(1), "{}", text(&out));
    let guide = fs::read_to_string(root.join("content/index.md")).expect("the guide");
    assert!(guide.contains("#createWidget)"), "{guide}");
}

#[test]
fn openapi_a_shallow_clone_offers_the_closest_without_an_error() {
    let dir = widgets(WIDGETS);
    let root = dir.path();
    fs::write(
        root.join("api/openapi.yaml"),
        WIDGETS.replace("createWidget", "widgets_create"),
    )
    .expect("the spec, regenerated");
    git(root, &["commit", "-q", "-am", "Regenerate the spec"]);
    let clone = tempfile::tempdir().expect("a temporary directory");
    let url = format!("file://{}", root.display());
    git(
        clone.path(),
        &[
            "clone",
            "-q",
            "--depth",
            "1",
            "--no-single-branch",
            "--branch",
            "regenerated",
            &url,
            "work",
        ],
    );
    let work = clone.path().join("work");
    let out = ascribe(&work, &["check", "--base", "origin/main"]);
    let said = text(&out);
    assert_eq!(out.status.code(), Some(1), "{said}");
    assert!(
        said.contains("`api/openapi.yaml` has no operation `createWidget`; the closest is `"),
        "{said}"
    );
    assert!(!said.contains("is now"), "{said}");
    assert!(!said.contains("error: "), "{said}");
}
