//! `ascribe check`: report every file-level and page-level problem, without
//! building.

use std::io::{self, Write};
use std::process::ExitCode;

use clap::{Args as ClapArgs, ValueEnum};
use tessera_check::LoadError;

use crate::cli::Global;
use crate::commands::diagnose::diagnose;
use crate::context::{Failure, load_project, stdout_is_terminal, use_color};
use crate::exit;
use crate::report::{Counts, FileTable, json, text};

/// Arguments of `ascribe check`.
#[derive(Debug, ClapArgs)]
pub struct Args {
    /// How to show the results.
    #[arg(long, value_enum, default_value_t = Format::Text, value_name = "FORMAT")]
    pub format: Format,

    /// Fail (exit code 1) on warnings too.
    #[arg(long)]
    pub deny_warnings: bool,

    /// Check only this build (repeat for several). By default every build of
    /// the content model is checked, and each problem is reported once,
    /// naming the builds it appears in.
    #[arg(long, value_name = "NAME")]
    pub build: Vec<String>,
}

/// The output format.
#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
pub enum Format {
    /// Diagnostics with source snippets, for people.
    Text,
    /// One JSON document (see the README for the schema), for tools.
    Json,
}

/// Runs the command. Exit codes: 0 with no errors, 1 with errors (or with
/// warnings under `--deny-warnings`), 2 when the project can't be checked.
pub fn run(global: &Global, args: Args) -> ExitCode {
    let stdout = io::stdout();
    let mut out = stdout.lock();
    let stderr = io::stderr();
    let mut err = stderr.lock();
    let code = check(global, &args, &mut out, &mut err);
    let _ = out.flush();
    exit::code(code)
}

fn check(global: &Global, args: &Args, out: &mut dyn Write, err: &mut dyn Write) -> u8 {
    let color = use_color(global, stdout_is_terminal());
    let project = match load_project(global) {
        Ok(project) => project,
        Err(failure) => return report_failure(failure, args, color, out, err),
    };
    let diagnostics = match diagnose(&project, &args.build) {
        Ok((diagnostics, _)) => diagnostics,
        Err(message) => {
            let _ = writeln!(err, "error: {message}");
            return exit::FAILURE;
        }
    };
    let files = FileTable::of_project(&project);
    let checked = project.sources().len();
    let written = match args.format {
        Format::Text => text::write(out, &files, &diagnostics, checked, color),
        Format::Json => json::write(out, &files, &diagnostics, checked, None),
    };
    if let Err(e) = written {
        // A closed pipe (`| head`) isn't a problem with the project.
        if e.kind() != io::ErrorKind::BrokenPipe {
            let _ = writeln!(err, "error: can't write the report: {e}");
            return exit::FAILURE;
        }
    }
    let counts = Counts::of(&diagnostics);
    if counts.errors > 0 || (args.deny_warnings && counts.warnings > 0) {
        exit::PROBLEMS
    } else {
        exit::OK
    }
}

/// Reports a project that couldn't be loaded, and returns the exit code.
///
/// Resolved Q58: a content model with errors is a configuration
/// failure (exit code 2), and its diagnostics are shown.
fn report_failure(
    failure: Failure,
    args: &Args,
    color: bool,
    out: &mut dyn Write,
    err: &mut dyn Write,
) -> u8 {
    let (message, files, diagnostics) = match failure {
        Failure::Config(message) => (message, FileTable::of_model(String::new()), Vec::new()),
        Failure::Load(LoadError::Model { text, diagnostics }) => (
            format!(
                "{} has errors, so nothing can be checked",
                tessera_check::MODEL_FILE
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
            let shown = diagnostics
                .iter()
                .try_for_each(|d| text::write_diagnostic(out, &files, d, color));
            let _ = shown;
            writeln!(err, "error: {message}")
        }
        Format::Json => json::write(out, &files, &diagnostics, 0, Some(&message)),
    };
    exit::FAILURE
}
