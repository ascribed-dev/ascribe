//! Retiring permalinks in `ascribe.addresses` (SPEC §7.6, §7.10): an entry
//! for each under its `permalink:` key, in `[retired]` with its reason, or,
//! with somewhere to lead, in `[moved]`. `ascribe permalinks retire` and the
//! editor's quick fix on `permalink-lost` both write through here.
//!
//! And moving a project's records out of `ascribe.toml`: `ascribe addresses
//! init` records what `[permalinks.retired]` and `[permalinks.aliases]` hold
//! in `ascribe.addresses`, and removes the tables ([`move_records`]).

use ascribe_model::AddressFile;
use ascribe_model::addresses::permalink_key;
use toml_edit::{DocumentMut, Item};

/// A permalink to retire.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Retirement<'a> {
    /// Its name.
    pub name: &'a str,
    /// Why, as the author writes it.
    pub reason: &'a str,
    /// Where its address leads now: a page, and `#` and a heading's id.
    /// With it, the permalink is moved there, and `reason` isn't recorded.
    pub to: Option<&'a str>,
}

/// `file` with an entry for each of `retiring`: `[retired]`'s, with its
/// reason, or, with `to`, `[moved]`'s, leading there. Each alias that led to
/// one leads where it does now, or is retired with it.
///
/// # Errors
///
/// The file records one of the names already.
pub fn retire(file: &AddressFile, retiring: &[Retirement<'_>]) -> Result<AddressFile, String> {
    let mut after = file.clone();
    for entry in retiring {
        let key = permalink_key(entry.name);
        if after.entry(&key).is_some() {
            return Err(format!(
                "`{}` records `{}` already.",
                ascribe_model::ADDRESSES_FILE,
                entry.name
            ));
        }
        match entry.to {
            Some(to) => after.record_move(&key, to),
            None => after.record_retired(&key, entry.reason),
        }
    }
    Ok(after)
}

/// What [`move_records`] made.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MovedRecords {
    /// `ascribe.toml`'s text without `[permalinks.retired]` and
    /// `[permalinks.aliases]`.
    pub model_text: String,
    /// `ascribe.addresses` with each of their records.
    pub file: AddressFile,
    /// The names recorded, in order. A name the file records already keeps
    /// its entry, and isn't here.
    pub names: Vec<String>,
}

/// `model_text`, `ascribe.toml`, without its `[permalinks.retired]` and
/// `[permalinks.aliases]`, and `file` with what `records`, the records read
/// from them, held: each under its `permalink:` key. `None` when
/// `ascribe.toml` has neither table. The rest of `ascribe.toml` is kept as
/// written; the comments over each table go with it.
///
/// # Errors
///
/// `ascribe.toml` isn't valid TOML, or its records aren't written in it but
/// taken from another file with `[from]`, whose project they belong to.
pub fn move_records(
    model_text: &str,
    file: &AddressFile,
    records: &ascribe_model::PermalinkRecords,
) -> Result<Option<MovedRecords>, String> {
    let mut doc: DocumentMut = model_text.parse().map_err(|e: toml_edit::TomlError| {
        format!("`ascribe.toml` isn't valid TOML: {}", e.message())
    })?;
    let Some(permalinks) = doc.get_mut("permalinks").and_then(Item::as_table_like_mut) else {
        return if records.is_empty() {
            Ok(None)
        } else {
            Err(taken())
        };
    };
    let removed = [permalinks.remove("retired"), permalinks.remove("aliases")];
    if removed.iter().all(Option::is_none) {
        return if records.is_empty() {
            Ok(None)
        } else {
            Err(taken())
        };
    }
    if permalinks.is_empty() {
        doc.remove("permalinks");
    }
    let mut after = file.clone();
    let names = after.record_permalinks(records);
    Ok(Some(MovedRecords {
        model_text: doc.to_string(),
        file: after,
        names,
    }))
}

/// Why records taken from another file aren't moved.
fn taken() -> String {
    "`[permalinks.retired]` or `[permalinks.aliases]` is taken from another file with `[from]`: move them in the project that writes them, then take nothing from `[permalinks]`.".to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;
    use ascribe_core::FileId;

    fn records(model: &str) -> ascribe_model::PermalinkRecords {
        ascribe_model::load_str(model, FileId::new(0))
            .expect("loads")
            .permalinks
    }

    #[test]
    fn retired_and_moved() {
        let mut file = AddressFile::default();
        file.record_move("permalink:keys", "permalink:rotate-keys");
        let after = retire(
            &file,
            &[
                Retirement {
                    name: "rotate-keys",
                    reason: "Merged into the \"Security\" guide",
                    to: Some("guides/security.md#keys"),
                },
                Retirement {
                    name: "legacy-sync",
                    reason: "Removed in 4.0",
                    to: None,
                },
            ],
        )
        .expect("retires");
        assert!(
            after.to_toml().ends_with(
                "[moved]\n\
                 \"permalink:keys\" = \"guides/security.md#keys\"\n\
                 \"permalink:rotate-keys\" = \"guides/security.md#keys\"\n\
                 \n[retired]\n\
                 \"permalink:legacy-sync\" = \"Removed in 4.0\"\n"
            ),
            "{}",
            after.to_toml()
        );
        let error = retire(
            &after,
            &[Retirement {
                name: "legacy-sync",
                reason: "again",
                to: None,
            }],
        )
        .expect_err("refused");
        assert!(error.contains("already"), "{error}");
    }

    #[test]
    fn records_move_out_of_ascribe_toml() {
        let model = "# The model.\nspec = \"0.1\"\n\n# Old names.\n[permalinks.aliases]\nkeys = \"rotate\" # old\n\n\
                     [permalinks.retired]\nold = { to = \"a.md#b\", reason = \"Moved\" }\ngone = { reason = \"Gone\" }\n\n\
                     [consumer]\nbase-path = \"/docs/\"\n";
        let moved = move_records(model, &AddressFile::default(), &records(model))
            .expect("moves")
            .expect("something to move");
        assert_eq!(
            moved.model_text,
            "# The model.\nspec = \"0.1\"\n\n[consumer]\nbase-path = \"/docs/\"\n"
        );
        assert_eq!(moved.names, ["old", "gone", "keys"]);
        assert!(moved.file.to_toml().ends_with(
            "[moved]\n\
             \"permalink:keys\" = \"permalink:rotate\"\n\
             \"permalink:old\" = \"a.md#b\"\n\
             \n[retired]\n\
             \"permalink:gone\" = \"Gone\"\n"
        ));
        let none = "spec = \"0.1\"\n";
        assert_eq!(
            move_records(none, &AddressFile::default(), &records(none)).expect("reads"),
            None
        );
    }

    #[test]
    fn a_table_taken_from_another_file_is_refused() {
        let taken = ascribe_model::PermalinkRecords {
            retired: vec![ascribe_model::RetiredPermalink {
                name: "old".to_owned(),
                reason: Some("Gone".to_owned()),
                to: None,
                span: ascribe_core::Span::empty(0),
                to_span: None,
                file: ascribe_model::RecordedIn::Model,
            }],
            aliases: Vec::new(),
        };
        let text =
            "spec = \"0.1\"\n[from.\"../docs/ascribe.toml\"]\ntake = [\"permalinks.retired\"]\n";
        let error =
            move_records(text, &AddressFile::default(), &taken).expect_err("taken isn't moved");
        assert!(error.contains("[from]"), "{error}");
    }
}
