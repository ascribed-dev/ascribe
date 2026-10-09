//! `ascribe check`: output, JSON, and exit codes, by running the binary.

#![allow(clippy::expect_used, clippy::panic)]

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use tempfile::TempDir;

const MODEL: &str = "spec = \"0.1\"\n\n[project]\ncontent-root = \"docs\"\n\n[types.page]\ndefault = true\n\n[types.page.frontmatter]\ntitle = \"string\"\n\n[phrases]\nproduct = \"Quill\"\n";

/// A project in a temporary directory: the model, and these `(path, text)`
/// files under `docs/`.
fn project(files: &[(&str, &str)]) -> TempDir {
    let dir = tempfile::tempdir().expect("a temporary directory");
    fs::write(dir.path().join("ascribe.toml"), MODEL).expect("write the model");
    for (path, text) in files {
        write(&dir.path().join("docs").join(path), text);
    }
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

fn code(output: &Output) -> i32 {
    output.status.code().expect("exited with a code")
}

fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

const CLEAN: &str = "---\ntitle: Home\n---\n\nHello {product}.\n";
const WITH_ERROR: &str = "---\ntitle: Home\n---\n\n[Gone](gone.md)\n";
const WITH_WARNING: &str = "---\ntitle: Home\n---\n\nSee {nope}.\n";

#[test]
fn a_clean_project_exits_0() {
    let dir = project(&[("index.md", CLEAN)]);
    let out = ascribe(dir.path(), &["check"]);
    assert_eq!(code(&out), 0, "{}{}", stdout(&out), stderr(&out));
    assert!(stdout(&out).contains("checked 1 file: 0 errors, 0 warnings"));
}

#[test]
fn errors_exit_1_and_show_the_source() {
    let dir = project(&[("index.md", WITH_ERROR)]);
    let out = ascribe(dir.path(), &["check"]);
    assert_eq!(code(&out), 1);
    let text = stdout(&out);
    assert!(text.contains("ASC036"), "{text}");
    assert!(text.contains("`gone.md` doesn't exist"), "{text}");
    assert!(text.contains("docs/index.md:5:"), "{text}");
    assert!(text.contains("[Gone](gone.md)"), "the snippet: {text}");
    assert!(text.contains("1 error"), "{text}");
}

#[test]
fn warnings_exit_0_unless_denied() {
    let dir = project(&[("index.md", WITH_WARNING)]);
    let out = ascribe(dir.path(), &["check"]);
    assert_eq!(code(&out), 0, "{}", stdout(&out));
    assert!(stdout(&out).contains("1 warning"));
    let out = ascribe(dir.path(), &["check", "--deny-warnings"]);
    assert_eq!(code(&out), 1);
}

#[test]
fn usage_errors_exit_2() {
    let dir = project(&[("index.md", CLEAN)]);
    for args in [
        vec!["check", "--no-such-flag"],
        vec!["check", "--format", "xml"],
        vec!["no-such-command"],
        vec![],
    ] {
        let out = ascribe(dir.path(), &args);
        assert_eq!(code(&out), 2, "{args:?}: {}", stderr(&out));
    }
}

#[test]
fn help_and_version_exit_0() {
    let dir = project(&[]);
    let out = ascribe(dir.path(), &["--version"]);
    assert_eq!(code(&out), 0);
    assert!(stdout(&out).starts_with("ascribe "), "{}", stdout(&out));
    let out = ascribe(dir.path(), &["check", "--help"]);
    assert_eq!(code(&out), 0);
    assert!(stdout(&out).contains("--deny-warnings"));
}

#[test]
fn no_content_model_exits_2() {
    let dir = tempfile::tempdir().expect("a temporary directory");
    let out = ascribe(dir.path(), &["check"]);
    assert_eq!(code(&out), 2);
    assert!(stderr(&out).contains("ascribe.toml"), "{}", stderr(&out));
}

#[test]
fn a_content_model_with_errors_exits_2_and_shows_them() {
    let dir = tempfile::tempdir().expect("a temporary directory");
    write(
        &dir.path().join("ascribe.toml"),
        "spec = \"0.1\"\nbogus = 1\n",
    );
    let out = ascribe(dir.path(), &["check"]);
    assert_eq!(code(&out), 2);
    assert!(stdout(&out).contains("ascribe.toml:2"), "{}", stdout(&out));
    assert!(
        stderr(&out).contains("nothing can be checked"),
        "{}",
        stderr(&out)
    );
}

#[test]
fn finds_the_model_in_a_parent_directory() {
    let dir = project(&[("guides/index.md", WITH_ERROR)]);
    let out = ascribe(&dir.path().join("docs/guides"), &["check"]);
    assert_eq!(code(&out), 1);
    assert!(
        stdout(&out).contains("docs/guides/index.md"),
        "{}",
        stdout(&out)
    );
}

#[test]
fn config_names_the_model() {
    let dir = project(&[("index.md", CLEAN)]);
    let elsewhere = tempfile::tempdir().expect("a temporary directory");
    let config: PathBuf = dir.path().join("ascribe.toml");
    let out = ascribe(
        elsewhere.path(),
        &["check", "--config", config.to_str().expect("utf-8")],
    );
    assert_eq!(code(&out), 0, "{}{}", stdout(&out), stderr(&out));
    let out = ascribe(
        elsewhere.path(),
        &["--config", dir.path().to_str().expect("utf-8"), "check"],
    );
    assert_eq!(code(&out), 0, "a directory names the model in it");
    let out = ascribe(elsewhere.path(), &["check", "--config", "missing.toml"]);
    assert_eq!(code(&out), 2);
}

#[test]
fn a_project_nested_in_the_content_root_is_left_to_its_own_check() {
    let dir = project(&[
        ("index.md", CLEAN),
        ("nested/ascribe.toml", MODEL),
        ("nested/docs/bad.md", WITH_ERROR),
    ]);
    let out = ascribe(dir.path(), &["check"]);
    assert_eq!(code(&out), 0, "{}{}", stdout(&out), stderr(&out));
    assert!(
        stdout(&out).contains("checked 1 file: 0 errors, 0 warnings"),
        "{}",
        stdout(&out)
    );
    // The nested project checks it.
    let nested = ascribe(&dir.path().join("docs").join("nested"), &["check"]);
    assert_eq!(code(&nested), 1, "{}", stdout(&nested));
    assert!(stdout(&nested).contains("ASC036"), "{}", stdout(&nested));
}

#[test]
fn a_content_root_above_the_project_keeps_the_projects_own_folder() {
    // The project is `ws/proj`, and its content root is `ws`.
    let dir = tempfile::tempdir().expect("a temporary directory");
    let ws = dir.path().join("ws");
    let proj = ws.join("proj");
    write(
        &proj.join("ascribe.toml"),
        &MODEL.replace(
            "content-root = \"docs\"",
            "content-root = \"..\"\noutput-dir = \"../../out\"",
        ),
    );
    write(&proj.join("own.md"), WITH_ERROR);
    write(&ws.join("index.md"), CLEAN);
    write(&ws.join("other").join("ascribe.toml"), MODEL);
    write(&ws.join("other").join("docs").join("bad.md"), WITH_ERROR);
    let out = ascribe(&proj, &["check"]);
    assert_eq!(code(&out), 1, "{}{}", stdout(&out), stderr(&out));
    let text = stdout(&out);
    assert!(text.contains("checked 2 files: 1 error"), "{text}");
    assert!(text.contains("own.md"), "{text}");
    assert!(!text.contains("bad.md"), "{text}");
}

#[test]
fn a_source_that_isnt_utf8_is_an_error_and_the_rest_is_checked() {
    // SPEC §8.2.
    let dir = project(&[("index.md", WITH_ERROR)]);
    fs::write(dir.path().join("docs/bad.md"), [0xff, 0xfe, 0x00]).expect("write bytes");
    let out = ascribe(dir.path(), &["check", "--format", "json"]);
    assert_eq!(code(&out), 1);
    let report: serde_json::Value = serde_json::from_str(&stdout(&out)).expect("valid JSON");
    let slugs: Vec<&str> = report["diagnostics"]
        .as_array()
        .expect("a list")
        .iter()
        .filter_map(|d| d["slug"].as_str())
        .collect();
    assert!(slugs.contains(&"source-unreadable"), "{slugs:?}");
    assert!(slugs.len() >= 2, "index.md is still checked: {slugs:?}");
}

#[test]
fn json_output_follows_the_documented_schema() {
    let dir = project(&[("index.md", WITH_ERROR), ("other.md", WITH_WARNING)]);
    let out = ascribe(dir.path(), &["check", "--format", "json"]);
    assert_eq!(code(&out), 1);
    let report: serde_json::Value = serde_json::from_str(&stdout(&out)).expect("valid JSON");
    assert_eq!(report["schema_version"], 1);
    assert!(report["ascribe_version"].is_string());
    assert!(report["error"].is_null());
    assert_eq!(report["files_checked"], 2);
    assert_eq!(report["summary"]["errors"], 1);
    assert_eq!(report["summary"]["warnings"], 1);
    let diagnostics = report["diagnostics"].as_array().expect("an array");
    assert_eq!(diagnostics.len(), 2);
    let d = diagnostics
        .iter()
        .find(|d| d["slug"] == "link-target-missing")
        .expect("the missing link");
    assert_eq!(d["code"], "ASC036");
    assert_eq!(d["severity"], "error");
    assert_eq!(d["file"], "docs/index.md");
    assert_eq!(d["message"], "`gone.md` doesn't exist");
    assert_eq!(d["range"]["start"]["line"], 5);
    assert_eq!(d["range"]["start"]["column"], 8);
    assert_eq!(d["range"]["end"]["column"], 15);
    assert!(d["range"]["start"]["offset"].is_u64());
    assert!(d["related"].as_array().expect("an array").is_empty());
    assert!(d["fixes"].as_array().expect("an array").is_empty());
}

#[test]
fn json_output_carries_fixes_as_edits() {
    let dir = project(&[
        ("index.md", "---\ntitle: Home\n---\n\n[Keys](/keys/)\n"),
        ("keys.md", "---\ntitle: Keys\n---\n"),
    ]);
    let out = ascribe(dir.path(), &["check", "--format", "json"]);
    assert_eq!(code(&out), 0, "a route is a warning");
    let report: serde_json::Value = serde_json::from_str(&stdout(&out)).expect("valid JSON");
    let fix = &report["diagnostics"][0]["fixes"][0];
    assert_eq!(fix["file"], "docs/index.md");
    assert_eq!(fix["edits"][0]["new_text"], "/keys.md");
    assert_eq!(fix["edits"][0]["range"]["start"]["column"], 8);
}

#[test]
fn json_reports_a_project_that_did_not_load() {
    let dir = tempfile::tempdir().expect("a temporary directory");
    write(
        &dir.path().join("ascribe.toml"),
        "spec = \"0.1\"\nbogus = 1\n",
    );
    let out = ascribe(dir.path(), &["check", "--format", "json"]);
    assert_eq!(code(&out), 2);
    let report: serde_json::Value = serde_json::from_str(&stdout(&out)).expect("valid JSON");
    assert!(report["error"].is_string());
    assert_eq!(report["diagnostics"][0]["file"], "ascribe.toml");
    assert_eq!(report["files_checked"], 0);
}

#[test]
fn json_reports_a_missing_model_as_an_error_object() {
    let dir = tempfile::tempdir().expect("a temporary directory");
    let out = ascribe(dir.path(), &["check", "--format", "json"]);
    assert_eq!(code(&out), 2);
    let report: serde_json::Value = serde_json::from_str(&stdout(&out)).expect("valid JSON");
    assert!(
        report["error"]
            .as_str()
            .expect("a message")
            .contains("ascribe.toml")
    );
    assert!(
        report["diagnostics"]
            .as_array()
            .expect("an array")
            .is_empty()
    );
}

#[test]
fn the_quill_example_has_no_errors() {
    let example = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/quill");
    let out = ascribe(&example, &["check", "--deny-warnings"]);
    assert_eq!(code(&out), 0, "{}{}", stdout(&out), stderr(&out));
}

/// The projects of `examples/monorepo`, one nested in another's content root.
const MONOREPO_PROJECTS: [&str; 3] = ["docs", "handbook", "handbook/pages/security"];

#[test]
fn the_monorepo_examples_projects_have_no_problems_and_are_canonical() {
    let example = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/monorepo");
    for project in MONOREPO_PROJECTS {
        let dir = project
            .split('/')
            .fold(example.clone(), |dir, part| dir.join(part));
        for args in [&["check", "--deny-warnings"][..], &["fmt", "--check"][..]] {
            let out = ascribe(&dir, args);
            assert_eq!(
                code(&out),
                0,
                "{project}: ascribe {}\n{}{}",
                args.join(" "),
                stdout(&out),
                stderr(&out)
            );
        }
    }
}

// ---------------------------------------------------------------------------
// Page-level checks and builds

const BUILDS_MODEL: &str = "spec = \"0.1\"\n\n[project]\ncontent-root = \"docs\"\n\n[types.page]\ndefault = true\n\n[types.page.frontmatter]\ntitle = \"string\"\n\n[dimensions.deployment]\nvalues = [\"cloud\", \"self-managed\"]\nversionless = [\"cloud\", \"self-managed\"]\n\n[builds.site]\nvariants = \"switch\"\navailability = \"badge\"\n\n[builds.cloud]\nvariants = { deployment = \"cloud\" }\navailability = \"badge\"\n";

fn builds_project(files: &[(&str, &str)]) -> TempDir {
    let dir = project(files);
    fs::write(dir.path().join("ascribe.toml"), BUILDS_MODEL).expect("write the model");
    dir
}

const DUPLICATE: &str = "---\ntitle: Home\n---\n\n## A\n@id: same\n\n## B\n@id: same\n";
const ONLY_SELF_MANAGED: &str =
    "---\ntitle: Home\n---\n\n@variant {deployment=self-managed}:\nSelf-managed only.\n@end\n";

#[test]
fn a_page_level_error_fails_the_check() {
    let dir = builds_project(&[("index.md", DUPLICATE)]);
    let out = ascribe(dir.path(), &["check"]);
    assert_eq!(code(&out), 1, "{}{}", stdout(&out), stderr(&out));
    assert!(stdout(&out).contains("id-duplicate"), "{}", stdout(&out));
    // Both builds have it, and it's reported once.
    assert_eq!(
        stdout(&out).matches("id-duplicate").count(),
        1,
        "{}",
        stdout(&out)
    );
    assert!(stdout(&out).contains("1 error"), "{}", stdout(&out));
}

#[test]
fn a_problem_in_some_builds_names_them() {
    let dir = builds_project(&[("index.md", ONLY_SELF_MANAGED)]);
    let out = ascribe(dir.path(), &["check"]);
    assert_eq!(code(&out), 0, "a warning: {}{}", stdout(&out), stderr(&out));
    assert!(
        stdout(&out).contains("variant-no-arm-survives"),
        "{}",
        stdout(&out)
    );
    assert!(stdout(&out).contains("build `cloud`"), "{}", stdout(&out));
    assert!(stdout(&out).contains("1 warning"), "{}", stdout(&out));
}

#[test]
fn build_checks_one_build_only() {
    let dir = builds_project(&[("index.md", ONLY_SELF_MANAGED)]);
    let site = ascribe(dir.path(), &["check", "--build", "site"]);
    assert_eq!(code(&site), 0, "{}{}", stdout(&site), stderr(&site));
    assert!(stdout(&site).contains("0 warnings"), "{}", stdout(&site));
    let cloud = ascribe(
        dir.path(),
        &["check", "--build", "cloud", "--deny-warnings"],
    );
    assert_eq!(code(&cloud), 1, "{}{}", stdout(&cloud), stderr(&cloud));
    assert!(stdout(&cloud).contains("variant-no-arm-survives"));
}

#[test]
fn an_unknown_build_is_a_failure_that_lists_the_builds() {
    let dir = builds_project(&[("index.md", DUPLICATE)]);
    let out = ascribe(dir.path(), &["check", "--build", "nope"]);
    assert_eq!(code(&out), 2, "{}{}", stdout(&out), stderr(&out));
    assert!(stderr(&out).contains("no build `nope`"), "{}", stderr(&out));
    assert!(stderr(&out).contains("site, cloud"), "{}", stderr(&out));
}

#[test]
fn a_problem_in_a_fragment_is_at_the_include_site() {
    let dir = builds_project(&[
        ("_f.md", "See [it](index.md#gone).\n"),
        ("index.md", "---\ntitle: Home\n---\n\n@include: _f.md\n"),
    ]);
    let out = ascribe(dir.path(), &["check", "--format", "json"]);
    assert_eq!(code(&out), 1, "{}{}", stdout(&out), stderr(&out));
    let json: serde_json::Value = serde_json::from_str(&stdout(&out)).expect("JSON");
    let diagnostics = json["diagnostics"].as_array().expect("a list");
    assert_eq!(diagnostics.len(), 1, "{diagnostics:#?}");
    let d = &diagnostics[0];
    assert_eq!(d["slug"], "link-id-missing");
    assert_eq!(d["file"], "docs/index.md");
    assert_eq!(d["range"]["start"]["line"], 5);
    assert_eq!(d["related"][0]["file"], "docs/_f.md");
    // Both builds have it, so it names both; nothing about it is unpublished.
    assert_eq!(d["builds"], serde_json::json!(["site", "cloud"]));
    assert_eq!(d["unpublished"], false);
}

#[test]
fn json_names_the_builds_of_a_problem_and_leaves_file_level_ones_empty() {
    let dir = builds_project(&[
        ("index.md", ONLY_SELF_MANAGED),
        ("other.md", "---\ntitle: Other\n---\n\n[Gone](gone.md)\n"),
    ]);
    let out = ascribe(dir.path(), &["check", "--format", "json"]);
    let json: serde_json::Value = serde_json::from_str(&stdout(&out)).expect("JSON");
    let by_slug = |slug: &str| {
        json["diagnostics"]
            .as_array()
            .expect("a list")
            .iter()
            .find(|d| d["slug"] == slug)
            .unwrap_or_else(|| panic!("no {slug}: {json}"))
            .clone()
    };
    // In some builds only.
    let variant = by_slug("variant-no-arm-survives");
    assert_eq!(variant["builds"], serde_json::json!(["cloud"]));
    assert_eq!(variant["unpublished"], false);
    // File-level: no build.
    let missing = by_slug("link-target-missing");
    assert_eq!(missing["builds"], serde_json::json!([]));
    assert_eq!(missing["unpublished"], false);
}

// ---------------------------------------------------------------------------
// Checking some files: paths, standard input, the editor's build

/// A fragment with a broken link, included by three pages; `b.md` also has a
/// warning of its own, and `index.md` an error of its own.
fn paths_project() -> TempDir {
    project(&[
        ("_f.md", "See [it](index.md#gone).\n"),
        (
            "index.md",
            "---\ntitle: Home\n---\n\n@include: _f.md\n\n[Gone](gone.md)\n",
        ),
        ("guides/a.md", "---\ntitle: A\n---\n\n@include: ../_f.md\n"),
        (
            "guides/b.md",
            "---\ntitle: B\n---\n\n@include: ../_f.md\n\nSee {nope}.\n",
        ),
    ])
}

fn json_of(output: &Output) -> serde_json::Value {
    serde_json::from_str(&stdout(output))
        .unwrap_or_else(|e| panic!("{e}: {}{}", stdout(output), stderr(output)))
}

fn diagnostics_of(report: &serde_json::Value) -> Vec<serde_json::Value> {
    report["diagnostics"].as_array().expect("a list").clone()
}

/// `(file, slug)` of each diagnostic, in order.
fn places(report: &serde_json::Value) -> Vec<(String, String)> {
    diagnostics_of(report)
        .iter()
        .map(|d| {
            (
                d["file"].as_str().expect("a file").to_owned(),
                d["slug"].as_str().expect("a slug").to_owned(),
            )
        })
        .collect()
}

fn place(file: &str, slug: &str) -> (String, String) {
    (file.to_owned(), slug.to_owned())
}

/// The diagnostics of a whole-project report that count for `paths`: those
/// whose file, or a related place's, is one of them or under one.
fn counting_for(report: &serde_json::Value, paths: &[&str]) -> Vec<serde_json::Value> {
    let under = |file: &serde_json::Value| {
        let file = file.as_str().expect("a file");
        paths
            .iter()
            .any(|p| file == *p || file.starts_with(&format!("{p}/")))
    };
    diagnostics_of(report)
        .into_iter()
        .filter(|d| {
            under(&d["file"])
                || d["related"]
                    .as_array()
                    .expect("a list")
                    .iter()
                    .any(|r| under(&r["file"]))
        })
        .collect()
}

#[test]
fn a_page_shows_its_own_problems_and_those_its_fragments_cause_on_it() {
    let dir = paths_project();
    let out = ascribe(
        dir.path(),
        &["check", "docs/guides/b.md", "--format", "json"],
    );
    assert_eq!(code(&out), 1, "{}{}", stdout(&out), stderr(&out));
    let report = json_of(&out);
    assert_eq!(
        places(&report),
        [
            place("docs/guides/b.md", "phrase-undeclared"),
            place("docs/guides/b.md", "link-id-missing"),
        ]
    );
    let broken = &diagnostics_of(&report)[1];
    assert_eq!(broken["related"][0]["file"], "docs/_f.md");
    assert_eq!(broken["repeats"], 0);
    assert_eq!(report["files_checked"], 4);
    assert_eq!(report["files_reported"], 1);
    assert_eq!(report["summary"]["errors"], 1);
    assert_eq!(report["summary"]["warnings"], 1);
}

#[test]
fn the_diagnostics_are_the_whole_checks_that_count_for_the_paths_in_its_order() {
    let dir = paths_project();
    let whole = json_of(&ascribe(dir.path(), &["check", "--format", "json"]));
    for paths in [
        &["docs/guides/b.md"][..],
        &["docs/guides"],
        &["docs/index.md", "docs/guides/a.md"],
        &["docs"],
        &["."],
    ] {
        let mut args = vec!["check"];
        args.extend(paths);
        args.extend(["--format", "json"]);
        let report = json_of(&ascribe(dir.path(), &args));
        let wanted: Vec<&str> = paths
            .iter()
            .map(|p| if *p == "." { "" } else { p })
            .collect();
        let expected = if wanted == [""] {
            diagnostics_of(&whole)
        } else {
            counting_for(&whole, &wanted)
        };
        assert_eq!(diagnostics_of(&report), expected, "{paths:?}");
    }
}

#[test]
fn a_directory_and_two_files_report_on_every_file_in_them() {
    let dir = paths_project();
    let report = json_of(&ascribe(
        dir.path(),
        &["check", "docs/guides", "--format", "json"],
    ));
    assert_eq!(
        places(&report),
        [
            place("docs/guides/b.md", "phrase-undeclared"),
            place("docs/guides/a.md", "link-id-missing"),
            place("docs/guides/b.md", "link-id-missing"),
        ]
    );
    assert_eq!(report["files_reported"], 2);

    let report = json_of(&ascribe(
        dir.path(),
        &[
            "check",
            "docs/index.md",
            "docs/guides/a.md",
            "--format",
            "json",
        ],
    ));
    assert_eq!(
        places(&report),
        [
            place("docs/index.md", "link-target-missing"),
            place("docs/guides/a.md", "link-id-missing"),
            place("docs/index.md", "link-id-missing"),
        ]
    );
    assert_eq!(report["files_reported"], 2);
}

#[test]
fn a_problem_in_a_fragment_is_shown_once_with_how_many_more_includes_have_it() {
    let dir = paths_project();
    let out = ascribe(dir.path(), &["check", "docs/_f.md", "--format", "json"]);
    assert_eq!(code(&out), 1, "{}{}", stdout(&out), stderr(&out));
    let report = json_of(&out);
    assert_eq!(
        places(&report),
        [place("docs/guides/a.md", "link-id-missing")]
    );
    let d = &diagnostics_of(&report)[0];
    assert_eq!(d["repeats"], 2, "at b.md and index.md too");
    assert_eq!(d["related"][0]["file"], "docs/_f.md");

    let text = stdout(&ascribe(dir.path(), &["check", "docs/_f.md"]));
    assert!(text.contains("(also at 2 other includes)"), "{text}");
    assert!(
        text.contains("checked 4 files, reported on 1: 1 error, 0 warnings"),
        "{text}"
    );
}

#[test]
fn a_path_that_does_not_exist_or_is_in_no_project_exits_2_naming_it() {
    let dir = paths_project();
    let out = ascribe(dir.path(), &["check", "docs/nope.md"]);
    assert_eq!(code(&out), 2);
    assert!(
        stderr(&out).contains("docs/nope.md doesn't exist"),
        "{}",
        stderr(&out)
    );

    let elsewhere = tempfile::tempdir().expect("a temporary directory");
    write(&elsewhere.path().join("page.md"), CLEAN);
    let page = elsewhere.path().join("page.md");
    let page = page.to_str().expect("utf-8");
    let out = ascribe(elsewhere.path(), &["check", page]);
    assert_eq!(code(&out), 2);
    assert!(
        stderr(&out).contains("isn't in an Ascribe project"),
        "{}",
        stderr(&out)
    );
    let out = ascribe(
        dir.path(),
        &["check", "docs/index.md", page, "--format", "json"],
    );
    assert_eq!(code(&out), 2);
    let report = json_of(&out);
    assert!(
        report["error"]
            .as_str()
            .expect("a message")
            .contains("isn't in the project"),
        "{report:#}"
    );
}

#[test]
fn a_path_from_the_repository_root_reports_what_the_projects_own_path_does() {
    let repo = tempfile::tempdir().expect("a temporary directory");
    let docs = repo.path().join("docs");
    write(
        &docs.join("ascribe.toml"),
        &MODEL.replace(
            "content-root = \"docs\"",
            "content-root = \".\"\noutput-dir = \"../out\"",
        ),
    );
    write(&docs.join("guides/install.md"), WITH_ERROR);
    write(&docs.join("index.md"), WITH_WARNING);
    let from_root = ascribe(
        repo.path(),
        &["check", "docs/guides/install.md", "--format", "json"],
    );
    let from_docs = ascribe(&docs, &["check", "guides/install.md", "--format", "json"]);
    assert_eq!(
        code(&from_root),
        1,
        "{}{}",
        stdout(&from_root),
        stderr(&from_root)
    );
    assert_eq!(stdout(&from_root), stdout(&from_docs));
    let report = json_of(&from_root);
    assert_eq!(
        places(&report),
        [place("guides/install.md", "link-target-missing")]
    );
    let d = &diagnostics_of(&report)[0];
    assert!(d["help"].as_str().is_some_and(|h| !h.is_empty()), "{d:#}");
    assert_eq!(
        d["docs"],
        "https://ascribed-dev.com/reference/diagnostics/#asc036-link-target-missing"
    );
    assert!(d["fixes"].is_array());
}

#[test]
fn paths_in_two_projects_exit_2() {
    let dir = project(&[
        ("index.md", CLEAN),
        ("nested/ascribe.toml", MODEL),
        ("nested/docs/bad.md", WITH_ERROR),
    ]);
    let out = ascribe(
        dir.path(),
        &["check", "docs/index.md", "docs/nested/docs/bad.md"],
    );
    assert_eq!(code(&out), 2, "{}{}", stdout(&out), stderr(&out));
    assert!(
        stderr(&out).contains("check each project on its own"),
        "{}",
        stderr(&out)
    );
    // Its own project checks it.
    let out = ascribe(dir.path(), &["check", "docs/nested/docs/bad.md"]);
    assert_eq!(code(&out), 1, "{}{}", stdout(&out), stderr(&out));
}

/// Runs `ascribe` with `input` on standard input.
fn ascribe_with_input(dir: &Path, args: &[&str], input: &str) -> Output {
    use std::io::Write as _;
    let mut child = Command::new(env!("CARGO_BIN_EXE_ascribe"))
        .current_dir(dir)
        .args(args)
        .env("NO_COLOR", "1")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("run ascribe");
    // A usage error exits before reading it.
    let _ = child
        .stdin
        .take()
        .expect("standard input")
        .write_all(input.as_bytes());
    child.wait_with_output().expect("wait for ascribe")
}

#[test]
fn standard_input_is_checked_as_a_new_file_without_writing_it() {
    let dir = paths_project();
    let out = ascribe_with_input(
        dir.path(),
        &[
            "check",
            "--stdin",
            "--path",
            "docs/guides/new.md",
            "--format",
            "json",
        ],
        "---
title: New
---

[Home](../index.md) and [gone](../gone.md).
",
    );
    assert_eq!(code(&out), 1, "{}{}", stdout(&out), stderr(&out));
    let report = json_of(&out);
    assert_eq!(
        places(&report),
        [place("docs/guides/new.md", "link-target-missing")]
    );
    assert_eq!(report["files_checked"], 5);
    assert_eq!(report["files_reported"], 1);
    assert!(!dir.path().join("docs/guides/new.md").exists());
}

#[test]
fn standard_input_replaces_a_files_text_on_disk_without_changing_it() {
    let dir = paths_project();
    let page = dir.path().join("docs/index.md");
    let before = fs::read_to_string(&page).expect("read");
    let out = ascribe_with_input(
        dir.path(),
        &[
            "check",
            "--stdin",
            "--path",
            "docs/index.md",
            "--format",
            "json",
        ],
        CLEAN,
    );
    assert_eq!(code(&out), 0, "{}{}", stdout(&out), stderr(&out));
    assert!(places(&json_of(&out)).is_empty());
    assert_eq!(fs::read_to_string(&page).expect("read"), before);
}

#[test]
fn standard_input_shows_only_its_files_problems_not_those_it_causes_elsewhere() {
    let dir = project(&[
        (
            "index.md",
            "---
title: Home
---

## Keep
@id: keep
",
        ),
        (
            "other.md",
            "---
title: Other
---

[Keep](index.md#keep)
",
        ),
    ]);
    let out = ascribe_with_input(
        dir.path(),
        &[
            "check",
            "--stdin",
            "--path",
            "docs/index.md",
            "--format",
            "json",
        ],
        CLEAN,
    );
    assert_eq!(code(&out), 0, "{}{}", stdout(&out), stderr(&out));
    assert!(places(&json_of(&out)).is_empty());
}

#[test]
fn standard_input_needs_one_path_to_a_source_file() {
    let dir = paths_project();
    for args in [
        &["check", "--stdin"][..],
        &["check", "--path", "docs/index.md"],
        &[
            "check",
            "--stdin",
            "--path",
            "docs/index.md",
            "docs/guides/a.md",
        ],
    ] {
        let out = ascribe_with_input(dir.path(), args, CLEAN);
        assert_eq!(code(&out), 2, "{args:?}: {}{}", stdout(&out), stderr(&out));
    }
    let out = ascribe_with_input(
        dir.path(),
        &["check", "--stdin", "--path", "notes.md"],
        CLEAN,
    );
    assert_eq!(code(&out), 2, "{}{}", stdout(&out), stderr(&out));
    assert!(
        stderr(&out).contains("can't be a source file"),
        "{}",
        stderr(&out)
    );
}

#[test]
fn the_editor_build_checks_one_build_and_says_which() {
    let dir = builds_project(&[("index.md", ONLY_SELF_MANAGED)]);
    let whole = json_of(&ascribe(dir.path(), &["check", "--format", "json"]));
    assert_eq!(
        whole["builds_checked"],
        serde_json::json!(["site", "cloud"])
    );
    assert_eq!(
        places(&whole),
        [place("docs/index.md", "variant-no-arm-survives")]
    );

    let out = ascribe(dir.path(), &["check", "--editor-build", "--format", "json"]);
    assert_eq!(code(&out), 0, "{}{}", stdout(&out), stderr(&out));
    let report = json_of(&out);
    assert_eq!(report["builds_checked"], serde_json::json!(["site"]));
    assert!(places(&report).is_empty(), "the site build keeps every arm");

    let text = stdout(&ascribe(dir.path(), &["check", "--editor-build"]));
    assert!(
        text.contains("page-level checks of build `site` only"),
        "{text}"
    );

    let out = ascribe(dir.path(), &["check", "--editor-build", "--build", "cloud"]);
    assert_eq!(code(&out), 2, "{}{}", stdout(&out), stderr(&out));
}

#[test]
fn the_editor_build_for_some_files_reports_what_it_reports_for_the_project() {
    let dir = paths_project();
    let whole = json_of(&ascribe(
        dir.path(),
        &["check", "--editor-build", "--format", "json"],
    ));
    for path in ["docs/guides/b.md", "docs/_f.md", "docs/index.md"] {
        let report = json_of(&ascribe(
            dir.path(),
            &["check", path, "--editor-build", "--format", "json"],
        ));
        let mut expected = counting_for(&whole, &[path]);
        let mut got = diagnostics_of(&report);
        // A fragment's problem is collapsed into its first include.
        if path == "docs/_f.md" {
            expected.truncate(1);
            got[0]["repeats"] = serde_json::json!(0);
        }
        assert_eq!(got, expected, "{path}");
    }
}

#[test]
fn concise_output_is_a_line_per_diagnostic_grouped_by_file_and_cut_at_50() {
    let dir = paths_project();
    let out = ascribe(dir.path(), &["check", "--format", "concise"]);
    assert_eq!(code(&out), 1);
    assert_eq!(
        stdout(&out),
        concat!(
            "docs/guides/a.md:5: [ASC037] `index.md` has no heading with the id `gone`\n",
            "docs/guides/b.md:5: [ASC037] `index.md` has no heading with the id `gone`\n",
            "docs/guides/b.md:7: [ASC044] `{nope}` isn't a declared phrase, so its braces are literal text; declare it in [phrases], or write `\\{` to keep it literal\n",
            "docs/index.md:5: [ASC037] `index.md` has no heading with the id `gone`\n",
            "docs/index.md:7: [ASC036] `gone.md` doesn't exist\n",
            "checked 4 files: 4 errors, 1 warning\n",
        )
    );

    let many: String = (0..30)
        .map(|i| {
            format!(
                "See {{nope{i}}}.

"
            )
        })
        .collect();
    let page = format!(
        "---
title: Many
---

{many}"
    );
    let dir = project(&[("a.md", &page), ("b.md", &page)]);
    let out = ascribe(dir.path(), &["check", "--format", "concise"]);
    assert_eq!(code(&out), 0, "{}{}", stdout(&out), stderr(&out));
    let text = stdout(&out);
    let lines: Vec<&str> = text.lines().collect();
    assert_eq!(lines.len(), 52, "{text}");
    assert!(lines[49].starts_with("docs/b.md:"), "{text}");
    assert_eq!(
        lines[50],
        "and 10 more: ascribe check docs/b.md --format concise"
    );
    assert_eq!(lines[51], "checked 2 files: 0 errors, 60 warnings");
    // Checking one file, the rest of it is in the JSON.
    let dir = project(&[("a.md", &format!("{page}{many}"))]);
    let text = stdout(&ascribe(
        dir.path(),
        &["check", "docs/a.md", "--format", "concise"],
    ));
    assert!(
        text.contains("\nand 10 more: ascribe check docs/a.md --format json\n"),
        "{text}"
    );
}

#[test]
fn summary_counts_by_code_and_by_file_instead_of_listing() {
    let dir = paths_project();
    let text = stdout(&ascribe(dir.path(), &["check", "--summary"]));
    assert_eq!(
        text,
        concat!(
            "By code:\n",
            "  3  ASC037 link-id-missing (error)\n",
            "  1  ASC044 phrase-undeclared (warning)\n",
            "  1  ASC036 link-target-missing (error)\n",
            "By file:\n",
            "  2  docs/guides/b.md (1 error, 1 warning)\n",
            "  2  docs/index.md (2 errors, 0 warnings)\n",
            "  1  docs/guides/a.md (1 error, 0 warnings)\n",
            "checked 4 files: 4 errors, 1 warning\n",
        )
    );

    let report = json_of(&ascribe(
        dir.path(),
        &["check", "docs/guides", "--summary", "--format", "json"],
    ));
    assert!(places(&report).is_empty());
    assert_eq!(report["truncated"], true);
    assert_eq!(report["shown"], 0);
    assert_eq!(report["total"], 3);
    assert_eq!(
        report["next_command"],
        "ascribe check docs/guides --format json"
    );
    assert_eq!(report["summary"]["by_code"][0]["code"], "ASC037");
    assert_eq!(report["summary"]["by_code"][0]["count"], 2);
    assert_eq!(report["summary"]["by_file"][0]["file"], "docs/guides/b.md");
    assert_eq!(report["summary"]["by_file"][0]["warnings"], 1);

    let listed = json_of(&ascribe(dir.path(), &["check", "--format", "json"]));
    assert_eq!(listed["truncated"], false);
    assert_eq!(listed["shown"], 5);
    assert_eq!(listed["total"], 5);
    assert!(listed["next_command"].is_null());
    assert!(listed["summary"].get("by_code").is_none());
}

#[test]
fn every_fix_says_whether_it_is_safe() {
    let dir = project(&[
        (
            "index.md",
            "---
title: Home
---

[Keys](/keys/)

@notte: Hi.
",
        ),
        (
            "keys.md",
            "---
title: Keys
---
",
        ),
    ]);
    let report = json_of(&ascribe(dir.path(), &["check", "--format", "json"]));
    let fixes: Vec<(String, String)> = diagnostics_of(&report)
        .iter()
        .flat_map(|d| {
            let slug = d["slug"].as_str().expect("a slug").to_owned();
            d["fixes"]
                .as_array()
                .expect("a list")
                .iter()
                .map(move |f| {
                    (
                        slug.clone(),
                        f["applicability"].as_str().expect("a label").to_owned(),
                    )
                })
                .collect::<Vec<_>>()
        })
        .collect();
    assert_eq!(
        fixes,
        [
            ("link-route".to_owned(), "safe".to_owned()),
            ("directive-unknown".to_owned(), "unsafe".to_owned()),
        ]
    );
}

#[test]
fn prompt_format_writes_a_files_prompt_and_keeps_the_exit_code() {
    let dir = project(&[("index.md", WITH_ERROR), ("other.md", WITH_WARNING)]);
    let out = ascribe(
        dir.path(),
        &["check", "docs/index.md", "--format", "prompt"],
    );
    assert_eq!(code(&out), 1, "{}", stderr(&out));
    let text = stdout(&out);
    assert!(
        text.starts_with(
            "Fix the problem `ascribe check` reports in `docs/index.md`.\n\nWhere: docs/index.md\n"
        ),
        "{text}"
    );
    assert!(
        text.contains("- 5: [ASC036] `gone.md` doesn't exist\n"),
        "{text}"
    );
    assert!(
        text.ends_with(
            "When you're done, run `ascribe check docs/index.md` and fix what it reports.\n"
        ),
        "{text}"
    );
}

#[test]
fn prompt_format_without_paths_writes_the_projects_prompt() {
    let dir = project(&[("index.md", WITH_ERROR), ("other.md", WITH_WARNING)]);
    let out = ascribe(dir.path(), &["check", "--format", "prompt"]);
    assert_eq!(code(&out), 1);
    let text = stdout(&out);
    assert!(
        text.starts_with("Fix the 2 problems `ascribe check` reports in this project.\n"),
        "{text}"
    );
    assert!(
        text.contains("- docs/index.md: 1 error\n- docs/other.md: 1 warning\n"),
        "{text}"
    );
}

#[test]
fn prompt_format_writes_nothing_without_problems() {
    let dir = project(&[("index.md", CLEAN)]);
    let out = ascribe(dir.path(), &["check", "--format", "prompt"]);
    assert_eq!(code(&out), 0);
    assert_eq!(stdout(&out), "");
}

#[test]
fn prompt_format_on_standard_input_says_to_save_the_file() {
    use std::io::Write as _;
    use std::process::Stdio;
    let dir = project(&[("index.md", CLEAN)]);
    let mut child = Command::new(env!("CARGO_BIN_EXE_ascribe"))
        .current_dir(dir.path())
        .args([
            "check",
            "--stdin",
            "--path",
            "docs/index.md",
            "--format",
            "prompt",
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("run ascribe");
    child
        .stdin
        .take()
        .expect("stdin")
        .write_all(WITH_ERROR.as_bytes())
        .expect("write stdin");
    let out = child.wait_with_output().expect("ascribe ends");
    assert_eq!(code(&out), 1);
    assert!(
        stdout(&out).contains(
            "Where: docs/index.md\nThe file has unsaved changes; save it before you start.\n"
        ),
        "{}",
        stdout(&out)
    );
}

#[test]
fn prompt_format_in_a_repository_names_the_projects_folder_and_agents_md() {
    let repo = tempfile::tempdir().expect("a temporary directory");
    fs::create_dir(repo.path().join(".git")).expect("a .git folder");
    write(&repo.path().join("AGENTS.md"), "# Rules\n");
    write(&repo.path().join("site/ascribe.toml"), MODEL);
    write(&repo.path().join("site/docs/index.md"), WITH_ERROR);
    let out = ascribe(
        repo.path(),
        &["check", "site/docs/index.md", "--format", "prompt"],
    );
    let text = stdout(&out);
    assert!(
        text.contains("Where: docs/index.md\nProject: site/\n"),
        "{text}"
    );
    assert!(
        text.ends_with(
            "Follow the project's rules in `AGENTS.md`.\nWhen you're done, run `ascribe check site/docs/index.md` and fix what it reports.\n"
        ),
        "{text}"
    );
}
