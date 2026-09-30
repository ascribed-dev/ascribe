//! The availability-spec parser (SPEC §4.4, Appendix A rule `availability`).
//!
//! An availability spec is what `@available` takes as its line primary
//! (SPEC §3.4), what the `available` frontmatter value holds, and what a
//! feature in the content model declares. This module parses the syntax only:
//! it doesn't know which targets, states, or versions the content model
//! declares. Checking names against the model is [`tessera-model`]'s job for
//! feature specs, and the document checks' job for specs in documents.
//!
//! ```text
//! availability = entry *( OWS "," OWS entry ) / feature-key
//! entry        = target [ RWS detail ]
//! detail       = version / state [ RWS version ] / "(" OWS history OWS ")"
//! history      = state RWS version *( OWS "," OWS state RWS version )
//! ```
//!
//! A lone name, such as `streaming-sync`, matches both `entry` and
//! `feature-key`. The parser doesn't decide which it is; it returns one entry
//! with no detail, and [`AvailabilitySpec::bare_name`] tells a resolver that
//! the spec could be a feature key.
//!
//! Every node carries a [`Span`] in the coordinates the caller chose, by
//! passing the byte offset of the text in its file.
//!
//! [`tessera-model`]: https://docs.rs/tessera-model

use std::cmp::Ordering;
use std::fmt;

use crate::Span;

/// A name-word: a target, a state, or a feature key (SPEC Appendix A).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Name {
    /// The name, spelled as written. Names are case-sensitive.
    pub text: String,
    /// Where it is written.
    pub span: Span,
}

/// A version: numbers separated by dots (SPEC Appendix A rule `version`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Version {
    /// The version, spelled as written (`3.4`).
    pub text: String,
    /// The numeric components (`[3, 4]`).
    pub components: Vec<u64>,
    /// Where it is written.
    pub span: Span,
}

impl Version {
    /// Compares two versions under the `numeric` versioning scheme:
    /// component by component from the left, numerically, with missing
    /// trailing components treated as `0`. `3.4` equals `3.4.0`, and `3.10`
    /// is later than `3.9`.
    pub fn compare(&self, other: &Version) -> Ordering {
        compare_components(&self.components, &other.components)
    }
}

/// Compares two component lists under the `numeric` version scheme.
pub fn compare_components(a: &[u64], b: &[u64]) -> Ordering {
    let len = a.len().max(b.len());
    for i in 0..len {
        let x = a.get(i).copied().unwrap_or(0);
        let y = b.get(i).copied().unwrap_or(0);
        match x.cmp(&y) {
            Ordering::Equal => {}
            other => return other,
        }
    }
    Ordering::Equal
}

/// One step of a history: a state and the version where it begins.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HistoryStep {
    /// The lifecycle state.
    pub state: Name,
    /// The version where the state begins.
    pub version: Version,
    /// The whole step, from the state to the version.
    pub span: Span,
}

/// What follows a target.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Detail {
    /// Nothing: generally available.
    None,
    /// A bare version: generally available since that version.
    Version(Version),
    /// A state, and optionally the version where it begins.
    State {
        /// The lifecycle state.
        state: Name,
        /// Where the state begins. Absent for a versionless target's state.
        version: Option<Version>,
    },
    /// A parenthesized history: each state lasts until the next begins.
    History(Vec<HistoryStep>),
}

/// A target and its lifecycle.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Entry {
    /// A dimension value, or a dimension name standing for all its values.
    pub target: Name,
    /// The lifecycle after the target.
    pub detail: Detail,
    /// The whole entry.
    pub span: Span,
}

/// A parsed availability spec.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AvailabilitySpec {
    /// The entries, in the order written. Never empty.
    pub entries: Vec<Entry>,
    /// The spec, without surrounding whitespace.
    pub span: Span,
}

impl AvailabilitySpec {
    /// The name, when the whole spec is one bare name. Such a spec is a
    /// feature key if the content model declares one by that name, and a
    /// target that is generally available otherwise (SPEC §4.4).
    pub fn bare_name(&self) -> Option<&Name> {
        match self.entries.as_slice() {
            [
                Entry {
                    target,
                    detail: Detail::None,
                    ..
                },
            ] => Some(target),
            _ => None,
        }
    }
}

/// A syntax error in an availability spec.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AvailabilityError {
    /// The offending text (or the empty span where something is missing).
    pub span: Span,
    /// What is wrong, as the `{detail}` of the `available-syntax` and
    /// `model-availability-syntax` messages.
    pub detail: String,
}

impl fmt::Display for AvailabilityError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.detail)
    }
}

impl std::error::Error for AvailabilityError {}

/// Parses an availability spec.
///
/// `offset` is the byte offset of `text` in its file, added to every span.
/// Surrounding whitespace is ignored. Reports the first error.
pub fn parse_availability(
    text: &str,
    offset: usize,
) -> Result<AvailabilitySpec, AvailabilityError> {
    let mut p = Parser {
        src: text,
        pos: 0,
        offset,
    };
    p.skip_ws();
    let start = p.pos;
    if p.at_end() {
        return Err(p.error_here("the spec is empty; expected a target such as `cloud`"));
    }
    let mut entries = vec![p.entry()?];
    loop {
        p.skip_ws();
        match p.peek() {
            None => break,
            Some(b',') => {
                p.pos += 1;
                p.skip_ws();
                if p.at_end() {
                    return Err(p.error_here("expected a target after `,`"));
                }
                entries.push(p.entry()?);
            }
            Some(_) => return Err(p.unexpected("expected `,` or the end of the spec")),
        }
    }
    let end = entries.last().map_or(start, |e| e.span.end() - offset);
    Ok(AvailabilitySpec {
        entries,
        span: Span::new(offset + start, offset + end),
    })
}

struct Parser<'a> {
    src: &'a str,
    pos: usize,
    offset: usize,
}

impl Parser<'_> {
    fn at_end(&self) -> bool {
        self.pos >= self.src.len()
    }

    fn peek(&self) -> Option<u8> {
        self.src.as_bytes().get(self.pos).copied()
    }

    fn skip_ws(&mut self) {
        while matches!(self.peek(), Some(b' ' | b'\t' | b'\r' | b'\n')) {
            self.pos += 1;
        }
    }

    fn span(&self, start: usize, end: usize) -> Span {
        Span::new(self.offset + start, self.offset + end)
    }

    fn error_here(&self, detail: &str) -> AvailabilityError {
        AvailabilityError {
            span: self.span(self.pos, self.pos),
            detail: detail.to_owned(),
        }
    }

    /// An error at the character under the cursor.
    fn unexpected(&self, expected: &str) -> AvailabilityError {
        let rest = &self.src[self.pos..];
        let width = rest.chars().next().map_or(0, char::len_utf8);
        let shown = rest.chars().next().unwrap_or(' ');
        AvailabilityError {
            span: self.span(self.pos, self.pos + width),
            detail: format!("unexpected `{shown}`; {expected}"),
        }
    }

    fn entry(&mut self) -> Result<Entry, AvailabilityError> {
        let start = self.pos;
        let target = self.name("a target such as `cloud`")?;
        let mut end = self.pos;
        let detail = match self.after_ws() {
            Some(b'(') => {
                let history = self.history()?;
                end = self.pos;
                Detail::History(history)
            }
            Some(b) if b.is_ascii_digit() => {
                let version = self.version()?;
                end = self.pos;
                Detail::Version(version)
            }
            Some(b) if b.is_ascii_alphabetic() => {
                let state = self.name("a lifecycle state")?;
                end = self.pos;
                let version = match self.after_ws() {
                    Some(b) if b.is_ascii_digit() => {
                        let v = self.version()?;
                        end = self.pos;
                        Some(v)
                    }
                    _ => None,
                };
                Detail::State { state, version }
            }
            _ => Detail::None,
        };
        self.pos = end;
        Ok(Entry {
            target,
            detail,
            span: self.span(start, end),
        })
    }

    /// The next byte after whitespace, moving the cursor past the whitespace
    /// only when there was some (a detail needs whitespace before it).
    fn after_ws(&mut self) -> Option<u8> {
        let before = self.pos;
        self.skip_ws();
        if self.pos == before {
            // No whitespace: whatever follows isn't a detail.
            return None;
        }
        match self.peek() {
            Some(b',') | None => {
                self.pos = before;
                None
            }
            other => other,
        }
    }

    fn history(&mut self) -> Result<Vec<HistoryStep>, AvailabilityError> {
        // At "(".
        self.pos += 1;
        self.skip_ws();
        let mut steps = Vec::new();
        loop {
            if self.at_end() {
                return Err(self.error_here("expected `)` to close the history"));
            }
            let start = self.pos;
            let state = self.name("a lifecycle state such as `preview`")?;
            if !matches!(self.peek(), Some(b' ' | b'\t')) {
                return Err(if self.at_end() {
                    self.error_here("expected a version after the state")
                } else {
                    self.unexpected("a history step is a state and the version where it begins, such as `preview 3.3`")
                });
            }
            self.skip_ws();
            if !matches!(self.peek(), Some(b) if b.is_ascii_digit()) {
                return if self.at_end() {
                    Err(self.error_here("expected a version after the state"))
                } else {
                    Err(self.unexpected("a history step is a state and the version where it begins, such as `preview 3.3`"))
                };
            }
            let version = self.version()?;
            steps.push(HistoryStep {
                state,
                span: self.span(start, self.pos),
                version,
            });
            self.skip_ws();
            match self.peek() {
                Some(b',') => {
                    self.pos += 1;
                    self.skip_ws();
                }
                Some(b')') => {
                    self.pos += 1;
                    return Ok(steps);
                }
                None => return Err(self.error_here("expected `)` to close the history")),
                Some(_) => return Err(self.unexpected("expected `,` or `)`")),
            }
        }
    }

    fn name(&mut self, expected: &str) -> Result<Name, AvailabilityError> {
        let start = self.pos;
        match self.peek() {
            Some(b) if b.is_ascii_alphabetic() => {}
            Some(_) => return Err(self.unexpected(&format!("expected {expected}"))),
            None => return Err(self.error_here(&format!("expected {expected}"))),
        }
        while matches!(self.peek(), Some(b) if b.is_ascii_alphanumeric() || b == b'_' || b == b'-')
        {
            self.pos += 1;
        }
        Ok(Name {
            text: self.src[start..self.pos].to_owned(),
            span: self.span(start, self.pos),
        })
    }

    fn version(&mut self) -> Result<Version, AvailabilityError> {
        let start = self.pos;
        let mut components = Vec::new();
        loop {
            let digits_start = self.pos;
            while matches!(self.peek(), Some(b) if b.is_ascii_digit()) {
                self.pos += 1;
            }
            if self.pos == digits_start {
                return Err(if self.at_end() {
                    self.error_here("expected a number after `.` in the version")
                } else {
                    self.unexpected("expected a number after `.` in the version")
                });
            }
            let digits = &self.src[digits_start..self.pos];
            let Ok(n) = digits.parse::<u64>() else {
                return Err(AvailabilityError {
                    span: self.span(digits_start, self.pos),
                    detail: format!("`{digits}` is too large for a version number"),
                });
            };
            components.push(n);
            if self.peek() == Some(b'.') {
                self.pos += 1;
            } else {
                break;
            }
        }
        // A letter or other name character glued to a version (`3.4beta`)
        // isn't a version: the grammar has no pre-release suffix.
        if let Some(b) = self.peek()
            && (b.is_ascii_alphabetic() || b == b'_' || b == b'-')
        {
            return Err(self.unexpected(
                "versions are numbers separated by dots; the spec has no pre-release suffix (use a lifecycle state such as `preview 3.4`)",
            ));
        }
        Ok(Version {
            text: self.src[start..self.pos].to_owned(),
            components,
            span: self.span(start, self.pos),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ok(text: &str) -> AvailabilitySpec {
        parse_availability(text, 0).unwrap_or_else(|e| panic!("`{text}`: {e}"))
    }

    fn err(text: &str) -> AvailabilityError {
        match parse_availability(text, 0) {
            Ok(spec) => panic!("`{text}` parsed: {spec:?}"),
            Err(e) => e,
        }
    }

    fn snippet(text: &str, e: &AvailabilityError) -> String {
        text[e.span.range()].to_owned()
    }

    #[test]
    fn a_bare_target() {
        let spec = ok("cloud");
        assert_eq!(spec.entries.len(), 1);
        assert_eq!(spec.entries[0].target.text, "cloud");
        assert_eq!(spec.entries[0].detail, Detail::None);
        assert_eq!(spec.bare_name().map(|n| n.text.as_str()), Some("cloud"));
    }

    #[test]
    fn a_bare_version_means_ga_since() {
        let spec = ok("self-managed 3.3");
        let Detail::Version(v) = &spec.entries[0].detail else {
            panic!("{spec:?}")
        };
        assert_eq!(v.components, [3, 3]);
        assert_eq!(v.span, Span::new(13, 16));
        assert!(spec.bare_name().is_none());
    }

    #[test]
    fn a_state_with_a_version() {
        let spec = ok("self-managed preview 3.4");
        let Detail::State { state, version } = &spec.entries[0].detail else {
            panic!("{spec:?}")
        };
        assert_eq!(state.text, "preview");
        assert_eq!(state.span, Span::new(13, 20));
        assert_eq!(version.as_ref().map(|v| v.text.as_str()), Some("3.4"));
    }

    #[test]
    fn a_state_alone() {
        let spec = ok("cloud removed");
        let Detail::State { state, version } = &spec.entries[0].detail else {
            panic!("{spec:?}")
        };
        assert_eq!(state.text, "removed");
        assert!(version.is_none());
    }

    #[test]
    fn a_history() {
        let text = "self-managed (preview 3.3, ga 3.5, deprecated 4.0)";
        let spec = ok(text);
        let Detail::History(steps) = &spec.entries[0].detail else {
            panic!("{spec:?}")
        };
        let got: Vec<_> = steps
            .iter()
            .map(|s| (s.state.text.as_str(), s.version.text.as_str()))
            .collect();
        assert_eq!(
            got,
            [("preview", "3.3"), ("ga", "3.5"), ("deprecated", "4.0")]
        );
        assert_eq!(&text[steps[1].span.range()], "ga 3.5");
        assert_eq!(spec.span, Span::new(0, text.len()));
    }

    #[test]
    fn two_targets() {
        let spec = ok("cloud, self-managed preview 3.3");
        assert_eq!(spec.entries.len(), 2);
        assert_eq!(spec.entries[0].target.text, "cloud");
        assert_eq!(spec.entries[1].target.text, "self-managed");
    }

    #[test]
    fn spec_examples_from_the_grammar() {
        for text in [
            "cloud",
            "self-managed 3.3",
            "self-managed preview 3.4",
            "self-managed (preview 3.3, ga 3.5, deprecated 4.0)",
            "cloud, self-managed preview 3.3",
            "cloud,self-managed 3",
            "cloud ,  self-managed   ( preview 3.3 , ga 3.5 )",
            "streaming-sync",
            "deployment",
            "cloud ga",
            "a_b-9 removed 1.2.3.4",
        ] {
            ok(text);
        }
    }

    #[test]
    fn spans_are_offset_and_whitespace_is_ignored() {
        let spec = parse_availability("  cloud, x 1.0  ", 100).unwrap();
        assert_eq!(spec.span, Span::new(102, 114));
        assert_eq!(spec.entries[0].target.span, Span::new(102, 107));
        assert_eq!(spec.entries[1].span, Span::new(109, 114));
    }

    #[test]
    fn malformed_specs_are_rejected_with_spans() {
        let cases: [(&str, &str); 14] = [
            ("", ""),
            ("   ", ""),
            ("cloud,", ""),
            (",cloud", ","),
            ("cloud,,x", ","),
            ("1.2", "1"),
            ("cloud 3.", ""),
            ("cloud 3..4", "."),
            ("cloud 3.4beta", "b"),
            ("cloud (preview 3.3", ""),
            ("cloud ()", ")"),
            ("cloud (preview)", ")"),
            ("cloud (preview 3.3,)", ")"),
            ("cloud ga 3.4 extra", "e"),
        ];
        for (text, at) in cases {
            let e = err(text);
            assert_eq!(snippet(text, &e), at, "`{text}`: {e}");
            assert!(!e.detail.is_empty());
        }
    }

    #[test]
    fn no_pre_release_text_or_ranges_or_negation() {
        for text in [
            "cloud >=3.4",
            "cloud !beta",
            "cloud 3.4-beta",
            "cloud | x",
            "cloud ^3",
        ] {
            err(text);
        }
    }

    #[test]
    fn a_version_glued_to_the_target_is_not_a_detail() {
        let e = err("cloud(preview 3.3)");
        assert_eq!(snippet("cloud(preview 3.3)", &e), "(");
    }

    #[test]
    fn huge_version_numbers_are_an_error_not_a_panic() {
        let text = "cloud 99999999999999999999999";
        let e = err(text);
        assert!(e.detail.contains("too large"));
    }

    #[test]
    fn non_ascii_is_an_error_not_a_panic() {
        let text = "cloud, é";
        let e = err(text);
        assert_eq!(snippet(text, &e), "é");
    }

    #[test]
    fn versions_compare_numerically() {
        let v = |t: &str| match ok(&format!("x {t}")).entries[0].detail.clone() {
            Detail::Version(v) => v,
            other => panic!("{other:?}"),
        };
        assert_eq!(v("3.4").compare(&v("3.4.0")), Ordering::Equal);
        assert_eq!(v("3.10").compare(&v("3.9")), Ordering::Greater);
        assert_eq!(v("4").compare(&v("3.99.1")), Ordering::Greater);
        assert_eq!(v("3.04").compare(&v("3.4")), Ordering::Equal);
        assert_eq!(v("3.3").compare(&v("3.3.1")), Ordering::Less);
    }
}
