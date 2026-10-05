//! `ascribe drift`: the pages whose examples changed between a base revision
//! and the working tree, split by whether the words around them changed
//! too.
//!
//! The working tree is read from disk; the base's code files and, when an
//! example changed, its pages are read from git (`tessera_diff::drift`).

use std::io::{self, Write};
use std::process::ExitCode;
use std::sync::Arc;

use clap::{Args as ClapArgs, ValueEnum};
use tessera_core::FileId;
use tessera_diff::{DiffError, DriftPage, DriftReport, Repository, Side, drift};

use crate::cli::Global;
use crate::commands::diagnose::select_builds;
use crate::commands::diff::{fail, fail_diff, failure_message};
use crate::context::load_project;
use crate::exit;

/// Arguments of `ascribe drift`.
#[derive(Debug, ClapArgs)]
pub struct Args {
    /// The revision to compare with, anything git accepts (a branch, a tag, a
    /// commit).
    ///
    /// By default, the repository's default branch, found as `ascribe diff`
    /// finds it. The comparison starts from the merge base of that revision
    /// and `HEAD`, as a pull request shows its changes. A shallow clone (what
    /// `actions/checkout` makes by default) may not have the merge base:
    /// fetch more history (`fetch-depth: 0`).
    #[arg(long, value_name = "REV")]
    pub base: Option<String>,

    /// Look only at this build.
    ///
    /// Repeat it for several. By default, every build in `ascribe.toml`.
    #[arg(long, value_name = "NAME")]
    pub build: Vec<String>,

    /// How to show the report.
    #[arg(long, value_enum, default_value_t = Format::Text, value_name = "FORMAT")]
    pub format: Format,

    /// Exit with 1 when a page's examples changed and the page didn't.
    #[arg(long)]
    pub exit_code: bool,
}

/// The output format.
#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
pub enum Format {
    /// The two groups of pages, with each example that changed, for people.
    Text,
    /// One JSON document, for tools.
    Json,
    /// Markdown for a CI job's summary, each page linked to the site; nothing
    /// when no example changed.
    Summary,
}

/// Runs the command. Exit codes: 0 whatever it finds (1 when a page's
/// examples changed and the page didn't, with `--exit-code`), 2 when it
/// couldn't run: not a git repository, an unknown revision, a shallow clone,
/// no `git`, or a project that doesn't load on either side.
pub fn run(global: &Global, args: Args) -> ExitCode {
    let stdout = io::stdout();
    let mut out = io::BufWriter::new(stdout.lock());
    let stderr = io::stderr();
    let mut err = stderr.lock();
    let code = report(global, &args, &mut out, &mut err);
    let _ = out.flush();
    exit::code(code)
}

fn report(global: &Global, args: &Args, out: &mut dyn Write, err: &mut dyn Write) -> u8 {
    let project = match load_project(global) {
        Ok(project) => project,
        Err(failure) => return fail(err, &failure_message(failure)),
    };
    let builds = match select_builds(&project, &args.build) {
        Ok(builds) => builds,
        Err(message) => return fail(err, &message),
    };
    let found = Repository::discover(project.root()).and_then(|repo| {
        let base = repo.base(args.base.as_deref(), false)?;
        Ok((repo, base))
    });
    let (repo, base) = match found {
        Ok(found) => found,
        Err(e) => return fail_drift(err, e),
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
    let now = Side {
        project: &now_project,
        model_text: now_model,
    };
    let names: Vec<&str> = builds.iter().map(|b| b.name.as_str()).collect();
    let report = match drift(&repo, &base, now, &names) {
        Ok(report) => report,
        Err(e) => return fail_drift(err, e),
    };

    let site = project.model().consumer.site.as_deref();
    let written = match args.format {
        Format::Text => write_text(out, &report),
        Format::Json => serde_json::to_writer_pretty(&mut *out, &report)
            .map_err(io::Error::from)
            .and_then(|()| writeln!(out)),
        Format::Summary => write_summary(out, &report, site),
    };
    if let Err(e) = written
        && e.kind() != io::ErrorKind::BrokenPipe
    {
        return fail(err, &format!("can't write the report: {e}"));
    }
    if args.exit_code && report.unchanged_pages().next().is_some() {
        exit::PROBLEMS
    } else {
        exit::OK
    }
}

/// The lead of the first group.
const UNCHANGED: &str =
    "Examples that changed. The page shows the new code; check the words around it:";
/// The lead of the second group.
const CHANGED: &str = "Examples that changed along with the page:";

fn short(commit: &str) -> String {
    commit.chars().take(7).collect()
}

/// `compared with main (1a2b3c4)`, and the merge base when it's another
/// commit.
fn compared(report: &DriftReport, code: fn(&str) -> String) -> String {
    let base = &report.base;
    let mut text = format!(
        "compared with {} ({})",
        code(&base.requested),
        code(&short(&base.commit))
    );
    if let Some(merge_base) = &base.merge_base
        && *merge_base != base.commit
    {
        text.push_str(&format!(
            ", from its merge base with HEAD ({})",
            code(&short(merge_base))
        ));
    }
    text
}

/// `+4 −1`.
fn amount(added: usize, removed: usize) -> String {
    format!("+{added} \u{2212}{removed}")
}

fn write_text(out: &mut dyn Write, report: &DriftReport) -> io::Result<()> {
    writeln!(out, "{}", compared(report, |s| s.to_owned()))?;
    if report.pages.is_empty() {
        return writeln!(out, "\nNo examples changed.");
    }
    let groups: [(&str, Vec<&DriftPage>); 2] = [
        (UNCHANGED, report.unchanged_pages().collect()),
        (CHANGED, report.changed_pages().collect()),
    ];
    for (lead, pages) in groups {
        if pages.is_empty() {
            continue;
        }
        writeln!(out, "\n{lead}")?;
        for page in pages {
            writeln!(out, "  {}", page.path)?;
            for example in &page.examples {
                writeln!(
                    out,
                    "    {} ({})",
                    example.address,
                    amount(example.added, example.removed)
                )?;
            }
        }
    }
    Ok(())
}

/// Markdown for a CI job's summary: nothing when there's nothing to report.
fn write_summary(out: &mut dyn Write, report: &DriftReport, site: Option<&str>) -> io::Result<()> {
    if report.pages.is_empty() {
        return Ok(());
    }
    writeln!(out, "### Examples that changed\n")?;
    writeln!(
        out,
        "{}.",
        upper_first(&compared(report, |s| format!("`{s}`")))
    )?;
    let groups: [(&str, Vec<&DriftPage>); 2] = [
        (
            "The page shows the new code; check the words around it:",
            report.unchanged_pages().collect(),
        ),
        (
            "Changed along with the page:",
            report.changed_pages().collect(),
        ),
    ];
    for (lead, pages) in groups {
        if pages.is_empty() {
            continue;
        }
        writeln!(out, "\n{lead}\n")?;
        for page in pages {
            let name = escape(&page.path);
            match site {
                Some(site) => writeln!(
                    out,
                    "- [{name}]({}{})",
                    site.trim_end_matches('/'),
                    page.route
                )?,
                None => writeln!(out, "- {name}")?,
            }
            for example in &page.examples {
                writeln!(
                    out,
                    "  - `{}` ({})",
                    example.address,
                    amount(example.added, example.removed)
                )?;
            }
        }
    }
    Ok(())
}

fn upper_first(text: &str) -> String {
    let mut chars = text.chars();
    chars
        .next()
        .map(|c| c.to_uppercase().chain(chars).collect())
        .unwrap_or_default()
}

/// A path as Markdown text: the characters that would start markup, escaped.
fn escape(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for c in text.chars() {
        if matches!(c, '\\' | '[' | ']' | '*' | '_' | '`' | '<' | '>') {
            out.push('\\');
        }
        out.push(c);
    }
    out
}

/// `ascribe diff`'s messages, without the options `drift` doesn't have.
fn fail_drift(err: &mut dyn Write, e: DiffError) -> u8 {
    let message = match &e {
        DiffError::GitNotFound => {
            "git not found: ascribe drift runs `git`, so it needs git installed and on the path"
                .to_owned()
        }
        DiffError::ShallowHistory(rev) => format!(
            "this clone doesn't have enough history to find where the branch left `{rev}`: fetch more of it (in GitHub Actions, `fetch-depth: 0` on actions/checkout)"
        ),
        DiffError::NoCommonHistory(rev) => {
            format!("`{rev}` and HEAD share no history, so there's no merge base to compare with")
        }
        _ => return fail_diff(err, e),
    };
    fail(err, &message)
}
