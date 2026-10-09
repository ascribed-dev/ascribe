//! `ascribe agents hook`, by running the binary with each harness's hook
//! input on a small project in a temporary folder. Most tests check in the
//! hook's own process (`ASCRIBE_HOOK_SERVER=off`); `the_check_server_*`
//! goes through the server, with its file in a temporary cache folder.

#![allow(clippy::expect_used, clippy::unwrap_used, clippy::panic)]

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

use serde_json::{Value, json};
use tempfile::TempDir;

const MODEL: &str = "spec = \"0.1\"\n\n[project]\ncontent-root = \"docs\"\n";
const CLEAN: &str = "---\ntitle: Home\n---\n\nSee [the guide](guide.md).\n";
const ERROR: &str = "---\ntitle: Guide\n---\n\nSee [nothing](missing.md).\n";
const WARNING: &str = "---\ntitle: Notes\n---\n\nSee {nothere}.\n";

/// A project in a git repository, with a clean page, a page with an
/// error, and a page with a warning, all committed.
fn project() -> TempDir {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    fs::write(root.join("ascribe.toml"), MODEL).unwrap();
    fs::create_dir_all(root.join("docs")).unwrap();
    fs::write(root.join("docs/index.md"), CLEAN).unwrap();
    fs::write(root.join("docs/guide.md"), ERROR).unwrap();
    fs::write(root.join("docs/notes.md"), WARNING).unwrap();
    fs::write(root.join("README.md"), "# Not a page\n").unwrap();
    fs::write(root.join("docs/script.js"), "let x;\n").unwrap();
    git(root, &["init", "-q"]);
    git(root, &["add", "."]);
    git(
        root,
        &[
            "-c",
            "user.name=t",
            "-c",
            "user.email=t@example.com",
            "commit",
            "-qm",
            "start",
        ],
    );
    dir
}

fn git(dir: &Path, args: &[&str]) {
    let out = Command::new("git")
        .current_dir(dir)
        .args(args)
        .output()
        .expect("run git");
    assert!(out.status.success(), "git {args:?} failed");
}

/// Runs the hook with `input` on standard input, checking in its own
/// process, or with `cache`, through the check server.
fn hook(args: &[&str], input: &str, cache: Option<&Path>) -> Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_ascribe"));
    command
        .args(["agents", "hook"])
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    match cache {
        Some(cache) => command
            .env("ASCRIBE_CACHE_DIR", cache)
            .env_remove("ASCRIBE_HOOK_SERVER"),
        None => command.env("ASCRIBE_HOOK_SERVER", "off"),
    };
    let mut child = command.spawn().unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(input.as_bytes())
        .unwrap();
    child.wait_with_output().unwrap()
}

fn stdout(out: &Output) -> String {
    String::from_utf8_lossy(&out.stdout).into_owned()
}

/// An absolute path, as a harness gives it, with the platform's
/// separators.
fn path(root: &Path, rel: &str) -> String {
    rel.split('/')
        .fold(root.to_path_buf(), |p, s| p.join(s))
        .display()
        .to_string()
}

/// Each harness's input after its tool wrote `file`.
fn edit_input(harness: &str, root: &Path, file: &str) -> String {
    let cwd = root.display().to_string();
    let file = path(root, file);
    match harness {
        "claude-code" => json!({
            "session_id": "s",
            "cwd": cwd,
            "hook_event_name": "PostToolUse",
            "tool_name": "Edit",
            "tool_input": { "file_path": file, "old_string": "a", "new_string": "b" },
            "tool_response": { "filePath": file },
        }),
        "codex" => json!({
            "session_id": "s",
            "cwd": cwd,
            "hook_event_name": "PostToolUse",
            "tool_name": "apply_patch",
            "tool_input": {
                "command": format!("*** Begin Patch\n*** Update File: {file}\n@@\n-a\n+b\n*** End Patch\n"),
            },
        }),
        "copilot" => json!({
            "sessionId": "s",
            "timestamp": 1,
            "cwd": cwd,
            "toolName": "edit",
            "toolArgs": json!({ "path": file, "old_str": "a", "new_str": "b" }).to_string(),
            "toolResult": { "resultType": "success", "textResultForLlm": "ok" },
        }),
        _ => unreachable!(),
    }
    .to_string()
}

fn stop_input(harness: &str, root: &Path, active: bool) -> String {
    let cwd = root.display().to_string();
    match harness {
        "copilot" => json!({
            "sessionId": "s",
            "timestamp": 1,
            "cwd": cwd,
            "stopReason": "end_turn",
            "stop_hook_active": active,
        }),
        _ => json!({
            "session_id": "s",
            "cwd": cwd,
            "hook_event_name": "Stop",
            "stop_hook_active": active,
        }),
    }
    .to_string()
}

const HARNESSES: [&str; 3] = ["claude-code", "codex", "copilot"];

/// What the harness reads after an edit: its `additionalContext`.
fn context(harness: &str, out: &Output) -> String {
    let value: Value = serde_json::from_str(&stdout(out)).unwrap();
    let context = match harness {
        "copilot" => &value["additionalContext"],
        _ => {
            assert_eq!(value["hookSpecificOutput"]["hookEventName"], "PostToolUse");
            &value["hookSpecificOutput"]["additionalContext"]
        }
    };
    context.as_str().unwrap().to_owned()
}

/// What the hook writes when it has nothing to say.
fn quiet(harness: &str, event: &str) -> &'static str {
    if harness == "codex" && event == "stop" {
        "{}\n"
    } else {
        ""
    }
}

fn assert_quiet(harness: &str, event: &str, out: &Output) {
    assert_eq!(out.status.code(), Some(0), "{harness}");
    assert_eq!(stdout(out), quiet(harness, event), "{harness}");
    assert!(
        out.stderr.is_empty(),
        "{harness}: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

#[test]
fn an_edit_with_an_error_tells_the_agent() {
    let dir = project();
    for harness in HARNESSES {
        let out = hook(
            &[harness],
            &edit_input(harness, dir.path(), "docs/guide.md"),
            None,
        );
        assert_eq!(out.status.code(), Some(0), "{harness}");
        assert!(out.stderr.is_empty(), "{harness}");
        assert_eq!(
            context(harness, &out),
            "`ascribe check docs/guide.md` reports 1 error:\n\
             docs/guide.md:5: [ASC036] `missing.md` doesn't exist\n\
             Checked build `site` only. Run `ascribe explain <code>` for any you don't recognize.\n",
            "{harness}"
        );
    }
}

#[test]
fn an_edit_with_no_errors_says_nothing() {
    let dir = project();
    for harness in HARNESSES {
        for file in ["docs/index.md", "docs/notes.md"] {
            let out = hook(&[harness], &edit_input(harness, dir.path(), file), None);
            assert_quiet(harness, "edit", &out);
        }
    }
}

#[test]
fn an_edit_outside_a_page_says_nothing() {
    let dir = project();
    let outside = tempfile::tempdir().unwrap();
    fs::write(outside.path().join("notes.md"), "[x](missing.md)\n").unwrap();
    for harness in HARNESSES {
        for (root, file) in [
            (outside.path(), "notes.md"),
            (dir.path(), "README.md"),
            (dir.path(), "docs/script.js"),
            // Deleted, or never there.
            (dir.path(), "docs/gone.md"),
        ] {
            let out = hook(&[harness], &edit_input(harness, root, file), None);
            assert_quiet(harness, "edit", &out);
        }
        // A tool that writes no file.
        let out = hook(
            &[harness],
            r#"{"tool_name": "Bash", "tool_input": {"command": "ls"}}"#,
            None,
        );
        assert_quiet(harness, "edit", &out);
    }
}

#[test]
fn malformed_input_is_for_the_user() {
    for harness in HARNESSES {
        for input in ["", "not json", "[1, 2]"] {
            let out = hook(&[harness], input, None);
            assert_eq!(out.status.code(), Some(1), "{harness}");
            assert!(out.stdout.is_empty(), "{harness}");
            assert!(
                String::from_utf8_lossy(&out.stderr)
                    .starts_with("ascribe agents hook: can't read the hook's input:"),
                "{harness}"
            );
        }
    }
}

#[test]
fn a_stop_with_nothing_changed_says_nothing() {
    // The project has an error, but the agent changed nothing.
    let dir = project();
    for harness in HARNESSES {
        let out = hook(
            &[harness, "--event", "stop"],
            &stop_input(harness, dir.path(), false),
            None,
        );
        assert_quiet(harness, "stop", &out);
    }
}

#[test]
fn a_stop_with_errors_keeps_the_agent_working_once() {
    let dir = project();
    fs::write(
        dir.path().join("docs/index.md"),
        format!("{CLEAN}\nAnd [more](gone.md).\n"),
    )
    .unwrap();
    for harness in HARNESSES {
        let out = hook(
            &[harness, "--event", "stop"],
            &stop_input(harness, dir.path(), false),
            None,
        );
        assert_eq!(out.status.code(), Some(0), "{harness}");
        let value: Value = serde_json::from_str(&stdout(&out)).unwrap();
        assert_eq!(value["decision"], "block", "{harness}");
        assert_eq!(
            value["reason"],
            "`ascribe check` reports 2 errors and 1 warning:\n\
             docs/guide.md:5: [ASC036] `missing.md` doesn't exist\n\
             docs/index.md:7: [ASC036] `gone.md` doesn't exist\n\
             Fix these, then run `ascribe check`.\n",
            "{harness}"
        );
        // Already continuing because of it: let the agent stop.
        let out = hook(
            &[harness, "--event", "stop"],
            &stop_input(harness, dir.path(), true),
            None,
        );
        assert_quiet(harness, "stop", &out);
    }
}

#[test]
fn a_stop_with_no_errors_says_nothing() {
    let dir = project();
    fs::write(
        dir.path().join("docs/guide.md"),
        CLEAN.replace("guide.md", "index.md"),
    )
    .unwrap();
    fs::write(dir.path().join("docs/new.md"), WARNING).unwrap();
    for harness in HARNESSES {
        let out = hook(
            &[harness, "--event", "stop"],
            &stop_input(harness, dir.path(), false),
            None,
        );
        assert_quiet(harness, "stop", &out);
    }
}

#[test]
fn a_stop_outside_a_repository_checks_the_project_there() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join("ascribe.toml"), MODEL).unwrap();
    fs::create_dir_all(dir.path().join("docs")).unwrap();
    fs::write(dir.path().join("docs/guide.md"), ERROR).unwrap();
    let out = hook(
        &["claude-code", "--event", "stop"],
        &stop_input("claude-code", dir.path(), false),
        None,
    );
    let value: Value = serde_json::from_str(&stdout(&out)).unwrap();
    assert_eq!(value["decision"], "block");
}

#[test]
fn the_check_server_answers_and_stops_with_its_project() {
    let dir = project();
    let cache = tempfile::tempdir().unwrap();
    let input = edit_input("claude-code", dir.path(), "docs/guide.md");
    // The first hook starts the server, which may not answer in time.
    let mut answered = None;
    for _ in 0..20 {
        let out = hook(&["claude-code"], &input, Some(cache.path()));
        assert_eq!(out.status.code(), Some(0));
        if !out.stdout.is_empty() {
            answered = Some(out);
            break;
        }
    }
    let out = answered.expect("the check server answered");
    assert!(context("claude-code", &out).contains("[ASC036] `missing.md` doesn't exist"));
    let files: Vec<PathBuf> = fs::read_dir(cache.path().join("hooks"))
        .unwrap()
        .map(|e| e.unwrap().path())
        .collect();
    assert_eq!(files.len(), 1, "{files:?}");
    // An edit the server sees: fixed, nothing to say.
    fs::write(
        dir.path().join("docs/guide.md"),
        CLEAN.replace("guide.md", "index.md"),
    )
    .unwrap();
    assert_quiet(
        "claude-code",
        "edit",
        &hook(&["claude-code"], &input, Some(cache.path())),
    );
    // With its project gone, it stops and removes its file.
    drop(dir);
    for _ in 0..100 {
        if !files[0].exists() {
            return;
        }
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
    panic!("the check server didn't stop");
}
