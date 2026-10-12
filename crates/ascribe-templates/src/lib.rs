//! The page templates Ascribe ships: The Good Docs Project's, converted into
//! Ascribe pages by `scripts/templates/convert.ts` from the release it pins,
//! and committed in `set/`. Nothing here changes them: [`FILES`] is each file
//! as it was committed.
//!
//! `set/set.toml` lists the templates: each one's name, their index's
//! summary of it, and its template and guide, with their tag and the license;
//! [`set`] reads it. [`NOTES`] is Ascribe's notes on each type, written by hand
//! in `notes/`, which the conversion never opens. [`eject()`] writes templates
//! into a project, as `ascribe templates eject` does, and [`init()`] starts a
//! project, as `ascribe init` does.

pub mod eject;
pub mod init;

pub use eject::{
    Added, Eject, EjectAnswer, EjectError, LOCK_FILE, Refusal, RefusalReason, TEMPLATES_DIR, eject,
};
pub use init::{Init, InitAnswer, InitError, MODEL_TEXT, init};

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
        name: "reference.guide.md",
        text: include_str!("../set/reference.guide.md"),
    },
    File {
        name: "reference.md",
        text: include_str!("../set/reference.md"),
    },
    File {
        name: "release-notes.guide.md",
        text: include_str!("../set/release-notes.guide.md"),
    },
    File {
        name: "release-notes.md",
        text: include_str!("../set/release-notes.md"),
    },
    File {
        name: "troubleshooting.guide.md",
        text: include_str!("../set/troubleshooting.guide.md"),
    },
    File {
        name: "troubleshooting.md",
        text: include_str!("../set/troubleshooting.md"),
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

/// Ascribe's notes on each type, by name, as committed: what's particular to
/// writing a page of that type in Ascribe, and nothing their guide says. Each
/// is written by hand, as `notes/<name>.ascribe.md`, and keeps that name in a
/// project.
pub const NOTES: &[File] = &[
    File {
        name: "concept.ascribe.md",
        text: include_str!("../notes/concept.ascribe.md"),
    },
    File {
        name: "how-to.ascribe.md",
        text: include_str!("../notes/how-to.ascribe.md"),
    },
    File {
        name: "reference.ascribe.md",
        text: include_str!("../notes/reference.ascribe.md"),
    },
    File {
        name: "release-notes.ascribe.md",
        text: include_str!("../notes/release-notes.ascribe.md"),
    },
    File {
        name: "troubleshooting.ascribe.md",
        text: include_str!("../notes/troubleshooting.ascribe.md"),
    },
    File {
        name: "tutorial.ascribe.md",
        text: include_str!("../notes/tutorial.ascribe.md"),
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

    /// Ascribe's notes on its type, from [`NOTES`], when there are any.
    pub fn notes(&self) -> Option<&'static File> {
        let name = format!("{}.ascribe.md", self.name);
        NOTES.iter().find(|f| f.name == name)
    }
}

/// The set, read from `set.toml`.
pub fn set() -> Set {
    // `set.toml` is compiled in, written by the conversion, and a test reads
    // it with this function, so it always parses.
    #[allow(clippy::expect_used)]
    toml::from_str(FILES[0].text).expect("set.toml parses")
}
