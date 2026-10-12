//! The page templates Ascribe ships: The Good Docs Project's, converted into
//! Ascribe pages by `scripts/templates/convert.ts` from the release it pins,
//! and committed in `set/`. Nothing here changes them: [`FILES`] is each file
//! as it was committed.
//!
//! `set/set.toml` lists the templates: each one's name, their index's
//! summary of it, and its template and guide, with their tag and the license;
//! [`set`] reads it. [`eject()`] writes templates into a project, as
//! `ascribe templates eject` does.

pub mod eject;

pub use eject::{
    Added, Eject, EjectAnswer, EjectError, LOCK_FILE, Refusal, RefusalReason, TEMPLATES_DIR, eject,
};

/// A file of the set.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct File {
    /// Its name in `set/`.
    pub name: &'static str,
    /// Its text, as committed.
    pub text: &'static str,
}

/// Every file of the set, `set.toml` first, then the rest by name.
pub const FILES: &[File] = &[
    File {
        name: "set.toml",
        text: include_str!("../set/set.toml"),
    },
    File {
        name: "concept.guide.md",
        text: include_str!("../set/concept.guide.md"),
    },
    File {
        name: "concept.md",
        text: include_str!("../set/concept.md"),
    },
    File {
        name: "how-to.guide.md",
        text: include_str!("../set/how-to.guide.md"),
    },
    File {
        name: "how-to.md",
        text: include_str!("../set/how-to.md"),
    },
    File {
        name: "tutorial.guide.md",
        text: include_str!("../set/tutorial.guide.md"),
    },
    File {
        name: "tutorial.md",
        text: include_str!("../set/tutorial.md"),
    },
];

/// The set, as `set.toml` lists it.
#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
pub struct Set {
    /// Their repository.
    pub repository: String,
    /// The tag of theirs the copies were converted from.
    pub tag: String,
    /// The tag's commit.
    pub commit: String,
    /// Their license.
    pub license: String,
    /// The templates, in the order `set.toml` lists them.
    #[serde(rename = "template")]
    pub templates: Vec<Template>,
}

/// A template of the set.
#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
pub struct Template {
    /// Its name, which is its type's.
    pub name: String,
    /// Their index's summary of it.
    pub summary: String,
    /// Its template's file, in [`FILES`].
    pub template: String,
    /// Its guide's file, in [`FILES`].
    pub guide: String,
}

impl Template {
    /// The text of one of its files, by name.
    pub fn text(&self, name: &str) -> &'static str {
        FILES.iter().find(|f| f.name == name).map_or("", |f| f.text)
    }
}

/// The set, read from `set.toml`.
pub fn set() -> Set {
    // `set.toml` is compiled in, written by the conversion, and a test reads
    // it with this function, so it always parses.
    #[allow(clippy::expect_used)]
    toml::from_str(FILES[0].text).expect("set.toml parses")
}
