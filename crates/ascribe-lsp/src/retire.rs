//! Retiring permalinks in `ascribe.toml` (SPEC §7.6): an entry in
//! `[permalinks.retired]` for each, written in place, keeping the file's
//! comments and order, as the content model's other edits are
//! ([`ModelFile`]). `ascribe permalinks retire` and the editor's quick fix on
//! `permalink-removed` both write through here.

use ascribe_core::TextEdit;
use toml_edit::{InlineTable, Value};

use crate::model_file::{ModelTexts, TakenFile, path};

/// A permalink to retire.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Retirement<'a> {
    /// Its name.
    pub name: &'a str,
    /// Why, as the author writes it.
    pub reason: &'a str,
    /// Where its address leads now: a page, and `#` and a heading's id.
    pub to: Option<&'a str>,
}

/// What retiring wrote.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Retired {
    /// `ascribe.toml`'s text after it.
    pub text: String,
    /// The edits to the text before it, when one permalink was retired in
    /// it: one insertion. Empty when several were, or none was.
    pub edits: Vec<TextEdit>,
    /// The files `[from]` takes `[permalinks.retired]` from that it changed,
    /// with their texts after it (SPEC §7.9).
    pub taken: Vec<TakenFile>,
}

/// `model_text` with an entry in `[permalinks.retired]` for each of
/// `retiring`, after the table's last entry, or in a new table at the end
/// of the file: of `ascribe.toml`, or of the file of `taken` that `[from]`
/// takes the table from.
///
/// # Errors
///
/// Why the file can't be changed in place: it isn't valid TOML, already
/// retires one of the names, or writes the table in a form this can't add
/// to, such as inline.
pub fn retire(
    model_text: &str,
    taken: &[TakenFile],
    retiring: &[Retirement<'_>],
) -> Result<Retired, String> {
    let table = ["permalinks", "retired"];
    let mut texts = ModelTexts::new(model_text, taken.to_vec());
    let mut edits = Vec::new();
    for entry in retiring {
        let file = texts.parse()?;
        let mut at = path(&table);
        at.push(crate::model_file::Seg::Key(entry.name.to_owned()));
        if file.get(&at).is_some() {
            return Err(format!("`ascribe.toml` retires `{}` already.", entry.name));
        }
        let mut value = InlineTable::new();
        if let Some(to) = entry.to {
            value.insert("to", Value::from(to));
        }
        value.insert("reason", Value::from(entry.reason));
        value.fmt();
        let edit = file.add_entry(&table, entry.name, Value::InlineTable(value))?;
        let finished = file.finish(vec![edit])?;
        edits.clone_from(&finished.edits);
        texts.apply(&finished);
    }
    if retiring.len() != 1 {
        edits.clear();
    }
    Ok(Retired {
        taken: texts.changed(),
        text: texts.text,
        edits,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn retired(text: &str, retiring: &[Retirement<'_>]) -> String {
        retire(text, &[], retiring).expect("retires").text
    }

    #[test]
    fn a_table_is_made_at_the_end_and_added_to_after() {
        let text =
            "# The model.\nspec = \"0.1\"\n\n[permalinks.aliases]\nkeys = \"rotate\" # old\n";
        let one = retire(
            text,
            &[],
            &[Retirement {
                name: "rotate-keys",
                reason: "Merged into the \"Security\" guide",
                to: Some("guides/security.md#keys"),
            }],
        )
        .expect("retires");
        assert_eq!(one.edits.len(), 1);
        assert_eq!(
            one.text,
            "# The model.\nspec = \"0.1\"\n\n[permalinks.aliases]\nkeys = \"rotate\" # old\n\n\
             [permalinks.retired]\n\
             rotate-keys = { to = \"guides/security.md#keys\", reason = 'Merged into the \"Security\" guide' }\n"
        );
        let two = retired(
            &one.text,
            &[
                Retirement {
                    name: "legacy-sync",
                    reason: "Removed in 4.0",
                    to: None,
                },
                Retirement {
                    name: "v1.2",
                    reason: "Old",
                    to: None,
                },
            ],
        );
        assert!(
            two.ends_with(
                "rotate-keys = { to = \"guides/security.md#keys\", reason = 'Merged into the \"Security\" guide' }\n\
                 legacy-sync = { reason = \"Removed in 4.0\" }\n\
                 \"v1.2\" = { reason = \"Old\" }\n"
            ),
            "{two}"
        );
    }

    #[test]
    fn a_name_retired_already_is_refused() {
        let text = "[permalinks.retired]\na = { reason = \"r\" }\n";
        let error = retire(
            text,
            &[],
            &[Retirement {
                name: "a",
                reason: "again",
                to: None,
            }],
        )
        .expect_err("refused");
        assert!(error.contains("already"), "{error}");
    }

    #[test]
    fn a_table_from_takes_is_added_to_where_it_s_written() {
        let text =
            "spec = \"0.1\"\n[from.\"../docs/ascribe.toml\"]\ntake = [\"permalinks.retired\"]\n";
        let docs = TakenFile {
            file: "../docs/ascribe.toml".to_owned(),
            path: std::path::PathBuf::from("/repo/docs/ascribe.toml"),
            text: "spec = \"0.1\"\n\n[permalinks.retired]\nold = { reason = \"Gone\" }\n"
                .to_owned(),
        };
        let retired = retire(
            text,
            std::slice::from_ref(&docs),
            &[Retirement {
                name: "older",
                reason: "Gone too",
                to: None,
            }],
        )
        .expect("retires");
        assert_eq!(retired.text, text);
        assert!(retired.edits.is_empty());
        assert_eq!(retired.taken.len(), 1);
        assert_eq!(
            retired.taken[0].text,
            "spec = \"0.1\"\n\n[permalinks.retired]\nold = { reason = \"Gone\" }\nolder = { reason = \"Gone too\" }\n"
        );
        // A file it takes from that couldn't be read isn't written in.
        let refused = retire(
            text,
            &[],
            &[Retirement {
                name: "older",
                reason: "Gone too",
                to: None,
            }],
        )
        .expect_err("refused");
        assert!(refused.contains("../docs/ascribe.toml"), "{refused}");
    }
}
