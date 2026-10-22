//! The baseline: `ascribe.baseline`, beside `ascribe.toml` and committed,
//! which lists a project's advice by [`Identity`], so `ascribe check` holds
//! it back and a team adopting the checks sees only what it adds (decision
//! #789).
//!
//! The file is one line a finding, after a line that names the format and
//! its version:
//!
//! ```text
//! # ascribe baseline 1
//! # Advice put off for now: ascribe baseline write adds, ascribe baseline prune removes.
//! # check fingerprint order file
//! description-too-long 3f9a2c1b9e0d4a77 0 guides/install.md
//! ```
//!
//! `#` starts a comment, and a blank line is ignored. The fields are
//! separated by one space, and the file is last, so a path with spaces needs
//! no quoting. [`text`] writes the lines sorted by file, then check, then
//! fingerprint, then order.
//!
//! Only advice is held back: an entry whose check a project has since
//! raised to a warning or an error matches its finding, and doesn't hide it.
//! An entry that matches no finding is reported, as advice, so a stale
//! entry can't hide a later finding with the same fingerprint.

use std::collections::{BTreeSet, HashSet};

use ascribe_core::{FileId, Issue, Location, Span, diagnostics};

use crate::identity::{Identity, identities};
use crate::{Diagnostic, Project, Severity, apply_levels};

/// The baseline's file, beside `ascribe.toml`.
pub const BASELINE_FILE: &str = "ascribe.baseline";

/// The id of `ascribe.baseline`, for locations in it: just before the
/// navigation file's.
pub const BASELINE_FILE_ID: FileId = FileId::new(0x7FFF_FFFC);

/// The version of the format this Ascribe reads and writes.
pub const VERSION: u32 = 1;

/// The line that starts the file, naming the format and its version.
const HEADER: &str = "# ascribe baseline 1";

/// The comment [`text`] writes after the header.
const COMMENT: &str = "# Advice put off for now: ascribe baseline write adds, ascribe baseline prune removes.\n# check fingerprint order file\n";

/// One line of the baseline: a finding, by its identity.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Entry {
    /// The finding it holds back.
    pub identity: Identity,
    /// The line, in the file's text, without its end.
    pub span: Span,
}

/// `ascribe.baseline`, read.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Baseline {
    /// The entries, in the file's order.
    pub entries: Vec<Entry>,
}

/// A line of `ascribe.baseline` that doesn't parse: the baseline can't be
/// read, so nothing is checked with it.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[error("{BASELINE_FILE}:{line}: {message}")]
pub struct BaselineError {
    /// The line, from 1.
    pub line: usize,
    /// What's wrong with it.
    pub message: String,
}

impl ascribe_core::Coded for BaselineError {
    fn code(&self) -> &'static str {
        "baseline_invalid"
    }
}

impl Baseline {
    /// Reads the text of `ascribe.baseline`.
    ///
    /// # Errors
    ///
    /// The first line that doesn't parse: a first line that isn't
    /// `# ascribe baseline 1`, or an entry without its four fields.
    pub fn parse(text: &str) -> Result<Baseline, BaselineError> {
        let mut entries = Vec::new();
        let mut offset = 0;
        for (i, raw) in text.split_inclusive('\n').enumerate() {
            let start = offset;
            offset += raw.len();
            let line = raw.strip_suffix('\n').unwrap_or(raw);
            let line = line.strip_suffix('\r').unwrap_or(line);
            let number = i + 1;
            let wrong = |message: String| BaselineError {
                line: number,
                message,
            };
            if i == 0 {
                header(line).map_err(wrong)?;
                continue;
            }
            if line.trim().is_empty() || line.starts_with('#') {
                continue;
            }
            let identity = entry(line).map_err(wrong)?;
            entries.push(Entry {
                identity,
                span: Span::new(start, start + line.len()),
            });
        }
        if text.is_empty() {
            header("").map_err(|message| BaselineError { line: 1, message })?;
        }
        Ok(Baseline { entries })
    }
}

/// Whether `line` is the header: `# ascribe baseline 1`.
fn header(line: &str) -> Result<(), String> {
    if line == HEADER {
        return Ok(());
    }
    let newer = line
        .strip_prefix("# ascribe baseline ")
        .and_then(|v| v.parse::<u32>().ok())
        .filter(|v| *v > VERSION);
    Err(match newer {
        Some(version) => format!(
            "this is a baseline of version {version}, and this Ascribe reads version {VERSION}; update Ascribe"
        ),
        None => format!("the first line is `{HEADER}`, which names the file's format"),
    })
}

/// The identity an entry's line names: `check fingerprint order file`.
fn entry(line: &str) -> Result<Identity, String> {
    let shape = "an entry is `check fingerprint order file`, separated by one space";
    let mut fields = line.splitn(4, ' ');
    let (Some(check), Some(fingerprint), Some(order), Some(file)) =
        (fields.next(), fields.next(), fields.next(), fields.next())
    else {
        return Err(format!("{shape}, and this line has fewer fields"));
    };
    let kebab = !check.is_empty()
        && check.split('-').all(|w| {
            !w.is_empty()
                && w.chars()
                    .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit())
        });
    if !kebab {
        return Err(format!(
            "{shape}; `{check}` isn't a check's name, such as `nav-label-long`"
        ));
    }
    let hex = fingerprint.len() == 16
        && fingerprint
            .chars()
            .all(|c| c.is_ascii_digit() || ('a'..='f').contains(&c));
    if !hex {
        return Err(format!(
            "{shape}; `{fingerprint}` isn't a fingerprint, 16 lowercase hexadecimal digits"
        ));
    }
    let digits = !order.is_empty() && order.chars().all(|c| c.is_ascii_digit());
    let order = digits
        .then(|| order.parse::<usize>().ok())
        .flatten()
        .filter(|_| order == "0" || !order.starts_with('0'))
        .ok_or_else(|| format!("{shape}; `{order}` isn't an order, a whole number from 0"))?;
    if file.is_empty() || file.starts_with(' ') {
        return Err(format!("{shape}, and the file is missing"));
    }
    Ok(Identity {
        check: check.to_owned(),
        file: file.to_owned(),
        fingerprint: fingerprint.to_owned(),
        order,
    })
}

/// The text of a baseline that holds `identities`: the header, the comment,
/// and one line each, sorted by file, then check, then fingerprint, then
/// order, each once. The same identities always give the same text.
pub fn text<'a>(identities: impl IntoIterator<Item = &'a Identity>) -> String {
    let sorted: BTreeSet<(&str, &str, &str, usize)> = identities
        .into_iter()
        .map(|i| {
            (
                i.file.as_str(),
                i.check.as_str(),
                i.fingerprint.as_str(),
                i.order,
            )
        })
        .collect();
    let mut out = format!("{HEADER}\n{COMMENT}");
    for (file, check, fingerprint, order) in sorted {
        out.push_str(&format!("{check} {fingerprint} {order} {file}\n"));
    }
    out
}

/// What [`apply`] did to a check's diagnostics.
#[derive(Debug, Default)]
pub struct Held {
    /// The diagnostics still reported, in their order, then a
    /// `baseline-entry-unmatched` for each entry that matches nothing.
    pub diagnostics: Vec<Diagnostic>,
    /// The advice the baseline holds back, which isn't reported.
    pub held: Vec<Diagnostic>,
}

/// `diagnostics`, a check's, with what `baseline` holds back set aside:
/// the advice whose identity it lists. A warning or an error is never held
/// back. With `unmatched`, each entry no diagnostic matches, whatever its
/// severity, is reported as a `baseline-entry-unmatched` at its line, with
/// the content model's level for it, apart from the entries of the checks
/// `not_run` names, which this check didn't run.
///
/// `diagnostics` are the whole project's, so each one's order among those
/// alike is what `ascribe baseline write` gives it.
pub fn apply(
    project: &Project,
    baseline: &Baseline,
    diagnostics: Vec<Diagnostic>,
    unmatched: bool,
    not_run: &[&str],
) -> Held {
    let listed: HashSet<&Identity> = baseline.entries.iter().map(|e| &e.identity).collect();
    let found = identities(project, &diagnostics);
    let mut matched: HashSet<&Identity> = HashSet::new();
    let mut held = Held::default();
    for (d, identity) in diagnostics.into_iter().zip(&found) {
        let Some(&entry) = listed.get(identity) else {
            held.diagnostics.push(d);
            continue;
        };
        matched.insert(entry);
        if d.severity == Severity::Advice {
            held.held.push(d);
        } else {
            held.diagnostics.push(d);
        }
    }
    if unmatched {
        let stale = baseline
            .entries
            .iter()
            .filter(|e| !matched.contains(&e.identity))
            .filter(|e| !not_run.contains(&e.identity.check.as_str()))
            .map(|e| {
                let at = Location::new(BASELINE_FILE_ID, e.span);
                Diagnostic::from_issue(
                    &Issue::new(diagnostics::BASELINE_ENTRY_UNMATCHED, at)
                        .with_arg("check", &e.identity.check)
                        .with_arg("file", &e.identity.file),
                )
            })
            .collect();
        held.diagnostics
            .extend(apply_levels(&project.model().checks, stale));
    }
    held
}

/// What `ascribe baseline write` makes of a baseline.
#[derive(Debug)]
pub struct Written {
    /// The file's new text.
    pub text: String,
    /// How many entries it has.
    pub entries: usize,
    /// How many files they're in.
    pub files: usize,
    /// How many of them are new.
    pub added: usize,
    /// How many errors were left out, which a baseline never holds.
    pub skipped_errors: usize,
    /// How many warnings were left out, which a baseline never holds.
    pub skipped_warnings: usize,
}

/// `existing` with every advice of `diagnostics`, the whole project's
/// without a baseline, added: what `ascribe baseline write` writes. Its
/// entries are kept, so a finding is put off until it's fixed and pruned.
pub fn write(
    project: &Project,
    existing: Option<&Baseline>,
    diagnostics: &[Diagnostic],
) -> Written {
    let found = identities(project, diagnostics);
    let mut all: BTreeSet<&Identity> = existing
        .map(|b| b.entries.iter().map(|e| &e.identity).collect())
        .unwrap_or_default();
    let before = all.len();
    let mut skipped_errors = 0;
    let mut skipped_warnings = 0;
    for (d, identity) in diagnostics.iter().zip(&found) {
        match d.severity {
            Severity::Advice => {
                all.insert(identity);
            }
            Severity::Warning => skipped_warnings += 1,
            Severity::Error => skipped_errors += 1,
        }
    }
    Written {
        text: text(all.iter().copied()),
        entries: all.len(),
        files: all.iter().map(|i| &i.file).collect::<HashSet<_>>().len(),
        added: all.len() - before,
        skipped_errors,
        skipped_warnings,
    }
}

/// What `ascribe baseline prune` makes of a baseline.
#[derive(Debug)]
pub struct Pruned {
    /// The file's new text.
    pub text: String,
    /// How many entries it has.
    pub entries: usize,
    /// How many files they're in.
    pub files: usize,
    /// How many entries were removed.
    pub removed: usize,
}

/// `existing` without the entries that match none of `diagnostics`, the
/// whole project's without a baseline: what `ascribe baseline prune`
/// writes.
pub fn prune(project: &Project, existing: &Baseline, diagnostics: &[Diagnostic]) -> Pruned {
    let found: HashSet<Identity> = identities(project, diagnostics).into_iter().collect();
    let all: BTreeSet<&Identity> = existing.entries.iter().map(|e| &e.identity).collect();
    let kept: Vec<&Identity> = all.iter().copied().filter(|i| found.contains(*i)).collect();
    Pruned {
        text: text(kept.iter().copied()),
        entries: kept.len(),
        files: kept.iter().map(|i| &i.file).collect::<HashSet<_>>().len(),
        removed: all.len() - kept.len(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn identity(check: &str, file: &str, fingerprint: &str, order: usize) -> Identity {
        Identity {
            check: check.to_owned(),
            file: file.to_owned(),
            fingerprint: fingerprint.to_owned(),
            order,
        }
    }

    #[test]
    fn the_text_is_sorted_by_file_then_check_then_fingerprint_then_order() {
        let a = identity(
            "link-text-generic",
            "guides/install.md",
            "91c04e2d7b5a6f18",
            1,
        );
        let b = identity(
            "description-too-long",
            "guides/install.md",
            "3f9a2c1b9e0d4a77",
            0,
        );
        let c = identity("description-too-long", "a b.md", "3f9a2c1b9e0d4a77", 10);
        let d = identity("description-too-long", "a b.md", "3f9a2c1b9e0d4a77", 2);
        let text = text([&a, &b, &c, &d, &a]);
        assert_eq!(
            text,
            "# ascribe baseline 1\n\
             # Advice put off for now: ascribe baseline write adds, ascribe baseline prune removes.\n\
             # check fingerprint order file\n\
             description-too-long 3f9a2c1b9e0d4a77 2 a b.md\n\
             description-too-long 3f9a2c1b9e0d4a77 10 a b.md\n\
             description-too-long 3f9a2c1b9e0d4a77 0 guides/install.md\n\
             link-text-generic 91c04e2d7b5a6f18 1 guides/install.md\n"
        );
        let read = Baseline::parse(&text).map(|b| b.entries.len());
        assert_eq!(read, Ok(4));
    }

    #[test]
    fn comments_blank_lines_and_crlf_are_read() {
        let text = "# ascribe baseline 1\r\n\r\n# a note\r\n  \r\nnav-label-long 0123456789abcdef 0 docs/a page.md\r\n";
        let baseline = Baseline::parse(text).map_err(|e| e.to_string());
        let entries = baseline.map(|b| b.entries);
        let Ok([entry]) = entries.as_deref() else {
            panic!("{entries:?}");
        };
        assert_eq!(
            entry.identity,
            identity("nav-label-long", "docs/a page.md", "0123456789abcdef", 0)
        );
        assert_eq!(
            &text[entry.span.range()],
            "nav-label-long 0123456789abcdef 0 docs/a page.md"
        );
    }

    #[test]
    fn a_line_that_doesnt_parse_is_named() {
        let wrong = |text: &str| Baseline::parse(text).map_err(|e| e.to_string());
        let ok = "# ascribe baseline 1\n";
        assert_eq!(
            wrong(""),
            Err("ascribe.baseline:1: the first line is `# ascribe baseline 1`, which names the file's format".to_owned())
        );
        assert_eq!(
            wrong("nav-label-long 0123456789abcdef 0 a.md\n"),
            Err("ascribe.baseline:1: the first line is `# ascribe baseline 1`, which names the file's format".to_owned())
        );
        assert_eq!(
            wrong("# ascribe baseline 2\n"),
            Err("ascribe.baseline:1: this is a baseline of version 2, and this Ascribe reads version 1; update Ascribe".to_owned())
        );
        let line_2 = |line: &str| wrong(&format!("{ok}{line}\n"));
        for (line, says) in [
            ("nav-label-long 0123456789abcdef 0", "fewer fields"),
            (
                "nav-label-long  0123456789abcdef 0 a.md",
                "isn't a fingerprint",
            ),
            ("Nav-Label 0123456789abcdef 0 a.md", "isn't a check's name"),
            (
                "nav-label-long 0123456789ABCDEF 0 a.md",
                "isn't a fingerprint",
            ),
            (
                "nav-label-long 0123456789abcde 0 a.md",
                "isn't a fingerprint",
            ),
            ("nav-label-long 0123456789abcdef -1 a.md", "isn't an order"),
            ("nav-label-long 0123456789abcdef 01 a.md", "isn't an order"),
            ("nav-label-long 0123456789abcdef 0 ", "the file is missing"),
            (
                "nav-label-long 0123456789abcdef 0  a.md",
                "the file is missing",
            ),
        ] {
            let got = line_2(line);
            assert!(
                got.as_ref()
                    .is_err_and(|e| e.starts_with("ascribe.baseline:2: ") && e.contains(says)),
                "{line:?}: {got:?}"
            );
        }
    }
}
