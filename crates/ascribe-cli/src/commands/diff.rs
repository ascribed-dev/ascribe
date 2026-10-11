//! `ascribe diff`: what changed between a base revision and the working tree,
//! page by page and block by block, as readers will see it.
//!
//! The working tree is read from disk, as `ascribe build` reads it; the base
//! is read from git (`ascribe_diff`). Each build asked for is resolved on
//! both sides and compared.

use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::rc::Rc;

use ascribe_check::prompt::Builds;
use ascribe_check::{Diagnostic, LoadError};
use ascribe_core::{RelPath, percent_decode};
use ascribe_diff::{
    Anchor, BuildDiff, Change, ChangeKind, DiffError, DiffOptions, ProjectDiff, Report,
    diff_project,
};
use clap::{Args as ClapArgs, ValueEnum};

use crate::answer::{self, FromDisk, Loaded, Projects};
use crate::cli::Global;
use crate::context::Failure;
use crate::exit;

/// Arguments of `ascribe diff`.
#[derive(Debug, ClapArgs)]
pub struct Args {
    /// Show only this page's changes, block by block, or, for a fragment,
    /// the pages that changed through it: a path from the current directory,
    /// or from the content root.
    ///
    /// With `--format prompt`, the prompt is about this page alone, in the
    /// first build it changed in, or about the pages that changed through a
    /// fragment.
    #[arg(value_name = "PAGE")]
    pub page: Option<PathBuf>,

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

    /// List the changed pages without their block-level changes.
    ///
    /// With `--format json`, each page's `changes` is empty and its `counts`
    /// still count them, so the report stays short on a large change. With a
    /// PAGE, text leaves out the page's blocks; without one, it lists only
    /// pages anyway. HTML needs the blocks.
    #[arg(long)]
    pub pages_only: bool,
}

/// The output format.
#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
pub enum Format {
    /// The changed pages of each build, with counts, for people; with a
    /// PAGE, its changes too, each block as it was and as it is.
    Text,
    /// One JSON document, for tools.
    Json,
    /// One self-contained HTML file that shows every changed page rendered,
    /// with its changes marked, for reviewers.
    Html,
    /// A prompt for an agent that reviews the changes as readers will see
    /// them: about every changed page, or about the PAGE named. Nothing when
    /// nothing changed.
    Prompt,
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

/// Compares the project with the base: what the command reports, before
/// it's written, and the project. With `--pages-only`, the JSON report's
/// blocks are left out.
///
/// # Errors
///
/// The comparison couldn't be made; the message says why.
pub fn answer(
    projects: &dyn Projects,
    global: &Global,
    args: &Args,
) -> Result<(Rc<Loaded>, ProjectDiff), String> {
    let loaded = answer::load(projects, global, args.page.as_deref()).map_err(failure_message)?;
    let options = DiffOptions {
        base: args.base.as_deref(),
        base_exact: args.base_exact,
        builds: &args.build,
    };
    let mut diff = diff_project(&loaded.project, &options).map_err(diff_message)?;
    if args.pages_only && args.format == Format::Json {
        diff.report.omit_blocks();
    }
    Ok((loaded, diff))
}

fn diff(global: &Global, args: &Args, out: &mut dyn Write, err: &mut dyn Write) -> u8 {
    let (loaded, mut diff) = match answer(&FromDisk, global, args) {
        Ok(answer) => answer,
        Err(message) => return fail(err, &message),
    };
    // The prompt finds its PAGE itself, in the whole report.
    let page = match &args.page {
        Some(given) if args.format != Format::Prompt => {
            match narrow(&loaded.project, &mut diff.report, given) {
                Ok(path) => Some(path),
                Err(message) => return fail(err, &message),
            }
        }
        _ => None,
    };
    // Its prompts, in the HTML report too, are for an agent here.
    if let Ok(here) = std::env::current_dir() {
        diff.seen_from(here);
    }
    let project = &loaded.project;
    let report = &diff.report;
    if report.working_tree_errors > 0 {
        let _ = writeln!(
            err,
            "warning: {}",
            errors_line(report.working_tree_errors, &args.build)
        );
    }

    let written = match args.format {
        Format::Text => match &page {
            Some(path) => write_page_text(out, &diff, path, !args.pages_only),
            None => write_text(out, report),
        },
        Format::Json => serde_json::to_writer_pretty(&mut *out, report)
            .map_err(io::Error::from)
            .and_then(|()| writeln!(out)),
        Format::Html => out.write_all(diff.html().as_bytes()),
        Format::Prompt => match prompt(project, &diff, args) {
            Ok(Some(text)) => out.write_all(text.as_bytes()),
            Ok(None) => Ok(()),
            Err(message) => return fail(err, &message),
        },
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

/// The prompt `--format prompt` writes: about the PAGE named, or every
/// changed page. `None` when nothing it's about changed; an error when the
/// PAGE isn't one of the project's files, then or now.
pub(crate) fn prompt(
    project: &ascribe_check::Project,
    diff: &ProjectDiff,
    args: &Args,
) -> Result<Option<String>, String> {
    let Some(given) = &args.page else {
        let builds = if args.build.is_empty() {
            Builds::All
        } else {
            Builds::Named(args.build.clone())
        };
        return Ok(diff.pages_prompt(builds));
    };
    let candidates = answer::content_paths(project, given);
    if let Some(text) = candidates.iter().find_map(|path| diff.prompt_about(path)) {
        return Ok(Some(text));
    }
    if candidates
        .iter()
        .any(|path| project.source_at(path).is_some())
    {
        return Ok(None);
    }
    Err(not_a_file(given))
}

/// Narrows `report` to the PAGE named, as [`Report::narrow_to`] does, and
/// returns its content path. An error when it isn't one of the project's
/// files, then or now.
fn narrow(
    project: &ascribe_check::Project,
    report: &mut Report,
    given: &Path,
) -> Result<RelPath, String> {
    let candidates = answer::content_paths(project, given);
    let path = candidates
        .iter()
        .find(|path| report.reaches(path))
        .or_else(|| {
            candidates
                .iter()
                .find(|path| project.source_at(path).is_some())
        })
        .ok_or_else(|| not_a_file(given))?;
    report.narrow_to(path);
    Ok(path.clone())
}

fn not_a_file(given: &Path) -> String {
    format!(
        "{} isn't a file of the project, now or at the base",
        given.display()
    )
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
    write_base(out, report)?;
    for build in &report.builds {
        write_build(out, build)?;
    }
    Ok(())
}

/// The text about the PAGE at `path`: in each build, what changed on it, and
/// with `blocks`, each change, the block as it was and as it is. For a
/// fragment, the same about each page that changed through it.
fn write_page_text(
    out: &mut dyn Write,
    diff: &ProjectDiff,
    path: &RelPath,
    blocks: bool,
) -> io::Result<()> {
    let report = &diff.report;
    write_base(out, report)?;
    for build in &report.builds {
        if build.pages.is_empty() {
            writeln!(out, "{}: {path}: no changes", build.build)?;
        }
        for page in &build.pages {
            writeln!(
                out,
                "{}: {}: {}",
                build.build,
                page.path,
                page.describe(str::to_owned)
            )?;
            if blocks {
                for change in &page.changes {
                    write_change(out, diff, change)?;
                }
            }
        }
    }
    Ok(())
}

/// One change: what happened to the block, where, and its lines, `-` as
/// they were and `+` as they are (a moved block's, unchanged, after a
/// space). Changed prose is its text as a reader
/// reads it, with the words that differ marked as `git diff --word-diff`
/// marks them.
fn write_change(out: &mut dyn Write, diff: &ProjectDiff, change: &Change) -> io::Result<()> {
    let place = |anchor: &Option<Anchor>| anchor.as_ref().map(place).unwrap_or_default();
    let lines = |anchor: &Option<Anchor>, base: bool| {
        anchor
            .as_ref()
            .and_then(|a| diff.written(a, base))
            .unwrap_or_default()
    };
    match change.kind {
        ChangeKind::Changed => {
            writeln!(out, "  changed {}", place(&change.now))?;
            match &change.words {
                Some(words) => {
                    let was = marked(&words.was_text, &words.was, "[-", "-]");
                    let now = marked(&words.now_text, &words.now, "{+", "+}");
                    write_lines(out, '-', &[was.as_str()])?;
                    write_lines(out, '+', &[now.as_str()])
                }
                None => {
                    write_lines(out, '-', &lines(&change.was, true))?;
                    write_lines(out, '+', &lines(&change.now, false))
                }
            }
        }
        ChangeKind::Added => {
            writeln!(out, "  added {}", place(&change.now))?;
            write_lines(out, '+', &lines(&change.now, false))
        }
        ChangeKind::Removed => {
            writeln!(out, "  removed {} in the base", place(&change.was))?;
            match (lines(&change.was, true), &change.text) {
                (written, Some(text)) if written.is_empty() => {
                    write_lines(out, '-', &[text.as_str()])
                }
                (written, _) => write_lines(out, '-', &written),
            }
        }
        ChangeKind::Moved => {
            writeln!(
                out,
                "  moved {}, from {} in the base",
                place(&change.now),
                place(&change.was)
            )?;
            write_lines(out, ' ', &lines(&change.now, false))
        }
    }
}

/// Lines under a change, each after `sign`.
fn write_lines(out: &mut dyn Write, sign: char, lines: &[&str]) -> io::Result<()> {
    for line in lines {
        writeln!(out, "    {}", format!("{sign} {line}").trim_end())?;
    }
    Ok(())
}

/// `text` with each of `ranges` (character offsets, end exclusive) between
/// `open` and `close`.
fn marked(text: &str, ranges: &[[usize; 2]], open: &str, close: &str) -> String {
    let mut out = String::new();
    for (i, c) in text.chars().enumerate() {
        if ranges.iter().any(|[start, _]| *start == i) {
            out.push_str(open);
        }
        out.push(c);
        if ranges.iter().any(|[_, end]| *end == i + 1) {
            out.push_str(close);
        }
    }
    out
}

/// A block's place, as `<path>:<lines>` with its path decoded and one line
/// as one number, and the include it's shown through when it's written in
/// another file.
fn place(anchor: &Anchor) -> String {
    let source = |source: &str| match source.rsplit_once(':') {
        Some((path, lines)) => {
            let lines = match lines.split_once('-') {
                Some((first, last)) if first == last => first,
                _ => lines,
            };
            format!("{}:{lines}", percent_decode(path))
        }
        None => percent_decode(source),
    };
    let mut text = source(&anchor.source);
    if let Some(outer) = anchor.via.first() {
        text.push_str(&format!(", shown through {}", source(outer)));
    }
    text
}

/// The line that says what was compared with.
fn write_base(out: &mut dyn Write, report: &Report) -> io::Result<()> {
    let short = |commit: &str| commit.chars().take(7).collect::<String>();
    let base = named_base(&report.base.requested, &report.base.commit);
    match &report.base.merge_base {
        Some(merge_base) if *merge_base != report.base.commit => writeln!(
            out,
            "compared with {base}, from its merge base with HEAD ({})",
            short(merge_base)
        )?,
        _ => writeln!(out, "compared with {base}")?,
    }
    Ok(())
}

/// The base as given, with its commit after it unless what was given is
/// the commit: `main (1a2b3c4)`, and `1a2b3c4` alone.
fn named_base(requested: &str, commit: &str) -> String {
    let short: String = commit.chars().take(7).collect();
    let is_commit = requested.len() >= 4
        && requested.chars().all(|c| c.is_ascii_hexdigit())
        && commit.starts_with(&requested.to_ascii_lowercase());
    if is_commit {
        requested.to_owned()
    } else {
        format!("{requested} ({short})")
    }
}

fn write_build(out: &mut dyn Write, build: &BuildDiff) -> io::Result<()> {
    match build.pages.len() {
        0 => return writeln!(out, "{}: no changes", build.build),
        1 => writeln!(out, "{}: 1 page changed", build.build)?,
        n => writeln!(out, "{}: {n} pages changed", build.build)?,
    }
    for page in &build.pages {
        writeln!(out, "  {}: {}", page.path, page.describe(str::to_owned))?;
    }
    Ok(())
}

pub(crate) fn failure_message(failure: Failure) -> String {
    match failure {
        Failure::Config(e) => e.to_string(),
        Failure::Load(LoadError::Model { diagnostics, .. }) => model_errors(
            &format!("{} has errors", ascribe_check::MODEL_FILE),
            diagnostics.iter().map(|d| d.message.clone()),
        ),
        Failure::Load(e @ LoadError::Read { .. }) => e.to_string(),
    }
}

pub(crate) fn fail_diff(err: &mut dyn Write, e: DiffError) -> u8 {
    fail(err, &diff_message(e))
}

/// What a comparison that couldn't be made says.
fn diff_message(e: DiffError) -> String {
    match &e {
        DiffError::BaseModel { issues, .. } => model_errors(
            &e.to_string(),
            issues.iter().map(|i| Diagnostic::from_issue(i).message),
        ),
        _ => e.to_string(),
    }
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
