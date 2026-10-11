//! `ascribe_lsp::rename`, what `ascribe rename` calls: renaming by name
//! what the editor renames at a position. That its edits are the editor's
//! for a phrase and a dimension's value is held in `model_edits.rs`; this
//! file holds what only renaming by name does: a heading's id named by the
//! page and the id, one made from the heading's text getting an `@id`, a
//! page that moves, and why a rename is refused.

#![allow(clippy::expect_used, clippy::panic, clippy::unwrap_used)]

use std::collections::BTreeMap;

use ascribe_core::RelPath;
use ascribe_lsp::{RenameError, Renaming, rename};

use crate::support::{Fixture, MODEL};

const KEYS: &str = "---\ntitle: Keys\n---\n\n## Create a key\n@id: create-key\n\nText.\n\n## Set up\n\nFirst.\n\n## Set up\n\nSecond, see [](#set-up-1).\n";

const INSTALL: &str = "---\ntitle: Install\n---\n\nSee [](keys.md#create-key), [](keys.md#set-up), and [](keys.md#set-up-1).\n\nAlso [](guides/deep.md).\n";

const DEEP: &str = "---\ntitle: Deep\n---\n\nBack to [](../install.md).\n";

fn fixture() -> Fixture {
    Fixture::new(
        MODEL,
        &[
            ("docs/keys.md", KEYS),
            ("docs/install.md", INSTALL),
            ("docs/guides/deep.md", DEEP),
        ],
    )
}

fn path(text: &str) -> RelPath {
    RelPath::parse(text).unwrap()
}

/// Each file's text after `renaming`, by its path from the content root.
fn after(f: &Fixture, renaming: &Renaming) -> BTreeMap<String, String> {
    let content = f.path("docs");
    rename(&f.path("ascribe.toml"), renaming)
        .expect("renamed")
        .files
        .into_iter()
        .map(|file| {
            let rel = file.path.strip_prefix(&content).expect("in docs");
            (rel.to_string_lossy().replace('\\', "/"), file.text)
        })
        .collect()
}

fn id(file: &str, id: &str, new: &str) -> Renaming {
    Renaming::Id {
        file: path(file),
        id: id.to_owned(),
        new: new.to_owned(),
    }
}

#[test]
fn an_explicit_id_is_changed_with_the_links_that_name_it() {
    let f = fixture();
    let files = after(&f, &id("keys.md", "create-key", "new-key"));
    assert_eq!(
        files["keys.md"],
        KEYS.replace("@id: create-key", "@id: new-key")
    );
    assert_eq!(
        files["install.md"],
        INSTALL.replace("keys.md#create-key", "keys.md#new-key")
    );
    assert_eq!(files.len(), 2);
}

#[test]
fn an_id_made_from_the_text_gets_an_id_and_the_numbering_after_it_follows() {
    let f = fixture();
    let files = after(&f, &id("keys.md", "set-up", "setup"));
    // The first `Set up` is `setup` now, so the second is no longer
    // numbered: `set-up-1` is `set-up`.
    assert_eq!(
        files["keys.md"],
        KEYS.replacen("## Set up\n", "## Set up\n@id: setup\n", 1)
            .replace("(#set-up-1)", "(#set-up)")
    );
    assert_eq!(
        files["install.md"],
        INSTALL
            .replace("keys.md#set-up)", "keys.md#setup)")
            .replace("keys.md#set-up-1)", "keys.md#set-up)")
    );
}

#[test]
fn a_page_that_moves_takes_the_links_to_it_and_from_it_along() {
    let f = fixture();
    let renamed = rename(
        &f.path("ascribe.toml"),
        &Renaming::File {
            from: path("guides/deep.md"),
            to: path("deep.md"),
        },
    )
    .expect("renamed");
    assert_eq!(
        renamed.moved,
        Some((f.path("docs/guides/deep.md"), f.path("docs/deep.md")))
    );
    let files: BTreeMap<_, _> = renamed
        .files
        .into_iter()
        .map(|file| (file.path, file.text))
        .collect();
    assert_eq!(
        files[&f.path("docs/install.md")],
        INSTALL.replace("(guides/deep.md)", "(deep.md)")
    );
    // Its own links are changed where it is, before it moves.
    assert_eq!(
        files[&f.path("docs/guides/deep.md")],
        DEEP.replace("(../install.md)", "(install.md)")
    );
}

#[test]
fn what_isnt_there_is_missing_and_what_cant_be_renamed_is_refused() {
    let f = fixture();
    let config = f.path("ascribe.toml");
    let missing = |renaming: Renaming| {
        assert!(
            matches!(rename(&config, &renaming), Err(RenameError::Missing(_))),
            "{renaming:?}"
        );
    };
    missing(Renaming::Phrase {
        key: "product".to_owned(),
        new: "name".to_owned(),
    });
    missing(Renaming::Value {
        value: "cloud".to_owned(),
        new: "hosted".to_owned(),
    });
    missing(id("keys.md", "nothing", "new"));
    missing(id("nothing.md", "create-key", "new"));
    missing(Renaming::File {
        from: path("nothing.md"),
        to: path("new.md"),
    });

    let refused = |renaming: Renaming| {
        assert!(
            matches!(rename(&config, &renaming), Err(RenameError::Refused(_))),
            "{renaming:?}"
        );
    };
    refused(id("keys.md", "create-key", "create-key"));
    refused(id("keys.md", "create-key", "set-up"));
    refused(id("keys.md", "create-key", "two words"));
    refused(Renaming::File {
        from: path("keys.md"),
        to: path("install.md"),
    });
    refused(Renaming::File {
        from: path("keys.md"),
        to: path("keys.txt"),
    });
}

#[test]
fn a_permalink_is_refused_since_links_from_outside_use_it() {
    let keys = "---\ntitle: Keys\n---\n\n## Rotate\n@id {permalink=true}: rotate-keys\n";
    let f = Fixture::new(MODEL, &[("docs/keys.md", keys)]);
    let error = rename(
        &f.path("ascribe.toml"),
        &id("keys.md", "rotate-keys", "rotate"),
    )
    .expect_err("refused");
    assert!(
        matches!(&error, RenameError::Refused(m) if m.contains("permalink")),
        "{error}"
    );
}
