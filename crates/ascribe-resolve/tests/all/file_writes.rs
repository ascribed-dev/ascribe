//! A library returns what to write, and its caller writes it: `check --fix`
//! gets each file's new text, and `ascribe report` hands the link cache and
//! the histories an `ascribe_check::store::Store`. This test finds every
//! other place in the library crates' source that writes to the disk, and
//! fails unless the code says why it writes there itself: a comment
//! containing `Library write:` in the same paragraph (no blank line between
//! the comment and the write), or the file is listed below.
//!
//! `ascribe-cli` is the caller, so it isn't checked. Neither are unit tests
//! (everything after a file's first top-level `#[cfg(test)]`) or the fork
//! of comrak.

#![allow(clippy::expect_used, clippy::panic)]

use std::fs;
use std::path::{Path, PathBuf};

/// What writes to the disk.
const WRITES: &[&str] = &[
    "fs::write(",
    "fs::create_dir(",
    "fs::create_dir_all(",
    "fs::remove_file(",
    "fs::remove_dir(",
    "fs::remove_dir_all(",
    "fs::rename(",
    "fs::copy(",
    "fs::hard_link(",
    "fs::set_permissions(",
    "File::create(",
    "OpenOptions::new(",
    "fs::symlink(",
    "symlink_file(",
    "symlink_dir(",
];

/// What a write in a library says about itself.
const MARKER: &str = "Library write:";

/// The crate that writes what the libraries return.
const CALLER: &str = "ascribe-cli";

/// Files that write the disk throughout, and why.
const FILES: &[(&str, &str)] = &[
    (
        "crates/ascribe-emit/src/store.rs",
        "it's the output directory: a build is written there whole, through a \
         staging folder, or not at all",
    ),
    (
        "crates/ascribe-sources/src/copies.rs",
        "it writes the copies of sources in other repositories, and removes \
         the ones no longer named",
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

/// Every write in the library crates' source, as `path:line`, with no
/// reason given.
fn unexplained(root: &Path) -> Vec<String> {
    let mut files = Vec::new();
    for entry in fs::read_dir(root.join("crates")).expect("crates/") {
        let krate = entry.expect("a crate").path();
        if krate
            .file_name()
            .is_some_and(|n| n == "comrak-ascribe" || n == CALLER)
        {
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
        let rel = file
            .strip_prefix(root)
            .expect("in the workspace")
            .to_string_lossy()
            .replace('\\', "/");
        if FILES.iter().any(|(path, _)| *path == rel) {
            continue;
        }
        let text = fs::read_to_string(&file).expect("a source file");
        let lines: Vec<&str> = text.lines().collect();
        for (i, line) in lines.iter().enumerate() {
            if line.starts_with("#[cfg(test)]") {
                break;
            }
            let code = line.trim_start();
            if code.starts_with("//") || !WRITES.iter().any(|write| line.contains(write)) {
                continue;
            }
            let paragraph = lines[..=i]
                .iter()
                .rev()
                .take_while(|l| !l.trim().is_empty());
            if !paragraph.into_iter().any(|l| l.contains(MARKER)) {
                out.push(format!("{rel}:{}: {}", i + 1, code));
            }
        }
    }
    out
}

#[test]
fn every_library_write_says_why() {
    let root = workspace();
    for (path, _) in FILES {
        assert!(
            root.join(path).is_file(),
            "{path} is listed but isn't there"
        );
    }
    let found = unexplained(&root);
    assert!(
        found.is_empty(),
        "These library crates write to the disk themselves. Return what to \
         write and let the caller write it, as `check --fix` does, or take an \
         `ascribe_check::store::Store` from the caller, as the link cache does; \
         or say why the library must write here with a comment containing \
         `{MARKER}` just above:\n{}",
        found.join("\n")
    );
}
