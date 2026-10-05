//! `ascribe.lock` (SPEC §7.4): the commit each source in another repository
//! is pinned to, and the hash of each file copied from it.
//!
//! Ascribe writes the lock ([`Lock::to_toml`]) when it copies files or moves
//! a pin, and reads it ([`Lock::parse`]) whenever it checks a project.

use std::collections::BTreeMap;
use std::fmt::Write as _;

use sha2::{Digest, Sha256};
use tessera_core::{FileId, Issue, Location, Span, diagnostics};
use toml::de::{DeTable, DeValue};

use crate::toml_util::{V, describe, entries, sp};

/// The lock's file name, beside `ascribe.toml`.
pub const LOCK_FILE: &str = "ascribe.lock";

/// The lock's format, its `version`.
pub const LOCK_VERSION: i64 = 1;

/// What the lock records.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Lock {
    /// One entry per pinned source, in the order written (by name, when
    /// Ascribe writes it).
    pub sources: Vec<LockedSource>,
}

/// A source's pin.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LockedSource {
    /// The source's name.
    pub name: String,
    /// The repository's URL, as `ascribe.toml` gave it when it was pinned.
    pub git: String,
    /// The commit, in full.
    pub commit: String,
    /// The copies, by path relative to the source's folder.
    pub files: BTreeMap<String, LockedFile>,
    /// The entry's `name` value, for reporting.
    pub span: Span,
}

/// A copied file's entry.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LockedFile {
    /// The hash of its bytes: `sha256:` and 64 lowercase hexadecimal digits.
    pub hash: String,
    /// The entry's key, for reporting.
    pub span: Span,
}

/// A file's hash as the lock records it ([`LockedFile::hash`]).
pub fn file_hash(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut out = String::with_capacity(7 + 64);
    out.push_str("sha256:");
    for byte in digest {
        let _ = write!(out, "{byte:02x}");
    }
    out
}

impl Lock {
    /// Reads a lock's text. Issues are `lock-invalid`, located in `file`;
    /// with any, the lock isn't used.
    ///
    /// # Errors
    ///
    /// Every problem found, each at the value it's about.
    pub fn parse(text: &str, file: FileId) -> Result<Lock, Vec<Issue>> {
        let mut reader = Reader {
            file,
            issues: Vec::new(),
        };
        let lock = reader.lock(text);
        if reader.issues.is_empty() {
            Ok(lock)
        } else {
            Err(reader.issues)
        }
    }

    /// The pin of the source named `name`.
    pub fn source(&self, name: &str) -> Option<&LockedSource> {
        self.sources.iter().find(|s| s.name == name)
    }

    /// The lock as Ascribe writes it: the sources by name, each copy by
    /// path, and the same text for the same lock on every platform.
    pub fn to_toml(&self) -> String {
        let mut out = String::from(
            "# Written by `ascribe sources fetch` and `ascribe sources update`. Don't edit it by hand.\n",
        );
        let _ = writeln!(out, "version = {LOCK_VERSION}");
        let mut sources: Vec<&LockedSource> = self.sources.iter().collect();
        sources.sort_by(|a, b| a.name.cmp(&b.name));
        for source in sources {
            out.push_str("\n[[source]]\n");
            let _ = writeln!(out, "name = {}", quoted(&source.name));
            let _ = writeln!(out, "git = {}", quoted(&source.git));
            let _ = writeln!(out, "commit = {}", quoted(&source.commit));
            if !source.files.is_empty() {
                out.push_str("\n[source.files]\n");
                for (path, file) in &source.files {
                    let _ = writeln!(out, "{} = {}", quoted(path), quoted(&file.hash));
                }
            }
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
            '\n' => out.push_str("\\n"),
            '\t' => out.push_str("\\t"),
            '\r' => out.push_str("\\r"),
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
            Issue::new(diagnostics::LOCK_INVALID, Location::new(self.file, span))
                .with_arg("detail", detail.into()),
        );
    }

    fn lock(&mut self, text: &str) -> Lock {
        let mut lock = Lock::default();
        let parsed = match DeTable::parse(text) {
            Ok(parsed) => parsed,
            Err(e) => {
                let span = e
                    .span()
                    .map_or(Span::empty(0), |r| Span::new(r.start, r.end));
                self.problem(span, e.message().trim().to_owned());
                return lock;
            }
        };
        let root = parsed.get_ref();
        for (key, span, _) in entries(root) {
            if !matches!(key, "version" | "source") {
                self.problem(span, format!("it has no key `{key}`"));
            }
        }
        match root.get("version") {
            None => self.problem(Span::empty(0), "it has no `version`"),
            Some(v) => match v.get_ref() {
                DeValue::Integer(i) if i.as_str() == LOCK_VERSION.to_string() => {}
                DeValue::Integer(i) => self.problem(
                    sp(v),
                    format!(
                        "it's version {}, and this version of Ascribe reads version {LOCK_VERSION}",
                        i.as_str()
                    ),
                ),
                other => self.problem(
                    sp(v),
                    format!("`version` is {}, not a number", describe(other)),
                ),
            },
        }
        let Some(sources) = root.get("source") else {
            return lock;
        };
        let DeValue::Array(items) = sources.get_ref() else {
            self.problem(sp(sources), "`source` isn't an array of tables");
            return lock;
        };
        for item in items.iter() {
            let DeValue::Table(table) = item.get_ref() else {
                self.problem(sp(item), "`source` isn't an array of tables");
                continue;
            };
            let Some(source) = self.source(table, sp(item)) else {
                continue;
            };
            if lock.source(&source.name).is_some() {
                self.problem(
                    source.span,
                    format!("source `{}` is pinned twice", source.name),
                );
                continue;
            }
            lock.sources.push(source);
        }
        lock
    }

    fn source(&mut self, table: &DeTable<'_>, span: Span) -> Option<LockedSource> {
        for (key, key_span, _) in entries(table) {
            if !matches!(key, "name" | "git" | "commit" | "files") {
                self.problem(key_span, format!("a `[[source]]` has no key `{key}`"));
            }
        }
        let name = self.string(table, "name", span);
        let git = self.string(table, "git", span);
        let commit = self.string(table, "commit", span);
        if let Some((commit, commit_span)) = &commit
            && !is_commit(commit)
        {
            self.problem(
                *commit_span,
                format!("`{commit}` isn't a commit's full hash"),
            );
        }
        let mut files = BTreeMap::new();
        if let Some(v) = table.get("files") {
            match v.get_ref() {
                DeValue::Table(t) => {
                    for (path, path_span, hash) in entries(t) {
                        let Some(hash) = self.hash(path, hash) else {
                            continue;
                        };
                        files.insert(
                            path.to_owned(),
                            LockedFile {
                                hash,
                                span: path_span,
                            },
                        );
                    }
                }
                other => self.problem(
                    sp(v),
                    format!("`files` is {}, not a table", describe(other)),
                ),
            }
        }
        let ((name, name_span), (git, _), (commit, _)) = (name?, git?, commit?);
        Some(LockedSource {
            name,
            git,
            commit,
            files,
            span: name_span,
        })
    }

    fn string(&mut self, table: &DeTable<'_>, key: &str, span: Span) -> Option<(String, Span)> {
        let Some(v) = table.get(key) else {
            self.problem(span, format!("a `[[source]]` has no `{key}`"));
            return None;
        };
        match v.get_ref() {
            DeValue::String(s) if !s.is_empty() => Some((s.to_string(), sp(v))),
            DeValue::String(_) => {
                self.problem(sp(v), format!("`{key}` is empty"));
                None
            }
            other => {
                self.problem(
                    sp(v),
                    format!("`{key}` is {}, not a string", describe(other)),
                );
                None
            }
        }
    }

    fn hash(&mut self, path: &str, v: &V<'_>) -> Option<String> {
        match v.get_ref() {
            DeValue::String(s) if is_hash(s) => Some(s.to_string()),
            DeValue::String(s) => {
                self.problem(
                    sp(v),
                    format!("the hash of `{path}`, `{s}`, isn't `sha256:` and 64 lowercase hexadecimal digits"),
                );
                None
            }
            other => {
                self.problem(
                    sp(v),
                    format!("the hash of `{path}` is {}, not a string", describe(other)),
                );
                None
            }
        }
    }
}

/// A commit's full hash: 40 hexadecimal digits (SHA-1), or 64 (SHA-256).
pub fn is_commit(text: &str) -> bool {
    matches!(text.len(), 40 | 64) && text.bytes().all(|b| b.is_ascii_hexdigit())
}

fn is_hash(text: &str) -> bool {
    text.strip_prefix("sha256:").is_some_and(|hex| {
        hex.len() == 64
            && hex
                .bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const COMMIT: &str = "9f2c41d0e0c4a1b2c3d4e5f60718293a4b5c6d7e";

    fn lock() -> Lock {
        let mut files = BTreeMap::new();
        files.insert(
            "src/a \"b\".rs".to_owned(),
            LockedFile {
                hash: file_hash(b"fn a() {}\n"),
                span: Span::empty(0),
            },
        );
        Lock {
            sources: vec![
                LockedSource {
                    name: "web".into(),
                    git: "git@github.com:acme/web.git".into(),
                    commit: COMMIT.into(),
                    files: BTreeMap::new(),
                    span: Span::empty(0),
                },
                LockedSource {
                    name: "api".into(),
                    git: "https://github.com/acme/api.git".into(),
                    commit: COMMIT.into(),
                    files,
                    span: Span::empty(0),
                },
            ],
        }
    }

    #[test]
    fn hashes() {
        assert_eq!(
            file_hash(b""),
            "sha256:e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }

    #[test]
    fn written_and_read_back() {
        let text = lock().to_toml();
        assert!(text.find("name = \"api\"") < text.find("name = \"web\""));
        let read = Lock::parse(&text, FileId::new(0)).expect("it reads back");
        assert_eq!(read.sources.len(), 2);
        let api = read.source("api").expect("api");
        assert_eq!(api.commit, COMMIT);
        assert_eq!(api.files["src/a \"b\".rs"].hash, file_hash(b"fn a() {}\n"));
        assert!(read.source("web").expect("web").files.is_empty());
        // The same lock writes the same text.
        assert_eq!(read.to_toml(), text);
    }

    #[test]
    fn problems() {
        let bad = |text: &str| {
            Lock::parse(text, FileId::new(0))
                .expect_err(text)
                .iter()
                .map(|i| i.args.iter().map(|a| a.value.clone()).collect::<String>())
                .collect::<Vec<_>>()
        };
        assert!(bad("version = ")[0].contains("string"));
        assert_eq!(
            bad("version = 2\n"),
            ["it's version 2, and this version of Ascribe reads version 1"]
        );
        assert_eq!(bad(""), ["it has no `version`"]);
        let source =
            format!("version = 1\n[[source]]\nname = \"a\"\ngit = \"x\"\ncommit = \"{COMMIT}\"\n");
        assert!(Lock::parse(&source, FileId::new(0)).is_ok());
        assert_eq!(
            bad(&format!(
                "{source}[[source]]\nname = \"a\"\ngit = \"x\"\ncommit = \"{COMMIT}\"\n"
            )),
            ["source `a` is pinned twice"]
        );
        assert_eq!(
            bad("version = 1\n[[source]]\nname = \"a\"\ngit = \"x\"\ncommit = \"abc\"\n"),
            ["`abc` isn't a commit's full hash"]
        );
        assert_eq!(
            bad(&format!("{source}[source.files]\n\"a.rs\" = \"md5:1\"\n")),
            ["the hash of `a.rs`, `md5:1`, isn't `sha256:` and 64 lowercase hexadecimal digits"]
        );
        assert_eq!(
            bad("version = 1\n[[source]]\ngit = \"x\"\n"),
            [
                "a `[[source]]` has no `name`",
                "a `[[source]]` has no `commit`"
            ]
        );
    }
}
