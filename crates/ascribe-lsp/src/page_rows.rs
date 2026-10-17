//! The rows of This page's About and Health, and what the page includes: the
//! parts of `ascribe/page` that say whether a page is published, listed,
//! due, over size, or unfinished.
//!
//! Each row is taken from the code that decides the same thing for a check
//! or an output, never from a second reading of the model: the description
//! is measured as `description-too-long` measures it, the review date is
//! compared as `review-overdue` compares it, the builds are the builds'
//! own drops ([`ascribe_resolve::Project::dropped`]), the listing is what
//! the page's record in `pages.json` says ([`ascribe_resolve::pages::records`]),
//! the size is `page-size`'s measure ([`ascribe_check::page::markdown_size`]),
//! the placeholders are `template-placeholder-left`'s, and a tested
//! example's state is the one `example-stale` and `example-unobserved` read.
//!
//! None of it depends on the cursor, so [`crate::page::PageCache`] keeps it
//! for the page with the rest of what it keeps.

use ascribe_core::{LineIndex, Span};
use ascribe_model::{Build, FieldRole, ListingFrom};
use ascribe_resolve::{
    CodeFiles, DefaultRouter, DropReason, ExampleStatus, FileIndex, FileKind, FileSystem, Scope,
    llms, resolve_snippet,
};
use lsp_types::Range;
use serde::Serialize;
use serde_yaml_ng::Value;

use crate::nav::Ctx;

/// A page's description, measured against its line in `llms.txt`.
#[derive(Clone, Debug, Serialize, PartialEq)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct PageDescription {
    /// The frontmatter field the page's type gives the description role.
    pub field: String,
    /// Its length in characters, as plain text on one line, as
    /// `description-too-long` measures it; `null` when the page has none.
    pub length: Option<usize>,
    /// The most characters it has before `description-too-long` reports it;
    /// `null` when the project publishes no `llms.txt`
    /// (`[consumer] agents`), which is the only place it's too long for.
    pub limit: Option<usize>,
    /// The field's line, or the page's first line when it has none, in the
    /// negotiated position encoding.
    #[cfg_attr(feature = "json-schema", schemars(with = "crate::schema::LspRange"))]
    pub range: Range,
}

/// When a page is due for review.
#[derive(Clone, Debug, Serialize, PartialEq)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct PageReview {
    /// The frontmatter field with the review-date role.
    pub field: String,
    /// The date it holds, `YYYY-MM-DD`.
    pub date: String,
    /// The days from today to the date: negative once it's passed. `null`
    /// when the server has no day to count from.
    pub days: Option<i64>,
    /// Whether the date has passed, so `review-overdue` reports the page.
    pub overdue: bool,
    /// The field's line, in the negotiated position encoding.
    #[cfg_attr(feature = "json-schema", schemars(with = "crate::schema::LspRange"))]
    pub range: Range,
}

/// Whether a build publishes the page.
#[derive(Clone, Debug, Serialize, PartialEq)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct PageBuild {
    /// The build's name.
    pub name: String,
    /// Why the build leaves the page out; `null` when it publishes it.
    pub left_out: Option<LeftOut>,
}

/// Why a build leaves a page out (SPEC §9.3).
#[derive(Clone, Copy, Debug, Serialize, PartialEq, Eq)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub enum LeftOut {
    /// The page's `variant` names none of the values the build selects.
    Variant,
    /// The page's `available` makes it unavailable for the build.
    Availability,
}

/// Whether search engines list a page, and what sets that.
#[derive(Clone, Debug, Serialize, PartialEq)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct PageListing {
    /// The build it's read from: the editor's build when it publishes the
    /// page, and otherwise the first that does.
    pub build: String,
    /// `listed`, `unquoted`, or `unlisted`: `pages.json`'s `listing`.
    pub listing: String,
    /// What sets it, as `pages.json`'s `listing_from.level` says: `default`,
    /// `page`, `site`, `older-lines`, `folder`, or `type`.
    pub from: String,
    /// The rule of `[listing]` that sets it, as written; `null` otherwise.
    pub rule: Option<String>,
    /// The `file:` URI of the file that sets it: the page for its own
    /// `listing`, `ascribe.toml` for a rule; `null` when nothing does.
    pub uri: Option<String>,
    /// Where in that file, in the negotiated position encoding.
    #[cfg_attr(
        feature = "json-schema",
        schemars(with = "Option<crate::schema::LspRange>")
    )]
    pub range: Option<Range>,
}

/// A permalink declared in the file (SPEC §7.5).
#[derive(Clone, Debug, Serialize, PartialEq)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct PagePermalink {
    /// Its name.
    pub name: String,
    /// The text of the heading it leads to; `null` for the page's own.
    pub heading: Option<String>,
    /// The page's `permalink` line, or the heading's `@id` line, in the
    /// negotiated position encoding.
    #[cfg_attr(feature = "json-schema", schemars(with = "crate::schema::LspRange"))]
    pub range: Range,
}

/// What a page's template still leaves to fill in.
#[derive(Clone, Debug, Serialize, PartialEq)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct PageTemplate {
    /// The content type whose template it is.
    #[serde(rename = "type")]
    pub type_name: String,
    /// How many placeholders and template values are left, as
    /// `template-placeholder-left` finds them.
    pub left: usize,
}

/// How a page stands: its size, and its tested examples.
#[derive(Clone, Debug, Default, Serialize, PartialEq)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct PageHealth {
    /// Its plain Markdown's size in the build where it's largest; `null`
    /// for a fragment, and for a page no build publishes.
    pub size: Option<PageSize>,
    /// Each `@snippet` of the file that shows a tested example, in
    /// document order.
    pub examples: Vec<PageExample>,
}

/// A page's size, as `page-size` measures it.
#[derive(Clone, Debug, Serialize, PartialEq)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct PageSize {
    /// Characters of plain Markdown.
    pub characters: u64,
    /// The build it's that size in: the first of the largest.
    pub build: String,
    /// The size `page-size` reports it at; `null` when the check is off.
    pub limit: Option<u64>,
}

/// A tested example a `@snippet` of the file shows.
#[derive(Clone, Debug, Serialize, PartialEq)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct PageExample {
    /// The `@snippet`'s address, as written.
    pub address: String,
    /// Whether it's tested as it is now.
    pub state: ExampleState,
    /// What it was tested with, when it's tested.
    pub tested: Option<String>,
    /// The `@snippet` line, in the negotiated position encoding.
    #[cfg_attr(feature = "json-schema", schemars(with = "crate::schema::LspRange"))]
    pub range: Range,
}

/// A tested example's state.
#[derive(Clone, Copy, Debug, Serialize, PartialEq, Eq)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub enum ExampleState {
    /// An observation lists it as it is now.
    Tested,
    /// It changed after the observation that lists it (`example-stale`).
    Stale,
    /// No observation lists it (`example-unobserved`).
    Untested,
}

/// What a file includes: a fragment, or a snippet of a code file.
#[derive(Clone, Debug, Serialize, PartialEq)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct PageInclude {
    /// `include` for an `@include`, `snippet` for a `@snippet`.
    pub kind: IncludeKind,
    /// What it names, as written.
    pub written: String,
    /// The file it shows: a fragment's content path, or a code file's path
    /// from the project root; `null` when the address names no file.
    pub path: Option<String>,
    /// That file's `file:` URI.
    pub uri: Option<String>,
    /// The directive line, in the negotiated position encoding.
    #[cfg_attr(feature = "json-schema", schemars(with = "crate::schema::LspRange"))]
    pub range: Range,
}

/// What kind of directive includes it.
#[derive(Clone, Copy, Debug, Serialize, PartialEq, Eq)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub enum IncludeKind {
    /// `@include`.
    Include,
    /// `@snippet`.
    Snippet,
}

/// The rows of About beyond what the page is, for `file`.
pub(crate) struct AboutRows {
    pub description: Option<PageDescription>,
    pub review: Option<PageReview>,
    pub availability: Option<String>,
    pub builds: Vec<PageBuild>,
    pub listing: Option<PageListing>,
    pub permalinks: Vec<PagePermalink>,
    pub template: Option<PageTemplate>,
}

/// Every row that doesn't depend on the cursor.
pub(crate) struct Rows {
    pub about: AboutRows,
    pub health: PageHealth,
    pub includes: Vec<PageInclude>,
}

/// The rows for the requested file, whose line index is `lines`.
pub(crate) fn rows(ctx: &Ctx, file: &FileIndex, lines: &LineIndex) -> Rows {
    let range = |span: Span| ctx.encoding.range(lines, span);
    let page = file.kind == FileKind::Page;
    let builds = if page { builds(ctx, file) } else { Vec::new() };
    let published = published(ctx, &builds);
    let (listing, size) = match page {
        true => resolved_rows(ctx, file, &published),
        false => (None, None),
    };
    Rows {
        about: AboutRows {
            description: page.then(|| description(ctx, file, &range)).flatten(),
            review: page.then(|| review(ctx, file, &range)).flatten(),
            availability: page.then(|| availability(ctx, file)).flatten(),
            builds,
            listing,
            permalinks: permalinks(ctx, file, &range),
            template: page.then(|| template(ctx, file)).flatten(),
        },
        health: PageHealth {
            size,
            examples: examples(ctx, file, &range),
        },
        includes: includes(ctx, file, &range),
    }
}

/// Whether each of the project's builds publishes the page, in declaration
/// order: what `check` resolves the page for in each. The inventory's
/// `builds` are the ones that do.
pub(crate) fn builds(ctx: &Ctx, file: &FileIndex) -> Vec<PageBuild> {
    ctx.model
        .builds
        .iter()
        .map(|build| PageBuild {
            name: build.name.clone(),
            left_out: ctx.snapshot.project().dropped(&file.path, build).map(
                |reason| match reason {
                    DropReason::Variant => LeftOut::Variant,
                    DropReason::Unavailable => LeftOut::Availability,
                },
            ),
        })
        .collect()
}

/// The builds of `builds` that publish the page.
fn published<'m>(ctx: &'m Ctx, builds: &[PageBuild]) -> Vec<&'m Build> {
    builds
        .iter()
        .filter(|b| b.left_out.is_none())
        .filter_map(|b| ctx.model.build(&b.name))
        .collect()
}

fn description(
    ctx: &Ctx,
    file: &FileIndex,
    range: &dyn Fn(Span) -> Range,
) -> Option<PageDescription> {
    let field = file
        .content_type(&ctx.model)?
        .frontmatter
        .field_with_role(FieldRole::Description)?;
    let raw = file
        .frontmatter
        .as_ref()
        .and_then(|f| f.get(field.name.as_str()))
        .and_then(Value::as_str);
    let at = frontmatter_line(file, &field.name).unwrap_or_else(|| first_line(file));
    Some(PageDescription {
        field: field.name.clone(),
        length: raw
            .and_then(|raw| llms::description_text(&ctx.model, raw))
            .map(|text| text.chars().count()),
        limit: ctx.model.consumer.agents.then_some(llms::DESCRIPTION_LIMIT),
        range: range(at),
    })
}

fn review(ctx: &Ctx, file: &FileIndex, range: &dyn Fn(Span) -> Range) -> Option<PageReview> {
    let field = file
        .content_type(&ctx.model)?
        .frontmatter
        .field_with_role(FieldRole::ReviewDate)?;
    let Some(Value::String(text)) = file.frontmatter.as_ref()?.get(field.name.as_str()) else {
        return None;
    };
    let due = ascribe_core::Date::parse(text)?;
    let at = frontmatter_line(file, &field.name).unwrap_or_else(|| first_line(file));
    Some(PageReview {
        field: field.name.clone(),
        date: due.to_string(),
        days: ctx.today.map(|today| due.days_since(today)),
        // As `review-overdue` decides it.
        overdue: ctx.today.is_some_and(|today| due < today),
        range: range(at),
    })
}

/// The page's own `available`, as shown.
fn availability(ctx: &Ctx, file: &FileIndex) -> Option<String> {
    ctx.snapshot
        .project()
        .scopes(&file.path)
        .into_iter()
        .find(|a| a.scope == Scope::Page)
        .map(|a| a.text.clone())
}

/// The listing and the size, which each need the page resolved for a build
/// that publishes it.
fn resolved_rows(
    ctx: &Ctx,
    file: &FileIndex,
    published: &[&Build],
) -> (Option<PageListing>, Option<PageSize>) {
    let index = ctx.snapshot.project();
    let router = DefaultRouter::from_consumer(&ctx.model.consumer);
    let root = ctx.config.parent().unwrap_or(&ctx.config);
    let limit = ascribe_check::page::size_limit(index);
    let editor = ctx.model.editor_default_build();
    let mut listing = None;
    let mut size: Option<PageSize> = None;
    for build in published {
        let Some(resolved) = index.resolve_page(&file.path, build, &router) else {
            continue;
        };
        if listing.is_none() || build.name == editor.name {
            listing = Some(listing_of(ctx, file, build, &resolved));
        }
        let Some(characters) = ascribe_check::page::markdown_size(index, root, build, &resolved)
        else {
            continue;
        };
        // The first of the largest, as `page-size` names it.
        if size.as_ref().is_none_or(|s| characters > s.characters) {
            size = Some(PageSize {
                characters,
                build: build.name.clone(),
                limit,
            });
        }
    }
    (listing, size)
}

/// The page's listing in `build`, from its record in `pages.json`.
fn listing_of(
    ctx: &Ctx,
    file: &FileIndex,
    build: &Build,
    resolved: &ascribe_resolve::ResolvedPage,
) -> PageListing {
    let records = ascribe_resolve::pages::records(&ctx.model, [resolved]);
    let (listing, from) = records
        .pages
        .into_iter()
        .next()
        .map(|r| (r.listing, r.listing_from))
        .unwrap_or((resolved.listing, resolved.listing_from.clone()));
    let (uri, range) = match &from {
        ListingFrom::Default => (None, None),
        ListingFrom::Page => (
            ctx.uri_of(&file.path),
            frontmatter_line(file, ascribe_model::Listing::KEY)
                .map(|span| ctx.encoding.range(&LineIndex::new(&file.source), span)),
        ),
        ListingFrom::Rule(rule) => (
            crate::uri::path_to_uri(&ctx.config),
            Some(
                ctx.encoding
                    .range(&LineIndex::new(&ctx.model_text), rule.span),
            ),
        ),
    };
    PageListing {
        build: build.name.clone(),
        listing: listing.name().to_owned(),
        from: from.level_name().to_owned(),
        rule: match &from {
            ListingFrom::Rule(rule) => Some(rule.text.clone()),
            ListingFrom::Default | ListingFrom::Page => None,
        },
        uri: uri.map(|u| u.as_str().to_owned()),
        range,
    }
}

/// The permalinks the file declares, as `ascribe permalinks list` lists
/// them, in document order.
fn permalinks(ctx: &Ctx, file: &FileIndex, range: &dyn Fn(Span) -> Range) -> Vec<PagePermalink> {
    ctx.snapshot
        .project()
        .declared_permalinks()
        .into_iter()
        .filter(|p| p.file == file.path)
        .map(|p| PagePermalink {
            heading: p
                .id
                .as_deref()
                .and_then(|id| file.heading_by_id(id))
                .map(|h| h.text.clone()),
            name: p.name,
            range: range(p.at.span),
        })
        .collect()
}

/// What the page's template leaves, when it leaves anything.
fn template(ctx: &Ctx, file: &FileIndex) -> Option<PageTemplate> {
    let page_type = file
        .content_type(&ctx.model)
        .filter(|t| t.template.is_some())?;
    let left = crate::edit::start::project(ctx)
        .template_places(&page_type.name, &file.source)
        .len();
    (left > 0).then(|| PageTemplate {
        type_name: page_type.name.clone(),
        left,
    })
}

/// Each tested example the file's `@snippet`s show, read now, as the hover
/// on a `@snippet` reads it: an observation written since counts.
fn examples(ctx: &Ctx, file: &FileIndex, range: &dyn Fn(Span) -> Range) -> Vec<PageExample> {
    let fs: &dyn FileSystem = &*ctx.fs;
    let code = CodeFiles::new();
    file.snippets
        .iter()
        .filter_map(|snippet| {
            let Some(Ok(address)) = &snippet.address else {
                return None;
            };
            let example = resolve_snippet(snippet, address, &ctx.model, fs, &code)
                .ok()?
                .example?;
            Some(PageExample {
                address: snippet.written.clone(),
                state: match example.status {
                    ExampleStatus::Current => ExampleState::Tested,
                    ExampleStatus::Stale => ExampleState::Stale,
                    ExampleStatus::Unobserved => ExampleState::Untested,
                },
                tested: example.tested,
                range: range(snippet.span),
            })
        })
        .collect()
}

/// The file's `@include`s and `@snippet`s, in document order.
fn includes(ctx: &Ctx, file: &FileIndex, range: &dyn Fn(Span) -> Range) -> Vec<PageInclude> {
    let fs: &dyn FileSystem = &*ctx.fs;
    let code = CodeFiles::new();
    let root = ctx.config.parent();
    let mut out: Vec<(Span, PageInclude)> = Vec::new();
    for include in &file.includes {
        let target = include
            .target
            .as_ref()
            .filter(|t| ctx.snapshot.file(t).is_some());
        out.push((
            include.span,
            PageInclude {
                kind: IncludeKind::Include,
                written: written(file, include.primary, &include.written),
                path: target.map(ToString::to_string),
                uri: target
                    .and_then(|t| ctx.uri_of(t))
                    .map(|u| u.as_str().to_owned()),
                range: range(include.span),
            },
        ));
    }
    for snippet in &file.snippets {
        let shown = match &snippet.address {
            Some(Ok(address)) => resolve_snippet(snippet, address, &ctx.model, fs, &code).ok(),
            _ => None,
        };
        let path = shown.map(|s| s.path);
        out.push((
            snippet.span,
            PageInclude {
                kind: IncludeKind::Snippet,
                written: snippet.written.clone(),
                uri: path
                    .as_ref()
                    .zip(root)
                    .and_then(|(p, root)| crate::uri::path_to_uri(&root.join(p.as_str())))
                    .map(|u| u.as_str().to_owned()),
                path: path.map(|p| p.to_string()),
                range: range(snippet.span),
            },
        ));
    }
    out.sort_by_key(|(span, _)| span.start());
    out.into_iter().map(|(_, include)| include).collect()
}

/// An `@include`'s primary as written, with its `#id`.
fn written(file: &FileIndex, primary: Option<Span>, path: &str) -> String {
    primary
        .and_then(|span| file.source.get(span.range()))
        .map_or_else(|| path.to_owned(), str::to_owned)
}

/// The span of the frontmatter's top-level `key:` line, from its key to the
/// end of its value.
fn frontmatter_line(file: &FileIndex, key: &str) -> Option<Span> {
    let content = file.document.frontmatter.as_ref()?.content;
    let text = file.source.get(content.range())?;
    let line = ascribe_syntax::frontmatter_lines(text)
        .into_iter()
        .find(|l| l.indent == 0 && l.item.is_none() && !l.block && l.key == Some(key))?;
    let base = content.start();
    let end = base + line.value_start + line.value.len();
    Some(Span::new(base + line.start, end.max(base + line.start)))
}

/// The file's first line, without its line ending.
fn first_line(file: &FileIndex) -> Span {
    let end = file.source.find(['\n', '\r']).unwrap_or(file.source.len());
    Span::new(0, end)
}
