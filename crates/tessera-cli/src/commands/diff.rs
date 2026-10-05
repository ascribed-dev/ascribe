//! `ascribe diff`: what changed between a base revision and the working tree,
//! page by page and block by block, as readers will see it.
//!
//! The working tree is read from disk, as `ascribe build` reads it; the base
//! is read from git (`tessera_diff`). Each build asked for is resolved on
//! both sides and compared.

use std::io::{self, Write};
use std::process::ExitCode;
use std::sync::Arc;

use clap::{Args as ClapArgs, ValueEnum};
use tessera_check::{Diagnostic, LoadError};
use tessera_core::FileId;
use tessera_diff::html::{AssetFiles, DiskAssets, GitAssets, Version, write_html};
use tessera_diff::{
    BuildDiff, DiffError, PageDiff, PageStatus, Report, Repository, Revision, Side, compare_builds,
};

use crate::cli::Global;
use crate::commands::diagnose::{diagnose, select_builds};
use crate::context::{Failure, load_project};
use crate::exit;
use crate::report::Counts;

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
    let builds = match select_builds(&project, &args.build) {
        Ok(builds) => builds,
        Err(message) => return fail(err, &message),
    };
    let repo = match Repository::discover(project.root()) {
        Ok(repo) => repo,
        Err(e) => return fail_diff(err, e),
    };
    let base = match repo.base(args.base.as_deref(), args.base_exact) {
        Ok(base) => base,
        Err(e) => return fail_diff(err, e),
    };
    let before = match Revision::read(&repo, base.compared()) {
        Ok(before) => before,
        Err(e) => return fail_diff(err, e),
    };

    let now_project = tessera_resolve::Project::load(
        Arc::new(project.model().clone()),
        project.layout().clone(),
        project.file_system(),
    );
    let now_model = project
        .file(FileId::new(0))
        .map(|f| f.text)
        .unwrap_or_default();
    let before_project = before.as_ref().map(Revision::project);
    let before_side = before
        .as_ref()
        .zip(before_project.as_ref())
        .map(|(revision, project)| Side {
            project,
            model_text: &revision.model_text,
        });
    let now_side = Side {
        project: &now_project,
        model_text: now_model,
    };
    let names: Vec<&str> = builds.iter().map(|b| b.name.as_str()).collect();
    // Counting the errors is a full check of the working tree: it runs beside
    // the comparison, so the two take about as long as the slower one.
    let (diffs, errors) = std::thread::scope(|scope| {
        let errors = scope.spawn(|| working_tree_errors(&project, &args.build));
        let diffs = compare_builds(before_side, now_side, &names);
        (
            diffs,
            errors
                .join()
                .unwrap_or_else(|panic| std::panic::resume_unwind(panic)),
        )
    });
    let mut report = Report::new(&repo, &base, diffs);
    report.working_tree_errors = errors;
    if report.working_tree_errors > 0 {
        let _ = writeln!(
            err,
            "warning: {}",
            errors_line(report.working_tree_errors, &args.build)
        );
    }

    let written = match args.format {
        Format::Text => write_text(out, &report),
        Format::Json => serde_json::to_writer_pretty(&mut *out, &report)
            .map_err(io::Error::from)
            .and_then(|()| writeln!(out)),
        Format::Html => {
            let now_files = DiskAssets::new(project.root(), &now_project);
            let base_files = before.as_ref().map(|r| GitAssets::new(&repo, &r.fs));
            let base = before_project
                .as_ref()
                .zip(base_files.as_ref())
                .map(|(project, files)| Version {
                    project,
                    files: files as &dyn AssetFiles,
                });
            let now = Version {
                project: &now_project,
                files: &now_files,
            };
            out.write_all(write_html(&report, base, now).as_bytes())
        }
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

/// How many errors `ascribe check` finds for the builds compared, as it
/// would with the same `--build` options.
fn working_tree_errors(project: &tessera_check::Project, names: &[String]) -> usize {
    diagnose(project, names).map_or(0, |(diagnostics, _)| Counts::of(&diagnostics).errors)
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

fn failure_message(failure: Failure) -> String {
    match failure {
        Failure::Config(message) => message,
        Failure::Load(LoadError::Model { diagnostics, .. }) => model_errors(
            &format!("{} has errors", tessera_check::MODEL_FILE),
            diagnostics.iter().map(|d| d.message.clone()),
        ),
        Failure::Load(e @ LoadError::Read { .. }) => e.to_string(),
    }
}

fn fail_diff(err: &mut dyn Write, e: DiffError) -> u8 {
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

fn fail(err: &mut dyn Write, message: &str) -> u8 {
    let _ = writeln!(err, "error: {message}");
    exit::FAILURE
}
