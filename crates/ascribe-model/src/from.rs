//! `[from."<path>"]` (SPEC §7.9): the tables and keys a project takes from
//! another project's `ascribe.toml`.
//!
//! Before the sections are read, each key a `take` list names is read from
//! the other file and put in this file's tree, as if written here, so every
//! rule of the loader holds it as it holds a key written here. Its spans are
//! the name in `take`: a problem with a taken value is reported there, and the
//! editor's Go to Definition on the name opens the other file
//! ([`crate::Taken`]). The other file's own `[from]` isn't followed.
//!
//! The file is read through [`TakenFiles`], which the caller gives: the
//! project's `FileSystem`, which knows exact-case names and symbolic links.
//! Without one, as for [`crate::load_str`], `[from]` is checked as it's
//! written, and nothing is taken.

use std::borrow::Cow;
use std::io;
use std::ops::Range;

use ascribe_core::{RelPath, Span, diagnostics};
use toml::Spanned;
use toml::de::{DeTable, DeValue};

use crate::SourceBound;
use crate::loader::Loader;
use crate::model::Taken;
use crate::toml_util::{V, entries, sp};

/// Where the loader reads the files `[from]` names.
pub trait TakenFiles {
    /// The text of the file at `project_path`, relative to the folder
    /// `ascribe.toml` is in. The path may start with `..`: the loader has
    /// already decided it may be read.
    ///
    /// # Errors
    ///
    /// The file isn't there (`NotFound`), or can't be read as text.
    fn read_taken(&self, project_path: &RelPath) -> io::Result<String>;
}

/// The keys a project may not take: what makes it a project of its own.
const OWN: &[&str] = &["spec", "project", "from"];

/// One `[from."<path>"]` table, as written.
pub(crate) struct FromTable {
    /// The path, as written.
    file: String,
    file_span: Span,
    /// Each name in `take`: its text, its segments, and its span.
    takes: Vec<(String, Vec<String>, Span)>,
}

/// The paths `[from]` names in `text`, as written, without checking them: for
/// the editor, which watches them even when the model doesn't load.
pub fn from_files(text: &str) -> Vec<String> {
    let Ok(parsed) = DeTable::parse(text) else {
        return Vec::new();
    };
    match parsed.get_ref().get("from").map(Spanned::get_ref) {
        Some(DeValue::Table(t)) => entries(t).map(|(key, _, _)| key.to_owned()).collect(),
        _ => Vec::new(),
    }
}

/// Where the key at `keys` of the content model in `text` is written, when
/// `[from]` takes it from another file (SPEC §7.9): what `take` names that
/// holds it, the key itself or a table around it, with the file, as `[from]`
/// writes both. `None` when this file holds it, or the text isn't TOML.
///
/// An editor of the content model writes a key where this says it's
/// written. Paths are the same in both files, since what's taken is read at
/// the path it has in the other.
pub fn taken_from(text: &str, keys: &[&str]) -> Option<TakenName> {
    named_in_take(text, |taken| {
        keys.len() >= taken.len() && keys[..taken.len()] == *taken
    })
}

/// What `take` names inside the table at `keys` of the content model in
/// `text`, a key of it: a table around a taken key, which an edit of the
/// whole table would split between two files. `None` when nothing is.
pub fn taken_inside(text: &str, keys: &[&str]) -> Option<TakenName> {
    named_in_take(text, |taken| {
        keys.len() < taken.len() && taken[..keys.len()] == *keys
    })
}

/// The first name in a `take` list that `matches`, by its segments.
fn named_in_take(text: &str, matches: impl Fn(&[&str]) -> bool) -> Option<TakenName> {
    let parsed = DeTable::parse(text).ok()?;
    let DeValue::Table(from) = parsed.get_ref().get("from")?.get_ref() else {
        return None;
    };
    for (file, _, table) in entries(from) {
        let DeValue::Table(table) = table.get_ref() else {
            continue;
        };
        let Some(DeValue::Array(take)) = table.get("take").map(Spanned::get_ref) else {
            continue;
        };
        for name in take.iter() {
            let DeValue::String(name) = name.get_ref() else {
                continue;
            };
            let segments: Vec<&str> = name.split('.').collect();
            if matches(&segments) {
                return Some(TakenName {
                    name: name.to_string(),
                    file: file.to_owned(),
                });
            }
        }
    }
    None
}

/// A name in a `take` list, and the file it's taken from.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TakenName {
    /// The name, as `take` writes it: `releases`, `consumer.site`.
    pub name: String,
    /// The file, as `[from]` writes it: a path from the folder
    /// `ascribe.toml` is in.
    pub file: String,
}

/// `path`, written relative to the folder `base` names (a path relative to
/// the project root), as a path relative to the project root, with `.` and
/// `..` worked out: a source's `path` in a taken table, which is from the
/// other file's folder.
fn rebase(base: &str, path: &str) -> String {
    let mut out: Vec<&str> = Vec::new();
    for segment in base.split('/').chain(path.split('/')) {
        match segment {
            "" | "." => {}
            ".." if out.last().is_some_and(|last| *last != "..") => {
                out.pop();
            }
            other => out.push(other),
        }
    }
    if out.is_empty() {
        ".".to_owned()
    } else {
        out.join("/")
    }
}

/// The folder a path written in `[from]` is in: everything before its last
/// `/`, or the project root.
fn folder_of(file: &str) -> &str {
    file.rsplit_once('/').map_or("", |(folder, _)| folder)
}

/// A value from the other file, with every span in it, its keys' too, set
/// to `span`: the name in `take` that brought it.
fn respan<'i>(v: V<'i>, span: Range<usize>) -> V<'i> {
    let value = match v.into_inner() {
        DeValue::Array(items) => DeValue::Array(
            items
                .into_iter()
                .map(|item| respan(item, span.clone()))
                .collect(),
        ),
        DeValue::Table(t) => DeValue::Table(
            t.into_iter()
                .map(|(k, item)| {
                    (
                        Spanned::new(span.clone(), k.into_inner()),
                        respan(item, span.clone()),
                    )
                })
                .collect(),
        ),
        other => other,
    };
    Spanned::new(span, value)
}

/// A source's `path` in a taken value, rebased from the other file's folder,
/// `base`. `segments` are what `take` names: `sources`, a source, or its
/// `path`.
fn rebase_sources(value: &mut DeValue<'_>, segments: &[String], base: &str) {
    let rebased = |s: &mut DeValue<'_>| {
        if let DeValue::String(path) = s {
            *path = Cow::Owned(rebase(base, path));
        }
    };
    let source = |t: &mut DeValue<'_>| {
        if let DeValue::Table(t) = t
            && let Some(path) = t.get_mut("path")
        {
            rebased(path.get_mut());
        }
    };
    match segments {
        [_] => {
            if let DeValue::Table(t) = value {
                for (_, s) in t.iter_mut() {
                    source(s.get_mut());
                }
            }
        }
        [_, _] => source(value),
        [_, _, key] if key == "path" => rebased(value),
        _ => {}
    }
}

/// The line, from 1, and the column, in characters from 0, that `offset`
/// is at in `text`.
fn line_column(text: &str, offset: usize) -> (usize, usize) {
    let before = text.get(..offset).unwrap_or(text);
    let line_start = before.rfind('\n').map_or(0, |i| i + 1);
    let line = before.matches('\n').count() + 1;
    let column = before.get(line_start..).map_or(0, |s| s.chars().count());
    (line, column)
}

/// A TOML value as `toml` holds it, for [`Taken::value`].
fn plain(v: &DeValue<'_>) -> toml::Value {
    match v {
        DeValue::String(s) => toml::Value::String(s.to_string()),
        DeValue::Integer(i) => {
            let text = i.as_str().replace('_', "");
            let n = match i.radix() {
                10 => text.parse::<i64>().ok(),
                radix => {
                    let (neg, rest) = match text.strip_prefix('-') {
                        Some(r) => (true, r),
                        None => (false, text.trim_start_matches('+')),
                    };
                    i64::from_str_radix(rest.get(2..).unwrap_or(""), radix)
                        .ok()
                        .map(|n| if neg { -n } else { n })
                }
            };
            toml::Value::Integer(n.unwrap_or(0))
        }
        DeValue::Float(f) => {
            toml::Value::Float(f.as_str().replace('_', "").parse::<f64>().unwrap_or(0.0))
        }
        DeValue::Boolean(b) => toml::Value::Boolean(*b),
        DeValue::Datetime(d) => toml::Value::Datetime(*d),
        DeValue::Array(a) => toml::Value::Array(a.iter().map(|x| plain(x.get_ref())).collect()),
        DeValue::Table(t) => toml::Value::Table(
            entries(t)
                .map(|(k, _, x)| (k.to_owned(), plain(x.get_ref())))
                .collect(),
        ),
    }
}

/// The value at `segments` in `t`, with the span of its last key.
fn find<'a, 'i>(t: &'a DeTable<'i>, segments: &[String]) -> Option<(&'a V<'i>, Span)> {
    let (first, rest) = segments.split_first()?;
    let (key, value) = t.get_key_value(first.as_str())?;
    if rest.is_empty() {
        return Some((value, sp(key)));
    }
    match value.get_ref() {
        DeValue::Table(inner) => find(inner, rest),
        _ => None,
    }
}

/// Whether anything is written at `segments` in `t`: the key, or a value
/// that isn't a table where a table would hold it.
fn written(t: &DeTable<'_>, segments: &[String]) -> bool {
    let Some((first, rest)) = segments.split_first() else {
        return false;
    };
    match t.get(first.as_str()).map(Spanned::get_ref) {
        None => false,
        Some(_) if rest.is_empty() => true,
        Some(DeValue::Table(inner)) => written(inner, rest),
        Some(_) => true,
    }
}

/// Puts `value` at `segments` in `t`, making the tables on the way, each
/// spanning `span`. [`written`] has said nothing's there.
fn put<'i>(t: &mut DeTable<'i>, segments: &[String], value: V<'i>, span: &Range<usize>) {
    let Some((first, rest)) = segments.split_first() else {
        return;
    };
    let key = Spanned::new(span.clone(), Cow::Owned(first.clone()));
    if rest.is_empty() {
        t.insert(key, value);
        return;
    }
    if t.get(first.as_str()).is_none() {
        t.insert(
            key,
            Spanned::new(span.clone(), DeValue::Table(DeTable::default())),
        );
    }
    if let Some(DeValue::Table(inner)) = t.get_mut(first.as_str()).map(Spanned::get_mut) {
        put(inner, rest, value, span);
    }
}

impl Loader<'_> {
    /// `[from]` as written: each table's path and its `take` list, each
    /// name checked. Reports the shape's problems.
    pub(crate) fn froms(&mut self, root: &DeTable<'_>) -> Vec<FromTable> {
        let mut out: Vec<FromTable> = Vec::new();
        let Some(v) = root.get("from") else {
            return out;
        };
        let Some(t) = self.as_table("from", v) else {
            return out;
        };
        let mut seen: Vec<(Vec<String>, String)> = Vec::new();
        for (file, file_span, item) in entries(t) {
            let path = crate::toml_util::join("from", file);
            let Some(ft) = self.as_table(&path, item) else {
                continue;
            };
            self.check_table(&path, ft, "from.<path>");
            if file.trim().is_empty() {
                self.push(
                    self.issue(diagnostics::MODEL_EMPTY_TEXT, file_span)
                        .with_arg("key", path),
                );
                continue;
            }
            if crate::loader::is_absolute(file) {
                self.push(
                    self.issue(diagnostics::MODEL_PATH_ABSOLUTE, file_span)
                        .with_arg("key", path),
                );
                continue;
            }
            let Some(list) = self.require(&path, ft, file_span, "take") else {
                continue;
            };
            let Some(names) = self.strings(&format!("{path}.take"), list) else {
                continue;
            };
            let mut takes = Vec::new();
            for (name, span) in names {
                let segments: Vec<String> = name.split('.').map(str::to_owned).collect();
                if segments.iter().any(|s| s.trim().is_empty()) {
                    self.push(
                        self.issue(diagnostics::MODEL_FROM_TAKE, span)
                            .with_variant("name")
                            .with_arg("key", name.as_str()),
                    );
                    continue;
                }
                if let Some(own) = segments.first().filter(|s| OWN.contains(&s.as_str())) {
                    self.push(
                        self.issue(diagnostics::MODEL_FROM_TAKE, span)
                            .with_variant("own")
                            .with_arg("key", own.as_str()),
                    );
                    continue;
                }
                let overlap = seen.iter().find(|(other, _)| {
                    let n = other.len().min(segments.len());
                    other[..n] == segments[..n]
                });
                if let Some((other, other_file)) = overlap {
                    self.push(
                        self.issue(diagnostics::MODEL_FROM_TAKE, span)
                            .with_variant("twice")
                            .with_arg("key", name.as_str())
                            .with_arg("other", other.join("."))
                            .with_arg("file", other_file.as_str()),
                    );
                    continue;
                }
                seen.push((segments.clone(), file.to_owned()));
                takes.push((name, segments, span));
            }
            out.push(FromTable {
                file: file.to_owned(),
                file_span,
                takes,
            });
        }
        out
    }

    /// The text of each file `tables` names, read through the caller's
    /// [`TakenFiles`], when the project's folder is known and the file may be
    /// read: inside the project's repository, or with none, inside a folder
    /// [`SourceBound::Within`] names. `None` for one that isn't read, its
    /// problem reported.
    pub(crate) fn taken_texts(&mut self, tables: &[FromTable]) -> Vec<Option<String>> {
        tables.iter().map(|t| self.taken_text(t)).collect()
    }

    fn taken_text(&mut self, table: &FromTable) -> Option<String> {
        let files = self.files?;
        let dir = self.project_dir?;
        let file = table.file.as_str();
        let span = table.file_span;
        let resolved = self.resolve(file);
        match crate::loader::repository_root(dir) {
            Some(repository) => {
                if !resolved.starts_with(&repository) {
                    self.push(
                        self.issue(diagnostics::MODEL_FROM_FILE, span)
                            .with_variant("outside-repository")
                            .with_arg("path", file),
                    );
                    return None;
                }
            }
            None => {
                if let SourceBound::Within(opened) = self.bound {
                    // Outside FileSystem: the folders the user opened, compared
                    // as the file's path is, with links resolved.
                    let inside = opened.iter().any(|o| {
                        let o = o
                            .canonicalize()
                            .unwrap_or_else(|_| ascribe_core::path::normalize(o));
                        resolved.starts_with(o)
                    });
                    if !inside {
                        self.push(
                            self.issue(diagnostics::MODEL_FROM_FILE, span)
                                .with_variant("opened")
                                .with_arg("path", file),
                        );
                        return None;
                    }
                }
            }
        }
        let read = RelPath::parse(file)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e.to_string()))
            .and_then(|path| files.read_taken(&path));
        match read {
            Ok(text) => Some(text),
            Err(e) if e.kind() == io::ErrorKind::NotFound => {
                self.push(
                    self.issue(diagnostics::MODEL_FROM_FILE, span)
                        .with_arg("path", file),
                );
                None
            }
            Err(e) => {
                self.push(
                    self.issue(diagnostics::MODEL_FROM_FILE, span)
                        .with_variant("unreadable")
                        .with_arg("path", file)
                        .with_arg("detail", e.to_string()),
                );
                None
            }
        }
    }

    /// Puts what each table takes into `root`, from the other file in
    /// `texts`, and returns where each came from. A name the other file
    /// doesn't have, or that this file writes too, is reported, and nothing
    /// is put for it.
    pub(crate) fn take<'t>(
        &mut self,
        root: &mut DeTable<'t>,
        tables: &[FromTable],
        texts: &'t [Option<String>],
    ) -> Vec<Taken> {
        let mut out = Vec::new();
        for (table, text) in tables.iter().zip(texts) {
            let Some(text) = text else { continue };
            let other = match DeTable::parse(text) {
                Ok(t) => t,
                Err(e) => {
                    self.push(
                        self.issue(diagnostics::MODEL_FROM_FILE, table.file_span)
                            .with_variant("unreadable")
                            .with_arg("path", table.file.as_str())
                            .with_arg(
                                "detail",
                                format!("it isn't valid TOML ({})", e.message().trim()),
                            ),
                    );
                    continue;
                }
            };
            let other = other.get_ref();
            for (name, segments, span) in &table.takes {
                let Some((value, at)) = find(other, segments) else {
                    self.push(
                        self.issue(diagnostics::MODEL_FROM_TAKE, *span)
                            .with_arg("key", name.as_str())
                            .with_arg("file", table.file.as_str()),
                    );
                    continue;
                };
                if written(root, segments) {
                    self.push(
                        self.issue(diagnostics::MODEL_FROM_WRITTEN, *span)
                            .with_arg("key", name.as_str())
                            .with_arg("file", table.file.as_str()),
                    );
                    continue;
                }
                let range = span.start()..span.end();
                let mut value = respan(value.clone(), range.clone());
                if segments.first().is_some_and(|s| s == "sources") {
                    rebase_sources(value.get_mut(), segments, folder_of(&table.file));
                }
                let (line, column) = line_column(text, at.start());
                out.push(Taken {
                    key: name.clone(),
                    file: table.file.clone(),
                    span: *span,
                    line,
                    column,
                    value: plain(value.get_ref()),
                });
                put(root, segments, value, &range);
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_path_is_rebased_from_the_other_files_folder() {
        assert_eq!(rebase("../../docs", ".."), "../..");
        assert_eq!(rebase("../docs", "code"), "../docs/code");
        assert_eq!(rebase("", ".."), "..");
        assert_eq!(rebase("docs", ".."), ".");
        assert_eq!(rebase("../..", "../x"), "../../../x");
    }

    #[test]
    fn a_files_folder_is_before_its_last_slash() {
        assert_eq!(folder_of("../../docs/ascribe.toml"), "../../docs");
        assert_eq!(folder_of("ascribe.toml"), "");
    }

    #[test]
    fn lines_count_from_one_and_columns_from_zero() {
        assert_eq!(line_column("a = 1\n  b = 2\n", 8), (2, 2));
        assert_eq!(line_column("a", 0), (1, 0));
    }

    #[test]
    fn the_files_from_names_are_listed_unchecked() {
        let text = "spec = \"0.1\"\n[from.\"../docs/ascribe.toml\"]\ntake = []\n";
        assert_eq!(from_files(text), vec!["../docs/ascribe.toml".to_owned()]);
        assert!(from_files("not = [toml").is_empty());
    }

    #[test]
    fn a_key_is_written_where_take_says() {
        let text = "spec = \"0.1\"\n[from.\"../docs/ascribe.toml\"]\ntake = [\"releases\", \"consumer.site\"]\n[consumer]\nheaders = \"netlify\"\n";
        let taken = |keys: &[&str]| taken_from(text, keys).map(|t| (t.name, t.file));
        let docs = "../docs/ascribe.toml".to_owned();
        assert_eq!(
            taken(&["releases", "ascribe", "live"]),
            Some(("releases".to_owned(), docs.clone()))
        );
        assert_eq!(
            taken(&["consumer", "site"]),
            Some(("consumer.site".to_owned(), docs.clone()))
        );
        assert_eq!(taken(&["consumer", "host"]), None);
        assert_eq!(taken(&["consumer"]), None);
        assert_eq!(
            taken_inside(text, &["consumer"]).map(|t| t.name),
            Some("consumer.site".to_owned())
        );
        assert_eq!(taken_inside(text, &["releases", "ascribe"]), None);
        assert_eq!(taken_from("not = [toml", &["releases"]), None);
    }
}
