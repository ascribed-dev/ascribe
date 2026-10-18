//! Where a page, a fragment, a heading, or a content model entry is used:
//! `file:line` for each place, from the search the editor's Find All
//! References and its inventory use ([`Project::uses`]). Unlike a text
//! search, it finds a phrase only where it's a phrase, a heading through
//! the pages that include its fragment, and a fragment wherever it's
//! included. A memo file about the target is a place too (SPEC §7.7), at
//! its `about` line, whether or not the target exists.

use ascribe_core::{LineIndex, WideEncoding};
use ascribe_resolve::{
    BadTarget, EntryKind, FileKind, MemoFile, Project, Usable, UseKind, memo_target, place,
};
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
    /// Whether the target exists. When it doesn't, the only places are the
    /// memo files that name it.
    pub exists: bool,
    /// The places that use it, file by file in path order and in document
    /// order within a file, then the memo files about it, in path order; at
    /// most `shown` of them.
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
    /// `term`, `variant`, `note`, `widget`, or `memo`, a memo file about it.
    #[serde(rename = "use")]
    pub use_kind: &'static str,
    /// Whether it's a memo file kept on this machine only, in
    /// `.ascribe/memos/`, rather than the project's `memos/`; `false` for
    /// every place that isn't a memo.
    pub personal: bool,
}

/// What `refs` asks about: a target as `ascribe refs` and a memo's `about`
/// read one.
pub use ascribe_resolve::Named as Asked;

/// The prefixes that name a content model entry.
pub use ascribe_resolve::PREFIXES;

impl From<BadTarget> for QueryError {
    fn from(bad: BadTarget) -> QueryError {
        QueryError::BadTarget {
            given: bad.given,
            reason: bad.reason,
        }
    }
}

/// The places that use `asked`, and the memo files of `memos` about it, at
/// most `limit` of them; `given` is the target as the person wrote it.
pub fn refs(
    project: &Project,
    asked: &Asked,
    given: &str,
    limit: usize,
    memos: &[MemoFile],
) -> Refs {
    let (kind, usable) = usable(project, asked);
    let places = usable
        .as_ref()
        .map(|target| project.uses(target))
        .unwrap_or_default();
    // A placed memo is about what the target is, however each names it, and
    // a term's memo is on its heading; a detached one, about what it names
    // as written.
    let on = usable.clone().map(|u| memo_target(project, u));
    let about: Vec<&MemoFile> = memos
        .iter()
        .filter(|memo| {
            place(memo, project).is_some_and(|p| match (&p.target, &on) {
                (Some(target), Some(on)) => target == on,
                (None, None) => p.named == *asked,
                _ => false,
            })
        })
        .collect();
    let mut lines = Lines::new(project);
    let total = places.len() + about.len();
    let mut listed: Vec<Place> = places
        .iter()
        .take(limit)
        .map(|place| {
            let (line, column) = lines.at(&place.file, place.span.start());
            Place {
                file: project_path(project, &place.file),
                line,
                column,
                use_kind: use_name(place.kind),
                personal: false,
            }
        })
        .collect();
    let room = limit.saturating_sub(listed.len());
    listed.extend(about.iter().take(room).map(|memo| {
        let at = memo.about.as_ref().map_or(0, |a| a.line.start());
        let (line, column) = LineIndex::new(&memo.text)
            .wide_line_col(WideEncoding::Utf32, at)
            .map_or((1, 1), |p| (p.line + 1, p.col + 1));
        Place {
            file: memo.path.to_string(),
            line,
            column,
            use_kind: "memo",
            personal: memo.personal(),
        }
    }));
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
    let kind = match asked {
        Asked::Path { path, id: None } => match project.file(path).map(|f| f.kind) {
            Some(FileKind::Fragment) => TargetKind::Fragment,
            _ => TargetKind::Page,
        },
        Asked::Path { id: Some(_), .. } => TargetKind::Heading,
        Asked::Entry { kind, .. } => match kind {
            EntryKind::Phrase => TargetKind::Phrase,
            EntryKind::Feature => TargetKind::Feature,
            EntryKind::Term => TargetKind::Term,
            EntryKind::Dimension => TargetKind::Dimension,
            EntryKind::Note => TargetKind::Note,
            EntryKind::Widget => TargetKind::Widget,
        },
    };
    (kind, asked.find(project))
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
        // `ascribe refs` doesn't ask about content types, so it lists none.
        UseKind::Type => "type",
    }
}
