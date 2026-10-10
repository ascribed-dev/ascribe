//! The JSON report: `ascribe check --format json`. The schema is documented,
//! with an example, in `docs/content/reference/cli.md`. It's versioned:
//! [`SCHEMA_VERSION`] changes only when a field is removed or changes
//! meaning, and fields may be added without a new version.

use std::io::{self, Write};

use ascribe_check::{Diagnostic, Registry, Reported};
use serde::Serialize;

use super::{Counts, FileTable, Position, position_in, tally};

/// The version of the JSON schema.
pub const SCHEMA_VERSION: u32 = 1;

/// What `ascribe check --format json` and `ascribe build --format json`
/// write: one document, whatever the outcome. Fields can be added without a
/// new `schema_version`, so a reader ignores fields it doesn't know.
#[derive(Serialize)]
#[cfg_attr(test, derive(schemars::JsonSchema))]
pub(crate) struct Report<'a> {
    /// The version of this schema. It changes only when a field is removed
    /// or changes meaning.
    schema_version: u32,
    /// The version of Ascribe that wrote it.
    ascribe_version: &'static str,
    /// Why the project couldn't be checked (exit code 2), or `null`. When it
    /// isn't `null`, `diagnostics` holds what was found first: the content
    /// model's problems.
    error: Option<&'a str>,
    /// How many source files were checked: the project's.
    files_checked: usize,
    /// How many of them the report covers: the source files in the paths
    /// named, or every one when no path was.
    files_reported: usize,
    /// The builds whose page-level checks ran, in `ascribe.toml`'s order:
    /// every build, the ones named with `--build`, or the editor's with
    /// `--editor-build`. Empty when the project couldn't be checked.
    builds_checked: Vec<String>,
    /// Every diagnostic, in file order, and in source order within a file;
    /// advice after the errors and warnings, in the same order.
    /// With paths, only those that count for them. With `--summary`, none:
    /// see `truncated`.
    diagnostics: Vec<Entry>,
    /// Whether `diagnostics` leaves some out. It does with `--summary`,
    /// which lists none.
    truncated: bool,
    /// How many diagnostics `diagnostics` lists.
    shown: usize,
    /// How many diagnostics there are.
    total: usize,
    /// The command that lists the ones left out, when `truncated`; `null`
    /// otherwise.
    next_command: Option<String>,
    /// How many errors, warnings, and advice.
    summary: Summary,
}

/// How many diagnostics of each severity.
#[derive(Serialize)]
#[cfg_attr(test, derive(schemars::JsonSchema))]
struct Summary {
    /// How many errors.
    errors: usize,
    /// How many warnings.
    warnings: usize,
    /// How many advice.
    advice: usize,
    /// With `--summary`: how many diagnostics have each code, most first.
    #[serde(skip_serializing_if = "Option::is_none")]
    by_code: Option<Vec<CodeCount>>,
    /// With `--summary`: how many diagnostics are in each file, most first.
    #[serde(skip_serializing_if = "Option::is_none")]
    by_file: Option<Vec<FileCount>>,
}

/// How many diagnostics have one code.
#[derive(Serialize)]
#[cfg_attr(test, derive(schemars::JsonSchema))]
struct CodeCount {
    /// The code, such as `ASC036`.
    code: &'static str,
    /// The diagnostic's name, such as `link-target-missing`.
    slug: String,
    /// `error`, `warning`, or `advice`.
    severity: &'static str,
    /// How many.
    count: usize,
}

/// How many diagnostics are in one file.
#[derive(Serialize)]
#[cfg_attr(test, derive(schemars::JsonSchema))]
struct FileCount {
    /// The file, as a diagnostic's `file` is.
    file: String,
    /// How many errors.
    errors: usize,
    /// How many warnings.
    warnings: usize,
    /// How many advice.
    advice: usize,
}

/// A diagnostic.
#[derive(Serialize)]
#[cfg_attr(test, derive(schemars::JsonSchema))]
struct Entry {
    /// The code, such as `ASC036`.
    code: &'static str,
    /// The diagnostic's name, such as `link-target-missing`.
    slug: String,
    /// `error`, `warning`, or `advice`. Advice never fails the command.
    /// More severities may be added; a reader treats one it doesn't know as
    /// it treats advice.
    severity: &'static str,
    /// The kind of next step: `fix` when Ascribe can make the edit,
    /// `choose` when the author picks among things Ascribe can list,
    /// `write` when it needs writing or judgment, `outside` when nothing in
    /// the source can fix it, and `review` when it may be fine as it is.
    next: &'static str,
    /// The rule of the program that found it, such as `Ascribe.Repeated`
    /// from Vale, for a `prose` diagnostic. Absent for Ascribe's own.
    #[serde(skip_serializing_if = "Option::is_none")]
    rule: Option<String>,
    /// What's wrong, and what to do about it.
    message: String,
    /// The file, relative to the project root (the directory of
    /// `ascribe.toml`), with `/` separators. `ascribe.toml` for a
    /// content-model problem.
    file: String,
    /// Where in the file.
    range: Range,
    /// Other places that explain it.
    related: Vec<Related>,
    /// Edits that would fix it.
    fixes: Vec<Fix>,
    /// The builds a page-level diagnostic appears in, in `ascribe.toml`'s
    /// order. Empty for a file-level diagnostic, and for one in content no
    /// build publishes. With `--build`, only that build.
    builds: Vec<String>,
    /// Whether it's in content that no build publishes.
    unpublished: bool,
    /// How to fix it, in general: the diagnostics reference's advice for its
    /// code.
    help: String,
    /// The address of its entry in the diagnostics reference.
    docs: String,
    /// For a problem in included content reported because one of its related
    /// places is in a path named: at how many other includes it's reported
    /// too, collapsed into this one. `0` otherwise.
    repeats: usize,
}

/// A span of a file. `end` is just past its last character; an empty range
/// (an insertion) has equal positions.
#[derive(Serialize)]
#[cfg_attr(test, derive(schemars::JsonSchema))]
pub(crate) struct Range {
    /// Its first character.
    start: Pos,
    /// Just past its last character.
    end: Pos,
}

/// A position in a file.
#[derive(Serialize)]
#[cfg_attr(test, derive(schemars::JsonSchema))]
pub(crate) struct Pos {
    /// The line, from 1.
    line: u32,
    /// The column, from 1, in Unicode characters (not bytes or UTF-16
    /// units).
    column: u32,
    /// The byte offset from the start of the file.
    offset: usize,
}

/// Another place that explains a diagnostic.
#[derive(Serialize)]
#[cfg_attr(test, derive(schemars::JsonSchema))]
struct Related {
    /// The file, as a diagnostic's `file` is.
    file: String,
    /// Where in the file.
    range: Range,
    /// What it has to do with the diagnostic.
    message: String,
}

/// Edits that would fix a diagnostic.
#[derive(Serialize)]
#[cfg_attr(test, derive(schemars::JsonSchema))]
struct Fix {
    /// What the fix does.
    title: String,
    /// The file the edits are in, as a diagnostic's `file` is.
    file: String,
    /// The edits, each replacing the text of its range.
    edits: Vec<Edit>,
    /// `safe` when applying the edits as they are can't change what the page
    /// says and leaves nothing to decide; `unsafe` otherwise.
    applicability: &'static str,
}

/// One edit of a fix.
#[derive(Serialize)]
#[cfg_attr(test, derive(schemars::JsonSchema))]
pub(crate) struct Edit {
    /// The text it replaces.
    range: Range,
    /// The text that replaces it.
    new_text: String,
}

impl Edit {
    /// An edit of the text `index` indexes, with its positions worked out as
    /// a diagnostic's are.
    pub(crate) fn in_text(index: &ascribe_core::LineIndex, edit: &ascribe_core::TextEdit) -> Edit {
        Edit {
            range: Range {
                start: position_in(index, edit.span.start()).into(),
                end: position_in(index, edit.span.end()).into(),
            },
            new_text: edit.new_text.clone(),
        }
    }
}

impl From<Position> for Pos {
    fn from(p: Position) -> Pos {
        Pos {
            line: p.line,
            column: p.column,
            offset: p.offset,
        }
    }
}

fn range(files: &FileTable, at: ascribe_core::Location) -> Range {
    let (start, end) = files.range(at);
    Range {
        start: start.into(),
        end: end.into(),
    }
}

fn entry(files: &FileTable, d: &Diagnostic, repeats: usize) -> Entry {
    let registered = Registry::global().get(d.slug);
    Entry {
        code: d.code,
        slug: d.slug.to_string(),
        severity: d.severity.as_str(),
        next: d.next().map_or("write", ascribe_check::Next::as_str),
        rule: d.rule.clone(),
        message: d.message.clone(),
        file: files.path(d.location.file),
        range: range(files, d.location),
        related: d
            .related
            .iter()
            .map(|r| Related {
                file: files.path(r.location.file),
                range: range(files, r.location),
                message: r.message.clone(),
            })
            .collect(),
        fixes: d
            .fixes
            .iter()
            .map(|f| Fix {
                title: f.title.clone(),
                file: files.path(f.file),
                edits: f
                    .edits
                    .iter()
                    .map(|e| Edit {
                        range: range(files, ascribe_core::Location::new(f.file, e.span)),
                        new_text: e.new_text.clone(),
                    })
                    .collect(),
                applicability: f.applicability.as_str(),
            })
            .collect(),
        builds: d.builds.clone(),
        unpublished: d.unpublished,
        help: registered.and_then(|e| e.fix.clone()).unwrap_or_default(),
        docs: registered
            .map(ascribe_check::Entry::docs)
            .unwrap_or_default(),
        repeats,
    }
}

/// What a report is about, besides its diagnostics.
pub struct About<'a> {
    /// Why the command couldn't run to the end, when it couldn't; the
    /// diagnostics are whatever was found before that (a content model's
    /// problems, for example).
    pub error: Option<&'a str>,
    /// How many source files were checked.
    pub files_checked: usize,
    /// How many of them the report covers.
    pub files_reported: usize,
    /// The builds whose page-level checks ran.
    pub builds_checked: Vec<String>,
    /// With `--summary`, the command that lists the diagnostics: the list is
    /// left out, and counted instead.
    pub summary_only: Option<String>,
}

impl About<'_> {
    /// A command that stopped before checking anything, because of `error`.
    pub fn failed(error: &str) -> About<'_> {
        About {
            error: Some(error),
            files_checked: 0,
            files_reported: 0,
            builds_checked: Vec::new(),
            summary_only: None,
        }
    }
}

/// Writes the report as one JSON document and a newline.
pub fn write(
    out: &mut dyn Write,
    files: &FileTable,
    reported: &[Reported],
    about: &About<'_>,
) -> io::Result<()> {
    let counts = Counts::of_reported(reported);
    let total = reported.len();
    let (diagnostics, by_code, by_file) = if about.summary_only.is_some() {
        let (codes, by_file) = tally(files, reported);
        let codes = codes
            .into_iter()
            .map(|c| CodeCount {
                code: c.code,
                slug: c.slug,
                severity: c.severity.as_str(),
                count: c.count,
            })
            .collect();
        let by_file = by_file
            .into_iter()
            .map(|f| FileCount {
                file: f.file,
                errors: f.counts.errors,
                warnings: f.counts.warnings,
                advice: f.counts.advice,
            })
            .collect();
        (Vec::new(), Some(codes), Some(by_file))
    } else {
        let list = reported
            .iter()
            .map(|r| entry(files, &r.diagnostic, r.repeats))
            .collect();
        (list, None, None)
    };
    let shown = diagnostics.len();
    let truncated = shown < total;
    let report = Report {
        schema_version: SCHEMA_VERSION,
        ascribe_version: env!("CARGO_PKG_VERSION"),
        error: about.error,
        files_checked: about.files_checked,
        files_reported: about.files_reported,
        builds_checked: about.builds_checked.clone(),
        diagnostics,
        truncated,
        shown,
        total,
        next_command: about.summary_only.clone().filter(|_| truncated),
        summary: Summary {
            errors: counts.errors,
            warnings: counts.warnings,
            advice: counts.advice,
            by_code,
            by_file,
        },
    };
    serde_json::to_writer_pretty(&mut *out, &report).map_err(io::Error::other)?;
    writeln!(out)
}
