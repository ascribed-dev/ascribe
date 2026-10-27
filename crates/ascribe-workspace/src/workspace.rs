//! The [`Workspace`]: a project loaded once, and the [`WorkspaceSnapshot`]
//! everything that reads it is given.

use std::collections::HashMap;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, OnceLock};

use ascribe_core::{FileId, RelPath};
use ascribe_model::{ContentModel, SourceBound};
use ascribe_resolve::side_file::{Entry, Held, SideFile};
use ascribe_resolve::{
    CodeFiles, Committed, DiskFs, FileIds, FileSystem, IncrementalProject, Layout, MemoFile,
    MemoStore, Personal, Probe, SourceSet, Sources,
};

use crate::load::{
    LoadError, Loader, ModelFile, ReadFailure, SourceFile, read_side_files, read_sources,
};

/// A file system shared between threads, as a [`Workspace`] reads through.
pub type SharedFs = Arc<dyn FileSystem + Send + Sync>;

/// A project, loaded once: its content model and its text, every source
/// file under its content root, indexed, its side files, and the file
/// system it was read through, which what's read on demand (code files,
/// images, memo files) is read through too.
///
/// It owns the project's [`IncrementalProject`], and what it gives out is a
/// [`WorkspaceSnapshot`], which never changes. A command loads one and
/// never updates it, so it's loaded without what only an update needs
/// ([`IncrementalProject::load_deferred`]): loading costs what indexing the
/// sources costs.
pub struct Workspace {
    /// The source index, kept current through its changes.
    // What an update reads; the snapshot is all a loaded project's readers need.
    #[allow(dead_code)]
    index: IncrementalProject,
    snapshot: WorkspaceSnapshot,
}

impl std::fmt::Debug for Workspace {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Workspace")
            .field("snapshot", &self.snapshot)
            .finish_non_exhaustive()
    }
}

impl Workspace {
    /// Loads the project whose content model is at `config` with `loader`:
    /// the model, then every source file under its content root, as
    /// [`FileSystem::sources`] finds them, then the side files.
    ///
    /// A source file or directory that can't be read, or a source file that
    /// isn't valid UTF-8, is still a source file, with its [`ReadFailure`].
    ///
    /// # Errors
    ///
    /// As [`Loader::load_model`].
    pub fn load(loader: &Loader, config: &Path) -> Result<Workspace, LoadError> {
        Workspace::load_bounded(loader, config, SourceBound::Anywhere)
    }

    /// [`Workspace::load`], with a local source bounded by `bound` when no
    /// git repository is around the project ([`SourceBound`]).
    ///
    /// # Errors
    ///
    /// As [`Workspace::load`].
    pub fn load_bounded(
        loader: &Loader,
        config: &Path,
        bound: SourceBound<'_>,
    ) -> Result<Workspace, LoadError> {
        let model = loader.load_model_bounded(config, bound)?;
        Workspace::from_disk(loader.side_files, config, model)
    }

    /// The project of a content model [`Loader::load_model`] loaded from
    /// `config`, read from the disk as [`Workspace::load`] reads it, with
    /// the side files of `side_files`. A command that needs the model
    /// before the rest, to start on something beside the load, loads it
    /// this way.
    ///
    /// # Errors
    ///
    /// The model's content root isn't a relative path.
    pub fn from_disk(
        side_files: &'static [Entry],
        config: &Path,
        model: ModelFile,
    ) -> Result<Workspace, LoadError> {
        let content_root =
            RelPath::parse(&model.model.project.content_root).map_err(|e| LoadError::Read {
                path: config.display().to_string(),
                message: e.to_string(),
            })?;
        let layout = layout(&model.model, content_root);
        // A one-shot command reads a disk that doesn't change while it runs,
        // so listings are kept.
        let fs = Arc::new(DiskFs::with_listing_cache(&model.root, &layout));
        Ok(Workspace::load_with(
            side_files,
            model,
            layout.content_root,
            fs,
        ))
    }

    /// The project of a content model [`Loader::load_model`] loaded, with
    /// every source file under `content_root` read through `fs`, as
    /// [`FileSystem::sources`] lists them, and the side files of
    /// `side_files` read through it. Each source is listed once and read
    /// once, and the source index is built from what was read.
    ///
    /// A source file or directory that can't be read, or a source file that
    /// isn't valid UTF-8, is still a source file, with its [`ReadFailure`].
    pub fn load_with(
        side_files: &'static [Entry],
        model: ModelFile,
        content_root: RelPath,
        fs: SharedFs,
    ) -> Workspace {
        let (read, listing) = read_sources(&*fs);
        let side = read_side_files(side_files, &*fs);
        let failed = failures(&read);
        let texts = Texts::new(fs.clone(), listing, read);
        Workspace::assemble(model, content_root, texts, failed, side, fs)
    }

    /// A project whose source files are `files`, `(content path, text)`
    /// pairs, held in memory, with everything else read through `fs`: a
    /// project built in memory, as a test's or `ascribe explain`'s
    /// examples, or one whose pages are given rather than read. The files
    /// have ids from 1 in path order, as a loaded project's.
    pub fn from_texts(
        side_files: &'static [Entry],
        model: ModelFile,
        content_root: RelPath,
        files: impl IntoIterator<Item = (RelPath, String)>,
        fs: SharedFs,
    ) -> Workspace {
        let side = read_side_files(side_files, &*fs);
        let texts = Texts::held(fs.clone(), files.into_iter().map(|(p, t)| (p, Ok(t))));
        Workspace::assemble(model, content_root, texts, Vec::new(), side, fs)
    }

    fn assemble(
        model: ModelFile,
        content_root: RelPath,
        texts: Texts,
        failed: Vec<(RelPath, ReadFailure)>,
        side: Vec<Held>,
        fs: SharedFs,
    ) -> Workspace {
        let ModelFile { root, text, model } = model;
        let layout = layout(&model, content_root);
        let model = Arc::new(model);
        let index = IncrementalProject::load_deferred(model.clone(), layout, texts);
        let snapshot = WorkspaceSnapshot(Arc::new(Inner::assemble(
            Parts {
                root,
                model,
                model_text: text,
                index: index.snapshot().shared_project(),
                side,
                fs,
            },
            failed,
        )));
        Workspace { index, snapshot }
    }

    /// The project as it is: what every check and command reads.
    pub fn snapshot(&self) -> WorkspaceSnapshot {
        self.snapshot.clone()
    }
}

/// The layout of a content model, with its content root as given.
fn layout(model: &ContentModel, content_root: RelPath) -> Layout {
    Layout {
        content_root,
        output_dir: RelPath::parse(&model.project.output_dir).unwrap_or_default(),
    }
}

/// The sources of `read` that couldn't be read, in its order.
fn failures(read: &[(RelPath, Result<String, ReadFailure>)]) -> Vec<(RelPath, ReadFailure)> {
    read.iter()
        .filter_map(|(path, r)| Some((path.clone(), r.as_ref().err()?.clone())))
        .collect()
}

/// A [`Workspace`]'s project at one moment: the source index, the content
/// model and its text, the source files, the side files, and what's read
/// on demand (code files, images, memo files), through the file system the
/// project was read through. It never changes, it's cheap to clone, and it
/// can be read from any thread.
///
/// Its source files have the source index's ids, and their texts are the
/// index's.
#[derive(Clone)]
pub struct WorkspaceSnapshot(Arc<Inner>);

struct Inner {
    root: PathBuf,
    model: Arc<ContentModel>,
    model_text: String,
    sources: Vec<SourceFile>,
    /// Every readable source file's content path, lowercased, to its real
    /// spelling, for [`SourceSet`].
    source_paths: HashMap<String, RelPath>,
    index: Arc<ascribe_resolve::Project>,
    /// The side files the project has, each read with its text, in the order
    /// of the list it was loaded with.
    side: Vec<Held>,
    fs: SharedFs,
    /// The code files snippets have read, read as the checks ask for them.
    code: Arc<CodeFiles>,
    /// The committed memo files, read the first time they're asked for.
    memos: Arc<OnceLock<Vec<MemoFile>>>,
    /// The committed memo files and then the personal ones, read the first
    /// time they're asked for.
    all_memos: Arc<OnceLock<Vec<MemoFile>>>,
    /// The image files under the content root, by path from the project
    /// root and from the content root, once the image checks have listed
    /// them.
    images: OnceLock<Vec<(RelPath, RelPath)>>,
}

impl std::fmt::Debug for WorkspaceSnapshot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WorkspaceSnapshot")
            .field("root", &self.0.root)
            .field("sources", &self.0.sources.len())
            .field("side", &self.0.side)
            .finish_non_exhaustive()
    }
}

/// What a snapshot is made of, apart from its sources, which are its
/// index's.
struct Parts {
    root: PathBuf,
    model: Arc<ContentModel>,
    model_text: String,
    index: Arc<ascribe_resolve::Project>,
    side: Vec<Held>,
    fs: SharedFs,
}

impl Inner {
    /// The snapshot of `parts`, with the index's files as its sources in
    /// path order, then `failed`, the sources that couldn't be read, with
    /// ids past every other.
    fn assemble(parts: Parts, failed: Vec<(RelPath, ReadFailure)>) -> Inner {
        let Parts {
            root,
            model,
            model_text,
            index,
            side,
            fs,
        } = parts;
        let mut sources: Vec<SourceFile> = index
            .files()
            .map(|f| SourceFile {
                id: f.file,
                path: f.path.clone(),
                text: f.source.clone(),
                unreadable: None,
            })
            .collect();
        let first = sources
            .iter()
            .map(|s| s.id.index())
            .max()
            .map_or(1, |max| max + 1);
        sources.extend(
            (first..)
                .zip(failed)
                .map(|(id, (path, failure))| SourceFile {
                    id: FileId::new(id),
                    path,
                    text: Arc::from(""),
                    unreadable: Some(failure),
                }),
        );
        // A file that couldn't be read isn't a source another file can name,
        // as in the source index.
        let source_paths = sources
            .iter()
            .filter(|s| s.unreadable.is_none())
            .map(|s| (s.path.as_str().to_lowercase(), s.path.clone()))
            .collect();
        Inner {
            root,
            model,
            model_text,
            sources,
            source_paths,
            index,
            side,
            fs,
            code: Arc::default(),
            memos: Arc::default(),
            all_memos: Arc::default(),
            images: OnceLock::new(),
        }
    }
}

impl WorkspaceSnapshot {
    /// A snapshot of a source index someone else keeps: the language
    /// server's, over its open buffers. `fs` is the file system the index
    /// probes, the side files of `side_files` are read through it, and the
    /// source files are the index's, with the files it couldn't read after
    /// them, all in path order.
    pub fn of_index(
        root: PathBuf,
        model: Arc<ContentModel>,
        model_text: String,
        index: Arc<ascribe_resolve::Project>,
        side_files: &'static [Entry],
        fs: SharedFs,
    ) -> WorkspaceSnapshot {
        let failed = index
            .unreadable()
            .iter()
            .map(|u| {
                (
                    u.path.clone(),
                    ReadFailure {
                        reason: u.reason.clone(),
                        not_utf8: u.reason.contains("valid UTF-8"),
                    },
                )
            })
            .collect();
        let side = read_side_files(side_files, &*fs);
        let mut inner = Inner::assemble(
            Parts {
                root,
                model,
                model_text,
                index,
                side,
                fs,
            },
            failed,
        );
        inner.sources.sort_by(|a, b| a.path.cmp(&b.path));
        WorkspaceSnapshot(Arc::new(inner))
    }

    /// The project root: the directory containing `ascribe.toml`.
    pub fn root(&self) -> &Path {
        &self.0.root
    }

    /// Where the content root and the output directory are.
    pub fn layout(&self) -> &Layout {
        self.0.index.layout()
    }

    /// The content root, relative to the project root.
    pub fn content_root(&self) -> &RelPath {
        &self.layout().content_root
    }

    /// The content model.
    pub fn model(&self) -> &ContentModel {
        &self.0.model
    }

    /// The content model's text.
    pub fn model_text(&self) -> &str {
        &self.0.model_text
    }

    /// The source index: every source file indexed, and what each reference
    /// names.
    pub fn index(&self) -> &Arc<ascribe_resolve::Project> {
        &self.0.index
    }

    /// The source files: the index's, with the files that couldn't be read
    /// after them, each with its [`ReadFailure`].
    pub fn sources(&self) -> &[SourceFile] {
        &self.0.sources
    }

    /// A source file by content path.
    pub fn source_at(&self, path: &RelPath) -> Option<&SourceFile> {
        self.0.sources.iter().find(|s| &s.path == path)
    }

    /// The file system the project was read through, and what isn't a
    /// source file is read through.
    pub fn file_system(&self) -> &dyn FileSystem {
        &*self.0.fs
    }

    /// [`WorkspaceSnapshot::file_system`], shared.
    pub fn shared_file_system(&self) -> &SharedFs {
        &self.0.fs
    }

    /// The side files the project has, each read with its text.
    pub fn side_files(&self) -> &[Held] {
        &self.0.side
    }

    /// The side file `F` read, when the project has it.
    pub fn side_file<F: SideFile>(&self) -> Option<&F::Read> {
        self.0.side.iter().find_map(Held::get::<F>)
    }

    /// The text of the side file at `path`, from the project root, when the
    /// project has it.
    pub fn side_text(&self, path: &str) -> Option<&str> {
        self.0
            .side
            .iter()
            .find(|held| held.path() == path)
            .map(Held::text)
    }

    /// The code files snippets have read through the project's file system,
    /// so far.
    pub fn code_files(&self) -> &CodeFiles {
        &self.0.code
    }

    /// The committed memo files (SPEC §7.7), in path order, read through the
    /// project's [`FileSystem`] the first time they're asked for. None when
    /// `memos/` is inside the content root, where its files are pages.
    pub fn memos(&self) -> &[MemoFile] {
        self.0
            .memos
            .get_or_init(|| Committed::new(self.file_system(), self.layout()).memos())
    }

    /// The committed memo files, then the personal ones in `.ascribe/memos/`,
    /// each in path order.
    pub fn all_memos(&self) -> &[MemoFile] {
        self.0.all_memos.get_or_init(|| {
            let mut all = self.memos().to_vec();
            all.extend(Personal::new(self.file_system()).memos());
            all
        })
    }

    /// The image files under the content root, by path from the project
    /// root and from the content root, listed by `list` the first time
    /// they're asked for.
    pub fn images_or(
        &self,
        list: impl FnOnce() -> Vec<(RelPath, RelPath)>,
    ) -> &[(RelPath, RelPath)] {
        self.0.images.get_or_init(list)
    }

    /// The image files, once they've been listed.
    pub fn images(&self) -> Option<&[(RelPath, RelPath)]> {
        self.0.images.get().map(Vec::as_slice)
    }

    /// This snapshot with the source file at `path` (a content path) holding
    /// `text` instead of its own, or added when there's none: a file checked
    /// before it's saved. The readable sources are numbered again, from 1 in
    /// path order, and indexed again, as a project of these texts and no
    /// other folders; the files that can't be read follow them.
    #[must_use]
    pub fn with_source(&self, path: &RelPath, text: String) -> WorkspaceSnapshot {
        let mut files: Vec<(RelPath, String)> = self
            .readable()
            .filter(|s| &s.path != path)
            .map(|s| (s.path.clone(), s.text.to_string()))
            .collect();
        files.push((path.clone(), text));
        self.indexed_again(files, FileIds::new(), |s| &s.path != path)
    }

    /// This snapshot with each of `replaced`'s files, by content path,
    /// holding the text given instead of its own, every file keeping its
    /// id, and indexed again, as a project of these texts and no other
    /// folders: what an edit's result is checked in.
    #[must_use]
    pub fn with_texts(&self, replaced: &[(&RelPath, &str)]) -> WorkspaceSnapshot {
        let files: Vec<(RelPath, String)> = self
            .readable()
            .map(|s| {
                let text = match replaced.iter().find(|(path, _)| **path == s.path) {
                    Some((_, text)) => (*text).to_owned(),
                    None => s.text.to_string(),
                };
                (s.path.clone(), text)
            })
            .collect();
        let ids = FileIds::keeping(self.readable().map(|s| (s.path.clone(), s.id)));
        self.indexed_again(files, ids, |_| true)
    }

    fn readable(&self) -> impl Iterator<Item = &SourceFile> {
        self.0.sources.iter().filter(|s| s.unreadable.is_none())
    }

    /// A snapshot of `files`, indexed with `ids`, and this one's unreadable
    /// sources that `keep` keeps, sharing its side files and memo files.
    fn indexed_again(
        &self,
        files: Vec<(RelPath, String)>,
        mut ids: FileIds,
        keep: impl Fn(&SourceFile) -> bool,
    ) -> WorkspaceSnapshot {
        let texts = Texts::held(
            self.0.fs.clone(),
            files.into_iter().map(|(p, t)| (p, Ok(t))),
        );
        let index = ascribe_resolve::Project::load_with_ids(
            self.0.model.clone(),
            self.layout().clone(),
            &texts,
            &mut ids,
        );
        let failed = self
            .0
            .sources
            .iter()
            .filter(|s| keep(s))
            .filter_map(|s| Some((s.path.clone(), s.unreadable.clone()?)))
            .collect();
        let mut inner = Inner::assemble(
            Parts {
                root: self.0.root.clone(),
                model: self.0.model.clone(),
                model_text: self.0.model_text.clone(),
                index: Arc::new(index),
                side: self.0.side.clone(),
                fs: self.0.fs.clone(),
            },
            failed,
        );
        // A page's text doesn't change the memo files.
        inner.memos = self.0.memos.clone();
        inner.all_memos = self.0.all_memos.clone();
        WorkspaceSnapshot(Arc::new(inner))
    }
}

impl SourceSet for WorkspaceSnapshot {
    fn contains(&self, path: &RelPath) -> bool {
        self.0
            .source_paths
            .get(&path.as_str().to_lowercase())
            .is_some_and(|actual| actual == path)
    }

    fn case_twin(&self, path: &RelPath) -> Option<RelPath> {
        self.0
            .source_paths
            .get(&path.as_str().to_lowercase())
            .filter(|actual| *actual != path)
            .cloned()
    }

    fn pages(&self) -> Vec<RelPath> {
        let mut pages: Vec<RelPath> = self.0.source_paths.values().cloned().collect();
        pages.sort();
        pages
    }
}

/// The files of a project as its source index reads them: the source texts
/// already read, each given out once, and the listing they were read from,
/// given out once, so indexing reads nothing again; everything else, and
/// anything asked for again, from the file system underneath.
struct Texts {
    base: SharedFs,
    listing: Mutex<Option<Sources>>,
    texts: Mutex<HashMap<RelPath, Result<String, ReadFailure>>>,
}

impl Texts {
    /// The texts loading read, from `listing`.
    fn new(
        base: SharedFs,
        listing: Sources,
        texts: impl IntoIterator<Item = (RelPath, Result<String, ReadFailure>)>,
    ) -> Texts {
        Texts {
            base,
            listing: Mutex::new(Some(listing)),
            texts: Mutex::new(texts.into_iter().collect()),
        }
    }

    /// Texts held in memory, listed as the readable ones and no folders: a
    /// file that couldn't be read isn't a source another file can name, and
    /// is reported as `source-unreadable` at file level.
    fn held(
        base: SharedFs,
        texts: impl IntoIterator<Item = (RelPath, Result<String, ReadFailure>)>,
    ) -> Texts {
        let texts: HashMap<_, _> = texts.into_iter().collect();
        let listing = Sources {
            paths: texts
                .iter()
                .filter(|(_, text)| text.is_ok())
                .map(|(path, _)| path.clone())
                .collect(),
            unreadable: Vec::new(),
            nested: Vec::new(),
            own_folder: None,
        };
        Texts {
            base,
            listing: Mutex::new(Some(listing)),
            texts: Mutex::new(texts),
        }
    }
}

impl FileSystem for Texts {
    fn sources(&self) -> Sources {
        let given = self.listing.lock().ok().and_then(|mut l| l.take());
        given.unwrap_or_else(|| self.base.sources())
    }

    fn read(&self, path: &RelPath) -> io::Result<String> {
        let given = self.texts.lock().ok().and_then(|mut t| t.remove(path));
        match given {
            Some(Ok(text)) => Ok(text),
            // Failing as loading's read failed, for the index to list it as
            // unreadable for the same reason.
            Some(Err(failure)) => {
                let kind = if failure.not_utf8 {
                    io::ErrorKind::InvalidData
                } else {
                    io::ErrorKind::Other
                };
                Err(io::Error::new(kind, failure.reason))
            }
            None => self.base.read(path),
        }
    }

    fn probe(&self, project_path: &RelPath) -> Probe {
        self.base.probe(project_path)
    }

    fn read_file(&self, project_path: &RelPath) -> io::Result<Vec<u8>> {
        self.base.read_file(project_path)
    }

    fn real_path(&self, project_path: &RelPath) -> Option<RelPath> {
        self.base.real_path(project_path)
    }

    fn size(&self, project_path: &RelPath) -> Option<u64> {
        self.base.size(project_path)
    }

    fn files_in(&self, project_dir: &RelPath) -> Vec<RelPath> {
        self.base.files_in(project_dir)
    }

    fn files_named(
        &self,
        project_dir: &RelPath,
        name: &str,
        enter: &dyn Fn(&RelPath) -> bool,
    ) -> Vec<RelPath> {
        self.base.files_named(project_dir, name, enter)
    }
}
