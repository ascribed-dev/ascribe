//! Moving `ascribe.toml`'s `[permalinks]` into `ascribe.addresses`, where
//! what happened to a permalink is recorded now (SPEC §7.6): each retired
//! permalink as `go:<name>` in `[retired]`, or in `[moved]` when it had a
//! `to`, and each alias as `go:<name>` moved to `go:<target>`. The table is
//! removed from `ascribe.toml`, keeping the rest of the file as it is.
//! `ascribe addresses init` writes through here, and a base read from git
//! that still has the table is read as if it had been moved.

use crate::addresses::{AddressFile, Entry, permalink_key, split_place};
use toml_edit::{DocumentMut, Item, TableLike};

/// An entry `ascribe.toml`'s `[permalinks]` held, as `ascribe.addresses`
/// records it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MovedOut {
    /// The table it goes in: `moved` or `retired`.
    pub table: &'static str,
    /// Its key, `go:<name>`.
    pub key: String,
    /// Its value: where it leads, or why it was retired.
    pub value: String,
}

/// `model_text` without `[permalinks]`, and the entries it held, in the
/// order written; `None` when it has no `[permalinks]`.
///
/// # Errors
///
/// Why they can't be moved: the file isn't TOML, or an entry isn't one a
/// permalink's table can have.
pub fn permalinks_out(model_text: &str) -> Result<Option<(String, Vec<MovedOut>)>, String> {
    let mut doc: DocumentMut = model_text
        .parse()
        .map_err(|e: toml_edit::TomlError| format!("ascribe.toml isn't TOML: {}", e.message()))?;
    let Some(item) = doc.get("permalinks") else {
        return Ok(None);
    };
    let table = item
        .as_table_like()
        .ok_or("`permalinks` in ascribe.toml isn't a table")?;
    let mut out = Vec::new();
    if let Some(retired) = sub_table(table, "retired")? {
        for (name, value) in retired.iter() {
            let entry = value
                .as_table_like()
                .ok_or_else(|| format!("`permalinks.retired.{name}` isn't a table"))?;
            let text = |key: &str| entry.get(key).and_then(Item::as_str).map(str::trim);
            let entry = match (text("to"), text("reason")) {
                (Some(to), _) => {
                    let to = to.trim_start_matches('/');
                    if split_place(to).is_none() {
                        return Err(format!(
                            "`permalinks.retired.{name}` leads to `{to}`, which isn't a page's path, with `#` and a heading's id"
                        ));
                    }
                    MovedOut {
                        table: "moved",
                        key: key(name)?,
                        value: to.to_owned(),
                    }
                }
                (None, Some(reason)) if !reason.is_empty() => MovedOut {
                    table: "retired",
                    key: key(name)?,
                    value: reason.to_owned(),
                },
                (None, _) => {
                    return Err(format!("`permalinks.retired.{name}` has no `reason`"));
                }
            };
            out.push(entry);
        }
    }
    if let Some(aliases) = sub_table(table, "aliases")? {
        for (name, value) in aliases.iter() {
            let target = value
                .as_str()
                .ok_or_else(|| format!("`permalinks.aliases.{name}` isn't a permalink's name"))?;
            out.push(MovedOut {
                table: "moved",
                key: key(name)?,
                value: key(target)?,
            });
        }
    }
    doc.remove("permalinks");
    Ok(Some((doc.to_string(), out)))
}

impl AddressFile {
    /// Records each of `entries`, in place of any entry for its key.
    pub fn take_in(&mut self, entries: &[MovedOut]) {
        for entry in entries {
            self.remove(&entry.key);
            let table = if entry.table == "moved" {
                &mut self.moved
            } else {
                &mut self.retired
            };
            table.insert(entry.key.clone(), Entry::new(entry.value.clone()));
        }
    }
}

/// The table `key` of `[permalinks]`, when it has one.
fn sub_table<'a>(table: &'a dyn TableLike, key: &str) -> Result<Option<&'a dyn TableLike>, String> {
    match table.get(key) {
        None => Ok(None),
        Some(item) => item
            .as_table_like()
            .map(Some)
            .ok_or_else(|| format!("`permalinks.{key}` in ascribe.toml isn't a table")),
    }
}

/// `go:<name>`, when `name` is a permalink's name.
fn key(name: &str) -> Result<String, String> {
    if ascribe_core::is_id(name) {
        Ok(permalink_key(name))
    } else {
        Err(format!("`{name}` isn't a permalink's name"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_entry_moves_and_the_rest_of_the_file_stays() {
        let text = "# The model.\nspec = \"0.1\"\n\n\
                    [permalinks.retired]\n\
                    rotate-keys = { to = \"/guides/security.md#keys\", reason = \"Merged\" }\n\
                    legacy-sync = { reason = \"Removed in 4.0\" }\n\n\
                    [permalinks.aliases]\n\
                    agent-settings = \"settings\"\n\n\
                    [checks]\n# Kept.\n";
        let (after, moved) = permalinks_out(text).expect("moves").expect("has some");
        assert_eq!(after, "# The model.\nspec = \"0.1\"\n\n[checks]\n# Kept.\n");
        let entries: Vec<(&str, &str, &str)> = moved
            .iter()
            .map(|m| (m.table, m.key.as_str(), m.value.as_str()))
            .collect();
        assert_eq!(
            entries,
            [
                ("moved", "go:rotate-keys", "guides/security.md#keys"),
                ("retired", "go:legacy-sync", "Removed in 4.0"),
                ("moved", "go:agent-settings", "go:settings"),
            ]
        );
    }

    #[test]
    fn a_model_without_the_table_has_nothing_to_move() {
        assert_eq!(permalinks_out("spec = \"0.1\"\n"), Ok(None));
    }

    #[test]
    fn what_cant_be_moved_is_said() {
        assert_eq!(
            permalinks_out("[permalinks.retired]\na = { to = \"a.txt\" }\n"),
            Err(
                "`permalinks.retired.a` leads to `a.txt`, which isn't a page's path, with `#` and a heading's id"
                    .to_owned()
            )
        );
        assert_eq!(
            permalinks_out("[permalinks.retired]\na = {}\n"),
            Err("`permalinks.retired.a` has no `reason`".to_owned())
        );
        assert_eq!(
            permalinks_out("[permalinks.aliases]\na = \"b c\"\n"),
            Err("`b c` isn't a permalink's name".to_owned())
        );
    }
}
