//! A resolved page as its readers read it, block by block, with a map from
//! each range of its text back to the source: what `ascribe search` matches.
//!
//! A [`Reading`] is a page's blocks as a build publishes them: markup
//! removed (a link is its text, an image its alt text, emphasis and code
//! spans their contents), phrases replaced by their values, and white space
//! collapsed to single spaces. Each block's text is in [`Piece`]s, in order
//! and touching, each standing for a span of the source: text copied from
//! it, text made for it (a line break's space, a code span's contents), a
//! phrase's value, or a title a link with no text borrows from its target.
//! [`ReadBlock::locate`] turns a range of the text into the place to edit.
//!
//! What's read is what the build's plain output says of the page's own file:
//! headings, paragraphs, a list item's own text, block quotes' contents, the
//! titles and text the plain output shows of notes, details, and arms, each
//! table row, and each code block written in the page. Blocks that came
//! through an include, code a `@snippet` became, HTML blocks, and what the
//! content model writes (labels, fallbacks, availability lines) aren't.
//!
//! The matching is in [`Query`], and [`find`] gathers the places a query
//! matches across the readings of many pages and builds.

mod find;
mod query;

use std::ops::Range;

use ascribe_core::{FileId, LineIndex, RelPath, Span};
use ascribe_model::PlainContent;
use ascribe_syntax::{
    Block, BlockKind, Bound, CodeBlock, DirectiveLine, Inline, InlineKind, PrimaryValue, Table,
};

pub use find::{At, Found, Place, Through, find};
pub use query::{BadQuery, Query};

use crate::build::{LinkTarget, ResolvedBlock, ResolvedKind, ResolvedLink, Substitution};
use crate::index::RefKind;
use crate::{Project, ResolvedPage};

/// A page as a build's readers read it.
#[derive(Clone, Debug)]
pub struct Reading {
    /// The page's content path.
    pub page: RelPath,
    /// The build's name.
    pub build: String,
    /// Its blocks, in document order.
    pub blocks: Vec<ReadBlock>,
}

/// What a block, or a place found in one, is.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum In {
    /// A heading.
    Heading,
    /// Prose: a paragraph, a list item's own text, a block quote's, a
    /// note's or details' title and text, an arm's title.
    Text,
    /// A table's row.
    Table,
    /// A code block written in the page.
    Code,
    /// A phrase's value, at its line in `ascribe.toml`.
    Phrase,
}

impl In {
    /// Its name, as the command's JSON writes it.
    pub fn as_str(self) -> &'static str {
        match self {
            In::Heading => "heading",
            In::Text => "text",
            In::Table => "table",
            In::Code => "code",
            In::Phrase => "phrase",
        }
    }
}

/// A block of a reading.
#[derive(Clone, Debug)]
pub struct ReadBlock {
    /// What it is.
    pub kind: In,
    /// Its text as a reader reads it: no markup, white space collapsed to
    /// single spaces, none at either end.
    pub text: String,
    /// The pieces of `text`, in order and touching: each byte of it is in
    /// exactly one.
    pub pieces: Vec<Piece>,
    /// The file it's written in.
    pub file: FileId,
    /// Its span there: a table row's for a row, a title line's for a title.
    pub span: Span,
    /// The heading it's under, or for a heading, the heading itself.
    pub under: Option<Under>,
    /// The arm of a group it's written in, when it's in one: its attributes
    /// as written (`deployment=cloud`), or else its title.
    pub arm: Option<String>,
}

/// The heading a block is under.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Under {
    /// Its text as a reader reads it.
    pub text: String,
    /// Its id on the page in the build.
    pub id: String,
}

/// A range of a block's text, and the source it stands for.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Piece {
    /// The range of [`ReadBlock::text`], in bytes.
    pub range: Range<usize>,
    /// The source it stands for, in the block's file.
    pub source: Span,
    /// How.
    pub kind: PieceKind,
}

/// How a piece of a reading stands for its source.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PieceKind {
    /// The source text itself, byte for byte.
    Copied,
    /// Text that stands for the source span as a whole: a line break's
    /// space, a decoded escape, inline HTML's text.
    Made,
    /// A phrase's value, standing for its `{key}`.
    Phrase(String),
    /// A link with no text, read as the title of the page it links to, or
    /// the text of the heading it names: standing for the link.
    Borrowed {
        /// The page linked to.
        page: RelPath,
        /// The heading's id there, when the link names one.
        id: Option<String>,
    },
}

impl Reading {
    /// The reading of a page a build resolved.
    pub fn of(project: &Project, page: &ResolvedPage) -> Reading {
        let mut reader = Reader {
            project,
            page,
            blocks: Vec::new(),
            under: None,
        };
        reader.blocks(&page.blocks);
        Reading {
            page: page.path.clone(),
            build: page.build.clone(),
            blocks: reader.blocks,
        }
    }
}

impl ReadBlock {
    /// The piece the byte at `offset` of the text is in.
    pub fn piece_at(&self, offset: usize) -> Option<&Piece> {
        let i = self.pieces.partition_point(|p| p.range.end <= offset);
        self.pieces.get(i)
    }

    /// The pieces any of `range` is in, in order.
    pub fn pieces_in(&self, range: Range<usize>) -> impl Iterator<Item = &Piece> {
        let end = range.end.max(range.start + 1);
        self.pieces
            .iter()
            .filter(move |p| p.range.start < end && p.range.end > range.start)
    }

    /// The offset in the block's file where the text's byte at `offset` is
    /// written: itself, for copied text, and the start of what it stands for
    /// otherwise.
    pub fn source_offset(&self, offset: usize) -> usize {
        match self.piece_at(offset) {
            Some(p) if p.kind == PieceKind::Copied => p.source.start() + (offset - p.range.start),
            Some(p) => p.source.start(),
            None => self.span.start(),
        }
    }
}

/// Builds a page's reading.
struct Reader<'a> {
    project: &'a Project,
    page: &'a ResolvedPage,
    blocks: Vec<ReadBlock>,
    under: Option<Under>,
}

impl<'a> Reader<'a> {
    fn blocks(&mut self, blocks: &[ResolvedBlock]) {
        let mut skip_next = false;
        for block in blocks {
            if std::mem::take(&mut skip_next) {
                continue;
            }
            match &block.kind {
                ResolvedKind::Leaf(leaf) => skip_next = self.leaf(block, leaf),
                ResolvedKind::BlockQuote { children } => self.blocks(children),
                ResolvedKind::List { items, .. } => {
                    for item in items {
                        self.blocks(&item.children);
                    }
                }
                ResolvedKind::Container {
                    opener, children, ..
                } => {
                    if matches!(opener.name.as_str(), "note" | "details") {
                        self.title(block, opener);
                    }
                    if !self.dropped_by_plain(&opener.name) {
                        self.blocks(children);
                    }
                }
                ResolvedKind::Group { name, arms, .. } => {
                    if self.dropped_by_plain(name) {
                        continue;
                    }
                    // A variant group the build keeps several arms of names
                    // each arm by its values, and by its title only when it
                    // has none; any other group shows each arm's title.
                    let switcher = name == "variant" && arms.len() > 1;
                    for arm in arms {
                        let has_values = arm
                            .opener
                            .attributes
                            .as_ref()
                            .is_some_and(|a| !a.attributes.is_empty());
                        if !(switcher && has_values) {
                            self.title(block, &arm.opener);
                        }
                        self.blocks(&arm.children);
                    }
                }
            }
        }
    }

    /// Whether the plain output leaves out what a project widget wraps.
    fn dropped_by_plain(&self, name: &str) -> bool {
        self.project
            .model()
            .widget(name)
            .is_some_and(|w| w.plain_content == PlainContent::Drop)
    }

    /// Reads a leaf; returns whether it's a directive that has the plain
    /// output leave out the block after it.
    fn leaf(&mut self, block: &ResolvedBlock, leaf: &Block) -> bool {
        match &leaf.kind {
            BlockKind::Heading(h) => {
                if let Some(ids) = &block.heading {
                    self.under = Some(Under {
                        text: collapse(&ids.text),
                        id: ids.page_id.clone(),
                    });
                }
                if self.own(block) {
                    let mut w = self.writer(block);
                    w.inlines(&h.inlines);
                    self.push(block, In::Heading, block.span, w);
                }
            }
            BlockKind::Paragraph(p) if self.own(block) => {
                let mut w = self.writer(block);
                w.inlines(&p.inlines);
                self.push(block, In::Text, block.span, w);
            }
            BlockKind::Table(t) if self.own(block) => self.table(block, t),
            BlockKind::CodeBlock(c) if self.own(block) && block.snippet.is_none() => {
                let mut w = self.writer(block);
                w.code(block.span, c);
                self.push(block, In::Code, block.span, w);
            }
            BlockKind::Directive(line) => match line.name.as_str() {
                "note" => {
                    self.title(block, line);
                    if let Some(PrimaryValue::Text(text)) = &line.primary
                        && self.own(block)
                    {
                        let mut w = self.writer(block);
                        w.inlines(&text.inlines);
                        self.push(block, In::Text, text.span, w);
                    }
                }
                "details" => self.title(block, line),
                name => {
                    return line.binding == Some(Bound::FollowingBlock)
                        && self.dropped_by_plain(name);
                }
            },
            _ => {}
        }
        false
    }

    /// A directive's title line, as a block of text.
    fn title(&mut self, block: &ResolvedBlock, line: &DirectiveLine) {
        if let Some(title) = &line.title
            && self.own(block)
        {
            let mut w = self.writer(block);
            w.inlines(&title.inlines);
            self.push(block, In::Text, title.span, w);
        }
    }

    /// Each row of a table, its cells' text joined by spaces.
    fn table(&mut self, block: &ResolvedBlock, table: &Table) {
        for row in &table.rows {
            let mut w = self.writer(block);
            for cell in &row.cells {
                w.made(" ", Span::new(cell.span.start(), cell.span.start()));
                w.inlines(&cell.inlines);
            }
            self.push(block, In::Table, row.span, w);
        }
    }

    /// Whether a block is written in the page's own file.
    fn own(&self, block: &ResolvedBlock) -> bool {
        block.via.is_empty() && block.file == self.page.file
    }

    fn writer<'b>(&self, block: &'b ResolvedBlock) -> Writer<'b>
    where
        'a: 'b,
    {
        let source = self
            .project
            .file_by_id(block.file)
            .map_or("", |f| &*f.source);
        Writer {
            source,
            subs: &block.substitutions,
            links: &block.links,
            text: String::new(),
            pieces: Vec::new(),
            pending: None,
        }
    }

    fn push(&mut self, block: &ResolvedBlock, kind: In, span: Span, w: Writer<'_>) {
        let (text, pieces) = w.finish();
        if text.is_empty() {
            return;
        }
        let arm = self
            .project
            .file_by_id(block.file)
            .and_then(|f| arm_of(&f.document.blocks, span));
        self.blocks.push(ReadBlock {
            kind,
            text,
            pieces,
            file: block.file,
            span,
            under: self.under.clone(),
            arm,
        });
    }
}

/// Writes a block's text and its pieces.
struct Writer<'a> {
    source: &'a str,
    subs: &'a [Substitution],
    links: &'a [ResolvedLink],
    text: String,
    pieces: Vec<Piece>,
    /// White space waiting to be written as one space, and the source it
    /// stands for, written only once text follows it.
    pending: Option<Span>,
}

impl Writer<'_> {
    fn finish(self) -> (String, Vec<Piece>) {
        (self.text, self.pieces)
    }

    fn piece(&mut self, text: &str, source: Span, kind: PieceKind) {
        let start = self.text.len();
        self.text.push_str(text);
        self.pieces.push(Piece {
            range: start..self.text.len(),
            source,
            kind,
        });
    }

    fn space(&mut self, source: Span) {
        self.pending = Some(self.pending.map_or(source, |p| p.cover(source)));
    }

    /// Writes the space white space before text leaves, if any is waiting
    /// and text came before it.
    fn flush(&mut self) {
        if let Some(source) = self.pending.take()
            && !self.text.is_empty()
        {
            self.piece(" ", source, PieceKind::Made);
        }
    }

    /// Copies `text`, the source's at `at`: each run of white space as one
    /// space, made, and the rest copied.
    fn copied(&mut self, text: &str, at: usize) {
        let mut start = 0;
        while start < text.len() {
            let space = text[start..].starts_with(char::is_whitespace);
            let end = text[start..]
                .find(|c: char| c.is_whitespace() != space)
                .map_or(text.len(), |i| start + i);
            let source = Span::new(at + start, at + end);
            if space {
                self.space(source);
            } else {
                self.flush();
                self.piece(&text[start..end], source, PieceKind::Copied);
            }
            start = end;
        }
    }

    /// Writes text that stands for `source` as a whole, its white space
    /// collapsed.
    fn other(&mut self, text: &str, source: Span, kind: PieceKind) {
        if text.is_empty() {
            return;
        }
        let collapsed = collapse(text);
        if collapsed.is_empty() {
            self.space(source);
            return;
        }
        if text.starts_with(char::is_whitespace) {
            self.space(Span::new(source.start(), source.start()));
        }
        self.flush();
        self.piece(&collapsed, source, kind);
        if text.ends_with(char::is_whitespace) {
            self.space(Span::new(source.end(), source.end()));
        }
    }

    fn made(&mut self, text: &str, source: Span) {
        self.other(text, source, PieceKind::Made);
    }

    /// Text that the source at `span` reads as, with no phrase in it: the
    /// part both start with and the part both end with copied, and what's
    /// between (an escape, an entity, a container's indentation) made.
    fn aligned(&mut self, span: Span, text: &str) {
        let Some(source) = self.source.get(span.range()) else {
            self.made(text, span);
            return;
        };
        let prefix: usize = source
            .chars()
            .zip(text.chars())
            .take_while(|(a, b)| a == b)
            .map(|(a, _)| a.len_utf8())
            .sum();
        let (source_rest, text_rest) = (&source[prefix..], &text[prefix..]);
        let suffix: usize = source_rest
            .chars()
            .rev()
            .zip(text_rest.chars().rev())
            .take_while(|(a, b)| a == b)
            .map(|(a, _)| a.len_utf8())
            .sum();
        let at = span.start();
        self.copied(&text[..prefix], at);
        let middle = &text_rest[..text_rest.len() - suffix];
        let middle_source = Span::new(at + prefix, span.end() - suffix);
        self.made(middle, middle_source);
        self.copied(&text_rest[text_rest.len() - suffix..], span.end() - suffix);
    }

    /// Text that the source at `span` reads as: the phrases substituted in
    /// it are their values, and the rest is aligned with the source.
    fn text(&mut self, span: Span, text: &str) {
        let subs = self.subs;
        let mut at = span.start();
        let mut rest = text;
        for sub in subs.iter().filter(|s| span.contains_span(s.span)) {
            // The value is where the text before it is as long as its
            // source, unless an escape made it shorter; then the first place
            // it's found.
            let written = sub.span.start().saturating_sub(at);
            let found = match rest.get(written..) {
                Some(after) if after.starts_with(sub.value.as_str()) => Some(written),
                _ => rest.find(sub.value.as_str()),
            };
            let Some(found) = found else {
                break;
            };
            self.aligned(Span::new(at, sub.span.start()), &rest[..found]);
            self.other(&sub.value, sub.span, PieceKind::Phrase(sub.key.clone()));
            rest = &rest[found + sub.value.len()..];
            at = sub.span.end();
        }
        self.aligned(Span::new(at.min(span.end()), span.end()), rest);
    }

    fn inlines(&mut self, list: &[Inline]) {
        for inline in list {
            let span = inline.span;
            match &inline.kind {
                InlineKind::Text(text) => self.text(span, text),
                InlineKind::Code(code) => self.code_span(span, code),
                InlineKind::SoftBreak | InlineKind::HardBreak => self.made(" ", span),
                InlineKind::Html(html) if is_br(html) => self.made(" ", span),
                InlineKind::Html(html) => self.made(&html_text(html), span),
                InlineKind::Emphasis(children) | InlineKind::Strong(children) => {
                    self.inlines(children);
                }
                InlineKind::Link(link) => match self.borrowed(span) {
                    Some(kind) => self.other(&plain(&link.children), span, kind),
                    None => self.inlines(&link.children),
                },
                InlineKind::Image(image) => self.inlines(&image.children),
                InlineKind::Phrase(p) => self.aligned(span, &format!("{{{}}}", p.key)),
            }
        }
    }

    /// For a link that took its target's title or heading as its text,
    /// what it borrows from.
    fn borrowed(&self, span: Span) -> Option<PieceKind> {
        self.links
            .iter()
            .find(|l| l.span == span && l.kind == RefKind::Link)
            .and_then(|l| match &l.target {
                LinkTarget::Page {
                    page,
                    id,
                    text_filled: true,
                    ..
                } => Some(PieceKind::Borrowed {
                    page: page.clone(),
                    id: id.clone(),
                }),
                _ => None,
            })
    }

    /// A code span: its contents, between its backticks.
    fn code_span(&mut self, span: Span, code: &str) {
        let source = self.source.get(span.range()).unwrap_or_default();
        let open = source.len() - source.trim_start_matches('`').len();
        let close = source.len() - source.trim_end_matches('`').len();
        if open + close >= source.len() {
            self.made(code, span);
            return;
        }
        let inner = Span::new(span.start() + open, span.end() - close);
        let saved = std::mem::take(&mut self.subs);
        self.aligned(inner, code);
        self.subs = saved;
    }

    /// A code block: each line of its code at its line in the source, the
    /// lines joined by spaces.
    fn code(&mut self, span: Span, code: &CodeBlock) {
        let source = self.source.get(span.range()).unwrap_or_default();
        let mut lines: Vec<Span> = Vec::new();
        let mut at = span.start();
        for line in source.split_inclusive('\n') {
            let body = line.trim_end_matches(['\n', '\r']);
            lines.push(Span::new(at, at + body.len()));
            at += line.len();
        }
        let first = lines
            .first()
            .and_then(|l| self.source.get(l.range()))
            .map(str::trim_start)
            .is_some_and(|l| l.starts_with("```") || l.starts_with("~~~"));
        let lines = &lines[usize::from(first).min(lines.len())..];
        let literal = code.literal.strip_suffix('\n').unwrap_or(&code.literal);
        if literal.is_empty() {
            return;
        }
        for (n, text) in literal.split('\n').enumerate() {
            let text = text.trim_end_matches('\r');
            match lines.get(n) {
                Some(line) => {
                    self.made(" ", Span::new(line.start(), line.start()));
                    self.text(*line, text);
                }
                None => self.made(text, Span::new(span.end(), span.end())),
            }
        }
    }
}

/// The innermost arm of a group in `blocks` that holds `span`, named by its
/// attributes as written, or else by its title.
fn arm_of(blocks: &[Block], span: Span) -> Option<String> {
    for block in blocks {
        if !block.span.contains_span(span) {
            continue;
        }
        let inner = match &block.kind {
            BlockKind::BlockQuote(q) => arm_of(&q.children, span),
            BlockKind::List(list) => list
                .items
                .iter()
                .find_map(|item| arm_of(&item.children, span)),
            BlockKind::Container(c) => arm_of(&c.children, span),
            BlockKind::Group(g) => g
                .arms
                .iter()
                .find(|arm| arm.span.contains_span(span))
                .and_then(|arm| arm_of(&arm.children, span).or_else(|| arm_name(&arm.opener))),
            _ => None,
        };
        if inner.is_some() {
            return inner;
        }
    }
    None
}

/// An arm's name: `key=value` for each attribute, or else its title.
fn arm_name(opener: &DirectiveLine) -> Option<String> {
    let attributes: Vec<String> = opener
        .attributes
        .iter()
        .flat_map(|b| &b.attributes)
        .map(|a| match &a.value {
            Some(value) => format!("{}={}", a.key, value.members().join(",")),
            None => a.key.clone(),
        })
        .collect();
    if !attributes.is_empty() {
        return Some(attributes.join(" "));
    }
    opener
        .title
        .as_ref()
        .map(|t| plain(&t.inlines))
        .filter(|t| !t.is_empty())
}

/// Inline content as plain text, white space collapsed.
fn plain(list: &[Inline]) -> String {
    fn push(list: &[Inline], out: &mut String) {
        for node in list {
            match &node.kind {
                InlineKind::Text(v) | InlineKind::Code(v) => out.push_str(v),
                InlineKind::Html(v) => out.push_str(&html_text(v)),
                InlineKind::SoftBreak | InlineKind::HardBreak => out.push(' '),
                InlineKind::Emphasis(c) | InlineKind::Strong(c) => push(c, out),
                InlineKind::Link(link) => push(&link.children, out),
                InlineKind::Image(image) => push(&image.children, out),
                InlineKind::Phrase(p) => out.push_str(&format!("{{{}}}", p.key)),
            }
        }
    }
    let mut out = String::new();
    push(list, &mut out);
    collapse(&out)
}

/// Runs of white space as one space, and none at either end.
fn collapse(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Whether a piece of inline HTML is a `<br>` tag.
fn is_br(html: &str) -> bool {
    let lower = html.trim().to_ascii_lowercase();
    lower
        .strip_prefix("<br")
        .and_then(|rest| rest.strip_suffix('>'))
        .is_some_and(|rest| rest.trim_end_matches('/').trim().is_empty())
}

/// The text of raw HTML, as a reader reads it: tags, comments, and
/// `<script>` and `<style>` contents removed. The plain output writes raw
/// HTML in the source as this.
pub fn html_text(html: &str) -> String {
    let mut out = String::with_capacity(html.len());
    let mut rest = html;
    while let Some(open) = rest.find('<') {
        out.push_str(&rest[..open]);
        let tag = &rest[open..];
        if let Some(after) = tag.strip_prefix("<!--") {
            rest = after.find("-->").map_or("", |end| &after[end + 3..]);
            continue;
        }
        // A tag starts with `<` and a letter, `/`, `!`, or `?`; any other
        // `<` is text, as is one with no `>` after it.
        let starts_tag = tag[1..]
            .chars()
            .next()
            .is_some_and(|c| c.is_ascii_alphabetic() || matches!(c, '/' | '!' | '?'));
        let close = if starts_tag { tag.find('>') } else { None };
        let Some(close) = close else {
            out.push('<');
            rest = &tag[1..];
            continue;
        };
        let name: String = tag[1..close]
            .chars()
            .take_while(|c| c.is_ascii_alphanumeric())
            .collect::<String>()
            .to_ascii_lowercase();
        rest = &tag[close + 1..];
        if name == "script" || name == "style" {
            let end = format!("</{name}");
            let lower = rest.to_ascii_lowercase();
            rest = match lower.find(&end) {
                Some(at) => rest[at..].find('>').map_or("", |gt| &rest[at + gt + 1..]),
                None => "",
            };
        }
    }
    out.push_str(rest);
    out
}

/// The line, from 1, and the column, from 1 in Unicode characters, of a
/// byte offset of `source`.
pub(crate) fn line_col(index: &LineIndex, offset: usize) -> (u32, u32) {
    index
        .wide_line_col(ascribe_core::WideEncoding::Utf32, offset)
        .map_or((1, 1), |p| (p.line + 1, p.col + 1))
}
