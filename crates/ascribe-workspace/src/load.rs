//! Reading a project: its content model, its source files, and its side
//! files.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use ascribe_core::{FileId, Issue, RelPath};
use ascribe_model::{ContentModel, MODEL_FILE, SourceBound};
use ascribe_resolve::side_file::{Entry, Held};
use ascribe_resolve::{FileSystem, Sources};

/// One source file: a Markdown file under the content root.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SourceFile {
    /// The file's id, used in every location that points into it.
    pub id: FileId,
    /// The path relative to the content root, `/`-separated.
    pub path: RelPath,
    /// The text; empty when the file couldn't be read. A loaded project's
    /// is its source index's, shared.
    pub text: Arc<str>,
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

/// A content model as [`Loader::load_model`] loads it: what a
/// [`Workspace`](crate::Workspace) is loaded around.
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
/// the one every surface loads with, and a [`Workspace`](crate::Workspace)
/// is loaded with it.
#[derive(Clone, Copy, Debug)]
pub struct Loader {
    /// Loads the content model from its text, checking what the checks hold
    /// of it as it loads.
    pub model: ModelLoader,
    /// The side files a project has, each read by its [`Entry`].
    pub side_files: &'static [Entry],
}

impl Loader {
    /// Loads only the content model at `config`, the first step of
    /// [`Workspace::load`](crate::Workspace::load), for a command that
    /// doesn't need the pages read:
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
        let root = root_of(config);
        match (self.model)(&text, &root, bound) {
            Ok(model) => Ok(ModelFile { root, text, model }),
            Err(issues) => Err(LoadError::Model { text, issues }),
        }
    }
}

/// Reads every side file of `list` that `fs` has, each with its text, in
/// the order of the list. They're read like any other file beside the
/// content model, so a project in memory can have them.
pub(crate) fn read_side_files(list: &[Entry], fs: &dyn FileSystem) -> Vec<Held> {
    list.iter().filter_map(|entry| entry.read(fs)).collect()
}

/// A source file as it was read: its text, or why it couldn't be.
pub(crate) type Read = (RelPath, Result<String, ReadFailure>);

/// Reads every source file `fs` lists, once each, and the listing: the
/// files that could be read in path order, with their texts, then those
/// that couldn't, each with its [`ReadFailure`].
pub(crate) fn read_sources(fs: &dyn FileSystem) -> (Vec<Read>, Sources) {
    let listing = fs.sources();
    let mut paths = listing.paths.clone();
    paths.sort();
    paths.dedup();
    let mut sources = Vec::with_capacity(paths.len());
    let mut failed = Vec::new();
    for path in paths {
        match fs.read(&path) {
            Ok(text) => sources.push((path, Ok(text))),
            Err(e) => failed.push((
                path,
                Err(ReadFailure {
                    not_utf8: e.kind() == io::ErrorKind::InvalidData,
                    reason: e.to_string(),
                }),
            )),
        }
    }
    failed.extend(listing.unreadable.iter().map(|u| {
        (
            u.path.clone(),
            Err(ReadFailure {
                reason: u.reason.clone(),
                not_utf8: false,
            }),
        )
    }));
    sources.extend(failed);
    (sources, listing)
}

/// The project root of the content model at `config`: the directory it's
/// in, or `.` for a bare file name.
pub fn root_of(config: &Path) -> PathBuf {
    match config.parent() {
        Some(p) if !p.as_os_str().is_empty() => p.to_owned(),
        _ => PathBuf::from("."),
    }
}

fn read_error(path: &Path, e: std::io::Error) -> LoadError {
    LoadError::Read {
        path: path.display().to_string(),
        message: e.to_string(),
    }
}
