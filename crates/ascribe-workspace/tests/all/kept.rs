//! A `Kept` project follows its files on disk: the listing's differences
//! become changes that read only the files that differ, a file created,
//! deleted, edited, or renamed, a side file appearing, and a code file
//! changing outside the project's folder each reach the snapshot, and what
//! can't be applied in place loads the project again. A `Workspace` told of
//! a side file says what its change makes stale.

#![allow(clippy::expect_used, clippy::unwrap_used)]

use std::fs;
use std::path::Path;
use std::sync::Arc;

use ascribe_core::{FileId, Issue, RelPath};
use ascribe_model::{ContentModel, SourceBound};
use ascribe_resolve::side_file::{Addresses, Entry, Lock, Nav, Permalinks};
use ascribe_resolve::{ApplyError, Change, DiskFs, Layout};
use ascribe_workspace::{Changed, Kept, Listing, Loader, Workspace};

static SIDE_FILES: [Entry; 4] = [
    Entry::of::<Lock>(),
    Entry::of::<Permalinks>(),
    Entry::of::<Nav>(),
    Entry::of::<Addresses>(),
];

fn model(text: &str, _root: &Path, _bound: SourceBound<'_>) -> Result<ContentModel, Vec<Issue>> {
    ascribe_model::load_str(text, FileId::new(0))
}

const LOADER: Loader = Loader {
    model,
    side_files: &SIDE_FILES,
};

const MODEL: &str = "spec = \"0.1\"\n[project]\ncontent-root = \"docs\"\n[sources.code]\npath = \"../code\"\n[builds.site]\n";

fn write(path: &Path, text: &str) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, text).unwrap();
}

fn path(text: &str) -> RelPath {
    RelPath::parse(text).unwrap()
}

/// A project in `project/` beside the folder of code its snippets show.
fn project() -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().join("project");
    write(&root.join("ascribe.toml"), MODEL);
    write(
        &root.join("docs/index.md"),
        "---\ntitle: Home\n---\n\n[Guide](guide.md#start)\n\n@snippet: code:app.py\n",
    );
    write(
        &root.join("docs/guide.md"),
        "---\ntitle: Guide\n---\n\n## Start\n",
    );
    write(&dir.path().join("code/app.py"), "print(1)\n");
    dir
}

fn kept(dir: &Path) -> Kept {
    Kept::load(
        LOADER,
        &dir.join("project/ascribe.toml"),
        SourceBound::Anywhere,
    )
    .unwrap()
}

/// The ids of the headings of the page at `page`.
fn headings(kept: &Kept, page: &str) -> Vec<String> {
    kept.snapshot()
        .index()
        .file(&path(page))
        .map(|f| f.headings.iter().map(|h| h.source_id.clone()).collect())
        .unwrap_or_default()
}

#[test]
fn nothing_changed_keeps_the_snapshot() {
    let dir = project();
    let mut kept = kept(dir.path());
    let before = kept.snapshot();
    kept.refresh(SourceBound::Anywhere).unwrap();
    assert!(kept.snapshot().same_as(&before));
}

#[test]
fn a_file_edited_created_deleted_and_renamed_reaches_the_snapshot() {
    let dir = project();
    let docs = dir.path().join("project/docs");
    let mut kept = kept(dir.path());
    assert_eq!(headings(&kept, "guide.md"), ["start"]);

    write(
        &docs.join("guide.md"),
        "---\ntitle: Guide\n---\n\n## Begin here\n",
    );
    kept.refresh(SourceBound::Anywhere).unwrap();
    assert_eq!(headings(&kept, "guide.md"), ["begin-here"]);

    write(&docs.join("new.md"), "---\ntitle: New\n---\n\n## Fresh\n");
    kept.refresh(SourceBound::Anywhere).unwrap();
    assert_eq!(headings(&kept, "new.md"), ["fresh"]);

    fs::rename(docs.join("new.md"), docs.join("moved.md")).unwrap();
    kept.refresh(SourceBound::Anywhere).unwrap();
    assert!(kept.snapshot().source_at(&path("new.md")).is_none());
    assert_eq!(headings(&kept, "moved.md"), ["fresh"]);

    fs::remove_file(docs.join("moved.md")).unwrap();
    kept.refresh(SourceBound::Anywhere).unwrap();
    assert!(kept.snapshot().source_at(&path("moved.md")).is_none());
}

#[test]
fn a_side_file_appearing_is_read() {
    let dir = project();
    let root = dir.path().join("project");
    let mut kept = kept(dir.path());
    assert!(kept.snapshot().side_text("ascribe.nav").is_none());
    let nav = "version = 1\n\n[[sections]]\nfolder = \"\"\npages = [\"guide.md\", \"index.md\"]\n";
    write(&root.join("ascribe.nav"), nav);
    kept.refresh(SourceBound::Anywhere).unwrap();
    assert_eq!(kept.snapshot().side_text("ascribe.nav"), Some(nav));
    // The index reads it too.
    assert!(kept.snapshot().index().nav_file().is_some());
}

#[test]
fn a_code_file_outside_the_projects_folder_is_followed() {
    let dir = project();
    let mut kept = kept(dir.path());
    let code = |kept: &Kept| {
        let snapshot = kept.snapshot();
        let index = snapshot.index();
        let page = index.file(&path("index.md")).unwrap();
        index
            .snippet_at(&page.path, page.snippets[0].span)
            .map(|s| s.code.clone())
    };
    assert_eq!(code(&kept).as_deref(), Some("print(1)\n"));
    // The first call sees what the snippets read.
    kept.refresh(SourceBound::Anywhere).unwrap();
    write(&dir.path().join("code/app.py"), "print(22)\n");
    kept.refresh(SourceBound::Anywhere).unwrap();
    assert_eq!(code(&kept).as_deref(), Some("print(22)\n"));
}

#[test]
fn a_nested_project_appearing_loads_the_project_again() {
    let dir = project();
    let docs = dir.path().join("project/docs");
    write(&docs.join("inner/page.md"), "---\ntitle: Inner\n---\n");
    let mut kept = kept(dir.path());
    assert!(kept.snapshot().source_at(&path("inner/page.md")).is_some());
    write(&docs.join("inner/ascribe.toml"), MODEL);
    kept.refresh(SourceBound::Anywhere).unwrap();
    assert!(kept.snapshot().source_at(&path("inner/page.md")).is_none());
}

#[test]
fn ascribe_toml_changing_is_applied_and_one_that_doesnt_load_is_an_error() {
    let dir = project();
    let config = dir.path().join("project/ascribe.toml");
    let mut kept = kept(dir.path());
    write(&config, &format!("{MODEL}[phrases]\nproduct = \"Quill\"\n"));
    kept.refresh(SourceBound::Anywhere).unwrap();
    assert_eq!(kept.snapshot().model().phrases.len(), 1);
    write(&config, "spec = \"0.1\"\n[project\n");
    assert!(kept.refresh(SourceBound::Anywhere).is_err());
}

#[test]
fn the_differences_are_changes_to_the_files_that_differ() {
    let dir = project();
    let root = dir.path().join("project");
    let kept = kept(dir.path());
    let snapshot = kept.snapshot();
    let list = || Listing::of(&root, snapshot.layout(), snapshot.model(), &SIDE_FILES);
    let before = list();
    write(
        &root.join("docs/guide.md"),
        "---\ntitle: Guide\n---\n\n## Later\n",
    );
    write(&root.join("docs/logo.png"), "png");
    write(&root.join("ascribe.permalinks"), "version = 1\n");
    write(&root.join("site/index.html"), "built");
    let now = list();
    let changes = before.differences(&now).changes(&root, &snapshot).unwrap();
    let shown: Vec<String> = changes
        .iter()
        .map(|c| match c {
            Changed::Index(Change::Edited { path, text }) => format!("edited {path} {text:?}"),
            Changed::Index(Change::AssetCreated { path }) => format!("created {path}"),
            Changed::SideFile(path) => format!("side {path}"),
            other => format!("{other:?}"),
        })
        .collect();
    assert_eq!(
        shown,
        [
            "side ascribe.permalinks",
            "edited guide.md \"---\\ntitle: Guide\\n---\\n\\n## Later\\n\"",
            "created docs/logo.png",
        ]
    );
}

/// The project of `config`, read from the disk as it is at each read.
fn on_disk(config: &Path) -> Workspace {
    let model = LOADER.load_model(config).unwrap();
    let layout = Layout::from_model(&model.model);
    let fs = Arc::new(DiskFs::new(&model.root, &layout));
    Workspace::load_with(&SIDE_FILES, model, layout.content_root, fs)
}

#[test]
fn a_side_file_changing_says_what_it_makes_stale() {
    let dir = project();
    let root = dir.path().join("project");
    let mut workspace = on_disk(&root.join("ascribe.toml"));

    write(&root.join("ascribe.lock"), "version = 1\n");
    let updated = workspace
        .apply([Changed::SideFile("ascribe.lock".to_owned())])
        .unwrap();
    assert!(updated.changed && !updated.across && !updated.every_file);
    assert!(updated.index.is_empty(), "the index doesn't read the lock");

    write(
        &root.join("ascribe.nav"),
        "version = 1\n\n[[sections]]\nfolder = \"\"\n",
    );
    let updated = workspace
        .apply([Changed::SideFile("ascribe.nav".to_owned())])
        .unwrap();
    assert!(updated.changed && updated.across);
    assert!(workspace.snapshot().index().nav_file().is_some());

    // Read again as it was, it changes nothing.
    let before = workspace.snapshot();
    let updated = workspace
        .apply([Changed::SideFile("ascribe.nav".to_owned())])
        .unwrap();
    assert!(!updated.changed);
    assert!(workspace.snapshot().same_as(&before));
}

#[test]
fn a_model_that_moves_the_content_root_is_refused() {
    let dir = project();
    let root = dir.path().join("project");
    let config = root.join("ascribe.toml");
    let mut workspace = on_disk(&config);
    let before = workspace.snapshot();
    write(&config, &MODEL.replace("\"docs\"", "\"pages\""));
    write(&root.join("pages/index.md"), "# Home\n");
    let model = LOADER.load_model(&config).unwrap();
    assert_eq!(
        workspace
            .apply([Changed::Model(Box::new(model))])
            .unwrap_err(),
        ApplyError::LayoutChanged
    );
    assert!(workspace.snapshot().same_as(&before));
}
