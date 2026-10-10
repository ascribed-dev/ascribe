//! `ascribe build`: check the project, then write each build's outputs.
//!
//! The checks run first, for every build asked for, and print exactly what
//! `ascribe check` prints (the same report code on the same diagnostics). A
//! build with errors writes nothing (SPEC §8.2: "a build MUST fail on
//! errors"). Otherwise each build is resolved once and each output is
//! written from that resolved tree, into a staging
//! directory that replaces the previous output only on success.

use std::io::{self, Write};
use std::process::ExitCode;

use ascribe_check::{Diagnosed, LoadError, Project, Reported, Severity, diagnose};
use ascribe_emit::{EmitError, Output, WriteEvent, WriteOptions};
use ascribe_model::Build;
use clap::{Args as ClapArgs, ValueEnum};

use crate::cli::Global;
use crate::context::{Failure, load_project, stdout_is_terminal, use_color};
use crate::exit;
use crate::report::json::About;
use crate::report::{Counts, FileTable, json, text};

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
    let project = match load_project(global) {
        Ok(project) => project,
        Err(failure) => return report_failure(failure, args, color, out, err),
    };
    // The checks, for every build asked for, before anything is written.
    let Diagnosed {
        mut diagnostics,
        builds,
        acknowledged,
    } = match diagnose(&project, &args.build) {
        Ok(found) => found,
        Err(e) => return exit::fail(err, &e),
    };
    // Advice after errors and warnings, as `ascribe check` lists it.
    diagnostics.sort_by_key(|d| d.severity == Severity::Advice);
    let files = FileTable::of_project(&project);
    let checked = project.sources().len();
    let written = match args.format {
        Format::Text => text::write(
            out,
            &files,
            &diagnostics,
            checked,
            acknowledged.len(),
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
                builds_checked: builds.iter().map(|b| b.name.clone()).collect(),
                acknowledged: &acknowledged,
                summary_only: None,
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
    let mut report = |event: &WriteEvent| {
        let _ = match event {
            WriteEvent::Warning(warning) => writeln!(err, "warning: {warning}"),
            WriteEvent::Written(w) => writeln!(
                err,
                "built {}/{}: {} page{}, {} asset{}{}",
                w.build,
                w.output,
                w.pages,
                plural(w.pages),
                w.assets,
                plural(w.assets),
                if w.removed > 0 {
                    format!(", removed {} stale file{}", w.removed, plural(w.removed))
                } else {
                    String::new()
                }
            ),
        };
    };
    ascribe_emit::write_outputs(&index, project.root(), builds, &options, &mut report).map(|_| ())
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
