//! `tessera fmt [paths] [--check]`: rewrites Tessera constructs into
//! canonical form (SPEC §8.3) with `tessera-fmt`.
//!
//! The command is [`run`], which the CLI calls with the parsed [`Options`];
//! [`parse_args`] reads them from the words after `fmt` for a CLI without an
//! argument parser of its own. Nothing here depends on how the CLI is
//! structured.
//!
//! - **What it formats.** Every `.md` file under each path (a file, or a
//!   directory searched recursively), or, with no path, under the project's
//!   content root. Directories whose names start with `.` and `node_modules`
//!   are skipped.
//! - **The project.** The nearest `tessera.toml` at or above the current
//!   directory. Its content model decides which lines are directives.
//! - **Without `--check`** files are rewritten in place, and each file that
//!   changed is listed. **With `--check`** nothing is written, and each file
//!   that would change is listed.
//! - **Exit status.** `0` when nothing needed formatting (`--check`) or every
//!   file was formatted; `1` under `--check` when a file would change; `2` for a
//!   problem: no project, an invalid model, an unreadable path or file.

use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use tessera_core::{FileId, apply_edits};
use tessera_model::ContentModel;

/// The command's arguments.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Options {
    /// Files and directories to format. Empty means the content root.
    pub paths: Vec<PathBuf>,
    /// Report what would change, and change nothing.
    pub check: bool,
}

/// Reads `[paths] [--check]`, the words after `fmt`.
pub fn parse_args(args: &[String]) -> Result<Options, String> {
    let mut options = Options::default();
    let mut paths_only = false;
    for arg in args {
        match arg.as_str() {
            "--" if !paths_only => paths_only = true,
            "--check" if !paths_only => options.check = true,
            flag if flag.starts_with('-') && !paths_only && flag != "-" => {
                return Err(format!("unknown option `{flag}`"));
            }
            path => options.paths.push(PathBuf::from(path)),
        }
    }
    Ok(options)
}

/// Runs `tessera fmt` on the process's own arguments (`tessera fmt ...`) and
/// streams. The interim entry point for a CLI without a command structure of
/// its own; a CLI with one parses the arguments itself and calls [`run`].
pub fn run_from_env() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(2).collect();
    match parse_args(&args) {
        Ok(options) => run(&options, &mut std::io::stdout(), &mut std::io::stderr()),
        Err(message) => {
            eprintln!("error: {message}\nusage: tessera fmt [--check] [paths]");
            ExitCode::from(2)
        }
    }
}

/// Runs `tessera fmt`: formatting output goes to `out`, problems to `err`.
pub fn run(options: &Options, out: &mut dyn Write, err: &mut dyn Write) -> ExitCode {
    match format_all(options, out) {
        Ok(changed) if options.check && changed > 0 => {
            let _ = writeln!(
                err,
                "{changed} file{} would be reformatted",
                if changed == 1 { "" } else { "s" }
            );
            ExitCode::from(1)
        }
        Ok(_) => ExitCode::SUCCESS,
        Err(message) => {
            let _ = writeln!(err, "error: {message}");
            ExitCode::from(2)
        }
    }
}

/// Formats every file, and returns how many changed (or would have).
fn format_all(options: &Options, out: &mut dyn Write) -> Result<usize, String> {
    let cwd =
        std::env::current_dir().map_err(|e| format!("can't read the current directory: {e}"))?;
    let project = find_project(&cwd)?;
    let model = load_model(&project)?;
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

/// The nearest directory at or above `from` that has a `tessera.toml`.
fn find_project(from: &Path) -> Result<PathBuf, String> {
    from.ancestors()
        .find(|dir| dir.join("tessera.toml").is_file())
        .map(Path::to_owned)
        .ok_or_else(|| {
            format!(
                "no tessera.toml found in {} or any directory above it",
                from.display()
            )
        })
}

fn load_model(project: &Path) -> Result<ContentModel, String> {
    let path = project.join("tessera.toml");
    tessera_model::load_with_file(&path, FileId::new(0)).map_err(|issues| {
        let slugs: Vec<&str> = issues.iter().map(|i| i.slug.as_str()).collect();
        format!(
            "{} isn't a valid content model ({}); run `tessera check` for details",
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

#[cfg(test)]
mod tests {
    use super::*;

    fn args(words: &[&str]) -> Vec<String> {
        words.iter().map(|w| (*w).to_owned()).collect()
    }

    #[test]
    fn reads_paths_and_check() {
        assert_eq!(parse_args(&[]), Ok(Options::default()));
        assert_eq!(
            parse_args(&args(&["docs", "--check", "a.md"])),
            Ok(Options {
                paths: vec!["docs".into(), "a.md".into()],
                check: true
            })
        );
        assert_eq!(
            parse_args(&args(&["--", "--check"])).map(|o| (o.paths, o.check)),
            Ok((vec![PathBuf::from("--check")], false))
        );
        assert!(parse_args(&args(&["--wat"])).is_err());
    }
}
