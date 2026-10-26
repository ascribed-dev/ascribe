//! `ascribe build`: check the project, then write each build's outputs.
//!
//! The checks run first, for every build asked for, on the diagnostics
//! `ascribe check` reports. The text shows each error in full, since errors
//! stop the build, and one summary line that points to `ascribe check` for
//! the warnings and advice; the JSON is exactly `ascribe check`'s. The
//! advice `ascribe.baseline` lists is held back and counted, as `ascribe
//! check` holds it back, unless `--no-baseline` says not to. A build
//! with errors writes nothing (SPEC §8.2: "a build MUST fail on
//! errors"). Otherwise each build is resolved once and each output is
//! written from that resolved tree, into a staging
//! directory that replaces the previous output only on success.

use std::io::{self, Write};
use std::process::ExitCode;

use ascribe_check::{Checked, LoadError, Project, Reported, Request, Severity, diagnose_with};
use ascribe_emit::{EmitError, Output, WriteEvent, WriteOptions, Written};
use ascribe_model::Build;
use clap::{Args as ClapArgs, ValueEnum};

use crate::cli::Global;
use crate::context::{Failure, load_project, stdout_is_terminal, use_color};
use crate::exit;
use crate::report::json::About;
use crate::report::{Counts, FileTable, json, text};
use crate::shell::quote;

/// Arguments of `ascribe build`.
#[derive(Debug, ClapArgs)]
pub struct Args {
    /// Build only this build.
    ///
    /// Repeat it for several. By default, every build in `ascribe.toml`.
    #[arg(long, value_name = "NAME")]
    pub build: Vec<String>,

    /// Which outputs to write, separated by commas.
    #[arg(
        long,
        value_enum,
        value_delimiter = ',',
        default_values_t = [Emit::Site, Emit::Plain, Emit::Json],
        value_name = "OUTPUTS"
    )]
    pub emit: Vec<Emit>,

    /// How to show the checks' results, as for `ascribe check`.
    #[arg(long, value_enum, default_value_t = Format::Text, value_name = "FORMAT")]
    pub format: Format,

    /// Mark each block of the site output with the source file and lines it
    /// came from, for review.
    ///
    /// Each Markdown block gets an `<!--ascribe-anchor …-->` comment before
    /// it, and each element Ascribe writes gets `data-ascribe-source` (with
    /// `data-ascribe-via` for a block from a fragment). The site output's
    /// manifest records `"anchors": true`. Without it, the output has no
    /// anchors. The other outputs are the same either way.
    #[arg(long)]
    pub anchors: bool,

    /// Report the advice `ascribe.baseline` holds back too, without reading
    /// the file.
    ///
    /// By default, the checks hold back what the baseline lists while it's
    /// advice, and count it, as `ascribe check` does. A baseline that can't
    /// be read stops the build, unless this is given.
    #[arg(long)]
    pub no_baseline: bool,
}

/// An output.
#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
pub enum Emit {
    /// Markdown plus web components, for an Astro site.
    Site,
    /// Fully resolved CommonMark with no HTML.
    Plain,
    /// The resolved tree as JSON.
    Json,
}

/// How to show the checks' results.
#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
pub enum Format {
    /// Diagnostics with source snippets, for people.
    Text,
    /// One JSON document, for tools.
    Json,
}

/// Runs the command. Exit codes: 0 on success, 1 when the checks found errors
/// (nothing is written), 2 when it couldn't do its work: a usage error, no
/// `ascribe.toml`, a content model with errors, or an output that couldn't be
/// written.
pub fn run(global: &Global, args: Args) -> ExitCode {
    // Buffered: a report is many small writes, and a locked stdout flushes
    // at every line.
    let stdout = io::stdout();
    let mut out = io::BufWriter::new(stdout.lock());
    let stderr = io::stderr();
    let mut err = stderr.lock();
    let code = build(global, &args, &mut out, &mut err);
    let _ = out.flush();
    exit::code(code)
}

fn build(global: &Global, args: &Args, out: &mut dyn Write, err: &mut dyn Write) -> u8 {
    let color = use_color(global, stdout_is_terminal());
    let mut project = match load_project(global) {
        Ok(project) => project,
        Err(failure) => return report_failure(failure, args, color, out, err),
    };
    // The checks, for every build asked for, before anything is written.
    let request = Request {
        builds: &args.build,
        baseline: !args.no_baseline,
        ..Request::default()
    };
    let Checked {
        reported,
        builds: names,
        acknowledged,
        baselined,
    } = match diagnose_with(&project, &request) {
        Ok(found) => found,
        Err(e) => return exit::fail(err, &e),
    };
    let mut diagnostics: Vec<_> = reported.into_iter().map(|r| r.diagnostic).collect();
    // Advice after errors and warnings, as `ascribe check` lists it.
    diagnostics.sort_by_key(|d| d.severity == Severity::Advice);
    let files = FileTable::of_project(&project);
    let checked = project.sources().len();
    let covered = if args.build.is_empty() {
        text::Covered::Every
    } else {
        text::Covered::Named(&names)
    };
    let written = match args.format {
        Format::Text => text::write_build(
            out,
            &files,
            &diagnostics,
            checked,
            &covered,
            acknowledged.len(),
            baselined.len(),
            &check_command(global, args),
            color,
        ),
        Format::Json => json::write(
            out,
            &files,
            &Reported::all(diagnostics.clone()),
            &About {
                error: None,
                files_checked: checked,
                files_reported: checked,
                builds_checked: names.clone(),
                acknowledged: &acknowledged,
                baselined: baselined.len(),
                summary_only: None,
                fixed: &[],
            },
        ),
    };
    if let Err(e) = written
        && e.kind() != io::ErrorKind::BrokenPipe
    {
        return fail(err, &format!("can't write the report: {e}"));
    }
    if Counts::of(&diagnostics).errors > 0 {
        let _ = writeln!(err, "error: the build failed; nothing was written");
        return exit::PROBLEMS;
    }

    // The outputs are written from an index of their own, so the one the
    // checks kept goes first.
    let builds: Vec<Build> = names
        .iter()
        .filter_map(|name| project.model().build(name))
        .cloned()
        .collect();
    project.release_link_index();
    let builds: Vec<&Build> = builds.iter().collect();
    match write_outputs(&project, &builds, &args.emit, args.anchors, err) {
        Ok(()) => exit::OK,
        Err(e) => exit::fail(err, &e),
    }
}

/// Resolves each build and writes its outputs, reporting each as it's
/// written.
fn write_outputs(
    project: &Project,
    builds: &[&Build],
    emit_names: &[Emit],
    anchors: bool,
    err: &mut dyn Write,
) -> Result<(), EmitError> {
    let index = project.index();
    let options = WriteOptions {
        outputs: emit_names
            .iter()
            .map(|e| match e {
                Emit::Site => Output::Site,
                Emit::Plain => Output::Plain,
                Emit::Json => Output::Json,
            })
            .collect(),
        anchors,
    };
    // One line per build, so a build's outputs are held until the next
    // build's first event, or the end.
    let mut pending: Vec<Written> = Vec::new();
    let mut report = |event: &WriteEvent| match event {
        WriteEvent::Warning(warning) => {
            flush(&mut pending, err);
            let _ = writeln!(err, "warning: {warning}");
        }
        WriteEvent::Written(w) => {
            if pending.first().is_some_and(|p| p.build != w.build) {
                flush(&mut pending, err);
            }
            pending.push(w.clone());
        }
    };
    let result = ascribe_emit::write_outputs(&index, project.root(), builds, &options, &mut report)
        .map(|_| ());
    flush(&mut pending, err);
    result
}

/// Writes one build's line and empties `pending`, its outputs: `built site:
/// 6 pages, 2 assets (site, plain, json)`. Outputs whose counts differ are
/// listed apart, `6 pages, 2 assets (site, plain); 6 pages, 0 assets
/// (json)`, and the stale files removed are added up.
fn flush(pending: &mut Vec<Written>, err: &mut dyn Write) {
    let Some(first) = pending.first() else {
        return;
    };
    let mut groups: Vec<(usize, usize, Vec<&str>)> = Vec::new();
    for w in pending.iter() {
        match groups
            .iter_mut()
            .find(|(pages, assets, _)| *pages == w.pages && *assets == w.assets)
        {
            Some((_, _, outputs)) => outputs.push(w.output),
            None => groups.push((w.pages, w.assets, vec![w.output])),
        }
    }
    let counts: Vec<String> = groups
        .iter()
        .map(|(pages, assets, outputs)| {
            format!(
                "{pages} page{}, {assets} asset{} ({})",
                plural(*pages),
                plural(*assets),
                outputs.join(", ")
            )
        })
        .collect();
    let removed: usize = pending.iter().map(|w| w.removed).sum();
    let _ = writeln!(
        err,
        "built {}: {}{}",
        first.build,
        counts.join("; "),
        if removed > 0 {
            format!(", removed {removed} stale file{}", plural(removed))
        } else {
            String::new()
        }
    );
    pending.clear();
}

/// The `ascribe check` command that lists each of the checks' diagnostics:
/// this build's `--config` and `--build`, as given.
fn check_command(global: &Global, args: &Args) -> String {
    let mut words = vec!["ascribe".to_owned(), "check".to_owned()];
    if let Some(config) = &global.config {
        words.push("--config".to_owned());
        words.push(quote(&config.to_string_lossy()));
    }
    for build in &args.build {
        words.push("--build".to_owned());
        words.push(quote(build));
    }
    if args.no_baseline {
        words.push("--no-baseline".to_owned());
    }
    words.join(" ")
}

fn plural(n: usize) -> &'static str {
    if n == 1 { "" } else { "s" }
}

fn fail(err: &mut dyn Write, message: &str) -> u8 {
    let _ = writeln!(err, "error: {message}");
    exit::FAILURE
}

/// Reports a project that couldn't be loaded, as `ascribe check` does:
/// a content model with errors is a configuration failure
/// (exit code 2), and its diagnostics are shown.
fn report_failure(
    failure: Failure,
    args: &Args,
    color: bool,
    out: &mut dyn Write,
    err: &mut dyn Write,
) -> u8 {
    let (message, files, diagnostics) = match failure {
        Failure::Config(e) => (
            e.to_string(),
            FileTable::of_model(String::new()),
            Vec::new(),
        ),
        Failure::Load(LoadError::Model { text, diagnostics }) => (
            format!(
                "{} has errors, so nothing can be built",
                ascribe_check::MODEL_FILE
            ),
            FileTable::of_model(text),
            diagnostics,
        ),
        Failure::Load(e @ LoadError::Read { .. }) => (
            e.to_string(),
            FileTable::of_model(String::new()),
            Vec::new(),
        ),
    };
    let _ = match args.format {
        Format::Text => {
            let _ = text::write_diagnostics(out, &files, &diagnostics, color);
            writeln!(err, "error: {message}")
        }
        Format::Json => json::write(
            out,
            &files,
            &Reported::all(diagnostics),
            &About::failed(&message),
        ),
    };
    exit::FAILURE
}
