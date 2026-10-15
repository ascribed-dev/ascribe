//! `ascribe fmt [paths] [--check]`: rewrites Ascribe constructs into
//! canonical form (SPEC §8.3) with `ascribe-fmt`.
//!
//! - **What it formats.** Every `.md` file under each path (a file, or a
//!   directory searched recursively), or, with no path, under the project's
//!   content root. Directories whose names start with `.`, `node_modules`,
//!   and directories below the starting one that hold an `ascribe.toml`
//!   (another project's folder, unless it's this project's own) are skipped.
//! - **The project.** `--config`, or the nearest `ascribe.toml` at or above the
//!   current directory. Its content model decides which lines are directives.
//! - **Links.** A file in the content root reached through a symbolic link
//!   that leads to a file that isn't a source file of the content root is
//!   left alone and reported on standard error as `check` reports it
//!   (`source-unreadable`, SPEC §2.1).
//! - **Without `--check`** files are rewritten in place, and each file that
//!   changed is listed. **With `--check`** nothing is written, and each file
//!   that would change is listed. **With `--format json`** the list is one
//!   JSON document with each file's edits.
//! - **Exit status.** `0` when nothing needed formatting (`--check`) or every
//!   file was formatted; `1` under `--check` when a file would change; `2` for a
//!   problem: no project, an invalid model, an unreadable path or file, or a
//!   link out of the content root (the other files are still formatted).

use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use ascribe_check::{Diagnostic, LoadError, LocateError, Project};
use ascribe_core::diagnostics::{MODEL_TOML_SYNTAX, SOURCE_UNREADABLE};
use ascribe_core::path::{normalize, relative_path};
use ascribe_core::{Coded, FileId, Issue, LineIndex, Location, Span, TextEdit};
use ascribe_fmt::{FormatFilesError, Formatted};
use ascribe_resolve::{DiskFs, Layout};
use clap::Args as ClapArgs;
use serde::Serialize;

use crate::answer::{self, Format};
use crate::cli::Global;
use crate::context::locate;
use crate::exit;
use crate::report::json::Edit;

/// The version of `--format json`'s schema.
pub const SCHEMA_VERSION: u32 = 1;

/// Arguments of `ascribe fmt`.
#[derive(Debug, Default, ClapArgs)]
pub struct Args {
    /// Change nothing, and list each file that would change.
    ///
    /// Exits with 1 if any would. Use it in CI.
    #[arg(long)]
    pub check: bool,

    /// Files and directories to format: the `.md` files among them.
    ///
    /// By default, every `.md` file under the content root.
    ///
    /// Directories whose names start with `.`, `node_modules`, and
    /// directories inside the searched ones that hold an `ascribe.toml` other
    /// than the project's own (another project, formatted under its own
    /// model) are skipped. So is a file in the content root reached through a
    /// symbolic link that leads to a file that isn't a source file of the
    /// content root, and it's reported.
    pub paths: Vec<PathBuf>,

    /// How to show what changed.
    ///
    /// `json` lists each file with the edits that format it; with
    /// `--check`, that's the edits without writing them.
    #[arg(long, value_enum, default_value_t = Format::Text, value_name = "FORMAT")]
    pub format: Format,
}

/// What `ascribe fmt --format json` writes: one document. Fields can be
/// added without a new `schema_version`, so a reader ignores fields it
/// doesn't know.
#[derive(Serialize)]
#[cfg_attr(test, derive(schemars::JsonSchema))]
pub(crate) struct FmtReport {
    /// The version of this schema. It changes only when a field is removed
    /// or changes meaning.
    schema_version: u32,
    /// The version of Ascribe that wrote it.
    ascribe_version: &'static str,
    /// Whether the files were rewritten: `false` with `--check`, which
    /// writes nothing.
    written: bool,
    /// Each file that changed, or with `--check` would change, in path
    /// order, with the edits that format it.
    files: Vec<FmtFile>,
    /// The files left alone because a symbolic link on the way to them
    /// leads to a file that isn't a source file of the content root.
    refused: Vec<RefusedFile>,
}

/// A file and the edits that format it.
#[derive(Serialize)]
#[cfg_attr(test, derive(schemars::JsonSchema))]
struct FmtFile {
    /// The file, relative to the project root (the directory of
    /// `ascribe.toml`), with `/` separators; a file outside it as it was
    /// found.
    file: String,
    /// The edits, each replacing the text of its range in the file as it
    /// was before formatting. They don't overlap, and are in file order.
    edits: Vec<Edit>,
}

/// A file left alone.
#[derive(Serialize)]
#[cfg_attr(test, derive(schemars::JsonSchema))]
struct RefusedFile {
    /// The file, as a formatted file's `file` is.
    file: String,
    /// Why, as `ascribe check` words it.
    reason: String,
}

/// Runs the command.
pub fn run(global: &Global, args: Args) -> ExitCode {
    let mut out = std::io::stdout().lock();
    let mut err = std::io::stderr().lock();
    let formatted = match args.format {
        Format::Text => {
            let verb = if args.check {
                "would reformat"
            } else {
                "formatted"
            };
            format_all(global, &args, &mut |file, _, _| {
                let _ = writeln!(out, "{verb} {}", file.display());
            })
        }
        Format::Json => answer(global, &args).and_then(|(done, report)| {
            match answer::written(answer::write_json(&mut out, &report), &mut err) {
                Some(code) => Err(FmtError::Written(code)),
                None => Ok(done),
            }
        }),
    };
    let done = match formatted {
        Ok(done) => done,
        Err(FmtError::Written(code)) => return exit::code(code),
        Err(e) => return exit::code(exit::fail(&mut err, &e)),
    };
    for refused in &done.refused {
        let issue = Issue::new(
            SOURCE_UNREADABLE,
            Location::new(FileId::new(0), Span::empty(0)),
        )
        .with_arg("reason", refused.reason.clone());
        let d = Diagnostic::from_issue(&issue);
        let _ = writeln!(
            err,
            "{}: [{}] Error: {}",
            refused.path.display(),
            d.code,
            d.message
        );
    }
    let changed = done.changed.len();
    if args.check && changed > 0 {
        let _ = writeln!(
            err,
            "{changed} file{} would be reformatted",
            if changed == 1 { "" } else { "s" }
        );
    }
    if !done.refused.is_empty() {
        exit::code(exit::FAILURE)
    } else if args.check && changed > 0 {
        exit::code(exit::PROBLEMS)
    } else {
        exit::code(exit::OK)
    }
}

/// Why `ascribe fmt` couldn't format.
#[derive(Debug, thiserror::Error)]
pub enum FmtError {
    /// No content model was found.
    #[error(transparent)]
    Locate(#[from] LocateError),
    /// The content model can't be used, worded for `ascribe fmt`: the pages
    /// aren't read, so a project whose pages have errors can still be
    /// formatted.
    #[error(
        "{} isn't a valid content model ({}); run `ascribe check` for details",
        config.display(),
        model_slugs(error).join(", ")
    )]
    Model {
        /// The content model.
        config: PathBuf,
        /// Why it can't be used.
        error: LoadError,
    },
    /// A file couldn't be read, written, or formatted.
    #[error(transparent)]
    Format(#[from] FormatFilesError),
    /// The report couldn't be written; it's been reported, with this exit
    /// code.
    #[error("the report couldn't be written")]
    Written(u8),
}

impl Coded for FmtError {
    fn code(&self) -> &'static str {
        match self {
            FmtError::Locate(e) => e.code(),
            FmtError::Model { error, .. } => error.code(),
            FmtError::Format(e) => e.code(),
            FmtError::Written(_) => "report_unwritable",
        }
    }
}

/// The slugs of the content model's problems.
fn model_slugs(error: &LoadError) -> Vec<&str> {
    match error {
        LoadError::Model { diagnostics, .. } => {
            diagnostics.iter().map(|d| d.slug.as_str()).collect()
        }
        // The file couldn't be read, or isn't UTF-8: the content model's
        // syntax error.
        LoadError::Read { .. } => vec![MODEL_TOML_SYNTAX.as_str()],
    }
}

/// Formats every file, listing each that changed (or would have), and
/// returns what it did.
fn format_all(
    global: &Global,
    options: &Args,
    on_changed: &mut dyn FnMut(&Path, &str, &[TextEdit]),
) -> Result<Formatted, FmtError> {
    let config = locate(global)?;
    let file = Project::load_model(&config).map_err(|error| FmtError::Model {
        config: config.clone(),
        error,
    })?;
    let model = file.model;
    let boundary = DiskFs::new(file.root, &Layout::from_model(&model));
    Ok(ascribe_fmt::format_files(
        &config,
        &model,
        &options.paths,
        options.check,
        &boundary,
        on_changed,
    )?)
}

/// Formats every file, as [`format_all`] does, and returns what
/// `--format json` writes about it: each file with its edits.
///
/// # Errors
///
/// As [`format_all`].
pub(crate) fn answer(global: &Global, options: &Args) -> Result<(Formatted, FmtReport), FmtError> {
    let config = locate(global)?;
    let root = normalize(&std::path::absolute(&config).unwrap_or(config.clone()));
    let root = root.parent().map(Path::to_owned).unwrap_or_default();
    let shown = |path: &Path| {
        let path = normalize(&std::path::absolute(path).unwrap_or(path.to_owned()));
        relative_path(&root, &path)
            .filter(ascribe_core::RelPath::is_inside)
            .map_or_else(|| path.display().to_string(), |rel| rel.to_string())
    };
    let mut files = Vec::new();
    let done = format_all(global, options, &mut |path, source, edits| {
        let index = LineIndex::new(source);
        files.push(FmtFile {
            file: shown(path),
            edits: edits.iter().map(|e| Edit::in_text(&index, e)).collect(),
        });
    })?;
    let refused = done
        .refused
        .iter()
        .map(|r| RefusedFile {
            file: shown(&r.path),
            reason: r.reason.clone(),
        })
        .collect();
    let report = FmtReport {
        schema_version: SCHEMA_VERSION,
        ascribe_version: env!("CARGO_PKG_VERSION"),
        written: !options.check,
        files,
        refused,
    };
    Ok((done, report))
}
