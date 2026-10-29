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
//!   JSON document with each file's edits. **With `--format github`**, only
//!   with `--check`, each file that would change is a GitHub Actions
//!   workflow command, with its path from the repository's root.
//! - **Exit status.** `0` when nothing needed formatting (`--check`) or every
//!   file was formatted; `1` under `--check` when a file would change; `2` for a
//!   problem: no project, an invalid model, an unreadable path or file, or a
//!   link out of the content root (the other files are still formatted).

use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use ascribe_check::Diagnostic;
use ascribe_core::diagnostics::{MODEL_TOML_SYNTAX, SOURCE_UNREADABLE};
use ascribe_core::path::{normalize, relative_path};
use ascribe_core::{Coded, FileId, Issue, LineIndex, Location, Span, TextEdit};
use ascribe_fmt::{FormatFilesError, Formatted};
use ascribe_resolve::{DiskFs, Layout};
use ascribe_workspace::{LoadError, LocateError};
use clap::{Args as ClapArgs, ValueEnum};
use serde::Serialize;

use crate::answer;
use crate::cli::Global;
use crate::context::locate;
use crate::exit;
use crate::guard;
use crate::report::github;
use crate::report::json::Edit;

mod changes;

/// The version of `--format json`'s schema.
pub const SCHEMA_VERSION: u32 = 1;

/// Arguments of `ascribe fmt`.
#[derive(Debug, Default, ClapArgs)]
pub struct Args {
    /// Change nothing, and show what would change as a unified diff, which
    /// `git apply` applies.
    ///
    /// Exits with 1 if any file would change. Use it in CI.
    #[arg(long)]
    pub check: bool,

    /// `--check`'s old spelling, kept for a release.
    #[arg(long, hide = true, conflicts_with = "format")]
    pub diff: bool,

    /// `--dry-run`, which lists each change it would make, the file and
    /// line and the lines as they are and as they'd be, and `--force`.
    #[command(flatten)]
    pub guard: guard::Options,

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
    /// `--check`, that's the edits without writing them. `github`, only with
    /// `--check`, is for GitHub Actions.
    #[arg(
        long,
        value_enum,
        default_value_t = Format::Text,
        value_name = "FORMAT",
        requires_if("github", "check")
    )]
    pub format: Format,
}

/// How `ascribe fmt` shows what changed.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, ValueEnum)]
pub enum Format {
    /// Short text, for people and agents to read.
    #[default]
    #[value(alias = "concise")]
    Text,
    /// One JSON document, for tools.
    Json,
    /// A GitHub Actions workflow command per file that would change, which
    /// shows it on the pull request at its first change, with the path from
    /// the repository's root: for CI.
    Github,
}

impl Args {
    /// Whether it fails when a file would change: `--check`, or its old
    /// spelling, `--diff`.
    fn checks(&self) -> bool {
        self.check || self.diff
    }

    /// Whether it writes nothing: `--check`, `--diff`, or `--dry-run`.
    fn writes_nothing(&self) -> bool {
        self.checks() || self.guard.dry_run
    }
}

/// A file as `--check` and `--dry-run` name it: from the current directory
/// when it's inside it, with `/` separators, so `git apply` run there finds
/// it; otherwise in full.
fn shown(file: &Path) -> String {
    let path = normalize(&std::path::absolute(file).unwrap_or(file.to_owned()));
    std::env::current_dir()
        .ok()
        .and_then(|here| relative_path(&normalize(&here), &path))
        .filter(ascribe_core::RelPath::is_inside)
        .map_or_else(|| path.display().to_string(), |rel| rel.to_string())
}

/// A file as `--format github` names it: from the repository's root, when
/// it's in the project and the project is in a repository; otherwise as
/// `--check` names it.
fn in_repository(global: &Global) -> impl Fn(&Path) -> String {
    let project = locate(global)
        .ok()
        .map(|config| normalize(&std::path::absolute(&config).unwrap_or(config)))
        .and_then(|config| config.parent().map(Path::to_owned));
    let prefix = project
        .as_deref()
        .and_then(|dir| ascribe_diff::Repository::discover(dir).ok())
        .map(|repo| repo.prefix);
    move |file| {
        let path = normalize(&std::path::absolute(file).unwrap_or(file.to_owned()));
        match (&project, &prefix) {
            (Some(project), Some(prefix)) => relative_path(project, &path)
                .filter(ascribe_core::RelPath::is_inside)
                .map_or_else(
                    || shown(file),
                    |rel| github::in_repository(prefix, rel.as_str()),
                ),
            _ => shown(file),
        }
    }
}

/// The workflow command for a file that would change: an error at its first
/// change, which says how many there are.
fn annotation(file: &str, found: &[changes::Change]) -> String {
    let line = found.first().map_or(1, |c| c.line + 1);
    let places = match found.len() {
        1 => "1 place".to_owned(),
        n => format!("{n} places"),
    };
    github::command(
        "error",
        &[
            ("file", file.to_owned()),
            ("line", line.to_string()),
            ("title", "ascribe fmt".to_owned()),
        ],
        &format!(
            "`ascribe fmt` would change this file in {places}; run `ascribe fmt` to format it"
        ),
    )
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
    /// Whether the files were rewritten: `false` with `--check` or
    /// `--dry-run`, which write nothing.
    written: bool,
    /// Each file that changed, or with `--check` or `--dry-run` would change, in path
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
    if let Err(e) = guarded(global, &args) {
        return exit::code(exit::fail(&mut err, &e));
    }
    let formatted = match args.format {
        Format::Text if args.checks() => format_all(global, &args, &mut |file, source, edits| {
            let found = changes::changes(source, edits);
            let _ = changes::write_diff(&mut out, &shown(file), source, &found);
        }),
        Format::Text if args.guard.dry_run => {
            format_all(global, &args, &mut |file, source, edits| {
                let found = changes::changes(source, edits);
                let _ = changes::write_lines(&mut out, &shown(file), &found);
            })
        }
        Format::Text => format_all(global, &args, &mut |file, _, _| {
            let _ = writeln!(out, "formatted {}", file.display());
        }),
        Format::Github => {
            let in_repository = in_repository(global);
            format_all(global, &args, &mut |file, source, edits| {
                let found = changes::changes(source, edits);
                let _ = writeln!(out, "{}", annotation(&in_repository(file), &found));
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
    if args.writes_nothing() && changed > 0 {
        let _ = writeln!(
            err,
            "{changed} file{} would be reformatted",
            if changed == 1 { "" } else { "s" }
        );
    }
    if !done.refused.is_empty() {
        exit::code(exit::FAILURE)
    } else if args.checks() && changed > 0 {
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
    /// The git guard refused to change a file.
    #[error(transparent)]
    Guard(#[from] guard::GuardError),
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
            FmtError::Guard(e) => e.code(),
            FmtError::Written(_) => "report_unwritable",
        }
    }
}

/// The slugs of the content model's problems.
fn model_slugs(error: &LoadError) -> Vec<&str> {
    match error {
        LoadError::Model { issues, .. } => issues.iter().map(|i| i.slug.as_str()).collect(),
        // The file couldn't be read, or isn't UTF-8: the content model's
        // syntax error.
        LoadError::Read { .. } => vec![MODEL_TOML_SYNTAX.as_str()],
    }
}

/// Whether the git guard lets `ascribe fmt` change the files it would:
/// always with `--force`, or when it writes nothing; otherwise, a pass that
/// writes nothing lists them first, and git must be able to put back each.
fn guarded(global: &Global, args: &Args) -> Result<(), FmtError> {
    if args.guard.force || args.writes_nothing() {
        return Ok(());
    }
    let mut changing = Vec::new();
    format_files(global, args, true, &mut |file, _, _| {
        changing.push(file.to_owned());
    })?;
    Ok(args.guard.allow(&changing)?)
}

/// Formats every file, listing each that changed (or would have), and
/// returns what it did.
fn format_all(
    global: &Global,
    options: &Args,
    on_changed: &mut dyn FnMut(&Path, &str, &[TextEdit]),
) -> Result<Formatted, FmtError> {
    format_files(global, options, options.writes_nothing(), on_changed)
}

/// Formats every file, writing each that changed unless `nothing`, and
/// listing it.
fn format_files(
    global: &Global,
    options: &Args,
    nothing: bool,
    on_changed: &mut dyn FnMut(&Path, &str, &[TextEdit]),
) -> Result<Formatted, FmtError> {
    let config = locate(global)?;
    let file = ascribe_check::LOADER
        .load_model(&config)
        .map_err(|error| FmtError::Model {
            config: config.clone(),
            error,
        })?;
    let model = file.model;
    let boundary = DiskFs::new(file.root, &Layout::from_model(&model));
    Ok(ascribe_fmt::format_files(
        &config,
        &model,
        &options.paths,
        nothing,
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
        written: !options.writes_nothing(),
        files,
        refused,
    };
    Ok((done, report))
}
