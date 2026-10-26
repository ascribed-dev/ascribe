//! `ascribe.addresses` (SPEC §7.10): where the addresses the docs have
//! published went, so none stops working without someone deciding it
//! should. `[moved]` says where a source's addresses lead now, `[retired]`
//! why they were removed, and `[former]` the settings the docs were
//! published under before. A permalink's name is a key too, `permalink:`
//! and the name (§7.6): retired, moved to a page, or, moved to another
//! permalink, an alias ([`AddressFile::permalinks`]).
//!
//! Ascribe writes the file whole ([`AddressFile::to_toml`]), each table
//! sorted, one entry per line, when a move is recorded or a command adds an
//! entry, and reads it ([`AddressFile::parse`]) whenever it checks or builds
//! a project. A project without one records nothing.

use std::collections::BTreeMap;
use std::fmt::Write as _;

use ascribe_core::{FileId, Issue, Location, Span, TextEdit, diagnostics};

use toml::de::{DeTable, DeValue};

use crate::model::{PermalinkAlias, PermalinkRecords, RecordedIn, RetiredPermalink};
use crate::toml_util::{describe, entries, sp};

/// What a key, or a `[moved]` value, that names a permalink starts with:
/// `permalink:` and the name.
pub const PERMALINK_PREFIX: &str = "permalink:";

/// The file's name, beside `ascribe.toml`.
pub const ADDRESSES_FILE: &str = "ascribe.addresses";

/// The file's format, its `version`.
pub const ADDRESSES_VERSION: i64 = 1;

/// The comment the file starts with, which writing it keeps.
pub const ADDRESSES_HEADER: &str = "# Where addresses the docs published went. Written by `ascribe rename`, the\n\
     # editor, `ascribe addresses`, and `ascribe check --fix`; edit only\n\
     # [retired]'s reasons.\n";

/// The reason a retired address is written with for the author to replace.
/// It isn't a reason: an entry that still has it is
/// `addresses-file-invalid`.
pub const RETIRED_PLACEHOLDER: &str = "why this address was retired";

/// What `ascribe.addresses` records.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct AddressFile {
    /// `[moved]`: where each key's addresses lead now, by key.
    pub moved: BTreeMap<String, Entry>,
    /// `[former]`: the settings the docs were published under before.
    pub former: Former,
    /// `[retired]`: why each key's addresses were removed, by key.
    pub retired: BTreeMap<String, Entry>,
}

/// An entry of `[moved]` or `[retired]`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Entry {
    /// The value: in `[moved]`, a source path, or a source path, `#`, and
    /// an id; in `[retired]`, the reason.
    pub value: String,
    /// The entry's key, for reporting; empty for an entry not read from a
    /// file.
    pub span: Span,
}

impl Entry {
    /// An entry with `value`, not read from a file.
    pub fn new(value: impl Into<String>) -> Entry {
        Entry {
            value: value.into(),
            span: Span::empty(0),
        }
    }
}

/// One step from an entry's value to where another entry leads it
/// ([`AddressFile::step`]).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Step {
    /// To another place.
    Moved(String),
    /// To a retired entry, with its reason.
    Retired(String),
}

/// `[former]`: each routing setting the docs were published under before,
/// oldest first.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Former {
    /// `base-path`.
    pub base_path: Vec<String>,
    /// `permalink-path`.
    pub permalink_path: Vec<String>,
    /// `agents`.
    pub agents: Vec<bool>,
    /// `builds`: a build's old name to its new one.
    pub builds: BTreeMap<String, String>,
}

impl Former {
    /// Whether it records nothing.
    pub fn is_empty(&self) -> bool {
        self.base_path.is_empty()
            && self.permalink_path.is_empty()
            && self.agents.is_empty()
            && self.builds.is_empty()
    }
}

/// What a key of `[moved]` or `[retired]` names.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Key<'a> {
    /// A source file, by its content path.
    Source(&'a str),
    /// A heading's id, in the file the `@id` is written in.
    Id {
        /// The file's content path.
        path: &'a str,
        /// The id.
        id: &'a str,
    },
    /// An address from the host's root, from before the project used
    /// Ascribe.
    Address(&'a str),
    /// A permalink, by its name: what its address under the permalinks'
    /// path leads to.
    Permalink(&'a str),
}

impl<'a> Key<'a> {
    /// What `text` names, when it's a key the file can hold.
    pub fn parse(text: &'a str) -> Option<Key<'a>> {
        if let Some(name) = text.strip_prefix(PERMALINK_PREFIX) {
            return ascribe_core::is_id(name).then_some(Key::Permalink(name));
        }
        if text.starts_with('/') {
            return (!text.chars().any(char::is_whitespace)).then_some(Key::Address(text));
        }
        match split_place(text)? {
            (path, None) => Some(Key::Source(path)),
            (path, Some(id)) => Some(Key::Id { path, id }),
        }
    }

    /// The source file it names, when it names one.
    pub fn path(self) -> Option<&'a str> {
        match self {
            Key::Source(path) | Key::Id { path, .. } => Some(path),
            Key::Address(_) | Key::Permalink(_) => None,
        }
    }
}

/// The permalink `text` names, as a key or a `[moved]` value writes one:
/// `permalink:` and a valid name.
pub fn permalink_name(text: &str) -> Option<&str> {
    text.strip_prefix(PERMALINK_PREFIX)
        .filter(|name| ascribe_core::is_id(name))
}

/// `name` as a key: `permalink:` and the name.
pub fn permalink_key(name: &str) -> String {
    format!("{PERMALINK_PREFIX}{name}")
}

/// A place as the file writes one: a content path ending in `.md`, and
/// optionally `#` and an id; split, when it's one.
pub fn split_place(text: &str) -> Option<(&str, Option<&str>)> {
    let (path, id) = match text.split_once('#') {
        Some((path, id)) => (path, Some(id)),
        None => (text, None),
    };
    let path_ok = path.len() > 3
        && path.ends_with(".md")
        && !path.starts_with('/')
        && !path.contains('\\')
        && !path.split('/').any(|part| part.is_empty() || part == "..");
    (path_ok && id.is_none_or(ascribe_core::is_id)).then_some((path, id))
}

impl AddressFile {
    /// Reads the file's text. Issues are `addresses-file-invalid`, located
    /// in `file`; with any, the file isn't used.
    ///
    /// # Errors
    ///
    /// Every problem found, each at the value it's about.
    pub fn parse(text: &str, file: FileId) -> Result<AddressFile, Vec<Issue>> {
        let mut reader = Reader {
            file,
            issues: Vec::new(),
        };
        let out = reader.file(text);
        if reader.issues.is_empty() {
            Ok(out)
        } else {
            Err(reader.issues)
        }
    }

    /// What the file records of permalinks, each under its `permalink:` key:
    /// a `[retired]` entry is a retired permalink, with its reason; a
    /// `[moved]` entry to a page is one moved there; and one to another
    /// permalink is an alias of it. In key order, each table's.
    pub fn permalinks(&self) -> PermalinkRecords {
        let mut out = PermalinkRecords::default();
        for (key, entry) in &self.retired {
            if let Some(name) = permalink_name(key) {
                out.retired.push(RetiredPermalink {
                    name: name.to_owned(),
                    reason: Some(entry.value.clone()),
                    to: None,
                    span: entry.span,
                    to_span: None,
                    file: RecordedIn::Addresses,
                });
            }
        }
        for (key, entry) in &self.moved {
            let Some(name) = permalink_name(key) else {
                continue;
            };
            match permalink_name(&entry.value) {
                Some(target) => out.aliases.push(PermalinkAlias {
                    name: name.to_owned(),
                    target: target.to_owned(),
                    span: entry.span,
                    target_span: entry.span,
                    file: RecordedIn::Addresses,
                }),
                None => out.retired.push(RetiredPermalink {
                    name: name.to_owned(),
                    reason: None,
                    to: Some(entry.value.clone()),
                    span: entry.span,
                    to_span: Some(entry.span),
                    file: RecordedIn::Addresses,
                }),
            }
        }
        out
    }

    /// Records `records`, permalinks' records `ascribe.toml` had, each
    /// under its `permalink:` key: a retired permalink in `[retired]`, or,
    /// with a `to`, in `[moved]`, leading there; and an alias in `[moved]`,
    /// leading to the permalink it names. A name the file records already
    /// keeps its entry. The names recorded, in order.
    pub fn record_permalinks(&mut self, records: &PermalinkRecords) -> Vec<String> {
        let mut out = Vec::new();
        for retired in &records.retired {
            let key = permalink_key(&retired.name);
            if self.entry(&key).is_some() {
                continue;
            }
            match (&retired.to, &retired.reason) {
                (Some(to), _) => {
                    self.moved.insert(key, Entry::new(to.clone()));
                }
                (None, Some(reason)) => {
                    self.retired.insert(key, Entry::new(reason.clone()));
                }
                (None, None) => continue,
            }
            out.push(retired.name.clone());
        }
        for alias in &records.aliases {
            let key = permalink_key(&alias.name);
            if self.entry(&key).is_some() {
                continue;
            }
            self.moved
                .insert(key, Entry::new(permalink_key(&alias.target)));
            out.push(alias.name.clone());
        }
        out
    }

    /// The table that has `key`, and its entry: `moved` or `retired`.
    pub fn entry(&self, key: &str) -> Option<(&'static str, &Entry)> {
        self.moved
            .get(key)
            .map(|e| ("moved", e))
            .or_else(|| self.retired.get(key).map(|e| ("retired", e)))
    }

    /// Records that `old` moved to `new`: `[moved]`'s entry for `old`, in
    /// place of any `[retired]` one. Each entry that led to `old` leads to
    /// `new` instead, so every pair stays flat; and an entry keyed by `new`,
    /// whose source is back at its key, goes.
    ///
    /// When `old` is a page, an entry that leads to one of its headings,
    /// `old#id`, leads to `new#id`: the heading moved with its page.
    pub fn record_move(&mut self, old: &str, new: &str) {
        self.retired.remove(old);
        let page = !old.contains('#') && !new.contains('#');
        for entry in self.moved.values_mut() {
            if entry.value == old {
                entry.value = new.to_owned();
            } else if let Some(id) = entry
                .value
                .strip_prefix(old)
                .and_then(|rest| rest.strip_prefix('#'))
                .filter(|_| page)
            {
                entry.value = format!("{new}#{id}");
            }
        }
        self.moved.insert(old.to_owned(), Entry::new(new));
        self.moved.remove(new);
        self.retired.remove(new);
        self.moved.retain(|key, entry| *key != entry.value);
    }

    /// Records that `old` was retired, for `reason`, in place of any
    /// `[moved]` entry for it. Each entry that led to `old`, or to one of
    /// its headings, is retired with it, for the same reason: what it led
    /// to is gone.
    pub fn record_retired(&mut self, old: &str, reason: &str) {
        self.moved.remove(old);
        let leading: Vec<String> = self
            .moved
            .iter()
            .filter(|(_, entry)| {
                entry.value == old
                    || (!old.contains('#')
                        && entry
                            .value
                            .strip_prefix(old)
                            .is_some_and(|rest| rest.starts_with('#')))
            })
            .map(|(key, _)| key.clone())
            .collect();
        for key in leading {
            self.moved.remove(&key);
            self.retired.insert(key, Entry::new(reason));
        }
        self.retired.insert(old.to_owned(), Entry::new(reason));
    }

    /// Where `value`, a `[moved]` value, leads in the end: itself, or, when
    /// it's another entry's key, or a heading on a page that is, where that
    /// entry leads, and so on. `None` when the entries lead back to where
    /// they started, or to a retired entry. The build writes each address
    /// to where its entry leads in the end, so every pair it writes is one
    /// step.
    pub fn follow(&self, value: &str) -> Option<String> {
        let mut seen: Vec<String> = Vec::new();
        let mut current = value.to_owned();
        loop {
            if seen.contains(&current) {
                return None;
            }
            match self.step(&current) {
                Some(Step::Moved(next)) => {
                    seen.push(std::mem::replace(&mut current, next));
                }
                Some(Step::Retired(_)) => return None,
                None => return Some(current),
            }
        }
    }

    /// Where an entry's value leads next, when it's another entry's key, or
    /// a heading on a page whose entry leads to a page.
    pub fn step(&self, value: &str) -> Option<Step> {
        if let Some(entry) = self.moved.get(value) {
            return Some(Step::Moved(entry.value.clone()));
        }
        if let Some(entry) = self.retired.get(value) {
            return Some(Step::Retired(entry.value.clone()));
        }
        let (path, id) = value.split_once('#')?;
        match self.moved.get(path) {
            Some(entry) if !entry.value.contains('#') => {
                Some(Step::Moved(format!("{}#{id}", entry.value)))
            }
            _ => None,
        }
    }

    /// Removes the entry for `key`, from whichever table has it.
    pub fn remove(&mut self, key: &str) {
        self.moved.remove(key);
        self.retired.remove(key);
    }

    /// The one edit that turns `text`, the file as it is, into the file as
    /// Ascribe writes this: the lines that differ, replaced. `None` when
    /// they're the same. Each edit an entry makes is of its own lines, so
    /// edits that record different entries don't overlap unless their
    /// lines are next to each other.
    pub fn edit_from(&self, text: &str) -> Option<TextEdit> {
        let after = self.to_toml();
        (after != text).then(|| line_edit(text, &after))
    }

    /// The file as Ascribe writes it: the header, `version`, and each table
    /// with one line for each entry, in order, and the same text for the
    /// same file on every platform. `[former]` is written only when it
    /// records something.
    pub fn to_toml(&self) -> String {
        let mut out = String::from(ADDRESSES_HEADER);
        let _ = writeln!(out, "version = {ADDRESSES_VERSION}");
        out.push_str("\n[moved]\n");
        for (key, entry) in &self.moved {
            let _ = writeln!(out, "{} = {}", quoted(key), quoted(&entry.value));
        }
        if !self.former.is_empty() {
            out.push_str("\n[former]\n");
            let list = |values: &[String]| {
                values
                    .iter()
                    .map(|v| quoted(v))
                    .collect::<Vec<_>>()
                    .join(", ")
            };
            if !self.former.agents.is_empty() {
                let values: Vec<String> =
                    self.former.agents.iter().map(ToString::to_string).collect();
                let _ = writeln!(out, "agents = [{}]", values.join(", "));
            }
            if !self.former.base_path.is_empty() {
                let _ = writeln!(out, "base-path = [{}]", list(&self.former.base_path));
            }
            if !self.former.permalink_path.is_empty() {
                let _ = writeln!(
                    out,
                    "permalink-path = [{}]",
                    list(&self.former.permalink_path)
                );
            }
            if !self.former.builds.is_empty() {
                out.push_str("\n[former.builds]\n");
                for (old, new) in &self.former.builds {
                    let _ = writeln!(out, "{} = {}", quoted(old), quoted(new));
                }
            }
        }
        out.push_str("\n[retired]\n");
        for (key, entry) in &self.retired {
            let _ = writeln!(out, "{} = {}", quoted(key), quoted(&entry.value));
        }
        out
    }
}

/// The edit that turns `before` into `after`: the lines between the lines
/// they start with and end with in common, replaced.
fn line_edit(before: &str, after: &str) -> TextEdit {
    let a: Vec<&str> = before.split_inclusive('\n').collect();
    let b: Vec<&str> = after.split_inclusive('\n').collect();
    let prefix = a.iter().zip(&b).take_while(|(x, y)| x == y).count();
    let suffix = a[prefix..]
        .iter()
        .rev()
        .zip(b[prefix..].iter().rev())
        .take_while(|(x, y)| x == y)
        .count();
    let start: usize = a[..prefix].iter().map(|l| l.len()).sum();
    let end = before.len() - a[a.len() - suffix..].iter().map(|l| l.len()).sum::<usize>();
    TextEdit::replace(start..end, b[prefix..b.len() - suffix].concat())
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
                diagnostics::ADDRESSES_FILE_INVALID,
                Location::new(self.file, span),
            )
            .with_arg("detail", detail.into()),
        );
    }

    fn file(&mut self, text: &str) -> AddressFile {
        let mut out = AddressFile::default();
        // A key written twice in one table, as a merge can leave, isn't
        // TOML; it's said as itself, with both values.
        if self.twice_in_a_table(text) {
            return out;
        }
        let parsed = match DeTable::parse(text) {
            Ok(parsed) => parsed,
            Err(e) => {
                let span = e
                    .span()
                    .map_or(Span::empty(0), |r| Span::new(r.start, r.end));
                self.problem(span, e.message().trim().to_owned());
                return out;
            }
        };
        let root = parsed.get_ref();
        for (key, span, _) in entries(root) {
            if !matches!(key, "version" | "moved" | "former" | "retired") {
                self.problem(span, format!("it has no key `{key}`"));
            }
        }
        match root.get("version") {
            None => self.problem(Span::empty(0), "it has no `version`"),
            Some(v) => match v.get_ref() {
                DeValue::Integer(i) if i.as_str() == ADDRESSES_VERSION.to_string() => {}
                DeValue::Integer(i) => self.problem(
                    sp(v),
                    format!(
                        "it's version {}, and this version of Ascribe reads version {ADDRESSES_VERSION}; update Ascribe",
                        i.as_str()
                    ),
                ),
                other => self.problem(
                    sp(v),
                    format!("`version` is {}, not a number", describe(other)),
                ),
            },
        }
        for (key, span, value) in self.table(root, "moved") {
            let permalink = permalink_name(&key).is_some();
            if split_place(&value).is_some() || (permalink && permalink_name(&value).is_some()) {
                out.moved.insert(key, Entry { value, span });
            } else if permalink {
                self.problem(
                    span,
                    format!(
                        "`{key}` moved to `{value}`, which isn't a file's path ending in `.md`, with `#` and an id for a heading, or `permalink:` and a permalink's name"
                    ),
                );
            } else {
                self.problem(
                    span,
                    format!(
                        "`{key}` moved to `{value}`, which isn't a file's path ending in `.md`, with `#` and an id for a heading"
                    ),
                );
            }
        }
        for (key, span, reason) in self.table(root, "retired") {
            if reason.trim().is_empty() {
                self.problem(span, format!("the retired address `{key}` has no reason"));
            } else if reason.trim() == RETIRED_PLACEHOLDER {
                self.problem(
                    span,
                    format!(
                        "the retired address `{key}` still has the placeholder for its reason; say why it was retired"
                    ),
                );
            } else {
                out.retired.insert(
                    key,
                    Entry {
                        value: reason,
                        span,
                    },
                );
            }
        }
        if let Some(value) = root.get("former") {
            out.former = self.former(value);
        }
        for (key, retired) in &out.retired {
            if let Some(moved) = out.moved.get(key) {
                let later = if retired.span.start() > moved.span.start() {
                    retired.span
                } else {
                    moved.span
                };
                self.twice(
                    later,
                    key,
                    &format!("moved to `{}`", moved.value),
                    &format!("retired: “{}”", retired.value),
                );
            }
        }
        out
    }

    /// `addresses-key-twice` at `span`, for `key` with both its values.
    fn twice(&mut self, span: Span, key: &str, first: &str, second: &str) {
        self.issues.push(
            Issue::new(
                diagnostics::ADDRESSES_KEY_TWICE,
                Location::new(self.file, span),
            )
            .with_arg("key", key.to_owned())
            .with_arg("first", first.to_owned())
            .with_arg("second", second.to_owned()),
        );
    }

    /// Reports each key of `[moved]` or `[retired]` written a second time in
    /// its table, as a line `"<key>" = "<value>"`: whether there was one.
    fn twice_in_a_table(&mut self, text: &str) -> bool {
        let mut table = "";
        let mut seen: BTreeMap<(String, String), String> = BTreeMap::new();
        let mut found = false;
        let mut at = 0;
        for line in text.split_inclusive('\n') {
            let start = at;
            at += line.len();
            let trimmed = line.trim();
            if let Some(name) = trimmed.strip_prefix('[') {
                table = name.split(']').next().unwrap_or_default().trim();
                continue;
            }
            if !matches!(table, "moved" | "retired") {
                continue;
            }
            let Some((key, value)) = key_line(trimmed) else {
                continue;
            };
            let offset = start + (line.len() - line.trim_start().len());
            let key_len = trimmed
                .find(" =")
                .or_else(|| trimmed.find('='))
                .unwrap_or(0);
            let span = Span::new(offset, offset + key_len);
            let said = |value: &str| {
                if table == "moved" {
                    format!("moved to `{value}`")
                } else {
                    format!("retired: “{value}”")
                }
            };
            match seen.get(&(table.to_owned(), key.clone())) {
                Some(first) => {
                    let first = said(first);
                    self.twice(span, &key, &first, &said(&value));
                    found = true;
                }
                None => {
                    seen.insert((table.to_owned(), key), value);
                }
            }
        }
        found
    }

    /// The entries of the table `key`, each a key the file can hold with a
    /// string value; every other entry is a problem.
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
            if Key::parse(name).is_none() {
                self.problem(
                    span,
                    format!(
                        "`{name}` isn't a key `[{key}]` can have: a file's path ending in `.md`, with `#` and an id for a heading, an address starting with `/`, or `permalink:` and a permalink's name"
                    ),
                );
                continue;
            }
            match value.get_ref() {
                DeValue::String(text) => out.push((name.to_owned(), span, text.to_string())),
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

    fn former(&mut self, value: &toml::Spanned<DeValue<'_>>) -> Former {
        let mut out = Former::default();
        let DeValue::Table(table) = value.get_ref() else {
            self.problem(
                sp(value),
                format!("`former` is {}, not a table", describe(value.get_ref())),
            );
            return out;
        };
        for (key, span, value) in entries(table) {
            match key {
                "base-path" | "permalink-path" => {
                    let values = self.strings(key, value);
                    if key == "base-path" {
                        out.base_path = values;
                    } else {
                        out.permalink_path = values;
                    }
                }
                "agents" => {
                    let DeValue::Array(items) = value.get_ref() else {
                        self.problem(
                            sp(value),
                            format!(
                                "`former.agents` is {}, not an array",
                                describe(value.get_ref())
                            ),
                        );
                        continue;
                    };
                    for item in items.iter() {
                        match item.get_ref() {
                            DeValue::Boolean(b) => out.agents.push(*b),
                            other => self.problem(
                                sp(item),
                                format!(
                                    "a value of `former.agents` is {}, not a boolean",
                                    describe(other)
                                ),
                            ),
                        }
                    }
                }
                "builds" => {
                    let DeValue::Table(builds) = value.get_ref() else {
                        self.problem(
                            sp(value),
                            format!(
                                "`former.builds` is {}, not a table",
                                describe(value.get_ref())
                            ),
                        );
                        continue;
                    };
                    for (old, _, new) in entries(builds) {
                        match new.get_ref() {
                            DeValue::String(text) => {
                                out.builds.insert(old.to_owned(), text.to_string());
                            }
                            other => self.problem(
                                sp(new),
                                format!(
                                    "the value of `{old}` in `[former.builds]` is {}, not a build's name",
                                    describe(other)
                                ),
                            ),
                        }
                    }
                }
                _ => self.problem(span, format!("`[former]` has no key `{key}`")),
            }
        }
        out
    }

    fn strings(&mut self, key: &str, value: &toml::Spanned<DeValue<'_>>) -> Vec<String> {
        let DeValue::Array(items) = value.get_ref() else {
            self.problem(
                sp(value),
                format!(
                    "`former.{key}` is {}, not an array",
                    describe(value.get_ref())
                ),
            );
            return Vec::new();
        };
        let mut out = Vec::new();
        for item in items.iter() {
            match item.get_ref() {
                DeValue::String(text) => out.push(text.to_string()),
                other => self.problem(
                    sp(item),
                    format!(
                        "a value of `former.{key}` is {}, not a string",
                        describe(other)
                    ),
                ),
            }
        }
        out
    }
}

/// A line `"<key>" = "<value>"`, or with a bare key, as its key and value:
/// the strings as written, unescaped as far as `\"` and `\\`.
fn key_line(line: &str) -> Option<(String, String)> {
    let (key, rest) = if let Some(quoted) = line.strip_prefix('"') {
        let (key, rest) = basic_string(quoted)?;
        (key, rest.trim_start().strip_prefix('=')?)
    } else {
        let (key, rest) = line.split_once('=')?;
        (key.trim().to_owned(), rest)
    };
    let rest = rest.trim_start();
    let value = match rest.strip_prefix('"') {
        Some(quoted) => basic_string(quoted)?.0,
        None => rest.trim().to_owned(),
    };
    (!key.is_empty()).then_some((key, value))
}

/// The string at the start of `text`, after its opening `"`, and what
/// follows its closing one.
fn basic_string(text: &str) -> Option<(String, &str)> {
    let mut out = String::new();
    let mut chars = text.char_indices();
    while let Some((i, c)) = chars.next() {
        match c {
            '"' => return Some((out, &text[i + 1..])),
            '\\' => out.push(chars.next()?.1),
            c => out.push(c),
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn read(text: &str) -> AddressFile {
        AddressFile::parse(text, FileId::new(0)).expect("it reads")
    }

    fn bad(text: &str) -> Vec<String> {
        AddressFile::parse(text, FileId::new(0))
            .expect_err(text)
            .iter()
            .map(|i| i.args.iter().map(|a| a.value.clone()).collect::<String>())
            .collect()
    }

    #[test]
    fn written_and_read_back() {
        let text = "# Where addresses the docs published went. Written by `ascribe rename`, the\n\
                    # editor, `ascribe addresses`, and `ascribe check --fix`; edit only\n\
                    # [retired]'s reasons.\n\
                    version = 1\n\
                    \n\
                    [moved]\n\
                    \"/docs/intro.html\" = \"getting-started.md\"\n\
                    \"guides/install.md#install-with-npm\" = \"guides/install.md#install-npm\"\n\
                    \"guides/keys.md\" = \"security/keys.md\"\n\
                    \n\
                    [former]\n\
                    base-path = [\"/docs/\"]\n\
                    \n\
                    [retired]\n\
                    \"reference/legacy-sync.md\" = \"The sync agent was removed in 4.0\"\n";
        let file = read(text);
        assert_eq!(file.moved.len(), 3);
        assert_eq!(file.former.base_path, ["/docs/"]);
        assert_eq!(file.to_toml(), text);
    }

    #[test]
    fn a_new_file_has_its_header_and_empty_tables() {
        let text = AddressFile::default().to_toml();
        assert_eq!(
            text,
            format!("{ADDRESSES_HEADER}version = 1\n\n[moved]\n\n[retired]\n")
        );
        assert_eq!(read(&text), AddressFile::default());
    }

    #[test]
    fn written_sorted_whatever_the_order_read() {
        let file = read(
            "version = 1\n[retired]\n\"b.md\" = \"Why\"\n[moved]\n\"z.md\" = \"a.md\"\n\"c.md\" = \"a.md\"\n",
        );
        assert!(file.to_toml().ends_with(
            "[moved]\n\"c.md\" = \"a.md\"\n\"z.md\" = \"a.md\"\n\n[retired]\n\"b.md\" = \"Why\"\n"
        ));
    }

    #[test]
    fn a_move_keeps_every_pair_flat() {
        let mut file = AddressFile::default();
        file.record_move("a.md", "b.md");
        file.record_move("b.md", "c.md");
        let moved: Vec<(&str, &str)> = file
            .moved
            .iter()
            .map(|(k, e)| (k.as_str(), e.value.as_str()))
            .collect();
        assert_eq!(moved, [("a.md", "c.md"), ("b.md", "c.md")]);
        // A move back to a key removes its entry.
        file.record_move("c.md", "a.md");
        let moved: Vec<(&str, &str)> = file
            .moved
            .iter()
            .map(|(k, e)| (k.as_str(), e.value.as_str()))
            .collect();
        assert_eq!(moved, [("b.md", "a.md"), ("c.md", "a.md")]);
    }

    #[test]
    fn a_page_s_move_takes_the_entries_to_its_headings_with_it() {
        let mut file = AddressFile::default();
        file.record_move("old.md#install", "keys.md#install");
        file.record_move("keys.md", "security/keys.md");
        assert_eq!(
            file.moved["old.md#install"].value,
            "security/keys.md#install"
        );
    }

    #[test]
    fn retiring_a_page_retires_the_entries_that_lead_to_it() {
        let mut file = AddressFile::default();
        file.record_move("a.md", "b.md");
        file.record_move("c.md#x", "b.md#y");
        file.record_move("d.md", "e.md");
        file.record_retired("b.md", "Gone");
        let retired: Vec<&str> = file.retired.keys().map(String::as_str).collect();
        assert_eq!(retired, ["a.md", "b.md", "c.md#x"]);
        assert_eq!(file.moved.len(), 1);
    }

    #[test]
    fn the_edit_is_the_lines_that_change() {
        let mut file = AddressFile::default();
        file.record_move("a.md", "b.md");
        let text = file.to_toml();
        file.record_move("c.md", "d.md");
        let edit = file.edit_from(&text).expect("an edit");
        assert_eq!(edit.new_text, "\"c.md\" = \"d.md\"\n");
        assert_eq!(
            ascribe_core::apply_edits(&text, &[edit]).expect("applies"),
            file.to_toml()
        );
        assert!(file.edit_from(&file.to_toml()).is_none());
    }

    #[test]
    fn a_key_twice_names_both_values() {
        let found = AddressFile::parse(
            "version = 1\n[moved]\n\"a.md\" = \"b.md\"\n\"a.md\" = \"c.md\"\n",
            FileId::new(0),
        )
        .expect_err("twice");
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].slug, diagnostics::ADDRESSES_KEY_TWICE);
        assert_eq!(found[0].arg("first"), Some("moved to `b.md`"));
        assert_eq!(found[0].arg("second"), Some("moved to `c.md`"));
        let both = AddressFile::parse(
            "version = 1\n[moved]\n\"a.md\" = \"b.md\"\n[retired]\n\"a.md\" = \"Gone\"\n",
            FileId::new(0),
        )
        .expect_err("in both");
        assert_eq!(both[0].slug, diagnostics::ADDRESSES_KEY_TWICE);
        assert_eq!(both[0].arg("second"), Some("retired: “Gone”"));
    }

    #[test]
    fn retiring_replaces_a_move() {
        let mut file = AddressFile::default();
        file.record_move("a.md", "b.md");
        file.record_retired("a.md", "Gone");
        assert!(file.moved.is_empty());
        assert_eq!(file.retired["a.md"].value, "Gone");
        file.record_move("a.md", "c.md");
        assert!(file.retired.is_empty());
    }

    #[test]
    fn keys() {
        assert_eq!(Key::parse("a/b.md"), Some(Key::Source("a/b.md")));
        assert_eq!(
            Key::parse("_f.md#install"),
            Some(Key::Id {
                path: "_f.md",
                id: "install"
            })
        );
        assert_eq!(
            Key::parse("/docs/intro.html"),
            Some(Key::Address("/docs/intro.html"))
        );
        assert_eq!(
            Key::parse("permalink:cli-check"),
            Some(Key::Permalink("cli-check"))
        );
        assert_eq!(Key::parse("permalink:a/b"), None);
        assert_eq!(Key::parse("a.txt"), None);
        assert_eq!(Key::parse("../a.md"), None);
        assert_eq!(Key::parse("/a b"), None);
    }

    #[test]
    fn permalinks_are_keys() {
        let file = read(
            "version = 1\n[moved]\n\
             \"permalink:agent-settings\" = \"permalink:settings\"\n\
             \"permalink:rotate-keys\" = \"guides/security.md#keys\"\n\
             [retired]\n\"permalink:legacy-sync\" = \"Removed in 4.0\"\n",
        );
        let records = file.permalinks();
        assert_eq!(records.alias_end("agent-settings"), Some("settings"));
        let moved = records.retired("rotate-keys").expect("moved");
        assert_eq!(moved.to.as_deref(), Some("guides/security.md#keys"));
        assert_eq!(moved.reason, None);
        assert_eq!(moved.why(), "Moved to guides/security.md#keys");
        let retired = records.retired("legacy-sync").expect("retired");
        assert_eq!(retired.reason.as_deref(), Some("Removed in 4.0"));
        assert_eq!(retired.to, None);
        // What `ascribe.toml` had, recorded as the file has it.
        let mut again = AddressFile::default();
        let recorded = again.record_permalinks(&records);
        assert_eq!(recorded, ["legacy-sync", "rotate-keys", "agent-settings"]);
        assert_eq!(
            again.to_toml(),
            {
                let mut file = file.clone();
                for entry in file.moved.values_mut().chain(file.retired.values_mut()) {
                    entry.span = Span::empty(0);
                }
                file
            }
            .to_toml()
        );
        // A name the file has keeps its entry.
        assert!(again.record_permalinks(&records).is_empty());
        // A permalink leads to a page or another permalink, and only a
        // permalink leads to a permalink.
        assert_eq!(
            bad("version = 1\n[moved]\n\"permalink:a\" = \"b\"\n"),
            [
                "`permalink:a` moved to `b`, which isn't a file's path ending in `.md`, with `#` and an id for a heading, or `permalink:` and a permalink's name"
            ]
        );
        assert_eq!(
            bad("version = 1\n[moved]\n\"a.md\" = \"permalink:b\"\n"),
            [
                "`a.md` moved to `permalink:b`, which isn't a file's path ending in `.md`, with `#` and an id for a heading"
            ]
        );
    }

    #[test]
    fn problems() {
        assert_eq!(
            bad("version = 2\n"),
            ["it's version 2, and this version of Ascribe reads version 1; update Ascribe"]
        );
        assert_eq!(bad(""), ["it has no `version`"]);
        assert_eq!(bad("version = 1\nlines = 1\n"), ["it has no key `lines`"]);
        assert_eq!(
            bad("version = 1\nmoved = 1\n"),
            ["`moved` is a number, not a table"]
        );
        assert_eq!(
            bad("version = 1\n[moved]\n\"a.txt\" = \"b.md\"\n"),
            [
                "`a.txt` isn't a key `[moved]` can have: a file's path ending in `.md`, with `#` and an id for a heading, an address starting with `/`, or `permalink:` and a permalink's name"
            ]
        );
        assert_eq!(
            bad("version = 1\n[moved]\n\"a.md\" = \"/b\"\n"),
            [
                "`a.md` moved to `/b`, which isn't a file's path ending in `.md`, with `#` and an id for a heading"
            ]
        );
        assert_eq!(
            bad("version = 1\n[moved]\n\"a.md\" = true\n"),
            ["the value of `a.md` in `[moved]` is a boolean, not a string"]
        );
        assert_eq!(
            bad("version = 1\n[retired]\n\"a.md\" = \" \"\n"),
            ["the retired address `a.md` has no reason"]
        );
        assert_eq!(
            bad("version = 1\n[retired]\n\"a.md\" = \"why this address was retired\"\n"),
            [
                "the retired address `a.md` still has the placeholder for its reason; say why it was retired"
            ]
        );
        assert_eq!(
            bad("version = 1\n[former]\ntrailing-slash = [\"always\"]\n"),
            ["`[former]` has no key `trailing-slash`"]
        );
        assert_eq!(
            bad("version = 1\n[former]\nbase-path = \"/docs/\"\n"),
            ["`former.base-path` is a string, not an array"]
        );
        assert!(AddressFile::parse("version = 1\n", FileId::new(0)).is_ok());
    }
}
