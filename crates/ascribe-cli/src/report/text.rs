//! The readable report: each diagnostic with a snippet of its source.

use std::collections::HashSet;
use std::io::{self, Write};
use std::ops::Range;

use ariadne::{Cache, Config, FnCache, Label, Report, ReportKind};
use ascribe_check::{Diagnostic, Registry, Reported, Severity};

use super::{ByCode, ByFile, Counts, FileTable};

/// Writes what `ascribe build` reports of its checks: each error with its
/// snippet, since errors stop the build, then one summary line.
///
/// The line is [`build_summary`]'s. `check` is the `ascribe check` command
/// that lists each diagnostic, which the line points to when there are
/// warnings or advice it doesn't show.
#[allow(clippy::too_many_arguments)] // The line's parts, each needed.
pub fn write_build(
    out: &mut dyn Write,
    files: &FileTable,
    diagnostics: &[Diagnostic],
    files_checked: usize,
    covered: &Covered<'_>,
    acknowledged: usize,
    baselined: usize,
    check: &str,
    color: bool,
) -> io::Result<()> {
    let errors: Vec<Diagnostic> = diagnostics
        .iter()
        .filter(|d| d.severity == Severity::Error)
        .cloned()
        .collect();
    write_diagnostics(out, files, &errors, color)?;
    writeln!(
        out,
        "{}",
        build_summary(
            Counts::of(diagnostics),
            files_checked,
            covered,
            acknowledged,
            baselined,
            check
        )
    )
}

/// The summary line of `ascribe build`'s checks: `checked 8 files in every
/// build: no problems` when they found none, and otherwise the counts, with
/// the command that lists each warning and advice when there are any, as
/// `checked 8 files in every build: 0 errors, 1 warning, 3 advice: ascribe
/// check for each`. The problems acknowledged and those the baseline holds
/// back are counted after, as `ascribe check` counts them.
pub fn build_summary(
    counts: Counts,
    files_checked: usize,
    covered: &Covered<'_>,
    acknowledged: usize,
    baselined: usize,
    check: &str,
) -> String {
    let builds = match covered {
        Covered::Every => "in every build".to_owned(),
        Covered::Named([one]) => format!("in build `{one}`"),
        Covered::Named(names) => format!("in builds {}", listed(names)),
        Covered::Editor(build) => format!("with the page-level checks of build `{build}` only"),
    };
    let checked = format!("checked {} {builds}", plural(files_checked, "file"));
    if counts.total() == 0 {
        return format!(
            "{checked}: no problems{}{}",
            self::acknowledged(acknowledged),
            self::baselined(baselined)
        );
    }
    let pointer = if counts.warnings + counts.advice > 0 {
        format!(": {check} for each")
    } else {
        String::new()
    };
    format!(
        "{checked}: {}, {}, {} advice{}{}{pointer}",
        plural(counts.errors, "error"),
        plural(counts.warnings, "warning"),
        counts.advice,
        self::acknowledged(acknowledged),
        self::baselined(baselined)
    )
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
    let mut shown = ShownHelp::default();
    for d in diagnostics {
        write_diagnostic(out, files, &mut cache, &mut shown, d, 0, color)?;
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
    let mut shown = ShownHelp::default();
    for r in reported {
        write_diagnostic(
            out,
            files,
            &mut cache,
            &mut shown,
            &r.diagnostic,
            r.repeats,
            color,
        )?;
    }
    writeln!(out, "{summary}")
}

/// The most files `--summary` lists. The rest are counted, with the command
/// that lists them all.
pub const TALLY_FILES: usize = 20;

/// Writes the counts by code and by file, most first, for `--summary`: at
/// most [`TALLY_FILES`] files, then `and N more files: <more>`, then
/// `first`, the command that lists the first code's diagnostics. The
/// summary line follows.
pub fn write_tally(
    out: &mut dyn Write,
    codes: &[ByCode],
    by_file: &[ByFile],
    more: &str,
    first: Option<&str>,
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
    write_code_lines(out, codes, width)?;
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
    if let Some(first) = first {
        writeln!(out, "To work through the first code: {first}")?;
    }
    Ok(())
}

/// Writes a line for each code, as `--summary` does, with its count right
/// aligned to the widest.
pub fn write_codes(out: &mut dyn Write, codes: &[ByCode]) -> io::Result<()> {
    let width = codes
        .iter()
        .map(|c| c.count)
        .max()
        .unwrap_or_default()
        .to_string()
        .len();
    write_code_lines(out, codes, width)
}

fn write_code_lines(out: &mut dyn Write, codes: &[ByCode], width: usize) -> io::Result<()> {
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
    Ok(())
}

/// `1 error, 2 warnings`, and `, 3 advice` when there's any.
pub fn counted(counts: Counts) -> String {
    let mut text = format!(
        "{}, {}",
        plural(counts.errors, "error"),
        plural(counts.warnings, "warning")
    );
    if counts.advice > 0 {
        text.push_str(&format!(", {} advice", counts.advice));
    }
    text
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

/// The builds a check covered, as `ascribe check`'s summary names them.
pub enum Covered<'a> {
    /// Every build in `ascribe.toml`: `for every build`.
    Every,
    /// The builds `--build` named: `for build `site``, or
    /// `for builds `site` and `cloud``.
    Named(&'a [String]),
    /// The editor's build, whose page-level checks alone ran, with
    /// `--editor-build`.
    Editor(&'a str),
}

/// The summary line of `ascribe check`, in one form whatever was asked: the
/// files checked, the builds covered, the files the paths named when there
/// were paths (`reported on 2 files in the paths`), how many files have a
/// problem reported, the codes `--code` narrowed it to (`for code
/// `ASC036``), and all three counts, as `checked 12 files for every build,
/// 1 with problems: 1 error, 0 warnings, 0 advice`.
pub fn check_summary(
    counts: Counts,
    files_checked: usize,
    covered: &Covered<'_>,
    files_named: Option<usize>,
    with_problems: usize,
    codes: &[String],
) -> String {
    let builds = match covered {
        Covered::Every => "for every build".to_owned(),
        Covered::Named([one]) => format!("for build `{one}`"),
        Covered::Named(names) => format!("for builds {}", listed(names)),
        Covered::Editor(build) => format!("with the page-level checks of build `{build}` only"),
    };
    let named = files_named.map_or_else(String::new, |n| {
        format!(", reported on {} in the paths", plural(n, "file"))
    });
    let only = match codes {
        [] => String::new(),
        [one] => format!(" for code `{one}`"),
        codes => format!(" for codes {}", listed(codes)),
    };
    format!(
        "checked {} {builds}{named}, {with_problems} with problems{only}: {}, {}, {} advice",
        plural(files_checked, "file"),
        plural(counts.errors, "error"),
        plural(counts.warnings, "warning"),
        counts.advice
    )
}

/// Names in backticks, the last after `and`: `` `a`, `b` and `c` ``.
fn listed(names: &[String]) -> String {
    let quoted: Vec<String> = names.iter().map(|n| format!("`{n}`")).collect();
    match quoted.split_last() {
        Some((last, [])) => last.clone(),
        Some((last, rest)) => format!("{} and {last}", rest.join(", ")),
        None => String::new(),
    }
}

/// What a summary line adds for the problems acknowledged as intended:
/// `, 2 acknowledged`, or nothing when there are none.
pub fn acknowledged(count: usize) -> String {
    if count == 0 {
        String::new()
    } else {
        format!(", {count} acknowledged")
    }
}

/// What a summary line adds for the problems the baseline holds back:
/// `, 3 in the baseline`, or nothing when there are none.
pub fn baselined(count: usize) -> String {
    if count == 0 {
        String::new()
    } else {
        format!(", {count} in the baseline")
    }
}

/// The help a report has printed in full, by code, so a later diagnostic
/// with the same code and the same help points back to it instead of
/// repeating it. Help that names this occurrence differs, so it's printed
/// every time.
#[derive(Default)]
struct ShownHelp(HashSet<(&'static str, String)>);

impl ShownHelp {
    /// The help to print for a diagnostic: in full the first time the code
    /// shows it, else a pointer to that first one.
    fn help(&mut self, code: &'static str, help: &str) -> String {
        if self.0.insert((code, help.to_owned())) {
            help.to_owned()
        } else {
            format!("as for {code} above (ascribe explain {code})")
        }
    }
}

/// Writes one diagnostic with its snippet, from the report's source cache,
/// and its help, shortened when `shown` has printed it.
fn write_diagnostic(
    out: &mut dyn Write,
    files: &FileTable,
    cache: &mut impl Cache<String>,
    shown: &mut ShownHelp,
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
        Severity::Advice => ReportKind::Advice,
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
    // The first fix's title, which says what applying it does; else the
    // registry's help, the next step `check`'s JSON gives as `help`, with
    // its Markdown links written for a terminal.
    let help = d.fixes.first().map(|fix| fix.title.as_str()).or_else(|| {
        Registry::global()
            .get(d.slug)
            .and_then(|e| e.fix.as_deref())
            .filter(|help| !help.is_empty())
    });
    if let Some(help) = help {
        report = report.with_help(shown.help(d.code, &super::links::plain(help)));
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
