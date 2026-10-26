//! The project the checks run on: the content model and the source files.

use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::{Arc, OnceLock};

use ascribe_core::path::{normalize, relative_path};
use ascribe_core::{Date, FileId, Issue, LineIndex, RelPath, Span};
use ascribe_model::{
    AddressFile, ContentModel, Lock, LockedSource, NavFile, PermalinkList, SourceBound,
};
use ascribe_resolve::side_file::{self, Held, SideFile};
use ascribe_resolve::{
    CodeFile, CodeFiles, Committed, DiskFs, FileSystem, Layout, MemoFile, MemoStore, Personal,
    SourceSet,
};

use crate::Diagnostic;
use crate::baseline::{BASELINE_FILE_ID, Baseline, BaselineError};
use crate::checks::template::Template;
use crate::side_files::{SIDE_FILES, side_file_by_id};

pub use ascribe_model::MODEL_FILE;

/// The id of `ascribe.lock` (SPEC §7.4), for locations in it.
pub const LOCK_FILE_ID: FileId = side_file::Lock::ID;

/// The id of `ascribe.permalinks` (SPEC §7.5), for locations in it.
pub const PERMALINKS_FILE_ID: FileId = side_file::Permalinks::ID;

/// The id of `ascribe.nav` (SPEC §7.8), for locations in it.
pub const NAV_FILE_ID: FileId = side_file::Nav::ID;

/// The id of `ascribe.addresses` (SPEC §7.10), for locations in it.
pub const ADDRESSES_FILE_ID: FileId = side_file::Addresses::ID;

/// The id of the first image file under the content root, for locations
/// in image files: the image checks list them in path order, and number them
/// from here ([`Project::images`]).
pub const IMAGE_FILE_IDS: u32 = 0x4000_0000;

/// The id of the first memo file (SPEC §7.7), for locations in memo files:
/// [`Project::checked_memos`] lists them, and numbers them from here.
pub const MEMO_FILE_IDS: u32 = 0x2000_0000;

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
    /// file under it; `None` for the content model and the side files.
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
    #[error("{}", not_found(dir, below))]
    NotFound {
        /// The directory the search started in.
        dir: PathBuf,
        /// The folders below it that hold an `ascribe.toml`, as
        /// [`Project::find_below`] finds them: the first eleven, of which
        /// the message names ten.
        below: Vec<PathBuf>,
    },
    /// The content model named isn't a file.
    #[error("{} doesn't exist or isn't a file", path.display())]
    NotAFile {
        /// The path named.
        path: PathBuf,
    },
}

/// The most projects below the directory searched that
/// [`LocateError::NotFound`] names.
const NOT_FOUND_NAMED: usize = 10;

impl LocateError {
    /// No `ascribe.toml` in `dir` or a parent; the error names the projects
    /// below `dir`, if there are any.
    pub fn not_found(dir: PathBuf) -> LocateError {
        let below = Project::find_below(&dir, NOT_FOUND_NAMED + 1);
        LocateError::NotFound { dir, below }
    }
}

/// What [`LocateError::NotFound`] says: with the projects below, where to
/// run ascribe instead, each folder from the current directory.
fn not_found(dir: &Path, below: &[PathBuf]) -> String {
    let none = format!(
        "no {MODEL_FILE} found in {} or any parent directory",
        dir.display()
    );
    // Each folder as it's typed where ascribe was run.
    let base = normalize(&std::env::current_dir().unwrap_or_else(|_| dir.to_owned()));
    let named: Vec<String> = below
        .iter()
        .take(NOT_FOUND_NAMED)
        .map(|child| match relative_path(&base, &normalize(child)) {
            Some(rel) => rel.as_str().to_owned(),
            None => child.display().to_string(),
        })
        .collect();
    match named.as_slice() {
        [] => format!("{none}; run ascribe from a project, or pass --config"),
        [one] => format!(
            "{none}; the project below it is in {one}: run ascribe there, or pass --config {one}"
        ),
        several => {
            let more = if below.len() > NOT_FOUND_NAMED {
                ", and more"
            } else {
                ""
            };
            format!(
                "{none}; the projects below it are in {}{more}: run ascribe in one of them, \
                 or pass --config with its folder",
                several.join(", ")
            )
        }
    }
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
    /// The side files the project has ([`SIDE_FILES`]), each read with its
    /// text, in the order of the list.
    side: Vec<Held>,
    /// The image files under the content root, by path from the project
    /// root and from the content root, once the image checks have listed
    /// them.
    images: OnceLock<Vec<(RelPath, RelPath)>>,
    /// The source index a broken link's closest targets are looked for in,
    /// and the page-level checks read: the one the caller gave
    /// ([`Project::with_link_index`]), or one built from the held texts the
    /// first time either needs it, so it's built once.
    link_index: OnceLock<Arc<ascribe_resolve::Project>>,
    /// The day the checks run on, which `review-overdue` compares with.
    today: Option<Date>,
    /// The templates the content types name, by type, read the first time a
    /// page of one is checked.
    templates: Arc<OnceLock<HashMap<String, Arc<Template>>>>,
    /// The committed memo files, read the first time they're asked for.
    memos: Arc<OnceLock<Vec<MemoFile>>>,
    /// The committed memo files and then the personal ones, read the first
    /// time they're asked for.
    all_memos: Arc<OnceLock<Vec<MemoFile>>>,
    /// Whether the checks read the personal memos too: the editor's do, and
    /// `ascribe check`'s don't ([`Project::with_personal_memos`]).
    personal_memos: bool,
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

    /// The folders below `dir` that hold an `ascribe.toml`, in path order,
    /// at most `limit`: a project's folder, and those below it too. Folders
    /// whose names start with `.`, `node_modules`, and `target` aren't
    /// looked in.
    pub fn find_below(dir: &Path, limit: usize) -> Vec<PathBuf> {
        fn walk(dir: &Path, limit: usize, found: &mut Vec<PathBuf>) {
            // Outside FileSystem: looking for projects, before there is one.
            let Ok(entries) = fs::read_dir(dir) else {
                return;
            };
            let mut dirs: Vec<PathBuf> = entries
                .flatten()
                .filter(|e| e.file_type().is_ok_and(|t| t.is_dir()))
                .filter(|e| {
                    let name = e.file_name();
                    let name = name.to_string_lossy();
                    !name.starts_with('.') && name != "node_modules" && name != "target"
                })
                .map(|e| e.path())
                .collect();
            dirs.sort();
            for child in dirs {
                if found.len() >= limit {
                    return;
                }
                // Outside FileSystem: as above.
                if child.join(MODEL_FILE).is_file() {
                    found.push(child.clone());
                }
                walk(&child, limit, found);
            }
        }
        let mut found = Vec::new();
        walk(dir, limit, &mut found);
        found
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
                Project::find_config(&cwd).ok_or_else(|| LocateError::not_found(cwd))?
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
        Project::load_bounded(config, SourceBound::Anywhere)
    }

    /// [`Project::load`], with a local source bounded by `bound` when no git
    /// repository is around the project ([`SourceBound`]).
    ///
    /// # Errors
    ///
    /// As [`Project::load`].
    pub fn load_bounded(config: &Path, bound: SourceBound<'_>) -> Result<Project, LoadError> {
        let ModelFile { root, text, model } = Project::load_model_bounded(config, bound)?;
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
        Project::load_model_bounded(config, SourceBound::Anywhere)
    }

    /// [`Project::load_model`], with a local source bounded by `bound` when
    /// no git repository is around the project ([`SourceBound`]).
    ///
    /// # Errors
    ///
    /// As [`Project::load_model`].
    pub fn load_model_bounded(
        config: &Path,
        bound: SourceBound<'_>,
    ) -> Result<ModelFile, LoadError> {
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
        match crate::type_files::load_model(&text, &root, bound) {
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
        // The side files are read like any other file beside the content
        // model, so a project in memory can have them.
        let side = SIDE_FILES
            .iter()
            .filter_map(|entry| entry.read(&*fs))
            .collect();
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
            side,
            images: OnceLock::new(),
            link_index: OnceLock::new(),
            today: None,
            templates: Arc::default(),
            memos: Arc::default(),
            all_memos: Arc::default(),
            personal_memos: false,
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

    /// The side file `F` read, when the project has it ([`SIDE_FILES`]).
    pub fn side_file<F: SideFile>(&self) -> Option<&F::Read> {
        self.side.iter().find_map(Held::get::<F>)
    }

    /// The text of the side file at `path`, from the project root, when the
    /// project has it.
    pub fn side_text(&self, path: &str) -> Option<&str> {
        self.side
            .iter()
            .find(|held| held.path() == path)
            .map(Held::text)
    }

    /// The text of `ascribe.lock`, when the project has one.
    pub fn lock_text(&self) -> Option<&str> {
        self.side_text(side_file::Lock::PATH)
    }

    /// `ascribe.lock` read, or its problems, when the project has one.
    pub(crate) fn lock(&self) -> Option<&Result<Lock, Vec<Issue>>> {
        self.side_file::<side_file::Lock>()
    }

    /// The text of `ascribe.permalinks`, when the project has one.
    pub fn permalinks_text(&self) -> Option<&str> {
        self.side_text(side_file::Permalinks::PATH)
    }

    /// `ascribe.permalinks` read, or its problems, when the project has one.
    pub fn permalinks(&self) -> Option<&Result<PermalinkList, Vec<Issue>>> {
        self.side_file::<side_file::Permalinks>()
    }

    /// `ascribe.nav` read, or its problems, when the project has one.
    pub fn nav(&self) -> Option<&Result<NavFile, Vec<Issue>>> {
        self.side_file::<side_file::Nav>()
    }

    /// The text of `ascribe.addresses`, when the project has one.
    pub fn addresses_text(&self) -> Option<&str> {
        self.side_text(side_file::Addresses::PATH)
    }

    /// `ascribe.addresses` read, or its problems, when the project has one.
    pub fn addresses(&self) -> Option<&Result<AddressFile, Vec<Issue>>> {
        self.side_file::<side_file::Addresses>()
    }

    /// The text of `ascribe.baseline`, when the project has one.
    pub fn baseline_text(&self) -> Option<&str> {
        self.side_text(crate::side_files::Baseline::PATH)
    }

    /// `ascribe.baseline` read, or the line that doesn't parse, when the
    /// project has one.
    pub fn baseline(&self) -> Option<&Result<Baseline, BaselineError>> {
        self.side_file::<crate::side_files::Baseline>()
    }

    /// The pin of the source `name` in another repository, when
    /// `ascribe.lock` can be read and pins it to the repository the content
    /// model names.
    pub(crate) fn pin(&self, name: &str) -> Option<&LockedSource> {
        let url = &self.model.source(name)?.git.as_ref()?.url;
        // A lock that can't be read is reported by the sources check.
        let lock = self.lock()?.as_ref().ok()?;
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
        // A side file's id can be in the images' range, so it's looked for
        // first.
        if let Some(entry) = side_file_by_id(id) {
            return self.side_text(entry.path).map(|text| FileEntry {
                id,
                display_path: entry.path.to_owned(),
                text,
                content_path: None,
            });
        }
        if let Some(i) = id.index().checked_sub(MEMO_FILE_IDS)
            && id.index() < IMAGE_FILE_IDS
        {
            let memo = self.checked_memos().get(i as usize)?;
            return Some(FileEntry {
                id,
                display_path: memo.path.to_string(),
                text: &memo.text,
                content_path: None,
            });
        }
        if let Some(i) = id.index().checked_sub(IMAGE_FILE_IDS)
            && id.index() < BASELINE_FILE_ID.index()
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
    /// the content model, a side file, and a source file, and the code file's
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
        .with_reads_of(self)
    }

    /// This project, with the templates and memo files another read of the
    /// same project has, since a page's text doesn't change them.
    fn with_reads_of(self, other: &Project) -> Project {
        Project {
            templates: other.templates.clone(),
            memos: other.memos.clone(),
            all_memos: other.all_memos.clone(),
            ..self
        }
    }

    /// The committed memo files (SPEC §7.7), in path order, read through the
    /// project's [`FileSystem`] the first time they're asked for. None when
    /// `memos/` is inside the content root, where its files are pages.
    pub fn memos(&self) -> &[MemoFile] {
        self.memos
            .get_or_init(|| Committed::new(self.file_system(), &self.layout).memos())
    }

    /// The committed memo files, then the personal ones in `.ascribe/memos/`,
    /// each in path order: what `ascribe outline` and `ascribe refs` list.
    pub fn all_memos(&self) -> &[MemoFile] {
        self.all_memos.get_or_init(|| {
            let mut all = self.memos().to_vec();
            all.extend(Personal::new(self.file_system()).memos());
            all
        })
    }

    /// This project, with the checks reading the personal memos as well as
    /// the committed ones: the editor's, which reports a personal memo's
    /// problems as it reports a team memo's.
    pub fn with_personal_memos(self) -> Project {
        Project {
            personal_memos: true,
            ..self
        }
    }

    /// The memo files the checks read: the committed ones, and the personal
    /// ones after them when [`Project::with_personal_memos`] says so. The
    /// memo at position `i` has the id `MEMO_FILE_IDS + i`.
    pub fn checked_memos(&self) -> &[MemoFile] {
        if self.personal_memos {
            self.all_memos()
        } else {
            self.memos()
        }
    }

    /// The ids of the memo files the checks read.
    pub fn memo_ids(&self) -> impl Iterator<Item = FileId> + '_ {
        (0..self.checked_memos().len()).map(memo_id)
    }

    /// The template content type `name` names, read through the project's
    /// [`FileSystem`]. None when it names none, or when the file can't be
    /// read, which is the content model's error (`model-type-template`).
    pub(crate) fn template(&self, name: &str) -> Option<Arc<Template>> {
        let templates = self.templates.get_or_init(|| {
            let mut out = HashMap::new();
            for t in &self.model.types {
                let Some(path) = t
                    .template
                    .as_ref()
                    .and_then(|f| RelPath::parse(&f.path).ok())
                else {
                    continue;
                };
                let Some(text) = self
                    .fs
                    .0
                    .read_file(&path)
                    .ok()
                    .and_then(|bytes| String::from_utf8(bytes).ok())
                else {
                    continue;
                };
                out.insert(t.name.clone(), Arc::new(Template::read(&text, &self.model)));
            }
            out
        });
        templates.get(name).cloned()
    }

    /// Where `text`, a page of type `type_name`, still has what the type's
    /// template left to fill in, in source order: each frontmatter field
    /// whose value is the template's, and each placeholder of the body,
    /// whole, from the first piece of one that runs across blocks to its
    /// last. They're found as `template-placeholder-left` finds them, whether
    /// or not the check is on. Empty when the type has no template, or its
    /// template can't be read.
    pub fn template_places(&self, type_name: &str, text: &str) -> Vec<Span> {
        self.template(type_name)
            .map(|template| crate::checks::template::places(&template, text, &self.model))
            .unwrap_or_default()
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

    /// This project, with `index` as the source index a broken link's
    /// closest targets are looked for in, so none is built for them: the
    /// language server gives its incremental index. `index` must be an index
    /// of the same files.
    #[must_use]
    pub fn with_link_index(self, index: Arc<ascribe_resolve::Project>) -> Project {
        let link_index = OnceLock::new();
        let _ = link_index.set(index);
        Project { link_index, ..self }
    }

    /// The source index a broken link's closest targets are looked for in,
    /// and the page-level checks read, built the first time it's needed.
    pub(crate) fn link_index(&self) -> &Arc<ascribe_resolve::Project> {
        self.link_index.get_or_init(|| Arc::new(self.held_index()))
    }

    /// Lets go of the source index the checks built and kept, for a caller
    /// that goes on to index the files its own way, as `ascribe build` does:
    /// holding both at once is a third more peak memory (#894).
    pub fn release_link_index(&mut self) {
        self.link_index = OnceLock::new();
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

/// The id of the memo file at position `i` of [`Project::checked_memos`].
pub(crate) fn memo_id(i: usize) -> FileId {
    // A project has far fewer memo files than the ids from MEMO_FILE_IDS to
    // the images'.
    #[allow(clippy::cast_possible_truncation)]
    FileId::new(MEMO_FILE_IDS + i as u32)
}

/// The id of the image at position `i` of [`Project::images`].
pub(crate) fn image_id(i: usize) -> FileId {
    // A project has far fewer images than the ids from IMAGE_FILE_IDS to the
    // navigation file's.
    #[allow(clippy::cast_possible_truncation)]
    FileId::new(IMAGE_FILE_IDS + i as u32)
}

fn read_error(path: &Path, e: std::io::Error) -> LoadError {
    LoadError::Read {
        path: path.display().to_string(),
        message: e.to_string(),
    }
}
