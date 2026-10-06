//! The JSON report: `ascribe check --format json`. The schema is documented,
//! with an example, in `docs/content/reference/cli.md`. It's versioned:
//! [`SCHEMA_VERSION`] changes only when a field is removed or changes
//! meaning, and fields may be added without a new version.

use std::io::{self, Write};

use ascribe_check::Diagnostic;
use serde::Serialize;

use super::{Counts, FileTable, Position};

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
    /// How many source files were checked.
    files_checked: usize,
    /// Every diagnostic, in file order, and in source order within a file.
    diagnostics: Vec<Entry>,
    /// How many errors and warnings.
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
}

/// A diagnostic.
#[derive(Serialize)]
#[cfg_attr(test, derive(schemars::JsonSchema))]
struct Entry {
    /// The code, such as `ASC036`.
    code: &'static str,
    /// The diagnostic's name, such as `link-target-missing`.
    slug: String,
    /// `error` or `warning`.
    severity: &'static str,
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
}

/// A span of a file. `end` is just past its last character; an empty range
/// (an insertion) has equal positions.
#[derive(Serialize)]
#[cfg_attr(test, derive(schemars::JsonSchema))]
struct Range {
    /// Its first character.
    start: Pos,
    /// Just past its last character.
    end: Pos,
}

/// A position in a file.
#[derive(Serialize)]
#[cfg_attr(test, derive(schemars::JsonSchema))]
struct Pos {
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
}

/// One edit of a fix.
#[derive(Serialize)]
#[cfg_attr(test, derive(schemars::JsonSchema))]
struct Edit {
    /// The text it replaces.
    range: Range,
    /// The text that replaces it.
    new_text: String,
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

fn entry(files: &FileTable, d: &Diagnostic) -> Entry {
    Entry {
        code: d.code,
        slug: d.slug.to_string(),
        severity: d.severity.as_str(),
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
            })
            .collect(),
        builds: d.builds.clone(),
        unpublished: d.unpublished,
    }
}

/// Writes the report as one JSON document and a newline.
///
/// `error` is why the command couldn't run to the end, when it couldn't; the
/// diagnostics are whatever was found before that (a content model's
/// problems, for example).
pub fn write(
    out: &mut dyn Write,
    files: &FileTable,
    diagnostics: &[Diagnostic],
    files_checked: usize,
    error: Option<&str>,
) -> io::Result<()> {
    let counts = Counts::of(diagnostics);
    let report = Report {
        schema_version: SCHEMA_VERSION,
        ascribe_version: env!("CARGO_PKG_VERSION"),
        error,
        files_checked,
        diagnostics: diagnostics.iter().map(|d| entry(files, d)).collect(),
        summary: Summary {
            errors: counts.errors,
            warnings: counts.warnings,
        },
    };
    serde_json::to_writer_pretty(&mut *out, &report).map_err(io::Error::other)?;
    writeln!(out)
}
