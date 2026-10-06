//! `ascribe fmt [paths] [--check]`: rewrites Ascribe constructs into
//! canonical form (SPEC §8.3) with `ascribe-fmt`.
//!
//! - **What it formats.** Every `.md` file under each path (a file, or a
//!   directory searched recursively), or, with no path, under the project's
//!   content root. Directories whose names start with `.`, `node_modules`,
//!   and directories below the starting one that hold an `ascribe.toml`
//!   (another project's folder, unless it's this project's own) are skipped.
//! - **The project.** `--config`, or the nearest `ascribe.toml` at or above the
//!   current directory. Its content model decides which lines are directives.
//! - **Without `--check`** files are rewritten in place, and each file that
//!   changed is listed. **With `--check`** nothing is written, and each file
//!   that would change is listed.
//! - **Exit status.** `0` when nothing needed formatting (`--check`) or every
//!   file was formatted; `1` under `--check` when a file would change; `2` for a
//!   problem: no project, an invalid model, an unreadable path or file.

use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use ascribe_check::{LoadError, LocateError, Project};
use ascribe_core::Coded;
use ascribe_core::diagnostics::MODEL_TOML_SYNTAX;
use ascribe_fmt::FormatFilesError;
use clap::Args as ClapArgs;

use crate::cli::Global;
use crate::context::locate;
use crate::exit;

/// Arguments of `ascribe fmt`.
#[derive(Debug, Default, ClapArgs)]
pub struct Args {
    /// Change nothing, and list each file that would change.
    ///
    /// Exits with 1 if any would. Use it in CI.
    #[arg(long)]
    pub check: bool,

    /// Files and directories to format: the `.md` files among them.
    ///
    /// By default, every `.md` file under the content root.
    ///
    /// Directories whose names start with `.`, `node_modules`, and
    /// directories inside the searched ones that hold an `ascribe.toml` other
    /// than the project's own (another project, formatted under its own
    /// model) are skipped.
    pub paths: Vec<PathBuf>,
}

/// Runs the command.
pub fn run(global: &Global, args: Args) -> ExitCode {
    let mut out = std::io::stdout().lock();
    let mut err = std::io::stderr().lock();
    match format_all(global, &args, &mut out) {
        Ok(changed) if args.check && changed > 0 => {
            let _ = writeln!(
                err,
                "{changed} file{} would be reformatted",
                if changed == 1 { "" } else { "s" }
            );
            exit::code(exit::PROBLEMS)
        }
        Ok(_) => exit::code(exit::OK),
        Err(e) => exit::code(exit::fail(&mut err, &e)),
    }
}

/// Why `ascribe fmt` couldn't format.
#[derive(Debug, thiserror::Error)]
pub enum FmtError {
    /// No content model was found.
    #[error(transparent)]
    Locate(#[from] LocateError),
    /// The content model can't be used, worded for `ascribe fmt`: the pages
    /// aren't read, so a project whose pages have errors can still be
    /// formatted.
    #[error(
        "{} isn't a valid content model ({}); run `ascribe check` for details",
        config.display(),
        model_slugs(error).join(", ")
    )]
    Model {
        /// The content model.
        config: PathBuf,
        /// Why it can't be used.
        error: LoadError,
    },
    /// A file couldn't be read, written, or formatted.
    #[error(transparent)]
    Format(#[from] FormatFilesError),
}

impl Coded for FmtError {
    fn code(&self) -> &'static str {
        match self {
            FmtError::Locate(e) => e.code(),
            FmtError::Model { error, .. } => error.code(),
            FmtError::Format(e) => e.code(),
        }
    }
}

/// The slugs of the content model's problems.
fn model_slugs(error: &LoadError) -> Vec<&str> {
    match error {
        LoadError::Model { diagnostics, .. } => {
            diagnostics.iter().map(|d| d.slug.as_str()).collect()
        }
        // The file couldn't be read, or isn't UTF-8: the content model's
        // syntax error.
        LoadError::Read { .. } => vec![MODEL_TOML_SYNTAX.as_str()],
    }
}

/// Formats every file, listing each that changed (or would have), and
/// returns how many did.
fn format_all(global: &Global, options: &Args, out: &mut dyn Write) -> Result<usize, FmtError> {
    let config = locate(global)?;
    let model = Project::load_model(&config)
        .map_err(|error| FmtError::Model {
            config: config.clone(),
            error,
        })?
        .model;
    let verb = if options.check {
        "would reformat"
    } else {
        "formatted"
    };
    let mut list = |file: &Path| {
        let _ = writeln!(out, "{verb} {}", file.display());
    };
    let changed =
        ascribe_fmt::format_files(&config, &model, &options.paths, options.check, &mut list)?;
    Ok(changed.len())
}
