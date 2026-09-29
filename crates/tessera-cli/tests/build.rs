//! `tessera build`: outputs, replacing a previous build, and reporting what
//! `tessera check` reports, by running the binary.

#![allow(clippy::expect_used, clippy::panic)]

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use tempfile::TempDir;

const MODEL: &str = r#"spec = "0.1"

[project]
content-root = "docs"

[types.page]
default = true

[types.page.frontmatter]
title = "string"

[dimensions.deployment]
values = ["cloud", "self-managed"]

[phrases]
product = "Quill"

[consumer]
site = "https://docs.example.com"

[builds.site]
variants = "switch"
availability = "badge"

[builds.cloud]
variants = { deployment = "cloud" }
availability = "badge"
"#;

fn project(model: &str, files: &[(&str, &str)]) -> TempDir {
    let dir = tempfile::tempdir().expect("a temporary directory");
    write(&dir.path().join("tessera.toml"), model);
    for (path, text) in files {
        write(&dir.path().join("docs").join(path), text);
    }
    dir
}

fn write(path: &Path, text: &str) {
    fs::create_dir_all(path.parent().expect("a parent")).expect("create directories");
    fs::write(path, text).expect("write a file");
}

fn read(path: &Path) -> String {
    fs::read_to_string(path).expect("read a file")
}

fn tessera(dir: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_tessera"))
        .current_dir(dir)
        .args(args)
        .env("NO_COLOR", "1")
        .output()
        .expect("run tessera")
}

fn code(output: &Output) -> i32 {
    output.status.code().expect("exited with a code")
}

fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

fn page(body: &str) -> String {
    format!("---\ntitle: A page\n---\n\n{body}")
}

fn copy_dir(from: &Path, to: &Path) {
    fs::create_dir_all(to).expect("a directory");
    for entry in fs::read_dir(from).expect("a listing") {
        let entry = entry.expect("an entry");
        let target = to.join(entry.file_name());
        if entry.file_type().expect("a type").is_dir() {
            copy_dir(&entry.path(), &target);
        } else {
            fs::copy(entry.path(), target).expect("a copy");
        }
    }
}

fn quill() -> TempDir {
    let dir = tempfile::tempdir().expect("a temporary directory");
    let source = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../examples/quill");
    copy_dir(&source, dir.path());
    dir
}

#[test]
fn builds_quill_under_every_build_with_both_emitters() {
    let dir = quill();
    let out = tessera(dir.path(), &["build"]);
    assert_eq!(code(&out), 0, "{}{}", stdout(&out), stderr(&out));
    let built = dir.path().join(".tessera/build");
    for build in ["site", "cloud", "self-managed-3.3"] {
        for (emitter, page) in [
            ("plain", "install-agent.md"),
            ("json", "install-agent.json"),
        ] {
            assert!(
                built.join(build).join(emitter).join(page).is_file(),
                "{build}/{emitter}"
            );
            assert!(
                built
                    .join(build)
                    .join(format!("{emitter}.manifest.json"))
                    .is_file()
            );
            assert!(
                built
                    .join(build)
                    .join(emitter)
                    .join("_fragments/prerequisites.png")
                    .is_file()
            );
        }
    }
    assert!(
        stderr(&out).contains("built cloud/plain: 3 pages, 2 assets"),
        "{}",
        stderr(&out)
    );
    // The report is what `tessera check` prints.
    let check = tessera(dir.path(), &["check"]);
    assert_eq!(stdout(&out), stdout(&check));
    // Building again changes nothing on disk that hasn't changed.
    let again = tessera(dir.path(), &["build"]);
    assert_eq!(code(&again), 0, "{}", stderr(&again));
}

#[test]
fn a_build_and_an_output_can_be_chosen() {
    let dir = quill();
    let out = tessera(
        dir.path(),
        &["build", "--build", "cloud", "--emit", "plain"],
    );
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    let built = dir.path().join(".tessera/build");
    assert!(built.join("cloud/plain/install-agent.md").is_file());
    assert!(!built.join("cloud/json").exists());
    assert!(!built.join("site").exists());
}

#[test]
fn the_site_output_and_unknown_builds_are_refused() {
    let dir = quill();
    let out = tessera(dir.path(), &["build", "--emit", "plain,site"]);
    assert_eq!(code(&out), 2);
    assert!(
        stderr(&out).contains("site output isn't available yet"),
        "{}",
        stderr(&out)
    );
    let out = tessera(dir.path(), &["build", "--build", "nope"]);
    assert_eq!(code(&out), 2);
    assert!(
        stderr(&out).contains("no build named `nope`"),
        "{}",
        stderr(&out)
    );
    assert!(stderr(&out).contains("`cloud`"), "{}", stderr(&out));
    assert!(!dir.path().join(".tessera").exists());
}

#[test]
fn removing_a_page_removes_its_old_output_and_leaves_users_files() {
    let dir = project(
        MODEL,
        &[
            ("index.md", &page("Home.\n")),
            ("gone.md", &page("Bye.\n")),
            ("g/deep.md", &page("Deep.\n")),
        ],
    );
    let out = tessera(dir.path(), &["build"]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    let built = dir.path().join(".tessera/build");
    // Files the user put in the output directory before the second build.
    write(&built.join("cloud/plain/notes.txt"), "my notes");
    write(&built.join("cloud/plain/g/mine.md"), "my file");
    write(&built.join("README.md"), "about this directory");
    for build in ["site", "cloud"] {
        assert!(built.join(build).join("plain/gone.md").is_file());
        assert!(built.join(build).join("json/gone.json").is_file());
    }

    fs::remove_file(dir.path().join("docs/gone.md")).expect("remove the page");
    fs::remove_file(dir.path().join("docs/g/deep.md")).expect("remove the page");
    let out = tessera(dir.path(), &["build"]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    assert!(
        stderr(&out).contains("removed 2 stale files"),
        "{}",
        stderr(&out)
    );
    for build in ["site", "cloud"] {
        for gone in [
            "plain/gone.md",
            "json/gone.json",
            "plain/g/deep.md",
            "json/g/deep.json",
        ] {
            assert!(!built.join(build).join(gone).exists(), "{build}/{gone}");
        }
        assert!(built.join(build).join("plain/index.md").is_file());
    }
    assert_eq!(read(&built.join("cloud/plain/notes.txt")), "my notes");
    assert_eq!(read(&built.join("cloud/plain/g/mine.md")), "my file");
    assert_eq!(read(&built.join("README.md")), "about this directory");
    // `g/` held only Tessera's file in the site build, so it's gone there; it still holds the user's in cloud.
    assert!(!built.join("site/plain/g").exists());
    assert!(built.join("cloud/plain/g").is_dir());
}

#[test]
fn a_page_a_selection_build_now_drops_loses_its_old_output_in_that_build_only() {
    let dir = project(
        MODEL,
        &[
            ("index.md", &page("Home.\n")),
            ("server.md", &page("Server.\n")),
        ],
    );
    assert_eq!(code(&tessera(dir.path(), &["build"])), 0);
    let built = dir.path().join(".tessera/build");
    write(&built.join("cloud/plain/notes.txt"), "my notes");
    assert!(built.join("cloud/plain/server.md").is_file());

    // The page now names a dimension value the cloud build doesn't select.
    write(
        &dir.path().join("docs/server.md"),
        "---\ntitle: A page\nvariant:\n  deployment: self-managed\n---\n\nServer.\n",
    );
    let out = tessera(dir.path(), &["build"]);
    assert_eq!(code(&out), 0, "{}{}", stdout(&out), stderr(&out));
    assert!(!built.join("cloud/plain/server.md").exists());
    assert!(!built.join("cloud/json/server.json").exists());
    assert!(
        built.join("site/plain/server.md").is_file(),
        "the switch build keeps every page"
    );
    assert_eq!(read(&built.join("cloud/plain/notes.txt")), "my notes");
}

#[test]
fn a_users_file_where_the_build_writes_fails_it_without_touching_anything() {
    let dir = project(MODEL, &[("index.md", &page("Home.\n"))]);
    let built = dir.path().join(".tessera/build");
    write(&built.join("cloud/plain/index.md"), "the user's own file");
    let out = tessera(dir.path(), &["build", "--build", "cloud"]);
    assert_eq!(code(&out), 2);
    assert!(
        stderr(&out).contains("isn't a file Tessera wrote"),
        "{}",
        stderr(&out)
    );
    assert_eq!(
        read(&built.join("cloud/plain/index.md")),
        "the user's own file"
    );
    assert!(!built.join("cloud/plain.manifest.json").exists());
    assert!(!built.join(".staging").exists());
}

#[test]
fn a_missing_site_origin_warns() {
    let model = MODEL.replace("[consumer]\nsite = \"https://docs.example.com\"\n", "");
    let dir = project(&model, &[("index.md", &page("[Self](index.md)\n"))]);
    let out = tessera(dir.path(), &["build", "--emit", "plain"]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    assert!(
        stderr(&out).contains("warning: [consumer] site isn't set"),
        "{}",
        stderr(&out)
    );
    assert_eq!(
        read(&dir.path().join(".tessera/build/site/plain/index.md")),
        "# A page\n\n[Self](/)\n"
    );
    // JSON has no absolute links to warn about.
    let out = tessera(dir.path(), &["build", "--emit", "json"]);
    assert!(!stderr(&out).contains("warning"), "{}", stderr(&out));
}

const BROKEN: &[(&str, &str)] = &[
    (
        "index.md",
        "---\ntitle: Home\n---\n\n[Gone](gone.md) and ![x](missing.png) and {nope}.\n\n@note {colour=red}: Hi.\n",
    ),
    ("other.md", "---\ntitle: Other\n---\n\nFine.\n"),
];

#[test]
fn build_reports_what_check_reports_and_writes_nothing() {
    let dir = project(MODEL, BROKEN);
    for format in ["text", "json"] {
        let check = tessera(dir.path(), &["check", "--format", format]);
        let build = tessera(dir.path(), &["build", "--format", format]);
        assert_eq!(code(&check), 1, "{}", stdout(&check));
        assert_eq!(code(&build), 1, "{}", stdout(&build));
        assert_eq!(stdout(&check), stdout(&build), "{format}");
        assert!(
            stderr(&build).contains("nothing was written"),
            "{}",
            stderr(&build)
        );
    }
    assert!(!dir.path().join(".tessera").exists());
}

#[test]
fn a_build_with_only_warnings_reports_them_and_builds() {
    let dir = project(MODEL, &[("index.md", &page("See {nope}.\n"))]);
    let check = tessera(dir.path(), &["check"]);
    let build = tessera(dir.path(), &["build"]);
    assert_eq!(code(&build), 0, "{}", stderr(&build));
    assert!(stdout(&build).contains("1 warning"), "{}", stdout(&build));
    // Two builds, and still one report: the same diagnostic isn't repeated per build.
    assert_eq!(stdout(&check), stdout(&build));
    assert!(
        dir.path()
            .join(".tessera/build/site/plain/index.md")
            .is_file()
    );
}

#[test]
fn a_content_model_with_errors_fails_like_check() {
    let dir = tempfile::tempdir().expect("a temporary directory");
    write(
        &dir.path().join("tessera.toml"),
        "spec = \"0.1\"\nbogus = 1\n",
    );
    let check = tessera(dir.path(), &["check"]);
    let build = tessera(dir.path(), &["build"]);
    assert_eq!(code(&build), 2);
    assert_eq!(code(&check), code(&build));
    assert_eq!(stdout(&check), stdout(&build));
    assert!(
        stderr(&build).contains("nothing can be built"),
        "{}",
        stderr(&build)
    );
}
