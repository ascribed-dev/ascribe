//! Writing a project's builds, as `ascribe build` does once its checks pass.

#![allow(clippy::expect_used)]

use std::fs;
use std::sync::Arc;

use ascribe_emit::{Output, WriteEvent, WriteOptions, Written, write_outputs};
use ascribe_resolve::{DiskFs, Layout, Project};

const MODEL: &str = "spec = \"0.1\"\n[project]\ncontent-root = \"docs\"\noutput-dir = \"out\"\n[builds.site]\n[builds.pdf]\n";

#[test]
fn each_output_of_each_build_is_written_once_and_reported_as_it_is() {
    let dir = tempfile::tempdir().expect("a temporary directory");
    let root = dir.path();
    fs::write(root.join("ascribe.toml"), MODEL).expect("write the model");
    fs::create_dir_all(root.join("docs")).expect("create docs");
    fs::write(root.join("docs/index.md"), "# Home\n\nHello.\n").expect("write a page");
    let model = ascribe_model::load(root.join("ascribe.toml")).expect("the model loads");
    let layout = Layout::from_model(&model);
    let index = Project::load(
        Arc::new(model.clone()),
        layout.clone(),
        &DiskFs::new(root, &layout),
    );

    let options = WriteOptions {
        outputs: vec![Output::Plain, Output::Json, Output::Plain],
        anchors: false,
    };
    let mut events = Vec::new();
    let builds: Vec<&ascribe_model::Build> = model.builds.iter().collect();
    let written = write_outputs(&index, root, &builds, &options, &mut |e| {
        events.push(e.clone());
    })
    .expect("written");

    let expected: Vec<Written> = [
        ("site", "plain"),
        ("site", "json"),
        ("pdf", "plain"),
        ("pdf", "json"),
    ]
    .iter()
    .map(|(build, output)| Written {
        build: (*build).to_owned(),
        output,
        pages: 1,
        assets: 0,
        removed: 0,
    })
    .collect();
    assert_eq!(written, expected);
    let reported: Vec<Written> = events
        .into_iter()
        .filter_map(|e| match e {
            WriteEvent::Written(w) => Some(w),
            WriteEvent::Warning(_) => None,
        })
        .collect();
    assert_eq!(reported, expected);
    assert!(root.join("out/site/plain").is_dir());
    assert!(root.join("out/pdf/json").is_dir());
}

/// Every file under `dir`, by its path relative to `dir`, with its bytes.
fn files_under(dir: &std::path::Path) -> Vec<(String, String)> {
    let mut out = Vec::new();
    let mut stack = vec![dir.to_path_buf()];
    while let Some(at) = stack.pop() {
        for entry in fs::read_dir(&at).expect("read a directory") {
            let path = entry.expect("an entry").path();
            if path.is_dir() {
                stack.push(path);
            } else {
                let name = path
                    .strip_prefix(dir)
                    .expect("under the directory")
                    .to_string_lossy()
                    .replace('\\', "/");
                out.push((name, fs::read_to_string(&path).expect("read a file")));
            }
        }
    }
    out.sort();
    out
}

/// Writes every output of a project of one page and returns what was
/// written.
fn outputs_of(page: &str, model: &str) -> Vec<(String, String)> {
    let dir = tempfile::tempdir().expect("a temporary directory");
    let root = dir.path();
    fs::write(root.join("ascribe.toml"), model).expect("write the model");
    fs::create_dir_all(root.join("docs")).expect("create docs");
    fs::write(root.join("docs/index.md"), page).expect("write a page");
    let model = ascribe_model::load(root.join("ascribe.toml")).expect("the model loads");
    let layout = Layout::from_model(&model);
    let index = Project::load(
        Arc::new(model.clone()),
        layout.clone(),
        &DiskFs::new(root, &layout),
    );
    let options = WriteOptions {
        outputs: vec![Output::Site, Output::Plain, Output::Json],
        anchors: false,
    };
    let builds: Vec<&ascribe_model::Build> = model.builds.iter().collect();
    write_outputs(&index, root, &builds, &options, &mut |_| {}).expect("written");
    files_under(&root.join("out"))
}

#[test]
fn acknowledgements_change_no_output() {
    let model =
        "spec = \"0.1\"\n[project]\ncontent-root = \"docs\"\noutput-dir = \"out\"\n[builds.site]\n";
    let ack = "@intended {check=some-check}: Why.";
    let entry = "intended:\n  - check: some-check\n    reason: Why.\n";
    let with = format!(
        "---\ntitle: Home\n{entry}---\n# Home\n\n{ack}\nHello.\n\n@note:\n{ack}\nInside.\n@end\n\n- {ack}\n  An item.\n"
    );
    // The same page with each acknowledgement written over with spaces, and
    // the frontmatter entry with a comment, so every position is the same.
    let blank = |text: &str| " ".repeat(text.len());
    let comment = |text: &str| {
        text.lines()
            .map(|l| format!("#{}\n", " ".repeat(l.len() - 1)))
            .collect::<String>()
    };
    let without = with
        .replace(entry, &comment(entry))
        .replace(ack, &blank(ack));
    let only_key = format!("---\n{entry}---\n# Home\n\nHello.\n");
    let frontmatter = format!("---\n{entry}---\n");
    let none = only_key.replace(
        &frontmatter,
        &frontmatter
            .lines()
            .map(|l| format!("{}\n", blank(l)))
            .collect::<String>(),
    );
    let written = outputs_of(&without, model);
    assert!(!written.is_empty());
    assert_eq!(outputs_of(&with, model), written);
    assert_eq!(outputs_of(&only_key, model), outputs_of(&none, model));
}
