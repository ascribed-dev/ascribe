//! `Project::from_parts_with_fs`: the file-level checks probe the file system
//! they're given, not the disk, so the language server's view of files that
//! aren't sources (an image the editor just created) is the source index's.

#![allow(clippy::expect_used, clippy::panic)]

use std::path::PathBuf;
use std::sync::Arc;

use ascribe_check::{Project, check_files};
use ascribe_core::{FileId, RelPath};
use ascribe_resolve::{Layout, MemoryFs};

const MODEL: &str = "spec = \"0.1\"\n[project]\ncontent-root = \"docs\"\n";
const PAGE: &str = "---\ntitle: T\n---\n![A logo](logo.png)\n";

fn project(fs: Option<MemoryFs>) -> Project {
    let model = ascribe_model::load_str(MODEL, FileId::new(0)).expect("a model");
    let content_root = RelPath::parse("docs").expect("a path");
    let sources =
        Project::from_sources([(RelPath::parse("index.md").expect("a path"), PAGE.into())]);
    let root = PathBuf::from("/no/such/project");
    match fs {
        Some(fs) => Project::from_parts_with_fs(
            root,
            content_root,
            model,
            MODEL.to_owned(),
            sources,
            Arc::new(fs),
        ),
        None => Project::from_parts(root, content_root, model, MODEL.to_owned(), sources),
    }
}

fn slugs(project: &Project) -> Vec<&'static str> {
    check_files(project)
        .iter()
        .map(|d| d.slug.as_str())
        .collect()
}

#[test]
fn an_image_only_the_given_file_system_has_is_found() {
    let model = ascribe_model::load_str(MODEL, FileId::new(0)).expect("a model");
    let layout = Layout::from_model(&model);
    let with = MemoryFs::new(&layout).with_file("docs/logo.png", "");
    assert_eq!(slugs(&project(Some(with))), Vec::<&str>::new());
}

#[test]
fn without_it_the_disk_is_probed() {
    assert_eq!(slugs(&project(None)), ["image-source-missing"]);
    let model = ascribe_model::load_str(MODEL, FileId::new(0)).expect("a model");
    let layout = Layout::from_model(&model);
    let without = MemoryFs::new(&layout);
    assert_eq!(slugs(&project(Some(without))), ["image-source-missing"]);
}

#[test]
fn the_file_system_is_used_for_case_too() {
    let model = ascribe_model::load_str(MODEL, FileId::new(0)).expect("a model");
    let layout = Layout::from_model(&model);
    let twin = MemoryFs::new(&layout).with_file("docs/Logo.png", "");
    assert_eq!(slugs(&project(Some(twin))), ["image-source-missing"]);
}
