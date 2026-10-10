//! `ascribe permalinks`: the project's list of permalinks,
//! `ascribe.permalinks` (SPEC §7.5), and retiring them (§7.6). `sync`
//! brings the list up to date, `list` shows every permalink and whether the
//! list has it, `verify` checks names against a list alone, as a product's
//! CI does, and `retire` ends permalinks on purpose, with a reason.

use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use ascribe_core::Coded;
use ascribe_model::{PERMALINKS_FILE, is_retired_placeholder};
use ascribe_query::permalinks::{ListState, ListedAs};
use ascribe_query::{PermalinksAnswer, Retire, RetireAnswer, SyncAnswer, VerifyAnswer};
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
    /// Retire permalinks, with a reason, in ascribe.toml's
    /// [permalinks.retired]: names gone from the content, every permalink
    /// on a page, or every one under a folder. Their addresses show the
    /// reason, or lead to --to. With ascribe.permalinks, records them there
    /// too.
    #[command(after_help = docs_link!("cli-permalinks-retire"))]
    Retire(RetireArgs),
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

/// Arguments of `ascribe permalinks retire`.
#[derive(Debug, ClapArgs)]
pub struct RetireArgs {
    /// Names to retire: permalinks the content no longer declares, which
    /// ascribe.permalinks has.
    #[arg(value_name = "NAME", required_unless_present_any = ["page", "under"])]
    pub names: Vec<String>,

    /// Why they were retired, for a reader whose link leads there.
    #[arg(long, value_name = "TEXT")]
    pub reason: String,

    /// Where their addresses lead now: a page, by its path from the content
    /// root, and `#` and a heading's id. Without it, an address shows the
    /// reason.
    #[arg(long, value_name = "PAGE")]
    pub to: Option<String>,

    /// Retire every permalink this page declares, or that ascribe.permalinks
    /// has on it, by its path from the content root.
    #[arg(long, value_name = "PAGE", conflicts_with = "under")]
    pub page: Option<String>,

    /// Retire every permalink declared in a file under this folder, or that
    /// ascribe.permalinks has there, by its path from the content root.
    #[arg(long, value_name = "FOLDER")]
    pub under: Option<String>,

    /// How to show what was retired.
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
    /// The reason given to `retire` is empty, or the placeholder.
    #[error(
        "give --reason a sentence that says why they were retired: a reader whose link leads there sees it"
    )]
    NoReason,
    /// `ascribe.toml` can't be changed in place.
    #[error("{0}")]
    ModelUnchanged(String),
    /// `ascribe.toml` can't be written.
    #[error("can't write {}: {message}", path.display())]
    ModelUnwritable {
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
            PermalinksError::NoReason => "retire_reason_missing",
            PermalinksError::ModelUnchanged(_) => "model_edit_refused",
            PermalinksError::ModelUnwritable { .. } => "model_unwritable",
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
        Command::Retire(args) => run_retire(global, &args, &mut out, &mut err),
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
    for name in &s.aliased {
        writeln!(out, "recorded the alias {name}")?;
    }
    for name in &s.retired {
        writeln!(out, "recorded {name} as retired")?;
    }
    if s.changed {
        writeln!(out, "wrote {PERMALINKS_FILE}")?;
    } else {
        writeln!(out, "{PERMALINKS_FILE} is up to date.")?;
    }
    for name in &s.removed {
        writeln!(
            err,
            "warning: kept {name}, which the project no longer has; `ascribe check` reports it until it's back or retired"
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
    if a.permalinks.is_empty()
        && a.aliases.is_empty()
        && a.retired.is_empty()
        && a.removed.is_empty()
    {
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
    let behind = |state: ListState| match (state, a.has_list) {
        (ListState::NotListed, true) => format!("  (not in {PERMALINKS_FILE} yet)"),
        _ => String::new(),
    };
    for alias in &a.aliases {
        let state = behind(alias.state);
        writeln!(
            out,
            "ascribe.toml: {} is an alias of {}{state}",
            alias.name, alias.target
        )?;
    }
    for retired in &a.retired {
        let state = behind(retired.state);
        let to = retired
            .to
            .as_deref()
            .map_or_else(String::new, |to| format!(", leading to {to}"));
        writeln!(
            out,
            "ascribe.toml: {} is retired{to}: {}{state}",
            retired.name, retired.reason
        )?;
    }
    for r in &a.removed {
        let was = match r.was {
            ListedAs::Permalink => format!("was in {}, and nothing declares it now", r.place),
            ListedAs::Alias => format!(
                "was an alias of {}, and ascribe.toml doesn't declare it now",
                r.place
            ),
            ListedAs::Retired => "was retired, and ascribe.toml doesn't retire it now".to_owned(),
        };
        writeln!(out, "{PERMALINKS_FILE}: {} {was}", r.name)?;
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

/// Exit codes: 0 when the list has every name, 1 when it doesn't have one
/// or has it as retired, 2 when the list or the names can't be read.
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
    if verified.missing.is_empty() && verified.retired.is_empty() {
        exit::OK
    } else {
        exit::PROBLEMS
    }
}

fn write_verify(out: &mut dyn Write, err: &mut dyn Write, v: &VerifyAnswer) -> io::Result<()> {
    for retired in &v.retired {
        writeln!(
            err,
            "error: {} was retired from the docs: {}",
            retired.name, retired.reason
        )?;
    }
    for name in &v.missing {
        writeln!(err, "error: {name} isn't a permalink in {}", v.list)?;
    }
    if v.missing.is_empty() && v.retired.is_empty() {
        let count = v.found.len();
        let names = if count == 1 { "name is" } else { "names are" };
        writeln!(out, "{count} {names} permalinks in {}.", v.list)?;
    }
    Ok(())
}

/// Exit codes: 0 when the names are retired; 2 when the project doesn't
/// load, a name can't be retired, or a file can't be read or written.
fn run_retire(global: &Global, args: &RetireArgs, out: &mut dyn Write, err: &mut dyn Write) -> u8 {
    let reason = args.reason.trim();
    if reason.is_empty() || is_retired_placeholder(reason) {
        return answer::fail(err, &PermalinksError::NoReason);
    }
    let config = match crate::context::locate(global) {
        Ok(config) => config,
        Err(e) => return answer::fail(err, &e),
    };
    let project = match load_project(global) {
        Ok(project) => project,
        Err(failure) => return answer::report_failure(err, &failure),
    };
    let index = project.index();
    let retire = Retire {
        names: &args.names,
        page: args.page.as_deref(),
        under: args.under.as_deref(),
    };
    let names = match ascribe_query::permalinks::retiring(&index, project.permalinks(), &retire) {
        Ok(names) => names,
        Err(e) => return answer::fail(err, &e),
    };
    let entries: Vec<ascribe_lsp::Retirement<'_>> = names
        .iter()
        .map(|name| ascribe_lsp::Retirement {
            name,
            reason,
            to: args.to.as_deref(),
        })
        .collect();
    let retired = match ascribe_lsp::retire(project.model_text(), &entries) {
        Ok(retired) => retired,
        Err(message) => return answer::fail(err, &PermalinksError::ModelUnchanged(message)),
    };
    if let Err(e) = std::fs::write(&config, &retired.text) {
        let error = PermalinksError::ModelUnwritable {
            path: config,
            message: e.to_string(),
        };
        return answer::fail(err, &error);
    }
    let list = match project.permalinks() {
        Some(Ok(list)) => ascribe_query::permalinks::record_retired(&index, list, &names, reason),
        _ => None,
    };
    if let Some(text) = &list
        && let Err(e) = std::fs::write(project.root().join(PERMALINKS_FILE), text)
    {
        let error = PermalinksError::Unwritable {
            path: PathBuf::from(PERMALINKS_FILE),
            message: e.to_string(),
        };
        return answer::fail(err, &error);
    }
    let done = RetireAnswer::new(names, reason, args.to.as_deref(), list.is_some());
    let result = match args.format {
        Format::Json => answer::write_json(out, &done),
        Format::Text => write_retire(out, &done, project.permalinks().is_some()),
    };
    answer::written(result, err).unwrap_or(exit::OK)
}

fn write_retire(out: &mut dyn Write, r: &RetireAnswer, has_list: bool) -> io::Result<()> {
    for name in &r.retired {
        match &r.to {
            Some(to) => writeln!(out, "retired {name}, leading to {to}")?,
            None => writeln!(out, "retired {name}")?,
        }
    }
    writeln!(out, "wrote ascribe.toml")?;
    if r.list_changed {
        writeln!(out, "wrote {PERMALINKS_FILE}")?;
    } else if has_list {
        writeln!(
            out,
            "The content still declares some of them: remove them, then run `ascribe permalinks sync`."
        )?;
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
