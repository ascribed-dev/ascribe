//! `tessera check`: output, JSON, and exit codes, by running the binary.

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
    fs::write(dir.path().join("tessera.toml"), MODEL).expect("write the model");
    for (path, text) in files {
        write(&dir.path().join("docs").join(path), text);
    }
    dir
}

fn write(path: &Path, text: &str) {
    fs::create_dir_all(path.parent().expect("a parent")).expect("create directories");
    fs::write(path, text).expect("write a file");
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

const CLEAN: &str = "---\ntitle: Home\n---\n\nHello {product}.\n";
const WITH_ERROR: &str = "---\ntitle: Home\n---\n\n[Gone](gone.md)\n";
const WITH_WARNING: &str = "---\ntitle: Home\n---\n\nSee {nope}.\n";

#[test]
fn a_clean_project_exits_0() {
    let dir = project(&[("index.md", CLEAN)]);
    let out = tessera(dir.path(), &["check"]);
    assert_eq!(code(&out), 0, "{}{}", stdout(&out), stderr(&out));
    assert!(stdout(&out).contains("checked 1 file: 0 errors, 0 warnings"));
}

#[test]
fn errors_exit_1_and_show_the_source() {
    let dir = project(&[("index.md", WITH_ERROR)]);
    let out = tessera(dir.path(), &["check"]);
    assert_eq!(code(&out), 1);
    let text = stdout(&out);
    assert!(text.contains("TSR036"), "{text}");
    assert!(text.contains("`gone.md` doesn't exist"), "{text}");
    assert!(text.contains("docs/index.md:5:"), "{text}");
    assert!(text.contains("[Gone](gone.md)"), "the snippet: {text}");
    assert!(text.contains("1 error"), "{text}");
}

#[test]
fn warnings_exit_0_unless_denied() {
    let dir = project(&[("index.md", WITH_WARNING)]);
    let out = tessera(dir.path(), &["check"]);
    assert_eq!(code(&out), 0, "{}", stdout(&out));
    assert!(stdout(&out).contains("1 warning"));
    let out = tessera(dir.path(), &["check", "--deny-warnings"]);
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
        let out = tessera(dir.path(), &args);
        assert_eq!(code(&out), 2, "{args:?}: {}", stderr(&out));
    }
}

#[test]
fn help_and_version_exit_0() {
    let dir = project(&[]);
    let out = tessera(dir.path(), &["--version"]);
    assert_eq!(code(&out), 0);
    assert!(stdout(&out).starts_with("tessera "), "{}", stdout(&out));
    let out = tessera(dir.path(), &["check", "--help"]);
    assert_eq!(code(&out), 0);
    assert!(stdout(&out).contains("--deny-warnings"));
}

#[test]
fn no_content_model_exits_2() {
    let dir = tempfile::tempdir().expect("a temporary directory");
    let out = tessera(dir.path(), &["check"]);
    assert_eq!(code(&out), 2);
    assert!(stderr(&out).contains("tessera.toml"), "{}", stderr(&out));
}

#[test]
fn a_content_model_with_errors_exits_2_and_shows_them() {
    let dir = tempfile::tempdir().expect("a temporary directory");
    write(
        &dir.path().join("tessera.toml"),
        "spec = \"0.1\"\nbogus = 1\n",
    );
    let out = tessera(dir.path(), &["check"]);
    assert_eq!(code(&out), 2);
    assert!(stdout(&out).contains("tessera.toml:2"), "{}", stdout(&out));
    assert!(
        stderr(&out).contains("nothing can be checked"),
        "{}",
        stderr(&out)
    );
}

#[test]
fn finds_the_model_in_a_parent_directory() {
    let dir = project(&[("guides/index.md", WITH_ERROR)]);
    let out = tessera(&dir.path().join("docs/guides"), &["check"]);
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
    let config: PathBuf = dir.path().join("tessera.toml");
    let out = tessera(
        elsewhere.path(),
        &["check", "--config", config.to_str().expect("utf-8")],
    );
    assert_eq!(code(&out), 0, "{}{}", stdout(&out), stderr(&out));
    let out = tessera(
        elsewhere.path(),
        &["--config", dir.path().to_str().expect("utf-8"), "check"],
    );
    assert_eq!(code(&out), 0, "a directory names the model in it");
    let out = tessera(elsewhere.path(), &["check", "--config", "missing.toml"]);
    assert_eq!(code(&out), 2);
}

#[test]
fn a_source_that_isnt_utf8_is_an_error_and_the_rest_is_checked() {
    // SPEC §8.2 (resolved Q52).
    let dir = project(&[("index.md", WITH_ERROR)]);
    fs::write(dir.path().join("docs/bad.md"), [0xff, 0xfe, 0x00]).expect("write bytes");
    let out = tessera(dir.path(), &["check", "--format", "json"]);
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
    let out = tessera(dir.path(), &["check", "--format", "json"]);
    assert_eq!(code(&out), 1);
    let report: serde_json::Value = serde_json::from_str(&stdout(&out)).expect("valid JSON");
    assert_eq!(report["schema_version"], 1);
    assert!(report["tessera_version"].is_string());
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
    assert_eq!(d["code"], "TSR036");
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
    let out = tessera(dir.path(), &["check", "--format", "json"]);
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
        &dir.path().join("tessera.toml"),
        "spec = \"0.1\"\nbogus = 1\n",
    );
    let out = tessera(dir.path(), &["check", "--format", "json"]);
    assert_eq!(code(&out), 2);
    let report: serde_json::Value = serde_json::from_str(&stdout(&out)).expect("valid JSON");
    assert!(report["error"].is_string());
    assert_eq!(report["diagnostics"][0]["file"], "tessera.toml");
    assert_eq!(report["files_checked"], 0);
}

#[test]
fn json_reports_a_missing_model_as_an_error_object() {
    let dir = tempfile::tempdir().expect("a temporary directory");
    let out = tessera(dir.path(), &["check", "--format", "json"]);
    assert_eq!(code(&out), 2);
    let report: serde_json::Value = serde_json::from_str(&stdout(&out)).expect("valid JSON");
    assert!(
        report["error"]
            .as_str()
            .expect("a message")
            .contains("tessera.toml")
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
    let out = tessera(&example, &["check", "--deny-warnings"]);
    assert_eq!(code(&out), 0, "{}{}", stdout(&out), stderr(&out));
}

// ---------------------------------------------------------------------------
// Page-level checks and builds

const BUILDS_MODEL: &str = "spec = \"0.1\"\n\n[project]\ncontent-root = \"docs\"\n\n[types.page]\ndefault = true\n\n[types.page.frontmatter]\ntitle = \"string\"\n\n[dimensions.deployment]\nvalues = [\"cloud\", \"self-managed\"]\nversionless = [\"cloud\", \"self-managed\"]\n\n[builds.site]\nvariants = \"switch\"\navailability = \"badge\"\n\n[builds.cloud]\nvariants = { deployment = \"cloud\" }\navailability = \"badge\"\n";

fn builds_project(files: &[(&str, &str)]) -> TempDir {
    let dir = project(files);
    fs::write(dir.path().join("tessera.toml"), BUILDS_MODEL).expect("write the model");
    dir
}

const DUPLICATE: &str = "---\ntitle: Home\n---\n\n## A\n@id: same\n\n## B\n@id: same\n";
const ONLY_SELF_MANAGED: &str =
    "---\ntitle: Home\n---\n\n@variant {deployment=self-managed}:\nSelf-managed only.\n@end\n";

#[test]
fn a_page_level_error_fails_the_check() {
    let dir = builds_project(&[("index.md", DUPLICATE)]);
    let out = tessera(dir.path(), &["check"]);
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
    let out = tessera(dir.path(), &["check"]);
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
    let site = tessera(dir.path(), &["check", "--build", "site"]);
    assert_eq!(code(&site), 0, "{}{}", stdout(&site), stderr(&site));
    assert!(stdout(&site).contains("0 warnings"), "{}", stdout(&site));
    let cloud = tessera(
        dir.path(),
        &["check", "--build", "cloud", "--deny-warnings"],
    );
    assert_eq!(code(&cloud), 1, "{}{}", stdout(&cloud), stderr(&cloud));
    assert!(stdout(&cloud).contains("variant-no-arm-survives"));
}

#[test]
fn an_unknown_build_is_a_failure_that_lists_the_builds() {
    let dir = builds_project(&[("index.md", DUPLICATE)]);
    let out = tessera(dir.path(), &["check", "--build", "nope"]);
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
    let out = tessera(dir.path(), &["check", "--format", "json"]);
    assert_eq!(code(&out), 1, "{}{}", stdout(&out), stderr(&out));
    let json: serde_json::Value = serde_json::from_str(&stdout(&out)).expect("JSON");
    let diagnostics = json["diagnostics"].as_array().expect("a list");
    assert_eq!(diagnostics.len(), 1, "{diagnostics:#?}");
    let d = &diagnostics[0];
    assert_eq!(d["slug"], "link-id-missing");
    assert_eq!(d["file"], "docs/index.md");
    assert_eq!(d["range"]["start"]["line"], 5);
    assert_eq!(d["related"][0]["file"], "docs/_f.md");
}
