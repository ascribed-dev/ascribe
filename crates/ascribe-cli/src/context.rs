//! What every subcommand that works on a project does first: find the
//! content model and load the project.

use std::io::IsTerminal;
use std::path::PathBuf;

use ascribe_check::{LoadError, LocateError, Project};
use ascribe_core::Coded;

use crate::cli::{Color, Global};

/// Why a project couldn't be loaded, ready to report.
#[derive(Debug, thiserror::Error)]
pub enum Failure {
    /// No `ascribe.toml` was found, or `--config` names none.
    #[error(transparent)]
    Config(LocateError),
    /// Loading failed.
    #[error(transparent)]
    Load(LoadError),
}

impl Coded for Failure {
    fn code(&self) -> &'static str {
        match self {
            Failure::Config(e) => e.code(),
            Failure::Load(e) => e.code(),
        }
    }
}

/// Finds the content model (`--config`, or the nearest `ascribe.toml` in the
/// current directory or a parent) and loads the project. Every command that
/// works on a project loads it here; `fmt` takes only the first half, with
/// [`Project::load_model`].
pub fn load_project(global: &Global) -> Result<Project, Failure> {
    let config = locate(global).map_err(Failure::Config)?;
    Project::load(&config).map_err(Failure::Load)
}

/// The content model `--config` names, or the nearest one.
pub fn locate(global: &Global) -> Result<PathBuf, LocateError> {
    Project::locate(global.config.as_deref())
}

/// Whether to color output written to `stream_is_terminal`.
pub fn use_color(global: &Global, stream_is_terminal: bool) -> bool {
    match global.color {
        Color::Always => true,
        Color::Never => false,
        Color::Auto => stream_is_terminal && std::env::var_os("NO_COLOR").is_none(),
    }
}

/// Whether stdout is a terminal.
pub fn stdout_is_terminal() -> bool {
    std::io::stdout().is_terminal()
}
