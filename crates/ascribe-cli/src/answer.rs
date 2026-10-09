//! What the commands that answer questions share (`explain`, `model`,
//! `outline`, `link`, `render`, and `refs`): their output format, finding the
//! project from a path given on the command line, reading a page argument,
//! and writing the answer.

use std::io::{self, Write};
use std::path::{Path, PathBuf};

use ascribe_check::{LoadError, LocateError, MODEL_FILE, Project};
use ascribe_core::path::{normalize, relative_path};
use ascribe_core::{Coded, RelPath};
use ascribe_query::QueryError;
use clap::ValueEnum;
use serde::Serialize;

use crate::cli::Global;
use crate::context::Failure;
use crate::exit;

/// The docs site's address, for the links answers give.
pub const DOCS_SITE: &str = docs_site!();

/// How to show an answer.
#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
pub enum Format {
    /// Short text, for people and agents to read.
    Text,
    /// One JSON document, for tools.
    Json,
}

/// Finds the content model: `--config`, or else the nearest `ascribe.toml`
/// at or above `path` (a file or folder named on the command line, which
/// need not exist yet), or else at or above the current directory.
pub fn locate(global: &Global, path: Option<&Path>) -> Result<PathBuf, LocateError> {
    if global.config.is_some() {
        return crate::context::locate(global);
    }
    let Some(path) = path else {
        return crate::context::locate(global);
    };
    let cwd = std::env::current_dir().map_err(LocateError::CurrentDir)?;
    let start = normalize(&cwd.join(path));
    Project::find_config(&start).ok_or(LocateError::NotFound { dir: start })
}

/// Finds and loads the project, as [`locate`] finds it.
pub fn load(global: &Global, path: Option<&Path>) -> Result<Project, Failure> {
    let config = locate(global, path).map_err(Failure::Config)?;
    Project::load(&config).map_err(Failure::Load)
}

/// Reports a project that couldn't be loaded, and returns the exit code.
pub fn report_failure(err: &mut dyn Write, failure: &Failure) -> u8 {
    match failure {
        Failure::Load(LoadError::Model { .. }) => {
            let _ = writeln!(
                err,
                "error: {MODEL_FILE} has errors; `ascribe check` lists them"
            );
            exit::of(failure)
        }
        _ => exit::fail(err, failure),
    }
}

/// The content path of a source file named on the command line: a path from
/// the current directory into the content root, or else a content path. It
/// must be one of the project's source files.
pub fn source_path(project: &Project, given: &Path) -> Result<RelPath, QueryError> {
    let not_found = || QueryError::NotASource {
        path: given.display().to_string(),
    };
    let content = normalize(&project.root().join(project.content_root().as_str()));
    let from_cwd = std::env::current_dir()
        .ok()
        .and_then(|cwd| relative_path(&content, &normalize(&cwd.join(given))))
        .filter(RelPath::is_inside);
    let as_content = given
        .to_str()
        .map(|s| s.replace('\\', "/"))
        .and_then(|s| RelPath::parse(s.trim_start_matches("./")).ok());
    [from_cwd, as_content]
        .into_iter()
        .flatten()
        .find(|path| project.source_at(path).is_some())
        .ok_or_else(not_found)
}

/// Writes an answer as one JSON document and a newline.
pub fn write_json(out: &mut dyn Write, answer: &impl Serialize) -> io::Result<()> {
    serde_json::to_writer_pretty(&mut *out, answer).map_err(io::Error::other)?;
    writeln!(out)
}

/// The exit code for an answer that couldn't be written: none for a closed
/// pipe (`| head`), which isn't a problem with the project.
pub fn written(result: io::Result<()>, err: &mut dyn Write) -> Option<u8> {
    match result {
        Err(e) if e.kind() != io::ErrorKind::BrokenPipe => {
            let _ = writeln!(err, "error: can't write the answer: {e}");
            Some(exit::FAILURE)
        }
        _ => None,
    }
}

/// Reports an error that stopped a command, and returns its exit code.
pub fn fail(err: &mut dyn Write, error: &dyn Coded) -> u8 {
    exit::fail(err, error)
}

/// A fence for `text` in Markdown: backticks, one more than the longest run
/// in it, and at least three.
pub fn fence(text: &str) -> String {
    let longest = text.split(|c| c != '`').map(str::len).max().unwrap_or(0);
    "`".repeat((longest + 1).max(3))
}

/// The build `--build` names, if it names one.
pub fn build<'m>(
    model: &'m ascribe_model::ContentModel,
    name: Option<&str>,
) -> Result<Option<&'m ascribe_model::Build>, QueryError> {
    let Some(name) = name else {
        return Ok(None);
    };
    model
        .builds
        .iter()
        .find(|b| b.name == name)
        .map(Some)
        .ok_or_else(|| {
            QueryError::UnknownBuild(ascribe_check::UnknownBuild {
                name: name.to_owned(),
                known: build_names(model),
            })
        })
}

/// The build `--build` names, or else the model's only build.
pub fn one_build<'m>(
    model: &'m ascribe_model::ContentModel,
    name: Option<&str>,
) -> Result<&'m ascribe_model::Build, QueryError> {
    if let Some(build) = build(model, name)? {
        return Ok(build);
    }
    match model.builds.as_slice() {
        [only] => Ok(only),
        _ => Err(QueryError::BuildRequired {
            builds: build_names(model),
        }),
    }
}

fn build_names(model: &ascribe_model::ContentModel) -> Vec<String> {
    model.builds.iter().map(|b| b.name.clone()).collect()
}
