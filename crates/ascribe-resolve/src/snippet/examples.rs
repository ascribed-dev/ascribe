//! Tested examples on the docs side: whether a snippet's file is an example,
//! and whether the observations beside its source's `ascribe.examples.toml`
//! say it passed as it is now.
//!
//! The files are read through the [`FileSystem`], from the source's folder,
//! whatever the source's own `include` says, and only read: nothing here
//! starts a program. For a source in another repository, the folder is its
//! copies, `sources/<name>/`, where `ascribe sources fetch` puts the config
//! and the observations beside the files snippets use, and they're data
//! just the same.

use std::sync::Arc;

use ascribe_core::RelPath;
use ascribe_model::examples::{
    DEFAULT_OBSERVATION, EXAMPLES_FILE, ExamplesConfig, OBSERVATIONS_FOLDER, Observation, Problem,
    Status, observation_name, observation_path, tested_line,
};
use ascribe_model::{ContentModel, Source};

use super::CodeFiles;
use crate::fs::{FileSystem, MODEL_FILE, Probe};

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

/// An `ascribe.examples.toml` in the folder of a source in this repository,
/// below its root, where no source reads it: only the config at a source's
/// root is read, so its examples are never shown as tested.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UnreadConfig {
    /// The config, from the project root.
    pub path: RelPath,
    /// The folder it's in, from the project root: the `path` a source would
    /// need for it to be read.
    pub folder: RelPath,
    /// The source whose folder it's in: the innermost, when one source's
    /// folder is in another's.
    pub source: String,
    /// That source's folder, from the project root.
    pub root: RelPath,
}

/// Every `ascribe.examples.toml` in a source's folder that no source reads
/// ([`UnreadConfig`]), in path order. A source's folder is looked in where
/// it may hold a file the source reads ([`Source::may_use`]): not in a
/// folder its `include` can't match or its `ignore` leaves out, not in
/// another source's folder, which that source looks in, and not in a folder
/// that holds an `ascribe.toml`, which is a project of its own. A source in
/// another repository has only the copies its snippets use, so it isn't
/// looked in.
pub fn unread_configs(model: &ContentModel, fs: &dyn FileSystem) -> Vec<UnreadConfig> {
    let roots = source_roots(model);
    let mut out: Vec<UnreadConfig> = Vec::new();
    for source in model.sources.iter().filter(|s| s.git.is_none()) {
        let Ok(root) = RelPath::parse(&source.path) else {
            continue;
        };
        let enter = |dir: &RelPath| looks_in(source, &root, dir, &roots, fs);
        for path in fs.files_named(&root, EXAMPLES_FILE, &enter) {
            let Some(folder) = path.parent() else {
                continue;
            };
            if folder == root || out.iter().any(|u| u.path == path) {
                continue;
            }
            out.push(UnreadConfig {
                path,
                folder,
                source: source.name.clone(),
                root: root.clone(),
            });
        }
    }
    out.sort_by(|a, b| a.path.cmp(&b.path));
    out
}

/// The config at `path`, from the project root, when it's one
/// [`unread_configs`] lists: asked of one file, as the editor asks of a
/// config it opens, without looking in any other folder.
pub fn unread_config(
    model: &ContentModel,
    fs: &dyn FileSystem,
    path: &RelPath,
) -> Option<UnreadConfig> {
    if path.file_name() != Some(EXAMPLES_FILE) {
        return None;
    }
    let folder = path.parent()?;
    let roots = source_roots(model);
    if roots.contains(&folder) {
        return None;
    }
    // The innermost source whose folder holds it; one in another source's
    // folder is that source's to look in.
    let (source, root) = model
        .sources
        .iter()
        .filter_map(|s| Some((s, RelPath::parse(&s.path).ok()?)))
        .filter(|(_, root)| folder.starts_with(root))
        .max_by_key(|(_, root)| root.segments().count())?;
    if source.git.is_some() {
        return None;
    }
    // Each folder from the source's root down to the config's is looked in.
    let mut dir = Some(folder.clone());
    while let Some(at) = dir {
        if at == root {
            break;
        }
        if !looks_in(source, &root, &at, &roots, fs) {
            return None;
        }
        dir = at.parent();
    }
    Some(UnreadConfig {
        path: path.clone(),
        folder,
        source: source.name.clone(),
        root,
    })
}

/// Every source's folder, from the project root.
fn source_roots(model: &ContentModel) -> Vec<RelPath> {
    model
        .sources
        .iter()
        .filter_map(|s| RelPath::parse(&s.path).ok())
        .collect()
}

/// Whether `dir`, a folder below `root`, the folder of `source`, is looked
/// in for a config no source reads.
fn looks_in(
    source: &Source,
    root: &RelPath,
    dir: &RelPath,
    roots: &[RelPath],
    fs: &dyn FileSystem,
) -> bool {
    dir.relative_to(root)
        .is_some_and(|rest| source.may_use(rest.as_str()))
        && !roots.contains(dir)
        && dir
            .join(MODEL_FILE)
            .map_or(true, |model| fs.probe(&model) != Probe::File)
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
    /// The folder `ascribe observe` is run in, from the project root. For a
    /// source in another repository, the folder of its copies, where it
    /// isn't run: the test runs in that repository.
    pub folder: RelPath,
    /// Whether the source is in another repository, so the observation
    /// arrives with the next `ascribe sources update`.
    pub remote: bool,
    /// The observations that are current for it, in name order.
    pub current: Vec<CurrentObservation>,
    /// What the page says it was tested with, merged from the current
    /// observations ([`tested_line`]); `None` unless it's current.
    pub tested: Option<String>,
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
    let statuses: Vec<(&str, &Observation, Status)> = examples
        .readable()
        .map(|(name, o)| (name, o, o.status(path, text.as_bytes())))
        .collect();
    let status = match Status::across(statuses.iter().map(|(_, _, s)| *s)) {
        Status::Current => ExampleStatus::Current,
        Status::Stale => ExampleStatus::Stale,
        Status::Unobserved => ExampleStatus::Unobserved,
    };
    let current: Vec<(&str, &Observation)> = statuses
        .into_iter()
        .filter(|(_, _, s)| *s == Status::Current)
        .map(|(name, o, _)| (name, o))
        .collect();
    let tested = (!current.is_empty()).then(|| {
        let observations: Vec<&Observation> = current.iter().map(|(_, o)| *o).collect();
        tested_line(config, &observations)
    });
    Some(Example {
        status,
        folder,
        remote: source.git.is_some(),
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
