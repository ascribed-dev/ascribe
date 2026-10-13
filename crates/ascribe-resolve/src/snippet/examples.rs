//! Tested examples on the docs side: whether a snippet's file is an example,
//! and whether the observations beside its source's `ascribe.examples.toml`
//! say it passed as it is now.
//!
//! The files are read through the [`FileSystem`], from the source's folder,
//! whatever the source's own `include` says, and only read: nothing here
//! starts a program. A source in another repository has no tested line yet,
//! and reports nothing.

use std::sync::Arc;

use ascribe_core::RelPath;
use ascribe_model::examples::{
    DEFAULT_OBSERVATION, EXAMPLES_FILE, ExamplesConfig, OBSERVATIONS_FOLDER, Observation, Problem,
    observation_name, observation_path, tested_line,
};
use ascribe_model::{Source, example_hash};

use super::CodeFiles;
use crate::fs::{FileSystem, Probe};

/// A source folder's config and observations, as read.
#[derive(Debug)]
pub struct FolderExamples {
    /// The folder, from the project root.
    pub folder: RelPath,
    /// `ascribe.examples.toml`, when there is one.
    pub config: Option<Result<ExamplesConfig, Vec<Problem>>>,
    /// The observations in `examples.observations/`, in name order.
    pub observations: Vec<ReadObservation>,
}

/// An observation, as read.
#[derive(Debug)]
pub struct ReadObservation {
    /// Its name: `default`, or the `--name` it was written with.
    pub name: String,
    /// Its path from the config's folder: `examples.observations/default.toml`.
    pub path: String,
    /// What it says, or why it can't be read. One in a format this version
    /// of Ascribe doesn't know can't be read, and counts for none of its
    /// files.
    pub read: Result<Observation, Vec<Problem>>,
}

impl FolderExamples {
    /// Reads the folder's config and its observations through `fs`. An
    /// observation is a file in `examples.observations/` named as one
    /// ([`observation_name`]); any other file there is left alone.
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
        let mut names: Vec<String> = folder
            .join(OBSERVATIONS_FOLDER)
            .map(|dir| fs.files_in(&dir))
            .unwrap_or_default()
            .iter()
            .filter_map(|path| {
                let file = path.file_name()?;
                observation_name(&format!("{OBSERVATIONS_FOLDER}/{file}")).map(str::to_owned)
            })
            .filter(|name| {
                // Only the folder's own files, not those of a folder in it.
                folder
                    .join(&observation_path(name))
                    .is_ok_and(|p| fs.probe(&p) == Probe::File)
            })
            .collect();
        // A file system that can't list a folder still has the default.
        if !names.iter().any(|n| n == DEFAULT_OBSERVATION) {
            names.push(DEFAULT_OBSERVATION.to_owned());
        }
        names.sort();
        names.dedup();
        let config = read(EXAMPLES_FILE).map(|text| ExamplesConfig::parse(&text));
        let observations = names
            .into_iter()
            .filter_map(|name| {
                let path = observation_path(&name);
                let text = read(&path)?;
                Some(ReadObservation {
                    name,
                    path,
                    read: Observation::parse(&text),
                })
            })
            .collect();
        FolderExamples {
            folder: folder.clone(),
            config,
            observations,
        }
    }

    /// The observations that can be read, with their names.
    pub fn readable(&self) -> impl Iterator<Item = (&str, &Observation)> {
        self.observations
            .iter()
            .filter_map(|o| Some((o.name.as_str(), o.read.as_ref().ok()?)))
    }
}

/// What a snippet's file is, when it's an example (decision 11 of the tested
/// examples plan).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ExampleStatus {
    /// At least one observation lists the file with the hash it has now.
    Current,
    /// Observations list the file, none with the hash it has now.
    Stale,
    /// No observation lists the file.
    Unobserved,
}

impl ExampleStatus {
    /// The word the JSON output and the report use: `current`, `stale`, or
    /// `unobserved`.
    pub fn as_str(self) -> &'static str {
        match self {
            ExampleStatus::Current => "current",
            ExampleStatus::Stale => "stale",
            ExampleStatus::Unobserved => "unobserved",
        }
    }
}

/// An observation that's current for a file.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CurrentObservation {
    /// Its name.
    pub name: String,
    /// Its environment as it has it: `os`, `arch`, then each entry's kept
    /// line, in its order.
    pub environment: Vec<(String, String)>,
}

/// A snippet's file as an example.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Example {
    /// Its status.
    pub status: ExampleStatus,
    /// The folder `ascribe observe` is run in, from the project root.
    pub folder: RelPath,
    /// The observations that are current for it, in name order.
    pub current: Vec<CurrentObservation>,
    /// What the page says it was tested with, merged from the current
    /// observations ([`tested_line`]); `None` unless it's current.
    pub tested: Option<String>,
}

/// The file at `path` (relative to `source`'s folder), with text `text`, as
/// an example: `None` when it isn't one, because its source is in another
/// repository, has no config, has one that can't be read, or doesn't
/// include the file.
pub fn example(
    source: &Source,
    path: &str,
    text: &str,
    fs: &dyn FileSystem,
    code: &CodeFiles,
) -> Option<Example> {
    if source.git.is_some() {
        return None;
    }
    let folder = RelPath::parse(&source.path).ok()?;
    let examples = code.examples(fs, &folder);
    let config = examples.config.as_ref()?.as_ref().ok()?;
    if !config.is_example(path) {
        return None;
    }
    let hash = example_hash(text.as_bytes());
    let mut listed = false;
    let mut current: Vec<(&str, &Observation)> = Vec::new();
    for (name, observation) in examples.readable() {
        match observation.files.get(path) {
            Some(h) if *h == hash => current.push((name, observation)),
            Some(_) => listed = true,
            None => {}
        }
    }
    let status = if !current.is_empty() {
        ExampleStatus::Current
    } else if listed {
        ExampleStatus::Stale
    } else {
        ExampleStatus::Unobserved
    };
    let tested = (!current.is_empty()).then(|| {
        let observations: Vec<&Observation> = current.iter().map(|(_, o)| *o).collect();
        tested_line(config, &observations)
    });
    Some(Example {
        status,
        folder,
        current: current
            .into_iter()
            .map(|(name, o)| CurrentObservation {
                name: name.to_owned(),
                environment: o.environment.clone(),
            })
            .collect(),
        tested,
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
