//! The `ascribe/page` request: what's known about a page, and what links to
//! and includes the heading at the cursor, for the editor's This page view.
//!
//! It also lists the memo files about the heading or the page.
//!
//! One request answers what Used by asked two for (`ascribe/context`, then
//! `textDocument/references`): the heading whose section holds the cursor is
//! found here, and its links and includes come from the same search Find All
//! References lists from ([`ascribe_resolve::Project::uses`]). What doesn't
//! depend on the cursor, the links to and includes of the file and each of
//! its headings, is found in one search ([`Project::uses_of_file`]) and kept
//! for the file until the snapshot changes ([`PageCache`]), so a cursor move
//! only picks the heading's share. Every answer is from the current
//! snapshot, unsaved edits included.

use std::sync::{Arc, Mutex, PoisonError, Weak};

use ascribe_core::{LineIndex, RelPath, Router};
use ascribe_resolve::{AstroRouter, FileIndex, FileKind, Project, Usable, Use, UseKind};
use lsp_types::{Position, Range, TextDocumentIdentifier};
use serde::{Deserialize, Serialize};

use crate::nav::{Ctx, Lines};

/// The request's method name.
pub const METHOD: &str = "ascribe/page";

/// The parameters of `ascribe/page`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PageParams {
    /// The page or fragment.
    pub text_document: TextDocumentIdentifier,
    /// The cursor.
    pub position: Position,
}

/// The answer to `ascribe/page`. A document that isn't a page or a fragment
/// of the project gets an empty answer: no `about`, no heading, and empty
/// lists.
#[derive(Debug, Default, Serialize, PartialEq)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct PageResult {
    /// What the page is; `null` when the document isn't a page or a
    /// fragment.
    pub about: Option<PageAbout>,
    /// The heading whose section holds the cursor, the innermost one with an
    /// id; `null` when the cursor is in none, and the lists are then about
    /// the whole file.
    pub heading: Option<PageHeading>,
    /// The links to the heading, or to the file when there's no heading:
    /// from other files, and, for a heading, from its own page and from a
    /// page that includes it. File by file in content path order, and in
    /// document order within a file.
    pub linked_from: Vec<PagePlace>,
    /// The `@include`s of the file, or of the heading's section, in the
    /// same order.
    pub included_by: Vec<PagePlace>,
    /// The memo files about the heading, or about the file when there's no
    /// heading, or about a block in it (SPEC §7.7): the committed ones, then
    /// the personal ones, each in path order. A detached memo isn't listed.
    pub memos: Vec<PageMemo>,
}

/// What a page or fragment is.
#[derive(Debug, Serialize, PartialEq)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct PageAbout {
    /// Its content path.
    pub path: String,
    /// Whether it's a fragment, which is published only where it's included.
    pub fragment: bool,
    /// Its title (frontmatter `title`).
    pub title: Option<String>,
    /// Its content type; `null` for a fragment, and when no one type
    /// applies.
    #[serde(rename = "type")]
    pub content_type: Option<String>,
    /// Its address, from the host's root, as the site output's links write
    /// it; `null` for a fragment.
    pub address: Option<String>,
    /// Its Markdown version's address, from the host's root, when the
    /// project publishes one (`[consumer] agents = true`); `null` otherwise,
    /// and for a fragment.
    pub markdown_address: Option<String>,
}

/// A heading of the page.
#[derive(Debug, Serialize, PartialEq)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct PageHeading {
    /// Its source id: its `@id`, or its slug.
    pub id: String,
    /// Its text as plain text, with each declared phrase replaced by its
    /// value.
    pub text: String,
    /// The heading line, in the negotiated position encoding.
    #[cfg_attr(feature = "json-schema", schemars(with = "crate::schema::LspRange"))]
    pub range: Range,
}

/// A memo This page lists.
#[derive(Clone, Debug, Serialize, PartialEq)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct PageMemo {
    /// The memo file's `file:` URI.
    pub uri: String,
    /// Its path from the project root.
    pub path: String,
    /// The first line of its text that isn't blank.
    pub first_line: String,
    /// `placed`, or `outdated` when no block holds its quote.
    pub state: String,
    /// Whether it's a personal memo, kept in `.ascribe/memos/` on this
    /// machine only.
    pub personal: bool,
    /// Where its text starts, in the memo file, in the negotiated position
    /// encoding.
    #[cfg_attr(feature = "json-schema", schemars(with = "crate::schema::LspRange"))]
    pub body: Range,
}

/// A link or an `@include`.
#[derive(Clone, Debug, Serialize, PartialEq)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct PagePlace {
    /// The `file:` URI of the file it's written in.
    pub uri: String,
    /// That file's content path.
    pub path: String,
    /// The link or the directive line, in the negotiated position encoding.
    #[cfg_attr(feature = "json-schema", schemars(with = "crate::schema::LspRange"))]
    pub range: Range,
    /// The text of the line it starts on, trimmed.
    pub text: String,
}

/// What doesn't depend on the cursor, for the file asked about last, kept
/// between requests: a cursor move asks again about the same file, and the
/// search reads every file.
#[derive(Clone, Default)]
pub(crate) struct PageCache {
    entry: Arc<Mutex<Option<KeptFor>>>,
}

/// The project and the file what's kept was found for, and what's kept.
type KeptFor = (Weak<Project>, RelPath, Arc<Kept>);

/// What's kept for a file: its line index, and every place that uses it or
/// one of its headings, with what each uses.
struct Kept {
    index: LineIndex,
    uses: Vec<(Usable, UseKind, PagePlace)>,
}

impl PageCache {
    /// What's kept for the requested file in the request's snapshot, found
    /// again when the last request asked about another file or the snapshot
    /// has changed since.
    fn kept(&self, ctx: &Ctx, file: &FileIndex) -> Arc<Kept> {
        let project = ctx.snapshot.shared_project();
        let mut entry = self.entry.lock().unwrap_or_else(PoisonError::into_inner);
        if let Some((kept_from, path, kept)) = entry.as_ref()
            && *path == ctx.path
            && kept_from
                .upgrade()
                .is_some_and(|kept_from| Arc::ptr_eq(&kept_from, &project))
        {
            return kept.clone();
        }
        let mut lines = Lines::new(ctx);
        let uses = project
            .uses_of_file(&ctx.path)
            .into_iter()
            .filter_map(|(used, at)| Some((used, at.kind, place(ctx, &mut lines, &at)?)))
            .collect();
        let kept = Arc::new(Kept {
            index: LineIndex::new(&file.source),
            uses,
        });
        *entry = Some((Arc::downgrade(&project), ctx.path.clone(), kept.clone()));
        kept
    }
}

/// What's known about the requested file, and the heading at `position`.
pub(crate) fn page(ctx: &Ctx, cache: &PageCache, position: Position) -> PageResult {
    let Some(file) = ctx.file() else {
        return PageResult::default();
    };
    let kept = cache.kept(ctx, file);
    let offset = ctx
        .encoding
        .offset_lenient(&kept.index, &file.source, position);
    let heading = file
        .headings
        .iter()
        .filter(|h| !h.source_id.is_empty())
        .filter(|h| h.section.start() <= offset && offset <= h.section.end())
        .min_by_key(|h| h.section.len());
    let target = match heading {
        Some(h) => Usable::Heading {
            file: ctx.path.clone(),
            id: h.source_id.clone(),
        },
        None => Usable::File(ctx.path.clone()),
    };
    let mut linked_from = Vec::new();
    let mut included_by = Vec::new();
    for (_, kind, place) in kept.uses.iter().filter(|(used, _, _)| *used == target) {
        match kind {
            UseKind::Include => included_by.push(place.clone()),
            _ => linked_from.push(place.clone()),
        }
    }
    PageResult {
        about: Some(about(ctx, file)),
        heading: heading.map(|h| PageHeading {
            id: h.source_id.clone(),
            text: h.text.clone(),
            range: ctx.encoding.range(&kept.index, h.span),
        }),
        memos: crate::memos::about(ctx, &target),
        linked_from,
        included_by,
    }
}

fn about(ctx: &Ctx, file: &FileIndex) -> PageAbout {
    let fragment = file.kind == FileKind::Fragment;
    let router = (!fragment).then(|| AstroRouter::from_consumer(&ctx.model.consumer));
    PageAbout {
        path: file.path.to_string(),
        fragment,
        title: file.title.clone(),
        content_type: file.content_type(&ctx.model).map(|t| t.name.clone()),
        address: router.as_ref().map(|r| r.route(&file.path)),
        markdown_address: router
            .as_ref()
            .filter(|_| ctx.model.consumer.agents)
            .map(|r| r.markdown_url(&file.path)),
    }
}

fn place(ctx: &Ctx, lines: &mut Lines<'_>, at: &Use) -> Option<PagePlace> {
    let (path, span) = (&at.file, at.span);
    let source = &ctx.snapshot.file(path)?.source;
    let start = source.get(..span.start())?.rfind('\n').map_or(0, |n| n + 1);
    let end = source
        .get(start..)?
        .find('\n')
        .map_or(source.len(), |n| start + n);
    Some(PagePlace {
        uri: ctx.uri_of(path)?.as_str().to_owned(),
        path: path.to_string(),
        range: lines.range(path, span)?,
        text: source.get(start..end)?.trim().to_owned(),
    })
}
