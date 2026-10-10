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
}

/// A text edit to the file, and the change it makes.
pub(crate) type Edit = (TextEdit, Change);

/// `ascribe.toml`'s text, parsed with the spans of everything in it.
pub(crate) struct ModelFile<'a> {
    text: &'a str,
    doc: Document<&'a str>,
    nl: &'static str,
}

impl<'a> ModelFile<'a> {
    /// Parses the text.
    pub(crate) fn parse(text: &'a str) -> Result<ModelFile<'a>, String> {
        let doc = Document::parse(text)
            .map_err(|e| format!("`ascribe.toml` isn't valid TOML: {}", e.message()))?;
        Ok(ModelFile {
            text,
            doc,
            nl: if text.contains("\r\n") { "\r\n" } else { "\n" },
        })
    }

    /// The item at a path.
    pub(crate) fn get(&self, path: &[Seg]) -> Option<&Item> {
        let mut item = self.doc.as_item();
        for seg in path {
            item = match seg {
                Seg::Key(key) => item.get(key.as_str())?,
                Seg::Index(i) => item.get(*i)?,
            };
        }
        Some(item)
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
                Ok((self.append(&text), change))
            }
            Some(Item::Table(t)) if !t.is_dotted() && !t.is_implicit() => {
                let (at, last) = self.after_entries(t);
                let indent = last.map_or("", |(k, _)| self.indent(k));
                let sep = self.separator(last);
                let line = format!("{indent}{key_text}{sep}{value}");
                Ok((self.insert_line(at, &line), change))
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
        Ok((edit, change))
    }

    /// Replaces the value at `path`, keeping what's around it.
    pub(crate) fn replace_value(&self, at: &[Seg], value: Value) -> Result<Edit, String> {
        let span = self
            .get(at)
            .and_then(Item::span)
            .ok_or_else(|| "The value can't be found in `ascribe.toml`.".to_owned())?;
        let edit = TextEdit::replace(Span::new(span.start, span.end), value.to_string());
        Ok((edit, Change::Set(at.to_vec(), value)))
    }

    /// Renames the last key of `at`.
    pub(crate) fn rename_key(&self, at: &[Seg], new: &str) -> Result<Edit, String> {
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
        Ok((edit, Change::RenameKey(at.to_vec(), new.to_owned())))
    }

    /// The edits, checked: applied to the text, they give a file with the
    /// same values as the changes made through `toml_edit`'s API. Returns
    /// the edits, sorted, and the text after them.
    pub(crate) fn finish(&self, edits: Vec<Edit>) -> Result<(Vec<TextEdit>, String), String> {
        let refused =
            |why: &str| format!("The change can't be made in place in `ascribe.toml`: {why}.");
        let mut expected: DocumentMut = self
            .text
            .parse()
            .map_err(|_| refused("the file isn't valid TOML"))?;
        let mut text_edits = Vec::with_capacity(edits.len());
        for (edit, change) in edits {
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

    fn apply(text: &str, edits: Vec<Edit>) -> String {
        let file = ModelFile::parse(text).expect("parses");
        let (edits, after) = file.finish(edits).expect("checks");
        assert_eq!(apply_edits(text, &edits).expect("applies"), after);
        after
    }

    const MODEL: &str = "# The content model.\nspec = \"0.1\"\n\n[phrases]\n# The product.\nproduct = \"Quill\"   # its name\n\n# Builds.\n[builds.site]\nvariants = \"switch\"\n";

    #[test]
    fn an_entry_goes_after_its_tables_last_one_keeping_everything_else() {
        let file = ModelFile::parse(MODEL).expect("parses");
        let edit = file
            .add_entry(&["phrases"], "cloud", Value::from("Quill Cloud"))
            .expect("adds");
        assert!(edit.0.span.is_empty(), "an insertion, not a rewrite");
        assert_eq!(edit.0.new_text, "cloud = \"Quill Cloud\"\n");
        let after = apply(MODEL, vec![edit]);
        assert_eq!(
            after,
            "# The content model.\nspec = \"0.1\"\n\n[phrases]\n# The product.\nproduct = \"Quill\"   # its name\ncloud = \"Quill Cloud\"\n\n# Builds.\n[builds.site]\nvariants = \"switch\"\n"
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
        let text = "[glossary]\nmatch = \"every\"\n\n[glossary.terms.api]\nterm = \"API\" # short\ndefinition = \"An interface.\"\n\n[builds.site]\n";
        let file = ModelFile::parse(text).expect("parses");
        let edit = file
            .add_table(
                &["glossary", "terms", "sdk"],
                &[
                    ("term", Value::from("SDK")),
                    ("aliases", Value::Array(["SDKs"].into_iter().collect())),
                    ("definition", Value::from("A kit.")),
                ],
                &["glossary"],
            )
            .expect("adds");
        assert_eq!(
            apply(text, vec![edit]),
            "[glossary]\nmatch = \"every\"\n\n[glossary.terms.api]\nterm = \"API\" # short\ndefinition = \"An interface.\"\n\n[glossary.terms.sdk]\nterm = \"SDK\"\naliases = [\"SDKs\"]\ndefinition = \"A kit.\"\n\n[builds.site]\n"
        );
        let text = "spec = \"0.1\"\n";
        let file = ModelFile::parse(text).expect("parses");
        let edit = file
            .add_table(
                &["glossary", "terms", "sdk"],
                &[("term", Value::from("SDK"))],
                &["glossary"],
            )
            .expect("adds");
        assert_eq!(
            apply(text, vec![edit]),
            "spec = \"0.1\"\n\n[glossary.terms.sdk]\nterm = \"SDK\"\n"
        );
    }

    #[test]
    fn a_table_inside_an_inline_one_is_refused() {
        let text = "[glossary]\nterms = { api = { term = \"API\", definition = \"x\" } }\n";
        let file = ModelFile::parse(text).expect("parses");
        let edit = file
            .add_table(
                &["glossary", "terms", "sdk"],
                &[("term", Value::from("SDK"))],
                &["glossary"],
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
        assert_eq!(&text[sso.0.span.range()], "\"cloud beta\"");
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
        let (mut edit, change) = file
            .add_entry(&["phrases"], "cloud", Value::from("Quill Cloud"))
            .expect("adds");
        edit.new_text = "cloud = \"Quill\"\n".to_owned();
        assert!(file.finish(vec![(edit, change)]).is_err());
    }
}
