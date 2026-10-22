//! Edits to `ascribe.toml` that keep its comments, blank lines, and order.
//!
//! `toml_edit` reads the file and renders what goes in it, but never writes
//! the file back: an edit that replaced the whole file would lose the cursor
//! and mark every line changed. Each change finds its place from the parsed
//! document's spans (after a table's last entry for an insertion, a value's
//! or a key's span for a replacement), renders only the new text there, and
//! is one [`TextEdit`]. [`ModelFile::finish`] checks the edits: the edited
//! text, parsed, must hold the same values as the document changed through
//! `toml_edit`'s own API. An edit that doesn't is refused, never made by
//! rewriting the file.
//!
//! A key `[from]` takes from another project's `ascribe.toml` (SPEC §7.9) is
//! edited where it's written: a file parsed with the files it takes from
//! ([`ModelTexts`]) sends each change at a taken key to the file that holds
//! it, at the same path, and [`ModelFile::finish`] checks each file's edits
//! on their own.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use ascribe_core::{Span, TextEdit, apply_edits};
use serde_json::{Map, Value as Json};
use toml_edit::{Document, DocumentMut, Item, Key, Table, TableLike, Value};

/// One step of a path into the document.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Seg {
    /// A table's key.
    Key(String),
    /// An array's item.
    Index(usize),
}

/// A path of keys: `["phrases", "product"]`.
pub(crate) fn path(keys: &[&str]) -> Vec<Seg> {
    keys.iter().map(|k| Seg::Key((*k).to_owned())).collect()
}

/// A change as `toml_edit`'s API makes it, which a text edit must agree with.
#[derive(Clone, Debug)]
pub(crate) enum Change {
    /// Sets the value at a path, adding the tables on the way that aren't
    /// there.
    Set(Vec<Seg>, Value),
    /// Renames the last key of a path, keeping its value.
    RenameKey(Vec<Seg>, String),
    /// Removes the last key of a path, with its value.
    Remove(Vec<Seg>),
}

/// A text edit, the change it makes, and the file it's in: `None` for
/// `ascribe.toml`, or a file it takes from, as `[from]` writes its path.
#[derive(Clone, Debug)]
pub(crate) struct Edit {
    edit: TextEdit,
    change: Change,
    file: Option<String>,
}

impl Edit {
    fn new(edit: TextEdit, change: Change) -> Edit {
        Edit {
            edit,
            change,
            file: None,
        }
    }

    /// The same edit, made in `file`, a file `[from]` takes from.
    fn in_taken(self, file: &TakenFile) -> Edit {
        Edit {
            file: Some(file.file.clone()),
            ..self
        }
    }
}

/// Another project's `ascribe.toml` that `[from]` takes from (SPEC §7.9),
/// as an edit reads it and leaves it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TakenFile {
    /// Its path, as `[from]` writes it: from the folder `ascribe.toml` is in.
    pub file: String,
    /// Its path, absolute.
    pub path: PathBuf,
    /// Its text.
    pub text: String,
}

/// `ascribe.toml`'s text, and each file `[from]` takes from that could be
/// read, for edits that write a taken key where it's written. Edits are made
/// to it in steps, each parsed from what the step before left.
#[derive(Clone, Debug)]
pub(crate) struct ModelTexts {
    /// `ascribe.toml`'s text.
    pub text: String,
    /// The files it takes from.
    taken: Vec<TakenFile>,
    /// Which of `taken` an edit changed.
    changed: Vec<bool>,
}

impl ModelTexts {
    /// `text`, with the files of `taken` it takes from, as read already.
    pub(crate) fn new(text: impl Into<String>, taken: Vec<TakenFile>) -> ModelTexts {
        ModelTexts {
            text: text.into(),
            changed: vec![false; taken.len()],
            taken,
        }
    }

    /// The texts, parsed.
    pub(crate) fn parse(&self) -> Result<ModelFile<'_>, String> {
        ModelFile::parse_taking(&self.text, &self.taken)
    }

    /// The texts after `finished`, the edits of a step.
    pub(crate) fn apply(&mut self, finished: &Finished) {
        self.text.clone_from(&finished.text);
        for changed in &finished.taken {
            if let Some(i) = self.taken.iter().position(|t| t.file == changed.file.file) {
                self.taken[i].text.clone_from(&changed.file.text);
                self.changed[i] = true;
            }
        }
    }

    /// The files it takes from that the edits changed, with their texts now.
    pub(crate) fn changed(&self) -> Vec<TakenFile> {
        self.taken
            .iter()
            .zip(&self.changed)
            .filter(|(_, changed)| **changed)
            .map(|(t, _)| t.clone())
            .collect()
    }
}

/// The files the content model in `text`, the text of `config`, takes
/// from with `[from]`, read from the disk as the loader reads them. One that
/// can't be read is left out, so an edit of a key it holds is refused.
pub fn read_taken(text: &str, config: &Path) -> Vec<TakenFile> {
    let root = config.parent().unwrap_or(Path::new("."));
    ascribe_model::from_files(text)
        .into_iter()
        .filter_map(|file| {
            let read = ascribe_check::type_files::read_taken(root, &file).ok()?;
            Some(TakenFile {
                path: ascribe_core::path::normalize(&root.join(&file)),
                file,
                text: read,
            })
        })
        .collect()
}

/// The edits of a step, checked: `ascribe.toml`'s and each other file's.
#[derive(Clone, Debug)]
pub(crate) struct Finished {
    /// The edits to `ascribe.toml`, sorted.
    pub edits: Vec<TextEdit>,
    /// `ascribe.toml`'s text after them.
    pub text: String,
    /// Each file `[from]` takes from that the step changes.
    pub taken: Vec<FinishedTaken>,
}

/// The edits to a file `[from]` takes from.
#[derive(Clone, Debug)]
pub(crate) struct FinishedTaken {
    /// The file, with its text after the edits.
    pub file: TakenFile,
    /// Its text before them.
    pub before: String,
    /// The edits, sorted.
    pub edits: Vec<TextEdit>,
}

impl Finished {
    /// The files `[from]` takes from that the step changes, as the loader
    /// reads them: each path as `[from]` writes it, and its text after.
    pub(crate) fn taken_texts(&self) -> Vec<(String, String)> {
        self.taken
            .iter()
            .map(|t| (t.file.file.clone(), t.file.text.clone()))
            .collect()
    }

    /// The edits to the files `[from]` takes from as the client's, by the
    /// file's URI, with `encoding`'s columns. `None` when a file can't be
    /// named as a URI.
    pub(crate) fn taken_changes(
        &self,
        encoding: crate::position::Encoding,
    ) -> Option<Vec<(lsp_types::Uri, Vec<lsp_types::TextEdit>)>> {
        self.taken
            .iter()
            .map(|t| {
                let index = ascribe_core::LineIndex::new(&t.before);
                let edits = t
                    .edits
                    .iter()
                    .map(|e| lsp_types::TextEdit {
                        range: encoding.range(&index, e.span),
                        new_text: e.new_text.clone(),
                    })
                    .collect();
                Some((crate::uri::path_to_uri(&t.file.path)?, edits))
            })
            .collect()
    }
}

/// `ascribe.toml`'s text, parsed with the spans of everything in it, and the
/// files `[from]` takes from, parsed too.
pub(crate) struct ModelFile<'a> {
    text: &'a str,
    doc: Document<&'a str>,
    nl: &'static str,
    taken: Vec<(&'a TakenFile, ModelFile<'a>)>,
}

/// The keys at the start of a path, up to an array's item.
fn keys(at: &[Seg]) -> Vec<&str> {
    at.iter()
        .map_while(|seg| match seg {
            Seg::Key(key) => Some(key.as_str()),
            Seg::Index(_) => None,
        })
        .collect()
}

impl<'a> ModelFile<'a> {
    /// Parses the text alone: a change at a key `[from]` takes is refused.
    pub(crate) fn parse(text: &'a str) -> Result<ModelFile<'a>, String> {
        ModelFile::parse_taking(text, &[])
    }

    /// Parses the text, and each file of `taken` it takes from, so a change
    /// at a taken key is made in the file that holds it. One of `taken`
    /// that isn't TOML is left out.
    pub(crate) fn parse_taking(
        text: &'a str,
        taken: &'a [TakenFile],
    ) -> Result<ModelFile<'a>, String> {
        let doc = Document::parse(text)
            .map_err(|e| format!("`ascribe.toml` isn't valid TOML: {}", e.message()))?;
        Ok(ModelFile {
            text,
            doc,
            nl: if text.contains("\r\n") { "\r\n" } else { "\n" },
            taken: taken
                .iter()
                .filter_map(|t| Some((t, ModelFile::parse(&t.text).ok()?)))
                .collect(),
        })
    }

    /// The item at a path, in the file that holds it.
    pub(crate) fn get(&self, path: &[Seg]) -> Option<&Item> {
        match self.holder(path) {
            Ok(Some(i)) => self.taken[i].1.get(path),
            Err(_) if ascribe_model::taken_from(self.text, &keys(path)).is_some() => None,
            _ => self.get_here(path),
        }
    }

    /// The item at a path, in this file.
    fn get_here(&self, path: &[Seg]) -> Option<&Item> {
        let mut item = self.doc.as_item();
        for seg in path {
            item = match seg {
                Seg::Key(key) => item.get(key.as_str())?,
                Seg::Index(i) => item.get(*i)?,
            };
        }
        Some(item)
    }

    /// Which file holds what's at `at` (SPEC §7.9): `None` for this one, or
    /// the index of the file `[from]` takes it from. Refused when that file
    /// couldn't be read, or when `at` is a table `[from]` takes a key of,
    /// which a change would split between two files.
    fn holder(&self, at: &[Seg]) -> Result<Option<usize>, String> {
        let keys = keys(at);
        if let Some(taken) = ascribe_model::taken_from(self.text, &keys) {
            return self
                .taken
                .iter()
                .position(|(t, _)| t.file == taken.file)
                .map(Some)
                .ok_or_else(|| {
                    format!(
                        "`{}` is taken from `{}` by `[from]`, and that file can't be read, so it can't be changed. Edit `{}`.",
                        taken.name, taken.file, taken.file
                    )
                });
        }
        if let Some(taken) = ascribe_model::taken_inside(self.text, &keys) {
            return Err(format!(
                "`{}` is taken from `{}` by `[from]`, so `{}` is written in two files and can't be changed whole.",
                taken.name,
                taken.file,
                keys.join(".")
            ));
        }
        Ok(None)
    }

    /// The key of the table-like item at `table` that `offset` is on, with
    /// its span, in this file.
    pub(crate) fn key_at(&self, table: &[&str], offset: usize) -> Option<(String, Span)> {
        let t = self.get_here(&path(table))?.as_table_like()?;
        t.iter().find_map(|(name, _)| {
            let (key, _) = t.get_key_value(name)?;
            let span = key.span()?;
            (span.start <= offset && offset <= span.end)
                .then(|| (name.to_owned(), Span::new(span.start, span.end)))
        })
    }

    /// The table-like item at a path: a `[header]` table, a dotted one, or an
    /// inline table.
    pub(crate) fn table_like(&self, path: &[Seg]) -> Option<&dyn TableLike> {
        self.get(path)?.as_table_like()
    }

    /// Adds `key = value` after the last entry of the `[header]` table at
    /// `table`, or, when the file has no such table, adds the table at the
    /// end of the file.
    pub(crate) fn add_entry(
        &self,
        table: &[&str],
        key: &str,
        value: Value,
    ) -> Result<Edit, String> {
        let full: Vec<Seg> = path(table)
            .into_iter()
            .chain([Seg::Key(key.to_owned())])
            .collect();
        if let Some(i) = self.holder(&full)? {
            return Ok(self.taken[i]
                .1
                .add_entry(table, key, value)?
                .in_taken(self.taken[i].0));
        }
        let change = Change::Set(full, value.clone());
        let key_text = Key::new(key).display_repr().into_owned();
        match self.get(&path(table)) {
            None => {
                let header = table
                    .iter()
                    .map(|k| Key::new(*k).display_repr().into_owned())
                    .collect::<Vec<_>>()
                    .join(".");
                let nl = self.nl;
                let sep = self.separator(None);
                let text = format!("[{header}]{nl}{key_text}{sep}{value}{nl}");
                Ok(Edit::new(self.append(&text), change))
            }
            Some(Item::Table(t)) if table.is_empty() => {
                // The top level: after its last value, or before the first
                // table and the comment that touches its header.
                let last = t
                    .iter()
                    .filter_map(|(k, _)| t.get_key_value(k))
                    .filter(|(_, item)| item.is_value())
                    .max_by_key(|(_, item)| item.span().map_or(0, |s| s.end));
                let sep = self.separator(last);
                let line = format!("{key_text}{sep}{value}");
                let edit = match last.and_then(|(_, item)| item.span()) {
                    Some(span) => {
                        let at = self.text[span.end..]
                            .find('\n')
                            .map_or(self.text.len(), |i| span.end + i + 1);
                        self.insert_line(at, &line)
                    }
                    None => match self.first_header() {
                        Some(at) => TextEdit::insert(at, format!("{line}{nl}{nl}", nl = self.nl)),
                        None => self.append(&format!("{line}{}", self.nl)),
                    },
                };
                Ok(Edit::new(edit, change))
            }
            Some(Item::Table(t)) if !t.is_dotted() && !t.is_implicit() => {
                let (at, last) = self.after_entries(t);
                let indent = last.map_or("", |(k, _)| self.indent(k));
                let sep = self.separator(last);
                let line = format!("{indent}{key_text}{sep}{value}");
                Ok(Edit::new(self.insert_line(at, &line), change))
            }
            Some(_) => Err(format!(
                "`ascribe.toml` writes `{}` in a form this action can't add to. Add `{key}` to it by hand.",
                table.join(".")
            )),
        }
    }

    /// Adds a `[header]` table with these entries after the last `[header]`
    /// table whose path starts with `after`, or at the end of the file when
    /// there's none.
    pub(crate) fn add_table(
        &self,
        table: &[&str],
        entries: &[(&str, Value)],
        after: &[&str],
    ) -> Result<Edit, String> {
        if let Some(i) = self.holder(&path(table))? {
            return Ok(self.taken[i]
                .1
                .add_table(table, entries, after)?
                .in_taken(self.taken[i].0));
        }
        if self.get(&path(table)).is_some() {
            return Err(format!(
                "`ascribe.toml` already has `[{}]`.",
                table.join(".")
            ));
        }
        let nl = self.nl;
        let sep = self.separator(None);
        let header = table
            .iter()
            .map(|k| Key::new(*k).display_repr().into_owned())
            .collect::<Vec<_>>()
            .join(".");
        let mut text = format!("[{header}]{nl}");
        let mut inline = toml_edit::InlineTable::new();
        for (key, value) in entries {
            let key_text = Key::new(*key).display_repr().into_owned();
            text.push_str(&format!("{key_text}{sep}{value}{nl}"));
            inline.insert(*key, value.clone());
        }
        let change = Change::Set(path(table), Value::InlineTable(inline));
        let mut last: Option<&Table> = None;
        if let Some(item) = self.get(&path(after)) {
            headers(item, &mut |t| {
                if last.is_none_or(|l| l.span().map(|s| s.start) < t.span().map(|s| s.start)) {
                    last = Some(t);
                }
            });
        }
        let edit = match last {
            None => self.append(&text),
            Some(t) => {
                // After the previous table's last entry, with a blank line
                // between.
                let (at, _) = self.after_entries(t);
                self.insert_line(at, &format!("{nl}{}", text.trim_end()))
            }
        };
        Ok(Edit::new(edit, change))
    }

    /// Replaces the value at `path`, keeping what's around it.
    pub(crate) fn replace_value(&self, at: &[Seg], value: Value) -> Result<Edit, String> {
        if let Some(i) = self.holder(at)? {
            return Ok(self.taken[i]
                .1
                .replace_value(at, value)?
                .in_taken(self.taken[i].0));
        }
        let span = self
            .get(at)
            .and_then(Item::span)
            .ok_or_else(|| "The value can't be found in `ascribe.toml`.".to_owned())?;
        let edit = TextEdit::replace(Span::new(span.start, span.end), value.to_string());
        Ok(Edit::new(edit, Change::Set(at.to_vec(), value)))
    }

    /// The edit to this file's `take` that follows a rename of the last key
    /// of `at` to `new`, when `take` names that key itself, as
    /// `releases.ascribe` or `phrases.version`: the key is renamed in the
    /// file that holds it, and `take` names it by its new name. `None` when
    /// `take` doesn't name it.
    pub(crate) fn retake(&self, at: &[Seg], new: &str) -> Option<Edit> {
        let keys = keys(at);
        let taken = ascribe_model::taken_from(self.text, &keys)?;
        if taken.name != keys.join(".") {
            return None;
        }
        let take = vec![
            Seg::Key("from".to_owned()),
            Seg::Key(taken.file.clone()),
            Seg::Key("take".to_owned()),
        ];
        let i = self
            .get_here(&take)?
            .as_array()?
            .iter()
            .position(|name| name.as_str() == Some(taken.name.as_str()))?;
        let mut renamed = keys[..keys.len() - 1].to_vec();
        renamed.push(new);
        let mut item = take;
        item.push(Seg::Index(i));
        let value = Value::from(renamed.join("."));
        let span = self.get_here(&item)?.span()?;
        Some(Edit::new(
            TextEdit::replace(Span::new(span.start, span.end), value.to_string()),
            Change::Set(item, value),
        ))
    }

    /// Renames the last key of `at`.
    pub(crate) fn rename_key(&self, at: &[Seg], new: &str) -> Result<Edit, String> {
        if let Some(i) = self.holder(at)? {
            return Ok(self.taken[i]
                .1
                .rename_key(at, new)?
                .in_taken(self.taken[i].0));
        }
        let missing = || "The key can't be found in `ascribe.toml`.".to_owned();
        let (Some(Seg::Key(old)), parent) = (at.last(), &at[..at.len().saturating_sub(1)]) else {
            return Err(missing());
        };
        let (key, _) = self
            .table_like(parent)
            .and_then(|t| t.get_key_value(old))
            .ok_or_else(missing)?;
        let span = key.span().ok_or_else(missing)?;
        let edit = TextEdit::replace(
            Span::new(span.start, span.end),
            Key::new(new).display_repr().into_owned(),
        );
        Ok(Edit::new(
            edit,
            Change::RenameKey(at.to_vec(), new.to_owned()),
        ))
    }

    /// Removes `key = value` from the `[header]` table at `table`, with its
    /// line and the comment that touches it from above: the comment lines
    /// directly over the key, with no blank line between, are about it, and
    /// left behind would be about nothing. A comment with a blank line
    /// before the key stays, since it may be about the lines around it too
    /// (decision 8 of the editor's next views).
    pub(crate) fn remove_entry(&self, table: &[&str], key: &str) -> Result<Edit, String> {
        if let Some(i) = self.holder(&path(&[table, &[key]].concat()))? {
            return Ok(self.taken[i]
                .1
                .remove_entry(table, key)?
                .in_taken(self.taken[i].0));
        }
        let refused = || {
            format!(
                "`ascribe.toml` writes `{key}` in `{}` in a form this can't remove. Remove it by hand.",
                table.join(".")
            )
        };
        let Some(Item::Table(t)) = self.get(&path(table)) else {
            return Err(refused());
        };
        if t.is_dotted() || t.is_implicit() {
            return Err(refused());
        }
        let (k, item) = t.get_key_value(key).ok_or_else(refused)?;
        let (Some(k_span), Some(v_span)) = (k.span(), item.span()) else {
            return Err(refused());
        };
        let line = self.text[..k_span.start].rfind('\n').map_or(0, |i| i + 1);
        if !self.text[line..k_span.start].trim().is_empty() {
            return Err(refused());
        }
        let start = self.comment_above(line);
        let end = self.text[v_span.end..]
            .find('\n')
            .map_or(self.text.len(), |i| v_span.end + i + 1);
        let full: Vec<Seg> = path(table)
            .into_iter()
            .chain([Seg::Key(key.to_owned())])
            .collect();
        Ok(Edit::new(
            TextEdit::replace(Span::new(start, end), String::new()),
            Change::Remove(full),
        ))
    }

    /// The edits, checked: applied to each file's text, they give a file
    /// with the same values as the changes made through `toml_edit`'s API.
    /// Returns each file's edits, sorted, and its text after them.
    pub(crate) fn finish(&self, edits: Vec<Edit>) -> Result<Finished, String> {
        let mut here = Vec::new();
        let mut taken: BTreeMap<String, Vec<Edit>> = BTreeMap::new();
        for edit in edits {
            match edit.file.clone() {
                None => here.push(edit),
                Some(file) => taken
                    .entry(file)
                    .or_default()
                    .push(Edit { file: None, ..edit }),
            }
        }
        let (edits, text) = self.finish_here(here)?;
        let mut finished = Finished {
            edits,
            text,
            taken: Vec::new(),
        };
        for (name, edits) in taken {
            let (file, model) = self
                .taken
                .iter()
                .find(|(t, _)| t.file == name)
                .ok_or_else(|| format!("`{name}` can't be read, so it can't be changed."))?;
            let (edits, text) = model.finish_here(edits)?;
            finished.taken.push(FinishedTaken {
                file: TakenFile {
                    text,
                    ..(*file).clone()
                },
                before: file.text.clone(),
                edits,
            });
        }
        Ok(finished)
    }

    /// [`ModelFile::finish`] for the edits to this file.
    fn finish_here(&self, edits: Vec<Edit>) -> Result<(Vec<TextEdit>, String), String> {
        let refused =
            |why: &str| format!("The change can't be made in place in `ascribe.toml`: {why}.");
        let mut expected: DocumentMut = self
            .text
            .parse()
            .map_err(|_| refused("the file isn't valid TOML"))?;
        let mut text_edits = Vec::with_capacity(edits.len());
        for Edit { edit, change, .. } in edits {
            apply_change(expected.as_item_mut(), &change)
                .ok_or_else(|| refused("its place in the file can't be found"))?;
            text_edits.push(edit);
        }
        text_edits.sort_by_key(|e| (e.span.start(), e.span.end()));
        let after =
            apply_edits(self.text, &text_edits).map_err(|_| refused("the edits overlap"))?;
        let edited: DocumentMut = after
            .parse()
            .map_err(|_| refused("the result wouldn't be valid TOML"))?;
        if plain(edited.as_item()) != plain(expected.as_item()) {
            return Err(refused("the result would differ from the change"));
        }
        Ok((text_edits, after))
    }

    /// Where an entry goes after a header table's last value, and that
    /// value's key and item: the start of the line after it, or after the
    /// header when the table has no values.
    fn after_entries(&self, table: &'a Table) -> (usize, Option<(&'a Key, &'a Item)>) {
        let last = table
            .iter()
            .filter_map(|(k, _)| table.get_key_value(k))
            .filter(|(_, item)| item.is_value())
            .max_by_key(|(_, item)| item.span().map_or(0, |s| s.end));
        let end = last
            .and_then(|(_, item)| item.span())
            .or_else(|| table.span())
            .map_or(self.text.len(), |s| s.end);
        let at = self.text[end..]
            .find('\n')
            .map_or(self.text.len(), |i| end + i + 1);
        (at, last)
    }

    /// The start of the first `[header]` table's header, or of the comment
    /// lines that touch it from above; `None` when the file has no table.
    fn first_header(&self) -> Option<usize> {
        let mut first: Option<usize> = None;
        let root = self.doc.as_table();
        headers(self.doc.as_item(), &mut |t| {
            if let Some(span) = t.span().filter(|_| !std::ptr::eq(t, root)) {
                first = Some(first.map_or(span.start, |f| f.min(span.start)));
            }
        });
        let line = self.text[..first?].rfind('\n').map_or(0, |i| i + 1);
        Some(self.comment_above(line))
    }

    /// The start of the comment lines directly above the line that starts at
    /// `line`, with no blank line between; `line` when there are none.
    fn comment_above(&self, line: usize) -> usize {
        let mut start = line;
        while start > 0 {
            let above = self.text[..start - 1].rfind('\n').map_or(0, |i| i + 1);
            if !self.text[above..start].trim_start().starts_with('#') {
                break;
            }
            start = above;
        }
        start
    }

    /// An insertion of `line` as a line of its own at `at`, the start of a
    /// line or the end of the file.
    fn insert_line(&self, at: usize, line: &str) -> TextEdit {
        let nl = self.nl;
        if at == self.text.len() && !self.text.is_empty() && !self.text.ends_with('\n') {
            TextEdit::insert(at, format!("{nl}{line}"))
        } else {
            TextEdit::insert(at, format!("{line}{nl}"))
        }
    }

    /// An insertion of `text` at the end of the file, after a blank line.
    fn append(&self, text: &str) -> TextEdit {
        let nl = self.nl;
        let before = if self.text.trim().is_empty() || self.text.ends_with(&nl.repeat(2)) {
            String::new()
        } else if self.text.ends_with('\n') {
            nl.to_owned()
        } else {
            nl.repeat(2)
        };
        TextEdit::insert(self.text.len(), format!("{before}{text}"))
    }

    /// The whitespace before a key, when it's all whitespace.
    fn indent(&self, key: &Key) -> &'a str {
        let Some(span) = key.span() else {
            return "";
        };
        let start = self.text[..span.start].rfind('\n').map_or(0, |i| i + 1);
        let indent = &self.text[start..span.start];
        if indent.trim().is_empty() { indent } else { "" }
    }

    /// What goes between a key and its value: as `entry` writes it, or as
    /// the file's first entry does, or ` = `.
    fn separator(&self, entry: Option<(&Key, &Item)>) -> &'a str {
        let written = |key: &Key, item: &Item| {
            let (k, v) = (key.span()?, item.span()?);
            let between = self.text.get(k.end..v.start)?;
            (between.trim() == "=" && !between.contains('\n')).then_some(between)
        };
        if let Some((key, item)) = entry {
            return written(key, item).unwrap_or(" = ");
        }
        let mut found = None;
        first_entry(self.doc.as_table(), &mut |key, item| {
            found = found.or_else(|| written(key, item));
        });
        found.unwrap_or(" = ")
    }
}

/// Calls `f` with each `[header]` table at or under `item`.
fn headers<'a>(item: &'a Item, f: &mut dyn FnMut(&'a Table)) {
    if let Item::Table(t) = item {
        if !t.is_implicit() && !t.is_dotted() {
            f(t);
        }
        for (_, child) in t.iter() {
            headers(child, f);
        }
    }
}

/// Calls `f` with the key and item of each value in `table` and the tables
/// under it.
fn first_entry<'a>(table: &'a Table, f: &mut dyn FnMut(&'a Key, &'a Item)) {
    for (k, _) in table.iter() {
        if let Some((key, item)) = table.get_key_value(k) {
            match item {
                Item::Table(t) => first_entry(t, f),
                Item::Value(_) => f(key, item),
                _ => {}
            }
        }
    }
}

/// Makes a change through `toml_edit`'s API. `None` when its path isn't
/// there.
fn apply_change(root: &mut Item, change: &Change) -> Option<()> {
    match change {
        Change::Set(at, value) => {
            let (last, parents) = at.split_last()?;
            let mut item = root;
            for seg in parents {
                item = match seg {
                    Seg::Key(key) => {
                        let table = item.as_table_like_mut()?;
                        if table.get(key).is_none() {
                            let mut new = Table::new();
                            new.set_implicit(true);
                            table.insert(key, Item::Table(new));
                        }
                        table.get_mut(key)?
                    }
                    Seg::Index(i) => item.get_mut(*i)?,
                };
            }
            match last {
                Seg::Key(key) => {
                    item.as_table_like_mut()?
                        .insert(key, Item::Value(value.clone()));
                }
                Seg::Index(i) => *item.get_mut(*i)? = Item::Value(value.clone()),
            }
            Some(())
        }
        Change::RenameKey(at, new) => {
            let (Seg::Key(old), parents) = at.split_last()? else {
                return None;
            };
            let mut item = root;
            for seg in parents {
                item = match seg {
                    Seg::Key(key) => item.get_mut(key.as_str())?,
                    Seg::Index(i) => item.get_mut(*i)?,
                };
            }
            let table = item.as_table_like_mut()?;
            let value = table.remove(old)?;
            table.insert(new, value);
            Some(())
        }
        Change::Remove(at) => {
            let (Seg::Key(old), parents) = at.split_last()? else {
                return None;
            };
            let mut item = root;
            for seg in parents {
                item = match seg {
                    Seg::Key(key) => item.get_mut(key.as_str())?,
                    Seg::Index(i) => item.get_mut(*i)?,
                };
            }
            item.as_table_like_mut()?.remove(old)?;
            Some(())
        }
    }
}

/// An item's values alone, without how they're written, for comparing two
/// documents. An empty implicit table counts as nothing.
fn plain(item: &Item) -> Json {
    match item {
        Item::None => Json::Null,
        Item::Value(v) => plain_value(v),
        Item::Table(t) => table_json(t.iter()),
        Item::ArrayOfTables(a) => Json::Array(a.iter().map(|t| table_json(t.iter())).collect()),
    }
}

fn table_json<'a>(entries: impl Iterator<Item = (&'a str, &'a Item)>) -> Json {
    let map: Map<String, Json> = entries
        .map(|(k, v)| (k.to_owned(), plain(v)))
        .filter(|(_, v)| !v.is_null())
        .collect();
    Json::Object(map)
}

fn plain_value(value: &Value) -> Json {
    match value {
        Value::String(s) => Json::String(s.value().clone()),
        Value::Integer(i) => Json::from(*i.value()),
        Value::Float(f) => Json::String(f.value().to_string()),
        Value::Boolean(b) => Json::Bool(*b.value()),
        Value::Datetime(d) => Json::String(d.value().to_string()),
        Value::Array(a) => Json::Array(a.iter().map(plain_value).collect()),
        Value::InlineTable(t) => {
            let map: Map<String, Json> = t
                .iter()
                .map(|(k, v)| (k.to_owned(), plain_value(v)))
                .collect();
            Json::Object(map)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use toml_edit::Array;

    fn apply(text: &str, edits: Vec<Edit>) -> String {
        let file = ModelFile::parse(text).expect("parses");
        let finished = file.finish(edits).expect("checks");
        assert_eq!(
            apply_edits(text, &finished.edits).expect("applies"),
            finished.text
        );
        finished.text
    }

    const MODEL: &str = "# The content model.\nspec = \"0.1\"\n\n[phrases]\n# The product.\nproduct = \"Quill\"   # its name\n\n# Builds.\n[builds.site]\nvariants = \"switch\"\n";

    #[test]
    fn an_entry_goes_after_its_tables_last_one_keeping_everything_else() {
        let file = ModelFile::parse(MODEL).expect("parses");
        let edit = file
            .add_entry(&["phrases"], "cloud", Value::from("Quill Cloud"))
            .expect("adds");
        assert!(edit.edit.span.is_empty(), "an insertion, not a rewrite");
        assert_eq!(edit.edit.new_text, "cloud = \"Quill Cloud\"\n");
        let after = apply(MODEL, vec![edit]);
        assert_eq!(
            after,
            "# The content model.\nspec = \"0.1\"\n\n[phrases]\n# The product.\nproduct = \"Quill\"   # its name\ncloud = \"Quill Cloud\"\n\n# Builds.\n[builds.site]\nvariants = \"switch\"\n"
        );
    }

    #[test]
    fn a_key_from_takes_is_edited_where_it_s_written() {
        let text = "spec = \"0.1\"\n\n[from.\"../docs/ascribe.toml\"]\ntake = [\"releases\", \"consumer.site\", \"phrases.version\"]\n\n[consumer]\nbase-path = \"/\"\n";
        let docs = TakenFile {
            file: "../docs/ascribe.toml".to_owned(),
            path: PathBuf::from("/repo/docs/ascribe.toml"),
            text: "spec = \"0.1\"\n\n[phrases]\nversion = \"0.4\"\n\n[consumer]\nsite = \"https://a.example\"\n\n[releases.ascribe]\nlive = [\"0.4\"]\n".to_owned(),
        };
        let taken = std::slice::from_ref(&docs);
        let file = ModelFile::parse_taking(text, taken).expect("parses");
        // What's taken is read and changed in the other file.
        assert_eq!(
            file.get(&path(&["consumer", "site"]))
                .and_then(Item::as_str),
            Some("https://a.example")
        );
        let edits = vec![
            file.replace_value(
                &path(&["releases", "ascribe", "live"]),
                Value::from(Array::from_iter(["0.5", "0.4"])),
            )
            .expect("replaces"),
            file.add_entry(
                &["releases", "ascribe"],
                "planned",
                Value::from(Array::from_iter(["0.6"])),
            )
            .expect("adds"),
            file.replace_value(
                &path(&["consumer", "site"]),
                Value::from("https://b.example"),
            )
            .expect("replaces"),
            // A key of the same table that isn't taken is this file's.
            file.add_entry(&["consumer"], "headers", Value::from("netlify"))
                .expect("adds"),
        ];
        let finished = file.finish(edits).expect("checks");
        assert_eq!(
            finished.text,
            "spec = \"0.1\"\n\n[from.\"../docs/ascribe.toml\"]\ntake = [\"releases\", \"consumer.site\", \"phrases.version\"]\n\n[consumer]\nbase-path = \"/\"\nheaders = \"netlify\"\n"
        );
        assert_eq!(finished.taken.len(), 1);
        assert_eq!(finished.taken[0].before, docs.text);
        assert_eq!(
            finished.taken[0].file.text,
            "spec = \"0.1\"\n\n[phrases]\nversion = \"0.4\"\n\n[consumer]\nsite = \"https://b.example\"\n\n[releases.ascribe]\nlive = [\"0.5\", \"0.4\"]\nplanned = [\"0.6\"]\n"
        );
        // A key taken by name is renamed there, and taken by its new name.
        let rename = file
            .rename_key(&path(&["phrases", "version"]), "release")
            .expect("renames");
        let retake = file
            .retake(&path(&["phrases", "version"]), "release")
            .expect("retakes");
        let finished = file.finish(vec![rename, retake]).expect("checks");
        assert!(
            finished
                .text
                .contains("take = [\"releases\", \"consumer.site\", \"phrases.release\"]"),
            "{}",
            finished.text
        );
        assert!(
            finished.taken[0].file.text.contains("release = \"0.4\""),
            "{}",
            finished.taken[0].file.text
        );
        // A table split between the two files isn't changed whole.
        let split = file
            .add_table(&["consumer"], &[], &[])
            .expect_err("refused");
        assert!(split.contains("written in two files"), "{split}");
    }

    #[test]
    fn a_key_from_takes_is_refused_when_its_file_isn_t_read() {
        let text = "spec = \"0.1\"\n\n[from.\"../docs/ascribe.toml\"]\ntake = [\"releases\", \"consumer.site\"]\n\n[consumer]\nbase-path = \"/\"\n";
        let file = ModelFile::parse(text).expect("parses");
        let refused = file
            .replace_value(
                &path(&["releases", "ascribe", "live"]),
                Value::from("0.5.0"),
            )
            .expect_err("refused");
        assert!(
            refused.contains("`releases` is taken from `../docs/ascribe.toml`"),
            "{refused}"
        );
        assert!(
            file.add_entry(&["consumer"], "site", Value::from("https://x.example"))
                .is_err()
        );
        assert!(file.add_table(&["releases", "other"], &[], &[]).is_err());
        // A key of the same table that isn't taken is this file's.
        assert!(
            file.add_entry(&["consumer"], "headers", Value::from("netlify"))
                .is_ok()
        );
    }

    #[test]
    fn an_entry_takes_the_tables_spacing_and_indent() {
        let text = "[phrases]\n  a=\"1\"\n";
        let file = ModelFile::parse(text).expect("parses");
        let edit = file
            .add_entry(&["phrases"], "b", Value::from("2"))
            .expect("adds");
        assert_eq!(apply(text, vec![edit]), "[phrases]\n  a=\"1\"\n  b=\"2\"\n");
    }

    #[test]
    fn a_missing_table_is_made_at_the_end() {
        let text = "spec = \"0.1\"\n";
        let file = ModelFile::parse(text).expect("parses");
        let edit = file
            .add_entry(&["phrases"], "product", Value::from("Quill"))
            .expect("adds");
        assert_eq!(
            apply(text, vec![edit]),
            "spec = \"0.1\"\n\n[phrases]\nproduct = \"Quill\"\n"
        );
        let text = "spec = \"0.1\"";
        let file = ModelFile::parse(text).expect("parses");
        let edit = file
            .add_entry(&["phrases"], "product", Value::from("Quill"))
            .expect("adds");
        assert_eq!(
            apply(text, vec![edit]),
            "spec = \"0.1\"\n\n[phrases]\nproduct = \"Quill\"\n"
        );
    }

    #[test]
    fn a_top_level_entry_goes_after_the_last_one_or_before_the_first_table() {
        let file = ModelFile::parse(MODEL).expect("parses");
        let edit = file
            .add_entry(&[], "version", Value::from("1"))
            .expect("adds");
        assert_eq!(
            apply(MODEL, vec![edit]),
            MODEL.replace("spec = \"0.1\"\n", "spec = \"0.1\"\nversion = \"1\"\n")
        );
        // Before the first table, and the comment that touches its header.
        let text = "# The content model.\n\n# Phrases.\n[phrases]\nproduct = \"Quill\"\n";
        let file = ModelFile::parse(text).expect("parses");
        let edit = file
            .add_entry(&[], "spec", Value::from("0.1"))
            .expect("adds");
        assert_eq!(
            apply(text, vec![edit]),
            "# The content model.\n\nspec = \"0.1\"\n\n# Phrases.\n[phrases]\nproduct = \"Quill\"\n"
        );
        let file = ModelFile::parse("").expect("parses");
        let edit = file
            .add_entry(&[], "spec", Value::from("0.1"))
            .expect("adds");
        assert_eq!(apply("", vec![edit]), "spec = \"0.1\"\n");
    }

    #[test]
    fn an_entry_is_removed_with_the_comment_that_touches_it() {
        let file = ModelFile::parse(MODEL).expect("parses");
        let edit = file.remove_entry(&["phrases"], "product").expect("removes");
        assert_eq!(
            apply(MODEL, vec![edit]),
            MODEL.replace("# The product.\nproduct = \"Quill\"   # its name\n", "")
        );
        // A comment with a blank line before the key stays.
        let text = "[phrases]\n# Names.\n\nproduct = \"Quill\"\ncloud = \"Quill Cloud\"\n";
        let file = ModelFile::parse(text).expect("parses");
        let edit = file.remove_entry(&["phrases"], "product").expect("removes");
        assert_eq!(
            apply(text, vec![edit]),
            "[phrases]\n# Names.\n\ncloud = \"Quill Cloud\"\n"
        );
        // So does the line of the value above, and the header.
        let text = "[phrases]\nproduct = \"Quill\" # the name\ncloud = \"Quill Cloud\"\n";
        let file = ModelFile::parse(text).expect("parses");
        let edit = file.remove_entry(&["phrases"], "cloud").expect("removes");
        assert_eq!(
            apply(text, vec![edit]),
            "[phrases]\nproduct = \"Quill\" # the name\n"
        );
    }

    #[test]
    fn an_empty_table_gets_its_entry_under_the_header() {
        let text = "[phrases] # none yet\n\n[x]\n";
        let file = ModelFile::parse(text).expect("parses");
        let edit = file
            .add_entry(&["phrases"], "a", Value::from("A"))
            .expect("adds");
        assert_eq!(
            apply(text, vec![edit]),
            "[phrases] # none yet\na = \"A\"\n\n[x]\n"
        );
    }

    #[test]
    fn an_entry_after_a_last_line_without_a_line_ending() {
        let text = "[phrases]\na = \"A\"";
        let file = ModelFile::parse(text).expect("parses");
        let edit = file
            .add_entry(&["phrases"], "b", Value::from("B"))
            .expect("adds");
        assert_eq!(apply(text, vec![edit]), "[phrases]\na = \"A\"\nb = \"B\"");
    }

    #[test]
    fn crlf_files_keep_crlf() {
        let text = "[phrases]\r\na = \"A\"\r\n";
        let file = ModelFile::parse(text).expect("parses");
        let edit = file
            .add_entry(&["phrases"], "b", Value::from("B"))
            .expect("adds");
        assert_eq!(
            apply(text, vec![edit]),
            "[phrases]\r\na = \"A\"\r\nb = \"B\"\r\n"
        );
    }

    #[test]
    fn an_inline_or_dotted_table_is_not_added_to() {
        for text in ["phrases = { a = \"A\" }\n", "phrases.a = \"A\"\n"] {
            let file = ModelFile::parse(text).expect("parses");
            let error = file
                .add_entry(&["phrases"], "b", Value::from("B"))
                .expect_err("refused");
            assert!(error.contains("by hand"), "{error}");
        }
    }

    #[test]
    fn a_table_goes_after_the_last_table_of_its_section() {
        let text = "[types]\nmatch = \"every\"\n\n[types.page.api]\nterm = \"API\" # short\ndefinition = \"An interface.\"\n\n[builds.site]\n";
        let file = ModelFile::parse(text).expect("parses");
        let edit = file
            .add_table(
                &["types", "page", "sdk"],
                &[
                    ("term", Value::from("SDK")),
                    ("aliases", Value::Array(["SDKs"].into_iter().collect())),
                    ("definition", Value::from("A kit.")),
                ],
                &["types"],
            )
            .expect("adds");
        assert_eq!(
            apply(text, vec![edit]),
            "[types]\nmatch = \"every\"\n\n[types.page.api]\nterm = \"API\" # short\ndefinition = \"An interface.\"\n\n[types.page.sdk]\nterm = \"SDK\"\naliases = [\"SDKs\"]\ndefinition = \"A kit.\"\n\n[builds.site]\n"
        );
        let text = "spec = \"0.1\"\n";
        let file = ModelFile::parse(text).expect("parses");
        let edit = file
            .add_table(
                &["types", "page", "sdk"],
                &[("term", Value::from("SDK"))],
                &["types"],
            )
            .expect("adds");
        assert_eq!(
            apply(text, vec![edit]),
            "spec = \"0.1\"\n\n[types.page.sdk]\nterm = \"SDK\"\n"
        );
    }

    #[test]
    fn a_table_inside_an_inline_one_is_refused() {
        let text = "[types]\npage = { api = { term = \"API\", definition = \"x\" } }\n";
        let file = ModelFile::parse(text).expect("parses");
        let edit = file
            .add_table(
                &["types", "page", "sdk"],
                &[("term", Value::from("SDK"))],
                &["types"],
            )
            .expect("found a place");
        assert!(file.finish(vec![edit]).is_err());
    }

    #[test]
    fn a_value_is_replaced_in_place_in_any_form() {
        let text = "[features.sso]\nname = \"SSO\"\navailable = \"cloud beta\" # soon\n[features]\nlog = { name = \"Log\", available = 'cloud' }\n";
        let file = ModelFile::parse(text).expect("parses");
        let sso = file
            .replace_value(
                &path(&["features", "sso", "available"]),
                Value::from("cloud"),
            )
            .expect("found");
        assert_eq!(&text[sso.edit.span.range()], "\"cloud beta\"");
        let log = file
            .replace_value(
                &path(&["features", "log", "available"]),
                Value::from("cloud ga"),
            )
            .expect("found");
        assert_eq!(
            apply(text, vec![sso, log]),
            "[features.sso]\nname = \"SSO\"\navailable = \"cloud\" # soon\n[features]\nlog = { name = \"Log\", available = \"cloud ga\" }\n"
        );
    }

    #[test]
    fn array_items_and_keys_are_replaced_in_place() {
        let text = "[dimensions.pm]\nvalues = [\"npm\", # the default\n  \"yarn\"]\nlabels = { npm = \"npm\", yarn = \"Yarn\" }\n";
        let file = ModelFile::parse(text).expect("parses");
        let mut at = path(&["dimensions", "pm", "values"]);
        at.push(Seg::Index(1));
        let item = file.replace_value(&at, Value::from("pnpm")).expect("found");
        let key = file
            .rename_key(&path(&["dimensions", "pm", "labels", "yarn"]), "pnpm")
            .expect("found");
        assert_eq!(
            apply(text, vec![item, key]),
            "[dimensions.pm]\nvalues = [\"npm\", # the default\n  \"pnpm\"]\nlabels = { npm = \"npm\", pnpm = \"Yarn\" }\n"
        );
    }

    #[test]
    fn an_edit_that_misses_its_change_is_refused() {
        let file = ModelFile::parse(MODEL).expect("parses");
        let mut edit = file
            .add_entry(&["phrases"], "cloud", Value::from("Quill Cloud"))
            .expect("adds");
        edit.edit.new_text = "cloud = \"Quill\"\n".to_owned();
        assert!(file.finish(vec![edit]).is_err());
    }
}
