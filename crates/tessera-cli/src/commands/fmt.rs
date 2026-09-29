//! `ascribe fmt [paths] [--check]`: rewrites Ascribe constructs into
//! canonical form (SPEC §8.3) with `tessera-fmt`.
//!
//! - **What it formats.** Every `.md` file under each path (a file, or a
//!   directory searched recursively), or, with no path, under the project's
//!   content root. Directories whose names start with `.` and `node_modules`
//!   are skipped.
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
use tessera_core::{FileId, apply_edits};
use tessera_model::ContentModel;

use crate::cli::Global;
use crate::exit;

/// Arguments of `ascribe fmt`.
#[derive(Debug, Default, ClapArgs)]
pub struct Args {
    /// Files and directories to format. By default, the content root.
    pub paths: Vec<PathBuf>,

    /// Report the files that would change, and change nothing. Exits with 1
    /// if any would.
    #[arg(long)]
    pub check: bool,
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
    let config = find_config(global)?;
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
    let mut files = Vec::new();
    for root in &roots {
        collect(root, &mut files)?;
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

/// `--config`, or the nearest `ascribe.toml` in the current directory or a parent.
fn find_config(global: &Global) -> Result<PathBuf, String> {
    let config = match &global.config {
        Some(path) if path.is_dir() => path.join(tessera_check::MODEL_FILE),
        Some(path) => path.clone(),
        None => {
            let cwd = std::env::current_dir()
                .map_err(|e| format!("can't read the current directory: {e}"))?;
            tessera_check::Project::find_config(&cwd).ok_or_else(|| {
                format!(
                    "no {} found in {} or any parent directory; run ascribe from a project, or pass --config",
                    tessera_check::MODEL_FILE,
                    cwd.display()
                )
            })?
        }
    };
    if config.is_file() {
        Ok(config)
    } else {
        Err(format!(
            "{} doesn't exist or isn't a file",
            config.display()
        ))
    }
}

fn load_model(path: &Path) -> Result<ContentModel, String> {
    tessera_model::load_with_file(path, FileId::new(0)).map_err(|issues| {
        let slugs: Vec<&str> = issues.iter().map(|i| i.slug.as_str()).collect();
        format!(
            "{} isn't a valid content model ({}); run `ascribe check` for details",
            path.display(),
            slugs.join(", ")
        )
    })
}

/// Adds every `.md` file at or under `path` to `files`.
fn collect(path: &Path, files: &mut Vec<PathBuf>) -> Result<(), String> {
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
            if !name.starts_with('.') && name != "node_modules" {
                collect(&child, files)?;
            }
        } else if child.extension().is_some_and(|e| e == "md") {
            files.push(child);
        }
    }
    Ok(())
}
