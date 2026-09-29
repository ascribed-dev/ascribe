//! The JSON report: `tessera check --format json`. The schema is documented,
//! with an example, in `crates/tessera-cli/README.md`. It's versioned:
//! [`SCHEMA_VERSION`] changes only when a field is removed or changes
//! meaning, and fields may be added without a new version.

use std::io::{self, Write};

use serde::Serialize;
use tessera_check::Diagnostic;

use super::{Counts, FileTable, Position};

/// The version of the JSON schema.
pub const SCHEMA_VERSION: u32 = 1;

#[derive(Serialize)]
struct Report<'a> {
    schema_version: u32,
    tessera_version: &'static str,
    /// Why the command couldn't run, or `None`.
    error: Option<&'a str>,
    files_checked: usize,
    diagnostics: Vec<Entry>,
    summary: Summary,
}

#[derive(Serialize)]
struct Summary {
    errors: usize,
    warnings: usize,
}

#[derive(Serialize)]
struct Entry {
    code: &'static str,
    slug: String,
    severity: &'static str,
    message: String,
    file: String,
    range: Range,
    related: Vec<Related>,
    fixes: Vec<Fix>,
    builds: Vec<String>,
    unpublished: bool,
}

#[derive(Serialize)]
struct Range {
    start: Pos,
    end: Pos,
}

#[derive(Serialize)]
struct Pos {
    line: u32,
    column: u32,
    offset: usize,
}

#[derive(Serialize)]
struct Related {
    file: String,
    range: Range,
    message: String,
}

#[derive(Serialize)]
struct Fix {
    title: String,
    file: String,
    edits: Vec<Edit>,
}

#[derive(Serialize)]
struct Edit {
    range: Range,
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

fn range(files: &FileTable, at: tessera_core::Location) -> Range {
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
                        range: range(files, tessera_core::Location::new(f.file, e.span)),
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
        tessera_version: env!("CARGO_PKG_VERSION"),
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
