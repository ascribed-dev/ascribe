//! A project's files are read through `FileSystem` (`src/fs.rs`), which knows
//! the content root, the boundary, exact-case names, and symbolic links. This
//! test finds every other place in the crates' source that reads the disk,
//! and fails unless the code says why it isn't a project read: a comment
//! containing `Outside FileSystem:` in the same paragraph (no blank line
//! between the comment and the read), or the file is listed below.
//!
//! Unit tests (everything after a file's first top-level `#[cfg(test)]`) and
//! the fork of comrak aren't checked.

#![allow(clippy::expect_used, clippy::panic)]

use std::fs;
use std::path::{Path, PathBuf};

/// What reads the disk.
const READS: &[&str] = &[
    "fs::read(",
    "fs::read_to_string(",
    "fs::read_dir(",
    "fs::metadata(",
    "fs::symlink_metadata(",
    "fs::canonicalize(",
    "fs::read_link(",
    "File::open(",
    ".is_file()",
    ".is_dir()",
    ".exists()",
    ".try_exists()",
    ".canonicalize()",
    ".read_dir()",
    ".metadata()",
    ".symlink_metadata()",
    ".read_link()",
];

/// What a read outside `FileSystem` says about itself.
const MARKER: &str = "Outside FileSystem:";

/// Files that read the disk throughout, and why.
const FILES: &[(&str, &str)] = &[
    ("crates/tessera-resolve/src/fs.rs", "it's `FileSystem`"),
    (
        "crates/tessera-emit/src/store.rs",
        "it writes the output directory, and reads what's there, its manifest, \
         and the asset files it's handed to copy (from `EmitContext::asset_source`)",
    ),
    (
        "crates/tessera-cli/src/docs.rs",
        "a test, compiled only with `cfg(test)`, that reads this repository's docs",
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

/// Every read in the crates' source, as `path:line`, with no reason given.
fn unexplained(root: &Path) -> Vec<String> {
    let mut files = Vec::new();
    for entry in fs::read_dir(root.join("crates")).expect("crates/") {
        let krate = entry.expect("a crate").path();
        if krate.file_name().is_some_and(|n| n == "comrak-tessera") {
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
            if code.starts_with("//") || !READS.iter().any(|read| line.contains(read)) {
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
fn every_read_outside_file_system_says_why() {
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
        "These read the disk without going through tessera_resolve::FileSystem. \
         Read a project's files through it, or say why this isn't a project read \
         with a comment containing `{MARKER}` just above:\n{}",
        found.join("\n")
    );
}
