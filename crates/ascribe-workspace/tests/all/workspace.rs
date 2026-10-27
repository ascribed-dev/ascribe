//! A `Workspace`: its snapshot is the source index `ascribe_resolve` loads
//! from the same files, each source is listed once and read once, and a
//! snapshot with text laid over it is indexed again.

#![allow(clippy::expect_used, clippy::unwrap_used)]

use std::collections::BTreeMap;
use std::io;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use ascribe_core::{FileId, RelPath};
use ascribe_model::ContentModel;
use ascribe_resolve::{FileSystem, Layout, MemoryFs, Probe, Sources};
use ascribe_workspace::{ModelFile, SharedFs, Workspace, WorkspaceSnapshot};

const MODEL: &str = "spec = \"0.1\"\n[project]\ncontent-root = \"docs\"\n[builds.site]\n";

fn path(text: &str) -> RelPath {
    RelPath::parse(text).expect("a path")
}

fn model() -> ContentModel {
    ascribe_model::load_str(MODEL, FileId::new(0)).expect("the model loads")
}

fn model_file() -> ModelFile {
    ModelFile {
        root: PathBuf::from("/project"),
        text: MODEL.to_owned(),
        model: model(),
    }
}

const PAGES: &[(&str, &str)] = &[
    (
        "index.md",
        "---\ntitle: Home\n---\n\n[Guide](guide.md#start)\n\n@include: _f.md\n",
    ),
    (
        "guide.md",
        "---\ntitle: Guide\n---\n\n## Start\n\n[gone](gone.md)\n",
    ),
    ("_f.md", "![Logo](logo.png)\n"),
    ("nested/ascribe.toml", ""),
    ("nested/page.md", "# Not a source\n"),
];

fn memory() -> MemoryFs {
    let layout = Layout::from_model(&model());
    let mut fs = MemoryFs::new(&layout).with_file("docs/logo.png", "");
    for (path, text) in PAGES {
        fs = fs.with_source(path, text);
    }
    fs
}

/// The file system `inner`, with each listing of the sources and each read
/// of one counted.
struct Counting {
    inner: MemoryFs,
    listed: AtomicUsize,
    reads: Mutex<BTreeMap<RelPath, usize>>,
}

impl FileSystem for Counting {
    fn sources(&self) -> Sources {
        self.listed.fetch_add(1, Ordering::SeqCst);
        self.inner.sources()
    }

    fn read(&self, path: &RelPath) -> io::Result<String> {
        *self.reads.lock().unwrap().entry(path.clone()).or_default() += 1;
        self.inner.read(path)
    }

    fn probe(&self, project_path: &RelPath) -> Probe {
        self.inner.probe(project_path)
    }

    fn read_file(&self, project_path: &RelPath) -> io::Result<Vec<u8>> {
        self.inner.read_file(project_path)
    }

    fn real_path(&self, project_path: &RelPath) -> Option<RelPath> {
        self.inner.real_path(project_path)
    }

    fn files_in(&self, project_dir: &RelPath) -> Vec<RelPath> {
        self.inner.files_in(project_dir)
    }
}

/// Everything an index says of each file, and of the project, as text.
fn dump(index: &ascribe_resolve::Project) -> Vec<String> {
    let mut out: Vec<String> = index
        .files()
        .map(|f| {
            format!(
                "{} {} {:?} {:?} {:?}",
                f.file.index(),
                f.path,
                f.source,
                index.resolutions(&f.path),
                index.problems(&f.path),
            )
        })
        .collect();
    out.push(format!("{:?}", index.unreadable()));
    out.push(format!("{:?}", index.nested_projects()));
    out.push(format!("{:?}", index.own_folder()));
    out
}

fn load(fs: SharedFs) -> Workspace {
    Workspace::load_with(&[], model_file(), path("docs"), fs)
}

#[test]
fn a_loaded_snapshot_is_the_index_resolve_loads_from_the_same_files() {
    let snapshot = load(Arc::new(memory())).snapshot();
    let layout = Layout::from_model(&model());
    let fresh = ascribe_resolve::Project::load(Arc::new(model()), layout, &memory());
    assert_eq!(dump(snapshot.index()), dump(&fresh));
    // Its sources are the index's, with its ids and texts.
    let sources: Vec<(u32, &str, &str)> = snapshot
        .sources()
        .iter()
        .map(|s| (s.id.index(), s.path.as_str(), &*s.text))
        .collect();
    let indexed: Vec<(u32, &str, &str)> = fresh
        .files()
        .map(|f| (f.file.index(), f.path.as_str(), &*f.source))
        .collect();
    assert_eq!(sources, indexed);
}

#[test]
fn loading_lists_the_sources_once_and_reads_each_once() {
    let counting = Arc::new(Counting {
        inner: memory(),
        listed: AtomicUsize::new(0),
        reads: Mutex::default(),
    });
    let snapshot = load(counting.clone()).snapshot();
    assert_eq!(counting.listed.load(Ordering::SeqCst), 1);
    let reads = counting.reads.lock().unwrap().clone();
    let sources: Vec<&RelPath> = snapshot.sources().iter().map(|s| &s.path).collect();
    assert_eq!(reads.keys().collect::<Vec<_>>(), sources);
    assert!(reads.values().all(|n| *n == 1), "{reads:?}");
}

#[test]
fn a_project_from_texts_numbers_them_in_path_order() {
    let files = [("b.md", "# B\n"), ("a.md", "# A\n")];
    let workspace = Workspace::from_texts(
        &[],
        model_file(),
        path("docs"),
        files.iter().map(|(p, t)| (path(p), (*t).to_owned())),
        Arc::new(memory()),
    );
    let snapshot = workspace.snapshot();
    let ids: Vec<(u32, &str)> = snapshot
        .sources()
        .iter()
        .map(|s| (s.id.index(), s.path.as_str()))
        .collect();
    assert_eq!(ids, [(1, "a.md"), (2, "b.md")]);
    // The texts given are the sources, whatever the file system holds.
    assert!(snapshot.index().file(&path("index.md")).is_none());
}

fn ids(snapshot: &WorkspaceSnapshot) -> Vec<(u32, String)> {
    snapshot
        .sources()
        .iter()
        .map(|s| (s.id.index(), s.path.to_string()))
        .collect()
}

#[test]
fn text_laid_over_a_snapshot_is_indexed_again() {
    let snapshot = load(Arc::new(memory())).snapshot();
    let before = ids(&snapshot);

    // A new file numbers every file again, in path order.
    let added = snapshot.with_source(&path("about.md"), "# About\n".to_owned());
    assert_eq!(
        ids(&added),
        [
            (1, "_f.md"),
            (2, "about.md"),
            (3, "guide.md"),
            (4, "index.md")
        ]
        .map(|(id, p)| (id, p.to_owned()))
    );
    assert!(added.index().file(&path("about.md")).is_some());
    assert_eq!(ids(&snapshot), before, "the snapshot itself doesn't change");

    // Replaced texts keep every file's id.
    let guide = path("guide.md");
    let replaced = snapshot.with_texts(&[(&guide, "---\ntitle: Guide\n---\n")]);
    assert_eq!(ids(&replaced), before);
    assert!(replaced.index().problems(&guide).is_empty());
    assert!(!snapshot.index().problems(&guide).is_empty());
}
