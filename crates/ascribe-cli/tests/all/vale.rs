//! `ascribe vale eject`, by running the binary in a temporary project.

#![allow(clippy::expect_used, clippy::unwrap_used)]

use std::fs;
use std::path::Path;
use std::process::{Command, Output};

use serde_json::Value;
use tempfile::TempDir;

const MODEL: &str =
    "spec = \"0.1\"\n\n[project]\ncontent-root = \"docs\"\n\n[checks.vale]\npreset = \"quiet\"\n";

fn project() -> TempDir {
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join("ascribe.toml"), MODEL).unwrap();
    fs::create_dir(dir.path().join("docs")).unwrap();
    dir
}

fn ascribe(dir: &Path, args: &[&str]) -> Output {
    // The folder isn't a repository, so eject passes the git guard with
    // `--force`; `guard.rs` tests the guard.
    let writes = args.contains(&"eject") && !args.contains(&"--dry-run");
    Command::new(env!("CARGO_BIN_EXE_ascribe"))
        .current_dir(dir)
        .args(args)
        .args(writes.then_some("--force"))
        .env("NO_COLOR", "1")
        .output()
        .expect("run ascribe")
}

fn code_of(output: &Output) -> i32 {
    output.status.code().expect("exited with a code")
}

/// The text and the JSON list the same files, `ascribe.toml` last.
#[test]
fn eject_says_what_it_wrote_in_text_and_json() {
    let text = project();
    let out = ascribe(text.path(), &["vale", "eject"]);
    assert_eq!(code_of(&out), 0, "{}", String::from_utf8_lossy(&out.stderr));
    let text = String::from_utf8_lossy(&out.stdout).into_owned();
    let wrote: Vec<&str> = text
        .lines()
        .filter_map(|line| line.strip_prefix("wrote "))
        .collect();
    assert_eq!(wrote.last(), Some(&"ascribe.toml"), "{text}");
    assert!(
        text.contains("The `quiet` preset is now the project's own"),
        "{text}"
    );

    let json = project();
    let out = ascribe(json.path(), &["vale", "eject", "--format", "json"]);
    assert_eq!(code_of(&out), 0, "{}", String::from_utf8_lossy(&out.stderr));
    let ejected: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(ejected["schema_version"], 1);
    assert_eq!(ejected["preset"], "quiet");
    let written: Vec<&str> = ejected["written"]
        .as_array()
        .unwrap()
        .iter()
        .map(|path| path.as_str().unwrap())
        .collect();
    assert_eq!(written, wrote);
    assert!(
        fs::read_to_string(json.path().join("ascribe.toml"))
            .unwrap()
            .contains("config = \".vale.ini\"")
    );
}
