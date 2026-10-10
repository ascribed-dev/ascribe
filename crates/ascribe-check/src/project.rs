//! The project the checks run on: the content model and the source files.

use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::{Arc, OnceLock};

use ascribe_core::{Date, FileId, Issue, LineIndex, RelPath};
use ascribe_model::{ContentModel, LOCK_FILE, Lock, LockedSource};
use ascribe_resolve::{CodeFile, CodeFiles, DiskFs, FileSystem, Layout, SourceSet};

use crate::Diagnostic;

pub use ascribe_model::MODEL_FILE;

/// The id of `ascribe.lock` (SPEC §7.4), for locations in it: past any
/// source file's, and before the code files'.
pub const LOCK_FILE_ID: FileId = FileId::new(0x7FFF_FFFF);

/// The id of the first image file under the content root, for locations
/// in image files: the image checks list them in path order, and number them
/// from here ([`Project::images`]).
pub const IMAGE_FILE_IDS: u32 = 0x4000_0000;

/// One source file: a Markdown file under the content root.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SourceFile {
    /// The file's id, used in every location that points into it.
    pub id: FileId,
    /// The path relative to the content root, `/`-separated.
    pub path: RelPath,
    /// The text; empty when the file couldn't be read.
    pub text: String,
    /// Why the file couldn't be read, if it couldn't. It's reported as
    /// `source-unreadable`, and nothing else is checked in it (SPEC §8.2).
    pub unreadable: Option<ReadFailure>,
}

/// Why a source file couldn't be read.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReadFailure {
    /// The operating system's reason.
    pub reason: String,
    /// Whether the file was read but isn't valid UTF-8.
    pub not_utf8: bool,
}

/// What a file id names, for reporting.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FileEntry<'a> {
    /// The id.
    pub id: FileId,
    /// The path relative to the project root, `/`-separated: how a person
    /// would type it. `ascribe.toml` for the content model.
    pub display_path: String,
    /// The text.
    pub text: &'a str,
    /// The path relative to the content root, for a source file or an image
    /// file under it; `None` for the content model and the lock.
    pub content_path: Option<&'a RelPath>,
}

/// Why a project couldn't be loaded.
#[derive(Debug, thiserror::Error)]
pub enum LoadError {
    /// A file or directory couldn't be read.
    #[error("can't read {path}: {message}")]
    Read {
        /// The path.
        path: String,
        /// What went wrong.
        message: String,
    },
    /// The content model has errors.
    #[error("{} has errors", MODEL_FILE)]
    Model {
        /// The content model's text, for showing snippets.
        text: String,
        /// The problems found, with the warnings found before them, in
        /// source order. Every location is in file [`FileId::new(0)`](FileId).
        diagnostics: Vec<Diagnostic>,
    },
}

impl ascribe_core::Coded for LoadError {
    fn code(&self) -> &'static str {
        match self {
            LoadError::Read { .. } => "project_unreadable",
            LoadError::Model { .. } => "model_invalid",
        }
    }
}

/// Why [`Project::locate`] found no content model.
#[derive(Debug, thiserror::Error)]
pub enum LocateError {
    /// The current directory, where the search starts, can't be read.
    #[error("can't read the current directory: {0}")]
    CurrentDir(io::Error),
    /// No `ascribe.toml` is in the current directory or a parent.
    #[error(
        "no {} found in {} or any parent directory; run ascribe from a project, or pass --config",
        MODEL_FILE,
        dir.display()
    )]
    NotFound {
        /// The directory the search started in.
        dir: PathBuf,
    },
    /// The content model named isn't a file.
    #[error("{} doesn't exist or isn't a file", path.display())]
    NotAFile {
        /// The path named.
        path: PathBuf,
    },
}

impl ascribe_core::Coded for LocateError {
    fn code(&self) -> &'static str {
        match self {
            LocateError::CurrentDir(_) => "current_dir_unreadable",
            LocateError::NotFound { .. } => "model_not_found",
            LocateError::NotAFile { .. } => "model_not_a_file",
        }
    }
}

/// A content model as [`Project::load_model`] loads it.
#[derive(Clone, Debug)]
pub struct ModelFile {
    /// The project root: the directory containing `ascribe.toml`.
    pub root: PathBuf,
    /// The content model's text.
    pub text: String,
    /// The content model.
    pub model: ContentModel,
}

/// A documentation set: its content model and source files.
///
/// The content model is file id 0, and the source files have ids 1, 2, … in
/// path order when a project is loaded (the same numbering as
/// `ascribe_resolve::Project`). A project built from parts may use any
/// distinct ids from 1 (it finds a file by id, not by position): the language
/// server gives the ids of an `ascribe_resolve::Snapshot`, which are stable
/// per path across updates and have gaps where files were deleted
/// (`ascribe_resolve::incremental`). Source texts are held in memory: the
/// language server builds a `Project` from its open buffers, and the command
/// line from the files on disk. Files that aren't sources (images, other
/// downloads) are looked for on disk, through `ascribe_resolve`'s
/// [`FileSystem`], when a reference needs them; a source file counts as
/// existing even when it's only in memory.
///
/// Which files are sources, and every rule about what a reference names, are
/// `ascribe_resolve`'s, so this crate and the source index can't disagree.
#[derive(Clone, Debug)]
pub struct Project {
    root: PathBuf,
    layout: Layout,
    model: ContentModel,
    model_text: String,
    sources: Vec<SourceFile>,
    model_warnings: Vec<Diagnostic>,
    /// Every source file's content path, lowercased, to its real spelling,
    /// for [`SourceSet`].
    source_paths: HashMap<String, RelPath>,
    fs: Files,
    /// The code files snippets have read, read as the checks ask for them.
    code: Arc<CodeFiles>,
    /// The text of `ascribe.lock`, when there is one.
    lock_text: Option<String>,
    /// `ascribe.lock` read, or its problems, when there's one.
    lock: Option<Result<Lock, Vec<Issue>>>,
    /// The image files under the content root, by path from the project
    /// root and from the content root, once the image checks have listed
    /// them.
    images: OnceLock<Vec<(RelPath, RelPath)>>,
    /// The day the checks run on, which `review-overdue` compares with.
    today: Option<Date>,
}

/// The file system a project probes for what isn't a source file. It has a
/// `Debug` of its own because `dyn FileSystem` doesn't.
#[derive(Clone)]
struct Files(Arc<dyn FileSystem + Send + Sync>);

impl std::fmt::Debug for Files {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("FileSystem")
    }
}

impl Project {
    /// The nearest `ascribe.toml` in `start` or one of its parents.
    pub fn find_config(start: &Path) -> Option<PathBuf> {
        // Outside FileSystem: looking for a project, before there is one.
        start
            .ancestors()
            .map(|dir| dir.join(MODEL_FILE))
            .find(|p| p.is_file())
    }

    /// The content model a command works on: `config` when given (a file, or
    /// a directory that holds `ascribe.toml`), or else the nearest
    /// `ascribe.toml` in the current directory or a parent.
    ///
    /// # Errors
    ///
    /// The current directory can't be read, no `ascribe.toml` is found, or
    /// `config` names none.
    pub fn locate(config: Option<&Path>) -> Result<PathBuf, LocateError> {
        // Outside FileSystem: the path given with `--config`, or the one found,
        // before there's a project.
        let config = match config {
            Some(path) if path.is_dir() => path.join(MODEL_FILE),
            Some(path) => path.to_owned(),
            None => {
                let cwd = std::env::current_dir().map_err(LocateError::CurrentDir)?;
                Project::find_config(&cwd).ok_or(LocateError::NotFound { dir: cwd })?
            }
        };
        if config.is_file() {
            Ok(config)
        } else {
            Err(LocateError::NotAFile { path: config })
        }
    }

    /// Loads the project whose content model is at `config`: the model, then
    /// every source file under its content root, as
    /// [`ascribe_resolve::FileSystem::sources`] finds them.
    ///
    /// A source file or directory that can't be read, or a source file that
    /// isn't valid UTF-8, is still a source file, with its [`ReadFailure`];
    /// `check_files` reports it and checks the rest.
    pub fn load(config: &Path) -> Result<Project, LoadError> {
        let ModelFile { root, text, model } = Project::load_model(config)?;
        let content_root =
            RelPath::parse(&model.project.content_root).map_err(|e| LoadError::Read {
                path: config.display().to_string(),
                message: e.to_string(),
            })?;
        let sources = Project::read_sources(&root, &content_root)?;
        Ok(Project::from_parts(
            root,
            content_root,
            model,
            text,
            sources,
        ))
    }

    /// Loads only the content model at `config`, the first step of
    /// [`Project::load`], for a command that doesn't need the pages read:
    /// `ascribe fmt`, which reads the files it formats itself.
    ///
    /// # Errors
    ///
    /// The file can't be read or isn't valid UTF-8 ([`LoadError::Read`]), or
    /// the content model has errors ([`LoadError::Model`]).
    pub fn load_model(config: &Path) -> Result<ModelFile, LoadError> {
        // Outside FileSystem: the content model says where the content root is,
        // so it's read before there's a FileSystem to read through.
        let text = fs::read(config)
            .map_err(|e| read_error(config, e))
            .and_then(|bytes| {
                String::from_utf8(bytes).map_err(|_| LoadError::Read {
                    path: config.display().to_string(),
                    message: "the file isn't valid UTF-8".into(),
                })
            })?;
        let root = match config.parent() {
            Some(p) if !p.as_os_str().is_empty() => p.to_owned(),
            _ => PathBuf::from("."),
        };
        match ascribe_model::load_str_in(&text, FileId::new(0), &root) {
            Ok(model) => Ok(ModelFile { root, text, model }),
            Err(issues) => Err(LoadError::Model {
                text,
                diagnostics: issues.iter().map(Diagnostic::from_issue).collect(),
            }),
        }
    }

    /// Reads every source file under `root/content_root`. The files that
    /// could be read have ids from 1, in path order, as in
    /// `ascribe_resolve::Project`; those that couldn't come after them, with
    /// their [`ReadFailure`].
    pub fn read_sources(root: &Path, content_root: &RelPath) -> Result<Vec<SourceFile>, LoadError> {
        let layout = Layout {
            content_root: content_root.clone(),
            output_dir: RelPath::root(),
        };
        let disk = DiskFs::new(root, &layout);
        let found = disk.sources();
        let mut sources = Vec::with_capacity(found.paths.len());
        let mut failed = Vec::new();
        for path in found.paths {
            match disk.read(&path) {
                Ok(text) => sources.push((path, text)),
                Err(e) => failed.push((
                    path,
                    ReadFailure {
                        not_utf8: e.kind() == io::ErrorKind::InvalidData,
                        reason: e.to_string(),
                    },
                )),
            }
        }
        failed.extend(found.unreadable.into_iter().map(|u| {
            (
                u.path,
                ReadFailure {
                    reason: u.reason,
                    not_utf8: false,
                },
            )
        }));
        let out = sources
            .into_iter()
            .map(|(path, text)| (path, text, None))
            .chain(failed.into_iter().map(|(p, f)| (p, String::new(), Some(f))))
            .enumerate()
            .map(|(i, (path, text, unreadable))| SourceFile {
                id: FileId::new(i as u32 + 1),
                path,
                text,
                unreadable,
            })
            .collect();
        Ok(out)
    }

    /// A project from parts the caller already has: the language server's
    /// buffers, or a test.
    ///
    /// `root` is the project root (the directory of `ascribe.toml`),
    /// `content_root` is relative to it, and `model_text` is the content
    /// model's text (for showing snippets). `sources` should have distinct ids
    /// from 1, not necessarily consecutive; [`Project::from_sources`] assigns
    /// 1, 2, … in path order.
    pub fn from_parts(
        root: PathBuf,
        content_root: RelPath,
        model: ContentModel,
        model_text: String,
        sources: Vec<SourceFile>,
    ) -> Project {
        Project::assemble(root, content_root, model, model_text, sources, None)
    }

    /// [`Project::from_parts`], with the file system the checks probe for what
    /// isn't a source file (images, downloads, a route-like link's target)
    /// instead of the disk.
    ///
    /// The language server passes the file system it keeps its source index
    /// on: the disk with the files the editor and the file watcher have
    /// reported appearing or disappearing layered over it, so an unsaved or
    /// just-created image is seen by the file-level checks exactly as it is by
    /// the source index. Source texts still come from `sources`.
    pub fn from_parts_with_fs(
        root: PathBuf,
        content_root: RelPath,
        model: ContentModel,
        model_text: String,
        sources: Vec<SourceFile>,
        fs: Arc<dyn FileSystem + Send + Sync>,
    ) -> Project {
        Project::assemble(root, content_root, model, model_text, sources, Some(fs))
    }

    fn assemble(
        root: PathBuf,
        content_root: RelPath,
        model: ContentModel,
        model_text: String,
        sources: Vec<SourceFile>,
        fs: Option<Arc<dyn FileSystem + Send + Sync>>,
    ) -> Project {
        let model_warnings = model.warnings.iter().map(Diagnostic::from_issue).collect();
        // A file that couldn't be read isn't a source another file can name,
        // as in the source index.
        let source_paths = sources
            .iter()
            .filter(|s| s.unreadable.is_none())
            .map(|s| (s.path.as_str().to_lowercase(), s.path.clone()))
            .collect();
        let layout = Layout {
            content_root,
            output_dir: RelPath::parse(&model.project.output_dir).unwrap_or_default(),
        };
        // A caller that doesn't bring a file system is a one-shot command (or a
        // test): the disk doesn't change while it runs, so listings are kept.
        let fs: Arc<dyn FileSystem + Send + Sync> =
            fs.unwrap_or_else(|| Arc::new(DiskFs::with_listing_cache(&root, &layout)));
        // The lock is read like any other file beside the content model, so
        // a project in memory can have one.
        let lock_text = RelPath::parse(LOCK_FILE)
            .ok()
            .and_then(|path| fs.read_file(&path).ok())
            .map(|bytes| String::from_utf8_lossy(&bytes).into_owned());
        let lock = lock_text
            .as_deref()
            .map(|text| Lock::parse(text, LOCK_FILE_ID));
        Project {
            root,
            layout,
            model,
            model_text,
            sources,
            model_warnings,
            source_paths,
            fs: Files(fs),
            code: Arc::default(),
            lock_text,
            lock,
            images: OnceLock::new(),
            today: None,
        }
    }

    /// Source files from `(path, text)` pairs, sorted by path, with ids from 1.
    pub fn from_sources(files: impl IntoIterator<Item = (RelPath, String)>) -> Vec<SourceFile> {
        let mut files: Vec<_> = files.into_iter().collect();
        files.sort_by(|a, b| a.0.cmp(&b.0));
        files
            .into_iter()
            .enumerate()
            .map(|(i, (path, text))| SourceFile {
                id: FileId::new(i as u32 + 1),
                path,
                text,
                unreadable: None,
            })
            .collect()
    }

    /// The project root: the directory containing `ascribe.toml`.
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// The content root, relative to the project root.
    pub fn content_root(&self) -> &RelPath {
        &self.layout.content_root
    }

    /// Where the content root and the output directory are.
    pub fn layout(&self) -> &Layout {
        &self.layout
    }

    /// The source index over the same files: what a command that resolves
    /// builds (`build`, `diff`, `drift`, `sources`) works on. It's read again
    /// through [`Project::file_system`], so the two can't disagree about which
    /// files exist; reading them once is left for later.
    pub fn index(&self) -> ascribe_resolve::Project {
        ascribe_resolve::Project::load(
            Arc::new(self.model.clone()),
            self.layout.clone(),
            self.file_system(),
        )
    }

    /// The source index over the texts this project holds, which can differ
    /// from the files on disk: text laid over it with
    /// [`Project::with_source`], or a project built in memory. Only its
    /// sources are what's held; images and other files are read as
    /// [`Project::index`] reads them.
    pub fn held_index(&self) -> ascribe_resolve::Project {
        crate::page::held_index(self)
    }

    /// The content model's text.
    pub fn model_text(&self) -> &str {
        &self.model_text
    }

    /// The files the checks probe for what isn't a source file held in memory:
    /// the disk, or the file system given to [`Project::from_parts_with_fs`].
    pub fn file_system(&self) -> &dyn FileSystem {
        &*self.fs.0
    }

    /// The code files the checks have read for snippets (SPEC §4.8), so far.
    /// A diagnostic's related location can be in one: its id is the code
    /// file's, from [`ascribe_resolve::snippet::CODE_FILE_IDS`] up.
    pub fn code_files(&self) -> &CodeFiles {
        &self.code
    }

    /// The code file with this id, if the checks have read it.
    pub fn code_file(&self, id: FileId) -> Option<Arc<CodeFile>> {
        self.code.by_id(id)
    }

    /// The content model.
    pub fn model(&self) -> &ContentModel {
        &self.model
    }

    /// The source files, in path order.
    pub fn sources(&self) -> &[SourceFile] {
        &self.sources
    }

    /// The diagnostics for the content model's warnings.
    pub fn model_warnings(&self) -> &[Diagnostic] {
        &self.model_warnings
    }

    /// A source file by content path.
    pub fn source_at(&self, path: &RelPath) -> Option<&SourceFile> {
        self.sources.iter().find(|s| &s.path == path)
    }

    /// The text of `ascribe.lock`, when the project has one.
    pub fn lock_text(&self) -> Option<&str> {
        self.lock_text.as_deref()
    }

    /// `ascribe.lock` read, or its problems, when the project has one.
    pub(crate) fn lock(&self) -> Option<&Result<Lock, Vec<Issue>>> {
        self.lock.as_ref()
    }

    /// The pin of the source `name` in another repository, when
    /// `ascribe.lock` can be read and pins it to the repository the content
    /// model names.
    pub(crate) fn pin(&self, name: &str) -> Option<&LockedSource> {
        let url = &self.model.source(name)?.git.as_ref()?.url;
        // A lock that can't be read is reported by the sources check.
        let lock = self.lock.as_ref()?.as_ref().ok()?;
        lock.source(name).filter(|l| &l.git == url)
    }

    /// The image files under the content root, by path from the project
    /// root and from the content root, in path order, listed by `list` the
    /// first time they're asked for. The image at position `i` has the id
    /// `IMAGE_FILE_IDS + i`.
    pub(crate) fn images_or(
        &self,
        list: impl FnOnce() -> Vec<(RelPath, RelPath)>,
    ) -> &[(RelPath, RelPath)] {
        self.images.get_or_init(list)
    }

    /// The ids of the image files the image checks listed; none when they
    /// haven't run.
    pub fn image_ids(&self) -> impl Iterator<Item = FileId> + '_ {
        (0..self.images.get().map_or(0, Vec::len)).map(image_id)
    }

    /// What a file id names.
    pub fn file(&self, id: FileId) -> Option<FileEntry<'_>> {
        if let Some(i) = id.index().checked_sub(IMAGE_FILE_IDS)
            && id.index() < LOCK_FILE_ID.index()
        {
            let (path, content) = self.images.get()?.get(i as usize)?;
            return Some(FileEntry {
                id,
                display_path: path.to_string(),
                text: "",
                content_path: Some(content),
            });
        }
        if id == FileId::new(0) {
            return Some(FileEntry {
                id,
                display_path: MODEL_FILE.to_owned(),
                text: &self.model_text,
                content_path: None,
            });
        }
        if id == LOCK_FILE_ID {
            return self.lock_text.as_deref().map(|text| FileEntry {
                id,
                display_path: LOCK_FILE.to_owned(),
                text,
                content_path: None,
            });
        }
        let source = self.sources.iter().find(|s| s.id == id)?;
        Some(FileEntry {
            id,
            display_path: self
                .layout
                .content_root
                .join(source.path.as_str())
                .map_or_else(|_| source.path.to_string(), |p| p.to_string()),
            text: &source.text,
            content_path: Some(&source.path),
        })
    }

    /// The path a report shows for a file id: [`FileEntry::display_path`] for
    /// the content model, the lock, and a source file, and the code file's
    /// path for a code file a snippet read.
    pub fn display_path(&self, id: FileId) -> Option<String> {
        match self.file(id) {
            Some(entry) => Some(entry.display_path),
            None => self.code_file(id).map(|code| code.path.to_string()),
        }
    }

    /// This project with the source file at `path` (a content path) holding
    /// `text` instead of what was read, or added when there's none: a file
    /// checked before it's saved. The ids are numbered again, in path order.
    pub fn with_source(&self, path: &RelPath, text: String) -> Project {
        let (readable, unreadable): (Vec<&SourceFile>, Vec<&SourceFile>) = self
            .sources
            .iter()
            .filter(|s| &s.path != path)
            .partition(|s| s.unreadable.is_none());
        let mut files: Vec<(RelPath, String)> = readable
            .into_iter()
            .map(|s| (s.path.clone(), s.text.clone()))
            .collect();
        files.push((path.clone(), text));
        let mut sources = Project::from_sources(files);
        let first = sources.len() as u32 + 1;
        sources.extend(unreadable.into_iter().enumerate().map(|(i, s)| SourceFile {
            id: FileId::new(first + i as u32),
            ..s.clone()
        }));
        Project::assemble(
            self.root.clone(),
            self.layout.content_root.clone(),
            self.model.clone(),
            self.model_text.clone(),
            sources,
            Some(self.fs.0.clone()),
        )
        .with_today(self.today)
    }

    /// This project, checked as on `today`: the day `review-overdue`
    /// compares a page's review date with. A library doesn't read the clock,
    /// so a project has no day until its caller gives it one, and without
    /// one, no review is overdue.
    #[must_use]
    pub fn with_today(mut self, today: Option<Date>) -> Project {
        self.today = today;
        self
    }

    /// The day the checks run on, if the caller gave one.
    pub fn today(&self) -> Option<Date> {
        self.today
    }

    /// A line index for a file's text, for turning spans into lines and columns.
    pub fn line_index(&self, id: FileId) -> Option<LineIndex> {
        self.file(id).map(|f| LineIndex::new(f.text))
    }
}

impl SourceSet for Project {
    fn contains(&self, path: &RelPath) -> bool {
        self.source_paths
            .get(&path.as_str().to_lowercase())
            .is_some_and(|actual| actual == path)
    }

    fn case_twin(&self, path: &RelPath) -> Option<RelPath> {
        self.source_paths
            .get(&path.as_str().to_lowercase())
            .filter(|actual| *actual != path)
            .cloned()
    }

    fn pages(&self) -> Vec<RelPath> {
        let mut pages: Vec<RelPath> = self.source_paths.values().cloned().collect();
        pages.sort();
        pages
    }
}

/// The id of the image at position `i` of [`Project::images`].
pub(crate) fn image_id(i: usize) -> FileId {
    // A project has far fewer images than the ids from IMAGE_FILE_IDS to the
    // lock's.
    #[allow(clippy::cast_possible_truncation)]
    FileId::new(IMAGE_FILE_IDS + i as u32)
}

fn read_error(path: &Path, e: std::io::Error) -> LoadError {
    LoadError::Read {
        path: path.display().to_string(),
        message: e.to_string(),
    }
}
