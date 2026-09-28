//! What every subcommand that works on a project does first: find the
//! content model and load the project.

use std::io::IsTerminal;
use tessera_check::{LoadError, Project};

use crate::cli::{Color, Global};

/// Why a project couldn't be loaded, ready to report.
pub enum Failure {
    /// No `tessera.toml` was found, or `--config` names none.
    Config(String),
    /// Loading failed.
    Load(LoadError),
}

/// Finds the content model (`--config`, or the nearest `tessera.toml` in the
/// current directory or a parent) and loads the project.
pub fn load_project(global: &Global) -> Result<Project, Failure> {
    let config = match &global.config {
        Some(path) if path.is_dir() => path.join(tessera_check::MODEL_FILE),
        Some(path) => path.clone(),
        None => {
            let cwd = std::env::current_dir()
                .map_err(|e| Failure::Config(format!("can't read the current directory: {e}")))?;
            Project::find_config(&cwd).ok_or_else(|| {
                Failure::Config(format!(
                    "no {} found in {} or any parent directory; run tessera from a project, or pass --config",
                    tessera_check::MODEL_FILE,
                    cwd.display()
                ))
            })?
        }
    };
    if !config.is_file() {
        return Err(Failure::Config(format!(
            "{} doesn't exist or isn't a file",
            config.display()
        )));
    }
    Project::load(&config).map_err(Failure::Load)
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
