//! `ascribe.permalinks` (SPEC §7.5): every permalink a project has declared,
//! each with where it's declared, and each name `ascribe.addresses` moves or
//! retires, committed with the docs so that a permalink that goes away is
//! noticed.
//!
//! Ascribe writes the list ([`PermalinkList::to_toml`]) when asked to bring
//! it up to date, and reads it ([`PermalinkList::parse`]) whenever it checks
//! a project. A project without one isn't told about its permalinks.

use std::collections::BTreeMap;
use std::fmt::Write as _;

use ascribe_core::{FileId, Issue, Location, Span, diagnostics};

use toml::de::{DeTable, DeValue};

use crate::addresses::{End, NAME_RULE, RecordedPermalinks, split_place};
use crate::toml_util::{describe, entries, sp};

/// The list's file name, beside `ascribe.toml`.
pub const PERMALINKS_FILE: &str = "ascribe.permalinks";

/// The list's format, its `version`.
pub const PERMALINKS_VERSION: i64 = 1;

/// What the list records: each name once, in one of its four tables.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PermalinkList {
    /// `[permalinks]`: each permalink the content declares, by name.
    pub permalinks: BTreeMap<String, ListedPermalink>,
    /// `[aliases]`: each name `ascribe.addresses` moves to another
    /// permalink, by name.
    pub aliases: BTreeMap<String, ListedAlias>,
    /// `[moved]`: each name `ascribe.addresses` moves to a page, by name.
    pub moved: BTreeMap<String, ListedMoved>,
    /// `[retired]`: each name `ascribe.addresses` retires, by name.
    pub retired: BTreeMap<String, ListedRetired>,
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

/// An alias's entry.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ListedAlias {
    /// The name it leads to.
    pub target: String,
    /// The entry's key, for reporting.
    pub span: Span,
}

/// A moved permalink's entry.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ListedMoved {
    /// Where it leads: a page's content path, and `#` and an id for a
    /// heading.
    pub place: String,
    /// The entry's key, for reporting.
    pub span: Span,
}

/// A retired permalink's entry.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ListedRetired {
    /// Why it was retired.
    pub reason: String,
    /// The entry's key, for reporting.
    pub span: Span,
}

/// What the project has now, which the list should record: each name the
/// content declares, with the places it's declared, the first being where a
/// new or moved entry points; and the names `ascribe.addresses` moves or
/// retires that the content doesn't declare too.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Current {
    /// Each declared name, with its places.
    pub declared: BTreeMap<String, Vec<String>>,
    /// Each name moved to another permalink, with the name it's moved to.
    pub aliases: BTreeMap<String, String>,
    /// Each name moved to a page, with the place.
    pub moved: BTreeMap<String, String>,
    /// Each retired name, with its reason: one `[retired]` has, or one
    /// moved to a name that's retired.
    pub retired: BTreeMap<String, String>,
}

impl Current {
    /// What the project has: `declared`, and what `recorded` moves or
    /// retires. A name the content declares is a permalink, whatever
    /// `ascribe.addresses` says of it (`addresses-entry-live` reports
    /// that), and a name whose entries lead back to it is none.
    pub fn new(declared: BTreeMap<String, Vec<String>>, recorded: &RecordedPermalinks) -> Current {
        let mut out = Current {
            declared,
            ..Current::default()
        };
        for entry in &recorded.names {
            if out.declared.contains_key(&entry.name) {
                continue;
            }
            let name = entry.name.clone();
            match &entry.end {
                End::Retired(reason) => {
                    out.retired.insert(name, reason.clone());
                }
                End::Cycle => {}
                End::Place(_) | End::Permalink(_) => {
                    if let Some(target) = entry.alias_of() {
                        out.aliases.insert(name, target.to_owned());
                    } else if let Some(place) = entry.moved_to() {
                        out.moved.insert(name, place.to_owned());
                    }
                }
            }
        }
        out
    }

    /// Whether the project has `name`, in any of the four ways.
    pub fn has(&self, name: &str) -> bool {
        self.declared.contains_key(name)
            || self.aliases.contains_key(name)
            || self.moved.contains_key(name)
            || self.retired.contains_key(name)
    }
}

/// What [`PermalinkList::synced`] changed.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Synced {
    /// The list as it is now.
    pub list: PermalinkList,
    /// The permalinks it gained, in order.
    pub added: Vec<String>,
    /// The names whose place changed, each with the place it had, in order.
    pub moved: Vec<(String, String)>,
    /// The aliases it gained, or whose target changed, in order.
    pub aliased: Vec<String>,
    /// The names moved to a page it gained, or whose place changed, in
    /// order.
    pub moved_to: Vec<String>,
    /// The retired permalinks it gained, or whose reason changed, in order.
    pub retired: Vec<String>,
    /// The names it keeps that the project doesn't have any more, in
    /// order.
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
            seen: Vec::new(),
        };
        let list = reader.list(text);
        if reader.issues.is_empty() {
            Ok(list)
        } else {
            Err(reader.issues)
        }
    }

    /// Whether the list has `name`, in any of its tables.
    pub fn has(&self, name: &str) -> bool {
        self.span_of(name).is_some()
    }

    /// Where the list has `name`, for messages.
    pub fn span_of(&self, name: &str) -> Option<Span> {
        self.permalinks
            .get(name)
            .map(|e| e.span)
            .or_else(|| self.aliases.get(name).map(|e| e.span))
            .or_else(|| self.moved.get(name).map(|e| e.span))
            .or_else(|| self.retired.get(name).map(|e| e.span))
    }

    /// The list brought up to date with what the project has. A declared
    /// name not in `[permalinks]` is added; an entry whose place isn't one
    /// of its name's moves to the first; a name moved or retired is
    /// recorded as `ascribe.addresses` has it; and a name the project doesn't
    /// have stays where it is, since removing a permalink is never done for
    /// you.
    pub fn synced(&self, current: &Current) -> Synced {
        let mut out = Synced::default();
        let kept = |name: &String| !current.has(name);
        for (name, entry) in self.permalinks.iter().filter(|(n, _)| kept(n)) {
            out.removed.push(name.clone());
            out.list.permalinks.insert(
                name.clone(),
                ListedPermalink {
                    place: entry.place.clone(),
                    span: Span::empty(0),
                },
            );
        }
        for (name, entry) in self.aliases.iter().filter(|(n, _)| kept(n)) {
            out.removed.push(name.clone());
            out.list.aliases.insert(
                name.clone(),
                ListedAlias {
                    target: entry.target.clone(),
                    span: Span::empty(0),
                },
            );
        }
        for (name, entry) in self.moved.iter().filter(|(n, _)| kept(n)) {
            out.removed.push(name.clone());
            out.list.moved.insert(
                name.clone(),
                ListedMoved {
                    place: entry.place.clone(),
                    span: Span::empty(0),
                },
            );
        }
        for (name, entry) in self.retired.iter().filter(|(n, _)| kept(n)) {
            out.removed.push(name.clone());
            out.list.retired.insert(
                name.clone(),
                ListedRetired {
                    reason: entry.reason.clone(),
                    span: Span::empty(0),
                },
            );
        }
        out.removed.sort();
        for (name, places) in &current.declared {
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
        for (name, target) in &current.aliases {
            if self.aliases.get(name).is_none_or(|e| e.target != *target) {
                out.aliased.push(name.clone());
            }
            out.list.aliases.insert(
                name.clone(),
                ListedAlias {
                    target: target.clone(),
                    span: Span::empty(0),
                },
            );
        }
        for (name, place) in &current.moved {
            if self.moved.get(name).is_none_or(|e| e.place != *place) {
                out.moved_to.push(name.clone());
            }
            out.list.moved.insert(
                name.clone(),
                ListedMoved {
                    place: place.clone(),
                    span: Span::empty(0),
                },
            );
        }
        for (name, reason) in &current.retired {
            if self.retired.get(name).is_none_or(|e| e.reason != *reason) {
                out.retired.push(name.clone());
            }
            out.list.retired.insert(
                name.clone(),
                ListedRetired {
                    reason: reason.clone(),
                    span: Span::empty(0),
                },
            );
        }
        out
    }

    /// The list as Ascribe writes it: one line for each name, in order, and
    /// the same text for the same list on every platform. `[aliases]`,
    /// `[moved]`, and `[retired]` are written only when they have an entry.
    pub fn to_toml(&self) -> String {
        let mut out =
            String::from("# Written by Ascribe. Don't edit it: run `ascribe permalinks sync`.\n");
        let _ = writeln!(out, "version = {PERMALINKS_VERSION}");
        out.push_str("\n[permalinks]\n");
        for (name, entry) in &self.permalinks {
            let _ = writeln!(out, "{} = {}", key(name), quoted(&entry.place));
        }
        if !self.aliases.is_empty() {
            out.push_str("\n[aliases]\n");
            for (name, entry) in &self.aliases {
                let _ = writeln!(out, "{} = {}", key(name), quoted(&entry.target));
            }
        }
        if !self.moved.is_empty() {
            out.push_str("\n[moved]\n");
            for (name, entry) in &self.moved {
                let _ = writeln!(out, "{} = {}", key(name), quoted(&entry.place));
            }
        }
        if !self.retired.is_empty() {
            out.push_str("\n[retired]\n");
            for (name, entry) in &self.retired {
                let _ = writeln!(out, "{} = {}", key(name), quoted(&entry.reason));
            }
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
    /// Each name read, with its first key's span.
    seen: Vec<(String, Span)>,
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
            if !matches!(
                key,
                "version" | "permalinks" | "aliases" | "moved" | "retired"
            ) {
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
        for (name, span, place) in self.table(root, "permalinks") {
            if is_place(&place) {
                list.permalinks
                    .insert(name, ListedPermalink { place, span });
            } else {
                self.problem(
                    span,
                    format!(
                        "the place of `{name}`, `{place}`, isn't a file's path ending in `.md`, with `#` and an id for a heading"
                    ),
                );
            }
        }
        for (name, span, target) in self.table(root, "aliases") {
            if ascribe_core::is_id(&target) {
                list.aliases.insert(name, ListedAlias { target, span });
            } else {
                self.problem(
                    span,
                    format!(
                        "the alias `{name}` leads to `{target}`, which isn't a permalink's name"
                    ),
                );
            }
        }
        for (name, span, place) in self.table(root, "moved") {
            if is_place(&place) {
                list.moved.insert(name, ListedMoved { place, span });
            } else {
                self.problem(
                    span,
                    format!(
                        "`{name}` moved to `{place}`, which isn't a file's path ending in `.md`, with `#` and an id for a heading"
                    ),
                );
            }
        }
        for (name, span, reason) in self.table(root, "retired") {
            if reason.trim().is_empty() {
                self.problem(
                    span,
                    format!("the retired permalink `{name}` has no reason"),
                );
            } else {
                list.retired.insert(name, ListedRetired { reason, span });
            }
        }
        for (name, span) in self.seen.clone() {
            let tables: Vec<&str> = [
                ("permalinks", list.permalinks.contains_key(&name)),
                ("aliases", list.aliases.contains_key(&name)),
                ("moved", list.moved.contains_key(&name)),
                ("retired", list.retired.contains_key(&name)),
            ]
            .into_iter()
            .filter_map(|(table, has)| has.then_some(table))
            .collect();
            if let [first, second, ..] = tables.as_slice() {
                self.problem(
                    span,
                    format!(
                        "`{name}` is in both `[{first}]` and `[{second}]`; a name is one thing"
                    ),
                );
            }
        }
        list
    }

    /// The entries of the table `key`, each a valid name with a string
    /// value; every other entry is a problem. Each name is noted once, for
    /// a name in two tables.
    fn table(&mut self, root: &DeTable<'_>, key: &str) -> Vec<(String, Span, String)> {
        let mut out = Vec::new();
        let Some(value) = root.get(key) else {
            return out;
        };
        let DeValue::Table(table) = value.get_ref() else {
            self.problem(
                sp(value),
                format!("`{key}` is {}, not a table", describe(value.get_ref())),
            );
            return out;
        };
        for (name, span, value) in entries(table) {
            if !ascribe_core::is_id(name) {
                self.problem(
                    span,
                    format!("`{name}` isn't a permalink's name: {NAME_RULE}"),
                );
                continue;
            }
            match value.get_ref() {
                DeValue::String(text) => {
                    if !self.seen.iter().any(|(n, _)| n == name) {
                        self.seen.push((name.to_owned(), span));
                    }
                    out.push((name.to_owned(), span, text.to_string()));
                }
                other => self.problem(
                    sp(value),
                    format!(
                        "the value of `{name}` in `[{key}]` is {}, not a string",
                        describe(other)
                    ),
                ),
            }
        }
        out
    }
}

/// A place as the list writes it: a content path ending in `.md`, and
/// optionally `#` and an id.
fn is_place(text: &str) -> bool {
    split_place(text).is_some()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn declared(pairs: &[(&str, &[&str])]) -> Current {
        Current {
            declared: pairs
                .iter()
                .map(|(name, places)| {
                    (
                        (*name).to_owned(),
                        places.iter().map(|p| (*p).to_owned()).collect(),
                    )
                })
                .collect(),
            ..Current::default()
        }
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
    fn aliases_and_retired_names_are_written_after() {
        let mut current = declared(&[("settings", &["settings.md"])]);
        current
            .aliases
            .insert("agent-settings".to_owned(), "settings".to_owned());
        current.retired.insert(
            "rotate-keys".to_owned(),
            "Merged into \"Security\"".to_owned(),
        );
        let synced = PermalinkList::default().synced(&current);
        let text = synced.list.to_toml();
        assert_eq!(
            text,
            "# Written by Ascribe. Don't edit it: run `ascribe permalinks sync`.\n\
             version = 1\n\
             \n\
             [permalinks]\n\
             settings = \"settings.md\"\n\
             \n\
             [aliases]\n\
             agent-settings = \"settings\"\n\
             \n\
             [retired]\n\
             rotate-keys = \"Merged into \\\"Security\\\"\"\n"
        );
        assert_eq!(list(&text).to_toml(), text);
        assert_eq!(synced.aliased, ["agent-settings"]);
        assert_eq!(synced.retired, ["rotate-keys"]);
        // Up to date, it changes nothing.
        let again = list(&text).synced(&current);
        assert!(again.aliased.is_empty() && again.retired.is_empty());
        assert_eq!(again.list.to_toml(), text);
    }

    #[test]
    fn retiring_moves_a_name_and_bringing_it_back_moves_it_again() {
        let before = list("version = 1\n[permalinks]\nold = \"a.md#old\"\n");
        let mut current = Current::default();
        current.retired.insert("old".to_owned(), "Gone".to_owned());
        let synced = before.synced(&current);
        assert!(synced.removed.is_empty());
        assert_eq!(synced.retired, ["old"]);
        assert!(synced.list.permalinks.is_empty());
        let back = synced.list.synced(&declared(&[("old", &["b.md#old"])]));
        assert_eq!(back.added, ["old"]);
        assert!(back.list.retired.is_empty());
    }

    #[test]
    fn current_leaves_out_what_the_content_declares() {
        let recorded = crate::AddressFile::parse(
            "version = 1\n[moved]\n\"go:alias\" = \"go:both\"\n\"go:also\" = \"go:both\"\n\
             \"go:keys\" = \"guides/keys.md#rotate\"\n\"go:loop\" = \"go:loop\"\n\
             \"go:old\" = \"go:gone\"\n\
             [retired]\n\"go:back\" = \"r\"\n\"go:gone\" = \"Gone\"\n",
            FileId::new(0),
        )
        .expect("reads")
        .permalinks();
        let mut places = BTreeMap::new();
        places.insert("both".to_owned(), vec!["a.md".to_owned()]);
        places.insert("also".to_owned(), vec!["b.md".to_owned()]);
        places.insert("back".to_owned(), vec!["c.md".to_owned()]);
        let current = Current::new(places, &recorded);
        // A name moved to a retired one is retired with its reason, and
        // one the content declares is the content's.
        assert_eq!(current.retired.keys().collect::<Vec<_>>(), ["gone", "old"]);
        assert_eq!(current.retired["old"], "Gone");
        assert_eq!(current.aliases.keys().collect::<Vec<_>>(), ["alias"]);
        assert_eq!(current.moved["keys"], "guides/keys.md#rotate");
        assert!(current.has("also") && current.has("gone") && !current.has("loop"));
    }

    #[test]
    fn syncing_adds_and_moves_and_never_drops() {
        let before = list(
            "version = 1\n[permalinks]\n\
             gone = \"a.md#gone\"\n\
             kept = \"b.md#kept\"\n\
             moved = \"a.md#moved\"\n\
             shared = \"c.md#shared\"\n\
             [aliases]\nold-alias = \"kept\"\n\
             [retired]\nold-retired = \"Why\"\n",
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
        assert_eq!(synced.removed, ["gone", "old-alias", "old-retired"]);
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
        assert_eq!(synced.list.aliases["old-alias"].target, "kept");
        assert_eq!(synced.list.retired["old-retired"].reason, "Why");
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
            ["the value of `a` in `[permalinks]` is a boolean, not a string"]
        );
        assert_eq!(
            bad("version = 1\n[aliases]\na = \"b/c\"\n"),
            ["the alias `a` leads to `b/c`, which isn't a permalink's name"]
        );
        assert_eq!(
            bad("version = 1\n[retired]\na = \"\"\n"),
            ["the retired permalink `a` has no reason"]
        );
        assert_eq!(
            bad("version = 1\n[permalinks]\na = \"a.md\"\n[retired]\na = \"Why\"\n"),
            ["`a` is in both `[permalinks]` and `[retired]`; a name is one thing"]
        );
    }
}
