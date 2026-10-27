//! `ascribe report`: what state a project is in, section by section, for
//! what's too slow, too networked, or too broad for `ascribe check`
//! ([`ascribe_query::report`]). Each section is independent; one that can't
//! run says why, and the others still run.

pub mod json;
mod store;
mod text;

use std::io::{self, Write};
use std::process::ExitCode;
use std::sync::Arc;
use std::time::Duration;

use ascribe_check::links::{Cache, History};
use ascribe_check::prompt::{self, Builds as PromptBuilds};
use ascribe_check::report_history::ReportHistory;
use ascribe_check::store::Store;
use ascribe_check::{Diagnostic, Project, Reported, Severity};
use ascribe_core::Coded;
use ascribe_query::report::{Options, Ran, Report, Section, Tools};
use clap::{Args as ClapArgs, ValueEnum};

use crate::cli::Global;
use crate::context::{Failure, load_project};
use crate::exit;
use crate::report::FileTable;
use crate::shell::quote;

/// How many items each list of text or Markdown shows, unless `--limit` says
/// otherwise.
const TEXT_LIMIT: usize = 20;
/// How many items each list of JSON holds, unless `--limit` says otherwise.
const JSON_LIMIT: usize = 500;

/// The variable that sets how many seconds `links` waits before asking
/// again about an address that answered with a server error, so a test
/// doesn't wait.
const RETRY_WAIT_VARIABLE: &str = "ASCRIBE_LINKS_RETRY_WAIT";

/// Arguments of `ascribe report`.
#[derive(Debug, ClapArgs)]
pub struct Args {
    /// The sections to report, in any order.
    ///
    /// By default, those that need nothing outside the project: `problems`,
    /// `inventory`, and `builds`.
    #[arg(value_enum, value_name = "SECTION")]
    pub sections: Vec<SectionArg>,

    /// How to show the report.
    #[arg(long, value_enum, default_value_t = Format::Text, value_name = "FORMAT")]
    pub format: Format,

    /// Report only on this build: its problems, what it leaves out, and its
    /// pages.
    ///
    /// Repeat it for several. By default, every build in `ascribe.toml`. An
    /// unknown build name is a usage error, and the message lists the builds.
    #[arg(long, value_name = "NAME")]
    pub build: Vec<String>,

    /// For `pages`: list only the pages that aren't `listed`, those
    /// `unquoted` and `unlisted`, with where each one's listing comes from.
    #[arg(long)]
    pub not_listed: bool,

    /// The address of a built site, for `delivery`: the published site, or a
    /// preview of it.
    #[arg(long, value_name = "URL")]
    pub site: Option<String>,

    /// List at most this many items in each list.
    ///
    /// By default, 20 in text and Markdown and 500 in JSON. What's left out
    /// is counted, with the command that lists it.
    #[arg(long, value_name = "N")]
    pub limit: Option<usize>,

    /// For `links`: ask about every address, reusing no answer from an
    /// earlier run, and keep what this run gets.
    ///
    /// By default, an address that answered within `[checks.links]
    /// reuse-days` isn't asked about again.
    #[arg(long)]
    pub recheck: bool,

    /// Report as a first run would: say nothing about what's new in
    /// `problems`, `links`, and `delivery`, and report every failure of
    /// `links`. `problems` and `delivery` keep nothing for the next report,
    /// and `links` still keeps what this run sees.
    ///
    /// By default, `problems` and `links` say which problems are new since
    /// the last report and how many it listed are gone, `delivery` says which
    /// checks are failing since the last report, and a link that answered
    /// before and fails now is listed as unconfirmed until it has kept
    /// failing for `[checks.links] confirm-days`.
    #[arg(long)]
    pub no_history: bool,

    /// Report the advice `ascribe.baseline` holds back too, without reading
    /// the file.
    ///
    /// By default, `problems` and `inventory` leave out what the baseline
    /// lists while it's advice, as `ascribe check` does, and `problems`
    /// counts it. A baseline that can't be read stops a report with either
    /// section, unless this is given.
    #[arg(long)]
    pub no_baseline: bool,

    /// Exit with 1 when a section finds something at this severity or
    /// above, and with 2 when a section asked for couldn't run: for a
    /// scheduled job that opens an issue.
    ///
    /// `--exit-code` alone is `--exit-code=advice`: any problem fails.
    /// Without it, the report exits with 0 whenever it ran.
    #[arg(
        long,
        value_enum,
        value_name = "LEVEL",
        num_args = 0..=1,
        require_equals = true,
        default_missing_value = "advice"
    )]
    pub exit_code: Option<Level>,
}

/// A section of the report.
#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
pub enum SectionArg {
    /// What `ascribe check` finds, counted by check, by file, and by kind
    /// of next step; and every acknowledgement, with its reason.
    Problems,
    /// Pages by type and by owner; overdue rechecks, orphan pages, and
    /// unused entries.
    Inventory,
    /// What each build leaves out that another keeps.
    Builds,
    /// Every page's record, sorted by address: its title, description,
    /// canonical address, and listing, and where the listing comes from.
    Pages,
    /// External links that fail, redirect, or time out. Needs a link
    /// checker, lychee, and the network.
    Links,
    /// The delivery spec's checks on a built site. Needs `afdocs`, the
    /// network, and `--site`.
    #[value(alias = "agents")]
    Delivery,
}

impl SectionArg {
    fn section(self) -> Section {
        match self {
            SectionArg::Problems => Section::Problems,
            SectionArg::Inventory => Section::Inventory,
            SectionArg::Builds => Section::Builds,
            SectionArg::Pages => Section::Pages,
            SectionArg::Links => Section::Links,
            SectionArg::Delivery => Section::Delivery,
        }
    }
}

/// The output format.
#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
pub enum Format {
    /// Each section, for people.
    #[value(alias = "concise")]
    Text,
    /// Markdown, for a CI job's summary or an issue's body.
    Markdown,
    /// One JSON document, for tools.
    Json,
    /// A prompt for an agent that acts on one section's problems: only
    /// with one section, `problems`, `links`, or `delivery`. Nothing when it
    /// finds nothing.
    Prompt,
}

/// The least severity `--exit-code` fails on.
#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
pub enum Level {
    /// Any problem.
    Advice,
    /// Warnings and errors.
    Warning,
    /// Errors only.
    Error,
}

impl Level {
    fn severity(self) -> Severity {
        match self {
            Level::Advice => Severity::Advice,
            Level::Warning => Severity::Warning,
            Level::Error => Severity::Error,
        }
    }
}

/// Why the report can't be made as asked.
#[derive(Debug, thiserror::Error)]
pub enum ReportError {
    /// `--format prompt` with other than one of the sections it writes.
    #[error(
        "--format prompt writes one section's prompt: name one of `problems`, `links`, or `delivery`"
    )]
    PromptSection,
}

impl Coded for ReportError {
    fn code(&self) -> &'static str {
        match self {
            ReportError::PromptSection => "prompt_needs_one_section",
        }
    }
}

/// Runs the command. Exit codes: 0 when it ran, whatever it found. With
/// `--exit-code`, 1 when a section finds something at its level or above,
/// and 2 when a section asked for couldn't run. 2 when the project can't be
/// loaded, or a build named isn't the content model's.
pub fn run(global: &Global, args: Args) -> ExitCode {
    let stdout = io::stdout();
    let mut out = io::BufWriter::new(stdout.lock());
    let stderr = io::stderr();
    let mut err = stderr.lock();
    let mut tools = Tools::installed();
    if let Some(seconds) = std::env::var(RETRY_WAIT_VARIABLE)
        .ok()
        .and_then(|v| v.trim().parse().ok())
    {
        tools.links_retry_wait = Duration::from_secs(seconds);
    }
    let code = report(global, &args, &tools, &mut out, &mut err);
    let _ = out.flush();
    exit::code(code)
}

/// The sections asked for, each once, in the report's order; those that
/// need nothing outside the project when none is.
pub(crate) fn sections(args: &Args) -> Vec<Section> {
    if args.sections.is_empty() {
        return Section::LOCAL.to_vec();
    }
    let asked: Vec<Section> = args.sections.iter().map(|s| s.section()).collect();
    Section::ALL
        .into_iter()
        .filter(|s| asked.contains(s))
        .collect()
}

pub(crate) fn report(
    global: &Global,
    args: &Args,
    tools: &Tools<'_>,
    out: &mut dyn Write,
    err: &mut dyn Write,
) -> u8 {
    let sections = sections(args);
    if args.format == Format::Prompt
        && !matches!(
            sections.as_slice(),
            [Section::Problems | Section::Links | Section::Delivery]
        )
    {
        return exit::fail(err, &ReportError::PromptSection);
    }
    let project = match load_project(global) {
        Ok(project) => project,
        Err(failure) => {
            let message = failure_message(failure);
            if args.format == Format::Json {
                let _ = json::write(out, &json::failed(&message, &sections));
            }
            let _ = writeln!(err, "error: {message}");
            return exit::FAILURE;
        }
    };
    // The files the report keeps between runs, in the project's folder.
    let store: Arc<dyn Store> = Arc::new(store::DiskStore::new(project.root()));
    // The store of answers `links` reuses, in the project's `.ascribe/`,
    // on the day the binary gives.
    let link_cache = project
        .today()
        .filter(|_| sections.contains(&Section::Links))
        .map(|today| Cache::open(&project, store.clone(), today, args.recheck));
    // What `links` remembers from earlier runs, beside it.
    let link_history = link_cache.as_ref().and(project.today()).map(|today| {
        let confirm_days = project.model().checks.links.confirm_days;
        History::open(store.clone(), today, confirm_days, args.no_history)
    });
    // What the other sections remember from the last report, beside it;
    // with `--no-history`, it's neither read nor written.
    let history = project
        .today()
        .filter(|_| {
            (sections.contains(&Section::Problems) || sections.contains(&Section::Delivery))
                && !args.no_history
        })
        .and_then(|today| {
            let builds = ascribe_check::select_builds(&project, &args.build).ok()?;
            let names: Vec<String> = builds.iter().map(|b| b.name.clone()).collect();
            Some(ReportHistory::open(store.clone(), today, &names))
        });
    let options = Options {
        sections: &sections,
        builds: &args.build,
        baseline: !args.no_baseline,
        not_listed: args.not_listed,
        site: args.site.as_deref(),
        tools,
        link_cache: link_cache.as_ref(),
        link_history: link_history.as_ref(),
        history: history.as_ref(),
    };
    let report = match ascribe_query::report(&project, &options) {
        Ok(report) => report,
        Err(e) => {
            if args.format == Format::Json {
                let _ = json::write(out, &json::failed(&e.to_string(), &sections));
            }
            return exit::fail(err, &e);
        }
    };
    if args.format == Format::Prompt {
        // A prompt about a section that couldn't run would be empty, as one
        // that found nothing is: say why on standard error.
        for (section, why) in report.not_run() {
            let _ = writeln!(
                err,
                "note: `{}` didn't run: {}",
                section.as_str(),
                why.reason
            );
        }
    }
    let written = write(out, global, args, &project, &sections, &report);
    if let Err(e) = written
        && e.kind() != io::ErrorKind::BrokenPipe
    {
        let _ = writeln!(err, "error: can't write the report: {e}");
        return exit::FAILURE;
    }
    exit_code(&report, args.exit_code)
}

/// What a project that can't be loaded says: why, and a content model's
/// errors under it.
fn failure_message(failure: Failure) -> String {
    match failure {
        Failure::Config(e) => e.to_string(),
        Failure::Load(ascribe_workspace::LoadError::Model { issues, .. }) => {
            let mut text = format!(
                "{} has errors, so nothing can be reported",
                ascribe_check::MODEL_FILE
            );
            for d in issues.iter().map(ascribe_check::Diagnostic::from_issue) {
                text.push_str(&format!("\n  {}", d.message));
            }
            text
        }
        Failure::Load(e @ ascribe_workspace::LoadError::Read { .. }) => e.to_string(),
    }
}

/// The exit code for what the report found: with `--exit-code`, 2 when a
/// section couldn't run, else 1 with a problem at its level or above; 0
/// otherwise.
fn exit_code(report: &Report, level: Option<Level>) -> u8 {
    let Some(level) = level else {
        return exit::OK;
    };
    if !report.not_run().is_empty() {
        exit::FAILURE
    } else if report
        .all_problems()
        .any(|d| d.severity >= level.severity())
    {
        exit::PROBLEMS
    } else {
        exit::OK
    }
}

fn write(
    out: &mut dyn Write,
    global: &Global,
    args: &Args,
    project: &Project,
    sections: &[Section],
    report: &Report,
) -> io::Result<()> {
    let files = FileTable::of_project(project);
    let limit = args.limit.unwrap_or(match args.format {
        Format::Json => JSON_LIMIT,
        Format::Text | Format::Markdown | Format::Prompt => TEXT_LIMIT,
    });
    let longest = json::longest(report);
    let next_command = (longest > limit).then(|| command(global, args, sections, longest));
    let list_command = check_command(global, args);
    match args.format {
        Format::Json => json::write(
            out,
            &json::of(
                report,
                &files,
                &json::About {
                    sections,
                    limit,
                    next_command,
                    list_command,
                },
            ),
        ),
        Format::Text | Format::Markdown => text::write(
            out,
            &files,
            report,
            text::Style {
                markdown: args.format == Format::Markdown,
                limit,
            },
            &text::About {
                next_command: next_command.as_deref(),
                list_command: &list_command,
            },
        ),
        Format::Prompt => write_prompt(out, global, args, project, sections, report),
    }
}

/// The words of `--config` and `--build`, as given.
fn options(global: &Global, args: &Args) -> Vec<String> {
    let mut words = Vec::new();
    if let Some(config) = &global.config {
        words.push("--config".to_owned());
        words.push(quote(&config.to_string_lossy()));
    }
    for build in &args.build {
        words.push("--build".to_owned());
        words.push(quote(build));
    }
    words
}

/// This command, listing `limit` items in each list.
fn command(global: &Global, args: &Args, sections: &[Section], limit: usize) -> String {
    let mut words = vec!["ascribe".to_owned(), "report".to_owned()];
    if !args.sections.is_empty() {
        words.extend(sections.iter().map(|s| s.as_str().to_owned()));
    }
    words.extend(options(global, args));
    if args.not_listed {
        words.push("--not-listed".to_owned());
    }
    if let Some(site) = &args.site {
        words.push("--site".to_owned());
        words.push(quote(site));
    }
    if args.no_history {
        // Without it, the lists would differ.
        words.push("--no-history".to_owned());
    }
    if args.no_baseline {
        words.push("--no-baseline".to_owned());
    }
    words.extend(["--limit".to_owned(), limit.to_string()]);
    match args.format {
        Format::Json => words.extend(["--format".to_owned(), "json".to_owned()]),
        Format::Markdown => words.extend(["--format".to_owned(), "markdown".to_owned()]),
        Format::Text | Format::Prompt => {}
    }
    words.join(" ")
}

/// The `ascribe check` command that lists each of `problems`' diagnostics.
fn check_command(global: &Global, args: &Args) -> String {
    let mut words = vec!["ascribe".to_owned(), "check".to_owned()];
    words.extend(options(global, args));
    if args.no_baseline {
        words.push("--no-baseline".to_owned());
    }
    if args.format == Format::Json {
        words.extend(["--format".to_owned(), "json".to_owned()]);
    } else {
        words.extend(["--format".to_owned(), "concise".to_owned()]);
    }
    words.join(" ")
}

/// Writes the prompt about the one section asked for: nothing when it found
/// nothing, or couldn't run.
fn write_prompt(
    out: &mut dyn Write,
    global: &Global,
    args: &Args,
    project: &Project,
    sections: &[Section],
    report: &Report,
) -> io::Result<()> {
    let builds = if args.build.is_empty() {
        PromptBuilds::All
    } else {
        PromptBuilds::Named(args.build.clone())
    };
    let context = prompt::Context::seen_from(
        project.root(),
        std::env::current_dir().ok().as_deref(),
        builds,
    );
    // The command that verifies the work, from the prompt's base.
    let verify = |section: Section| {
        let mut words = vec!["ascribe".to_owned(), "report".to_owned()];
        words.push(section.as_str().to_owned());
        match &context.folder {
            Some(folder) => words.extend(["--config".to_owned(), prompt::shell_word(folder)]),
            None => {
                if let Some(config) = &global.config {
                    words.extend(["--config".to_owned(), quote(&config.to_string_lossy())]);
                }
            }
        }
        if let Some(site) = &args.site {
            words.extend(["--site".to_owned(), quote(site)]);
        }
        words.join(" ")
    };
    let text = match sections {
        [Section::Problems] => report.problems.as_ref().and_then(|p| {
            let reported = Reported::all(p.diagnostics.clone());
            prompt::project(project, &context, &[], &reported)
        }),
        [Section::Links] => match &report.links {
            Some(Ran::Done(links)) => {
                let problems: Vec<&Diagnostic> = links.diagnostics.iter().collect();
                prompt::batch(
                    project,
                    &context,
                    &format!(
                        "Fix the {} `ascribe report links` finds broken or moved.",
                        if problems.len() == 1 {
                            "external link".to_owned()
                        } else {
                            format!("{} external links", problems.len())
                        }
                    ),
                    Vec::new(),
                    &problems,
                    &verify(Section::Links),
                )
            }
            _ => None,
        },
        [Section::Delivery] => match &report.delivery {
            Some(Ran::Done(delivery)) => {
                let problems: Vec<&Diagnostic> = delivery.diagnostics.iter().collect();
                prompt::batch(
                    project,
                    &context,
                    "Change what the delivery spec's checks find on the published site. None of it is in the pages: it's the hosting's settings, or a bug to report in Ascribe.",
                    vec![format!("Site: {}", delivery.site)],
                    &problems,
                    &verify(Section::Delivery),
                )
            }
            _ => None,
        },
        _ => None,
    };
    match text {
        Some(text) => out.write_all(text.as_bytes()),
        None => Ok(()),
    }
}

#[cfg(test)]
mod tests;
