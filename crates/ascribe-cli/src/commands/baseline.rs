//! `ascribe baseline`: the project's baseline, `ascribe.baseline`, which
//! lists the advice `ascribe check` holds back for now. `write` adds every
//! current advice, and `prune` removes the entries that match nothing
//! (decision #789).

use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use ascribe_check::baseline::{self, BASELINE_FILE, Baseline, Pruned, Written};
use ascribe_check::{Diagnostic, Project};
use ascribe_core::Coded;
use clap::{Args as ClapArgs, Subcommand};
use serde::Serialize;

use crate::answer::{self, Format, FromDisk};
use crate::cli::Global;
use crate::commands::check;
use crate::exit;

/// Arguments of `ascribe baseline`.
#[derive(Debug, ClapArgs)]
pub struct Args {
    #[command(subcommand)]
    pub command: Command,
}

/// The subcommands of `ascribe baseline`.
#[derive(Debug, Subcommand)]
pub enum Command {
    /// Add every advice `ascribe check` reports to ascribe.baseline, making
    /// it when there's none, so the check holds them back from then on.
    /// Warnings and errors are never added.
    #[command(after_help = docs_link!("cli-baseline-write"))]
    Write(WriteArgs),
    /// Remove the entries of ascribe.baseline that match nothing the
    /// project has any more: problems fixed, or whose text changed.
    #[command(after_help = docs_link!("cli-baseline-prune"))]
    Prune(PruneArgs),
}

/// Arguments of `ascribe baseline write`.
#[derive(Debug, ClapArgs)]
pub struct WriteArgs {
    /// How to show what was written.
    #[arg(long, value_enum, default_value_t = Format::Text, value_name = "FORMAT")]
    pub format: Format,
}

/// Arguments of `ascribe baseline prune`.
#[derive(Debug, ClapArgs)]
pub struct PruneArgs {
    /// How to show what was removed.
    #[arg(long, value_enum, default_value_t = Format::Text, value_name = "FORMAT")]
    pub format: Format,
}

/// Why a baseline command couldn't do its work.
#[derive(Debug, thiserror::Error)]
pub enum BaselineCommandError {
    /// `ascribe.baseline` can't be written.
    #[error("can't write {BASELINE_FILE}: {0}")]
    Unwritable(String),
}

impl Coded for BaselineCommandError {
    fn code(&self) -> &'static str {
        match self {
            BaselineCommandError::Unwritable(_) => "baseline_unwritable",
        }
    }
}

/// What `ascribe baseline write --format json` writes.
#[derive(Debug, Serialize)]
#[cfg_attr(test, derive(schemars::JsonSchema))]
pub(crate) struct WriteJson {
    /// The version of this schema. It changes only when a field is removed
    /// or changes meaning.
    schema_version: u32,
    /// The version of Ascribe that wrote it.
    ascribe_version: &'static str,
    /// The file written, from the project's folder: `ascribe.baseline`.
    file: &'static str,
    /// Whether the project had no baseline, so this made one.
    created: bool,
    /// Whether the file changed: it's new, or its text differs.
    changed: bool,
    /// How many problems the baseline holds, after this.
    entries: usize,
    /// How many files they're in.
    files: usize,
    /// How many of them this added.
    added: usize,
    /// How many errors and warnings it left out, which a baseline never
    /// holds.
    skipped: Skipped,
}

/// The errors and warnings `ascribe baseline write` left out.
#[derive(Debug, Serialize)]
#[cfg_attr(test, derive(schemars::JsonSchema))]
struct Skipped {
    /// How many errors.
    errors: usize,
    /// How many warnings.
    warnings: usize,
}

/// What `ascribe baseline prune --format json` writes.
#[derive(Debug, Serialize)]
#[cfg_attr(test, derive(schemars::JsonSchema))]
pub(crate) struct PruneJson {
    /// The version of this schema. It changes only when a field is removed
    /// or changes meaning.
    schema_version: u32,
    /// The version of Ascribe that wrote it.
    ascribe_version: &'static str,
    /// The file pruned, from the project's folder: `ascribe.baseline`.
    file: &'static str,
    /// Whether the project has a baseline. Without one, there's nothing to
    /// prune, and nothing is written.
    found: bool,
    /// Whether the file changed: some entry was removed.
    changed: bool,
    /// How many problems the baseline holds, after this.
    entries: usize,
    /// How many files they're in.
    files: usize,
    /// How many entries it removed.
    removed: usize,
}

/// The version of both documents' schema.
const SCHEMA_VERSION: u32 = 1;

/// Runs the command.
pub fn run(global: &Global, args: Args) -> ExitCode {
    let stdout = io::stdout();
    let mut out = io::BufWriter::new(stdout.lock());
    let stderr = io::stderr();
    let mut err = stderr.lock();
    let code = match args.command {
        Command::Write(args) => run_write(global, &args, &mut out, &mut err),
        Command::Prune(args) => run_prune(global, &args, &mut out, &mut err),
    };
    let _ = out.flush();
    exit::code(code)
}

/// The whole project's diagnostics without the baseline, as `ascribe check`
/// finds them for every build, and the project; or the exit code of a check
/// that couldn't be made, once it's reported.
fn checked(global: &Global, err: &mut dyn Write) -> Result<(check::Outcome, Vec<Diagnostic>), u8> {
    let args = check::Args {
        paths: Vec::new(),
        stdin: false,
        path: None,
        build: Vec::new(),
        editor_build: false,
        code: Vec::new(),
        summary: false,
        format: check::Format::Text,
        limit: None,
        deny_warnings: false,
        fix: false,
        vale: false,
        no_baseline: true,
        vale_timeout: None,
        vale_runs: check::ValeRuns::Any,
    };
    match check::run_check(&FromDisk, global, &args, &mut io::empty()) {
        Ok(outcome) => {
            let diagnostics = outcome
                .parts()
                .0
                .iter()
                .map(|r| r.diagnostic.clone())
                .collect();
            Ok((outcome, diagnostics))
        }
        Err(stopped) => Err(check::report_stopped(
            stopped,
            &args,
            false,
            &mut io::sink(),
            err,
        )),
    }
}

/// The project's baseline, when it has one; or the exit code once a
/// baseline that can't be read is reported.
fn existing(project: &Project, err: &mut dyn Write) -> Result<Option<Baseline>, u8> {
    match project.baseline() {
        None => Ok(None),
        Some(Ok(baseline)) => Ok(Some(baseline.clone())),
        Some(Err(e)) => Err(exit::fail(err, e)),
    }
}

/// Exit codes: 0 when it ran, whether or not the file changed; 2 when the
/// project can't be checked, or the baseline can't be read or written.
fn run_write(global: &Global, args: &WriteArgs, out: &mut dyn Write, err: &mut dyn Write) -> u8 {
    let (outcome, diagnostics) = match checked(global, err) {
        Ok(found) => found,
        Err(code) => return code,
    };
    let project = outcome.project();
    let existing = match existing(project, err) {
        Ok(existing) => existing,
        Err(code) => return code,
    };
    let written = baseline::write(project, existing.as_ref(), &diagnostics);
    let changed = project.baseline_text() != Some(written.text.as_str());
    if changed && let Err(e) = write_file(project.root(), &written.text) {
        return answer::fail(err, &e);
    }
    let result = match args.format {
        Format::Json => answer::write_json(
            out,
            &WriteJson {
                schema_version: SCHEMA_VERSION,
                ascribe_version: env!("CARGO_PKG_VERSION"),
                file: BASELINE_FILE,
                created: existing.is_none(),
                changed,
                entries: written.entries,
                files: written.files,
                added: written.added,
                skipped: Skipped {
                    errors: written.skipped_errors,
                    warnings: written.skipped_warnings,
                },
            },
        ),
        Format::Text => write_written(out, &written, changed),
    };
    answer::written(result, err).unwrap_or(exit::OK)
}

fn write_written(out: &mut dyn Write, w: &Written, changed: bool) -> io::Result<()> {
    let held = format!(
        "{} in {}",
        plural(w.entries, "problem"),
        plural(w.files, "file")
    );
    if changed {
        writeln!(
            out,
            "wrote {BASELINE_FILE}: {held}, {} added; commit it with the docs",
            w.added
        )?;
    } else {
        writeln!(out, "{BASELINE_FILE} is up to date: {held}")?;
    }
    let skipped = match (w.skipped_errors, w.skipped_warnings) {
        (0, 0) => return Ok(()),
        (0, warnings) => plural(warnings, "warning"),
        (errors, 0) => plural(errors, "error"),
        (errors, warnings) => format!(
            "{} and {}",
            plural(errors, "error"),
            plural(warnings, "warning")
        ),
    };
    writeln!(
        out,
        "skipped {skipped}, since a baseline holds advice only; to put one off, lower its check to \"advice\" in [checks] in ascribe.toml, then run `ascribe baseline write` again"
    )
}

/// Exit codes: 0 when it ran, whether or not the file changed; 2 when the
/// project can't be checked, or the baseline can't be read or written.
fn run_prune(global: &Global, args: &PruneArgs, out: &mut dyn Write, err: &mut dyn Write) -> u8 {
    let (outcome, diagnostics) = match checked(global, err) {
        Ok(found) => found,
        Err(code) => return code,
    };
    let project = outcome.project();
    let existing = match existing(project, err) {
        Ok(existing) => existing,
        Err(code) => return code,
    };
    let pruned = existing
        .as_ref()
        .map(|b| baseline::prune(project, b, &diagnostics));
    let changed = pruned.as_ref().is_some_and(|p| p.removed > 0);
    if let Some(pruned) = pruned.as_ref().filter(|_| changed)
        && let Err(e) = write_file(project.root(), &pruned.text)
    {
        return answer::fail(err, &e);
    }
    let result = match args.format {
        Format::Json => answer::write_json(
            out,
            &PruneJson {
                schema_version: SCHEMA_VERSION,
                ascribe_version: env!("CARGO_PKG_VERSION"),
                file: BASELINE_FILE,
                found: pruned.is_some(),
                changed,
                entries: pruned.as_ref().map_or(0, |p| p.entries),
                files: pruned.as_ref().map_or(0, |p| p.files),
                removed: pruned.as_ref().map_or(0, |p| p.removed),
            },
        ),
        Format::Text => write_pruned(out, pruned.as_ref()),
    };
    answer::written(result, err).unwrap_or(exit::OK)
}

fn write_pruned(out: &mut dyn Write, pruned: Option<&Pruned>) -> io::Result<()> {
    let Some(p) = pruned else {
        return writeln!(
            out,
            "there's no {BASELINE_FILE} to prune; `ascribe baseline write` makes one"
        );
    };
    let held = format!(
        "{} in {}",
        plural(p.entries, "problem"),
        plural(p.files, "file")
    );
    if p.removed == 0 {
        writeln!(
            out,
            "{BASELINE_FILE} is up to date: every entry matches a problem; {held}"
        )
    } else {
        writeln!(
            out,
            "removed {} that matched nothing from {BASELINE_FILE}: {held} left; commit it with the docs",
            plural(p.removed, "entry")
        )
    }
}

/// `n` and `word`, with an `s` (or `ies`) when `n` isn't 1.
fn plural(n: usize, word: &str) -> String {
    match (n, word.strip_suffix('y')) {
        (1, _) => format!("1 {word}"),
        (_, Some(stem)) => format!("{n} {stem}ies"),
        _ => format!("{n} {word}s"),
    }
}

/// Writes `ascribe.baseline` in the project's folder.
fn write_file(root: &Path, text: &str) -> Result<(), BaselineCommandError> {
    let path: PathBuf = root.join(BASELINE_FILE);
    // Outside FileSystem: writing the project's own file beside
    // ascribe.toml, as `ascribe permalinks sync` writes its list.
    if ascribe_resolve::is_link(&path) {
        return Err(BaselineCommandError::Unwritable(
            "it's a link; replace it with a file, or remove it".to_owned(),
        ));
    }
    std::fs::write(&path, text).map_err(|e| BaselineCommandError::Unwritable(e.to_string()))
}
