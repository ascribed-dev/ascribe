//! Reading a project: its content model, its source files, and its side
//! files.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use ascribe_core::{FileId, Issue, RelPath};
use ascribe_model::{ContentModel, MODEL_FILE, SourceBound};
use ascribe_resolve::side_file::{Entry, Held};
use ascribe_resolve::{DiskFs, FileSystem, Layout, Sources};

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
        issues: Vec<Issue>,
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

/// A content model as [`Loader::load_model`] loads it.
#[derive(Clone, Debug)]
pub struct ModelFile {
    /// The project root: the directory containing `ascribe.toml`.
    pub root: PathBuf,
    /// The content model's text.
    pub text: String,
    /// The content model.
    pub model: ContentModel,
}

/// Loads the content model in its text, the `ascribe.toml` of the project
/// in the folder given, with a local source bounded as given: the content
/// model, or its problems, with its warnings, at their places in the text.
pub type ModelLoader = fn(&str, &Path, SourceBound<'_>) -> Result<ContentModel, Vec<Issue>>;

/// How a project is loaded: what loading takes from the crate that checks
/// it, so this crate needs nothing of the checks. `ascribe-check` gives
/// the one every surface loads with.
#[derive(Clone, Copy, Debug)]
pub struct Loader {
    /// Loads the content model from its text, checking what the checks hold
    /// of it as it loads.
    pub model: ModelLoader,
    /// The side files a project has, each read by its [`Entry`].
    pub side_files: &'static [Entry],
}

/// A project as [`Loader::load`] read it: the parts a project the checks
/// see is built from.
pub struct Loaded {
    /// The project root: the directory containing `ascribe.toml`.
    pub root: PathBuf,
    /// The content root, relative to the project root.
    pub content_root: RelPath,
    /// The content model.
    pub model: ContentModel,
    /// The content model's text.
    pub model_text: String,
    /// The source files, as [`read_sources`] read them.
    pub sources: Vec<SourceFile>,
    /// The listing the sources were read from, as [`FileSystem::sources`]
    /// gave it.
    pub listing: Sources,
    /// The side files the project has, each read with its text, in the
    /// order of the [`Loader`]'s list.
    pub side: Vec<Held>,
    /// The file system everything was read through.
    pub fs: Arc<dyn FileSystem + Send + Sync>,
}

impl std::fmt::Debug for Loaded {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Loaded")
            .field("root", &self.root)
            .field("content_root", &self.content_root)
            .field("sources", &self.sources)
            .field("side", &self.side)
            .finish_non_exhaustive()
    }
}

impl Loader {
    /// Loads the project whose content model is at `config`: the model, then
    /// every source file under its content root, as
    /// [`ascribe_resolve::FileSystem::sources`] finds them, then the side
    /// files.
    ///
    /// A source file or directory that can't be read, or a source file that
    /// isn't valid UTF-8, is still a source file, with its [`ReadFailure`].
    ///
    /// # Errors
    ///
    /// As [`Loader::load_model`].
    pub fn load(&self, config: &Path) -> Result<Loaded, LoadError> {
        self.load_bounded(config, SourceBound::Anywhere)
    }

    /// [`Loader::load`], with a local source bounded by `bound` when no git
    /// repository is around the project ([`SourceBound`]).
    ///
    /// # Errors
    ///
    /// As [`Loader::load`].
    pub fn load_bounded(&self, config: &Path, bound: SourceBound<'_>) -> Result<Loaded, LoadError> {
        let loaded = self.load_model_bounded(config, bound)?;
        let content_root =
            RelPath::parse(&loaded.model.project.content_root).map_err(|e| LoadError::Read {
                path: config.display().to_string(),
                message: e.to_string(),
            })?;
        let layout = Layout {
            content_root,
            output_dir: RelPath::parse(&loaded.model.project.output_dir).unwrap_or_default(),
        };
        // A one-shot command reads a disk that doesn't change while it runs,
        // so listings are kept.
        let fs = Arc::new(DiskFs::with_listing_cache(&loaded.root, &layout));
        Ok(self.load_from(loaded, layout.content_root, fs))
    }

    /// The project of a content model [`Loader::load_model`] loaded, with
    /// every source file under `content_root` read through `fs`, as
    /// [`FileSystem::sources`] lists them, and the side files read through
    /// it. Each source is read once and listed once.
    ///
    /// A source file or directory that can't be read, or a source file that
    /// isn't valid UTF-8, is still a source file, with its [`ReadFailure`].
    pub fn load_from(
        &self,
        model: ModelFile,
        content_root: RelPath,
        fs: Arc<dyn FileSystem + Send + Sync>,
    ) -> Loaded {
        let ModelFile { root, text, model } = model;
        let (sources, listing) = read_sources(&*fs);
        let side = read_side_files(self.side_files, &*fs);
        Loaded {
            root,
            content_root,
            model,
            model_text: text,
            sources,
            listing,
            side,
            fs,
        }
    }

    /// Loads only the content model at `config`, the first step of
    /// [`Loader::load`], for a command that doesn't need the pages read:
    /// `ascribe fmt`, which reads the files it formats itself.
    ///
    /// # Errors
    ///
    /// The file can't be read or isn't valid UTF-8 ([`LoadError::Read`]), or
    /// the content model has errors ([`LoadError::Model`]).
    pub fn load_model(&self, config: &Path) -> Result<ModelFile, LoadError> {
        self.load_model_bounded(config, SourceBound::Anywhere)
    }

    /// [`Loader::load_model`], with a local source bounded by `bound` when
    /// no git repository is around the project ([`SourceBound`]).
    ///
    /// # Errors
    ///
    /// As [`Loader::load_model`].
    pub fn load_model_bounded(
        &self,
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
        match (self.model)(&text, &root, bound) {
            Ok(model) => Ok(ModelFile { root, text, model }),
            Err(issues) => Err(LoadError::Model { text, issues }),
        }
    }
}

/// Reads every side file of `list` that `fs` has, each with its text, in
/// the order of the list. They're read like any other file beside the
/// content model, so a project in memory can have them.
pub fn read_side_files(list: &[Entry], fs: &dyn FileSystem) -> Vec<Held> {
    list.iter().filter_map(|entry| entry.read(fs)).collect()
}

/// Reads every source file `fs` lists, and the listing. The files that could
/// be read have ids from 1, in path order, as in `ascribe_resolve::Project`;
/// those that couldn't come after them, with their [`ReadFailure`].
pub fn read_sources(fs: &dyn FileSystem) -> (Vec<SourceFile>, Sources) {
    let listing = fs.sources();
    let mut paths = listing.paths.clone();
    paths.sort();
    paths.dedup();
    let mut sources = Vec::with_capacity(paths.len());
    let mut failed = Vec::new();
    for path in paths {
        match fs.read(&path) {
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
    failed.extend(listing.unreadable.iter().map(|u| {
        (
            u.path.clone(),
            ReadFailure {
                reason: u.reason.clone(),
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
    (out, listing)
}

fn read_error(path: &Path, e: std::io::Error) -> LoadError {
    LoadError::Read {
        path: path.display().to_string(),
        message: e.to_string(),
    }
}
