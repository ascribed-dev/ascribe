//! What every subcommand that works on a project does first: find the
//! content model and load the project.

use std::io::IsTerminal;
use std::path::{Path, PathBuf};

use ascribe_check::Project;
use ascribe_core::Coded;
use ascribe_workspace::{LoadError, LocateError, ModelFile, Workspace, WorkspaceSnapshot};

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
/// current directory or a parent) and loads the project, once. Every command
/// that works on a project loads it here, and checks what [`checked`] makes
/// of it; `fmt` takes only the first half, with [`ascribe_check::LOADER`]'s
/// [`load_model`](ascribe_workspace::Loader::load_model).
pub fn load_project(global: &Global) -> Result<Workspace, Failure> {
    let (config, model) = load_model(global)?;
    load_rest(&config, model)
}

/// [`load_project`]'s first half: the content model, found and loaded, with
/// the path it was found at. A command that starts on something beside the
/// load, with the model, loads the rest with [`load_rest`].
pub fn load_model(global: &Global) -> Result<(PathBuf, ModelFile), Failure> {
    let config = locate(global).map_err(Failure::Config)?;
    let model = ascribe_check::LOADER
        .load_model(&config)
        .map_err(Failure::Load)?;
    Ok((config, model))
}

/// [`load_project`]'s second half: the project of the content model
/// [`load_model`] loaded from `config`.
pub fn load_rest(config: &Path, model: ModelFile) -> Result<Workspace, Failure> {
    Workspace::from_disk(ascribe_check::LOADER.side_files, config, model).map_err(Failure::Load)
}

/// The checks' view of a command's project, on today.
pub fn checked(workspace: &Workspace) -> Project {
    checked_snapshot(workspace.snapshot())
}

/// The checks' view of a snapshot of a project, on today.
pub fn checked_snapshot(snapshot: WorkspaceSnapshot) -> Project {
    Project::new(snapshot)
        .with_today(crate::clock::today())
        .with_plain_markdown(ascribe_emit::plain_markdown)
}

/// The content model `--config` names, or the nearest one.
pub fn locate(global: &Global) -> Result<PathBuf, LocateError> {
    ascribe_workspace::locate(global.config.as_deref())
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
