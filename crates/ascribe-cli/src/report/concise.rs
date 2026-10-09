//! The concise report, `--format concise`: one line per diagnostic, for an
//! agent reading text. `file:line: [code] message`, grouped by file, at most
//! [`LIMIT`] of them, then the summary line.

use std::io::{self, Write};

use ascribe_check::Reported;

use super::FileTable;
use super::text::repeats_note;

/// The most diagnostics the report lists. The rest are counted, with the
/// command that narrows the check.
pub const LIMIT: usize = 50;

/// The order the concise report lists a report in: grouped by file, in file
/// order (the content model, then the source files in path order), and in
/// line order within a file. Indexes into `reported`.
pub fn order(files: &FileTable, reported: &[Reported]) -> Vec<usize> {
    let mut order: Vec<usize> = (0..reported.len()).collect();
    // Stable: diagnostics at one place keep the report's order.
    order.sort_by_key(|&i| {
        let at = reported[i].diagnostic.location;
        (at.file, files.position(at.file, at.span.start()).offset)
    });
    order
}

/// Writes the first [`LIMIT`] diagnostics in [`order`], then, if some were
/// left out, `and N more: <command>`, where `more` gives the command for the
/// first one left out, then `summary`.
pub fn write(
    out: &mut dyn Write,
    files: &FileTable,
    reported: &[Reported],
    summary: &str,
    more: &dyn Fn(&Reported) -> String,
) -> io::Result<()> {
    let order = order(files, reported);
    for &i in order.iter().take(LIMIT) {
        write_line(out, files, &reported[i])?;
    }
    if let Some(&next) = order.get(LIMIT) {
        writeln!(
            out,
            "and {} more: {}",
            order.len() - LIMIT,
            more(&reported[next])
        )?;
    }
    writeln!(out, "{summary}")
}

/// Writes one diagnostic's line.
pub fn write_line(out: &mut dyn Write, files: &FileTable, r: &Reported) -> io::Result<()> {
    let d = &r.diagnostic;
    let start = files.position(d.location.file, d.location.span.start());
    write!(
        out,
        "{}:{}: [{}] {}",
        files.path(d.location.file),
        start.line,
        d.code,
        d.message
    )?;
    match repeats_note(r.repeats) {
        Some(note) => writeln!(out, " ({note})"),
        None => writeln!(out),
    }
}
