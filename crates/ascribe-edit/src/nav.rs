//! What an edit works from, and the helpers that read a file's syntax and
//! index at a cursor.
//!
//! An edit takes a [`Ctx`]: a snapshot of the project, the content model, and
//! the position encoding its edits are written in. The language server makes
//! one under its lock for each request, with the documents the editor has
//! open, and computes without the lock; a command loads one from the disk
//! ([`crate::load`]).

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use ascribe_core::path::normalize;
use ascribe_core::schema::Builtin;
use ascribe_core::{LineIndex, RelPath, Span};
use ascribe_model::{ContentModel, Listing};
use ascribe_resolve::{FileIndex, Heading, Include, PhraseUse, Reference, Snapshot};
use ascribe_syntax::{Block, BlockKind, DirectiveLine, PrimaryValue};
use lsp_types::{Location, Position, Range, Uri};

use crate::fsx::LayerFs;
use crate::position::Encoding;
use crate::uri::path_to_uri;

/// What an edit, or a request about a file, works from.
pub struct Ctx {
    /// The project as of the request.
    pub snapshot: Snapshot,
    /// The requested file's content path, or [`Ctx::PROJECT`].
    pub path: RelPath,
    /// The content model.
    pub model: Arc<ContentModel>,
    /// The text of `ascribe.toml`, for going to a phrase's or feature's entry.
    pub model_text: String,
    /// Whether `ascribe.toml` as it is now doesn't load, so `model` and
    /// `model_text` are the last that did.
    pub model_problem: bool,
    /// The project's `ascribe.toml`.
    pub config: PathBuf,
    /// The content root, absolute.
    pub content_dir: PathBuf,
    /// How the client counts columns.
    pub encoding: Encoding,
    /// The project's files that aren't sources, as the file-level checks
    /// read them: images, and the files of the content model's sources.
    pub fs: Arc<LayerFs>,
    /// The files `ascribe.toml`'s `[from]` takes from, read when an edit
    /// first asks ([`Ctx::taken`]).
    pub taken: std::sync::OnceLock<Vec<crate::model_file::TakenFile>>,
}

impl Ctx {
    /// The path of a request about the whole project asked through a file
    /// that isn't a source: a name at the content root that no source file
    /// can have.
    pub const PROJECT: &'static str = "ascribe.toml";

    /// The files `ascribe.toml`'s `[from]` takes from, as the disk has them,
    /// for an edit that writes a taken key where it's written (SPEC §7.9).
    pub fn taken(&self) -> &[crate::model_file::TakenFile] {
        self.taken
            .get_or_init(|| crate::model_file::read_taken(&self.model_text, &self.config))
    }

    /// `ascribe.toml`, parsed with the files it takes from.
    pub fn model_file(&self) -> Result<crate::model_file::ModelFile<'_>, String> {
        crate::model_file::ModelFile::parse_taking(&self.model_text, self.taken())
    }

    /// The requested page's content type, as the source index has it: the
    /// one its `type` names. `None` for a fragment, and for a page with no
    /// type or one that isn't declared.
    pub fn page_type(&self) -> Option<&ascribe_model::ContentType> {
        self.snapshot.file(&self.path)?.content_type(&self.model)
    }

    /// The name of the requested page's content type ([`Ctx::page_type`]).
    pub fn page_type_name(&self) -> Option<&str> {
        self.page_type().map(|t| t.name.as_str())
    }

    /// The requested file's index.
    pub fn file(&self) -> Option<&FileIndex> {
        self.snapshot.file(&self.path)
    }

    /// The `file:` URI of a content path (a source file or an asset).
    pub fn uri_of(&self, path: &RelPath) -> Option<Uri> {
        path_to_uri(&normalize(&self.content_dir.join(path.as_str())))
    }
}

/// Line indexes of files, built once per request.
pub struct Lines<'a> {
    ctx: &'a Ctx,
    cache: HashMap<RelPath, Arc<LineIndex>>,
}

impl<'a> Lines<'a> {
    /// No line index built yet, for the files of `ctx`'s snapshot.
    pub fn new(ctx: &'a Ctx) -> Lines<'a> {
        Lines {
            ctx,
            cache: HashMap::new(),
        }
    }

    fn index(&mut self, path: &RelPath) -> Option<Arc<LineIndex>> {
        if let Some(index) = self.cache.get(path) {
            return Some(index.clone());
        }
        let index = Arc::new(LineIndex::new(&self.ctx.snapshot.file(path)?.source));
        self.cache.insert(path.clone(), index.clone());
        Some(index)
    }

    /// The range of a span of a source file, in the client's encoding.
    pub fn range(&mut self, path: &RelPath, span: Span) -> Option<Range> {
        let index = self.index(path)?;
        Some(self.ctx.encoding.range(&index, span))
    }

    /// The position of a byte offset of a source file.
    pub fn position(&mut self, path: &RelPath, offset: usize) -> Option<Position> {
        let index = self.index(path)?;
        self.ctx.encoding.position(&index, offset)
    }

    /// Where a file, or a heading in it, is.
    pub fn location(&mut self, path: &RelPath, heading: Option<&Heading>) -> Option<Location> {
        let uri = self.ctx.uri_of(path)?;
        let range = match heading {
            Some(h) => self.range(path, h.span)?,
            None => Range::default(),
        };
        Some(Location { uri, range })
    }
}

/// What is under the cursor.
pub enum Hit<'a> {
    /// A declared or undeclared `{key}`.
    Phrase(&'a PhraseUse),
    /// A link or image, by its position in [`FileIndex::references`].
    Reference(usize, &'a Reference),
    /// The path of an `@include`.
    Include(&'a Include),
    /// The address of a `@snippet`.
    Snippet(&'a ascribe_resolve::SnippetUse),
    /// The primary of an `@available`, as text.
    Availability(&'a str, Span),
    /// The value of a frontmatter `available:` key.
    Frontmatter(String, Span),
    /// A frontmatter `listing:` line, key and value.
    Listing(Span),
    /// The value of a frontmatter `replaces:` key.
    Replaces(Span),
    /// The link of an `@operation` (SPEC §4.13), as written, and its span.
    Operation(&'a str, Span),
    /// A directive's name, or one of its attribute keys.
    Directive(&'a DirectiveLine),
}

/// The smallest thing under `offset`: a phrase inside a link's text beats the
/// link, and a link beats nothing.
pub fn hit_at(file: &FileIndex, offset: usize) -> Option<Hit<'_>> {
    fn touches(span: Span, offset: usize) -> bool {
        span.start() <= offset && offset <= span.end()
    }
    fn offer<'a>(best: &mut Option<(usize, Hit<'a>)>, offset: usize, span: Span, hit: Hit<'a>) {
        if touches(span, offset) && best.as_ref().is_none_or(|(len, _)| span.len() < *len) {
            *best = Some((span.len(), hit));
        }
    }
    let mut best: Option<(usize, Hit<'_>)> = None;
    for phrase in &file.phrases {
        offer(&mut best, offset, phrase.phrase.span, Hit::Phrase(phrase));
    }
    for (i, reference) in file.references.iter().enumerate() {
        offer(
            &mut best,
            offset,
            reference.span,
            Hit::Reference(i, reference),
        );
    }
    for include in &file.includes {
        if let Some(primary) = include.primary {
            offer(&mut best, offset, primary, Hit::Include(include));
        }
    }
    for snippet in &file.snippets {
        if let Some(primary) = snippet.primary {
            offer(&mut best, offset, primary, Hit::Snippet(snippet));
        }
    }
    for marker in &file.availability {
        if let Some((text, span)) = &marker.primary {
            offer(&mut best, offset, *span, Hit::Availability(text, *span));
        }
    }
    if let Some((text, span)) = frontmatter_available(file, offset) {
        offer(&mut best, offset, span, Hit::Frontmatter(text, span));
    }
    if let Some(span) = frontmatter_listing(file, offset) {
        offer(&mut best, offset, span, Hit::Listing(span));
    }
    if let Some(span) = ascribe_resolve::replaces_span(file) {
        offer(&mut best, offset, span, Hit::Replaces(span));
    }
    if let Some((_, hit)) = best {
        return Some(hit);
    }
    let d = directive_at(&file.document.blocks, offset)?;
    if d.name == Builtin::Operation.name()
        && let Some((text, span)) = identifier_primary(d)
        && touches(span, offset)
    {
        return Some(Hit::Operation(text, span));
    }
    let on_key = d
        .attributes
        .as_ref()
        .is_some_and(|b| b.attributes.iter().any(|a| touches(a.key_span, offset)));
    (touches(d.name_span, offset) || on_key).then_some(Hit::Directive(d))
}

/// The top-level frontmatter line with the key `key` when `offset` is on
/// it, and where the frontmatter's content starts.
fn frontmatter_line<'a>(
    file: &'a FileIndex,
    offset: usize,
    key: &str,
) -> Option<(ascribe_syntax::FrontmatterLine<'a>, usize)> {
    let content = file.document.frontmatter.as_ref()?.content;
    if !(content.start() <= offset && offset <= content.end()) {
        return None;
    }
    let source: &str = &file.source;
    let start = line_start(source, offset).max(content.start()) - content.start();
    let line = ascribe_syntax::frontmatter_lines(source.get(content.range())?)
        .into_iter()
        .find(|l| l.start == start && l.indent == 0 && l.item.is_none())
        .filter(|l| l.key == Some(key) && !l.block)?;
    Some((line, content.start()))
}

/// The key of the top-level frontmatter line `offset` is on, with its span,
/// when `offset` is on the key.
pub fn frontmatter_key(file: &FileIndex, offset: usize) -> Option<(&str, Span)> {
    let content = file.document.frontmatter.as_ref()?.content;
    if !(content.start() <= offset && offset <= content.end()) {
        return None;
    }
    let source: &str = &file.source;
    let text = source.get(content.range())?;
    let start = line_start(source, offset).max(content.start()) - content.start();
    let line = ascribe_syntax::frontmatter_lines(text)
        .into_iter()
        .find(|l| l.start == start && l.indent == 0 && l.item.is_none() && !l.block)?;
    let key = line.key?;
    // The key as written, quotes and all, up to its colon.
    let written = text.get(line.start..line.value_start)?;
    let written = written.get(..written.find(':')?)?.trim_end();
    let span = Span::new(
        content.start() + line.start,
        content.start() + line.start + written.len(),
    );
    (span.start() <= offset && offset <= span.end()).then_some((key, span))
}

/// The span of the frontmatter's `listing:` line, its key to the end of its
/// value, when `offset` is on it.
pub fn frontmatter_listing(file: &FileIndex, offset: usize) -> Option<Span> {
    let (line, base) = frontmatter_line(file, offset, Listing::KEY)?;
    let end = base + line.value_start + line.value.len();
    Some(Span::new(base + line.start, end.max(base + line.start)))
}

/// The value of the frontmatter's `available:` key when `offset` is on its
/// line, with the span of the value.
pub fn frontmatter_available(file: &FileIndex, offset: usize) -> Option<(String, Span)> {
    let (line, base) = frontmatter_line(file, offset, "available")?;
    let text = line.value;
    let unquoted = text
        .strip_prefix('"')
        .and_then(|t| t.strip_suffix('"'))
        .or_else(|| text.strip_prefix('\'').and_then(|t| t.strip_suffix('\'')));
    let (text, quote) = match unquoted {
        Some(inner) => (inner, 1),
        None => (text, 0),
    };
    let from = base + line.value_start + quote;
    Some((text.to_owned(), Span::new(from, from + text.len())))
}

/// The byte offset of the start of the line holding `offset`.
pub fn line_start(source: &str, offset: usize) -> usize {
    source[..offset].rfind('\n').map_or(0, |i| i + 1)
}

/// The directive line whose name, attributes, or primary contains `offset`.
pub fn directive_at(blocks: &[Block], offset: usize) -> Option<&DirectiveLine> {
    for block in blocks {
        if !block.span.contains(offset) && block.span.end() != offset {
            continue;
        }
        let found = match &block.kind {
            BlockKind::Directive(d) => Some(d),
            BlockKind::BlockQuote(q) => directive_at(&q.children, offset),
            BlockKind::List(l) => l
                .items
                .iter()
                .find_map(|item| directive_at(&item.children, offset)),
            BlockKind::Container(c) => {
                if c.opener.span.contains(offset) || c.opener.name_span.contains(offset) {
                    Some(&c.opener)
                } else {
                    directive_at(&c.children, offset)
                }
            }
            BlockKind::Group(g) => g.arms.iter().find_map(|arm| {
                if arm.opener.span.contains(offset) {
                    Some(&arm.opener)
                } else {
                    directive_at(&arm.children, offset)
                }
            }),
            _ => None,
        };
        if let Some(d) = found
            && (d.span.contains(offset) || d.span.end() == offset)
        {
            return Some(d);
        }
    }
    None
}

/// The text of a directive's identifier primary, with its span.
pub fn identifier_primary(d: &DirectiveLine) -> Option<(&str, Span)> {
    match &d.primary {
        Some(PrimaryValue::Identifier(p)) => Some((p.text.as_str(), p.span)),
        _ => None,
    }
}

/// The headings a link to `file` can name, in document order
/// ([`Snapshot::link_headings`], without the files they're written in).
pub fn link_headings<'a>(snapshot: &'a Snapshot, file: &'a FileIndex) -> Vec<&'a Heading> {
    snapshot
        .link_headings(file)
        .into_iter()
        .map(|(_, h)| h)
        .collect()
}

pub use ascribe_resolve::{encode_destination, link_path as relative_path, named_headings};
