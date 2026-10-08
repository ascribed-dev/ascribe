//! `ascribe diff`: comparing the working tree with a git revision, by running
//! the binary in a temporary repository.

#![allow(clippy::expect_used, clippy::unwrap_used)]

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use tempfile::TempDir;

const MODEL: &str = r#"spec = "0.1"

[project]
content-root = "docs"

[phrases]
product = "Quill"

[builds.site]

[builds.other]
"#;

const PAGE: &str = "---\ntitle: Install\n---\n\n# Install\n\nIntro for {product}.\n\n@include: _fragments/prereqs.md\n\nThe end.\n";

/// A repository whose project is in `site/`, committed on `main`.
fn repo() -> TempDir {
    let dir = tempfile::tempdir().expect("a temporary directory");
    git(dir.path(), &["init", "-q", "-b", "main"]);
    write(dir.path(), "site/ascribe.toml", MODEL);
    write(dir.path(), "site/docs/install.md", PAGE);
    write(dir.path(), "site/docs/about.md", "# About\n\nAbout us.\n");
    write(
        dir.path(),
        "site/docs/_fragments/prereqs.md",
        "You need agent 2.2 or later.\n",
    );
    commit(dir.path(), "first");
    dir
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
    assert!(out.status.success(), "git {args:?} failed");
}

fn commit(dir: &Path, message: &str) {
    git(dir, &["add", "-A"]);
    git(dir, &["commit", "-q", "-m", message]);
}

fn write(root: &Path, rel: &str, text: &str) {
    let path: PathBuf = rel.split('/').fold(root.to_path_buf(), |p, s| p.join(s));
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

fn site(dir: &TempDir) -> PathBuf {
    dir.path().join("site")
}

#[test]
fn no_changes() {
    let dir = repo();
    let out = ascribe(&site(&dir), &["diff"]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    let text = stdout(&out);
    assert!(text.starts_with("compared with main ("), "{text}");
    assert!(text.contains("site: no changes\n"), "{text}");
    assert!(text.contains("other: no changes\n"), "{text}");
    let out = ascribe(&site(&dir), &["diff", "--exit-code"]);
    assert_eq!(code(&out), 0);
}

#[test]
fn a_change_through_a_fragment_in_text() {
    let dir = repo();
    write(
        dir.path(),
        "site/docs/_fragments/prereqs.md",
        "You need agent 2.4 or later.\n",
    );
    // Uncommitted and unstaged: the working tree is what's compared.
    let out = ascribe(&site(&dir), &["diff", "--build", "site"]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    let text = stdout(&out);
    assert!(
        text.contains(
            "site: 1 page changed\n  install.md: 1 changed (through _fragments/prereqs.md)\n"
        ),
        "{text}"
    );
    assert!(!text.contains("other:"), "{text}");
    let out = ascribe(&site(&dir), &["diff", "--exit-code"]);
    assert_eq!(code(&out), 1);
}

#[test]
fn json_output() {
    let dir = repo();
    write(
        dir.path(),
        "site/docs/install.md",
        &PAGE.replace("The end.", "The very end."),
    );
    write(dir.path(), "site/docs/new.md", "# New\n");
    let out = ascribe(
        &site(&dir),
        &["diff", "--format", "json", "--build", "site"],
    );
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    let json: serde_json::Value = serde_json::from_str(&stdout(&out)).expect("JSON");
    assert_eq!(json["schema_version"], 1);
    assert_eq!(json["base"]["requested"], "main");
    assert_eq!(json["base"]["commit"], json["base"]["merge_base"]);
    assert_eq!(json["repository"]["project_prefix"], "site/");
    let pages = json["builds"][0]["pages"].as_array().expect("pages");
    assert_eq!(pages.len(), 2);
    assert_eq!(pages[0]["path"], "install.md");
    assert_eq!(pages[0]["status"], "changed");
    assert_eq!(pages[0]["changes"][0]["now"]["source"], "install.md:11-11");
    assert_eq!(pages[1]["path"], "new.md");
    assert_eq!(pages[1]["status"], "added");
    assert_eq!(pages[1]["route"], "/new/");
}

#[test]
fn a_phrase_change_names_the_model() {
    let dir = repo();
    write(
        dir.path(),
        "site/ascribe.toml",
        &MODEL.replace("\"Quill\"", "\"Quill Cloud\""),
    );
    let out = ascribe(&site(&dir), &["diff", "--build", "site"]);
    assert!(
        stdout(&out).contains("  install.md: 1 changed (through ascribe.toml)\n"),
        "{}",
        stdout(&out)
    );
}

#[test]
fn a_base_branch_and_its_merge_base() {
    let dir = repo();
    git(dir.path(), &["checkout", "-q", "-b", "feature"]);
    write(
        dir.path(),
        "site/docs/about.md",
        "# About\n\nAbout all of us.\n",
    );
    commit(dir.path(), "on the branch");
    git(dir.path(), &["checkout", "-q", "main"]);
    write(dir.path(), "site/docs/later.md", "# Later\n");
    commit(dir.path(), "on main");
    git(dir.path(), &["checkout", "-q", "feature"]);

    // From the merge base: main's later page isn't a change of this branch.
    let out = ascribe(&site(&dir), &["diff", "--build", "site"]);
    let text = stdout(&out);
    assert!(text.contains("from its merge base with HEAD"), "{text}");
    assert!(text.contains("  about.md: 1 changed\n"), "{text}");
    assert!(!text.contains("later.md"), "{text}");

    // Against main itself, its later page is gone in the working tree.
    let out = ascribe(
        &site(&dir),
        &["diff", "--build", "site", "--base", "main", "--base-exact"],
    );
    let text = stdout(&out);
    assert!(text.contains("  later.md: removed\n"), "{text}");
}

#[test]
fn a_project_new_since_the_base_is_all_added() {
    let dir = tempfile::tempdir().unwrap();
    git(dir.path(), &["init", "-q", "-b", "main"]);
    write(dir.path(), "README.md", "# Repo\n");
    commit(dir.path(), "first");
    write(dir.path(), "site/ascribe.toml", MODEL);
    write(dir.path(), "site/docs/about.md", "# About\n");
    let out = ascribe(&site(&dir), &["diff", "--build", "site"]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    assert!(
        stdout(&out).contains("site: 1 page changed\n  about.md: added\n"),
        "{}",
        stdout(&out)
    );
}

#[test]
fn failures_exit_with_2() {
    let dir = repo();
    let out = ascribe(&site(&dir), &["diff", "--base", "no-such-branch"]);
    assert_eq!(code(&out), 2);
    assert!(
        stderr(&out).contains("`no-such-branch` isn't a revision"),
        "{}",
        stderr(&out)
    );

    let out = ascribe(&site(&dir), &["diff", "--build", "nope"]);
    assert_eq!(code(&out), 2);
    assert!(stderr(&out).contains("no build `nope`"), "{}", stderr(&out));

    // Not a repository.
    let plain = tempfile::tempdir().unwrap();
    write(plain.path(), "ascribe.toml", MODEL);
    write(plain.path(), "docs/index.md", "# Home\n");
    let out = ascribe(plain.path(), &["diff"]);
    assert_eq!(code(&out), 2);
    assert!(
        stderr(&out).contains("isn't in a git repository"),
        "{}",
        stderr(&out)
    );
}

#[test]
fn a_base_model_with_errors_exits_with_2() {
    let dir = repo();
    write(
        dir.path(),
        "site/ascribe.toml",
        "spec = \"0.1\"\n[builds.site]\nunknown-key = 1\n",
    );
    commit(dir.path(), "a broken model");
    write(dir.path(), "site/ascribe.toml", MODEL);
    let out = ascribe(&site(&dir), &["diff", "--base", "HEAD", "--base-exact"]);
    assert_eq!(code(&out), 2);
    assert!(
        stderr(&out).contains("ascribe.toml at "),
        "{}",
        stderr(&out)
    );
    assert!(stderr(&out).contains("has errors, so nothing can be compared"));
}

#[test]
fn a_title_change_is_named() {
    let dir = repo();
    write(
        dir.path(),
        "site/docs/install.md",
        &PAGE.replace("title: Install", "title: Set up"),
    );
    let out = ascribe(&site(&dir), &["diff", "--build", "site"]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    assert!(
        stdout(&out).contains("  install.md: title changed\n"),
        "{}",
        stdout(&out)
    );
}

/// The data a `--format html` report's script reads, with the commits it
/// names replaced, so it can be compared from run to run.
fn report_data(html: &str, commits: &[&str]) -> serde_json::Value {
    let open = "<script type=\"application/json\" id=\"ascribe-review-data\">";
    let start = html.find(open).expect("the data") + open.len();
    let end = start + html[start..].find("</script>").expect("its end");
    let mut text = html[start..end].to_owned();
    for commit in commits {
        text = text.replace(commit, "<commit>");
    }
    let version = format!("\"ascribe_version\":\"{}\"", env!("CARGO_PKG_VERSION"));
    assert!(text.contains(&version), "{text}");
    text = text.replace(&version, "\"ascribe_version\":\"<version>\"");
    serde_json::from_str(&text).expect("JSON")
}

fn head(dir: &Path) -> String {
    let out = Command::new("git")
        .current_dir(dir)
        .args(["rev-parse", "HEAD"])
        .output()
        .expect("run git");
    String::from_utf8_lossy(&out.stdout).trim().to_owned()
}

#[test]
fn html_report_shows_every_page_a_fragment_reaches() {
    let dir = repo();
    write(
        dir.path(),
        "site/docs/guide.md",
        "# Guide\n\n@include: _fragments/prereqs.md\n",
    );
    write(dir.path(), "site/docs/logo.png", "\u{89}PNG");
    commit(dir.path(), "second");
    let base = head(dir.path());
    write(
        dir.path(),
        "site/docs/_fragments/prereqs.md",
        "You need agent 2.4 or later.\n\n![Logo](../logo.png)\n",
    );
    let out = ascribe(
        &site(&dir),
        &["diff", "--format", "html", "--build", "site"],
    );
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    let html = stdout(&out);
    assert!(html.starts_with("<!doctype html>\n"), "{html}");
    assert!(html.contains("<title>Review: 2 changed pages</title>"));
    let data = report_data(&html, &[&base]);
    insta::assert_snapshot!(
        "html_report_fragment",
        serde_json::to_string_pretty(&data).expect("JSON")
    );
}

#[test]
fn html_report_loads_nothing_from_the_network() {
    let dir = repo();
    write(
        dir.path(),
        "site/docs/about.md",
        "# About\n\nAbout us.\n\n![Remote](https://example.com/a.png)\n",
    );
    let out = ascribe(&site(&dir), &["diff", "--format", "html"]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    let html = stdout(&out);
    // Outside the data its script reads (pages, whose images the script
    // replaces with what the file holds), nothing names a URL to load.
    let open = "<script type=\"application/json\" id=\"ascribe-review-data\">";
    let start = html.find(open).expect("the data");
    let end = start + html[start..].find("</script>").expect("its end");
    let rest = format!("{}{}", &html[..start], &html[end..]).to_ascii_lowercase();
    for (i, _) in rest.match_indices("src=") {
        let value = rest[i + 4..].trim_start_matches(['"', '\'']);
        assert!(!value.starts_with("http") && !value.starts_with("//"));
    }
    assert!(!rest.contains("<link"));
    for (i, _) in rest.match_indices("url(") {
        let value = rest[i + 4..].trim_start_matches(['"', '\'', ' ']);
        assert!(value.starts_with("data:") || value.starts_with('#'));
    }
    assert!(
        rest.contains("content-security-policy\" content=\"default-src 'none'; img-src data:;")
    );
    let data = report_data(&html, &[]);
    assert!(data["images"].as_object().expect("images").is_empty());
}

#[test]
fn errors_in_the_working_tree_are_counted_not_fatal() {
    let dir = repo();
    write(
        dir.path(),
        "site/docs/about.md",
        "---\ntitle: About\n---\n\n# About\n\nAbout us.\n\n@end\n",
    );
    let out = ascribe(&site(&dir), &["diff", "--build", "site"]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    assert!(
        stdout(&out).contains("site: 1 page changed\n"),
        "{}",
        stdout(&out)
    );
    assert_eq!(
        stderr(&out),
        "warning: the working tree has 1 error; `ascribe check --build site` lists it\n"
    );
    let out = ascribe(&site(&dir), &["diff", "--format", "json"]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    assert!(
        stderr(&out).contains("`ascribe check` lists it"),
        "{}",
        stderr(&out)
    );
    let json: serde_json::Value = serde_json::from_str(&stdout(&out)).expect("JSON");
    assert_eq!(json["working_tree_errors"], 1);
}

#[test]
fn a_clean_working_tree_says_nothing_on_standard_error() {
    let dir = repo();
    write(
        dir.path(),
        "site/docs/about.md",
        "---\ntitle: About\n---\n\n# About\n\nAbout them.\n",
    );
    let out = ascribe(&site(&dir), &["diff", "--format", "json"]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    assert_eq!(stderr(&out), "");
    let json: serde_json::Value = serde_json::from_str(&stdout(&out)).expect("JSON");
    assert_eq!(json["working_tree_errors"], 0);
}

#[test]
fn html_report_formats_a_title_with_code() {
    let dir = repo();
    write(
        dir.path(),
        "site/ascribe.toml",
        &format!(
            "{MODEL}\n[types.page]\ndefault = true\n\n[types.page.frontmatter]\ntitle = {{ type = \"string\", inline = \"code\" }}\n"
        ),
    );
    write(
        dir.path(),
        "site/docs/keys.md",
        "---\ntitle: \"`ascribe.toml` keys\"\n---\n\nKeys.\n",
    );
    commit(dir.path(), "second");
    write(
        dir.path(),
        "site/docs/keys.md",
        "---\ntitle: \"`ascribe.toml` keys\"\n---\n\nEvery key.\n",
    );
    let out = ascribe(
        &site(&dir),
        &["diff", "--format", "html", "--build", "site"],
    );
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    let data = report_data(&stdout(&out), &[]);
    let page = &data["builds"][0]["pages"][0];
    assert_eq!(page["path"], "keys.md");
    // The plain title for tooltips and search, and the formatted one for the
    // page list and the heading.
    assert_eq!(page["title"], "ascribe.toml keys");
    assert_eq!(
        page["formatted_title"],
        serde_json::json!([
            { "type": "code", "value": "ascribe.toml" },
            { "type": "text", "value": " keys" },
        ])
    );
}
