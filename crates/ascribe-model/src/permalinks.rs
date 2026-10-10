//! `ascribe.permalinks` (SPEC §7.5): every permalink a project has declared,
//! each with where it's declared, committed with the docs so that a
//! permalink that goes away is noticed.
//!
//! Ascribe writes the list ([`PermalinkList::to_toml`]) when asked to bring
//! it up to date, and reads it ([`PermalinkList::parse`]) whenever it checks
//! a project. A project without one isn't told about its permalinks.

use std::collections::BTreeMap;
use std::fmt::Write as _;

use ascribe_core::{FileId, Issue, Location, Span, diagnostics};
use toml::de::{DeTable, DeValue};

use crate::toml_util::{describe, entries, sp};

/// The list's file name, beside `ascribe.toml`.
pub const PERMALINKS_FILE: &str = "ascribe.permalinks";

/// The list's format, its `version`.
pub const PERMALINKS_VERSION: i64 = 1;

/// What the list records.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PermalinkList {
    /// Each permalink, by name.
    pub permalinks: BTreeMap<String, ListedPermalink>,
}

/// A permalink's entry.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ListedPermalink {
    /// Where it's declared: the content path of the file it's written in,
    /// and `#` and the id for a heading, as `reference/cli.md#cli-check`.
    pub place: String,
    /// The entry's key, for reporting.
    pub span: Span,
}

/// What [`PermalinkList::synced`] changed.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Synced {
    /// The list as it is now.
    pub list: PermalinkList,
    /// The names it gained, in order.
    pub added: Vec<String>,
    /// The names whose place changed, each with the place it had, in order.
    pub moved: Vec<(String, String)>,
    /// The names it keeps that nothing declares any more, in order.
    pub removed: Vec<String>,
}

impl PermalinkList {
    /// Reads a list's text. Issues are `permalinks-list-invalid`, located in
    /// `file`; with any, the list isn't used.
    ///
    /// # Errors
    ///
    /// Every problem found, each at the value it's about.
    pub fn parse(text: &str, file: FileId) -> Result<PermalinkList, Vec<Issue>> {
        let mut reader = Reader {
            file,
            issues: Vec::new(),
        };
        let list = reader.list(text);
        if reader.issues.is_empty() {
            Ok(list)
        } else {
            Err(reader.issues)
        }
    }

    /// The list brought up to date with what the content declares: each
    /// name with the places it's declared, the first being where a new or
    /// moved entry points. A name not in the list is added; an entry whose
    /// place isn't one of its name's moves to the first; a name nothing
    /// declares stays, since removing a permalink is never done for you.
    pub fn synced(&self, declared: &BTreeMap<String, Vec<String>>) -> Synced {
        let mut out = Synced::default();
        for (name, entry) in &self.permalinks {
            if !declared.contains_key(name) {
                out.removed.push(name.clone());
                out.list.permalinks.insert(
                    name.clone(),
                    ListedPermalink {
                        place: entry.place.clone(),
                        span: Span::empty(0),
                    },
                );
            }
        }
        for (name, places) in declared {
            let Some(first) = places.first() else {
                continue;
            };
            let place = match self.permalinks.get(name) {
                Some(entry) if places.contains(&entry.place) => entry.place.clone(),
                Some(entry) => {
                    out.moved.push((name.clone(), entry.place.clone()));
                    first.clone()
                }
                None => {
                    out.added.push(name.clone());
                    first.clone()
                }
            };
            out.list.permalinks.insert(
                name.clone(),
                ListedPermalink {
                    place,
                    span: Span::empty(0),
                },
            );
        }
        out
    }

    /// The list as Ascribe writes it: one line for each name, in order, and
    /// the same text for the same list on every platform.
    pub fn to_toml(&self) -> String {
        let mut out =
            String::from("# Written by Ascribe. Don't edit it: run `ascribe permalinks sync`.\n");
        let _ = writeln!(out, "version = {PERMALINKS_VERSION}");
        out.push_str("\n[permalinks]\n");
        for (name, entry) in &self.permalinks {
            let _ = writeln!(out, "{} = {}", key(name), quoted(&entry.place));
        }
        out
    }
}

/// A key: bare when TOML allows it, which a name with a `.` it doesn't.
fn key(name: &str) -> String {
    let bare = !name.is_empty()
        && name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_'));
    if bare { name.to_owned() } else { quoted(name) }
}

/// A TOML basic string.
fn quoted(text: &str) -> String {
    let mut out = String::with_capacity(text.len() + 2);
    out.push('"');
    for c in text.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            c if c.is_control() => {
                let _ = write!(out, "\\u{:04X}", c as u32);
            }
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

struct Reader {
    file: FileId,
    issues: Vec<Issue>,
}

impl Reader {
    fn problem(&mut self, span: Span, detail: impl Into<String>) {
        self.issues.push(
            Issue::new(
                diagnostics::PERMALINKS_LIST_INVALID,
                Location::new(self.file, span),
            )
            .with_arg("detail", detail.into()),
        );
    }

    fn list(&mut self, text: &str) -> PermalinkList {
        let mut list = PermalinkList::default();
        let parsed = match DeTable::parse(text) {
            Ok(parsed) => parsed,
            Err(e) => {
                let span = e
                    .span()
                    .map_or(Span::empty(0), |r| Span::new(r.start, r.end));
                self.problem(span, e.message().trim().to_owned());
                return list;
            }
        };
        let root = parsed.get_ref();
        for (key, span, _) in entries(root) {
            if !matches!(key, "version" | "permalinks") {
                self.problem(span, format!("it has no key `{key}`"));
            }
        }
        match root.get("version") {
            None => self.problem(Span::empty(0), "it has no `version`"),
            Some(v) => match v.get_ref() {
                DeValue::Integer(i) if i.as_str() == PERMALINKS_VERSION.to_string() => {}
                DeValue::Integer(i) => self.problem(
                    sp(v),
                    format!(
                        "it's version {}, and this version of Ascribe reads version {PERMALINKS_VERSION}",
                        i.as_str()
                    ),
                ),
                other => self.problem(
                    sp(v),
                    format!("`version` is {}, not a number", describe(other)),
                ),
            },
        }
        let Some(permalinks) = root.get("permalinks") else {
            return list;
        };
        let DeValue::Table(table) = permalinks.get_ref() else {
            self.problem(
                sp(permalinks),
                format!(
                    "`permalinks` is {}, not a table",
                    describe(permalinks.get_ref())
                ),
            );
            return list;
        };
        for (name, span, value) in entries(table) {
            if !ascribe_core::is_id(name) {
                self.problem(
                    span,
                    format!(
                        "`{name}` isn't a permalink's name: letters, digits, hyphens, underscores, and periods"
                    ),
                );
                continue;
            }
            match value.get_ref() {
                DeValue::String(place) if is_place(place) => {
                    list.permalinks.insert(
                        name.to_owned(),
                        ListedPermalink {
                            place: place.to_string(),
                            span,
                        },
                    );
                }
                DeValue::String(place) => self.problem(
                    sp(value),
                    format!(
                        "the place of `{name}`, `{place}`, isn't a file's path ending in `.md`, with `#` and an id for a heading"
                    ),
                ),
                other => self.problem(
                    sp(value),
                    format!(
                        "the place of `{name}` is {}, not a string",
                        describe(other)
                    ),
                ),
            }
        }
        list
    }
}

/// A place as the list writes it: a content path ending in `.md`, and
/// optionally `#` and an id.
fn is_place(text: &str) -> bool {
    let (path, id) = match text.split_once('#') {
        Some((path, id)) => (path, Some(id)),
        None => (text, None),
    };
    let path_ok = path.len() > 3
        && path.ends_with(".md")
        && !path.starts_with('/')
        && !path.split('/').any(|part| part.is_empty() || part == "..");
    path_ok && id.is_none_or(ascribe_core::is_id)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn declared(pairs: &[(&str, &[&str])]) -> BTreeMap<String, Vec<String>> {
        pairs
            .iter()
            .map(|(name, places)| {
                (
                    (*name).to_owned(),
                    places.iter().map(|p| (*p).to_owned()).collect(),
                )
            })
            .collect()
    }

    fn list(text: &str) -> PermalinkList {
        PermalinkList::parse(text, FileId::new(0)).expect("it reads")
    }

    #[test]
    fn written_and_read_back() {
        let synced = PermalinkList::default().synced(&declared(&[
            ("cli-reference", &["reference/cli.md"]),
            ("cli-check", &["reference/cli.md#cli-check"]),
            ("v1.2", &["guides/old versions.md#v1.2"]),
            ("ASC036", &["_generated/diagnostics.md#ASC036"]),
        ]));
        let text = synced.list.to_toml();
        assert_eq!(
            text,
            "# Written by Ascribe. Don't edit it: run `ascribe permalinks sync`.\n\
             version = 1\n\
             \n\
             [permalinks]\n\
             ASC036 = \"_generated/diagnostics.md#ASC036\"\n\
             cli-check = \"reference/cli.md#cli-check\"\n\
             cli-reference = \"reference/cli.md\"\n\
             \"v1.2\" = \"guides/old versions.md#v1.2\"\n"
        );
        assert_eq!(list(&text).to_toml(), text);
        assert_eq!(
            synced.added,
            ["ASC036", "cli-check", "cli-reference", "v1.2"]
        );
    }

    #[test]
    fn syncing_adds_and_moves_and_never_drops() {
        let before = list(
            "version = 1\n[permalinks]\n\
             gone = \"a.md#gone\"\n\
             kept = \"b.md#kept\"\n\
             moved = \"a.md#moved\"\n\
             shared = \"c.md#shared\"\n",
        );
        let synced = before.synced(&declared(&[
            ("kept", &["b.md#kept"]),
            ("moved", &["d.md#moved"]),
            ("new", &["e.md"]),
            ("shared", &["a.md#shared", "c.md#shared"]),
        ]));
        assert_eq!(synced.added, ["new"]);
        assert_eq!(
            synced.moved,
            [("moved".to_owned(), "a.md#moved".to_owned())]
        );
        assert_eq!(synced.removed, ["gone"]);
        let places: Vec<(&str, &str)> = synced
            .list
            .permalinks
            .iter()
            .map(|(n, e)| (n.as_str(), e.place.as_str()))
            .collect();
        assert_eq!(
            places,
            [
                ("gone", "a.md#gone"),
                ("kept", "b.md#kept"),
                ("moved", "d.md#moved"),
                ("new", "e.md"),
                // Still one of the places it's declared, so it stays.
                ("shared", "c.md#shared"),
            ]
        );
    }

    #[test]
    fn problems() {
        let bad = |text: &str| {
            PermalinkList::parse(text, FileId::new(0))
                .expect_err(text)
                .iter()
                .map(|i| i.args.iter().map(|a| a.value.clone()).collect::<String>())
                .collect::<Vec<_>>()
        };
        assert_eq!(
            bad("version = 2\n"),
            ["it's version 2, and this version of Ascribe reads version 1"]
        );
        assert_eq!(bad(""), ["it has no `version`"]);
        assert!(bad("version = ")[0].contains("string"));
        assert!(PermalinkList::parse("version = 1\n", FileId::new(0)).is_ok());
        assert_eq!(bad("version = 1\nnames = 1\n"), ["it has no key `names`"]);
        assert_eq!(
            bad("version = 1\npermalinks = 1\n"),
            ["`permalinks` is a number, not a table"]
        );
        assert_eq!(
            bad("version = 1\n[permalinks]\n\"a/b\" = \"a.md\"\n"),
            ["`a/b` isn't a permalink's name: letters, digits, hyphens, underscores, and periods"]
        );
        assert_eq!(
            bad("version = 1\n[permalinks]\na = \"a.txt\"\n"),
            [
                "the place of `a`, `a.txt`, isn't a file's path ending in `.md`, with `#` and an id for a heading"
            ]
        );
        assert_eq!(
            bad("version = 1\n[permalinks]\na = \"../a.md\"\n"),
            [
                "the place of `a`, `../a.md`, isn't a file's path ending in `.md`, with `#` and an id for a heading"
            ]
        );
        assert_eq!(
            bad("version = 1\n[permalinks]\na = true\n"),
            ["the place of `a` is a boolean, not a string"]
        );
    }
}
