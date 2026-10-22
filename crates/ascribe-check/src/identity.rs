//! A finding's identity: what makes a diagnostic of one run the same
//! finding as one of another, so a report can say what's new since the
//! last, and a baseline can list the findings a project has (decision
//! #787).
//!
//! A finding is the same as another when it has the same:
//!
//! 1. **check**, its slug;
//! 2. **file**, from the project's root, with `/`;
//! 3. **fingerprint**, [`fingerprint`]: a hash of the source text of its
//!    span, or of its line's when the span is empty;
//! 4. **order** among the findings with the first three alike, from 0, in
//!    the file's order.
//!
//! So a finding keeps its identity when lines are added above it, and is a
//! new one when the text it's about is edited. Its place, its message, and
//! its builds aren't part of it: a finding in several builds is one, as
//! `ascribe check` reports it once.

use std::collections::BTreeMap;
use std::fmt::Write as _;

use ascribe_core::Span;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::{Diagnostic, Project};

/// How many hexadecimal digits of the hash a fingerprint keeps.
const DIGITS: usize = 16;

/// What tells one finding from another, across runs.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Identity {
    /// The check's slug, such as `link-target-missing`.
    pub check: String,
    /// The file, relative to the project's root, `/`-separated, as a
    /// report shows it.
    pub file: String,
    /// [`fingerprint`] of the finding's span in the file.
    pub fingerprint: String,
    /// Its place among the findings with the same check, file, and
    /// fingerprint, from 0, in the file's order.
    pub order: usize,
}

/// The fingerprint of `span` in `text`: the first 16 hexadecimal digits,
/// lowercase, of the SHA-256 of the span's text, or of its line's text when
/// the span is empty, with each run of whitespace read as one space and
/// the ends trimmed. Whitespace is what Unicode calls it, so a line's end,
/// `\r` or `\n`, is whitespace too. A span outside the text, or that cuts a
/// character, is read as empty text.
pub fn fingerprint(text: &str, span: Span) -> String {
    let source = if span.is_empty() {
        line_at(text, span.start())
    } else {
        text.get(span.range()).unwrap_or_default()
    };
    let folded = source.split_whitespace().collect::<Vec<_>>().join(" ");
    let mut out = String::with_capacity(DIGITS);
    for byte in Sha256::digest(folded.as_bytes()).iter().take(DIGITS / 2) {
        let _ = write!(out, "{byte:02x}");
    }
    out
}

/// The line of `text` that `offset` is on, without its end.
fn line_at(text: &str, offset: usize) -> &str {
    let Some(before) = text.get(..offset) else {
        return "";
    };
    let start = before.rfind('\n').map_or(0, |i| i + 1);
    let end = text[offset..].find('\n').map_or(text.len(), |i| offset + i);
    &text[start..end]
}

/// The identity of each of `diagnostics`, in their order, in `project`.
pub fn identities(project: &Project, diagnostics: &[Diagnostic]) -> Vec<Identity> {
    let found = diagnostics
        .iter()
        .map(|d| {
            let file = d.location.file;
            let text = match project.file(file) {
                Some(entry) => Some(entry.text.to_owned()),
                None => project.code_file(file).map(|code| code.text.to_string()),
            };
            let identity = Identity {
                check: d.slug.to_string(),
                file: project
                    .display_path(file)
                    .unwrap_or_else(|| format!("<file {}>", file.index())),
                fingerprint: fingerprint(text.as_deref().unwrap_or_default(), d.location.span),
                order: 0,
            };
            (identity, d.location.span)
        })
        .collect();
    ordered(found)
}

/// The identity of each of `diagnostics`, in their order, when all of them
/// are located in one file: `file`, from the project's root with `/`, whose
/// text is `text`. What the editor works out for a file it checks on its
/// own, such as Vale's alerts in a page (#790).
pub fn identities_in(file: &str, text: &str, diagnostics: &[Diagnostic]) -> Vec<Identity> {
    let found = diagnostics
        .iter()
        .map(|d| {
            let identity = Identity {
                check: d.slug.to_string(),
                file: file.to_owned(),
                fingerprint: fingerprint(text, d.location.span),
                order: 0,
            };
            (identity, d.location.span)
        })
        .collect();
    ordered(found)
}

/// `found`'s identities, each with its order among those alike.
fn ordered(mut found: Vec<(Identity, Span)>) -> Vec<Identity> {
    // Each group of alike findings, in the file's order: by where each
    // starts and ends, then in the order given.
    let mut groups: BTreeMap<(String, String, String), Vec<usize>> = BTreeMap::new();
    for (i, (identity, _)) in found.iter().enumerate() {
        let key = (
            identity.check.clone(),
            identity.file.clone(),
            identity.fingerprint.clone(),
        );
        groups.entry(key).or_default().push(i);
    }
    for mut group in groups.into_values() {
        group.sort_by_key(|i| (found[*i].1.start(), found[*i].1.end(), *i));
        for (order, i) in group.into_iter().enumerate() {
            found[i].0.order = order;
        }
    }
    found.into_iter().map(|(identity, _)| identity).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The fingerprint of `needle`'s first place in `text`.
    fn of(text: &str, needle: &str) -> String {
        let start = text.find(needle).unwrap_or(text.len());
        fingerprint(text, Span::new(start, start + needle.len()))
    }

    #[test]
    fn it_is_16_hex_digits_of_the_sha_256_of_the_folded_text() {
        // `printf 'a b' | sha256sum` starts c8687a08aa5d6ed2.
        assert_eq!(of("a b", "a b"), "c8687a08aa5d6ed2");
        assert_eq!(of("x", "x").len(), 16);
    }

    #[test]
    fn whitespace_runs_read_as_one_space_and_the_ends_are_trimmed() {
        let one = of("See [the docs](a.md).", "See [the docs](a.md)");
        assert_eq!(
            of("  See  [the\n  docs](a.md).", "See  [the\n  docs](a.md)"),
            one
        );
        assert_eq!(
            of(
                "\t See [the docs](a.md) \r\n",
                "\t See [the docs](a.md) \r\n"
            ),
            one
        );
        assert_ne!(of("See [the doc](a.md)", "See [the doc](a.md)"), one);
        assert_ne!(of("See[the docs](a.md)", "See[the docs](a.md)"), one);
    }

    #[test]
    fn an_empty_span_is_its_line() {
        let text = "# Title\n\nThe  line it's on.\r\nAnother.\n";
        let at = text.find("line").unwrap_or_default();
        let line = of(text, "The  line it's on.");
        assert_eq!(fingerprint(text, Span::empty(at)), line);
        // At the line's start, and at its end.
        let start = text.find("The").unwrap_or_default();
        assert_eq!(fingerprint(text, Span::empty(start)), line);
        let end = text.find('\r').unwrap_or_default();
        assert_eq!(fingerprint(text, Span::empty(end)), line);
        // The last line, without an end, and a span outside the text.
        assert_eq!(fingerprint("one\ntwo", Span::empty(5)), of("two", "two"));
        assert_eq!(fingerprint("one", Span::new(2, 9)), of("", ""));
    }
}
