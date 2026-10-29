//! The [`Workspace`]: a project loaded once, and the [`WorkspaceSnapshot`]
//! everything that reads it is given.

use std::collections::{BTreeSet, HashMap};
use std::io;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, OnceLock, PoisonError};

use ascribe_core::{FileId, RelPath};
use ascribe_model::{ContentModel, SourceBound};
use ascribe_resolve::side_file::{Entry, Held, Indexed, SideFile};
use ascribe_resolve::{
    Affected, ApplyError, Change, CodeFiles, Committed, DiskFs, FileIds, FileSystem,
    IncrementalProject, Layout, MemoFile, MemoStore, Personal, Probe, SourceSet, Sources,
    is_memo_path,
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
/// sources costs. One kept between calls takes what changed on its files
/// ([`Workspace::apply`]) and makes the next snapshot, and the first update
/// makes what loading left out.
pub struct Workspace {
    /// The source index, kept current through its changes.
    index: IncrementalProject,
    /// What its file system's probes answer.
    probes: Probes,
    snapshot: WorkspaceSnapshot,
}

/// A change to one of a project's files, as [`Workspace::apply`] takes it.
#[derive(Clone, Debug)]
pub enum Changed {
    /// A change to the sources, or to a file that isn't one, as the source
    /// index takes it: source files by content path, other files by path
    /// from the project root. A change to the model or the side files is
    /// given as [`Changed::Model`] or [`Changed::SideFile`], which keep the
    /// snapshot's text of them with them.
    Index(Change),
    /// The side file at this path from the project root, one of the list
    /// the project was loaded with, appeared, changed, or went: it's read
    /// again. A path in no entry of the list changes nothing.
    SideFile(String),
    /// `ascribe.toml` changed and loaded: this is the new model and its
    /// text.
    Model(Box<ModelFile>),
    /// A file the snapshot reads on demand changed where the source index
    /// isn't told of it, by path from the project root, as an open memo
    /// file's buffer is: what the snapshot read of its kind is read again
    /// when it's asked for.
    Read(RelPath),
}

/// What [`Workspace::apply`] made stale.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Updated {
    /// What the source index's update touched.
    pub index: Affected,
    /// Whether the checks across the project are to run again: a side file
    /// they compare with the content changed ([`ascribe_resolve::side_file::Stale::across`]).
    pub across: bool,
    /// Whether every file is to be checked again: a side file that holds
    /// back problems in every file changed
    /// ([`ascribe_resolve::side_file::Stale::every_file`]).
    pub every_file: bool,
    /// Whether the changes made a new snapshot. A batch that changes
    /// nothing keeps the one there was.
    pub changed: bool,
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
        Workspace::assemble(model, content_root, texts, failed, (side_files, side), fs)
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
        Workspace::assemble(
            model,
            content_root,
            texts,
            Vec::new(),
            (side_files, side),
            fs,
        )
    }

    fn assemble(
        model: ModelFile,
        content_root: RelPath,
        texts: Texts,
        failed: Vec<(RelPath, ReadFailure)>,
        (side_files, side): (&'static [Entry], Vec<Held>),
        fs: SharedFs,
    ) -> Workspace {
        let ModelFile { root, text, model } = model;
        let layout = layout(&model, content_root);
        let model = Arc::new(model);
        let probes = texts.probes.clone();
        let index = IncrementalProject::load_deferred(model.clone(), layout, texts);
        let snapshot = WorkspaceSnapshot(Arc::new(Inner::assemble(
            Parts {
                root,
                model,
                model_text: text,
                index: index.snapshot().shared_project(),
                side,
                side_list: side_files,
                fs,
            },
            Some(failed),
        )));
        Workspace {
            index,
            probes,
            snapshot,
        }
    }

    /// The project as it is: what every check and command reads.
    pub fn snapshot(&self) -> WorkspaceSnapshot {
        self.snapshot.clone()
    }

    /// [`Workspace::snapshot`], borrowed.
    pub fn current(&self) -> &WorkspaceSnapshot {
        &self.snapshot
    }

    /// Makes now what loading left out and the first [`Workspace::apply`]
    /// would make ([`IncrementalProject::prepare`]): for a project kept to
    /// be updated as it's typed in, whose first update should cost no more
    /// than the next.
    pub fn prepare(&mut self) {
        self.index.prepare();
    }

    /// The source index's latest version.
    pub fn version(&self) -> ascribe_resolve::Version {
        self.index.version()
    }

    /// Whether what was computed for the source at `path` from `snapshot`,
    /// one of [`Workspace::index_snapshot`]'s, is still right after every
    /// update since ([`IncrementalProject::is_file_current`]).
    pub fn is_file_current(&self, snapshot: &ascribe_resolve::Snapshot, path: &RelPath) -> bool {
        self.index.is_file_current(snapshot, path)
    }

    /// The source index's own snapshot, with the versions it reflects: what
    /// an edit computed beside the language server's works from.
    pub fn index_snapshot(&self) -> ascribe_resolve::Snapshot {
        self.index.snapshot()
    }

    /// The ids of every path a source has been at ([`FileIds`]).
    pub fn ids(&self) -> &FileIds {
        self.index.ids()
    }

    /// Takes a batch of changes to the project's files and makes the next
    /// snapshot, as a fresh load of the same files would be. A change to a
    /// source or a file that isn't one goes to the source index; a side file
    /// is read again through the project's file system, and what its
    /// [`Stale`](ascribe_resolve::side_file::Stale) names follows it: the
    /// index is given the side files it reads ([`Change::SideFiles`]), and
    /// [`Updated`] says when the checks across the project, or every file's,
    /// are stale. A memo file, an image, or a code file drops what the
    /// snapshot read of its kind on demand, to be read again when it's asked
    /// for. A batch that changes nothing keeps the snapshot there was. The
    /// snapshot lets go of the index while it's updated, so the update
    /// changes it in place rather than copying it.
    ///
    /// What's read again is read through the file system the project was
    /// loaded with, which must show the files as they are now: one a command
    /// loads ([`Workspace::load`]) keeps what it lists, since a command's
    /// project is never updated, and [`Kept`](crate::Kept) has the disk read
    /// again once its listing shows a change.
    ///
    /// # Errors
    ///
    /// The project must be loaded again, and nothing changed: a nested
    /// project's `ascribe.toml` appeared or went
    /// ([`ApplyError::NestedProjectChanged`]), or the new model moves the
    /// content root or the output directory ([`ApplyError::LayoutChanged`]).
    pub fn apply(
        &mut self,
        changes: impl IntoIterator<Item = Changed>,
    ) -> Result<Updated, ApplyError> {
        let old = &self.snapshot.0;
        let layout = old.index().layout().clone();
        let mut to_index = Vec::new();
        let mut model = None;
        let mut side_paths = BTreeSet::new();
        let mut drop = Dropped::default();
        for change in changes {
            match change {
                Changed::Index(change) => {
                    drop.note(&change, &layout, old.index());
                    to_index.push(change);
                }
                Changed::SideFile(path) => {
                    if let Some(entry) = old.side_list.iter().find(|e| e.path == path) {
                        side_paths.insert(entry.path);
                    }
                }
                Changed::Model(file) => model = Some(*file),
                Changed::Read(path) => {
                    if is_memo_path(&path, &layout) {
                        drop.memos = true;
                    } else {
                        drop.images |= path.starts_with(&layout.content_root);
                        drop.code = true;
                    }
                }
            }
        }

        // Each side file named, as it reads now, and what its change makes
        // stale.
        let mut side = old.side.clone();
        let mut side_changed = false;
        let (mut indexed, mut across, mut every_file) = (false, false, false);
        for entry in old.side_list.iter().filter(|e| side_paths.contains(e.path)) {
            let now = entry.read(&*old.fs);
            let at = side.iter().position(|held| held.path() == entry.path);
            let before = at.map(|at| side[at].text());
            if before == now.as_ref().map(Held::text) {
                continue;
            }
            side_changed = true;
            indexed |= entry.stale.index;
            across |= entry.stale.across;
            every_file |= entry.stale.every_file;
            match (at, now) {
                (Some(at), Some(now)) => side[at] = now,
                (Some(at), None) => {
                    side.remove(at);
                }
                (None, Some(now)) => side.push(now),
                (None, None) => {}
            }
        }
        // In the order of the list, as loading reads them.
        side.sort_by_key(|held| old.side_list.iter().position(|e| e.path == held.path()));
        if indexed {
            to_index.push(Change::SideFiles(Indexed::read(&*old.fs)));
        }

        let new_model = model.as_ref().map(|m| Arc::new(m.model.clone()));
        if let Some(model) = &new_model {
            to_index.push(Change::Model(model.clone()));
            drop = Dropped::all();
        }
        let new_model = new_model.zip(model.map(|m| m.text));
        if to_index.is_empty() {
            let version = self.index.version();
            let affected = Affected {
                version,
                previous: version,
                parsed: BTreeSet::new(),
                indexed: BTreeSet::new(),
                recheck: BTreeSet::new(),
                removed: BTreeSet::new(),
                unreadable: BTreeSet::new(),
                re_resolve: BTreeSet::new(),
                model: None,
            };
            let changed = side_changed || drop.any();
            let updated = Updated {
                across: across || drop.memos,
                every_file,
                changed,
                index: affected,
            };
            if changed {
                let next = Next::of(old, None, side, &drop);
                self.snapshot = next.assemble(self.index.snapshot().shared_project());
            }
            return Ok(updated);
        }

        for change in &to_index {
            self.probes.before(change, old.index());
        }
        let next = Next::of(old, new_model, side, &drop);
        // The snapshot lets go of the index while it's updated, when nothing
        // else holds it, so the update changes the index in place rather
        // than copying it whole.
        let let_go = Arc::get_mut(&mut self.snapshot.0).is_some_and(|inner| {
            inner.index = None;
            true
        });
        let affected = match self.index.apply(to_index) {
            Ok(affected) => affected,
            Err(e) => {
                // Nothing changed: the snapshot is the one there was, with
                // the index it had.
                self.take_back(let_go);
                return Err(e);
            }
        };
        if affected.is_empty() && !side_changed && !next.changes_anything() {
            // The index is as it was, so the snapshot there was still
            // describes it.
            self.take_back(let_go);
            return Ok(Updated {
                across,
                every_file,
                changed: false,
                index: affected,
            });
        }
        self.snapshot = next.assemble(self.index.snapshot().shared_project());
        Ok(Updated {
            across: across || drop.memos || !affected.is_empty(),
            every_file,
            changed: true,
            index: affected,
        })
    }

    /// Gives the snapshot back the index it let go of, if it did.
    fn take_back(&mut self, let_go: bool) {
        if let_go && let Some(inner) = Arc::get_mut(&mut self.snapshot.0) {
            inner.index = Some(self.index.snapshot().shared_project());
        }
    }
}

/// What a snapshot [`Workspace::apply`] makes takes from the one before it,
/// apart from the index, and what it takes instead.
struct Next {
    root: PathBuf,
    model: Arc<ContentModel>,
    model_text: String,
    side_list: &'static [Entry],
    fs: SharedFs,
    code: Arc<CodeFiles>,
    memos: Arc<OnceLock<Vec<MemoFile>>>,
    all_memos: Arc<OnceLock<Vec<MemoFile>>>,
    images: Arc<OnceLock<Vec<(RelPath, RelPath)>>>,
    /// The new model and its text, if it changed.
    new_model: Option<(Arc<ContentModel>, String)>,
    /// The side files as they read now.
    side: Vec<Held>,
    /// What's dropped of what was read on demand.
    drop: Dropped,
}

impl Next {
    /// Whether the next snapshot differs from the one before apart from
    /// its index: the model changed, or something read on demand is
    /// dropped.
    fn changes_anything(&self) -> bool {
        self.new_model.is_some() || self.drop.any()
    }

    fn of(
        old: &Inner,
        model: Option<(Arc<ContentModel>, String)>,
        side: Vec<Held>,
        drop: &Dropped,
    ) -> Next {
        Next {
            root: old.root.clone(),
            model: old.model.clone(),
            model_text: old.model_text.clone(),
            side_list: old.side_list,
            fs: old.fs.clone(),
            code: old.code.clone(),
            memos: old.memos.clone(),
            all_memos: old.all_memos.clone(),
            images: old.images.clone(),
            new_model: model,
            side,
            drop: drop.clone(),
        }
    }

    /// The next snapshot, of `index`.
    fn assemble(self, index: Arc<ascribe_resolve::Project>) -> WorkspaceSnapshot {
        let (model, model_text) = self.new_model.unwrap_or((self.model, self.model_text));
        let mut inner = Inner::assemble(
            Parts {
                root: self.root,
                model,
                model_text,
                index,
                side: self.side,
                side_list: self.side_list,
                fs: self.fs,
            },
            None,
        );
        if !self.drop.code {
            inner.code = self.code;
        }
        if !self.drop.memos {
            inner.memos = self.memos;
            inner.all_memos = self.all_memos;
        }
        if !self.drop.images {
            inner.images = self.images;
        }
        WorkspaceSnapshot(Arc::new(inner))
    }
}

/// What a batch of changes drops of what a snapshot reads on demand.
#[derive(Clone, Default)]
struct Dropped {
    code: bool,
    memos: bool,
    images: bool,
}

impl Dropped {
    fn all() -> Dropped {
        Dropped {
            code: true,
            memos: true,
            images: true,
        }
    }

    fn any(&self) -> bool {
        self.code || self.memos || self.images
    }

    /// What `change` drops. A file that isn't a source under the content
    /// root may be an image, and one anywhere else a code file a snippet
    /// reads or what its source says of its examples.
    fn note(&mut self, change: &Change, layout: &Layout, index: &ascribe_resolve::Project) {
        let not_source = |path: &RelPath| !index.is_source(path);
        match change {
            Change::Created { path, .. }
            | Change::Edited { path, .. }
            | Change::Deleted { path }
            | Change::Unreadable { path, .. } => {
                if not_source(path) {
                    self.images = true;
                    self.code = true;
                }
            }
            Change::Renamed { from, to } => {
                if not_source(from) || not_source(to) {
                    self.images = true;
                    self.code = true;
                }
            }
            Change::AssetCreated { path } | Change::AssetDeleted { path } => {
                if is_memo_path(path, layout) {
                    self.memos = true;
                } else {
                    self.images |= path.starts_with(&layout.content_root);
                    self.code = true;
                }
            }
            Change::Model(_) => *self = Dropped::all(),
            Change::SideFiles(_) | Change::BaseApis(_) => {}
        }
    }
}

/// The layout of a content model, with its content root as given.
fn layout(model: &ContentModel, content_root: RelPath) -> Layout {
    Layout {
        content_root,
        output_dir: RelPath::parse(&model.project.output_dir).unwrap_or_default(),
    }
}

/// The sources the index couldn't read, each with its [`ReadFailure`], in
/// its order.
fn unreadable(index: &ascribe_resolve::Project) -> Vec<(RelPath, ReadFailure)> {
    index
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
        .collect()
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
    /// The sources that couldn't be read, in their order; `None` for the
    /// index's, in path order, found when the sources are first asked for.
    failed: Option<Vec<(RelPath, ReadFailure)>>,
    /// Whether the sources are listed in path order, the files that
    /// couldn't be read among the others, rather than after them.
    by_path: bool,
    /// The sources, made the first time they're asked for, so a snapshot
    /// made for each keystroke costs nothing until it's read.
    sources: OnceLock<Vec<SourceFile>>,
    /// Every readable source file's content path, lowercased, to its real
    /// spelling, for [`SourceSet`], made the first time it's asked for.
    source_paths: OnceLock<HashMap<String, RelPath>>,
    /// The source index; `None` only while [`Workspace::apply`] updates
    /// the index of a snapshot nothing else holds.
    index: Option<Arc<ascribe_resolve::Project>>,
    /// The side files the project has, each read with its text, in the order
    /// of the list it was loaded with.
    side: Vec<Held>,
    /// The list it was loaded with.
    side_list: &'static [Entry],
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
    images: Arc<OnceLock<Vec<(RelPath, RelPath)>>>,
}

impl std::fmt::Debug for WorkspaceSnapshot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WorkspaceSnapshot")
            .field("root", &self.0.root)
            .field("sources", &self.0.sources().len())
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
    side_list: &'static [Entry],
    fs: SharedFs,
}

impl Inner {
    fn index(&self) -> &Arc<ascribe_resolve::Project> {
        // Taken only inside `Workspace::apply`, from a snapshot nothing
        // else holds, and put back or replaced before it returns.
        #[allow(clippy::expect_used)]
        self.index.as_ref().expect("a snapshot has its index")
    }

    /// The sources: the index's files in path order, then the ones that
    /// couldn't be read, with ids past every other, or all of them in path
    /// order ([`Inner::by_path`]).
    fn sources(&self) -> &[SourceFile] {
        self.sources.get_or_init(|| {
            let index = self.index();
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
            let in_index;
            let failed = match &self.failed {
                Some(failed) => failed,
                None => {
                    let mut failed = unreadable(index);
                    failed.sort_by(|a, b| a.0.cmp(&b.0));
                    in_index = failed;
                    &in_index
                }
            };
            sources.extend(
                (first..)
                    .zip(failed)
                    .map(|(id, (path, failure))| SourceFile {
                        id: FileId::new(id),
                        path: path.clone(),
                        text: Arc::from(""),
                        unreadable: Some(failure.clone()),
                    }),
            );
            if self.by_path {
                sources.sort_by(|a, b| a.path.cmp(&b.path));
            }
            sources
        })
    }

    /// Every readable source's content path, lowercased, to its spelling.
    fn source_paths(&self) -> &HashMap<String, RelPath> {
        self.source_paths.get_or_init(|| {
            // A file that couldn't be read isn't a source another file can
            // name, as in the source index.
            self.sources()
                .iter()
                .filter(|s| s.unreadable.is_none())
                .map(|s| (s.path.as_str().to_lowercase(), s.path.clone()))
                .collect()
        })
    }

    /// The snapshot of `parts`, with the index's files as its sources in
    /// path order, then `failed`, the sources that couldn't be read, with
    /// ids past every other.
    fn assemble(parts: Parts, failed: Option<Vec<(RelPath, ReadFailure)>>) -> Inner {
        let Parts {
            root,
            model,
            model_text,
            index,
            side,
            side_list,
            fs,
        } = parts;
        Inner {
            root,
            model,
            model_text,
            failed,
            by_path: false,
            sources: OnceLock::new(),
            source_paths: OnceLock::new(),
            index: Some(index),
            side,
            side_list,
            fs,
            code: Arc::default(),
            memos: Arc::default(),
            all_memos: Arc::default(),
            images: Arc::default(),
        }
    }
}

impl WorkspaceSnapshot {
    /// A snapshot of a source index with a content model given apart from
    /// it: what the language server checks an edit in, with the model the
    /// edit would leave. `fs` is the file system the index probes, the side
    /// files of `side_files` are read through it, and the source files are
    /// the index's, with the files it couldn't read after them, all in path
    /// order.
    pub fn of_index(
        root: PathBuf,
        model: Arc<ContentModel>,
        model_text: String,
        index: Arc<ascribe_resolve::Project>,
        side_files: &'static [Entry],
        fs: SharedFs,
    ) -> WorkspaceSnapshot {
        let failed = unreadable(&index);
        let side = read_side_files(side_files, &*fs);
        let mut inner = Inner::assemble(
            Parts {
                root,
                model,
                model_text,
                index,
                side,
                side_list: side_files,
                fs,
            },
            Some(failed),
        );
        inner.by_path = true;
        WorkspaceSnapshot(Arc::new(inner))
    }

    /// The project root: the directory containing `ascribe.toml`.
    pub fn root(&self) -> &Path {
        &self.0.root
    }

    /// Where the content root and the output directory are.
    pub fn layout(&self) -> &Layout {
        self.0.index().layout()
    }

    /// The content root, relative to the project root.
    pub fn content_root(&self) -> &RelPath {
        &self.layout().content_root
    }

    /// The content model.
    pub fn model(&self) -> &ContentModel {
        &self.0.model
    }

    /// [`WorkspaceSnapshot::model`], shared.
    pub fn shared_model(&self) -> &Arc<ContentModel> {
        &self.0.model
    }

    /// The content model's text.
    pub fn model_text(&self) -> &str {
        &self.0.model_text
    }

    /// The source index: every source file indexed, and what each reference
    /// names.
    pub fn index(&self) -> &Arc<ascribe_resolve::Project> {
        self.0.index()
    }

    /// The source files: the index's, with the files that couldn't be read
    /// after them, each with its [`ReadFailure`].
    pub fn sources(&self) -> &[SourceFile] {
        self.0.sources()
    }

    /// A source file by content path.
    pub fn source_at(&self, path: &RelPath) -> Option<&SourceFile> {
        self.0.sources().iter().find(|s| &s.path == path)
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

    /// Whether `other` is this snapshot, and not only one like it: what's
    /// kept with a snapshot belongs to it alone.
    pub fn same_as(&self, other: &WorkspaceSnapshot) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }

    /// The list of side files the project was loaded with: those it may
    /// have.
    pub fn side_file_list(&self) -> &'static [Entry] {
        self.0.side_list
    }

    /// The entry of the list the project was loaded with for the side file
    /// at `path`, from the project root, when it's one.
    pub fn side_entry(&self, path: &str) -> Option<&'static Entry> {
        self.0.side_list.iter().find(|entry| entry.path == path)
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
        self.0.sources().iter().filter(|s| s.unreadable.is_none())
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
            .sources()
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
                side_list: self.0.side_list,
                fs: self.0.fs.clone(),
            },
            Some(failed),
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
            .source_paths()
            .get(&path.as_str().to_lowercase())
            .is_some_and(|actual| actual == path)
    }

    fn case_twin(&self, path: &RelPath) -> Option<RelPath> {
        self.0
            .source_paths()
            .get(&path.as_str().to_lowercase())
            .filter(|actual| *actual != path)
            .cloned()
    }

    fn pages(&self) -> Vec<RelPath> {
        let mut pages: Vec<RelPath> = self.0.source_paths().values().cloned().collect();
        pages.sort();
        pages
    }
}

/// The files of a project as its source index reads them: the source texts
/// already read, each given out once, and the listing they were read from,
/// given out once, so indexing reads nothing again; everything else, and
/// anything asked for again, from the file system underneath.
///
/// Each probe is answered as the project was loaded ([`Probes`]): the
/// index lays the changes it's given over what its file system said, so
/// what it said mustn't change underneath it, while the snapshot's own
/// reads see the files as they are.
struct Texts {
    base: SharedFs,
    listing: Mutex<Option<Sources>>,
    texts: Mutex<HashMap<RelPath, Result<String, ReadFailure>>>,
    probes: Probes,
}

/// What each path probed as when the project was loaded: its answer the
/// first time it was asked, or, for a file that isn't a source that a change
/// was reported at before it was asked, what it was before the change. A
/// file that's there after the change is taken to have been missing before,
/// so a file that only changed is told as one that appeared, which redoes
/// what reads it, and never less. A source's path needs nothing: the index
/// lays its own record of the sources over what's probed.
#[derive(Clone, Default)]
struct Probes(Arc<Mutex<HashMap<RelPath, Probe>>>);

impl Probes {
    /// Remembers what `change` says each file it names that isn't a source
    /// of `index` was before, for a path not asked about yet.
    fn before(&self, change: &Change, index: &ascribe_resolve::Project) {
        let layout = index.layout();
        let mut probes = self.0.lock().unwrap_or_else(PoisonError::into_inner);
        let mut was = |path: &RelPath, content: bool, there: bool| {
            if content && index.is_source(path) {
                return;
            }
            let path = if content {
                layout.project_path(path)
            } else {
                path.clone()
            };
            probes
                .entry(path)
                .or_insert(if there { Probe::File } else { Probe::Missing });
        };
        match change {
            Change::Created { path, .. }
            | Change::Edited { path, .. }
            | Change::Unreadable { path, .. } => was(path, true, false),
            Change::Deleted { path } => was(path, true, true),
            Change::Renamed { from, to } => {
                was(from, true, true);
                was(to, true, false);
            }
            Change::AssetCreated { path } => was(path, false, false),
            Change::AssetDeleted { path } => was(path, false, true),
            Change::Model(_) | Change::SideFiles(_) | Change::BaseApis(_) => {}
        }
    }
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
            probes: Probes::default(),
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
            probes: Probes::default(),
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
        let mut probes = self.probes.0.lock().unwrap_or_else(PoisonError::into_inner);
        if let Some(answer) = probes.get(project_path) {
            return answer.clone();
        }
        let answer = self.base.probe(project_path);
        probes.insert(project_path.clone(), answer.clone());
        answer
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
