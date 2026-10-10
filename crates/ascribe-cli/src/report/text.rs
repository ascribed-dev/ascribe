//! The readable report: each diagnostic with a snippet of its source.

use std::io::{self, Write};
use std::ops::Range;

use ariadne::{Cache, Config, FnCache, Label, Report, ReportKind};
use ascribe_check::{Diagnostic, Reported, Severity};

use super::{ByCode, ByFile, Counts, FileTable};

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
    write_diagnostics(out, files, diagnostics, color)?;
    writeln!(out, "{}", summary(Counts::of(diagnostics), files_checked))
}

/// Writes each diagnostic with its snippet.
///
/// The snippets share one source cache, which reads a file's text and builds
/// its line table the first time a diagnostic shows that file, so the cost is
/// the diagnostics plus the files they show, not their product.
pub fn write_diagnostics(
    out: &mut dyn Write,
    files: &FileTable,
    diagnostics: &[Diagnostic],
    color: bool,
) -> io::Result<()> {
    let mut cache = FnCache::new(|path: &String| {
        files
            .text_at(path)
            .ok_or_else(|| format!("no file {path} in the report"))
    });
    for d in diagnostics {
        write_diagnostic(out, files, &mut cache, d, 0, color)?;
    }
    Ok(())
}

/// Writes each diagnostic of a report with its snippet, then `summary`.
pub fn write_reported(
    out: &mut dyn Write,
    files: &FileTable,
    reported: &[Reported],
    summary: &str,
    color: bool,
) -> io::Result<()> {
    let mut cache = FnCache::new(|path: &String| {
        files
            .text_at(path)
            .ok_or_else(|| format!("no file {path} in the report"))
    });
    for r in reported {
        write_diagnostic(out, files, &mut cache, &r.diagnostic, r.repeats, color)?;
    }
    writeln!(out, "{summary}")
}

/// The most files `--summary` lists. The rest are counted, with the command
/// that lists them all.
pub const TALLY_FILES: usize = 20;

/// Writes the counts by code and by file, most first, for `--summary`: at
/// most [`TALLY_FILES`] files, then `and N more files: <more>`. The summary
/// line follows.
pub fn write_tally(
    out: &mut dyn Write,
    codes: &[ByCode],
    by_file: &[ByFile],
    more: &str,
) -> io::Result<()> {
    if codes.is_empty() {
        return Ok(());
    }
    let width = codes
        .iter()
        .map(|c| c.count)
        .chain(by_file.iter().map(|f| f.counts.total()))
        .max()
        .unwrap_or_default()
        .to_string()
        .len();
    writeln!(out, "By code:")?;
    for c in codes {
        writeln!(
            out,
            "  {:>width$}  {} {} ({})",
            c.count,
            c.code,
            c.slug,
            c.severity.as_str()
        )?;
    }
    writeln!(out, "By file:")?;
    for f in by_file.iter().take(TALLY_FILES) {
        writeln!(
            out,
            "  {:>width$}  {} ({})",
            f.counts.total(),
            f.file,
            counted(f.counts)
        )?;
    }
    if by_file.len() > TALLY_FILES {
        writeln!(
            out,
            "  and {} more files: {more}",
            by_file.len() - TALLY_FILES
        )?;
    }
    Ok(())
}

/// `1 error, 2 warnings`.
fn counted(counts: Counts) -> String {
    format!(
        "{}, {}",
        plural(counts.errors, "error"),
        plural(counts.warnings, "warning")
    )
}

fn plural(n: usize, word: &str) -> String {
    format!("{n} {word}{}", if n == 1 { "" } else { "s" })
}

/// What a diagnostic reported at several includes adds to its message: at
/// how many others it's reported too.
pub fn repeats_note(repeats: usize) -> Option<String> {
    (repeats > 0).then(|| {
        format!(
            "also at {repeats} other include{}",
            if repeats == 1 { "" } else { "s" }
        )
    })
}

/// The summary line: `checked 12 files: 1 error, 2 warnings`.
pub fn summary(counts: Counts, files_checked: usize) -> String {
    format!(
        "checked {}: {}",
        plural(files_checked, "file"),
        counted(counts)
    )
}

/// The summary line of `ascribe check`: [`summary`]'s, plus how many files the
/// paths named, when paths were named (`checked 12 files, reported on 1`), and
/// the one build whose page-level checks ran, with `--editor-build`
/// (`, build site only`).
pub fn check_summary(
    counts: Counts,
    files_checked: usize,
    files_reported: Option<usize>,
    only_build: Option<&str>,
) -> String {
    let mut line = format!("checked {}", plural(files_checked, "file"));
    if let Some(n) = files_reported {
        line.push_str(&format!(", reported on {n}"));
    }
    if let Some(build) = only_build {
        line.push_str(&format!(", page-level checks of build `{build}` only"));
    }
    format!("{line}: {}", counted(counts))
}

/// Writes one diagnostic with its snippet, from the report's source cache.
fn write_diagnostic(
    out: &mut dyn Write,
    files: &FileTable,
    cache: &mut impl Cache<String>,
    d: &Diagnostic,
    repeats: usize,
    color: bool,
) -> io::Result<()> {
    let message = match repeats_note(repeats) {
        Some(note) => format!("{} ({note})", d.message),
        None => d.message.clone(),
    };
    let kind = match d.severity {
        Severity::Error => ReportKind::Error,
        Severity::Warning => ReportKind::Warning,
    };
    let primary = (files.path(d.location.file), char_range(files, d.location));
    let mut report = Report::build(kind, primary.clone())
        .with_code(d.code)
        .with_message(&message)
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
    report.finish().write(cache, out)
}

/// A location's span in characters (which is what ariadne counts), at least
/// one character wide when the text allows, so an empty span still shows.
fn char_range(files: &FileTable, at: ascribe_core::Location) -> Range<usize> {
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
