//! `ascribe build`: outputs, replacing a previous build, and reporting what
//! `ascribe check` reports, by running the binary.

#![allow(clippy::expect_used, clippy::panic)]

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use tempfile::TempDir;

use crate::links;

const MODEL: &str = r#"spec = "0.1"

[project]
content-root = "docs"

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
    write(&dir.path().join("ascribe.toml"), model);
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

fn ascribe(dir: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_ascribe"))
        .current_dir(dir)
        .args(args)
        .env("NO_COLOR", "1")
        .output()
        .expect("run ascribe")
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

/// What `ascribe build` prints of `ascribe check`'s text report: its
/// errors, each in full, without the warnings and advice or the summary
/// line.
fn errors_of(check: &str) -> String {
    let mut errors = String::new();
    let mut keep = false;
    let lines: Vec<&str> = check.lines().collect();
    let (_, report) = lines.split_last().unwrap_or((&"", &[]));
    for line in report {
        if line.starts_with("[ASC") {
            keep = line.contains("] Error: ");
        }
        if keep {
            errors.push_str(line);
            errors.push('\n');
        }
    }
    errors
}

/// `ascribe build`'s text report without its summary line.
fn without_summary(build: &str) -> String {
    let mut lines: Vec<&str> = build.lines().collect();
    lines.pop();
    lines.iter().map(|l| format!("{l}\n")).collect()
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
    let out = ascribe(dir.path(), &["build"]);
    assert_eq!(code(&out), 0, "{}{}", stdout(&out), stderr(&out));
    let built = dir.path().join(".ascribe/build");
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
        stderr(&out).contains("built cloud: 3 pages, 2 assets (site, plain, json)\n"),
        "{}",
        stderr(&out)
    );
    // One line per build.
    assert_eq!(stderr(&out).lines().count(), 4, "{}", stderr(&out));
    // The advice isn't shown, only counted, with where to see it.
    assert_eq!(
        stdout(&out),
        "checked 8 files in every build: 0 errors, 0 warnings, 3 advice: ascribe check for each\n"
    );
    // Building again changes nothing on disk that hasn't changed.
    let again = ascribe(dir.path(), &["build"]);
    assert_eq!(code(&again), 0, "{}", stderr(&again));
}

#[test]
fn a_project_nested_in_the_content_root_is_not_built() {
    let dir = project(
        MODEL,
        &[
            ("index.md", &page("Home.\n")),
            ("nested/ascribe.toml", MODEL),
            ("nested/docs/inner.md", &page("[Gone](gone.md)\n")),
        ],
    );
    let out = ascribe(dir.path(), &["build", "--build", "site", "--emit", "plain"]);
    assert_eq!(code(&out), 0, "{}{}", stdout(&out), stderr(&out));
    let plain = dir.path().join(".ascribe/build/site/plain");
    assert!(plain.join("index.md").is_file());
    assert!(!plain.join("nested").exists());
    assert!(
        stderr(&out).contains("built site: 1 page, 0 assets (plain)\n"),
        "{}",
        stderr(&out)
    );
}

#[test]
fn a_build_and_an_output_can_be_chosen() {
    let dir = quill();
    let out = ascribe(
        dir.path(),
        &["build", "--build", "cloud", "--emit", "plain"],
    );
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    let built = dir.path().join(".ascribe/build");
    assert!(built.join("cloud/plain/install-agent.md").is_file());
    assert!(!built.join("cloud/json").exists());
    assert!(!built.join("site").exists());
}

#[test]
fn the_site_output_is_built_under_every_build() {
    let dir = quill();
    let out = ascribe(dir.path(), &["build", "--emit", "site"]);
    assert_eq!(code(&out), 0, "{}{}", stdout(&out), stderr(&out));
    let built = dir.path().join(".ascribe/build");
    for build in ["site", "cloud", "self-managed-3.3"] {
        let root = built.join(build).join("site");
        for file in [
            "install-agent.md",
            "keys.md",
            "quickstart.md",
            "playground.png",
            "_fragments/prerequisites.png",
            "_ascribe/schema.ts",
        ] {
            assert!(root.join(file).is_file(), "{build}/site/{file}");
        }
        assert!(built.join(build).join("site.manifest.json").is_file());
        assert!(!built.join(build).join("plain").exists());
    }
    assert!(
        stderr(&out).contains("built cloud: 3 pages, 2 assets (site)\n"),
        "{}",
        stderr(&out)
    );
    let page = read(&built.join("cloud/site/install-agent.md"));
    assert!(page.contains("<ascribe-tabs sync=\"pm\">"), "{page}");
    assert!(page.contains("(/quickstart/#try-in-browser)"), "{page}");
}

#[test]
fn anchors_mark_the_site_output_and_the_manifest_says_so() {
    let dir = quill();
    let built = dir.path().join(".ascribe/build/site");
    let out = ascribe(dir.path(), &["build", "--build", "site", "--anchors"]);
    assert_eq!(code(&out), 0, "{}{}", stdout(&out), stderr(&out));
    let page = read(&built.join("site/quickstart.md"));
    assert!(
        page.contains(
            "<!--ascribe-anchor tag=\"h2\" source=\"quickstart.md:7-7\"-->\n## Try in the browser"
        ),
        "{page}"
    );
    assert!(read(&built.join("site.manifest.json")).contains("\"anchors\": true"));
    // Only the site output has anchors.
    assert!(!read(&built.join("plain/quickstart.md")).contains("ascribe-anchor"));
    assert!(!read(&built.join("plain.manifest.json")).contains("anchors"));

    // Without the flag, the output and the manifest are as they always were.
    let out = ascribe(dir.path(), &["build", "--build", "site"]);
    assert_eq!(code(&out), 0, "{}{}", stdout(&out), stderr(&out));
    assert!(!read(&built.join("site/quickstart.md")).contains("ascribe-anchor"));
    assert!(!read(&built.join("site.manifest.json")).contains("anchors"));
}

#[test]
fn a_generated_file_isnt_counted_as_an_asset() {
    let dir = project(MODEL, &[("index.md", &page("Hi.\n"))]);
    let out = ascribe(dir.path(), &["build", "--build", "site", "--emit", "site"]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    assert!(
        stderr(&out).contains("built site: 1 page, 0 assets (site)\n"),
        "{}",
        stderr(&out)
    );
    assert!(
        dir.path()
            .join(".ascribe/build/site/site/_ascribe/schema.ts")
            .is_file()
    );
}

#[test]
fn every_output_is_built_by_default() {
    let dir = quill();
    let out = ascribe(dir.path(), &["build", "--build", "cloud"]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    let built = dir.path().join(".ascribe/build/cloud");
    for emitter in ["site", "plain", "json"] {
        assert!(built.join(emitter).is_dir(), "{emitter}");
    }
}

#[test]
fn two_pages_with_one_route_fail_the_site_output() {
    // Without the outputs for agents, whose plain output is laid out by
    // route too.
    let model = MODEL.replace(
        "site = \"https://docs.example.com\"",
        "site = \"https://docs.example.com\"\nagents = false",
    );
    let dir = project(
        &model,
        &[("My File.md", &page("A.\n")), ("my-file.md", &page("B.\n"))],
    );
    let out = ascribe(dir.path(), &["build", "--emit", "site"]);
    assert_eq!(code(&out), 2, "{}", stderr(&out));
    assert!(
        stderr(&out).contains("can't publish two pages at one route"),
        "{}",
        stderr(&out)
    );
    // Nothing is written for the output that failed.
    assert!(!dir.path().join(".ascribe/build/site/site").exists());
    // The other outputs don't have the problem.
    let out = ascribe(dir.path(), &["build", "--emit", "plain"]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
}

#[test]
fn unknown_builds_are_refused() {
    let dir = quill();
    let out = ascribe(dir.path(), &["build", "--build", "nope"]);
    assert_eq!(code(&out), 2);
    assert!(stderr(&out).contains("no build `nope`"), "{}", stderr(&out));
    assert!(
        stderr(&out).contains("its builds are site, site@3.4, cloud"),
        "{}",
        stderr(&out)
    );
    assert!(!dir.path().join(".ascribe").exists());
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
    let out = ascribe(dir.path(), &["build"]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    let built = dir.path().join(".ascribe/build");
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
    let out = ascribe(dir.path(), &["build"]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    assert!(
        stderr(&out)
            .contains("built site: 1 page, 0 assets (site, plain, json), removed 6 stale files\n"),
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
    // `g/` held only Ascribe's file in the site build, so it's gone there; it still holds the user's in cloud.
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
    assert_eq!(code(&ascribe(dir.path(), &["build"])), 0);
    let built = dir.path().join(".ascribe/build");
    write(&built.join("cloud/plain/notes.txt"), "my notes");
    assert!(built.join("cloud/plain/server.md").is_file());

    // The page now names a dimension value the cloud build doesn't select.
    write(
        &dir.path().join("docs/server.md"),
        "---\ntitle: A page\nvariant:\n  deployment: self-managed\n---\n\nServer.\n",
    );
    let out = ascribe(dir.path(), &["build"]);
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
    let built = dir.path().join(".ascribe/build");
    write(&built.join("cloud/plain/index.md"), "the user's own file");
    let out = ascribe(dir.path(), &["build", "--build", "cloud"]);
    assert_eq!(code(&out), 2);
    assert!(
        stderr(&out).contains("isn't a file Ascribe wrote"),
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
    let out = ascribe(dir.path(), &["build", "--emit", "plain"]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    assert!(
        stderr(&out).contains("warning: [consumer] site isn't set"),
        "{}",
        stderr(&out)
    );
    assert_eq!(
        read(&dir.path().join(".ascribe/build/site/plain/index.md")),
        "# A page\n\n[Self](/)\n"
    );
    // JSON has no absolute links to warn about.
    let out = ascribe(dir.path(), &["build", "--emit", "json"]);
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
        let check = ascribe(dir.path(), &["check", "--format", format]);
        let build = ascribe(dir.path(), &["build", "--format", format]);
        assert_eq!(code(&check), 1, "{}", stdout(&check));
        assert_eq!(code(&build), 1, "{}", stdout(&build));
        if format == "json" {
            assert_eq!(stdout(&check), stdout(&build));
        } else {
            assert_eq!(errors_of(&stdout(&check)), without_summary(&stdout(&build)));
        }
        assert!(
            stderr(&build).contains("nothing was written"),
            "{}",
            stderr(&build)
        );
    }
    assert!(!dir.path().join(".ascribe").exists());
}

#[test]
fn a_build_with_only_warnings_reports_them_and_builds() {
    let dir = project(MODEL, &[("index.md", &page("See {nope}.\n"))]);
    let check = ascribe(dir.path(), &["check"]);
    let build = ascribe(dir.path(), &["build"]);
    assert_eq!(code(&build), 0, "{}", stderr(&build));
    // The warning is counted, not shown, and the check lists it.
    assert_eq!(
        stdout(&build),
        "checked 1 file in every build: 0 errors, 1 warning, 1 advice: ascribe check for each\n"
    );
    assert!(stdout(&check).contains("1 warning"), "{}", stdout(&check));
    assert!(
        dir.path()
            .join(".ascribe/build/site/plain/index.md")
            .is_file()
    );
}

#[test]
fn a_content_model_with_errors_fails_like_check() {
    let dir = tempfile::tempdir().expect("a temporary directory");
    write(
        &dir.path().join("ascribe.toml"),
        "spec = \"0.1\"\nbogus = 1\n",
    );
    let check = ascribe(dir.path(), &["check"]);
    let build = ascribe(dir.path(), &["build"]);
    assert_eq!(code(&build), 2);
    assert_eq!(code(&check), code(&build));
    assert_eq!(stdout(&check), stdout(&build));
    assert!(
        stderr(&build).contains("nothing can be built"),
        "{}",
        stderr(&build)
    );
}

#[test]
fn build_reports_what_check_reports_for_page_level_problems_too() {
    // A duplicate id is found on the assembled page, in every build; a group
    // with no arm left in the cloud build only in that one.
    let source = "---\ntitle: Home\n---\n\n## One\n@id: same\n\n## Two\n@id: same\n\n@variant {deployment=self-managed}:\nServer.\n@end\n";
    let dir = project(MODEL, &[("index.md", source)]);
    for args in [vec![], vec!["--build", "cloud"], vec!["--build", "site"]] {
        for format in ["text", "json"] {
            let mut check = vec!["check", "--format", format];
            let mut build = vec!["build", "--format", format];
            check.extend(&args);
            build.extend(&args);
            let (check, build) = (ascribe(dir.path(), &check), ascribe(dir.path(), &build));
            assert_eq!(code(&check), 1, "{args:?}: {}", stdout(&check));
            assert_eq!(code(&build), 1, "{args:?}");
            if format == "json" {
                assert_eq!(stdout(&check), stdout(&build), "{args:?}");
            } else {
                assert_eq!(
                    errors_of(&stdout(&check)),
                    without_summary(&stdout(&build)),
                    "{args:?}"
                );
            }
        }
    }
    assert!(stdout(&ascribe(dir.path(), &["build"])).contains("id-duplicate"));
    assert!(!dir.path().join(".ascribe").exists());
}

#[test]
fn several_builds_and_unknown_builds_are_reported_as_check_reports_them() {
    // Two selection builds both drop `sm.md`, which `index.md` links to.
    let model = format!(
        "{MODEL}\n[builds.cloud-pdf]\nvariants = {{ deployment = \"cloud\" }}\navailability = \"badge\"\n"
    );
    let dir = project(
        &model,
        &[
            ("index.md", &page("[Server](sm.md)\n")),
            (
                "sm.md",
                "---\ntitle: Server\nvariant:\n  deployment: self-managed\n---\n\nx\n",
            ),
        ],
    );
    for args in [
        vec![],
        vec!["--build", "cloud", "--build", "cloud-pdf"],
        vec!["--build", "cloud"],
        vec!["--build", "cloud", "--build", "site"],
        vec!["--build", "nope"],
        vec!["--build", "cloud", "--build", "nope"],
    ] {
        for format in ["text", "json"] {
            let mut check = vec!["check", "--format", format];
            let mut build = vec!["build", "--format", format];
            check.extend(&args);
            build.extend(&args);
            let (check, build) = (ascribe(dir.path(), &check), ascribe(dir.path(), &build));
            assert_eq!(code(&check), code(&build), "{args:?} {format}");
            if format == "json" || code(&check) == 2 {
                assert_eq!(stdout(&check), stdout(&build), "{args:?} {format}");
            } else {
                assert_eq!(
                    errors_of(&stdout(&check)),
                    without_summary(&stdout(&build)),
                    "{args:?}"
                );
            }
            assert_eq!(
                stderr(&check),
                stderr(&build).replace("error: the build failed; nothing was written\n", ""),
                "{args:?} {format}"
            );
        }
    }
    // Both builds share one diagnostic that names them.
    let out = ascribe(
        dir.path(),
        &["build", "--build", "cloud", "--build", "cloud-pdf"],
    );
    assert!(stdout(&out).contains("1 error"), "{}", stdout(&out));
    assert!(
        stdout(&out).contains("builds `cloud`, `cloud-pdf`"),
        "{}",
        stdout(&out)
    );
    let out = ascribe(dir.path(), &["build", "--build", "nope"]);
    assert_eq!(code(&out), 2);
    assert!(
        stderr(&out).contains("no build `nope`; its builds are"),
        "{}",
        stderr(&out)
    );
    assert!(!dir.path().join(".ascribe").exists());
}

#[test]
fn a_committed_link_in_the_output_directory_fails_the_build_and_nothing_outside_changes() {
    let dir = project(MODEL, &[("index.md", &page("Home.\n"))]);
    let outside = tempfile::tempdir().expect("a temporary directory");
    write(&outside.path().join("victim.md"), "keep me");
    // As a repository could commit them: an emitter root that's a link to a
    // folder outside, and a manifest listing a file there.
    let site = dir.path().join(".ascribe").join("build").join("site");
    fs::create_dir_all(&site).expect("create");
    links::dir(outside.path(), site.join("plain"));
    write(
        &site.join("plain.manifest.json"),
        r#"{"format":"ascribe-manifest","version":1,"build":"site","emitter":"plain","files":[{"path":"victim.md","kind":"page"}]}"#,
    );
    let out = ascribe(dir.path(), &["build"]);
    assert_eq!(code(&out), 2, "{}", stderr(&out));
    // The output directory is written as the model has it (`.ascribe/build`),
    // so on Windows the message mixes separators: compare with one.
    let slashes = |text: &str| text.replace('\\', "/");
    let shown = slashes(&site.join("plain").display().to_string());
    assert!(
        slashes(&stderr(&out)).contains(&shown) && stderr(&out).contains("is a link"),
        "{}",
        stderr(&out)
    );
    assert_eq!(read(&outside.path().join("victim.md")), "keep me");
    assert!(!outside.path().join("index.md").exists());
}

#[test]
fn a_committed_link_above_the_output_directory_fails_the_build() {
    let dir = project(MODEL, &[("index.md", &page("Home.\n"))]);
    let outside = tempfile::tempdir().expect("a temporary directory");
    links::dir(outside.path(), dir.path().join(".ascribe"));
    let out = ascribe(dir.path(), &["build"]);
    assert_eq!(code(&out), 2, "{}", stderr(&out));
    assert!(stderr(&out).contains("is a link"), "{}", stderr(&out));
    assert_eq!(
        fs::read_dir(outside.path()).expect("a listing").count(),
        0,
        "the build wrote through the link"
    );
}

/// With no repository around the project, as in a copy made with `git
/// archive`, `ascribe build` still reads a source above the project's folder
/// (decision 3; decision 77 bounds only the editor and `ascribe mcp`).
#[test]
fn with_no_repository_a_build_reads_a_source_above_the_project() {
    let dir = tempfile::tempdir().expect("a temporary directory");
    write(
        &dir.path().join("src/lib.rs"),
        "// :snippet-start: api\npub fn api() {}\n// :snippet-end:\n",
    );
    let project = dir.path().join("docs");
    write(
        &project.join("ascribe.toml"),
        "spec = \"0.1\"\n\n[project]\ncontent-root = \"content\"\n\n[sources.code]\npath = \"..\"\ninclude = [\"src/**\"]\n",
    );
    write(
        &project.join("content/index.md"),
        "---\ntitle: Home\n---\n\n@snippet: code:src/lib.rs#api\n",
    );
    let out = ascribe(&project, &["build", "--emit", "plain"]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    let built = project.join(".ascribe").join("build");
    let build = fs::read_dir(&built)
        .expect("the output")
        .map(|e| e.expect("an entry").path())
        .find(|p| p.is_dir() && !p.ends_with(".staging"))
        .expect("a build");
    let page = read(&build.join("plain").join("index.md"));
    assert!(page.contains("pub fn api() {}"), "{page}");
}

const VERSIONED: &str = r#"spec = "0.1"

[project]
content-root = "docs"

[dimensions.deployment]
values = ["cloud", "self-managed"]
versionless = ["cloud"]

[releases.self-managed]
live = ["3.4", "3.5"]

[consumer]
site = "https://docs.example.com"
base-path = "/docs"

[builds.site]
versions = "self-managed"
"#;

fn versioned() -> TempDir {
    project(
        VERSIONED,
        &[
            (
                "index.md",
                "---\ntitle: Home\n---\n\nSee [Install](install.md) and ![a picture](shot.png).\n",
            ),
            (
                "install.md",
                "---\ntitle: Install\n---\n\nBack [home](index.md).\n",
            ),
            (
                "webhooks.md",
                "---\ntitle: Webhooks\navailable: self-managed 3.5\n---\n\n[Install](install.md).\n",
            ),
            ("shot.png", "not really a picture"),
        ],
    )
}

#[test]
fn a_versioned_build_writes_an_output_for_each_live_line() {
    let dir = versioned();
    let out = ascribe(dir.path(), &["build", "--emit", "site"]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    let build = dir.path().join(".ascribe/build");
    // The current line keeps the build's name, and is at the base path.
    let current = build.join("site/site");
    assert!(current.join("webhooks.md").exists());
    assert!(read(&current.join("index.md")).contains("](/docs/install/)"));
    // The older line is `site@3.4`, under its number, without what 3.5 added.
    let older = build.join("site@3.4/site");
    assert!(!older.join("webhooks.md").exists());
    let index = read(&older.join("index.md"));
    assert!(index.contains("](/docs/3.4/install/)"), "{index}");
    // An image is relative to the page, so it's under the line's folder too.
    assert!(index.contains("](./shot.png)"), "{index}");
    assert!(older.join("shot.png").exists());
    assert!(read(&older.join("install.md")).contains("](/docs/3.4/)"));
    let pages = read(&older.join("_ascribe/pages.json"));
    // Its route is under its folder, and its canonical address is the
    // current line's page.
    assert!(pages.contains("\"/docs/3.4/install/\""), "{pages}");
    assert!(
        pages.contains("\"canonical\": \"https://docs.example.com/docs/install/\""),
        "{pages}"
    );
}

#[test]
fn each_line_points_agents_to_its_own_markdown_and_index() {
    let dir = versioned();
    let config = dir.path().join("ascribe.toml");
    let text = read(&config).replace(
        "base-path = \"/docs\"",
        "base-path = \"/docs\"\nagents = true",
    );
    std::fs::write(&config, text).expect("write ascribe.toml");
    let out = ascribe(dir.path(), &["build", "--emit", "site,plain"]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    let build = dir.path().join(".ascribe/build");
    let current = read(&build.join("site/site/install.md"));
    assert!(current.contains("(/docs/llms.txt)"), "{current}");
    let index = read(&build.join("site/plain/llms.txt"));
    assert!(index.starts_with("# Home (self-managed 3.5)\n"), "{index}");
    assert!(!index.contains("older version"), "{index}");
    // An older line's pages point to its own, under its folder.
    let older = read(&build.join("site@3.4/site/install.md"));
    assert!(older.contains("(/docs/3.4/llms.txt)"), "{older}");
    assert!(
        read(&build.join("site@3.4/site/_ascribe/pages.json"))
            .contains("\"markdown\": \"/docs/3.4/install.md\"")
    );
    let index = read(&build.join("site@3.4/plain/llms.txt"));
    assert!(
        index.starts_with(
            "# Home (self-managed 3.4)\n\n> Every page of this documentation, each linked to its \
             Markdown version.\n\nThis is an older version. The current version is 3.5: \
             [llms.txt](https://docs.example.com/docs/llms.txt).\n"
        ),
        "{index}"
    );
    assert!(
        index.contains("(https://docs.example.com/docs/3.4/install.md)"),
        "{index}"
    );
    let markdown = read(&build.join("site@3.4/plain/install.md"));
    assert!(
        markdown.contains("> Documentation for self-managed 3.4. The current version is 3.5: <https://docs.example.com/docs/install/>."),
        "{markdown}"
    );
}

#[test]
fn an_older_line_kept_out_of_an_index_is_out_of_its_llms_txt() {
    let dir = versioned();
    let config = dir.path().join("ascribe.toml");
    let text = read(&config).replace(
        "base-path = \"/docs\"",
        "base-path = \"/docs\"\nagents = true\n\n[listing]\nolder-lines = \"unlisted\"",
    );
    std::fs::write(&config, text).expect("write ascribe.toml");
    let out = ascribe(dir.path(), &["build", "--emit", "site,plain"]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    let build = dir.path().join(".ascribe/build");
    assert!(read(&build.join("site/plain/llms.txt")).contains("install.md)"));
    let index = read(&build.join("site@3.4/plain/llms.txt"));
    assert!(!index.contains("install.md)"), "{index}");
    let headers = read(&build.join("site@3.4/site/_ascribe/headers.json"));
    assert!(headers.contains("noindex"), "{headers}");
}

#[test]
fn agents_index_names_a_line_and_is_the_current_one_without() {
    let dir = versioned();
    let config = dir.path().join("ascribe.toml");
    let text = read(&config).replace(
        "base-path = \"/docs\"",
        "base-path = \"/docs\"\nagents = true",
    );
    std::fs::write(&config, text).expect("write ascribe.toml");
    let out = ascribe(dir.path(), &["agents", "index"]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    let current = String::from_utf8_lossy(&out.stdout).into_owned();
    assert!(current.contains("/docs/install.md"), "{current}");
    assert!(!current.contains("/docs/3.4/"), "{current}");
    let out = ascribe(dir.path(), &["agents", "index", "--build", "site@3.4"]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    let older = String::from_utf8_lossy(&out.stdout).into_owned();
    assert!(older.contains("/docs/3.4/install.md"), "{older}");
    assert!(!older.contains("webhooks"), "{older}");
}

#[test]
fn build_names_one_line_and_refuses_a_line_that_isnt_live() {
    let dir = versioned();
    let out = ascribe(
        dir.path(),
        &["build", "--build", "site@3.4", "--emit", "site"],
    );
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    assert!(dir.path().join(".ascribe/build/site@3.4/site").exists());
    assert!(!dir.path().join(".ascribe/build/site").exists());
    let out = ascribe(dir.path(), &["build", "--build", "site@3.3"]);
    assert_eq!(code(&out), 2);
    assert!(
        stderr(&out).contains("no build `site@3.3`; its builds are site, site@3.4"),
        "{}",
        stderr(&out)
    );
}

#[test]
fn a_page_where_an_older_lines_pages_are_fails_the_build() {
    let model = VERSIONED.replace(r#"["3.4", "3.5"]"#, r#"["3", "4"]"#);
    let dir = project(
        &model,
        &[
            ("index.md", &page("Home.\n")),
            ("3/notes.md", &page("Notes.\n")),
        ],
    );
    let out = ascribe(dir.path(), &["build", "--emit", "site"]);
    assert_eq!(code(&out), 2, "{}", stderr(&out));
    assert!(
        stderr(&out).contains("`3/notes.md` is under `site@3`'s pages"),
        "{}",
        stderr(&out)
    );
}

#[test]
fn a_versioned_build_with_one_live_line_builds_what_it_built_before() {
    let one = VERSIONED.replace(r#"["3.4", "3.5"]"#, r#"["3.5"]"#);
    let plain = one.replace("versions = \"self-managed\"\n", "");
    let files: &[(&str, &str)] = &[
        (
            "index.md",
            "---\ntitle: Home\n---\n\nSee [Install](install.md).\n",
        ),
        ("install.md", "---\ntitle: Install\n---\n\nIn 3.5.\n"),
    ];
    let a = project(&one, files);
    let b = project(&plain, files);
    for dir in [&a, &b] {
        let out = ascribe(dir.path(), &["build"]);
        assert_eq!(code(&out), 0, "{}", stderr(&out));
    }
    let mut left = std::collections::BTreeMap::new();
    let mut right = std::collections::BTreeMap::new();
    files_of(&a.path().join(".ascribe/build/site"), "", &mut left);
    files_of(&b.path().join(".ascribe/build/site"), "", &mut right);
    assert_eq!(left, right);
}

fn files_of(dir: &Path, prefix: &str, into: &mut std::collections::BTreeMap<String, Vec<u8>>) {
    for entry in fs::read_dir(dir).expect("a listing") {
        let entry = entry.expect("an entry");
        let rel = format!("{prefix}{}", entry.file_name().to_string_lossy());
        if entry.file_type().expect("a type").is_dir() {
            files_of(&entry.path(), &format!("{rel}/"), into);
        } else {
            into.insert(rel, fs::read(entry.path()).expect("read a file"));
        }
    }
}

#[test]
fn a_problem_in_every_line_is_said_once() {
    let model = format!(
        "{VERSIONED}\n[builds.cloud]\navailability = {{ filter = \"cloud\" }}\n\n[editor]\nbuild = \"site\"\n"
    );
    let dir = project(
        &model,
        &[
            (
                "index.md",
                "---\ntitle: Home\n---\n\nSee [Setup](setup.md).\n",
            ),
            (
                "setup.md",
                "---\ntitle: Setup\navailable: self-managed\n---\n\nSee [nowhere](index.md#nowhere).\n",
            ),
        ],
    );
    let out = ascribe(dir.path(), &["check"]);
    let text = stdout(&out);
    assert_eq!(text.matches("#nowhere").count(), 1, "{text}");
    assert!(text.contains("(only in every line of `site`)"), "{text}");
    let out = ascribe(dir.path(), &["check", "--format", "json"]);
    let json = stdout(&out);
    assert!(json.contains("\"site\",\n        \"site@3.4\""), "{json}");
}

#[test]
fn versions_json_says_what_a_banner_needs_and_the_lines_that_have_left() {
    let dir = versioned();
    let model = VERSIONED.replace(
        "live = [\"3.4\", \"3.5\"]\n",
        "live = [\"3.4\", \"3.5\"]\n\
         ends = { \"3.4\" = 2027-01-31 }\n\
         archived = { \"3.3\" = { url = \"https://archive.example.com/3.3/\", ended = 2026-06-30 }, \"3.2\" = {} }\n",
    );
    write(&dir.path().join("ascribe.toml"), &model);
    let out = ascribe(dir.path(), &["build", "--emit", "site"]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    let json: serde_json::Value = serde_json::from_str(&read(
        &dir.path().join(".ascribe/build/_ascribe/versions.json"),
    ))
    .expect("JSON");
    let site = &json["builds"]["site"];
    // The target's label, its name when it has none of its own.
    assert_eq!(site["label"], "self-managed");
    let lines: Vec<serde_json::Value> = site["lines"].as_array().expect("lines").clone();
    assert_eq!(lines[0].get("ends"), None);
    assert_eq!(lines[1]["ends"], "2027-01-31");
    // The archived lines come after the live ones, with no output.
    assert_eq!(
        lines[2],
        serde_json::json!({
            "line": "3.3",
            "status": "archived",
            "url": "https://archive.example.com/3.3/",
            "ended": "2026-06-30",
        })
    );
    assert_eq!(
        lines[3],
        serde_json::json!({ "line": "3.2", "status": "archived" })
    );
}

#[test]
fn versions_json_says_which_lines_have_each_page_and_goes_with_releases() {
    let dir = versioned();
    // Building one line writes what every line knows.
    let out = ascribe(
        dir.path(),
        &["build", "--build", "site@3.4", "--emit", "site"],
    );
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    let shared = dir.path().join(".ascribe/build/_ascribe");
    let json: serde_json::Value =
        serde_json::from_str(&read(&shared.join("versions.json"))).expect("JSON");
    let site = &json["builds"]["site"];
    assert_eq!(site["target"], "self-managed");
    assert_eq!(site["lines"][0]["line"], "3.5");
    assert_eq!(site["lines"][0]["status"], "current");
    assert_eq!(site["lines"][0]["build"], "site");
    assert_eq!(site["lines"][0]["base"], "/docs/");
    assert_eq!(site["lines"][1]["build"], "site@3.4");
    assert_eq!(site["lines"][1]["base"], "/docs/3.4/");
    assert_eq!(
        site["pages"]["install.md"]["lines"]["3.4"],
        "/docs/3.4/install/"
    );
    assert_eq!(
        site["pages"]["install.md"]["lines"]["3.5"],
        "/docs/install/"
    );
    assert!(site["pages"]["webhooks.md"]["lines"].get("3.4").is_none());
    assert!(shared.join("manifest.json").exists());

    // Without `[releases]`, the next build removes the file and its folder.
    let model = VERSIONED
        .replace("[releases.self-managed]\nlive = [\"3.4\", \"3.5\"]\n", "")
        .replace("versions = \"self-managed\"\n", "");
    write(&dir.path().join("ascribe.toml"), &model);
    let out = ascribe(dir.path(), &["build", "--emit", "site"]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    assert!(!shared.exists());
}

#[test]
fn an_ascribe_folder_ascribe_didnt_write_is_refused_and_left_alone() {
    let dir = versioned();
    let shared = dir.path().join(".ascribe/build/_ascribe");
    write(&shared.join("notes.txt"), "mine");
    let out = ascribe(dir.path(), &["build", "--emit", "site"]);
    assert_eq!(code(&out), 2, "{}", stderr(&out));
    assert!(
        stderr(&out).contains("isn't a folder Ascribe wrote"),
        "{}",
        stderr(&out)
    );
    assert_eq!(read(&shared.join("notes.txt")), "mine");
    assert!(!shared.join("versions.json").exists());
    // Without `[releases]`, it's not Ascribe's to remove either.
    let model = VERSIONED
        .replace("[releases.self-managed]\nlive = [\"3.4\", \"3.5\"]\n", "")
        .replace("versions = \"self-managed\"\n", "");
    write(&dir.path().join("ascribe.toml"), &model);
    let out = ascribe(dir.path(), &["build", "--emit", "site"]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    assert_eq!(read(&shared.join("notes.txt")), "mine");
}
