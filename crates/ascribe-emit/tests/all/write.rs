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
