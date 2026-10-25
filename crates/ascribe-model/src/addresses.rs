//! `ascribe.addresses` (SPEC §7.10): where the addresses the docs have
//! published went, so none stops working without someone deciding it
//! should. `[moved]` says where a source's addresses lead now, `[retired]`
//! why they were removed, and `[former]` the settings the docs were
//! published under before.
//!
//! Ascribe writes the file whole ([`AddressFile::to_toml`]), each table
//! sorted, one entry per line, when a move is recorded or a command adds an
//! entry, and reads it ([`AddressFile::parse`]) whenever it checks or builds
//! a project. A project without one records nothing.

use std::collections::BTreeMap;
use std::fmt::Write as _;

use ascribe_core::{FileId, Issue, Location, Span, diagnostics};

use toml::de::{DeTable, DeValue};

use crate::toml_util::{describe, entries, sp};

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
}

impl<'a> Key<'a> {
    /// What `text` names, when it's a key the file can hold.
    pub fn parse(text: &'a str) -> Option<Key<'a>> {
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
            Key::Address(_) => None,
        }
    }
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
    pub fn record_move(&mut self, old: &str, new: &str) {
        self.retired.remove(old);
        for entry in self.moved.values_mut() {
            if entry.value == old {
                entry.value = new.to_owned();
            }
        }
        self.moved.insert(old.to_owned(), Entry::new(new));
        self.moved.remove(new);
        self.retired.remove(new);
        self.moved.retain(|key, entry| *key != entry.value);
    }

    /// Records that `old` was retired, for `reason`, in place of any
    /// `[moved]` entry for it.
    pub fn record_retired(&mut self, old: &str, reason: &str) {
        self.moved.remove(old);
        self.retired.insert(old.to_owned(), Entry::new(reason));
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
            if split_place(&value).is_some() {
                out.moved.insert(key, Entry { value, span });
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
        out
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
                        "`{name}` isn't a key `[{key}]` can have: a file's path ending in `.md`, with `#` and an id for a heading, or an address starting with `/`"
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
        assert_eq!(Key::parse("a.txt"), None);
        assert_eq!(Key::parse("../a.md"), None);
        assert_eq!(Key::parse("/a b"), None);
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
                "`a.txt` isn't a key `[moved]` can have: a file's path ending in `.md`, with `#` and an id for a heading, or an address starting with `/`"
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
