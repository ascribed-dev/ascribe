//! What the commands that answer questions share (`explain`, `model`,
//! `outline`, `link`, `render`, and `refs`): their output format, finding the
//! project from a path given on the command line, reading a page argument,
//! and writing the answer. The MCP server's tools answer through the same
//! functions, with a [`Projects`] that keeps projects loaded between calls.

use std::cell::{OnceCell, RefCell};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::rc::Rc;

use ascribe_check::{Diagnostic, LoadError, LocateError, MODEL_FILE, Project};
use ascribe_core::path::{normalize, relative_path};
use ascribe_core::{Coded, RelPath};
use ascribe_query::QueryError;
use clap::ValueEnum;
use serde::Serialize;

use crate::cli::Global;
use crate::context::Failure;
use crate::exit;

/// How to show an answer.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, ValueEnum)]
pub enum Format {
    /// Short text, for people and agents to read.
    #[default]
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

/// A loaded project, and its source index once something asks for it.
pub struct Loaded {
    /// The project.
    pub project: Project,
    index: OnceCell<ascribe_resolve::Project>,
    /// Each check of the whole project so far: the builds asked for, the
    /// diagnostics, and the builds checked. A project kept between calls
    /// checks once for each set of builds.
    checks: RefCell<Vec<Checked>>,
}

/// A check of the whole project, kept.
struct Checked {
    asked: Vec<String>,
    diagnostics: Vec<Diagnostic>,
    builds: Vec<String>,
}

impl Loaded {
    /// A project, not yet indexed.
    pub fn new(project: Project) -> Loaded {
        Loaded {
            project,
            index: OnceCell::new(),
            checks: RefCell::new(Vec::new()),
        }
    }

    /// The diagnostics of the builds `asked` names, and the names of the
    /// builds checked: from `check` the first time, and kept for the next.
    ///
    /// # Errors
    ///
    /// What `check` returns, which isn't kept.
    pub fn diagnosed<E>(
        &self,
        asked: &[String],
        check: impl FnOnce(&Project) -> Result<(Vec<Diagnostic>, Vec<String>), E>,
    ) -> Result<(Vec<Diagnostic>, Vec<String>), E> {
        if let Some(kept) = self.checks.borrow().iter().find(|c| c.asked == asked) {
            return Ok((kept.diagnostics.clone(), kept.builds.clone()));
        }
        let (diagnostics, builds) = check(&self.project)?;
        self.checks.borrow_mut().push(Checked {
            asked: asked.to_vec(),
            diagnostics: diagnostics.clone(),
            builds: builds.clone(),
        });
        Ok((diagnostics, builds))
    }

    /// The project's source index ([`Project::index`]), made on first use.
    pub fn index(&self) -> &ascribe_resolve::Project {
        self.index.get_or_init(|| self.project.index())
    }
}

/// Where a command gets its project: read from disk for each command, or
/// kept between calls by the MCP server (`crate::mcp`).
pub trait Projects {
    /// The project whose content model is at `config`.
    ///
    /// # Errors
    ///
    /// The project can't be loaded.
    fn load(&self, config: &Path) -> Result<Rc<Loaded>, LoadError>;
}

/// Projects read from disk when they're asked for: the command line's.
pub struct FromDisk;

impl Projects for FromDisk {
    fn load(&self, config: &Path) -> Result<Rc<Loaded>, LoadError> {
        Project::load(config).map(|project| Rc::new(Loaded::new(project)))
    }
}

/// Finds and loads the project, as [`locate`] finds it.
pub fn load(
    projects: &dyn Projects,
    global: &Global,
    path: Option<&Path>,
) -> Result<Rc<Loaded>, Failure> {
    let config = locate(global, path).map_err(Failure::Config)?;
    projects.load(&config).map_err(Failure::Load)
}

/// Why a command that answers a question couldn't answer it.
#[derive(Debug)]
pub enum Stop {
    /// The project couldn't be found or loaded.
    Load(Failure),
    /// The question can't be answered as asked.
    Query(QueryError),
}

impl From<Failure> for Stop {
    fn from(failure: Failure) -> Stop {
        Stop::Load(failure)
    }
}

impl From<QueryError> for Stop {
    fn from(error: QueryError) -> Stop {
        Stop::Query(error)
    }
}

impl Stop {
    /// What went wrong, as the command reports it after `error: `.
    pub fn message(&self) -> String {
        match self {
            Stop::Load(failure) => failure_message(failure),
            Stop::Query(error) => error.to_string(),
        }
    }

    /// Reports it on `err`, and returns the exit code.
    pub fn report(&self, err: &mut dyn Write) -> u8 {
        let _ = writeln!(err, "error: {}", self.message());
        exit::FAILURE
    }
}

/// What a project that couldn't be loaded is reported as.
fn failure_message(failure: &Failure) -> String {
    match failure {
        Failure::Load(LoadError::Model { .. }) => {
            format!("{MODEL_FILE} has errors; `ascribe check` lists them")
        }
        _ => failure.to_string(),
    }
}

/// Reports a project that couldn't be loaded, and returns the exit code.
pub fn report_failure(err: &mut dyn Write, failure: &Failure) -> u8 {
    let _ = writeln!(err, "error: {}", failure_message(failure));
    exit::of(failure)
}

/// The content path of a source file named on the command line: a path from
/// the current directory into the content root, or else a content path. It
/// must be one of the project's source files.
pub fn source_path(project: &Project, given: &Path) -> Result<RelPath, QueryError> {
    content_paths(project, given)
        .into_iter()
        .find(|path| project.source_at(path).is_some())
        .ok_or_else(|| QueryError::NotASource {
            path: given.display().to_string(),
        })
}

/// What a path named on the command line can be as a content path: a path
/// from the current directory into the content root, then the path itself.
/// The file needn't exist: a page `ascribe diff` reports as removed doesn't.
pub fn content_paths(project: &Project, given: &Path) -> Vec<RelPath> {
    let content = normalize(&project.root().join(project.content_root().as_str()));
    let from_cwd = std::env::current_dir()
        .ok()
        .and_then(|cwd| relative_path(&content, &normalize(&cwd.join(given))))
        .filter(RelPath::is_inside);
    let as_content = given
        .to_str()
        .map(|s| s.replace('\\', "/"))
        .and_then(|s| RelPath::parse(s.trim_start_matches("./")).ok());
    [from_cwd, as_content].into_iter().flatten().collect()
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
