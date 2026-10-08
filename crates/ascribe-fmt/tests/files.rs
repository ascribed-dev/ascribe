//! Formatting a project's files on disk, as `ascribe fmt` does.

#![allow(clippy::expect_used)]

use std::fs;
use std::path::{Path, PathBuf};

use ascribe_core::SourceBoundary;
use ascribe_fmt::{FormatFilesError, format_files};

const MODEL: &str = "spec = \"0.1\"\n[project]\ncontent-root = \"docs\"\n";
const UNFORMATTED: &str = "@note{ type = tip }:Careful.\n";

fn write(path: &Path, text: &[u8]) {
    fs::create_dir_all(path.parent().expect("a parent")).expect("create directories");
    fs::write(path, text).expect("write");
}

/// A boundary that lets every file be read, except those named `refused.md`.
struct Boundary;

impl SourceBoundary for Boundary {
    fn check_link(&self, path: &Path) -> Result<(), String> {
        if path.file_name().is_some_and(|n| n == "refused.md") {
            Err("a link out".to_owned())
        } else {
            Ok(())
        }
    }
}

fn run(config: &Path, paths: &[PathBuf], check: bool) -> Result<Vec<PathBuf>, FormatFilesError> {
    let model = ascribe_model::load(config).expect("the model loads");
    let mut listed = Vec::new();
    let result = format_files(config, &model, paths, check, &Boundary, &mut |p| {
        listed.push(p.to_owned())
    });
    if let Ok(done) = &result {
        assert_eq!(done.changed, listed, "every change is listed as it's made");
        assert!(done.refused.is_empty());
    }
    result.map(|done| done.changed)
}

#[test]
fn a_file_the_boundary_refuses_is_returned_and_left_alone() {
    let dir = tempfile::tempdir().expect("a temporary directory");
    let root = dir.path();
    let config = root.join("ascribe.toml");
    write(&config, MODEL.as_bytes());
    write(&root.join("docs/a.md"), UNFORMATTED.as_bytes());
    write(&root.join("docs/sub/refused.md"), UNFORMATTED.as_bytes());

    let model = ascribe_model::load(&config).expect("the model loads");
    let done =
        format_files(&config, &model, &[], false, &Boundary, &mut |_| {}).expect("formatted");
    assert_eq!(done.changed, [root.join("docs").join("a.md")]);
    let refused: Vec<_> = done
        .refused
        .iter()
        .map(|r| (r.path.clone(), r.reason.as_str()))
        .collect();
    assert_eq!(
        refused,
        [(
            root.join("docs").join("sub").join("refused.md"),
            "a link out"
        )]
    );
    assert_eq!(
        fs::read(root.join("docs/sub/refused.md")).expect("read"),
        UNFORMATTED.as_bytes()
    );
}

#[test]
fn it_formats_the_content_root_and_skips_other_projects_and_tool_folders() {
    let dir = tempfile::tempdir().expect("a temporary directory");
    let root = dir.path();
    let config = root.join("ascribe.toml");
    write(&config, MODEL.as_bytes());
    write(&root.join("docs/b.md"), UNFORMATTED.as_bytes());
    write(&root.join("docs/a.md"), b"@note {type=tip}: Fine.\n");
    write(&root.join("docs/node_modules/c.md"), UNFORMATTED.as_bytes());
    write(&root.join("docs/.cache/d.md"), UNFORMATTED.as_bytes());
    write(&root.join("docs/other/ascribe.toml"), MODEL.as_bytes());
    write(&root.join("docs/other/e.md"), UNFORMATTED.as_bytes());

    let b = root.join("docs").join("b.md");
    assert_eq!(
        run(&config, &[], true).expect("checked"),
        std::slice::from_ref(&b)
    );
    assert_eq!(fs::read_to_string(&b).expect("read"), UNFORMATTED);

    assert_eq!(
        run(&config, &[], false).expect("formatted"),
        std::slice::from_ref(&b)
    );
    assert_eq!(
        fs::read_to_string(&b).expect("read"),
        "@note {type=tip}: Careful.\n"
    );
    assert_eq!(
        run(&config, &[], true).expect("checked"),
        Vec::<PathBuf>::new()
    );
}

#[test]
fn a_file_that_isnt_utf8_stops_it() {
    let dir = tempfile::tempdir().expect("a temporary directory");
    let config = dir.path().join("ascribe.toml");
    write(&config, MODEL.as_bytes());
    let bad = dir.path().join("docs/bad.md");
    write(&bad, &[0xff, 0xfe]);
    let err = run(&config, std::slice::from_ref(&bad), true).expect_err("not UTF-8");
    assert!(matches!(&err, FormatFilesError::NotUtf8 { path } if *path == bad));
    assert_eq!(
        err.to_string(),
        format!("{}: not valid UTF-8", bad.display())
    );
}
