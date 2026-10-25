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
//!
//! The rest of About, Health, and what the file includes are kept with it
//! ([`crate::page_rows`]): found again when the snapshot changes, when a
//! file outside the sources changes on disk (a source's observations, a
//! template), and on a new day. So are the memos about the file and its
//! headings, which every memo file is read for: found again, too, when a
//! memo file is opened, edited, or closed ([`PageCache::forget_memos`]).

use std::sync::{Arc, Mutex, PoisonError, Weak};

use ascribe_core::{LineIndex, RelPath, Router};
use ascribe_emit::{FormattedPiece, formatted_pieces};
use ascribe_model::inline;
use ascribe_resolve::{AstroRouter, FileIndex, FileKind, Project, Usable, Use, UseKind};
use lsp_types::{Position, Range, TextDocumentIdentifier};
use serde::{Deserialize, Serialize};

use crate::addresses::{PageOldAddress, Redirects};
use crate::nav::{Ctx, Lines};
use crate::page_rows::{
    PageBuild, PageDescription, PageHealth, PageInclude, PageListing, PagePermalink, PageReview,
    PageTemplate, Rows,
};

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
    /// How the page stands: its size and its tested examples. Empty when
    /// the document isn't a page or a fragment.
    pub health: PageHealth,
    /// The file's `@include`s and `@snippet`s, in document order.
    pub includes: Vec<PageInclude>,
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
    /// Its title (frontmatter `title`), as written.
    pub title: Option<String>,
    /// The title formatted, when its type's `title` sets `inline`: its
    /// pieces of text and code, as the JSON output writes a formatted title,
    /// with phrases substituted when the field takes them; `null` otherwise.
    pub formatted_title: Option<Vec<FormattedPiece>>,
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
    /// The project's published site, its origin (`[consumer] site`), which
    /// the addresses are on: `null` when the project names none, and for a
    /// fragment.
    pub site: Option<String>,
    /// Its description, when its type gives a field the description role;
    /// `null` for a fragment.
    pub description: Option<PageDescription>,
    /// When it's due for review, when its type gives a field the
    /// review-date role and the page has a date in it.
    pub review: Option<PageReview>,
    /// Its own `available`, as shown; `null` when it has none.
    pub availability: Option<String>,
    /// Whether each build publishes it, in declaration order; empty for a
    /// fragment, which is published only where it's included.
    pub builds: Vec<PageBuild>,
    /// Whether it's listed, and what sets that; `null` for a fragment and
    /// for a page no build publishes.
    pub listing: Option<PageListing>,
    /// The permalinks the file declares, in document order.
    pub permalinks: Vec<PagePermalink>,
    /// The old addresses that lead to it, each once, in address order: each
    /// `from` of a redirect a build writes to it in `redirects.json`, with
    /// its entry, as the server last compared the addresses, at the last
    /// save or change on disk. Empty for a fragment, and for a project
    /// without `ascribe.addresses`.
    pub also_answers_at: Vec<PageOldAddress>,
    /// What its type's template still leaves to fill in; `null` when its
    /// type has no template or nothing of it is left.
    pub template: Option<PageTemplate>,
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
    memos: Arc<Mutex<Option<MemosFor>>>,
}

/// The project and the file the memos kept were found for, and the memos
/// about the file, a heading in it, or a block in it, each with what it's
/// about.
type MemosFor = (Weak<Project>, RelPath, Arc<Vec<(Usable, PageMemo)>>);

/// The project, the file, and the day what's kept was found for, and
/// what's kept.
type KeptFor = (
    Weak<Project>,
    RelPath,
    Option<ascribe_core::Date>,
    Arc<Kept>,
);

/// What's kept for a file: its line index, every place that uses it or one
/// of its headings, with what each uses, and the rows that don't depend on
/// the cursor.
struct Kept {
    index: LineIndex,
    uses: Vec<(Usable, UseKind, PagePlace)>,
    rows: Rows,
}

impl PageCache {
    /// What's kept for the requested file in the request's snapshot, found
    /// again when the last request asked about another file or the snapshot
    /// has changed since.
    fn kept(&self, ctx: &Ctx, file: &FileIndex) -> Arc<Kept> {
        let project = ctx.snapshot.shared_project();
        let mut entry = self.entry.lock().unwrap_or_else(PoisonError::into_inner);
        if let Some((kept_from, path, day, kept)) = entry.as_ref()
            && *path == ctx.path
            && *day == ctx.today
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
        let index = LineIndex::new(&file.source);
        let rows = crate::page_rows::rows(ctx, file, &index);
        let kept = Arc::new(Kept { index, uses, rows });
        *entry = Some((
            Arc::downgrade(&project),
            ctx.path.clone(),
            ctx.today,
            kept.clone(),
        ));
        kept
    }

    /// The memos about the requested file in the request's snapshot, found
    /// again when the last request asked about another file, the snapshot
    /// has changed since, or a memo file has ([`PageCache::forget_memos`]).
    fn memos(&self, ctx: &Ctx) -> Arc<Vec<(Usable, PageMemo)>> {
        let project = ctx.snapshot.shared_project();
        let mut entry = self.memos.lock().unwrap_or_else(PoisonError::into_inner);
        if let Some((kept_from, path, memos)) = entry.as_ref()
            && *path == ctx.path
            && kept_from
                .upgrade()
                .is_some_and(|kept_from| Arc::ptr_eq(&kept_from, &project))
        {
            return memos.clone();
        }
        let memos = Arc::new(crate::memos::in_file(ctx));
        *entry = Some((Arc::downgrade(&project), ctx.path.clone(), memos.clone()));
        memos
    }

    /// Forgets what's kept: a file the snapshot doesn't hold changed, such
    /// as a source's observations or a template, which the rows read, or a
    /// memo file.
    pub(crate) fn forget(&self) {
        *self.entry.lock().unwrap_or_else(PoisonError::into_inner) = None;
        self.forget_memos();
    }

    /// Forgets the memos kept: a memo file was opened, edited, or closed,
    /// which changes its text as the editor has it.
    pub(crate) fn forget_memos(&self) {
        *self.memos.lock().unwrap_or_else(PoisonError::into_inner) = None;
    }
}

/// What's known about the requested file, and the heading at `position`,
/// with `redirects`, the old addresses the server last found.
pub(crate) fn page(
    ctx: &Ctx,
    cache: &PageCache,
    redirects: &Redirects,
    position: Position,
) -> PageResult {
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
        about: Some(about(ctx, file, &kept.rows, redirects)),
        heading: heading.map(|h| PageHeading {
            id: h.source_id.clone(),
            text: h.text.clone(),
            range: ctx.encoding.range(&kept.index, h.span),
        }),
        memos: cache
            .memos(ctx)
            .iter()
            .filter(|(about, _)| *about == target)
            .map(|(_, memo)| memo.clone())
            .collect(),
        linked_from,
        included_by,
        health: kept.rows.health.clone(),
        includes: kept.rows.includes.clone(),
    }
}

fn about(ctx: &Ctx, file: &FileIndex, rows: &Rows, redirects: &Redirects) -> PageAbout {
    let rows = &rows.about;
    let fragment = file.kind == FileKind::Fragment;
    let router = (!fragment).then(|| AstroRouter::from_consumer(&ctx.model.consumer));
    PageAbout {
        path: file.path.to_string(),
        fragment,
        title: file.title.clone(),
        formatted_title: formatted_title(ctx, file),
        content_type: file.content_type(&ctx.model).map(|t| t.name.clone()),
        address: router.as_ref().map(|r| r.route(&file.path)),
        markdown_address: router
            .as_ref()
            .filter(|_| ctx.model.consumer.agents)
            .map(|r| r.markdown_url(&file.path)),
        site: ctx.model.consumer.site.clone().filter(|_| !fragment),
        description: rows.description.clone(),
        review: rows.review.clone(),
        availability: rows.availability.clone(),
        builds: rows.builds.clone(),
        listing: rows.listing.clone(),
        permalinks: rows.permalinks.clone(),
        also_answers_at: if fragment {
            Vec::new()
        } else {
            redirects.of(&file.path)
        },
        template: rows.template.clone(),
    }
}

/// `file`'s title in pieces, when its type's `title` sets `inline`, read as
/// the build reads it: phrases substituted when the field takes them.
fn formatted_title(ctx: &Ctx, file: &FileIndex) -> Option<Vec<FormattedPiece>> {
    let title = file.title.as_deref()?;
    let field = file
        .content_type(&ctx.model)?
        .frontmatter
        .fields
        .iter()
        .find(|f| f.name == "title")
        .filter(|f| f.inline.is_some())?;
    let phrase = |key: &str| {
        field
            .phrases
            .then(|| ctx.model.phrase(key).map(str::to_owned))
            .flatten()
    };
    Some(formatted_pieces(&inline::parse(title, &phrase)))
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
