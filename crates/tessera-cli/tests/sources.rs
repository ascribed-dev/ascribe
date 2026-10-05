//! `ascribe sources`, and every other command over a project with sources in
//! other repositories, by running the binary in temporary repositories. The
//! code repositories are reached by `file://` URLs: a real `git` fetch, with
//! no network.

#![allow(clippy::expect_used, clippy::unwrap_used, clippy::panic)]

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use tempfile::TempDir;

const AUTH_V1: &str = "// :snippet-start: login\nfn login() {}\n// :snippet-end:\n";
const AUTH_V2: &str = "// :snippet-start: login\nfn login(user: &str) {}\n// :snippet-end:\n";

fn git(dir: &Path, args: &[&str]) -> String {
    let out = Command::new("git")
        .current_dir(dir)
        .args(["-c", "user.name=Test", "-c", "user.email=test@example.com"])
        .args(["-c", "commit.gpgsign=false", "-c", "core.autocrlf=false"])
        .args(args)
        .output()
        .expect("run git");
    assert!(
        out.status.success(),
        "git {args:?} failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).trim().to_owned()
}

fn commit(dir: &Path, message: &str) -> String {
    git(dir, &["add", "-A"]);
    git(dir, &["commit", "-q", "--allow-empty", "-m", message]);
    git(dir, &["rev-parse", "HEAD"])
}

fn write(root: &Path, rel: &str, text: &str) {
    let path: PathBuf = rel.split('/').fold(root.to_path_buf(), |p, s| p.join(s));
    fs::create_dir_all(path.parent().expect("a parent")).expect("create directories");
    fs::write(path, text).expect("write a file");
}

fn file_url(dir: &Path) -> String {
    let text = dir.display().to_string().replace('\\', "/");
    if text.starts_with('/') {
        format!("file://{text}")
    } else {
        format!("file:///{text}")
    }
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

/// Two code repositories, and a docs repository whose project takes an
/// example from each: committed on `main` with the copies fetched, and a
/// `feature` branch checked out.
struct Pair {
    api: TempDir,
    cli: TempDir,
    docs: TempDir,
    cache: TempDir,
}

impl Pair {
    fn new() -> Pair {
        let pair = Pair {
            api: tempfile::tempdir().unwrap(),
            cli: tempfile::tempdir().unwrap(),
            docs: tempfile::tempdir().unwrap(),
            cache: tempfile::tempdir().unwrap(),
        };
        for (dir, path, text) in [
            (pair.api.path(), "src/auth.rs", AUTH_V1),
            (pair.cli.path(), "cmd/main.go", "package main\n"),
        ] {
            git(dir, &["init", "-q", "-b", "main"]);
            git(dir, &["config", "uploadpack.allowFilter", "true"]);
            write(dir, path, text);
            commit(dir, "first");
        }
        let docs = pair.docs.path();
        git(docs, &["init", "-q", "-b", "main"]);
        write(
            docs,
            "ascribe.toml",
            &format!(
                "spec = \"0.1\"\n\n[project]\ncontent-root = \"docs\"\n\n[consumer]\nsite = \"https://docs.example.com\"\n\n[sources.api]\ngit = \"{}\"\nbranch = \"main\"\ninclude = [\"src/**\"]\n\n[sources.cli]\ngit = \"{}\"\n",
                file_url(pair.api.path()),
                file_url(pair.cli.path())
            ),
        );
        write(
            docs,
            "docs/auth.md",
            "---\ntitle: Sign in\n---\n\n# Sign in\n\nCall `login` first:\n\n@snippet: api:src/auth.rs#login\n",
        );
        write(
            docs,
            "docs/cli.md",
            "---\ntitle: The CLI\n---\n\n# The CLI\n\n@snippet: cli:cmd/main.go\n",
        );
        let out = pair.ascribe(&["sources", "fetch"]);
        assert_eq!(code_of(&out), 0, "{}", stderr(&out));
        commit(docs, "docs");
        git(docs, &["checkout", "-q", "-b", "feature"]);
        pair
    }

    fn docs(&self) -> &Path {
        self.docs.path()
    }

    fn command(&self, dir: &Path, args: &[&str]) -> Command {
        let mut command = Command::new(env!("CARGO_BIN_EXE_ascribe"));
        command
            .current_dir(dir)
            .args(args)
            .env("NO_COLOR", "1")
            .env("ASCRIBE_CACHE_DIR", self.cache.path());
        command
    }

    fn ascribe(&self, args: &[&str]) -> Output {
        self.command(self.docs(), args)
            .output()
            .expect("run ascribe")
    }

    /// The code changes upstream.
    fn change_api(&self) -> String {
        write(self.api.path(), "src/auth.rs", AUTH_V2);
        commit(self.api.path(), "Take a user when logging in")
    }

    fn read(&self, rel: &str) -> String {
        fs::read_to_string(
            rel.split('/')
                .fold(self.docs().to_path_buf(), |p, s| p.join(s)),
        )
        .unwrap()
    }
}

/// Runs `ascribe` with no way to reach another repository: the code
/// repositories moved away, an empty cache, and `git` allowed no transport.
fn offline(pair: &Pair, dir: &Path, args: &[&str]) -> Output {
    let empty = tempfile::tempdir().unwrap();
    let mut command = pair.command(dir, args);
    command
        .env("ASCRIBE_CACHE_DIR", empty.path())
        .env("GIT_ALLOW_PROTOCOL", "none");
    command.output().expect("run ascribe")
}

fn hide(dir: &Path) -> PathBuf {
    let hidden = dir.with_extension("hidden");
    fs::rename(dir, &hidden).unwrap();
    hidden
}

#[test]
fn update_names_the_page_and_rewrites_the_lock_and_the_copy() {
    let pair = Pair::new();
    let head = pair.change_api();

    let out = pair.ascribe(&["sources", "update", "--format", "summary"]);
    assert_eq!(code_of(&out), 0, "{}", stderr(&out));
    let summary = stdout(&out);
    assert!(
        summary.starts_with("### Sources\n\n- **api** `"),
        "{summary}"
    );
    assert!(
        summary.contains(&format!(
            "`{}`, the head of `main`: 1 commit\n  - `{}` Take a user when logging in\n  - Copies: `src/auth.rs` changed.\n",
            &head[..7],
            &head[..7]
        )),
        "{summary}"
    );
    assert!(
        summary.ends_with(
            "### Examples that changed\n\nThe page shows the new code; check the words around it:\n\n- [auth.md](https://docs.example.com/auth/)\n  - `api:src/auth.rs#login` (+1 \u{2212}1)\n"
        ),
        "{summary}"
    );
    assert_eq!(pair.read("sources/api/src/auth.rs"), AUTH_V2);
    assert!(pair.read("ascribe.lock").contains(&head));

    // Again: nothing to move, no file changes, and the summary is empty.
    let lock = pair.read("ascribe.lock");
    let out = pair.ascribe(&["sources", "update", "--format", "summary"]);
    assert_eq!(code_of(&out), 0, "{}", stderr(&out));
    assert_eq!(stdout(&out), "");
    let out = pair.ascribe(&["sources", "update"]);
    assert!(
        stdout(&out).ends_with("\nNothing to move, and no file changed.\n"),
        "{}",
        stdout(&out)
    );
    assert_eq!(pair.read("ascribe.lock"), lock);
}

#[test]
fn update_as_text_and_json() {
    let pair = Pair::new();
    let head = pair.change_api();
    let out = pair.ascribe(&["sources", "update", "api"]);
    assert_eq!(code_of(&out), 0, "{}", stderr(&out));
    let text = stdout(&out);
    assert!(
        text.contains(&format!(
            ", the head of main, 1 commit\n  {} Take a user when logging in\n  changed src/auth.rs\n\nExamples that changed. The page shows the new code; check the words around it:\n  auth.md\n    api:src/auth.rs#login (+1 \u{2212}1)\n",
            &head[..7]
        )),
        "{text}"
    );

    git(pair.docs(), &["checkout", "-q", "--", "."]);
    let out = pair.ascribe(&["sources", "update", "--format", "json"]);
    assert_eq!(code_of(&out), 0, "{}", stderr(&out));
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(json["schema_version"], 1);
    assert_eq!(json["changed"], true);
    let api = &json["sources"][0];
    assert_eq!(api["name"], "api");
    assert_eq!(api["to"], head.as_str());
    assert_eq!(api["moved"], true);
    assert_eq!(api["commits"]["count"], 1);
    assert_eq!(api["files"][0]["path"], "src/auth.rs");
    assert_eq!(api["files"][0]["change"], "changed");
    assert_eq!(json["sources"][1]["moved"], false);
    assert_eq!(json["pages"][0]["path"], "auth.md");
    assert_eq!(
        json["pages"][0]["examples"][0]["file"],
        "sources/api/src/auth.rs"
    );
    assert_eq!(json["pages_unavailable"], serde_json::Value::Null);
}

#[test]
fn a_region_gone_upstream_is_reported_and_check_fails() {
    let pair = Pair::new();
    write(pair.api.path(), "src/auth.rs", "fn sign_in() {}\n");
    commit(pair.api.path(), "Rename login");
    let out = pair.ascribe(&["sources", "update"]);
    assert_eq!(code_of(&out), 0, "{}", stderr(&out));
    assert!(
        stdout(&out).contains(
            "Examples that no longer resolve. `ascribe check` reports them too:\n  auth.md\n    api:src/auth.rs#login: the file has no region `login`\n"
        ),
        "{}",
        stdout(&out)
    );
    let out = pair.ascribe(&["check"]);
    assert_eq!(code_of(&out), 1);
    assert!(
        stdout(&out).contains("snippet-region-missing"),
        "{}",
        stdout(&out)
    );
}

/// The acceptance criterion: with the network off, a project with two
/// remote sources checks, builds, diffs, and reports drift.
#[test]
fn every_other_command_works_offline_over_a_moved_pin() {
    let pair = Pair::new();
    pair.change_api();
    let out = pair.ascribe(&["sources", "update"]);
    assert_eq!(code_of(&out), 0, "{}", stderr(&out));
    commit(pair.docs(), "Update the sources");
    let api = hide(pair.api.path());
    let cli = hide(pair.cli.path());

    let docs = pair.docs();
    let out = offline(&pair, docs, &["check", "--deny-warnings"]);
    assert_eq!(code_of(&out), 0, "{}", stderr(&out));
    let out = offline(&pair, docs, &["build"]);
    assert_eq!(code_of(&out), 0, "{}", stderr(&out));
    let out = offline(&pair, docs, &["sources", "status"]);
    assert_eq!(code_of(&out), 0, "{}", stderr(&out));
    assert!(
        stdout(&out).contains("  src/auth.rs: current\n"),
        "{}",
        stdout(&out)
    );

    // The pull request that moved the pin, as diff and drift see it: the
    // copies at the base come from the docs repository's own history.
    let out = offline(&pair, docs, &["diff", "--base", "main", "--format", "json"]);
    assert_eq!(code_of(&out), 0, "{}", stderr(&out));
    let diff: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    let pages = diff["builds"][0]["pages"].as_array().unwrap();
    assert_eq!(pages.len(), 1, "{diff}");
    assert_eq!(pages[0]["path"], "auth.md");
    let out = offline(
        &pair,
        docs,
        &["drift", "--base", "main", "--format", "json"],
    );
    assert_eq!(code_of(&out), 0, "{}", stderr(&out));
    let drift: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(drift["pages"][0]["path"], "auth.md");
    assert_eq!(drift["pages"][0]["page_changed"], false);
    assert_eq!(
        drift["pages"][0]["examples"][0]["address"],
        "api:src/auth.rs#login"
    );

    // And this is offline: fetching what's missing fails, naming the source.
    fs::remove_file(docs.join("sources").join("cli").join("cmd").join("main.go")).unwrap();
    let out = offline(&pair, docs, &["sources", "fetch"]);
    assert_eq!(code_of(&out), 2, "{}", stdout(&out));
    assert!(
        stderr(&out).starts_with("error: source `cli`: git failed: "),
        "{}",
        stderr(&out)
    );
    fs::rename(api, pair.api.path()).unwrap();
    fs::rename(cli, pair.cli.path()).unwrap();
}

/// The same commit of the docs builds the same output anywhere, and moving
/// a source from `git` to `path` changes no page.
#[test]
fn the_same_commit_builds_the_same_output_and_path_or_git_is_the_same() {
    let pair = Pair::new();
    let out = offline(&pair, pair.docs(), &["build"]);
    assert_eq!(code_of(&out), 0, "{}", stderr(&out));
    let built = tree(&pair.docs().join(".ascribe").join("build"));
    assert!(built.keys().any(|k| k.ends_with("auth.md")), "{built:?}");

    let elsewhere = tempfile::tempdir().unwrap();
    let clone = elsewhere.path().join("somewhere").join("else");
    fs::create_dir_all(&clone).unwrap();
    git(
        &clone,
        &[
            "clone",
            "-q",
            "--branch",
            "feature",
            &file_url(pair.docs()),
            ".",
        ],
    );
    let out = offline(&pair, &clone, &["build"]);
    assert_eq!(code_of(&out), 0, "{}", stderr(&out));
    assert_eq!(tree(&clone.join(".ascribe").join("build")), built);

    // The code moves into this repository: ascribe.toml changes, and no page.
    let model = fs::read_to_string(clone.join("ascribe.toml")).unwrap();
    let url = file_url(pair.api.path());
    let model = model.replace(
        &format!("git = \"{url}\"\nbranch = \"main\""),
        "path = \"vendor/api\"",
    );
    fs::write(clone.join("ascribe.toml"), model).unwrap();
    write(&clone, "vendor/api/src/auth.rs", AUTH_V1);
    let out = pair
        .command(&clone, &["sources", "fetch"])
        .output()
        .unwrap();
    assert_eq!(code_of(&out), 0, "{}", stderr(&out));
    assert!(!clone.join("sources").join("api").exists());
    let out = offline(&pair, &clone, &["check", "--deny-warnings"]);
    assert_eq!(code_of(&out), 0, "{}", stderr(&out));
    let out = offline(&pair, &clone, &["build"]);
    assert_eq!(code_of(&out), 0, "{}", stderr(&out));
    // The pages are the same; the build records where the code file is.
    let after: BTreeMap<String, Vec<u8>> = tree(&clone.join(".ascribe").join("build"))
        .into_iter()
        .map(|(k, v)| {
            let text = String::from_utf8(v).unwrap();
            (
                k,
                text.replace("\"vendor/api/", "\"sources/api/").into_bytes(),
            )
        })
        .collect();
    assert_eq!(after, built);
}

/// Every file under `dir`, by path, with its bytes.
fn tree(dir: &Path) -> BTreeMap<String, Vec<u8>> {
    fn walk(dir: &Path, prefix: &str, out: &mut BTreeMap<String, Vec<u8>>) {
        for entry in fs::read_dir(dir).unwrap() {
            let entry = entry.unwrap();
            let name = format!("{prefix}{}", entry.file_name().to_string_lossy());
            if entry.file_type().unwrap().is_dir() {
                walk(&entry.path(), &format!("{name}/"), out);
            } else {
                out.insert(name, fs::read(entry.path()).unwrap());
            }
        }
    }
    let mut out = BTreeMap::new();
    walk(dir, "", &mut out);
    out
}

/// `check`, `build`, and `sources status` don't run `git` at all: a `git`
/// first on the path that records being run is never run.
#[cfg(unix)]
#[test]
fn check_build_and_status_never_run_git() {
    use std::os::unix::fs::PermissionsExt;

    let pair = Pair::new();
    let bin = tempfile::tempdir().unwrap();
    let ran = bin.path().join("ran");
    let fake = bin.path().join("git");
    fs::write(
        &fake,
        format!("#!/bin/sh\necho \"$@\" >> '{}'\nexit 1\n", ran.display()),
    )
    .unwrap();
    fs::set_permissions(&fake, fs::Permissions::from_mode(0o755)).unwrap();
    for args in [
        &["check", "--deny-warnings"][..],
        &["build"],
        &["sources", "status"],
    ] {
        let out = pair
            .command(pair.docs(), args)
            .env("PATH", bin.path())
            .output()
            .unwrap();
        assert_eq!(code_of(&out), 0, "{args:?}: {}", stderr(&out));
    }
    assert!(
        !ran.exists(),
        "git ran: {}",
        fs::read_to_string(&ran).unwrap_or_default()
    );
    // The fake is on the path: a command that does run `git` runs it.
    let out = pair
        .command(pair.docs(), &["drift", "--base", "main"])
        .env("PATH", bin.path())
        .output()
        .unwrap();
    assert_eq!(code_of(&out), 2);
    assert!(ran.exists());
}

#[test]
fn fetch_and_status_say_what_they_did() {
    let pair = Pair::new();
    fs::remove_file(
        pair.docs()
            .join("sources")
            .join("api")
            .join("src")
            .join("auth.rs"),
    )
    .unwrap();
    let out = pair.ascribe(&["sources", "status", "--format", "json"]);
    let status: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(status["sources"][0]["files"][0]["state"], "missing");
    let out = pair.ascribe(&["sources", "fetch"]);
    assert_eq!(code_of(&out), 0, "{}", stderr(&out));
    assert_eq!(stdout(&out), "api: copied src/auth.rs\n");
    let out = pair.ascribe(&["sources", "fetch"]);
    assert_eq!(stdout(&out), "The copies already match ascribe.lock.\n");
    let out = pair.ascribe(&["sources", "fetch", "web"]);
    assert_eq!(code_of(&out), 2);
    assert_eq!(stderr(&out), "error: ascribe.toml has no source `web`\n");
    let out = pair.ascribe(&["sources", "update", "--to", "main"]);
    assert_eq!(code_of(&out), 2);
    assert_eq!(
        stderr(&out),
        "error: --to moves one source's pin: name the source\n"
    );
}
