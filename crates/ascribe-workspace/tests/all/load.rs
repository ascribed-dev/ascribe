//! Loading a project: the content model through the loader given, the
//! sources, and the side files of the list given.

#![allow(clippy::expect_used, clippy::panic)]

use std::fs;
use std::path::Path;

use ascribe_core::{FileId, Issue};
use ascribe_model::{ContentModel, MODEL_FILE, SourceBound};
use ascribe_resolve::side_file::{Entry, Lock, Nav, Permalinks, SideFile};
use ascribe_workspace::{LoadError, Loader};

const MODEL: &str =
    "spec = \"0.1\"\n[project]\ncontent-root = \"docs\"\n[builds.site]\n[builds.pdf]\n";

/// The content model as `ascribe-model` loads it, with no checks of its own.
fn plain(text: &str, root: &Path, bound: SourceBound<'_>) -> Result<ContentModel, Vec<Issue>> {
    ascribe_model::load_str_bounded(text, FileId::new(0), root, bound)
}

fn loader(side_files: &'static [Entry]) -> Loader {
    Loader {
        model: plain,
        side_files,
    }
}

fn project(model: &str, pages: &[(&str, &[u8])]) -> tempfile::TempDir {
    let dir = tempfile::tempdir().expect("a temporary directory");
    fs::write(dir.path().join(MODEL_FILE), model).expect("write the model");
    fs::create_dir_all(dir.path().join("docs")).expect("create docs");
    for (path, text) in pages {
        fs::write(dir.path().join("docs").join(path), text).expect("write a page");
    }
    dir
}

#[test]
fn load_model_reads_only_the_content_model() {
    // A page that isn't UTF-8 doesn't matter to it.
    let dir = project(MODEL, &[("broken.md", &[0xff, 0xfe])]);
    let loaded = loader(&[])
        .load_model(&dir.path().join(MODEL_FILE))
        .expect("the model loads");
    assert_eq!(loaded.root, dir.path());
    assert_eq!(loaded.text, MODEL);
    assert_eq!(loaded.model.builds.len(), 2);

    let bad = project("spec = \"0.1\"\n[project\n", &[]);
    match loader(&[]).load_model(&bad.path().join(MODEL_FILE)) {
        Err(LoadError::Model { text, issues }) => {
            assert_eq!(text, "spec = \"0.1\"\n[project\n");
            assert_eq!(issues[0].slug.as_str(), "model-toml-syntax");
        }
        other => panic!("expected the model's errors, got {other:?}"),
    }
}

#[test]
fn load_model_uses_the_loader_given() {
    fn refuses(_: &str, _: &Path, _: SourceBound<'_>) -> Result<ContentModel, Vec<Issue>> {
        Err(Vec::new())
    }
    let dir = project(MODEL, &[]);
    let refusing = Loader {
        model: refuses,
        side_files: &[],
    };
    assert!(matches!(
        refusing.load_model(&dir.path().join(MODEL_FILE)),
        Err(LoadError::Model { .. })
    ));
}

#[test]
fn a_model_that_cant_be_read_is_a_read_error() {
    let dir = tempfile::tempdir().expect("a temporary directory");
    fs::write(dir.path().join(MODEL_FILE), [0xff, 0xfe]).expect("write the model");
    let err = loader(&[])
        .load(&dir.path().join(MODEL_FILE))
        .expect_err("not UTF-8");
    assert!(
        matches!(&err, LoadError::Read { message, .. } if message == "the file isn't valid UTF-8"),
        "{err:?}"
    );
    assert!(matches!(
        loader(&[]).load(&dir.path().join("nowhere.toml")),
        Err(LoadError::Read { .. })
    ));
}

#[test]
fn the_sources_are_read_in_path_order_with_the_unreadable_after() {
    let dir = project(
        MODEL,
        &[
            ("b.md", b"# B\n"),
            ("a.md", b"# A\n"),
            ("broken.md", &[0xff, 0xfe]),
        ],
    );
    let loaded = loader(&[])
        .load(&dir.path().join(MODEL_FILE))
        .expect("the project loads");
    assert_eq!(loaded.content_root.as_str(), "docs");
    assert_eq!(loaded.model_text, MODEL);
    let read: Vec<(u32, &str, bool)> = loaded
        .sources
        .iter()
        .map(|s| (s.id.index(), s.path.as_str(), s.unreadable.is_some()))
        .collect();
    assert_eq!(
        read,
        [
            (1, "a.md", false),
            (2, "b.md", false),
            (3, "broken.md", true)
        ]
    );
    assert_eq!(loaded.sources[0].text, "# A\n");
    let failure = loaded.sources[2].unreadable.as_ref().expect("unreadable");
    assert!(failure.not_utf8);
    assert_eq!(loaded.listing.paths.len(), 3);
}

#[test]
fn the_loader_reads_each_side_file_it_is_given_and_only_those() {
    const GIVEN: &[Entry] = &[Entry::of::<Nav>(), Entry::of::<Lock>()];
    let dir = project(MODEL, &[("index.md", b"# Home\n")]);
    for side in [Lock::PATH, Permalinks::PATH, Nav::PATH] {
        fs::write(dir.path().join(side), "not what it should be\n").expect("write a side file");
    }
    let config = dir.path().join(MODEL_FILE);

    let loaded = loader(GIVEN).load(&config).expect("the project loads");
    let paths: Vec<&str> = loaded.side.iter().map(|held| held.path()).collect();
    assert_eq!(paths, [Nav::PATH, Lock::PATH], "in the order of the list");
    assert_eq!(loaded.side[0].text(), "not what it should be\n");
    assert!(loaded.side[0].get::<Nav>().is_some());

    let none = loader(&[]).load(&config).expect("the project loads");
    assert!(none.side.is_empty());

    // A side file the list names and the project hasn't isn't read.
    fs::remove_file(dir.path().join(Lock::PATH)).expect("remove the lock");
    let without = loader(GIVEN).load(&config).expect("the project loads");
    let paths: Vec<&str> = without.side.iter().map(|held| held.path()).collect();
    assert_eq!(paths, [Nav::PATH]);
}
