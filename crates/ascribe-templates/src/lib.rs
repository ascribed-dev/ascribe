//! The page templates Ascribe ships: The Good Docs Project's, converted into
//! Ascribe pages by `scripts/templates/convert.ts` from the release it pins,
//! and committed in `set/`. Nothing here changes them: [`FILES`] is each file
//! as it was committed.
//!
//! `set/set.toml` lists the templates: each one's name, their index's
//! summary of it, and its template and guide, with their tag and the license.

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
        name: "how-to.guide.md",
        text: include_str!("../set/how-to.guide.md"),
    },
    File {
        name: "how-to.md",
        text: include_str!("../set/how-to.md"),
    },
];
