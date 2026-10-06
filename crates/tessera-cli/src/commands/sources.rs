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
use tessera_core::FileId;
use tessera_diff::{DiffError, DriftPage, DriftReport, Repository, Side, drift};
use tessera_sources::{
    CopyState, FetchReport, FileChange, Options, SourceUpdate, SourcesError, StatusReport,
    UpdateReport, Workspace, short,
};

use crate::cli::Global;
use crate::commands::diff::fail;
use crate::commands::drift::{escape, write_summary_groups, write_text_groups};
use crate::context::{Failure, load_project};
use crate::exit;

/// The version of `status --format json`'s schema.
pub const STATUS_SCHEMA_VERSION: u32 = 1;

/// The version of `update --format json`'s schema.
pub const UPDATE_SCHEMA_VERSION: u32 = 1;

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
    /// Move each source's pin to the head of its branch, copy its files again
    /// there, and say which pages' examples changed.
    #[command(after_help = docs_page!("reference/cli/#ascribe-sources-update"))]
    Update(UpdateArgs),
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

/// Arguments of `ascribe sources update`.
#[derive(Debug, ClapArgs)]
pub struct UpdateArgs {
    /// The sources to update. By default, every source in another
    /// repository.
    #[arg(value_name = "NAME")]
    pub names: Vec<String>,

    /// Move the pin to this revision instead of the head of the source's
    /// branch: a commit's full hash, a branch, or a tag.
    ///
    /// Only for one source: name it.
    #[arg(long, value_name = "REV")]
    pub to: Option<String>,

    /// How to show what changed.
    #[arg(long, value_enum, default_value_t = UpdateFormat::Text, value_name = "FORMAT")]
    pub format: UpdateFormat,
}

/// The output format of `update`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
pub enum UpdateFormat {
    /// Each source's commits and copies, then the pages, for people.
    Text,
    /// One JSON document, for tools.
    Json,
    /// Markdown for a pull request's description.
    Summary,
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
        Command::Update(args) => run_update(global, &args, &mut out, &mut err),
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

/// Exit codes: 0 when it ran, whether or not a pin moved; 2 when it
/// couldn't (no network, no access, an unknown revision), naming the source
/// and repeating `git`'s message. A file that couldn't be copied is reported,
/// and `ascribe check` fails on its snippet.
fn run_update(global: &Global, args: &UpdateArgs, out: &mut dyn Write, err: &mut dyn Write) -> u8 {
    let (project, workspace) = match workspace(global, err) {
        Ok(found) => found,
        Err(code) => return code,
    };
    let options = match Options::from_env() {
        Ok(options) => options,
        Err(e) => return fail(err, &e.to_string()),
    };
    let report =
        match tessera_sources::update(&workspace, &args.names, args.to.as_deref(), &options) {
            Ok(report) => report,
            Err(e) => return fail_sources(err, &e),
        };
    // The pages whose examples changed: the working tree, with the new
    // copies, against HEAD.
    let pages = if report.changed {
        pages(global)
    } else {
        Ok(None)
    };
    let site = project.model().consumer.site.as_deref();
    let to = args.to.as_deref();
    let written = match args.format {
        UpdateFormat::Text => write_update_text(out, err, &report, to, &pages),
        UpdateFormat::Json => {
            let (pages, unavailable) = match &pages {
                Ok(drift) => (drift.as_ref().map(|d| d.pages.as_slice()), None),
                Err(reason) => (None, Some(reason.as_str())),
            };
            serde_json::to_writer_pretty(
                &mut *out,
                &UpdateJson {
                    schema_version: UPDATE_SCHEMA_VERSION,
                    ascribe_version: env!("CARGO_PKG_VERSION"),
                    changed: report.changed,
                    sources: &report.sources,
                    pages,
                    pages_unavailable: unavailable,
                },
            )
            .map_err(io::Error::from)
            .and_then(|()| writeln!(out))
        }
        UpdateFormat::Summary => write_update_summary(out, &report, to, &pages, site),
    };
    if let Err(e) = written
        && e.kind() != io::ErrorKind::BrokenPipe
    {
        return fail(err, &format!("can't write the report: {e}"));
    }
    exit::OK
}

#[derive(Serialize)]
struct UpdateJson<'a> {
    schema_version: u32,
    ascribe_version: &'static str,
    changed: bool,
    sources: &'a [SourceUpdate],
    pages: Option<&'a [DriftPage]>,
    pages_unavailable: Option<&'a str>,
}

/// `ascribe drift` between `HEAD` and the working tree, over every build:
/// the pages whose examples the new copies change. `Err` says why there's
/// no telling, as when the project isn't in a git repository.
fn pages(global: &Global) -> Result<Option<DriftReport>, String> {
    let project = load_project(global).map_err(|_| "the project doesn't load".to_owned())?;
    let why = |e: DiffError| match e {
        DiffError::NotARepository { .. } => {
            "the project isn't in a git repository, so there's nothing to compare the copies with"
                .to_owned()
        }
        DiffError::UnknownRevision(_) => {
            "the repository has no commit yet to compare the copies with".to_owned()
        }
        e => e.to_string(),
    };
    let repo = Repository::discover(project.root()).map_err(why)?;
    let base = repo.base(Some("HEAD"), true).map_err(why)?;
    let now_project = tessera_resolve::Project::load(
        Arc::new(project.model().clone()),
        project.layout().clone(),
        project.file_system(),
    );
    let now = Side {
        project: &now_project,
        model_text: project
            .file(FileId::new(0))
            .map(|f| f.text)
            .unwrap_or_default(),
    };
    let names: Vec<&str> = project
        .model()
        .builds
        .iter()
        .map(|b| b.name.as_str())
        .collect();
    drift(&repo, &base, now, project.file_system(), &names)
        .map(Some)
        .map_err(why)
}

/// `9f2c41d → a3a8411`, or `a3a8411` for a first pin.
fn movement(source: &SourceUpdate, code: fn(&str) -> String) -> String {
    match &source.from {
        Some(from) => format!("{} \u{2192} {}", code(short(from)), code(short(&source.to))),
        None => code(short(&source.to)),
    }
}

/// Text from another repository as a Markdown code span, so a commit subject
/// can't mention someone, refer to an issue (`Fixes #12`), or add markup in
/// the pull request it's quoted in.
fn code_span(text: &str) -> String {
    let longest = text.split(|c| c != '`').map(str::len).max().unwrap_or(0);
    let fence = "`".repeat(longest + 1);
    let pad = if text.starts_with('`') || text.ends_with('`') {
        " "
    } else {
        ""
    };
    format!("{fence}{pad}{text}{pad}{fence}")
}

/// The address a comparison of two commits is under, for a repository on
/// github.com: `https://github.com/acme/api`. None for any other host, whose
/// addresses the summary doesn't guess.
fn compare_url(git: &str) -> Option<String> {
    let path = git
        .strip_prefix("https://github.com/")
        .or_else(|| git.strip_prefix("ssh://git@github.com/"))
        .or_else(|| git.strip_prefix("git@github.com:"))?;
    let path = path.trim_end_matches('/');
    let path = path.strip_suffix(".git").unwrap_or(path);
    let mut parts = path.split('/');
    let (owner, repo) = (parts.next()?, parts.next()?);
    let plain = |s: &str| {
        !s.is_empty()
            && s.chars()
                .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.'))
    };
    (parts.next().is_none() && plain(owner) && plain(repo))
        .then(|| format!("https://github.com/{owner}/{repo}"))
}

/// What the pin moved to: `the head of main`, or `v2.0` for `--to`.
fn target(source: &SourceUpdate, to: Option<&str>, code: fn(&str) -> String) -> String {
    match (to, source.followed.as_str()) {
        (Some(rev), _) => code(rev),
        (None, "HEAD") => "the head of the default branch".to_owned(),
        (None, branch) => format!("the head of {}", code(branch)),
    }
}

fn change_word(change: FileChange) -> &'static str {
    match change {
        FileChange::Added => "added",
        FileChange::Changed => "changed",
        FileChange::Removed => "removed",
    }
}

fn write_update_text(
    out: &mut dyn Write,
    err: &mut dyn Write,
    report: &UpdateReport,
    to: Option<&str>,
    pages: &Result<Option<DriftReport>, String>,
) -> io::Result<()> {
    for source in &report.sources {
        let name = &source.name;
        let plain = |s: &str| s.to_owned();
        if !source.moved && source.files.is_empty() {
            writeln!(
                out,
                "{name}: nothing to move; pinned to {}, {}",
                short(&source.to),
                target(source, to, plain)
            )?;
        } else {
            let commits = match &source.commits {
                _ if source.back => ", moved back".to_owned(),
                Some(c) if c.count == 1 => ", 1 commit".to_owned(),
                Some(c) => format!(", {} commits", c.count),
                None => String::new(),
            };
            writeln!(
                out,
                "{name}: {}, {}{commits}",
                movement(source, plain),
                target(source, to, plain)
            )?;
            if let Some(commits) = &source.commits {
                for line in &commits.newest {
                    writeln!(out, "  {} {}", short(&line.commit), line.subject)?;
                }
                if commits.count > commits.newest.len() {
                    writeln!(out, "  and {} more", commits.count - commits.newest.len())?;
                }
            }
            for file in &source.files {
                writeln!(out, "  {} {}", change_word(file.change), file.path)?;
            }
        }
        out.flush()?;
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
    if !report.changed {
        return writeln!(out, "\nNothing to move, and no file changed.");
    }
    match pages {
        Ok(Some(drift)) if !drift.pages.is_empty() => write_text_groups(out, drift),
        Ok(_) => writeln!(out, "\nNo page's examples changed."),
        Err(reason) => writeln!(out, "\nThe pages aren't listed: {reason}."),
    }
}

/// Markdown for a pull request's description: each source that moved, its
/// commits and copies, then the pages, as `ascribe drift`'s summary groups
/// them. Nothing when nothing moved.
fn write_update_summary(
    out: &mut dyn Write,
    report: &UpdateReport,
    to: Option<&str>,
    pages: &Result<Option<DriftReport>, String>,
    site: Option<&str>,
) -> io::Result<()> {
    if !report.changed {
        return Ok(());
    }
    writeln!(out, "### Sources\n")?;
    for source in &report.sources {
        if !source.moved && source.files.is_empty() {
            continue;
        }
        let commits = match &source.commits {
            _ if source.back => ": moved back".to_owned(),
            Some(c) if c.count == 1 => ": 1 commit".to_owned(),
            Some(c) => format!(": {} commits", c.count),
            None => String::new(),
        };
        let code = |s: &str| format!("`{s}`");
        let moved = match (&source.from, compare_url(&source.git)) {
            (Some(from), Some(url)) if source.moved && !source.back => {
                format!(
                    "[{}]({url}/compare/{from}...{})",
                    movement(source, code),
                    source.to
                )
            }
            _ => movement(source, code),
        };
        writeln!(
            out,
            "- **{}** {moved}, {}{commits}",
            escape(&source.name),
            target(source, to, code)
        )?;
        if let Some(commits) = &source.commits {
            for line in &commits.newest {
                writeln!(
                    out,
                    "  - `{}` {}",
                    short(&line.commit),
                    code_span(&line.subject)
                )?;
            }
            if commits.count > commits.newest.len() {
                writeln!(out, "  - and {} more", commits.count - commits.newest.len())?;
            }
        }
        if !source.files.is_empty() {
            let files: Vec<String> = source
                .files
                .iter()
                .map(|f| format!("`{}` {}", f.path, change_word(f.change)))
                .collect();
            writeln!(out, "  - Copies: {}.", files.join(", "))?;
        }
        for failure in &source.failed {
            writeln!(
                out,
                "  - Not copied: `{}`: {}.",
                failure.path,
                escape(&failure.reason)
            )?;
        }
    }
    match pages {
        Ok(Some(drift)) if !drift.pages.is_empty() => {
            writeln!(out, "\n### Examples that changed")?;
            write_summary_groups(out, drift, site)
        }
        Ok(_) => writeln!(out, "\nNo page's examples changed."),
        Err(reason) => writeln!(out, "\nThe pages aren't listed: {}.", escape(reason)),
    }
}

#[cfg(test)]
mod tests {
    use super::{code_span, compare_url};

    #[test]
    fn code_span_holds_any_subject() {
        assert_eq!(code_span("Fixes #12 @someone"), "`Fixes #12 @someone`");
        assert_eq!(code_span("Use `login` now"), "``Use `login` now``");
        assert_eq!(code_span("Use `login`"), "`` Use `login` ``");
        assert_eq!(code_span("`a``"), "``` `a`` ```");
    }

    #[test]
    fn compare_url_is_github_only() {
        for git in [
            "https://github.com/acme/api",
            "https://github.com/acme/api.git",
            "https://github.com/acme/api/",
            "git@github.com:acme/api.git",
            "ssh://git@github.com/acme/api.git",
        ] {
            assert_eq!(
                compare_url(git).as_deref(),
                Some("https://github.com/acme/api"),
                "{git}"
            );
        }
        for git in [
            "https://gitlab.com/acme/api.git",
            "https://github.com/acme",
            "https://github.com/acme/api/tree/main",
            "https://github.com/acme/a)pi",
            "file:///tmp/api",
        ] {
            assert_eq!(compare_url(git), None, "{git}");
        }
    }
}
