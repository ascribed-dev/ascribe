//! The `ascribe/search` request: where the project's pages say something, as
//! their readers read it, for the editor's **Search Pages as Readers See
//! Them**. Each place is the line of the source to edit.
//!
//! The places are found by [`ascribe_resolve::reading::find`], the function
//! `ascribe search` calls, over the current snapshot, so unsaved text is
//! searched. Each build's reading of each page is kept with the snapshot
//! ([`Readings`]): made the first time a search asks for the build, and made
//! again for the pages an update may have changed, as the editor's
//! `ResolvedCache` is, so a query as you type is the scan alone.

use std::collections::{BTreeMap, HashMap};
use std::path::Path;
use std::sync::{Arc, Mutex, PoisonError};

use ascribe_core::path::normalize;
use ascribe_core::{FileId, LineIndex, RelPath, Span};
use ascribe_model::Build;
use ascribe_resolve::reading::{self, At, In, Nothing, Origin, Query, Reading, Through};
use ascribe_resolve::{Affected, AstroRouter, ModelImpact, Snapshot, Version};
use lsp_types::{Range, TextDocumentIdentifier};
use serde::{Deserialize, Serialize};

use crate::nav::Ctx;
use crate::uri::path_to_uri;

/// The request's method name.
pub const METHOD: &str = "ascribe/search";

/// How many places an answer lists, unless the request says otherwise: as
/// many as `ascribe search` shows.
const LIMIT: usize = 50;

/// How many of the pages that show a place it names.
const PAGES: usize = 3;

/// How many places [`Nothing::Unpublished`] names.
const WRITTEN: usize = 3;

/// How many updates are kept for the readings before they're all made
/// again instead.
const PENDING: usize = 64;

/// The parameters of `ascribe/search`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchParams {
    /// Any of the project's files: a source file, `ascribe.toml`, or another
    /// file in the project's folder. It says which project to search.
    pub text_document: TextDocumentIdentifier,
    /// What to find, as `ascribe search` takes it: words, which must all be
    /// in one block in any order, or a phrase in double quotes, matched as
    /// written. Case is ignored.
    pub query: String,
    /// The build to search. Without one, every build.
    #[serde(default)]
    pub build: Option<String>,
    /// The most places to list; 50 without one.
    #[serde(default)]
    pub limit: Option<usize>,
}

/// The answer to `ascribe/search`. A document that isn't one of the
/// project's files gets an answer with nothing searched.
#[derive(Debug, Default, Serialize, PartialEq)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct SearchResult {
    /// The builds searched, in the content model's order.
    pub builds: Vec<String>,
    /// The places, at most `limit` of them, in the order `ascribe search`
    /// lists them: titles, then headings, then the rest; in each, the places
    /// written in pages, then those in fragments and code files, each in
    /// path order and in order within a file.
    pub places: Vec<SearchPlace>,
    /// How many places match.
    pub total: usize,
    /// Whether the list was cut: there are more than `places`.
    pub truncated: bool,
    /// When nothing matched, what the source says about the words, as the
    /// last line of `ascribe search` says it: that no page has them, where
    /// they're written though no build searched publishes them there, or
    /// the builds not searched that do. `null` when something matched, or
    /// the query couldn't be read.
    pub nothing: Option<String>,
    /// Why nothing was searched: the query can't be read (a quote isn't
    /// closed, or it has no words), or the build isn't one of the content
    /// model's. `null` when it was searched.
    pub problem: Option<String>,
}

/// A place the query matches: a line of the source to edit.
#[derive(Debug, Serialize, PartialEq)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct SearchPlace {
    /// The `file:` URI of the file to edit: a page, a fragment, a snippet's
    /// code file, or `ascribe.toml` for a phrase.
    pub uri: String,
    /// The match's first character to the end of its line, in the file.
    /// For a phrase, its key to the end of its line in `ascribe.toml`.
    #[cfg_attr(feature = "json-schema", schemars(with = "crate::schema::LspRange"))]
    pub range: Range,
    /// The file as the editor names it: a source's content path, a code
    /// file's path from the project root, or `ascribe.toml`.
    pub file: String,
    /// What it is: `title`, `description`, `heading`, `text`, `table`,
    /// `code`, or `phrase`, a phrase's value at its line in `ascribe.toml`.
    #[serde(rename = "in")]
    pub kind: SearchIn,
    /// The block's text as a reader reads it, whole; for a phrase, its
    /// value.
    pub text: String,
    /// Where the match is in `text`: its first character and the one after
    /// its last, in Unicode characters.
    #[serde(rename = "match")]
    pub matched: [usize; 2],
    /// What the match reads through that the line doesn't spell, in order.
    pub through: Vec<SearchThrough>,
    /// The arm of a group the block is written in, as its attributes are
    /// written (`deployment=cloud`), or else its title.
    pub arm: Option<String>,
    /// The pages that show it, in content path order, at most three.
    pub pages: Vec<SearchPage>,
    /// How many pages show it; for a phrase, how many places in the source
    /// use it.
    pub pages_total: usize,
    /// The builds it's in, in the content model's order, when it isn't in
    /// every build searched; empty when it is.
    pub builds: Vec<String>,
}

/// What a place is.
#[derive(Clone, Copy, Debug, Serialize, PartialEq, Eq)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "lowercase")]
pub enum SearchIn {
    /// The page's title, at its key in the frontmatter.
    Title,
    /// The field the page's type marks `role = "description"`, at its key.
    Description,
    /// A heading.
    Heading,
    /// Prose.
    Text,
    /// A table's row.
    Table,
    /// A code block, or the code a `@snippet` shows.
    Code,
    /// A phrase's value, at its line in `ascribe.toml`.
    Phrase,
}

impl From<In> for SearchIn {
    fn from(kind: In) -> SearchIn {
        match kind {
            In::Title => SearchIn::Title,
            In::Description => SearchIn::Description,
            In::Heading => SearchIn::Heading,
            // Search never matches an HTML block or a directive's line.
            In::Text | In::Html | In::Directive => SearchIn::Text,
            In::Table => SearchIn::Table,
            In::Code => SearchIn::Code,
            In::Phrase => SearchIn::Phrase,
        }
    }
}

/// What a match reads through that its line doesn't spell.
#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "lowercase")]
pub enum SearchThrough {
    /// A phrase's value, by its key.
    Phrase(String),
    /// The title of a page a link with no text takes, or the text of the
    /// heading it names: `page.md` or `page.md#id`.
    Title(String),
    /// A snippet's code: its address, as written (`code:app.py#main`).
    Code(String),
}

/// A page that shows a place.
#[derive(Debug, Serialize, PartialEq)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct SearchPage {
    /// The page's content path.
    pub page: String,
    /// The heading the place is under there, as a reader reads it; for a
    /// heading, the heading itself.
    pub heading: Option<String>,
    /// That heading's id.
    pub id: Option<String>,
    /// The builds that show it there, in the content model's order.
    pub builds: Vec<String>,
}

/// Each build's reading of each page, kept between searches. An update the
/// server applies is queued here ([`Readings::absorb`]) and applied before
/// the next search: the pages it may have changed are read again, and after
/// a change to the content model beyond its warnings, every page.
#[derive(Default)]
pub(crate) struct Readings {
    kept: Mutex<Kept>,
    pending: Mutex<Vec<Pending>>,
}

#[derive(Default)]
struct Kept {
    /// By build, then page. `None` records a page the build doesn't publish.
    builds: HashMap<String, BTreeMap<RelPath, Option<Arc<Reading>>>>,
    /// The latest snapshot the updates applied so far reach.
    version: Option<Version>,
}

/// An update to apply to the readings.
enum Pending {
    /// The pages it may have changed.
    Update(Affected),
    /// Too many updates were queued: every page, as of this snapshot.
    All(Version),
}

impl Pending {
    fn version(&self) -> Version {
        match self {
            Pending::Update(affected) => affected.version,
            Pending::All(version) => *version,
        }
    }
}

impl Readings {
    /// Queues an update the server applied, for the next search.
    pub(crate) fn absorb(&self, affected: &Affected) {
        if affected.is_empty() {
            return;
        }
        let mut pending = self.pending.lock().unwrap_or_else(PoisonError::into_inner);
        if pending.len() >= PENDING {
            pending.clear();
            pending.push(Pending::All(affected.version));
        } else {
            pending.push(Pending::Update(affected.clone()));
        }
    }

    /// The readings of every page each of `builds` publishes, in path order,
    /// in `snapshot`: those kept, and the rest made and kept.
    fn of(&self, snapshot: &Snapshot, builds: &[&Build]) -> Vec<Vec<Arc<Reading>>> {
        let mut kept = self.kept.lock().unwrap_or_else(PoisonError::into_inner);
        {
            let mut pending = self.pending.lock().unwrap_or_else(PoisonError::into_inner);
            let now = snapshot.version();
            let due = pending.iter().take_while(|p| p.version() <= now).count();
            for update in pending.drain(..due) {
                kept.apply(&update);
            }
        }
        // A search from a snapshot older than one already applied reads what's
        // kept, but keeps nothing of its own.
        let keep = kept.version.is_none_or(|v| v <= snapshot.version());
        builds
            .iter()
            .map(|build| {
                let lined = snapshot.for_build(build);
                let lined = lined.as_ref();
                let router = AstroRouter::from_consumer(&lined.model().consumer);
                let pages = kept.builds.entry(build.name.clone()).or_default();
                let mut read = Vec::new();
                for page in snapshot.pages() {
                    let reading = match pages.get(&page.path) {
                        Some(hit) => hit.clone(),
                        None => {
                            let made = if snapshot.dropped(&page.path, build).is_some() {
                                None
                            } else {
                                lined
                                    .resolve_page(&page.path, build, &router)
                                    .map(|resolved| Arc::new(Reading::of(lined, &resolved)))
                            };
                            if keep {
                                pages.insert(page.path.clone(), made.clone());
                            }
                            made
                        }
                    };
                    read.extend(reading);
                }
                read
            })
            .collect()
    }
}

impl Kept {
    fn apply(&mut self, update: &Pending) {
        match update {
            Pending::All(_) => self.builds.clear(),
            Pending::Update(affected) => {
                for pages in self.builds.values_mut() {
                    if affected.model.is_some_and(|m| m > ModelImpact::Warnings) {
                        pages.clear();
                        continue;
                    }
                    for path in affected.re_resolve.iter().chain(&affected.removed) {
                        pages.remove(path);
                    }
                }
            }
        }
        let version = update.version();
        if self.version.is_none_or(|v| v < version) {
            self.version = Some(version);
        }
    }
}

/// Answers `ascribe/search` from `ctx`'s snapshot, with the readings kept in
/// `readings`.
pub(crate) fn search(ctx: &Ctx, readings: &Readings, params: &SearchParams) -> SearchResult {
    let query = match Query::parse(&params.query) {
        Ok(query) => query,
        Err(e) => {
            return SearchResult {
                problem: Some(format!("The query can't be read: {e}.")),
                ..SearchResult::default()
            };
        }
    };
    let model = &*ctx.model;
    let builds: Vec<&Build> = match &params.build {
        None => model.builds.iter().collect(),
        Some(name) => match model.build(name) {
            Some(build) => vec![build],
            None => {
                let names: Vec<&str> = model.builds.iter().map(|b| b.name.as_str()).collect();
                return SearchResult {
                    problem: Some(format!(
                        "The content model has no build named {name}. Its builds are {}.",
                        names.join(", ")
                    )),
                    ..SearchResult::default()
                };
            }
        },
    };
    let snapshot = &ctx.snapshot;
    let read = readings.of(snapshot, &builds);
    let found = reading::find(snapshot, read.iter().flatten().map(|r| &**r), &query);
    let names: Vec<String> = builds.iter().map(|b| b.name.clone()).collect();
    let nothing = found
        .places
        .is_empty()
        .then(|| nothing(ctx, readings, &builds, &query).said());
    let mut files = Files {
        ctx,
        read: &read,
        lines: HashMap::new(),
        model: LineIndex::new(&ctx.model_text),
    };
    let limit = params.limit.unwrap_or(LIMIT);
    let places = found
        .places
        .iter()
        .take(limit)
        .filter_map(|place| files.place(place, names.len()))
        .collect::<Vec<_>>();
    SearchResult {
        truncated: places.len() < found.places.len(),
        builds: names,
        places,
        total: found.places.len(),
        nothing,
        problem: None,
    }
}

/// What the source says about words no build searched publishes: the
/// builds not searched that publish them, when there are some, or else
/// where the source writes them. What `ascribe search` says.
fn nothing(ctx: &Ctx, readings: &Readings, searched: &[&Build], query: &Query) -> Nothing {
    let others: Vec<&Build> = ctx
        .model
        .builds
        .iter()
        .filter(|b| !searched.iter().any(|s| s.name == b.name))
        .collect();
    if !others.is_empty() {
        let read = readings.of(&ctx.snapshot, &others);
        let found = reading::find(&ctx.snapshot, read.iter().flatten().map(|r| &**r), query);
        let publish: Vec<String> = others
            .iter()
            .filter(|b| found.places.iter().any(|p| p.builds.contains(&b.name)))
            .map(|b| b.name.clone())
            .collect();
        if !publish.is_empty() {
            return Nothing::OtherBuilds(publish);
        }
    }
    let written = reading::scan(&ctx.snapshot, &ctx.model_text, query, None);
    if written.is_empty() {
        return Nothing::Unwritten;
    }
    Nothing::Unpublished {
        total: written.len(),
        written: written
            .into_iter()
            .take(WRITTEN)
            .map(|w| (w.file.to_string(), w.line))
            .collect(),
    }
}

/// The files places are in, and their lines, as a search turns places into
/// the editor's.
struct Files<'a> {
    ctx: &'a Ctx,
    read: &'a [Vec<Arc<Reading>>],
    /// Each source and code file's line index.
    lines: HashMap<FileId, LineIndex>,
    /// `ascribe.toml`'s.
    model: LineIndex,
}

impl Files<'_> {
    /// A place as the editor gets it; `None` for one in a file the editor
    /// can't open.
    fn place(&mut self, place: &reading::Place, builds: usize) -> Option<SearchPlace> {
        let ctx = self.ctx;
        let root = ctx.config.parent().unwrap_or(Path::new(""));
        let (path, file, range) = match &place.at {
            At::Source { file, offset, .. } => {
                let content = ctx.snapshot.path_of(*file)?;
                let source = &ctx.snapshot.file_by_id(*file)?.source;
                let range = self.rest_of_line(*file, source, *offset);
                (
                    ctx.content_dir.join(content.as_str()),
                    content.to_string(),
                    range,
                )
            }
            At::Code {
                file,
                path,
                from_root,
                offset,
                ..
            } => {
                let text = self.code_text(*file)?;
                let range = self.rest_of_line(*file, &text, *offset);
                (
                    normalize(&root.join(from_root.as_str())),
                    path.clone(),
                    range,
                )
            }
            At::Phrase { span, .. } => {
                let range = line_end_range(ctx, &self.model, &ctx.model_text, span.start());
                (
                    ctx.config.clone(),
                    ascribe_check::MODEL_FILE.to_owned(),
                    range,
                )
            }
        };
        let uri = path_to_uri(&path)?.as_str().to_owned();
        let chars = |byte: usize| place.text[..byte].chars().count();
        let through = place
            .through
            .iter()
            .map(|through| match through {
                Through::Phrase(key) => SearchThrough::Phrase(key.clone()),
                Through::Title { page, id } => SearchThrough::Title(match id {
                    Some(id) => format!("{page}#{id}"),
                    None => page.to_string(),
                }),
                Through::Code(address) => SearchThrough::Code(address.clone()),
            })
            .collect();
        let pages = place
            .shown
            .iter()
            .take(PAGES)
            .map(|shown| SearchPage {
                page: shown.page.to_string(),
                heading: shown.under.as_ref().map(|u| u.text.clone()),
                id: shown.under.as_ref().map(|u| u.id.clone()),
                builds: shown.builds.clone(),
            })
            .collect();
        let pages_total = match &place.at {
            At::Phrase { key, .. } => ctx
                .snapshot
                .uses(&ascribe_resolve::Usable::Phrase(key.clone()))
                .len(),
            At::Source { .. } | At::Code { .. } => place.shown.len(),
        };
        Some(SearchPlace {
            uri,
            range,
            file,
            kind: place.kind.into(),
            text: place.text.clone(),
            matched: [chars(place.matched.start), chars(place.matched.end)],
            through,
            arm: place.arm.clone(),
            pages,
            pages_total,
            builds: if place.builds.len() == builds {
                Vec::new()
            } else {
                place.builds.clone()
            },
        })
    }

    /// The range from `offset` to the end of its line, in a file.
    fn rest_of_line(&mut self, file: FileId, text: &str, offset: usize) -> Range {
        let index = self
            .lines
            .entry(file)
            .or_insert_with(|| LineIndex::new(text));
        line_end_range(self.ctx, index, text, offset)
    }

    /// A code file's text, as the snippet that shows it read it.
    fn code_text(&self, file: FileId) -> Option<Arc<str>> {
        self.read
            .iter()
            .flatten()
            .flat_map(|r| &r.blocks)
            .find_map(|block| match &block.origin {
                Origin::Snippet { snippet, .. } if block.file == file => Some(snippet.text.clone()),
                _ => None,
            })
    }
}

/// The range from `offset` to the end of its line in `text`, without the
/// line's ending.
fn line_end_range(ctx: &Ctx, index: &LineIndex, text: &str, offset: usize) -> Range {
    let offset = offset.min(text.len());
    let rest = text.get(offset..).unwrap_or_default();
    let end = offset + rest.find(['\n', '\r']).unwrap_or(rest.len());
    ctx.encoding.range(index, Span::new(offset, end))
}
