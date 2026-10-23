//! Where a page, a fragment, a heading, an image, or a content model entry
//! is used:
//! `file:line` for each place, from the search the editor's Find All
//! References and its inventory use ([`Project::uses`]). Unlike a text
//! search, it finds a phrase only where it's a phrase, a heading through
//! the pages that include its fragment, and a fragment wherever it's
//! included. A memo file about the target is a place too (SPEC §7.7), at
//! its `about` line, whether or not the target exists. An image's places
//! are the files that show it, link to it, or name it in raw HTML or a
//! frontmatter field, as `image-unused` finds them. A line of a release
//! target, `release:<target>@<version>`, is used by each availability spec
//! that names a version in it.

use ascribe_core::availability::{Version, parse_version};
use ascribe_core::{LineIndex, RelPath, Span, WideEncoding};
use ascribe_resolve::{
    BadTarget, EntryKind, FileKind, MemoFile, MentionKind, Project, RefKind, Usable, UseKind,
    memo_target, place,
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
    /// An image under the content root.
    Image,
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
    /// A line of a release target, `release:<target>@<version>`.
    Release,
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
    /// `term`, `variant`, `note`, `widget`, `image`, an image that shows it,
    /// `html` or `frontmatter`, raw HTML or a frontmatter field that names
    /// it, or `memo`, a memo file about it.
    #[serde(rename = "use")]
    pub use_kind: &'static str,
    /// Whether it's a memo file kept on this machine only, in
    /// `.ascribe/memos/`, rather than the project's `memos/`; `false` for
    /// every place that isn't a memo.
    pub personal: bool,
    /// The availability spec, as written, for an `availability` place; null
    /// for every other place.
    pub spec: Option<String>,
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
        .unwrap_or_default()
        .into_iter()
        .map(|place| (place.file, place.span, use_name(place.kind), place.spec))
        .collect();
    // A placed memo is about what the target is, however each names it, and
    // a term's memo is on its heading; a detached one, about what it names
    // as written.
    let on = usable.clone().map(|u| memo_target(project, u));
    let about = memos
        .iter()
        .filter(|memo| {
            place(memo, project).is_some_and(|p| match (&p.target, &on) {
                (Some(target), Some(on)) => target == on,
                (None, None) => p.named == *asked,
                _ => false,
            })
        })
        .collect();
    answer(
        project,
        Found {
            kind,
            exists: usable.is_some(),
            places,
            about,
        },
        given,
        limit,
    )
}

/// The places that use the image at `image`, a path from the content root,
/// and the memo files of `memos` that name it, at most `limit` of them;
/// `given` is the target as the person wrote it. The caller has found the
/// image ([`ascribe_check::image_file`]).
pub fn image_refs(
    project: &Project,
    image: &RelPath,
    given: &str,
    limit: usize,
    memos: &[MemoFile],
) -> Refs {
    let mut places: Vec<Spot> = project
        .asset_users(image)
        .iter()
        .map(|site| {
            let used = match site.kind {
                RefKind::Image => "image",
                RefKind::Link => "link",
            };
            (site.file.clone(), site.span, used, None)
        })
        .collect();
    for file in project.files() {
        places.extend(
            file.mentions()
                .into_iter()
                .filter(|m| m.path == *image)
                .map(|m| {
                    let used = match m.kind {
                        MentionKind::Html => "html",
                        MentionKind::Frontmatter => "frontmatter",
                    };
                    (file.path.clone(), m.span, used, None)
                }),
        );
    }
    places.sort_by(|a, b| (&a.0, a.1.start()).cmp(&(&b.0, b.1.start())));
    places.dedup();
    // A memo can't place an image, so one about it is detached, and names it
    // as written.
    let named = Asked::Path {
        path: image.clone(),
        id: None,
    };
    let about = memos
        .iter()
        .filter(|memo| place(memo, project).is_some_and(|p| p.target.is_none() && p.named == named))
        .collect();
    answer(
        project,
        Found {
            kind: TargetKind::Image,
            exists: true,
            places,
            about,
        },
        given,
        limit,
    )
}

/// A line of a release target, as `refs` reads `release:<target>@<version>`;
/// `None` when `given` doesn't start with `release:`.
///
/// # Errors
///
/// No target before `@`, or no version after it, or one that isn't a
/// version.
pub fn release_target(given: &str) -> Option<Result<(String, Version), QueryError>> {
    let rest = given.strip_prefix("release:")?;
    let bad = |reason: &str| QueryError::BadTarget {
        given: given.to_owned(),
        reason: reason.to_owned(),
    };
    let Some((target, version)) = rest.split_once('@') else {
        return Some(Err(bad(
            "give a release target and a version: `release:<target>@<version>`",
        )));
    };
    let target = target.trim();
    if target.is_empty() {
        return Some(Err(bad("give a release target before `@`")));
    }
    Some(
        parse_version(version.trim(), 0)
            .map(|v| (target.to_owned(), v))
            .map_err(|_| bad("give a version after `@`, such as `3.4`")),
    )
}

/// The availability specs that name a version in `line` of the release
/// target `target`, at most `limit` of them; `given` is the target as the
/// person wrote it. A target the project lists no releases for doesn't
/// exist, and has no places.
pub fn release_refs(
    project: &Project,
    target: &str,
    line: &Version,
    given: &str,
    limit: usize,
) -> Refs {
    let exists = project.model().releases_of(target).is_some();
    let places = if exists {
        project
            .release_uses(target, line)
            .into_iter()
            .map(|place| (place.file, place.span, use_name(place.kind), place.spec))
            .collect()
    } else {
        Vec::new()
    };
    answer(
        project,
        Found {
            kind: TargetKind::Release,
            exists,
            places,
            about: Vec::new(),
        },
        given,
        limit,
    )
}

/// One place: its file, where it is, its `use`, and its spec.
type Spot = (RelPath, Span, &'static str, Option<String>);

/// What a target is, and what names it.
struct Found<'m> {
    kind: TargetKind,
    exists: bool,
    /// The places that use it.
    places: Vec<Spot>,
    /// The memo files about it.
    about: Vec<&'m MemoFile>,
}

/// The answer that lists what `found` found, at most `limit` places.
fn answer(project: &Project, found: Found<'_>, given: &str, limit: usize) -> Refs {
    let Found {
        kind,
        exists,
        places,
        about,
    } = found;
    let mut lines = Lines::new(project);
    let total = places.len() + about.len();
    let mut listed: Vec<Place> = places
        .iter()
        .take(limit)
        .map(|(file, span, used, spec)| {
            let (line, column) = lines.at(file, span.start());
            Place {
                file: project_path(project, file),
                line,
                column,
                use_kind: used,
                personal: false,
                spec: spec.clone(),
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
            spec: None,
        }
    }));
    Refs {
        schema_version: SCHEMA_VERSION,
        ascribe_version: ASCRIBE_VERSION,
        target: given.to_owned(),
        kind,
        exists,
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
            Some(FileKind::Page) => TargetKind::Page,
            // A path that names nothing is what its extension says.
            None if is_image(path) => TargetKind::Image,
            None => TargetKind::Page,
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

/// Whether a path's extension is an image's, as the image checks read one.
fn is_image(path: &RelPath) -> bool {
    path.extension()
        .is_some_and(|e| ascribe_check::IMAGE_EXTENSIONS.contains(&e.to_ascii_lowercase().as_str()))
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
