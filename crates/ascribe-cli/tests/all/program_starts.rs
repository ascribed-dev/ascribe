//! Every place the workspace starts a program, listed with why, as
//! `crates/ascribe-resolve/tests/all/file_reads.rs` lists direct file reads.
//! A file starts a program when it uses `std::process::Command` and calls
//! `Command::new(`; a new one fails this test until it's listed here. Only
//! `ascribe observe` runs a line of shell, and only in the binary: a library
//! crate that starts one fails it.
//!
//! Unit tests (everything after a file's first top-level `#[cfg(test)]`) and
//! the fork of comrak aren't checked.

#![allow(clippy::expect_used, clippy::panic)]

use std::fs;
use std::path::{Path, PathBuf};

/// The files that start a program, and what each starts.
const STARTS: &[(&str, &str)] = &[
    (
        "crates/ascribe-check/src/tool.rs",
        "the programs a report runs and reads the answers of (Vale, lychee, afdocs), with a fixed argument list",
    ),
    (
        "crates/ascribe-diff/src/git.rs",
        "`git`, with arguments it builds",
    ),
    (
        "crates/ascribe-sources/src/remote.rs",
        "`git`, with arguments it builds",
    ),
    (
        "crates/ascribe-cli/src/agents/checker.rs",
        "the hook's check server: this binary, started again",
    ),
    (
        "crates/ascribe-cli/src/commands/observe.rs",
        "a line of shell: the `test` and `[environment]` commands of the ascribe.examples.toml in the folder `ascribe observe` is run in, and nothing else",
    ),
];

fn workspace() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("the workspace")
}

fn rust_files(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(dir).expect("a directory") {
        let path = entry.expect("an entry").path();
        if path.is_dir() {
            rust_files(&path, out);
        } else if path.extension().is_some_and(|e| e == "rs") {
            out.push(path);
        }
    }
}

/// The files in the crates' source that start a program, from the
/// workspace's root.
fn starting(root: &Path) -> Vec<String> {
    let mut files = Vec::new();
    for entry in fs::read_dir(root.join("crates")).expect("crates/") {
        let krate = entry.expect("a crate").path();
        if krate.file_name().is_some_and(|n| n == "comrak-ascribe") {
            continue;
        }
        let src = krate.join("src");
        if src.is_dir() {
            rust_files(&src, &mut files);
        }
    }
    files.sort();
    let mut out = Vec::new();
    for file in files {
        let text = fs::read_to_string(&file).expect("a source file");
        let code: Vec<&str> = text
            .lines()
            .take_while(|l| !l.starts_with("#[cfg(test)]"))
            .filter(|l| !l.trim_start().starts_with("//"))
            .collect();
        // A `process` module's `Command`, the standard library's or
        // another's (`tokio::process`), imported or written out
        // (`process::Command::new`). Another type named `Command`, as the
        // commands' own, isn't one.
        let uses_process = code.iter().any(|l| {
            l.contains("process::Command") || (l.contains("process::{") && l.contains("Command"))
        });
        if uses_process && code.iter().any(|l| l.contains("Command::new(")) {
            let rel = file
                .strip_prefix(root)
                .expect("in the workspace")
                .to_string_lossy()
                .replace('\\', "/");
            out.push(rel);
        }
    }
    out
}

#[test]
fn every_program_start_is_listed() {
    let root = workspace();
    let found = starting(&root);
    let listed: Vec<&str> = STARTS.iter().map(|(path, _)| *path).collect();
    let unlisted: Vec<&String> = found
        .iter()
        .filter(|f| !listed.contains(&f.as_str()))
        .collect();
    assert!(
        unlisted.is_empty(),
        "These start a program and aren't listed in STARTS. Only `ascribe observe` \
         runs a line of shell, and a library never does; list a new start with why:\n{unlisted:?}"
    );
    let stale: Vec<&&str> = listed
        .iter()
        .filter(|l| !found.iter().any(|f| f == *l))
        .collect();
    assert!(
        stale.is_empty(),
        "listed, but they start nothing: {stale:?}"
    );
}

#[test]
fn only_the_binary_runs_a_line_of_shell() {
    let root = workspace();
    for file in starting(&root) {
        let text = fs::read_to_string(root.join(&file)).expect("a source file");
        let shell = ["\"sh\"", "\"cmd\"", "\"bash\"", "\"powershell"]
            .iter()
            .any(|s| text.contains(s));
        if shell {
            assert!(
                file.starts_with("crates/ascribe-cli/"),
                "{file} starts a shell, and only the binary may"
            );
        }
    }
}

/// The test can tell: a library file that starts a program is found.
#[test]
fn a_library_start_is_found() {
    let dir = tempfile::tempdir().expect("a folder");
    let src = dir.path().join("crates/ascribe-x/src");
    fs::create_dir_all(&src).expect("a crate");
    fs::write(
        src.join("lib.rs"),
        "use std::process::Command;\nfn f() { let _ = Command::new(\"sh\"); }\n",
    )
    .expect("a file");
    assert_eq!(starting(dir.path()), ["crates/ascribe-x/src/lib.rs"]);
}
