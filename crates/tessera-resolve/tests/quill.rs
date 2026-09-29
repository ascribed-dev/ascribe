//! The Quill project (`examples/quill`) indexes and expands with no problems.

#![allow(clippy::expect_used, clippy::panic)]

use std::path::PathBuf;
use std::sync::Arc;

use tessera_resolve::{DiskFs, FileKind, Layout, Project, RefKind};

fn quill() -> Project {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../examples/quill");
    let model = tessera_model::load(root.join("ascribe.toml")).expect("the Quill model loads");
    let layout = Layout::from_model(&model);
    let fs = DiskFs::new(&root, &layout);
    Project::load(Arc::new(model), layout, &fs)
}

#[test]
fn quill_indexes_with_no_problems() {
    let project = quill();
    assert!(project.unreadable().is_empty());
    let names: Vec<String> = project.files().map(|f| f.path.to_string()).collect();
    assert_eq!(
        names,
        [
            "_fragments/prerequisites.md",
            "install-agent.md",
            "keys.md",
            "quickstart.md"
        ]
    );
    assert_eq!(project.pages().count(), 3);
    assert_eq!(project.fragments().count(), 1);
    for file in project.files() {
        let problems = project.problems(&file.path);
        assert!(problems.is_empty(), "{}: {problems:?}", file.path);
    }
    assert!(project.empty_slug_headings().is_empty());
}

#[test]
fn quill_expands_with_no_problems() {
    let project = quill();
    for file in project.files() {
        let page = project
            .expand(&file.path)
            .expect("the file is in the project");
        assert!(
            page.problems.is_empty(),
            "{}: {:?}",
            file.path,
            page.problems
        );
    }
}

#[test]
fn quills_install_page_includes_the_prerequisites_fragment() {
    let project = quill();
    let fragment = tessera_core::RelPath::parse("_fragments/prerequisites.md").expect("path");
    let page = tessera_core::RelPath::parse("install-agent.md").expect("path");
    assert_eq!(
        project.file(&fragment).map(|f| f.kind),
        Some(FileKind::Fragment)
    );
    assert_eq!(project.includers(&fragment).len(), 1);
    assert_eq!(
        project.including_pages(&fragment),
        std::slice::from_ref(&page)
    );

    // The fragment's image is the one beside the fragment.
    let assets: Vec<(String, RefKind, String)> = project
        .assets(&page)
        .iter()
        .map(|a| (a.path.to_string(), a.kind, a.written_in.to_string()))
        .collect();
    assert!(
        assets.contains(&(
            "_fragments/prerequisites.png".to_owned(),
            RefKind::Image,
            "_fragments/prerequisites.md".to_owned()
        )),
        "{assets:?}"
    );
    // Across the project, the two assets the conformance case lists.
    let mut all: Vec<String> = project
        .pages()
        .flat_map(|p| project.assets(&p.path))
        .map(|a| a.path.to_string())
        .collect();
    all.sort();
    all.dedup();
    assert_eq!(all, ["_fragments/prerequisites.png", "playground.png"]);
}

#[test]
fn quills_source_ids_match_the_headings() {
    let project = quill();
    let install = tessera_core::RelPath::parse("install-agent.md").expect("path");
    let ids: Vec<&str> = project
        .file(&install)
        .expect("install-agent.md")
        .headings
        .iter()
        .map(|h| h.source_id.as_str())
        .collect();
    assert!(ids.contains(&"install-agent"), "{ids:?}");
    assert!(ids.contains(&"prerequisites"), "{ids:?}");
}
