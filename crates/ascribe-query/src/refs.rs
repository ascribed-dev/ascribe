//! Where a page, a fragment, a heading, or a content model entry is used:
//! `file:line` for each place, from the search the editor's Find All
//! References and its inventory use ([`Project::uses`]). Unlike a text
//! search, it finds a phrase only where it's a phrase, a heading through
//! the pages that include its fragment, and a fragment wherever it's
//! included.

use ascribe_core::RelPath;
use ascribe_resolve::{FileKind, Project, Usable, UseKind};
use serde::Serialize;

use crate::lines::{Lines, project_path};
use crate::{ASCRIBE_VERSION, QueryError, SCHEMA_VERSION};

/// What `ascribe refs <TARGET>` answers.
#[derive(Debug, Serialize)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
pub struct Refs {
    /// The version of this schema. It changes only when a field is removed
    /// or changes meaning.
    pub schema_version: u32,
    /// The version of Ascribe that wrote it.
    pub ascribe_version: &'static str,
    /// The target, as given.
    pub target: String,
    /// What the target is.
    pub kind: TargetKind,
    /// Whether the target exists. When it doesn't, there are no places.
    pub exists: bool,
    /// The places that use it, file by file in path order and in document
    /// order within a file; at most `shown` of them.
    pub places: Vec<Place>,
    /// How many places use it.
    pub total: usize,
    /// How many are listed.
    pub shown: usize,
    /// Whether the list was cut: `shown` is less than `total`.
    pub truncated: bool,
    /// The command that lists them all, when the list was cut.
    pub next_command: Option<String>,
}

/// What a target is.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "lowercase")]
pub enum TargetKind {
    /// A page.
    Page,
    /// A fragment.
    Fragment,
    /// A heading, by its page and id.
    Heading,
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

/// A place a target is used.
#[derive(Debug, Serialize)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
pub struct Place {
    /// The file, from the project root, as `ascribe check` reports it.
    pub file: String,
    /// The line, from 1.
    pub line: u32,
    /// The column, from 1, in Unicode characters.
    pub column: u32,
    /// How it uses the target: `link`, `include`, `phrase`, `availability`,
    /// `term`, `variant`, `note`, or `widget`.
    #[serde(rename = "use")]
    pub use_kind: &'static str,
}

/// What `refs` asks about.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Asked {
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
        kind: TargetKind,
        /// Its key, id, or name.
        name: String,
    },
}

/// The prefixes that name a content model entry, and the kind each names.
pub const PREFIXES: &[(&str, TargetKind)] = &[
    ("phrase", TargetKind::Phrase),
    ("feature", TargetKind::Feature),
    ("term", TargetKind::Term),
    ("dimension", TargetKind::Dimension),
    ("note", TargetKind::Note),
    ("widget", TargetKind::Widget),
];

impl Asked {
    /// A content model entry, when `given` starts with one of [`PREFIXES`]
    /// and a colon: `phrase:product`.
    ///
    /// # Errors
    ///
    /// Nothing follows the colon.
    pub fn entry(given: &str) -> Option<Result<Asked, QueryError>> {
        let (prefix, name) = given.split_once(':')?;
        let (_, kind) = PREFIXES.iter().find(|(p, _)| *p == prefix)?;
        let name = name.trim();
        // `phrase:{product}` names `phrase:product`.
        let name = name
            .strip_prefix('{')
            .and_then(|n| n.strip_suffix('}'))
            .unwrap_or(name);
        if name.is_empty() {
            return Some(Err(QueryError::BadTarget {
                given: given.to_owned(),
                reason: format!("give a name after `{prefix}:`"),
            }));
        }
        Some(Ok(Asked::Entry {
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
    pub fn path(given: &str) -> Result<Asked, QueryError> {
        let (path, id) = match given.split_once('#') {
            Some((path, id)) => (path, Some(id).filter(|id| !id.is_empty())),
            None => (given, None),
        };
        let bad = |reason: &str| QueryError::BadTarget {
            given: given.to_owned(),
            reason: reason.to_owned(),
        };
        let path = RelPath::parse(path.trim_start_matches("./")).map_err(|_| {
            bad("it isn't a path in the project, or an entry such as `phrase:product`")
        })?;
        Ok(Asked::Path {
            path,
            id: id.map(str::to_owned),
        })
    }
}

/// The places that use `asked`, at most `limit` of them; `given` is the
/// target as the person wrote it.
pub fn refs(project: &Project, asked: &Asked, given: &str, limit: usize) -> Refs {
    let (kind, usable) = usable(project, asked);
    let places = usable
        .as_ref()
        .map(|target| project.uses(target))
        .unwrap_or_default();
    let mut lines = Lines::new(project);
    let total = places.len();
    let listed: Vec<Place> = places
        .iter()
        .take(limit)
        .map(|place| {
            let (line, column) = lines.at(&place.file, place.span.start());
            Place {
                file: project_path(project, &place.file),
                line,
                column,
                use_kind: use_name(place.kind),
            }
        })
        .collect();
    Refs {
        schema_version: SCHEMA_VERSION,
        ascribe_version: ASCRIBE_VERSION,
        target: given.to_owned(),
        kind,
        exists: usable.is_some(),
        shown: listed.len(),
        truncated: listed.len() < total,
        places: listed,
        total,
        next_command: None,
    }
}

/// What `asked` is, and the thing the search looks for, when it exists.
fn usable(project: &Project, asked: &Asked) -> (TargetKind, Option<Usable>) {
    let model = project.model();
    match asked {
        Asked::Path { path, id: None } => {
            let kind = match project.file(path).map(|f| f.kind) {
                Some(FileKind::Fragment) => TargetKind::Fragment,
                _ => TargetKind::Page,
            };
            let exists = project.file(path).is_some();
            (kind, exists.then(|| Usable::File(path.clone())))
        }
        Asked::Path { path, id: Some(id) } => {
            // A page's heading can be written in a fragment it includes:
            // that heading, where it's written.
            let found = project
                .page_heading(path, id)
                .map(|(file, h)| Usable::Heading {
                    file,
                    id: h.source_id.clone(),
                });
            (TargetKind::Heading, found)
        }
        Asked::Entry { kind, name } => {
            let name = name.clone();
            let found = match kind {
                TargetKind::Phrase => model.has_phrase(&name).then_some(Usable::Phrase(name)),
                TargetKind::Feature => model.feature(&name).map(|_| Usable::Feature(name)),
                TargetKind::Term => model
                    .glossary
                    .terms
                    .iter()
                    .any(|t| t.id == name)
                    .then_some(Usable::Term(name)),
                TargetKind::Dimension => model.dimension(&name).map(|_| Usable::Dimension(name)),
                TargetKind::Note => model.note_type(&name).map(|_| Usable::Note(name)),
                TargetKind::Widget => model.widget(&name).map(|_| Usable::Widget(name)),
                TargetKind::Page | TargetKind::Fragment | TargetKind::Heading => None,
            };
            (*kind, found)
        }
    }
}

fn use_name(kind: UseKind) -> &'static str {
    match kind {
        UseKind::Link => "link",
        UseKind::Include => "include",
        UseKind::Phrase => "phrase",
        UseKind::Availability => "availability",
        UseKind::Term => "term",
        UseKind::Variant => "variant",
        UseKind::Note => "note",
        UseKind::Widget => "widget",
    }
}
