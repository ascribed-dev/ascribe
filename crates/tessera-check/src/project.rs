//! The project the checks run on: the content model and the source files.

use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use tessera_core::{FileId, LineIndex, RelPath};
use tessera_model::{ContentModel, LOCK_FILE};
use tessera_resolve::{CodeFile, CodeFiles, DiskFs, FileSystem, Layout, SourceSet};

use crate::Diagnostic;

pub use tessera_model::MODEL_FILE;

/// The id of `ascribe.lock` (SPEC §7.4), for locations in it: past any
/// source file's, and before the code files'.
pub const LOCK_FILE_ID: FileId = FileId::new(0x7FFF_FFFF);

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
    /// The path relative to the content root, for a source file; `None` for
    /// the content model.
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
/// `tessera_resolve::Project`). A project built from parts may use any
/// distinct ids from 1 (it finds a file by id, not by position): the language
/// server gives the ids of a `tessera_resolve::Snapshot`, which are stable
/// per path across updates and have gaps where files were deleted
/// (`tessera_resolve::incremental`). Source texts are held in memory: the
/// language server builds a `Project` from its open buffers, and the command
/// line from the files on disk. Files that aren't sources (images, other
/// downloads) are looked for on disk, through `tessera_resolve`'s
/// [`FileSystem`], when a reference needs them; a source file counts as
/// existing even when it's only in memory.
///
/// Which files are sources, and every rule about what a reference names, are
/// `tessera_resolve`'s, so this crate and the source index can't disagree.
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
    /// [`tessera_resolve::FileSystem::sources`] finds them.
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
        match tessera_model::load_str_in(&text, FileId::new(0), &root) {
            Ok(model) => Ok(ModelFile { root, text, model }),
            Err(issues) => Err(LoadError::Model {
                text,
                diagnostics: issues.iter().map(Diagnostic::from_issue).collect(),
            }),
        }
    }

    /// Reads every source file under `root/content_root`. The files that
    /// could be read have ids from 1, in path order, as in
    /// `tessera_resolve::Project`; those that couldn't come after them, with
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
    pub fn index(&self) -> tessera_resolve::Project {
        tessera_resolve::Project::load(
            Arc::new(self.model.clone()),
            self.layout.clone(),
            self.file_system(),
        )
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
    /// file's, from [`tessera_resolve::snippet::CODE_FILE_IDS`] up.
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

    /// What a file id names.
    pub fn file(&self, id: FileId) -> Option<FileEntry<'_>> {
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

fn read_error(path: &Path, e: std::io::Error) -> LoadError {
    LoadError::Read {
        path: path.display().to_string(),
        message: e.to_string(),
    }
}
