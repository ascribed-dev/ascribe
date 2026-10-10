//! What the navigation requests share: the state one request works from, and
//! the helpers that read a file's syntax and index at a cursor.
//!
//! A request takes a [`Ctx`] under the server's lock (a snapshot, the model,
//! the negotiated position encoding) and computes without it, so every answer
//! comes from the project as it was when the request was handled, never from a
//! stale snapshot.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use ascribe_core::path::normalize;
use ascribe_core::{LineIndex, RelPath, Span};
use ascribe_emit::labels::plain_text;
use ascribe_model::ContentModel;
use ascribe_resolve::{FileIndex, Heading, Include, PhraseUse, Reference, Snapshot};
use ascribe_syntax::{Block, BlockKind, DirectiveLine, PrimaryValue};
use lsp_types::{Location, Position, Range, Uri};

use crate::fsx::LayerFs;
use crate::position::Encoding;
use crate::uri::path_to_uri;

/// What a navigation request works from.
pub(crate) struct Ctx {
    /// The project as of the request.
    pub snapshot: Snapshot,
    /// The requested file's content path, or [`Ctx::PROJECT`].
    pub path: RelPath,
    /// The requested document's version, when it's open in the editor.
    pub version: Option<i32>,
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
    /// The day the checks run on.
    pub today: Option<ascribe_core::Date>,
}

impl Ctx {
    /// The path of a request about the whole project asked through a file
    /// that isn't a source ([`crate::core::Core::project_target`]): a name at
    /// the content root that no source file can have.
    pub(crate) const PROJECT: &'static str = "ascribe.toml";

    /// The requested file's index.
    pub(crate) fn file(&self) -> Option<&FileIndex> {
        self.snapshot.file(&self.path)
    }

    /// The `file:` URI of a content path (a source file or an asset).
    pub(crate) fn uri_of(&self, path: &RelPath) -> Option<Uri> {
        path_to_uri(&normalize(&self.content_dir.join(path.as_str())))
    }
}

/// Line indexes of files, built once per request.
pub(crate) struct Lines<'a> {
    ctx: &'a Ctx,
    cache: HashMap<RelPath, Arc<LineIndex>>,
}

impl<'a> Lines<'a> {
    pub(crate) fn new(ctx: &'a Ctx) -> Lines<'a> {
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
    pub(crate) fn range(&mut self, path: &RelPath, span: Span) -> Option<Range> {
        let index = self.index(path)?;
        Some(self.ctx.encoding.range(&index, span))
    }

    /// The position of a byte offset of a source file.
    pub(crate) fn position(&mut self, path: &RelPath, offset: usize) -> Option<Position> {
        let index = self.index(path)?;
        self.ctx.encoding.position(&index, offset)
    }

    /// Where a file, or a heading in it, is.
    pub(crate) fn location(
        &mut self,
        path: &RelPath,
        heading: Option<&Heading>,
    ) -> Option<Location> {
        let uri = self.ctx.uri_of(path)?;
        let range = match heading {
            Some(h) => self.range(path, h.span)?,
            None => Range::default(),
        };
        Some(Location { uri, range })
    }
}

/// What is under the cursor.
pub(crate) enum Hit<'a> {
    /// A declared or undeclared `{key}`.
    Phrase(&'a PhraseUse),
    /// A link or image, by its position in [`FileIndex::references`].
    Reference(usize, &'a Reference),
    /// The path of an `@include`.
    Include(&'a Include),
    /// The primary of an `@available`, as text.
    Availability(&'a str, Span),
    /// The value of a frontmatter `available:` key.
    Frontmatter(String, Span),
    /// A directive's name, or one of its attribute keys.
    Directive(&'a DirectiveLine),
}

/// The smallest thing under `offset`: a phrase inside a link's text beats the
/// link, and a link beats nothing.
pub(crate) fn hit_at(file: &FileIndex, offset: usize) -> Option<Hit<'_>> {
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
    for marker in &file.availability {
        if let Some((text, span)) = &marker.primary {
            offer(&mut best, offset, *span, Hit::Availability(text, *span));
        }
    }
    if let Some((text, span)) = frontmatter_available(file, offset) {
        offer(&mut best, offset, span, Hit::Frontmatter(text, span));
    }
    if let Some((_, hit)) = best {
        return Some(hit);
    }
    let d = directive_at(&file.document.blocks, offset)?;
    let on_key = d
        .attributes
        .as_ref()
        .is_some_and(|b| b.attributes.iter().any(|a| touches(a.key_span, offset)));
    (touches(d.name_span, offset) || on_key).then_some(Hit::Directive(d))
}

/// The value of the frontmatter's `available:` key when `offset` is on its
/// line, with the span of the value.
pub(crate) fn frontmatter_available(file: &FileIndex, offset: usize) -> Option<(String, Span)> {
    let content = file.document.frontmatter.as_ref()?.content;
    if !(content.start() <= offset && offset <= content.end()) {
        return None;
    }
    let source: &str = &file.source;
    let start = line_start(source, offset).max(content.start()) - content.start();
    let line = ascribe_syntax::frontmatter_lines(source.get(content.range())?)
        .into_iter()
        .find(|l| l.start == start && l.indent == 0 && l.item.is_none())
        .filter(|l| l.key == Some("available") && !l.block)?;
    let text = line.value;
    let unquoted = text
        .strip_prefix('"')
        .and_then(|t| t.strip_suffix('"'))
        .or_else(|| text.strip_prefix('\'').and_then(|t| t.strip_suffix('\'')));
    let (text, quote) = match unquoted {
        Some(inner) => (inner, 1),
        None => (text, 0),
    };
    let from = content.start() + line.value_start + quote;
    Some((text.to_owned(), Span::new(from, from + text.len())))
}

/// The byte offset of the start of the line holding `offset`.
pub(crate) fn line_start(source: &str, offset: usize) -> usize {
    source[..offset].rfind('\n').map_or(0, |i| i + 1)
}

/// The text of the line up to the cursor.
pub(crate) fn line_prefix(source: &str, offset: usize) -> &str {
    &source[line_start(source, offset)..offset]
}

/// Replaces each declared `{key}` in `text` with its value.
pub(crate) fn substitute_phrases(model: &ContentModel, text: &str) -> String {
    if !text.contains('{') {
        return text.to_owned();
    }
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(open) = rest.find('{') {
        out.push_str(&rest[..open]);
        let after = &rest[open + 1..];
        match after.find('}') {
            Some(close) if model.has_phrase(&after[..close]) => {
                out.push_str(model.phrase(&after[..close]).unwrap_or_default());
                rest = &after[close + 1..];
            }
            _ => {
                out.push('{');
                rest = after;
            }
        }
    }
    out.push_str(rest);
    out
}

/// Cuts `text` to at most `max` characters, at a character boundary.
pub(crate) fn truncate(text: &str, max: usize) -> String {
    match text.char_indices().nth(max) {
        Some((at, _)) => format!("{}…", text[..at].trim_end()),
        None => text.to_owned(),
    }
}

/// The first paragraph of a file, or of a heading's section, as plain text with
/// declared phrases replaced by their values: the preview a link's hover
/// shows.
pub(crate) fn first_paragraph(
    file: &FileIndex,
    model: &ContentModel,
    section: Option<Span>,
) -> Option<String> {
    let found = find_paragraph(&file.document.blocks, section)?;
    let text = substitute_phrases(model, &plain_text(found));
    let text = text.split_whitespace().collect::<Vec<_>>().join(" ");
    (!text.is_empty()).then(|| truncate(&text, 280))
}

fn find_paragraph(blocks: &[Block], section: Option<Span>) -> Option<&[ascribe_syntax::Inline]> {
    for block in blocks {
        let inside = section.is_none_or(|s| s.contains_span(block.span));
        match &block.kind {
            BlockKind::Paragraph(p) if inside => return Some(&p.inlines),
            BlockKind::BlockQuote(q) => {
                if let Some(found) = find_paragraph(&q.children, section) {
                    return Some(found);
                }
            }
            BlockKind::List(l) => {
                for item in &l.items {
                    if let Some(found) = find_paragraph(&item.children, section) {
                        return Some(found);
                    }
                }
            }
            BlockKind::Container(c) => {
                if let Some(found) = find_paragraph(&c.children, section) {
                    return Some(found);
                }
            }
            BlockKind::Group(g) => {
                for arm in &g.arms {
                    if let Some(found) = find_paragraph(&arm.children, section) {
                        return Some(found);
                    }
                }
            }
            _ => {}
        }
    }
    None
}

/// The directive line whose name, attributes, or primary contains `offset`.
pub(crate) fn directive_at(blocks: &[Block], offset: usize) -> Option<&DirectiveLine> {
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
pub(crate) fn identifier_primary(d: &DirectiveLine) -> Option<(&str, Span)> {
    match &d.primary {
        Some(PrimaryValue::Identifier(p)) => Some((p.text.as_str(), p.span)),
        _ => None,
    }
}

/// The headings a link to `file` can name, in document order
/// ([`Snapshot::link_headings`], without the files they're written in).
pub(crate) fn link_headings<'a>(snapshot: &'a Snapshot, file: &'a FileIndex) -> Vec<&'a Heading> {
    snapshot
        .link_headings(file)
        .into_iter()
        .map(|(_, h)| h)
        .collect()
}

pub(crate) use ascribe_resolve::{encode_destination, link_path as relative_path, named_headings};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn truncation_is_on_a_character_boundary() {
        assert_eq!(truncate("héllo wörld", 5), "héllo…");
        assert_eq!(truncate("short", 20), "short");
    }
}
