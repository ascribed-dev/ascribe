//! `ascribe diff`: what changed between a base revision and the working tree,
//! page by page and block by block, as readers will see it.
//!
//! The working tree is read from disk, as `ascribe build` reads it; the base
//! is read from git (`tessera_diff`). Each build asked for is resolved on
//! both sides and compared.

use std::io::{self, Write};
use std::process::ExitCode;

use clap::{Args as ClapArgs, ValueEnum};
use tessera_check::{Diagnostic, LoadError};
use tessera_diff::{BuildDiff, DiffError, DiffOptions, PageDiff, PageStatus, Report, diff_project};

use crate::cli::Global;
use crate::context::{Failure, load_project};
use crate::exit;

/// Arguments of `ascribe diff`.
#[derive(Debug, ClapArgs)]
pub struct Args {
    /// The revision to compare with, anything git accepts (a branch, a tag, a
    /// commit).
    ///
    /// By default, the repository's default branch, the first of
    /// `origin/HEAD`, `origin/main`, `origin/master`, `main`, and `master`
    /// that exists.
    ///
    /// The comparison starts from the merge base of that revision and `HEAD`,
    /// as a pull request shows its changes, so commits made on the base
    /// branch since you branched aren't listed. A shallow clone (what
    /// `actions/checkout` makes by default) may not have the merge base: fetch
    /// more history (`fetch-depth: 0`) or use `--base-exact`.
    #[arg(long, value_name = "REV")]
    pub base: Option<String>,

    /// Compare with the revision itself instead of the merge base.
    #[arg(long)]
    pub base_exact: bool,

    /// Compare only this build.
    ///
    /// Repeat it for several. By default, every build in `ascribe.toml`.
    #[arg(long, value_name = "NAME")]
    pub build: Vec<String>,

    /// How to show the changes.
    #[arg(long, value_enum, default_value_t = Format::Text, value_name = "FORMAT")]
    pub format: Format,

    /// Exit with 1 when anything changed, as `git diff --exit-code` does.
    #[arg(long)]
    pub exit_code: bool,
}

/// The output format.
#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
pub enum Format {
    /// The changed pages of each build, with counts, for people.
    Text,
    /// One JSON document, for tools.
    Json,
    /// One self-contained HTML file that shows every changed page rendered,
    /// with its changes marked, for reviewers.
    Html,
}

/// Runs the command. Exit codes: 0 whether or not anything changed (1 when
/// it did, with `--exit-code`), 2 when it couldn't run: not a git
/// repository, an unknown revision, no `git`, or a project that doesn't load
/// on either side. Errors in the working tree's pages don't stop it, since
/// work in progress is worth comparing, but it says how many there are.
pub fn run(global: &Global, args: Args) -> ExitCode {
    let stdout = io::stdout();
    let mut out = io::BufWriter::new(stdout.lock());
    let stderr = io::stderr();
    let mut err = stderr.lock();
    let code = diff(global, &args, &mut out, &mut err);
    let _ = out.flush();
    exit::code(code)
}

fn diff(global: &Global, args: &Args, out: &mut dyn Write, err: &mut dyn Write) -> u8 {
    let project = match load_project(global) {
        Ok(project) => project,
        Err(failure) => return fail(err, &failure_message(failure)),
    };
    let options = DiffOptions {
        base: args.base.as_deref(),
        base_exact: args.base_exact,
        builds: &args.build,
    };
    let diff = match diff_project(&project, &options) {
        Ok(diff) => diff,
        Err(e) => return fail_diff(err, e),
    };
    let report = &diff.report;
    if report.working_tree_errors > 0 {
        let _ = writeln!(
            err,
            "warning: {}",
            errors_line(report.working_tree_errors, &args.build)
        );
    }

    let written = match args.format {
        Format::Text => write_text(out, report),
        Format::Json => serde_json::to_writer_pretty(&mut *out, report)
            .map_err(io::Error::from)
            .and_then(|()| writeln!(out)),
        Format::Html => out.write_all(diff.html().as_bytes()),
    };
    if let Err(e) = written
        && e.kind() != io::ErrorKind::BrokenPipe
    {
        return fail(err, &format!("can't write the report: {e}"));
    }
    if args.exit_code && report.has_changes() {
        exit::PROBLEMS
    } else {
        exit::OK
    }
}

/// The warning about the working tree's errors, naming the `ascribe check`
/// that lists them.
fn errors_line(errors: usize, names: &[String]) -> String {
    let (count, them) = if errors == 1 {
        ("1 error".to_owned(), "it")
    } else {
        (format!("{errors} errors"), "them")
    };
    let mut check = "ascribe check".to_owned();
    for name in names {
        check.push_str(&format!(" --build {name}"));
    }
    format!("the working tree has {count}; `{check}` lists {them}")
}

fn write_text(out: &mut dyn Write, report: &Report) -> io::Result<()> {
    let short = |commit: &str| commit.chars().take(7).collect::<String>();
    match &report.base.merge_base {
        Some(merge_base) if *merge_base != report.base.commit => writeln!(
            out,
            "compared with {} ({}), from its merge base with HEAD ({})",
            report.base.requested,
            short(&report.base.commit),
            short(merge_base)
        )?,
        _ => writeln!(
            out,
            "compared with {} ({})",
            report.base.requested,
            short(&report.base.commit)
        )?,
    }
    for build in &report.builds {
        write_build(out, build)?;
    }
    Ok(())
}

fn write_build(out: &mut dyn Write, build: &BuildDiff) -> io::Result<()> {
    match build.pages.len() {
        0 => return writeln!(out, "{}: no changes", build.build),
        1 => writeln!(out, "{}: 1 page changed", build.build)?,
        n => writeln!(out, "{}: {n} pages changed", build.build)?,
    }
    for page in &build.pages {
        writeln!(out, "  {}: {}", page.path, describe(page))?;
    }
    Ok(())
}

/// A page's line: what changed, and where the change comes from when it
/// isn't only the page's own file.
fn describe(page: &PageDiff) -> String {
    let mut text = match page.status {
        PageStatus::Added => "added".to_owned(),
        PageStatus::Removed => "removed".to_owned(),
        PageStatus::Changed => {
            let c = page.counts;
            let mut parts: Vec<String> = [
                (c.changed, "changed"),
                (c.added, "added"),
                (c.removed, "removed"),
                (c.moved, "moved"),
            ]
            .iter()
            .filter(|(n, _)| *n > 0)
            .map(|(n, kind)| format!("{n} {kind}"))
            .collect();
            if !page.page_changed.is_empty() {
                parts.push(format!("{} changed", and_list(&page.page_changed)));
            }
            parts.join(", ")
        }
    };
    if !page.because.is_empty() {
        let lead = if page.own_file_changed {
            "also through"
        } else {
            "through"
        };
        text.push_str(&format!(" ({lead} {})", page.because.join(", ")));
    }
    text
}

/// `a`, `a and b`, `a, b, and c`.
fn and_list(items: &[&str]) -> String {
    match items {
        [] => String::new(),
        [one] => (*one).to_owned(),
        [a, b] => format!("{a} and {b}"),
        [rest @ .., last] => format!("{}, and {last}", rest.join(", ")),
    }
}

pub(crate) fn failure_message(failure: Failure) -> String {
    match failure {
        Failure::Config(e) => e.to_string(),
        Failure::Load(LoadError::Model { diagnostics, .. }) => model_errors(
            &format!("{} has errors", tessera_check::MODEL_FILE),
            diagnostics.iter().map(|d| d.message.clone()),
        ),
        Failure::Load(e @ LoadError::Read { .. }) => e.to_string(),
    }
}

pub(crate) fn fail_diff(err: &mut dyn Write, e: DiffError) -> u8 {
    let message = match &e {
        DiffError::BaseModel { issues, .. } => model_errors(
            &e.to_string(),
            issues.iter().map(|i| Diagnostic::from_issue(i).message),
        ),
        _ => e.to_string(),
    };
    fail(err, &message)
}

/// A content model's problems under one line saying whose they are.
fn model_errors(lead: &str, messages: impl Iterator<Item = String>) -> String {
    let mut text = format!("{lead}, so nothing can be compared");
    for message in messages {
        text.push_str(&format!("\n  {message}"));
    }
    text
}

pub(crate) fn fail(err: &mut dyn Write, message: &str) -> u8 {
    let _ = writeln!(err, "error: {message}");
    exit::FAILURE
}
