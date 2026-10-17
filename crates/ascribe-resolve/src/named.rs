//! What `ascribe refs` and a memo file's `about` name (SPEC §7.7): a page or
//! a fragment by its content path, a heading as `page.md#id`, or a content
//! model entry as `<prefix>:<name>`, such as `phrase:product`. This is the
//! one reading of that grammar.

use ascribe_core::RelPath;

use crate::project::Project;
use crate::uses::Usable;

/// The kinds of content model entry a target can name.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum EntryKind {
    /// A phrase, `phrase:<key>`.
    Phrase,
    /// A feature, `feature:<key>`.
    Feature,
    /// A glossary term, `term:<id>`.
    Term,
    /// A dimension, `dimension:<name>`.
    Dimension,
    /// A note type, `note:<type>`.
    Note,
    /// A project widget, `widget:<name>`.
    Widget,
}

/// The prefixes that name a content model entry, and the kind each names.
pub const PREFIXES: &[(&str, EntryKind)] = &[
    ("phrase", EntryKind::Phrase),
    ("feature", EntryKind::Feature),
    ("term", EntryKind::Term),
    ("dimension", EntryKind::Dimension),
    ("note", EntryKind::Note),
    ("widget", EntryKind::Widget),
];

/// What a target names, as written: it may name nothing in the project.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Named {
    /// A source file, by content path, or a heading on it.
    Path {
        /// The file.
        path: RelPath,
        /// The heading's id, after `#`.
        id: Option<String>,
    },
    /// A content model entry.
    Entry {
        /// Which kind.
        kind: EntryKind,
        /// Its key, id, or name.
        name: String,
    },
}

/// Why a target can't be read.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BadTarget {
    /// The target, as given.
    pub given: String,
    /// What's wrong with it.
    pub reason: String,
}

impl Named {
    /// A content model entry, when `given` starts with one of [`PREFIXES`]
    /// and a colon, or else a content path with an optional `#id`.
    ///
    /// # Errors
    ///
    /// Nothing follows the colon, or `given` isn't a content path.
    pub fn parse(given: &str) -> Result<Named, BadTarget> {
        Named::entry(given).unwrap_or_else(|| Named::path(given))
    }

    /// A content model entry, when `given` starts with one of [`PREFIXES`]
    /// and a colon: `phrase:product`.
    ///
    /// # Errors
    ///
    /// Nothing follows the colon.
    pub fn entry(given: &str) -> Option<Result<Named, BadTarget>> {
        let (prefix, name) = given.split_once(':')?;
        let (_, kind) = PREFIXES.iter().find(|(p, _)| *p == prefix)?;
        let name = name.trim();
        // `phrase:{product}` names `phrase:product`.
        let name = name
            .strip_prefix('{')
            .and_then(|n| n.strip_suffix('}'))
            .unwrap_or(name);
        if name.is_empty() {
            return Some(Err(BadTarget {
                given: given.to_owned(),
                reason: format!("give a name after `{prefix}:`"),
            }));
        }
        Some(Ok(Named::Entry {
            kind: *kind,
            name: name.to_owned(),
        }))
    }

    /// A file and an optional heading, from a content path with an optional
    /// `#id`.
    ///
    /// # Errors
    ///
    /// `given` isn't a content path.
    pub fn path(given: &str) -> Result<Named, BadTarget> {
        let (path, id) = match given.split_once('#') {
            Some((path, id)) => (path, Some(id).filter(|id| !id.is_empty())),
            None => (given, None),
        };
        let path = RelPath::parse(path.trim_start_matches("./")).map_err(|_| BadTarget {
            given: given.to_owned(),
            reason: "it isn't a path in the project, or an entry such as `phrase:product`"
                .to_owned(),
        })?;
        Ok(Named::Path {
            path,
            id: id.map(str::to_owned),
        })
    }

    /// The thing in `project` this names, when it exists. A page's heading
    /// can be written in a fragment it includes: that heading, where it's
    /// written.
    pub fn find(&self, project: &Project) -> Option<Usable> {
        let model = project.model();
        match self {
            Named::Path { path, id: None } => project
                .file(path)
                .is_some()
                .then(|| Usable::File(path.clone())),
            Named::Path { path, id: Some(id) } => {
                project
                    .page_heading(path, id)
                    .map(|(file, h)| Usable::Heading {
                        file,
                        id: h.source_id.clone(),
                    })
            }
            Named::Entry { kind, name } => {
                let name = name.clone();
                match kind {
                    EntryKind::Phrase => model.has_phrase(&name).then_some(Usable::Phrase(name)),
                    EntryKind::Feature => model.feature(&name).map(|_| Usable::Feature(name)),
                    EntryKind::Term => project
                        .glossary()
                        .term(&name)
                        .is_some()
                        .then_some(Usable::Term(name)),
                    EntryKind::Dimension => model.dimension(&name).map(|_| Usable::Dimension(name)),
                    EntryKind::Note => model.note_type(&name).map(|_| Usable::Note(name)),
                    EntryKind::Widget => model.widget(&name).map(|_| Usable::Widget(name)),
                }
            }
        }
    }
}
