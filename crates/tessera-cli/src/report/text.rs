//! The readable report: each diagnostic with a snippet of its source.

use std::io::{self, Write};
use std::ops::Range;

use ariadne::{Config, Label, Report, ReportKind, sources};
use tessera_check::{Diagnostic, Severity};

use super::{Counts, FileTable};

/// Writes every diagnostic, then a one-line summary.
///
/// `files_checked` is how many source files were checked.
pub fn write(
    out: &mut dyn Write,
    files: &FileTable,
    diagnostics: &[Diagnostic],
    files_checked: usize,
    color: bool,
) -> io::Result<()> {
    for d in diagnostics {
        write_diagnostic(out, files, d, color)?;
    }
    writeln!(out, "{}", summary(Counts::of(diagnostics), files_checked))
}

/// The summary line: `checked 12 files: 1 error, 2 warnings`.
pub fn summary(counts: Counts, files_checked: usize) -> String {
    let plural = |n: usize, word: &str| format!("{n} {word}{}", if n == 1 { "" } else { "s" });
    format!(
        "checked {}: {}, {}",
        plural(files_checked, "file"),
        plural(counts.errors, "error"),
        plural(counts.warnings, "warning"),
    )
}

/// Writes one diagnostic with its snippet.
pub fn write_diagnostic(
    out: &mut dyn Write,
    files: &FileTable,
    d: &Diagnostic,
    color: bool,
) -> io::Result<()> {
    let kind = match d.severity {
        Severity::Error => ReportKind::Error,
        Severity::Warning => ReportKind::Warning,
    };
    let primary = (files.path(d.location.file), char_range(files, d.location));
    let mut report = Report::build(kind, primary.clone())
        .with_code(d.code)
        .with_message(&d.message)
        .with_config(Config::default().with_color(color))
        .with_label(Label::new(primary).with_message(d.slug));
    for related in &d.related {
        report = report.with_label(
            Label::new((
                files.path(related.location.file),
                char_range(files, related.location),
            ))
            .with_message(&related.message),
        );
    }
    if let Some(fix) = d.fixes.first() {
        report = report.with_help(&fix.title);
    }
    let cache = sources(files.texts());
    report.finish().write(cache, out)
}

/// A location's span in characters (which is what ariadne counts), at least
/// one character wide when the text allows, so an empty span still shows.
fn char_range(files: &FileTable, at: tessera_core::Location) -> Range<usize> {
    let Some(text) = files.text(at.file) else {
        return 0..0;
    };
    let chars = |byte: usize| {
        let byte = byte.min(text.len());
        text.get(..byte).map_or(0, |t| t.chars().count())
    };
    let (start, end) = (chars(at.span.start()), chars(at.span.end()));
    if end > start {
        return start..end;
    }
    let total = text.chars().count();
    if start < total {
        start..start + 1
    } else {
        start.saturating_sub(1)..start
    }
}
