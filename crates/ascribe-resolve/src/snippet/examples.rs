//! Tested examples on the docs side: whether a snippet's file is an example,
//! and whether the observation beside its source's `ascribe.examples.toml`
//! says it passed as it is now.
//!
//! Both files are read through the [`FileSystem`], from the source's folder,
//! whatever the source's own `include` says, and only read: nothing here
//! starts a program. For a source in another repository, the folder is its
//! copies, `sources/<name>/`, where `ascribe sources fetch` puts the config
//! and the observations beside the files snippets use, and they're data
//! just the same.

use std::sync::Arc;

use ascribe_core::RelPath;
use ascribe_model::Source;
use ascribe_model::examples::{
    DEFAULT_OBSERVATION, EXAMPLES_FILE, ExamplesConfig, Observation, Problem, Status,
    observation_path,
};

use super::CodeFiles;
use crate::fs::{FileSystem, Probe};

/// A source folder's config and observation, as read.
#[derive(Debug)]
pub struct FolderExamples {
    /// The folder, from the project root.
    pub folder: RelPath,
    /// `ascribe.examples.toml`, when there is one.
    pub config: Option<Result<ExamplesConfig, Vec<Problem>>>,
    /// The observation `ascribe observe` writes, when there is one.
    pub observation: Option<Result<Observation, Vec<Problem>>>,
}

impl FolderExamples {
    /// Reads the folder's config and its observation through `fs`.
    pub fn read(fs: &dyn FileSystem, folder: &RelPath) -> FolderExamples {
        let read = |name: &str| -> Option<String> {
            let path = folder.join(name).ok()?;
            if fs.probe(&path) != Probe::File {
                return None;
            }
            // A file that can't be read, or isn't UTF-8, is read as it can
            // be, and its problems are reported as the file's.
            fs.read_file(&path)
                .ok()
                .map(|bytes| String::from_utf8_lossy(&bytes).into_owned())
        };
        FolderExamples {
            folder: folder.clone(),
            config: read(EXAMPLES_FILE).map(|text| ExamplesConfig::parse(&text)),
            observation: read(&observation_path(DEFAULT_OBSERVATION))
                .map(|text| Observation::parse(&text)),
        }
    }
}

/// What a snippet's file is, when it's an example.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ExampleStatus {
    /// The observation lists the file with the hash it has now: what the page
    /// says it was tested with ([`Observation::tested_line`]).
    Current(String),
    /// The observation lists the file with another hash.
    Stale,
    /// No observation lists the file.
    Unobserved,
}

/// A snippet's file as an example.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Example {
    /// Its status.
    pub status: ExampleStatus,
    /// The folder `ascribe observe` is run in, from the project root. For a
    /// source in another repository, the folder of its copies, where it
    /// isn't run: the test runs in that repository.
    pub folder: RelPath,
    /// Whether the source is in another repository, so the observation
    /// arrives with the next `ascribe sources update`.
    pub remote: bool,
}

/// The file at `path` (relative to `source`'s folder), with text `text`, as
/// an example: `None` when it isn't one, because its source has no config,
/// has one that can't be read, or doesn't include the file.
pub fn example(
    source: &Source,
    path: &str,
    text: &str,
    fs: &dyn FileSystem,
    code: &CodeFiles,
) -> Option<Example> {
    let folder = RelPath::parse(&source.path).ok()?;
    let examples = code.examples(fs, &folder);
    let config = examples.config.as_ref()?.as_ref().ok()?;
    if !config.is_example(path) {
        return None;
    }
    let status = match examples.observation.as_ref().and_then(|o| o.as_ref().ok()) {
        None => ExampleStatus::Unobserved,
        Some(observation) => match observation.status(path, text.as_bytes()) {
            Status::Current => ExampleStatus::Current(observation.tested_line(config)),
            Status::Stale => ExampleStatus::Stale,
            Status::Unobserved => ExampleStatus::Unobserved,
        },
    };
    Some(Example {
        status,
        folder,
        remote: source.git.is_some(),
    })
}

impl CodeFiles {
    /// The config and observation of the source folder `folder`, read
    /// through `fs` the first time they're asked for.
    pub fn examples(&self, fs: &dyn FileSystem, folder: &RelPath) -> Arc<FolderExamples> {
        let mut examples = self
            .examples
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        examples
            .entry(folder.clone())
            .or_insert_with(|| Arc::new(FolderExamples::read(fs, folder)))
            .clone()
    }
}
