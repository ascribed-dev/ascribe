//! `ascribe fmt [paths] [--check]`: rewrites Ascribe constructs into
//! canonical form (SPEC §8.3) with `tessera-fmt`.
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

use clap::Args as ClapArgs;
use tessera_check::{LoadError, Project};
use tessera_core::apply_edits;
use tessera_core::diagnostics::MODEL_TOML_SYNTAX;
use tessera_model::ContentModel;

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
        Err(message) => {
            let _ = writeln!(err, "error: {message}");
            exit::code(exit::FAILURE)
        }
    }
}

/// Formats every file, and returns how many changed (or would have).
fn format_all(global: &Global, options: &Args, out: &mut dyn Write) -> Result<usize, String> {
    let config = locate(global).map_err(|e| e.to_string())?;
    let project = config
        .parent()
        .map_or_else(|| PathBuf::from("."), Path::to_owned);
    let model = load_model(&config)?;
    let parse_options = tessera_fmt::options_from_model(&model);

    let roots: Vec<PathBuf> = if options.paths.is_empty() {
        vec![project.join(&model.project.content_root)]
    } else {
        options.paths.clone()
    };
    let own = std::fs::canonicalize(&project).ok();
    let mut files = Vec::new();
    for root in &roots {
        collect(root, own.as_deref(), &mut files)?;
    }
    files.sort();
    files.dedup();

    let mut changed = 0;
    for file in files {
        let source = std::fs::read(&file).map_err(|e| format!("{}: {e}", file.display()))?;
        let source = String::from_utf8(source)
            .map_err(|_| format!("{}: not valid UTF-8", file.display()))?;
        let edits = tessera_fmt::format(&source, &parse_options, &model);
        if edits.is_empty() {
            continue;
        }
        changed += 1;
        let formatted = apply_edits(&source, &edits)
            .map_err(|e| format!("{}: the formatter made bad edits: {e}", file.display()))?;
        if !options.check {
            std::fs::write(&file, formatted).map_err(|e| format!("{}: {e}", file.display()))?;
        }
        let verb = if options.check {
            "would reformat"
        } else {
            "formatted"
        };
        let _ = writeln!(out, "{verb} {}", file.display());
    }
    Ok(changed)
}

/// The content model, or why it can't be used, as `ascribe fmt` words it:
/// the pages aren't read, so a project whose pages have errors can still be
/// formatted.
fn load_model(config: &Path) -> Result<ContentModel, String> {
    Project::load_model(config)
        .map(|loaded| loaded.model)
        .map_err(|e| {
            let slugs: Vec<&str> = match &e {
                LoadError::Model { diagnostics, .. } => {
                    diagnostics.iter().map(|d| d.slug.as_str()).collect()
                }
                // The file couldn't be read, or isn't UTF-8: the content
                // model's syntax error.
                LoadError::Read { .. } => vec![MODEL_TOML_SYNTAX.as_str()],
            };
            format!(
                "{} isn't a valid content model ({}); run `ascribe check` for details",
                config.display(),
                slugs.join(", ")
            )
        })
}

/// Whether a directory holds a file named exactly `ascribe.toml` and isn't
/// `own`, this project's folder: it's another project's folder, whose files
/// that project's model formats.
fn holds_model(dir: &Path, own: Option<&Path>) -> bool {
    if own.is_some() && std::fs::canonicalize(dir).ok().as_deref() == own {
        return false;
    }
    std::fs::read_dir(dir).is_ok_and(|entries| {
        entries
            .flatten()
            .any(|e| e.file_name() == tessera_check::MODEL_FILE && e.path().is_file())
    })
}

/// Adds every `.md` file at or under `path` to `files`. `own` is the project's
/// folder, canonical.
fn collect(path: &Path, own: Option<&Path>, files: &mut Vec<PathBuf>) -> Result<(), String> {
    let meta = std::fs::metadata(path).map_err(|e| format!("{}: {e}", path.display()))?;
    if meta.is_file() {
        files.push(path.to_owned());
        return Ok(());
    }
    let entries = std::fs::read_dir(path).map_err(|e| format!("{}: {e}", path.display()))?;
    for entry in entries {
        let entry = entry.map_err(|e| format!("{}: {e}", path.display()))?;
        let child = entry.path();
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if child.is_dir() {
            if !name.starts_with('.') && name != "node_modules" && !holds_model(&child, own) {
                collect(&child, own, files)?;
            }
        } else if child.extension().is_some_and(|e| e == "md") {
            files.push(child);
        }
    }
    Ok(())
}
