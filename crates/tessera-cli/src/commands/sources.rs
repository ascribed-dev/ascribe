//! `ascribe sources`: the copies of sources in other repositories (SPEC
//! §7.4). `fetch` makes them match `ascribe.lock`, and `status` says how
//! they stand. `fetch` is one of the two commands that reach another
//! repository; `status` reads only the project's files.

use std::io::{self, Write};
use std::process::ExitCode;
use std::sync::Arc;

use clap::{Args as ClapArgs, Subcommand, ValueEnum};
use serde::Serialize;
use tessera_check::{LoadError, Project};
use tessera_sources::{
    CopyState, FetchReport, Options, SourcesError, StatusReport, Workspace, short,
};

use crate::cli::Global;
use crate::commands::diff::fail;
use crate::context::{Failure, load_project};
use crate::exit;

/// The version of `status --format json`'s schema.
pub const STATUS_SCHEMA_VERSION: u32 = 1;

/// Arguments of `ascribe sources`.
#[derive(Debug, ClapArgs)]
pub struct Args {
    #[command(subcommand)]
    pub command: Command,
}

/// The subcommands of `ascribe sources`.
#[derive(Debug, Subcommand)]
pub enum Command {
    /// Copy the files snippets use from each source in another repository, at
    /// its pin in ascribe.lock.
    #[command(after_help = docs_page!("reference/cli/#ascribe-sources-fetch"))]
    Fetch(FetchArgs),
    /// Show each source in another repository: its pin, and the state of its
    /// copies. Reads only the project's files.
    #[command(after_help = docs_page!("reference/cli/#ascribe-sources-status"))]
    Status(StatusArgs),
}

/// Arguments of `ascribe sources fetch`.
#[derive(Debug, ClapArgs)]
pub struct FetchArgs {
    /// The sources to fetch. By default, every source in another repository.
    #[arg(value_name = "NAME")]
    pub names: Vec<String>,
}

/// Arguments of `ascribe sources status`.
#[derive(Debug, ClapArgs)]
pub struct StatusArgs {
    /// How to show it.
    #[arg(long, value_enum, default_value_t = StatusFormat::Text, value_name = "FORMAT")]
    pub format: StatusFormat,
}

/// The output format of `status`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
pub enum StatusFormat {
    /// Each source and its copies, for people.
    Text,
    /// One JSON document, for tools.
    Json,
}

/// Runs the command.
pub fn run(global: &Global, args: Args) -> ExitCode {
    let stdout = io::stdout();
    let mut out = io::BufWriter::new(stdout.lock());
    let stderr = io::stderr();
    let mut err = stderr.lock();
    let code = match args.command {
        Command::Fetch(args) => run_fetch(global, &args, &mut out, &mut err),
        Command::Status(args) => run_status(global, &args, &mut out, &mut err),
    };
    let _ = out.flush();
    exit::code(code)
}

/// The project, and the workspace the commands work on.
fn workspace(global: &Global, err: &mut dyn Write) -> Result<(Project, Workspace), u8> {
    let project = load_project(global).map_err(|failure| {
        let message = match failure {
            Failure::Config(message) => message,
            Failure::Load(LoadError::Model { diagnostics, .. }) => {
                let mut text = format!("{} has errors", tessera_check::MODEL_FILE);
                for d in diagnostics {
                    text.push_str(&format!("\n  {}", d.message));
                }
                text
            }
            Failure::Load(e @ LoadError::Read { .. }) => e.to_string(),
        };
        fail(err, &message)
    })?;
    let index = tessera_resolve::Project::load(
        Arc::new(project.model().clone()),
        project.layout().clone(),
        project.file_system(),
    );
    let workspace = Workspace::new(project.root(), project.model(), &index)
        .map_err(|e| fail(err, &e.to_string()))?;
    Ok((project, workspace))
}

/// Exit codes: 0 when the copies match the lock, 1 when a file a snippet
/// names couldn't be copied, 2 when it couldn't run.
fn run_fetch(global: &Global, args: &FetchArgs, out: &mut dyn Write, err: &mut dyn Write) -> u8 {
    let (_, workspace) = match workspace(global, err) {
        Ok(found) => found,
        Err(code) => return code,
    };
    let options = match Options::from_env() {
        Ok(options) => options,
        Err(e) => return fail(err, &e.to_string()),
    };
    let report = match tessera_sources::fetch(&workspace, &args.names, &options) {
        Ok(report) => report,
        Err(e) => return fail_sources(err, &e),
    };
    let _ = write_fetch(out, err, &report);
    if report.has_failures() {
        exit::PROBLEMS
    } else {
        exit::OK
    }
}

fn write_fetch(out: &mut dyn Write, err: &mut dyn Write, report: &FetchReport) -> io::Result<()> {
    for source in &report.sources {
        let name = &source.name;
        if let Some(commit) = &source.commit
            && source.pinned
        {
            writeln!(out, "{name}: pinned to {}", short(commit))?;
        }
        for path in &source.copied {
            writeln!(out, "{name}: copied {path}")?;
        }
        for path in &source.removed {
            writeln!(out, "{name}: removed {path}, which no snippet uses")?;
        }
        for failure in &source.failed {
            writeln!(
                err,
                "error: {name}: can't copy {}: {}",
                failure.path, failure.reason
            )?;
        }
        if source.first_copy {
            first_copy(err, name)?;
        }
    }
    for name in &report.unpinned {
        writeln!(
            out,
            "unpinned {name}, which ascribe.toml no longer declares in another repository"
        )?;
    }
    if report.lock_written {
        writeln!(out, "wrote ascribe.lock")?;
    } else if report
        .sources
        .iter()
        .all(|s| s.copied.is_empty() && s.removed.is_empty())
    {
        writeln!(out, "The copies already match ascribe.lock.")?;
    }
    Ok(())
}

/// What copying a source's files means, said the first time.
fn first_copy(err: &mut dyn Write, name: &str) -> io::Result<()> {
    writeln!(
        err,
        "note: these are the first files copied from {name}, into sources/{name}/. Commit them with the docs: everyone who can read this repository can read them, which matters when the code's repository is private."
    )
}

/// `git`'s failure, naming the source and repeating its message.
pub(crate) fn fail_sources(err: &mut dyn Write, e: &SourcesError) -> u8 {
    fail(err, &e.to_string())
}

#[derive(Serialize)]
struct StatusJson<'a> {
    schema_version: u32,
    ascribe_version: &'static str,
    #[serde(flatten)]
    report: &'a StatusReport,
}

/// Exit codes: 0 whatever it finds, 2 when the project can't be loaded.
fn run_status(global: &Global, args: &StatusArgs, out: &mut dyn Write, err: &mut dyn Write) -> u8 {
    let (_, workspace) = match workspace(global, err) {
        Ok(found) => found,
        Err(code) => return code,
    };
    let report = tessera_sources::status(&workspace);
    let written = match args.format {
        StatusFormat::Text => write_status(out, &report),
        StatusFormat::Json => serde_json::to_writer_pretty(
            &mut *out,
            &StatusJson {
                schema_version: STATUS_SCHEMA_VERSION,
                ascribe_version: env!("CARGO_PKG_VERSION"),
                report: &report,
            },
        )
        .map_err(io::Error::from)
        .and_then(|()| writeln!(out)),
    };
    if let Err(e) = written
        && e.kind() != io::ErrorKind::BrokenPipe
    {
        return fail(err, &format!("can't write the status: {e}"));
    }
    exit::OK
}

fn write_status(out: &mut dyn Write, report: &StatusReport) -> io::Result<()> {
    if report.sources.is_empty() {
        return writeln!(out, "No source is in another repository.");
    }
    for (i, source) in report.sources.iter().enumerate() {
        if i > 0 {
            writeln!(out)?;
        }
        match &source.branch {
            Some(branch) => writeln!(out, "{} ({}, {branch})", source.name, source.git)?,
            None => writeln!(out, "{} ({})", source.name, source.git)?,
        }
        match &source.commit {
            Some(commit) => writeln!(out, "  pinned to {commit}")?,
            None => writeln!(out, "  not pinned; `ascribe sources fetch` pins it")?,
        }
        if source.files.is_empty() {
            writeln!(out, "  no copies")?;
        }
        for file in &source.files {
            let state = match file.state {
                CopyState::Current => "current",
                CopyState::Changed => "changed here; `ascribe sources fetch` puts it back",
                CopyState::Missing => "missing; `ascribe sources fetch` copies it again",
                CopyState::Unlocked => "not in ascribe.lock; `ascribe sources fetch` replaces it",
                CopyState::Unused => "no snippet uses it; `ascribe sources fetch` removes it",
                CopyState::NotCopied => "not copied yet; `ascribe sources fetch` copies it",
            };
            writeln!(out, "  {}: {state}", file.path)?;
        }
    }
    Ok(())
}
