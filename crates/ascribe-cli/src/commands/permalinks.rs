//! `ascribe permalinks`: the project's list of permalinks,
//! `ascribe.permalinks` (SPEC §7.5). `sync` brings it up to date, `list`
//! shows every permalink and whether the list has it, and `verify` checks
//! names against a list alone, as a product's CI does.

use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use ascribe_core::Coded;
use ascribe_model::PERMALINKS_FILE;
use ascribe_query::permalinks::ListState;
use ascribe_query::{PermalinksAnswer, SyncAnswer, VerifyAnswer};
use clap::{Args as ClapArgs, Subcommand};

use crate::answer::{self, Format};
use crate::cli::Global;
use crate::context::load_project;
use crate::exit;

/// Arguments of `ascribe permalinks`.
#[derive(Debug, ClapArgs)]
pub struct Args {
    #[command(subcommand)]
    pub command: Command,
}

/// The subcommands of `ascribe permalinks`.
#[derive(Debug, Subcommand)]
pub enum Command {
    /// Bring ascribe.permalinks up to date with the permalinks the content
    /// declares, making it when there's none. A name that's gone from the
    /// content stays.
    #[command(after_help = docs_link!("cli-permalinks-sync"))]
    Sync(SyncArgs),
    /// Show every permalink the content declares, where, and whether
    /// ascribe.permalinks has it.
    #[command(after_help = docs_link!("cli-permalinks-list"))]
    List(ListArgs),
    /// Check that names are permalinks in ascribe.permalinks. Reads only the
    /// list, so a product's repository can check its links against a copy.
    #[command(after_help = docs_link!("cli-permalinks-verify"))]
    Verify(VerifyArgs),
}

/// Arguments of `ascribe permalinks sync`.
#[derive(Debug, ClapArgs)]
pub struct SyncArgs {
    /// How to show what changed.
    #[arg(long, value_enum, default_value_t = Format::Text, value_name = "FORMAT")]
    pub format: Format,
}

/// Arguments of `ascribe permalinks list`.
#[derive(Debug, ClapArgs)]
pub struct ListArgs {
    /// How to show the permalinks.
    #[arg(long, value_enum, default_value_t = Format::Text, value_name = "FORMAT")]
    pub format: Format,
}

/// Arguments of `ascribe permalinks verify`.
#[derive(Debug, ClapArgs)]
pub struct VerifyArgs {
    /// The names to check.
    #[arg(value_name = "NAME", required_unless_present = "from")]
    pub names: Vec<String>,

    /// Also check the names in this file, one on each line. Blank lines and
    /// lines that start with `#` are skipped.
    #[arg(long, value_name = "FILE")]
    pub from: Option<PathBuf>,

    /// The list to check against. By default, the ascribe.permalinks beside
    /// the project's ascribe.toml.
    #[arg(long, value_name = "PATH")]
    pub list: Option<PathBuf>,

    /// How to show the answer.
    #[arg(long, value_enum, default_value_t = Format::Text, value_name = "FORMAT")]
    pub format: Format,
}

/// Why a permalinks command couldn't do its work.
#[derive(Debug, thiserror::Error)]
pub enum PermalinksError {
    /// The list to check against isn't there.
    #[error("{} doesn't exist; pass the list with --list, or run `ascribe permalinks sync` in the docs to make one", .0.display())]
    ListNotFound(PathBuf),
    /// A file can't be read: the list, or the names.
    #[error("can't read {}: {message}", path.display())]
    Unreadable {
        /// The file.
        path: PathBuf,
        /// What went wrong.
        message: String,
    },
    /// The list can't be written.
    #[error("can't write {}: {message}", path.display())]
    Unwritable {
        /// The file.
        path: PathBuf,
        /// What went wrong.
        message: String,
    },
}

impl Coded for PermalinksError {
    fn code(&self) -> &'static str {
        match self {
            PermalinksError::ListNotFound(_) => "permalinks_list_not_found",
            PermalinksError::Unreadable { .. } => "permalinks_file_unreadable",
            PermalinksError::Unwritable { .. } => "permalinks_list_unwritable",
        }
    }
}

/// Runs the command.
pub fn run(global: &Global, args: Args) -> ExitCode {
    let stdout = io::stdout();
    let mut out = io::BufWriter::new(stdout.lock());
    let stderr = io::stderr();
    let mut err = stderr.lock();
    let code = match args.command {
        Command::Sync(args) => run_sync(global, &args, &mut out, &mut err),
        Command::List(args) => run_list(global, &args, &mut out, &mut err),
        Command::Verify(args) => run_verify(global, &args, &mut out, &mut err),
    };
    let _ = out.flush();
    exit::code(code)
}

/// Exit codes: 0 when the list is up to date, whether or not it changed; 2
/// when the project doesn't load, or the list can't be read or written.
fn run_sync(global: &Global, args: &SyncArgs, out: &mut dyn Write, err: &mut dyn Write) -> u8 {
    let project = match load_project(global) {
        Ok(project) => project,
        Err(failure) => return answer::report_failure(err, &failure),
    };
    let synced = match ascribe_query::permalinks::sync(
        &project.index(),
        project.permalinks(),
        project.permalinks_text(),
    ) {
        Ok(synced) => synced,
        Err(e) => return answer::fail(err, &e),
    };
    if synced.changed {
        let path = project.root().join(PERMALINKS_FILE);
        if let Err(e) = std::fs::write(&path, &synced.text) {
            let error = PermalinksError::Unwritable {
                path: PathBuf::from(PERMALINKS_FILE),
                message: e.to_string(),
            };
            return answer::fail(err, &error);
        }
    }
    let result = match args.format {
        Format::Json => answer::write_json(out, &synced),
        Format::Text => write_sync(out, err, &synced),
    };
    answer::written(result, err).unwrap_or(exit::OK)
}

fn write_sync(out: &mut dyn Write, err: &mut dyn Write, s: &SyncAnswer) -> io::Result<()> {
    if s.created {
        let count = s.added.len();
        let permalinks = if count == 1 {
            "permalink"
        } else {
            "permalinks"
        };
        return writeln!(
            out,
            "wrote {PERMALINKS_FILE}, with {count} {permalinks}; commit it with the docs"
        );
    }
    for name in &s.added {
        writeln!(out, "added {name}")?;
    }
    for moved in &s.moved {
        writeln!(out, "moved {}: {} to {}", moved.name, moved.from, moved.to)?;
    }
    if s.changed {
        writeln!(out, "wrote {PERMALINKS_FILE}")?;
    } else {
        writeln!(out, "{PERMALINKS_FILE} is up to date.")?;
    }
    for name in &s.removed {
        writeln!(
            err,
            "warning: kept {name}, which the content no longer declares; `ascribe check` reports it until it's back"
        )?;
    }
    Ok(())
}

/// Exit codes: 0 whatever the list says; 2 when the project doesn't load,
/// or its list can't be read.
fn run_list(global: &Global, args: &ListArgs, out: &mut dyn Write, err: &mut dyn Write) -> u8 {
    let project = match load_project(global) {
        Ok(project) => project,
        Err(failure) => return answer::report_failure(err, &failure),
    };
    let listed = match ascribe_query::permalinks::list(&project.index(), project.permalinks()) {
        Ok(listed) => listed,
        Err(e) => return answer::fail(err, &e),
    };
    let result = match args.format {
        Format::Json => answer::write_json(out, &listed),
        Format::Text => write_list(out, &listed),
    };
    answer::written(result, err).unwrap_or(exit::OK)
}

fn write_list(out: &mut dyn Write, a: &PermalinksAnswer) -> io::Result<()> {
    if a.permalinks.is_empty() && a.removed.is_empty() {
        return writeln!(out, "The content declares no permalinks.");
    }
    for p in &a.permalinks {
        let state = match (p.state, a.has_list) {
            (ListState::Listed, _) => String::new(),
            (ListState::NotListed, true) => format!("  (not in {PERMALINKS_FILE} yet)"),
            (ListState::NotListed, false) => String::new(),
            (ListState::Moved, _) => format!(
                "  ({PERMALINKS_FILE} has it in {})",
                p.listed_at.as_deref().unwrap_or_default()
            ),
        };
        writeln!(out, "{}:{}: {}{state}", p.file, p.line, p.name)?;
    }
    for r in &a.removed {
        writeln!(
            out,
            "{PERMALINKS_FILE}: {} was in {}, and nothing declares it now",
            r.name, r.place
        )?;
    }
    if !a.has_list {
        writeln!(out)?;
        writeln!(
            out,
            "No {PERMALINKS_FILE}: `ascribe permalinks sync` makes one, and then `ascribe check` reports a permalink that's removed."
        )?;
    }
    Ok(())
}

/// Exit codes: 0 when the list has every name, 1 when it doesn't have one,
/// 2 when the list or the names can't be read.
fn run_verify(global: &Global, args: &VerifyArgs, out: &mut dyn Write, err: &mut dyn Write) -> u8 {
    let list = match &args.list {
        Some(path) => path.clone(),
        None => match crate::context::locate(global) {
            Ok(config) => config
                .parent()
                .unwrap_or(Path::new("."))
                .join(PERMALINKS_FILE),
            Err(e) => return answer::fail(err, &e),
        },
    };
    let text = match read(&list) {
        Ok(text) => text,
        Err((_, io::ErrorKind::NotFound)) => {
            return answer::fail(err, &PermalinksError::ListNotFound(list));
        }
        Err((e, _)) => return answer::fail(err, &e),
    };
    let mut names = args.names.clone();
    if let Some(from) = &args.from {
        match read(from) {
            Ok(text) => names.extend(
                text.lines()
                    .map(str::trim)
                    .filter(|line| !line.is_empty() && !line.starts_with('#'))
                    .map(str::to_owned),
            ),
            Err((e, _)) => return answer::fail(err, &e),
        }
    }
    let shown = list.display().to_string();
    let verified = match ascribe_query::permalinks::verify(&text, &shown, &names) {
        Ok(verified) => verified,
        Err(e) => return answer::fail(err, &e),
    };
    let result = match args.format {
        Format::Json => answer::write_json(out, &verified),
        Format::Text => write_verify(out, err, &verified),
    };
    if let Some(code) = answer::written(result, err) {
        return code;
    }
    if verified.missing.is_empty() {
        exit::OK
    } else {
        exit::PROBLEMS
    }
}

fn write_verify(out: &mut dyn Write, err: &mut dyn Write, v: &VerifyAnswer) -> io::Result<()> {
    for name in &v.missing {
        writeln!(err, "error: {name} isn't a permalink in {}", v.list)?;
    }
    if v.missing.is_empty() {
        let count = v.found.len();
        let names = if count == 1 { "name is" } else { "names are" };
        writeln!(out, "{count} {names} permalinks in {}.", v.list)?;
    }
    Ok(())
}

/// A file named on the command line, or why it can't be read.
fn read(path: &Path) -> Result<String, (PermalinksError, io::ErrorKind)> {
    // Outside FileSystem: a file named on the command line, such as a copy
    // of a list in a product's repository, which `verify` reads with no
    // project at all.
    std::fs::read_to_string(path).map_err(|e| {
        let kind = e.kind();
        let error = PermalinksError::Unreadable {
            path: path.to_path_buf(),
            message: e.to_string(),
        };
        (error, kind)
    })
}
