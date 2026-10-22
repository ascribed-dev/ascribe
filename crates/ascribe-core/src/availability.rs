//! The availability-spec parser (SPEC §4.4, Appendix A rule `availability`).
//!
//! An availability spec is what `@available` takes as its line primary
//! (SPEC §3.4), what the `available` frontmatter value holds, and what a
//! feature in the content model declares. This module parses the syntax only:
//! it doesn't know which targets, states, or versions the content model
//! declares. Checking names against the model is [`ascribe-model`]'s job for
//! feature specs, and the document checks' job for specs in documents.
//!
//! ```text
//! availability = entry *( OWS "," OWS entry ) / feature-key
//! entry        = target [ RWS detail ]
//! detail       = when / state [ RWS when ] / "(" OWS history OWS ")"
//! history      = state RWS when *( OWS "," OWS state RWS when )
//! when         = version / %s"planned"
//! ```
//!
//! `planned` is the version word for the target's planned line. The parser
//! returns it as a [`Version`] whose [`Version::is_planned`] is true and
//! which has no components; the content model says which line it is
//! (`ascribe_model::ContentModel::resolve_planned`), and nothing compares it
//! before then.
//!
//! A lone name, such as `streaming-sync`, matches both `entry` and
//! `feature-key`. The parser doesn't decide which it is; it returns one entry
//! with no detail, and [`AvailabilitySpec::bare_name`] tells a resolver that
//! the spec could be a feature key.
//!
//! Every node carries a [`Span`] in the coordinates the caller chose, by
//! passing the byte offset of the text in its file.
//!
//! [`ascribe-model`]: https://docs.rs/ascribe-model

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

/// The version word for a target's planned line (SPEC §4.4).
pub const PLANNED: &str = "planned";

impl Version {
    /// The version word `planned`, written at `span`.
    pub fn planned(span: Span) -> Version {
        Version {
            text: PLANNED.to_owned(),
            components: Vec::new(),
            span,
        }
    }

    /// Whether this is the version word `planned`, not yet replaced by the
    /// line it names. It has no components, so it compares as `0` until
    /// then.
    pub fn is_planned(&self) -> bool {
        self.components.is_empty() && self.text == PLANNED
    }

    /// Compares two versions under the `numeric` versioning scheme:
    /// component by component from the left, numerically, with missing
    /// trailing components treated as `0`. `3.4` equals `3.4.0`, and `3.10`
    /// is later than `3.9`.
    pub fn compare(&self, other: &Version) -> Ordering {
        compare_components(&self.components, &other.components)
    }
}

/// Compares two component lists under the `numeric` version scheme.
pub(crate) fn compare_components(a: &[u64], b: &[u64]) -> Ordering {
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
    /// Every version the spec names, in the order written, `planned` among
    /// them.
    pub fn versions(&self) -> impl Iterator<Item = &Version> {
        self.entries.iter().flat_map(|e| match &e.detail {
            Detail::None | Detail::State { version: None, .. } => Vec::new(),
            Detail::Version(v)
            | Detail::State {
                version: Some(v), ..
            } => vec![v],
            Detail::History(steps) => steps.iter().map(|s| &s.version).collect(),
        })
    }

    /// Whether the spec writes `planned` anywhere.
    pub fn has_planned(&self) -> bool {
        self.versions().any(Version::is_planned)
    }

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

/// `spec` with what it says of `target` at or before `oldest` left out,
/// where that changes nothing after it: the simplest spec that means the
/// same at every version from `oldest` on. `None` when nothing changes.
///
/// A version at or before `oldest` goes, and the state stays:
/// `self-managed 3.2` becomes `self-managed`, and `self-managed deprecated
/// 3.2` becomes `self-managed deprecated`. A history drops the steps before
/// the one in effect at `oldest`, which keeps its version, and becomes that
/// step's state when no step starts after `oldest`: `self-managed (preview
/// 3.3, ga 3.5, deprecated 3.6)` becomes `self-managed (ga 3.5, deprecated
/// 3.6)` with `oldest` 3.5, and `self-managed deprecated` with `oldest`
/// 3.6. `ga` with no version is written as the target alone. Other targets'
/// entries are kept as they are.
///
/// A line's output shows a spec this way, with `oldest` its oldest live
/// line, and retiring a line rewrites each mark to it.
pub fn without_versions_through(
    spec: &AvailabilitySpec,
    target: &str,
    oldest: &Version,
) -> Option<AvailabilitySpec> {
    // `planned` is newer than every live line, whichever it names.
    let old = |v: &Version| !v.is_planned() && v.compare(oldest).is_le();
    let state_alone = |state: &Name| {
        if state.text == "ga" {
            Detail::None
        } else {
            Detail::State {
                state: state.clone(),
                version: None,
            }
        }
    };
    let mut changed = false;
    let entries = spec
        .entries
        .iter()
        .map(|e| {
            let detail = match &e.detail {
                _ if e.target.text != target => None,
                Detail::Version(v) if old(v) => Some(Detail::None),
                Detail::State {
                    state,
                    version: Some(v),
                } if old(v) => Some(state_alone(state)),
                Detail::History(steps) => match steps.iter().rposition(|s| old(&s.version)) {
                    Some(k) if k + 1 == steps.len() => Some(state_alone(&steps[k].state)),
                    Some(k) if k > 0 => Some(Detail::History(steps[k..].to_vec())),
                    _ => None,
                },
                _ => None,
            };
            match detail {
                Some(detail) => {
                    changed = true;
                    Entry {
                        detail,
                        ..e.clone()
                    }
                }
                None => e.clone(),
            }
        })
        .collect();
    changed.then_some(AvailabilitySpec {
        entries,
        span: spec.span,
    })
}

/// A spec written out as an author writes it (SPEC §4.4): entries separated
/// by `, `, and a history's steps too, one space inside each.
pub fn spec_text(spec: &AvailabilitySpec) -> String {
    spec.entries
        .iter()
        .map(entry_text)
        .collect::<Vec<_>>()
        .join(", ")
}

/// One entry written out as [`spec_text`] writes it: `self-managed preview
/// 3.4`, or `self-managed (preview 3.3, ga 3.5)`.
pub fn entry_text(entry: &Entry) -> String {
    let target = &entry.target.text;
    match &entry.detail {
        Detail::None => target.clone(),
        Detail::Version(v) => format!("{target} {}", v.text),
        Detail::State {
            state,
            version: None,
        } => format!("{target} {}", state.text),
        Detail::State {
            state,
            version: Some(v),
        } => format!("{target} {} {}", state.text, v.text),
        Detail::History(steps) => {
            let steps: Vec<String> = steps
                .iter()
                .map(|s| format!("{} {}", s.state.text, s.version.text))
                .collect();
            format!("{target} ({})", steps.join(", "))
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

/// Parses a version alone (SPEC Appendix A rule `version`), such as
/// a line in `[releases]`. `offset` is the byte offset of `text` in its file.
/// The whole text must be the version, with no surrounding whitespace.
pub fn parse_version(text: &str, offset: usize) -> Result<Version, AvailabilityError> {
    let mut p = Parser {
        src: text,
        pos: 0,
        offset,
    };
    let version = p.version()?;
    if p.at_end() {
        Ok(version)
    } else {
        Err(p.unexpected("expected the end of the version"))
    }
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
                if state.text == PLANNED {
                    Detail::Version(Version::planned(state.span))
                } else {
                    let version = match self.after_ws() {
                        Some(b) if b.is_ascii_digit() => {
                            let v = self.version()?;
                            end = self.pos;
                            Some(v)
                        }
                        Some(b) if b.is_ascii_alphabetic() => self.planned_word().inspect(|_| {
                            end = self.pos;
                        }),
                        _ => None,
                    };
                    Detail::State { state, version }
                }
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
            let version = match self.peek() {
                Some(b) if b.is_ascii_digit() => self.version()?,
                Some(b) if b.is_ascii_alphabetic() => match self.planned_word() {
                    Some(v) => v,
                    None => return Err(self.unexpected("a history step is a state and the version where it begins, such as `preview 3.3`")),
                },
                _ => {
                    return if self.at_end() {
                        Err(self.error_here("expected a version after the state"))
                    } else {
                        Err(self.unexpected("a history step is a state and the version where it begins, such as `preview 3.3`"))
                    };
                }
            };
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

    /// The version word `planned` at the cursor, moving past it; `None`, with
    /// the cursor where it was, for any other word.
    fn planned_word(&mut self) -> Option<Version> {
        let start = self.pos;
        let name = self.name("").ok()?;
        if name.text == PLANNED {
            Some(Version::planned(name.span))
        } else {
            self.pos = start;
            None
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

    #[test]
    fn a_version_alone() {
        let v = parse_version("0.2.10", 5).expect("a version");
        assert_eq!(v.components, [0, 2, 10]);
        assert_eq!(v.span, Span::new(5, 11));
        assert!(parse_version("0.2 ", 0).is_err());
        assert!(parse_version("v1", 0).is_err());
        assert!(parse_version("1.2beta", 0).is_err());
        assert!(parse_version("", 0).is_err());
    }

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
    fn planned_in_each_place_a_version_goes() {
        let planned = |d: &Detail| match d {
            Detail::Version(v)
            | Detail::State {
                version: Some(v), ..
            } => vec![v.is_planned()],
            Detail::History(steps) => steps.iter().map(|s| s.version.is_planned()).collect(),
            _ => Vec::new(),
        };
        let spec = ok("ascribe planned");
        assert_eq!(planned(&spec.entries[0].detail), [true]);
        let Detail::Version(v) = &spec.entries[0].detail else {
            panic!("{spec:?}")
        };
        assert_eq!(v.span, Span::new(8, 15));
        assert!(spec.bare_name().is_none());

        let spec = ok("cloud, self-managed preview planned");
        assert_eq!(planned(&spec.entries[1].detail), [true]);
        let spec = ok("self-managed (preview 3.3, ga planned)");
        assert_eq!(planned(&spec.entries[0].detail), [false, true]);
        assert_eq!(spec_text(&spec), "self-managed (preview 3.3, ga planned)");
        // A word that isn't `planned` after a state is still an error.
        assert_eq!(snippet("cloud ga later", &err("cloud ga later")), "l");
        assert_eq!(snippet("cloud (ga later)", &err("cloud (ga later)")), "l");
        // `planned` alone is a target, as any bare name is.
        assert_eq!(
            ok("planned").bare_name().map(|n| n.text.as_str()),
            Some("planned")
        );
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

    /// The rows of the table in the reference's section on retiring a
    /// line, with 3.5 the oldest live line.
    #[test]
    fn a_spec_without_the_versions_before_the_window() {
        let oldest = parse_version("3.5", 0).expect("a version");
        for (text, simpler) in [
            ("self-managed 3.4", Some("self-managed")),
            (
                "self-managed deprecated 3.2",
                Some("self-managed deprecated"),
            ),
            (
                "self-managed (preview 3.3, ga 3.5, deprecated 3.6)",
                Some("self-managed (ga 3.5, deprecated 3.6)"),
            ),
            ("cloud, self-managed 3.4", Some("cloud, self-managed")),
            ("self-managed ga 3.1", Some("self-managed")),
            ("self-managed (preview 3.1, ga 3.3)", Some("self-managed")),
            (
                "self-managed (ga 3.0, removed 3.4)",
                Some("self-managed removed"),
            ),
            ("self-managed (preview 3.3, ga 3.6)", None),
            ("self-managed 3.5.1", None),
            ("self-managed 3.6", None),
            ("self-managed", None),
            ("cloud", None),
            ("other 3.1", None),
            ("self-managed planned", None),
            ("self-managed (preview 3.1, ga planned)", None),
            (
                "self-managed (preview 3.1, ga 3.4, deprecated planned)",
                Some("self-managed (ga 3.4, deprecated planned)"),
            ),
        ] {
            let got = without_versions_through(&ok(text), "self-managed", &oldest);
            assert_eq!(got.as_ref().map(spec_text).as_deref(), simpler, "{text}");
        }
    }

    /// The state an entry has at `version`, as SPEC §9.3 reads it: `None`
    /// before its first step.
    fn state_at(entry: &Entry, version: &[u64]) -> Option<String> {
        let started = |v: &Version| compare_components(version, &v.components).is_ge();
        match &entry.detail {
            Detail::None => Some("ga".to_owned()),
            Detail::Version(v) => started(v).then(|| "ga".to_owned()),
            Detail::State { state, version } => version
                .as_ref()
                .is_none_or(started)
                .then(|| state.text.clone()),
            Detail::History(steps) => steps
                .iter()
                .rev()
                .find(|s| started(&s.version))
                .map(|s| s.state.text.clone()),
        }
    }

    proptest::proptest! {
        /// The rewritten spec and the original agree at every version from
        /// the oldest line on, whatever the history.
        #[test]
        fn the_simpler_spec_means_the_same_in_every_remaining_line(
            starts in proptest::collection::btree_set((0u64..6, 0u64..4), 1..5),
            states in proptest::collection::vec(0usize..4, 5),
            kind in 0usize..3,
            oldest in (0u64..7, 0u64..5),
        ) {
            let names = ["ga", "preview", "deprecated", "removed"];
            let starts: Vec<_> = starts.into_iter().collect();
            let detail = match kind {
                0 => format!("{}.{}", starts[0].0, starts[0].1),
                1 => format!("{} {}.{}", names[states[0]], starts[0].0, starts[0].1),
                _ => format!(
                    "({})",
                    starts
                        .iter()
                        .zip(&states)
                        .map(|((a, b), s)| format!("{} {a}.{b}", names[*s]))
                        .collect::<Vec<_>>()
                        .join(", ")
                ),
            };
            let text = format!("cloud, self-managed {detail}");
            let spec = ok(&text);
            let oldest = parse_version(&format!("{}.{}", oldest.0, oldest.1), 0).expect("a version");
            let Some(simpler) = without_versions_through(&spec, "self-managed", &oldest) else {
                return Ok(());
            };
            let simpler = ok(&spec_text(&simpler));
            proptest::prop_assert_eq!(&simpler.entries[0].target.text, "cloud");
            for major in 0..8 {
                for minor in 0..6 {
                    for patch in 0..2 {
                        let v = [major, minor, patch];
                        if compare_components(&v, &oldest.components).is_lt() {
                            continue;
                        }
                        proptest::prop_assert_eq!(
                            state_at(&spec.entries[1], &v),
                            state_at(&simpler.entries[1], &v),
                            "{} and {} at {:?}", text, spec_text(&simpler), v
                        );
                    }
                }
            }
        }
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
