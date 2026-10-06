//! `ascribe check` and the incremental project number files the same way.
//!
//! Every location names its file by id, so the ids have to mean the same
//! thing to `ascribe_resolve::IncrementalProject` (which the language server
//! keeps current) and to `ascribe_check::Project` (which it builds from a
//! snapshot to check). The rules are in `ascribe_resolve::incremental`'s
//! module documentation; this test holds the two crates to them.

#![allow(clippy::expect_used, clippy::panic)]

use std::path::PathBuf;
use std::sync::Arc;

use ascribe_check::{Project, SourceFile, check_files};
use ascribe_core::{FileId, RelPath};
use ascribe_resolve::{Change, IncrementalProject, Layout, MemoryFs, Snapshot};

const MODEL: &str = "spec = \"0.1\"\n[project]\ncontent-root = \"docs\"\n";

fn path(text: &str) -> RelPath {
    RelPath::parse(text).expect("a path")
}

fn incremental(files: &[(&str, &str)]) -> IncrementalProject {
    let model = Arc::new(ascribe_model::load_str(MODEL, FileId::new(0)).expect("a model"));
    let layout = Layout::from_model(&model);
    let mut fs = MemoryFs::new(&layout);
    for (name, text) in files {
        fs = fs.with_source(name, text);
    }
    IncrementalProject::load(model, layout, fs)
}

/// A `ascribe_check::Project` over a snapshot's files, with the snapshot's ids.
fn check_project(snapshot: &Snapshot) -> Project {
    let sources = snapshot
        .files()
        .map(|f| SourceFile {
            id: f.file,
            path: f.path.clone(),
            text: f.source.to_string(),
            unreadable: None,
        })
        .collect();
    let model = ascribe_model::load_str(MODEL, FileId::new(0)).expect("a model");
    Project::from_parts(
        PathBuf::from("/project"),
        path("docs"),
        model,
        MODEL.to_owned(),
        sources,
    )
}

#[test]
fn a_fresh_load_numbers_files_as_check_does() {
    let files = [
        ("b.md", "---\ntitle: B\n---\n"),
        ("a.md", "---\ntitle: A\n---\n"),
        ("sub/c.md", "x\n"),
    ];
    let inc = incremental(&files);
    let sources = Project::from_sources(files.iter().map(|(p, t)| (path(p), (*t).to_owned())));
    for source in &sources {
        assert_eq!(
            inc.snapshot().file(&source.path).map(|f| f.file),
            Some(source.id),
            "{}",
            source.path
        );
    }
    assert_eq!(
        sources.iter().map(|s| s.id.index()).collect::<Vec<_>>(),
        [1, 2, 3]
    );
}

#[test]
fn check_finds_files_by_the_ids_of_an_updated_snapshot() {
    let mut inc = incremental(&[
        ("a.md", "---\ntitle: A\n---\n\n[gone](b.md)\n"),
        ("b.md", "---\ntitle: B\n---\n"),
    ]);
    // Delete the first file and add another: the ids are no longer 1..n.
    inc.apply([
        Change::Deleted { path: path("a.md") },
        Change::Created {
            path: path("c.md"),
            text: "---\ntitle: C\n---\n\n[missing](nope.md)\n".to_owned(),
        },
    ])
    .expect("applies");
    let snapshot = inc.snapshot();
    let ids: Vec<u32> = snapshot.files().map(|f| f.file.index()).collect();
    assert_eq!(ids, [2, 3], "b keeps 2, c is new, 1 is retired");

    let project = check_project(&snapshot);
    let diagnostics = check_files(&project);
    let missing: Vec<_> = diagnostics
        .iter()
        .filter(|d| d.slug.as_str() == "link-target-missing")
        .collect();
    assert_eq!(missing.len(), 1);
    let entry = project
        .file(missing[0].location.file)
        .expect("the id names a file");
    assert_eq!(entry.content_path, Some(&path("c.md")));
    assert_eq!(
        snapshot.path_of(missing[0].location.file),
        Some(&path("c.md"))
    );
    // The retired id names nothing in either.
    assert!(project.file(FileId::new(1)).is_none());
    assert!(snapshot.path_of(FileId::new(1)).is_none());
    // Every file's id in the check project is the snapshot's.
    for file in snapshot.files() {
        assert_eq!(project.source_at(&file.path).map(|s| s.id), Some(file.file));
    }
}
