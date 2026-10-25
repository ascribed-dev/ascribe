//! `ascribe.addresses` and the commands around it, by running the binary in
//! a temporary repository holding `examples/astro-site` with
//! `[consumer] host = "netlify"`: recording a move, the check against a
//! base, the bases it can't read, and what the build writes.

#![allow(clippy::expect_used, clippy::unwrap_used, clippy::panic)]

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use tempfile::TempDir;

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

/// A repository whose root is the example's project, with Netlify as its
/// host, committed on `main`.
fn repo() -> TempDir {
    let dir = tempfile::tempdir().expect("a temporary directory");
    git(dir.path(), &["init", "-q", "-b", "main"]);
    write_project(dir.path());
    commit(dir.path(), "first");
    dir
}

/// The example's model, with Netlify as its host, its content, and its
/// API's spec.
fn write_project(root: &Path) {
    let model = fs::read_to_string(example().join("ascribe.toml")).expect("the model");
    fs::write(
        root.join("ascribe.toml"),
        model.replace("agents = true\n", "agents = true\nhost = \"netlify\"\n"),
    )
    .expect("write the model");
    copy(&example().join("content"), &root.join("content"));
    copy(&example().join("api"), &root.join("api"));
}

fn git(dir: &Path, args: &[&str]) -> String {
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
    String::from_utf8_lossy(&out.stdout).into_owned()
}

fn commit(dir: &Path, message: &str) {
    git(dir, &["add", "-A"]);
    git(dir, &["commit", "-q", "-m", message]);
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

/// Both streams, for a failed assertion.
fn both(output: &Output) -> String {
    format!("{}{}", stdout(output), stderr(output))
}

/// The project with `ascribe.addresses`, committed on `main`, and a branch
/// `restructure` where the setup guide is renamed and that's committed.
fn renamed() -> TempDir {
    let dir = repo();
    let out = ascribe(dir.path(), &["addresses", "init"]);
    assert_eq!(code(&out), 0, "{}", both(&out));
    assert_eq!(
        stdout(&out),
        "Wrote ascribe.addresses. Moves are recorded from now on.\n"
    );
    commit(dir.path(), "Keep the addresses we publish");
    git(dir.path(), &["switch", "-q", "-c", "restructure"]);
    let out = ascribe(
        dir.path(),
        &[
            "rename",
            "content/Guides/My Setup.md",
            "content/Guides/Setup.md",
        ],
    );
    assert_eq!(code(&out), 0, "{}", both(&out));
    assert!(
        stdout(&out).contains("recorded the move in ascribe.addresses\n"),
        "{}",
        both(&out)
    );
    commit(dir.path(), "Rename the setup guide");
    dir
}

fn site_file(dir: &Path, rel: &str) -> String {
    fs::read_to_string(dir.join(".ascribe/build/site/site").join(rel))
        .unwrap_or_else(|e| panic!("{rel}: {e}"))
}

const PAIRS: &str = "/docs/guides/my-setup /docs/guides/setup 301\n\
    /docs/guides/my-setup.md /docs/guides/setup.md 301\n";

#[test]
fn a_recorded_move_passes_and_a_deleted_page_is_reported() {
    let dir = renamed();
    let addresses = fs::read_to_string(dir.path().join("ascribe.addresses")).expect("the file");
    assert!(
        addresses.contains("[moved]\n\"Guides/My Setup.md\" = \"Guides/Setup.md\"\n"),
        "{addresses}"
    );
    let out = ascribe(dir.path(), &["check", "--base", "main", "--deny-warnings"]);
    assert_eq!(code(&out), 0, "{}", both(&out));
    assert!(!both(&out).contains("address-lost"), "{}", both(&out));

    git(
        dir.path(),
        &["rm", "-q", "content/Guides/Teams/Share a loom.md"],
    );
    let out = ascribe(dir.path(), &["check", "--base", "main", "--deny-warnings"]);
    assert_ne!(code(&out), 0, "{}", both(&out));
    assert!(
        both(&out).contains(
            "Guides/Teams/Share a loom.md was published at /docs/guides/teams/share-a-loom and \
             /docs/guides/teams/share-a-loom.md, and nothing publishes them now"
        ),
        "{}",
        both(&out)
    );
    // Retiring it is a decision, and the check passes.
    let out = ascribe(
        dir.path(),
        &[
            "addresses",
            "retire",
            "Guides/Teams/Share a loom.md",
            "--reason",
            "Sharing is in the setup guide now.",
        ],
    );
    assert_eq!(code(&out), 0, "{}", both(&out));
    let out = ascribe(dir.path(), &["check", "--base", "main", "--deny-warnings"]);
    assert_eq!(code(&out), 0, "{}", both(&out));

    let out = ascribe(dir.path(), &["build"]);
    assert_eq!(code(&out), 0, "{}", both(&out));
    // Then the retired page's addresses, each serving its page with a 404.
    assert_eq!(
        site_file(dir.path(), "_ascribe/host/_redirects"),
        format!(
            "{PAIRS}/docs/guides/teams/share-a-loom /docs/_ascribe/gone/guides/teams/share-a-loom 404\n\
             /docs/guides/teams/share-a-loom.md /docs/_ascribe/gone/guides/teams/share-a-loom 404\n"
        )
    );
    let redirects = site_file(dir.path(), "_ascribe/redirects.json");
    assert!(
        redirects.contains("\"from\": \"/docs/guides/my-setup\""),
        "{redirects}"
    );
    assert!(
        redirects.contains("\"path\": \"/docs/guides/teams/share-a-loom\""),
        "{redirects}"
    );
}

/// The seam: once the move is merged and the base has moved past it, the
/// base has the new page, the check has nothing to report, and the build
/// still writes the redirect, from the file alone.
#[test]
fn a_merged_move_keeps_its_redirect_once_the_base_moves_past_it() {
    let dir = renamed();
    git(dir.path(), &["switch", "-q", "main"]);
    git(dir.path(), &["merge", "-q", "--ff-only", "restructure"]);
    fs::write(dir.path().join("README.md"), "Later work.\n").expect("write a file");
    commit(dir.path(), "Later");
    git(dir.path(), &["switch", "-q", "-c", "later"]);
    let out = ascribe(dir.path(), &["check", "--base", "main", "--deny-warnings"]);
    assert_eq!(code(&out), 0, "{}", both(&out));
    let out = ascribe(dir.path(), &["build"]);
    assert_eq!(code(&out), 0, "{}", both(&out));
    assert_eq!(site_file(dir.path(), "_ascribe/host/_redirects"), PAIRS);
}

#[test]
fn a_shallow_clone_with_no_merge_base_says_the_addresses_werent_compared() {
    let dir = renamed();
    let clone = tempfile::tempdir().expect("a temporary directory");
    let url = format!("file://{}", dir.path().display());
    git(
        clone.path(),
        &[
            "clone",
            "-q",
            "--depth",
            "1",
            "--no-single-branch",
            "--branch",
            "restructure",
            &url,
            "work",
        ],
    );
    let work = clone.path().join("work");
    let out = ascribe(&work, &["check", "--base", "origin/main"]);
    assert!(
        both(&out).contains("the addresses weren't compared with the base:"),
        "{}",
        both(&out)
    );
    let out = ascribe(
        &work,
        &["check", "--base", "origin/main", "--deny-warnings"],
    );
    assert_ne!(code(&out), 0, "{}", both(&out));
}

#[test]
fn an_exact_base_thats_not_there_says_the_addresses_werent_compared() {
    let dir = renamed();
    let out = ascribe(
        dir.path(),
        &[
            "check",
            "--base",
            "0123456789abcdef0123456789abcdef01234567",
            "--base-exact",
        ],
    );
    assert!(
        both(&out).contains("the addresses weren't compared with the base:"),
        "{}",
        both(&out)
    );
}

#[test]
fn a_base_with_no_project_published_nothing_to_keep() {
    let dir = tempfile::tempdir().expect("a temporary directory");
    git(dir.path(), &["init", "-q", "-b", "main"]);
    fs::write(dir.path().join("README.md"), "Before the docs.\n").expect("write a file");
    commit(dir.path(), "first");
    git(dir.path(), &["switch", "-q", "-c", "docs"]);
    write_project(dir.path());
    let out = ascribe(dir.path(), &["addresses", "init"]);
    assert_eq!(code(&out), 0, "{}", both(&out));
    commit(dir.path(), "Add the docs");
    // It says nothing was compared, and isn't a warning.
    let out = ascribe(dir.path(), &["check", "--base", "main", "--deny-warnings"]);
    assert_eq!(code(&out), 0, "{}", both(&out));
    // The base is named by the revision asked for, and the commit compared.
    let text = both(&out);
    assert!(
        text.contains("the addresses weren't compared with the base: `main` (")
            && text.contains(") has no Ascribe project here, so it published nothing to keep"),
        "{text}"
    );
}

/// The project with `ascribe.addresses` on `main`, and a branch where a
/// page is moved with `git mv` and another deleted, as phase 2's goal has
/// it.
fn reorganized() -> TempDir {
    let dir = repo();
    let out = ascribe(dir.path(), &["addresses", "init"]);
    assert_eq!(code(&out), 0, "{}", both(&out));
    commit(dir.path(), "Keep the addresses we publish");
    git(dir.path(), &["switch", "-q", "-c", "reorganize"]);
    fs::create_dir_all(dir.path().join("content/Start")).expect("a folder");
    git(
        dir.path(),
        &[
            "mv",
            "content/Guides/My Setup.md",
            "content/Start/My Setup.md",
        ],
    );
    git(dir.path(), &["rm", "-q", "content/Reference/Options.md"]);
    dir
}

#[test]
fn a_move_git_sees_is_recorded_from_git_and_a_deleted_page_is_offered_its_fixes() {
    let dir = reorganized();
    let out = ascribe(dir.path(), &["check", "--base", "main", "--format", "json"]);
    let report: serde_json::Value = serde_json::from_slice(&out.stdout).expect("the report");
    let lost: Vec<&serde_json::Value> = report["diagnostics"]
        .as_array()
        .expect("diagnostics")
        .iter()
        .filter(|d| d["slug"] == "address-lost")
        .collect();
    assert_eq!(lost.len(), 2, "{}", both(&out));
    let titles = |d: &serde_json::Value| -> Vec<String> {
        d["fixes"]
            .as_array()
            .expect("fixes")
            .iter()
            .map(|f| f["title"].as_str().expect("a title").to_owned())
            .collect()
    };
    // The moved page: git's renames, which are safe.
    assert_eq!(
        titles(lost[0])[0],
        "git sees it renamed to Start/My Setup.md: record git's renames, as `ascribe addresses move --from-git` does"
    );
    assert_eq!(lost[0]["fixes"][0]["applicability"], "safe");
    // The deleted page: forwarded to the page most like it, or retired.
    let deleted = titles(lost[1]);
    assert!(
        deleted[0].starts_with("Forward it to the page most like it, "),
        "{deleted:?}"
    );
    assert!(
        deleted
            .last()
            .expect("a fix")
            .starts_with("Retire it, and write why"),
        "{deleted:?}"
    );

    let out = ascribe(
        dir.path(),
        &["addresses", "move", "--from-git", "--base", "main"],
    );
    assert_eq!(code(&out), 0, "{}", both(&out));
    assert_eq!(
        stdout(&out),
        "Recorded 1 move git sees: Guides/My Setup.md → Start/My Setup.md.\n"
    );
    let out = ascribe(dir.path(), &["check", "--base", "main"]);
    let text = both(&out);
    assert!(!text.contains("Guides/My Setup.md was published"), "{text}");
    assert!(
        text.contains("Reference/Options.md was published"),
        "{text}"
    );
    // Recorded once: again, there's nothing to record.
    let out = ascribe(
        dir.path(),
        &["addresses", "move", "--from-git", "--base", "main"],
    );
    assert!(
        stdout(&out)
            .starts_with("git sees no move of a page whose addresses are lost, against `main`"),
        "{}",
        both(&out)
    );
}

/// A page renamed and edited a little is a rename git sees; a copy isn't a
/// move, since the page it was copied from is still there.
#[test]
fn from_git_records_a_rename_with_an_edit_and_not_a_copy() {
    let dir = reorganized();
    let moved = dir.path().join("content/Start/My Setup.md");
    let text = fs::read_to_string(&moved).expect("the moved page");
    fs::write(&moved, format!("{text}\nOne more line.\n")).expect("edit the page");
    fs::copy(
        dir.path().join("content/index.md"),
        dir.path().join("content/Start/index.md"),
    )
    .expect("copy a page");
    git(dir.path(), &["add", "-A"]);
    let out = ascribe(
        dir.path(),
        &[
            "addresses",
            "move",
            "--from-git",
            "--base",
            "main",
            "--format",
            "json",
        ],
    );
    assert_eq!(code(&out), 0, "{}", both(&out));
    let report: serde_json::Value = serde_json::from_slice(&out.stdout).expect("the report");
    assert_eq!(
        report["moves"],
        serde_json::json!([{ "key": "Guides/My Setup.md", "value": "Start/My Setup.md" }]),
        "{report}"
    );
    assert_eq!(report["written"], true, "{report}");
}

#[test]
fn check_fix_records_git_s_renames() {
    let dir = reorganized();
    let out = ascribe(dir.path(), &["check", "--base", "main", "--fix"]);
    let text = both(&out);
    assert!(!text.contains("Guides/My Setup.md was published"), "{text}");
    let addresses = fs::read_to_string(dir.path().join("ascribe.addresses")).expect("the file");
    assert!(
        addresses.contains("\"Guides/My Setup.md\" = \"Start/My Setup.md\""),
        "{addresses}"
    );
}

#[test]
fn deleting_the_file_the_base_has_is_an_error() {
    let dir = renamed();
    fs::remove_file(dir.path().join("ascribe.addresses")).expect("remove it");
    commit(dir.path(), "Stop keeping the addresses");
    let out = ascribe(dir.path(), &["check", "--base", "main"]);
    assert_ne!(code(&out), 0, "{}", both(&out));
    assert!(
        both(&out).contains("ascribe.addresses was deleted, and the base has it, with 0 entries"),
        "{}",
        both(&out)
    );
    // Compared with its own head, the deletion is the decision.
    let out = ascribe(dir.path(), &["check", "--base", "HEAD"]);
    assert_eq!(code(&out), 0, "{}", both(&out));
}
