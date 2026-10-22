//! The concise report, `--format concise`: one line per diagnostic, for an
//! agent reading text. `file:line: [code] message`, grouped by file, at most
//! [`LIMIT`] of them unless `--limit` says otherwise, then the summary line.

use std::io::{self, Write};
use std::str::FromStr;

use ascribe_check::{Reported, Severity};

use super::FileTable;
use super::text::repeats_note;

/// The most diagnostics the report lists, unless `--limit` says otherwise.
/// The rest are counted, with the command that lists them all.
pub const LIMIT: usize = 50;

/// How many diagnostics the report lists: `--limit`, a number or `all`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Limit {
    /// At most this many.
    At(usize),
    /// Every one.
    All,
}

impl Default for Limit {
    fn default() -> Self {
        Limit::At(LIMIT)
    }
}

impl FromStr for Limit {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s == "all" {
            return Ok(Limit::All);
        }
        s.parse()
            .map(Limit::At)
            .map_err(|_| format!("`{s}` is neither a number nor `all`"))
    }
}

impl Limit {
    /// How many of `total` the report lists.
    fn of(self, total: usize) -> usize {
        match self {
            Limit::At(n) => n.min(total),
            Limit::All => total,
        }
    }
}

/// The order the concise report lists a report in: grouped by file, in file
/// order (the content model, then the source files in path order), and in
/// line order within a file; advice after everything else, in the same
/// order. Indexes into `reported`.
pub fn order(files: &FileTable, reported: &[Reported]) -> Vec<usize> {
    let mut order: Vec<usize> = (0..reported.len()).collect();
    // Stable: diagnostics at one place keep the report's order.
    order.sort_by_key(|&i| {
        let d = &reported[i].diagnostic;
        let at = d.location;
        (
            d.severity == Severity::Advice,
            at.file,
            files.position(at.file, at.span.start()).offset,
        )
    });
    order
}

/// Writes the first diagnostics in [`order`], as many as `limit` says, then,
/// if some were left out, `and N more: <command>`, where `all` is the
/// command that lists every one, then `summary`.
pub fn write(
    out: &mut dyn Write,
    files: &FileTable,
    reported: &[Reported],
    summary: &str,
    limit: Limit,
    all: &str,
) -> io::Result<()> {
    let order = order(files, reported);
    let listed = limit.of(order.len());
    for &i in &order[..listed] {
        write_line(out, files, &reported[i])?;
    }
    if listed < order.len() {
        writeln!(out, "and {} more: {all}", order.len() - listed)?;
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
