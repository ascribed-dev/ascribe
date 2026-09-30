//! Replacing a previous output (output-layout contract, §4).

#![allow(clippy::expect_used, clippy::panic)]

use std::fs;
use std::path::Path;

use tessera_core::RelPath;
use tessera_emit::{Contents, EmittedFile, FileKind, OutputDir, StoreError};

fn page(path: &str, text: &str) -> EmittedFile {
    EmittedFile {
        path: RelPath::parse(path).expect("a path"),
        kind: FileKind::Page,
        source: Some(RelPath::parse(path).expect("a path")),
        url: None,
        contents: Contents::Text(text.to_owned()),
    }
}

fn write(path: &Path, text: &str) {
    fs::create_dir_all(path.parent().expect("a parent")).expect("create directories");
    fs::write(path, text).expect("write a file");
}

fn read(path: &Path) -> String {
    fs::read_to_string(path).expect("read a file")
}

fn replace(
    dir: &Path,
    build: &str,
    files: &[EmittedFile],
) -> Result<tessera_emit::Replaced, StoreError> {
    OutputDir::lock(dir)?.replace(build, "plain", files)
}

#[test]
fn a_first_build_writes_its_files_and_a_manifest() {
    let dir = tempfile::tempdir().expect("a temporary directory");
    let out = dir.path().join("out");
    let replaced =
        replace(&out, "site", &[page("a.md", "A"), page("g/b.md", "B")]).expect("replaced");
    assert_eq!(
        (replaced.written, replaced.unchanged, replaced.removed),
        (2, 0, 0)
    );
    assert_eq!(read(&out.join("site/plain/a.md")), "A");
    assert_eq!(read(&out.join("site/plain/g/b.md")), "B");
    let manifest: serde_json::Value =
        serde_json::from_str(&read(&out.join("site/plain.manifest.json"))).expect("JSON");
    assert_eq!(manifest["format"], "ascribe-manifest");
    assert_eq!(manifest["version"], 1);
    assert_eq!(manifest["build"], "site");
    assert_eq!(manifest["emitter"], "plain");
    let paths: Vec<_> = manifest["files"]
        .as_array()
        .expect("files")
        .iter()
        .map(|f| f["path"].as_str().expect("a path"))
        .collect();
    assert_eq!(paths, ["a.md", "g/b.md"]);
    // Staging is gone; the lock file stays.
    assert!(!out.join(".staging").exists());
}

#[test]
fn a_second_build_removes_what_it_no_longer_produces_and_keeps_the_users_files() {
    let dir = tempfile::tempdir().expect("a temporary directory");
    let out = dir.path().join("out");
    replace(
        &out,
        "site",
        &[page("a.md", "A"), page("g/b.md", "B"), page("g/c.md", "C")],
    )
    .expect("first");
    write(&out.join("site/plain/notes.txt"), "mine");
    write(&out.join("site/plain/g/mine.txt"), "mine too");
    let replaced = replace(&out, "site", &[page("a.md", "A2")]).expect("second");
    assert_eq!(
        (replaced.written, replaced.unchanged, replaced.removed),
        (1, 0, 2)
    );
    assert_eq!(read(&out.join("site/plain/a.md")), "A2");
    assert!(!out.join("site/plain/g/b.md").exists());
    assert!(!out.join("site/plain/g/c.md").exists());
    assert_eq!(read(&out.join("site/plain/notes.txt")), "mine");
    // The directory still holds a user's file, so it stays.
    assert_eq!(read(&out.join("site/plain/g/mine.txt")), "mine too");
}

#[test]
fn a_directory_that_becomes_empty_is_removed() {
    let dir = tempfile::tempdir().expect("a temporary directory");
    let out = dir.path().join("out");
    replace(&out, "site", &[page("a.md", "A"), page("g/deep/b.md", "B")]).expect("first");
    replace(&out, "site", &[page("a.md", "A")]).expect("second");
    assert!(!out.join("site/plain/g").exists());
    assert!(out.join("site/plain").is_dir());
}

#[test]
fn unchanged_files_are_left_alone() {
    let dir = tempfile::tempdir().expect("a temporary directory");
    let out = dir.path().join("out");
    replace(&out, "site", &[page("a.md", "A"), page("b.md", "B")]).expect("first");
    let modified = |name: &str| {
        fs::metadata(out.join("site/plain").join(name))
            .and_then(|m| m.modified())
            .expect("a modification time")
    };
    let a_before = modified("a.md");
    let replaced = replace(&out, "site", &[page("a.md", "A"), page("b.md", "B2")]).expect("second");
    assert_eq!((replaced.written, replaced.unchanged), (1, 1));
    // The unchanged file is the same file, never rewritten (phase 26, P4).
    assert_eq!(modified("a.md"), a_before);
    assert_eq!(read(&out.join("site/plain/b.md")), "B2");
    // Same length, different bytes: written.
    let replaced = replace(&out, "site", &[page("a.md", "A"), page("b.md", "B3")]).expect("third");
    assert_eq!((replaced.written, replaced.unchanged), (1, 1));
    assert_eq!(read(&out.join("site/plain/b.md")), "B3");
}

#[test]
fn a_users_file_in_the_way_fails_the_build_and_changes_nothing() {
    let dir = tempfile::tempdir().expect("a temporary directory");
    let out = dir.path().join("out");
    replace(&out, "site", &[page("a.md", "A"), page("old.md", "OLD")]).expect("first");
    let manifest_before = read(&out.join("site/plain.manifest.json"));
    write(&out.join("site/plain/new.md"), "the user's");
    let err =
        replace(&out, "site", &[page("a.md", "A2"), page("new.md", "N")]).expect_err("a conflict");
    let StoreError::Conflicts(list) = &err else {
        panic!("wrong error: {err}");
    };
    assert_eq!(list.len(), 1, "{err}");
    assert!(
        list[0].contains("new.md") && list[0].contains("isn't a file Ascribe wrote"),
        "{err}"
    );
    // Everything is as it was.
    assert_eq!(read(&out.join("site/plain/new.md")), "the user's");
    assert_eq!(read(&out.join("site/plain/a.md")), "A");
    assert_eq!(read(&out.join("site/plain/old.md")), "OLD");
    assert_eq!(read(&out.join("site/plain.manifest.json")), manifest_before);
    assert!(!out.join(".staging").exists());
}

#[test]
fn a_directory_where_a_file_goes_and_a_file_where_a_directory_goes_fail() {
    let dir = tempfile::tempdir().expect("a temporary directory");
    let out = dir.path().join("out");
    write(&out.join("site/plain/a.md/keep.txt"), "x");
    write(&out.join("site/plain/g"), "a user's file");
    let err =
        replace(&out, "site", &[page("a.md", "A"), page("g/b.md", "B")]).expect_err("conflicts");
    let text = err.to_string();
    assert!(text.contains("a.md is a directory"), "{text}");
    assert!(
        text.contains("isn't Ascribe's, and a directory is needed"),
        "{text}"
    );
    assert!(out.join("site/plain/a.md/keep.txt").exists());
    assert_eq!(read(&out.join("site/plain/g")), "a user's file");
}

#[test]
fn colliding_paths_fail() {
    let dir = tempfile::tempdir().expect("a temporary directory");
    let out = dir.path().join("out");
    let err =
        replace(&out, "site", &[page("A.md", "1"), page("a.md", "2")]).expect_err("a collision");
    assert!(err.to_string().contains("differ only in case"), "{err}");
    let err =
        replace(&out, "site", &[page("a.md", "1"), page("a.md", "2")]).expect_err("a duplicate");
    assert!(
        err.to_string()
            .contains("two files would be written at a.md"),
        "{err}"
    );
    assert!(!out.join("site").exists());
}

#[test]
fn something_that_isnt_a_manifest_at_the_manifest_path_fails() {
    let dir = tempfile::tempdir().expect("a temporary directory");
    let out = dir.path().join("out");
    write(&out.join("site/plain.manifest.json"), "{\"hello\": 1}");
    let err = replace(&out, "site", &[page("a.md", "A")]).expect_err("not a manifest");
    assert!(matches!(err, StoreError::NotManifest { .. }), "{err}");
    write(&out.join("site/plain.manifest.json"), "not json");
    let err = replace(&out, "site", &[page("a.md", "A")]).expect_err("not a manifest");
    assert!(matches!(err, StoreError::NotManifest { .. }), "{err}");
    assert!(!out.join("site/plain").exists());
}

#[test]
fn a_manifest_that_lists_a_path_outside_the_root_is_refused() {
    let dir = tempfile::tempdir().expect("a temporary directory");
    let out = dir.path().join("out");
    replace(&out, "site", &[page("a.md", "A")]).expect("first");
    write(&dir.path().join("precious.txt"), "keep me");
    let manifest = "{\"format\":\"ascribe-manifest\",\"version\":1,\"build\":\"site\",\"emitter\":\"plain\",\"files\":[{\"path\":\"../../../precious.txt\",\"kind\":\"page\"}]}";
    write(&out.join("site/plain.manifest.json"), manifest);
    let err = replace(&out, "site", &[page("a.md", "A")]).expect_err("a bad entry");
    assert!(matches!(err, StoreError::BadEntry { .. }), "{err}");
    assert_eq!(read(&dir.path().join("precious.txt")), "keep me");
}

#[test]
fn a_manifest_of_an_unknown_version_is_refused() {
    let dir = tempfile::tempdir().expect("a temporary directory");
    let out = dir.path().join("out");
    let manifest = "{\"format\":\"ascribe-manifest\",\"version\":2,\"build\":\"site\",\"emitter\":\"plain\",\"files\":[]}";
    write(&out.join("site/plain.manifest.json"), manifest);
    let err = replace(&out, "site", &[page("a.md", "A")]).expect_err("unknown version");
    assert!(
        matches!(err, StoreError::UnknownVersion { version: 2, .. }),
        "{err}"
    );
}

#[test]
fn a_file_of_the_previous_output_can_become_a_directory() {
    let dir = tempfile::tempdir().expect("a temporary directory");
    let out = dir.path().join("out");
    replace(&out, "site", &[page("a", "a file")]).expect("first");
    replace(&out, "site", &[page("a/b.md", "B")]).expect("second");
    assert_eq!(read(&out.join("site/plain/a/b.md")), "B");
}

#[test]
fn each_build_and_emitter_is_replaced_on_its_own() {
    let dir = tempfile::tempdir().expect("a temporary directory");
    let out = dir.path().join("out");
    let output = OutputDir::lock(&out).expect("locked");
    output
        .replace("one", "plain", &[page("a.md", "1")])
        .expect("one");
    output
        .replace("two", "plain", &[page("a.md", "2")])
        .expect("two");
    output
        .replace("one", "json", &[page("a.json", "{}")])
        .expect("json");
    output
        .replace("one", "plain", &[page("b.md", "1b")])
        .expect("one again");
    assert!(!out.join("one/plain/a.md").exists());
    assert_eq!(read(&out.join("one/plain/b.md")), "1b");
    assert_eq!(read(&out.join("two/plain/a.md")), "2");
    assert_eq!(read(&out.join("one/json/a.json")), "{}");
}

#[test]
fn assets_are_copied_byte_for_byte() {
    let dir = tempfile::tempdir().expect("a temporary directory");
    let out = dir.path().join("out");
    let source = dir.path().join("logo.bin");
    fs::write(&source, [0u8, 159, 146, 150, 255]).expect("write");
    let file = EmittedFile {
        path: RelPath::parse("_ascribe/up/logo.bin").expect("a path"),
        kind: FileKind::Asset,
        source: Some(RelPath::parse("../logo.bin").expect("a path")),
        url: None,
        contents: Contents::Copy(source),
    };
    replace(&out, "site", &[file]).expect("replaced");
    assert_eq!(
        fs::read(out.join("site/plain/_ascribe/up/logo.bin")).expect("read"),
        [0u8, 159, 146, 150, 255]
    );
    let manifest = read(&out.join("site/plain.manifest.json"));
    assert!(
        manifest.contains("\"source\": \"../logo.bin\""),
        "{manifest}"
    );
}

#[test]
fn a_second_build_can_not_take_the_lock() {
    let dir = tempfile::tempdir().expect("a temporary directory");
    let out = dir.path().join("out");
    let _first = OutputDir::lock(&out).expect("locked");
    let err = OutputDir::lock(&out).expect_err("already locked");
    assert!(matches!(err, StoreError::Locked { .. }), "{err}");
    assert!(
        err.to_string()
            .contains("another ascribe build is writing to"),
        "{err}"
    );
}

#[test]
fn the_lock_is_released_when_the_build_ends() {
    let dir = tempfile::tempdir().expect("a temporary directory");
    let out = dir.path().join("out");
    drop(OutputDir::lock(&out).expect("locked"));
    OutputDir::lock(&out).expect("locked again");
}

#[test]
fn a_failure_while_staging_leaves_the_previous_output_alone() {
    let dir = tempfile::tempdir().expect("a temporary directory");
    let out = dir.path().join("out");
    replace(&out, "site", &[page("a.md", "A")]).expect("first");
    let manifest_before = read(&out.join("site/plain.manifest.json"));
    let missing = EmittedFile {
        path: RelPath::parse("logo.png").expect("a path"),
        kind: FileKind::Asset,
        source: Some(RelPath::parse("logo.png").expect("a path")),
        url: None,
        contents: Contents::Copy(dir.path().join("does-not-exist.png")),
    };
    let err = replace(&out, "site", &[page("a.md", "A2"), missing]).expect_err("a missing asset");
    assert!(matches!(err, StoreError::Io { .. }), "{err}");
    assert_eq!(read(&out.join("site/plain/a.md")), "A");
    assert_eq!(read(&out.join("site/plain.manifest.json")), manifest_before);
    assert!(!out.join(".staging").exists());
}
