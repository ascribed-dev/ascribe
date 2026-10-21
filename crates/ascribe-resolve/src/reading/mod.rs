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
//! What's read is the page's title and the field its type marks `role =
//! "description"`, as the build publishes them, then what the build's plain
//! output says: headings, paragraphs, a list item's own text, block quotes'
//! contents, the titles and text the plain output shows of notes, details,
//! and arms, each table row, and each code block, wherever each is written:
//! in the page, in a fragment an include brings in ([`Origin::Fragment`]), or
//! in the code file a `@snippet` shows ([`Origin::Snippet`]). What the
//! content model writes (labels, fallbacks, availability lines) isn't.
//! HTML blocks, and the directive lines the plain output doesn't read as
//! text, are read as written for `ascribe diff`, which takes each block's
//! text from here ([`own_text`]); search never matches them
//! ([`In::searched`]).
//!
//! The matching is in [`Query`], and [`find`] gathers the places a query
//! matches across the readings of many pages and builds. When none does,
//! [`scan`] looks in the source as written.

mod find;
mod query;
mod scan;

use std::ops::Range;
use std::sync::Arc;

use ascribe_core::{FileId, LineIndex, RelPath, Span};
use ascribe_model::PlainContent;
use ascribe_syntax::{
    Block, BlockKind, Bound, CodeBlock, DirectiveLine, Inline, InlineKind, PrimaryValue, Table,
};

pub use find::{At, Found, Place, Shown, Through, find};
pub use query::{BadQuery, Query};
pub use scan::{Nothing, Written, scan};

use crate::build::{LinkTarget, ResolvedBlock, ResolvedKind, ResolvedLink, Substitution};
use crate::index::{FileIndex, PhrasePlace, RefKind, frontmatter_key};
use crate::llms::description_field;
use crate::snippet::Snippet;
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
    /// The page's title, at its key in the frontmatter.
    Title,
    /// The field the page's type marks `role = "description"`, at its key.
    Description,
    /// A heading.
    Heading,
    /// Prose: a paragraph, a list item's own text, a block quote's, a
    /// note's or details' title and text, an arm's title.
    Text,
    /// A table's row.
    Table,
    /// A code block, or the code a `@snippet` shows.
    Code,
    /// A phrase's value, at its line in `ascribe.toml`.
    Phrase,
    /// An HTML block, as written: read for `ascribe diff`, which compares
    /// it, and never matched by search, which reads text, not markup.
    Html,
    /// A directive's title and primary that the plain output doesn't show
    /// as text (a widget's title, `@available`'s spec, `@id`'s id): read
    /// for `ascribe diff`, which compares them, and never matched by search.
    Directive,
}

impl In {
    /// Its name, as the command's JSON writes it.
    pub fn as_str(self) -> &'static str {
        match self {
            In::Title => "title",
            In::Description => "description",
            In::Heading => "heading",
            In::Text => "text",
            In::Table => "table",
            In::Code => "code",
            In::Phrase => "phrase",
            In::Html => "html",
            In::Directive => "directive",
        }
    }

    /// Whether search matches it.
    pub fn searched(self) -> bool {
        !matches!(self, In::Html | In::Directive)
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
    /// The file it's written in: a source file, or for a snippet's code,
    /// its code file.
    pub file: FileId,
    /// Its span there: a table row's for a row, a title line's for a
    /// directive's title, the key's line for a frontmatter field, the lines
    /// shown for a snippet's code.
    pub span: Span,
    /// The heading it's under, or for a heading, the heading itself.
    pub under: Option<Under>,
    /// The arm of a group it's written in, when it's in one: its attributes
    /// as written (`deployment=cloud`), or else its title.
    pub arm: Option<String>,
    /// Where it's written, when that isn't the page's own file.
    pub origin: Origin,
}

/// Where a block of a reading is written.
#[derive(Clone, Debug)]
pub enum Origin {
    /// In the page's own file.
    Page,
    /// In a fragment an include brings in, at any depth.
    Fragment,
    /// In the code file a `@snippet` shows.
    Snippet {
        /// The snippet.
        snippet: Arc<Snippet>,
        /// Whether each line of the code is placed at its own line of the
        /// file. When the code shown can't be lined up with what the file
        /// says, it's one piece, standing for the first line shown.
        exact: bool,
    },
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
        reader.frontmatter();
        reader.blocks(&page.blocks);
        Reading {
            page: page.path.clone(),
            build: page.build.clone(),
            blocks: reader.blocks,
        }
    }
}

/// A resolved block's own text as a reader reads it, without the blocks
/// inside it: what `ascribe diff` compares word by word. A table's rows, and
/// a note's title and text, are joined by a space.
pub fn own_text(project: &Project, page: &ResolvedPage, block: &ResolvedBlock) -> String {
    let mut reader = Reader {
        project,
        page,
        blocks: Vec::new(),
        under: None,
    };
    match &block.kind {
        ResolvedKind::Leaf(leaf) => {
            reader.leaf(block, leaf);
        }
        ResolvedKind::Container { opener, .. } => reader.opener(block, opener),
        _ => {}
    }
    let texts: Vec<&str> = reader.blocks.iter().map(|b| b.text.as_str()).collect();
    texts.join(" ")
}

impl ReadBlock {
    /// The text of the file it's written in.
    pub fn source<'p>(&'p self, project: &'p Project) -> &'p str {
        match &self.origin {
            Origin::Snippet { snippet, .. } => &snippet.text,
            Origin::Page | Origin::Fragment => {
                project.file_by_id(self.file).map_or("", |f| &f.source)
            }
        }
    }

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
    /// The page's title and description, as the build publishes them, each
    /// at its key's line.
    fn frontmatter(&mut self) {
        let Some(index) = self.project.file_by_id(self.page.file) else {
            return;
        };
        if let Some(title) = &self.page.title {
            self.field(index, "title", title, In::Title);
        }
        let frontmatter = self.page.frontmatter.as_ref();
        if let Some(field) = description_field(self.project.model(), frontmatter)
            && let Some(text) = frontmatter
                .and_then(|f| f.get(field.name.as_str()))
                .and_then(|v| v.as_str())
        {
            self.field(index, &field.name, text, In::Description);
        }
    }

    /// A field of the frontmatter whose value reads as `text`: aligned with
    /// the value when it's on the key's line, and else standing for that
    /// line as a whole.
    fn field(&mut self, index: &FileIndex, key: &str, text: &str, kind: In) {
        let Some(line) = frontmatter_key(index, key) else {
            return;
        };
        let written = index.source.get(line.range()).unwrap_or_default();
        let after = written.find(':').map_or(written.len(), |colon| colon + 1);
        let value = &written[after..];
        let start = line.start() + after + (value.len() - value.trim_start().len());
        let value = Span::new(start.min(line.end()), line.end());
        let model = self.project.model();
        let subs: Vec<Substitution> = index
            .phrases
            .iter()
            .filter(|p| p.declared && p.place == PhrasePlace::Frontmatter)
            .filter(|p| value.contains_span(p.phrase.span))
            .filter_map(|p| {
                Some(Substitution {
                    span: p.phrase.span,
                    key: p.phrase.key.clone(),
                    value: model.phrase(&p.phrase.key)?.to_owned(),
                })
            })
            .collect();
        let mut w = Writer::new(&index.source, &subs, &[]);
        let scalar = index
            .source
            .get(value.range())
            .is_none_or(|v| v.is_empty() || v.starts_with(['|', '>']));
        if scalar {
            w.made(text, line);
        } else {
            w.text(value, text);
        }
        self.add(kind, index.file, line, None, Origin::Page, w);
    }

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
                    self.opener(block, opener);
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
                let mut w = self.writer(block);
                w.inlines(&h.inlines);
                self.push(block, In::Heading, block.span, w);
            }
            BlockKind::Paragraph(p) => {
                let mut w = self.writer(block);
                w.inlines(&p.inlines);
                self.push(block, In::Text, block.span, w);
            }
            BlockKind::Table(t) => self.table(block, t),
            BlockKind::CodeBlock(c) => match &block.snippet {
                Some(snippet) => self.snippet(block, c, snippet),
                None => {
                    let mut w = self.writer(block);
                    w.code(block.span, &c.literal);
                    self.push(block, In::Code, block.span, w);
                }
            },
            BlockKind::HtmlBlock(h) => {
                let mut w = self.writer(block);
                w.code(block.span, &h.literal);
                self.push(block, In::Html, block.span, w);
            }
            BlockKind::Directive(line) => match line.name.as_str() {
                "note" => {
                    self.title(block, line);
                    if let Some(PrimaryValue::Text(text)) = &line.primary {
                        let mut w = self.writer(block);
                        w.inlines(&text.inlines);
                        self.push(block, In::Text, text.span, w);
                    }
                }
                "details" => self.title(block, line),
                name => {
                    self.directive(block, line);
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
        if let Some(title) = &line.title {
            let mut w = self.writer(block);
            w.inlines(&title.inlines);
            self.push(block, In::Text, title.span, w);
        }
    }

    /// A container's opening line: a note's or details' title as text, and
    /// any other's title and primary as a directive's.
    fn opener(&mut self, block: &ResolvedBlock, line: &DirectiveLine) {
        if matches!(line.name.as_str(), "note" | "details") {
            self.title(block, line);
        } else {
            self.directive(block, line);
        }
    }

    /// The title and primary of a directive line the plain output doesn't
    /// read, as one block search never matches.
    fn directive(&mut self, block: &ResolvedBlock, line: &DirectiveLine) {
        let mut w = self.writer(block);
        if let Some(title) = &line.title {
            w.inlines(&title.inlines);
        }
        match &line.primary {
            Some(PrimaryValue::Text(text)) => {
                w.made(" ", Span::new(text.span.start(), text.span.start()));
                w.inlines(&text.inlines);
            }
            Some(PrimaryValue::Identifier(id)) => {
                w.made(" ", Span::new(id.span.start(), id.span.start()));
                w.text(id.span, &id.text);
            }
            Some(PrimaryValue::Line(l)) => {
                w.made(" ", Span::new(l.span.start(), l.span.start()));
                w.text(l.span, &l.text);
            }
            Some(PrimaryValue::Unexpected(_)) | None => {}
        }
        self.push(block, In::Directive, line.span, w);
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

    /// The code a `@snippet` shows: each line at the part of its code file
    /// it shows, the lines joined by spaces.
    fn snippet(&mut self, block: &ResolvedBlock, code: &CodeBlock, snippet: &Arc<Snippet>) {
        let literal = code.literal.strip_suffix('\n').unwrap_or(&code.literal);
        if literal.is_empty() {
            return;
        }
        let lines: Vec<&str> = literal.split('\n').collect();
        let exact = lines.len() == snippet.spans.len();
        let first = snippet.spans.first().copied().unwrap_or_default();
        let last = snippet.spans.last().copied().unwrap_or_default();
        let mut w = Writer::new(&snippet.text, &[], &[]);
        let span = if exact {
            for (text, line) in lines.iter().zip(&snippet.spans) {
                w.made(" ", Span::new(line.start(), line.start()));
                w.text(*line, text.trim_end_matches('\r'));
            }
            first.cover(last)
        } else {
            w.made(literal, first);
            first
        };
        let arm = self.arm(block.file, block.span);
        let origin = Origin::Snippet {
            snippet: Arc::clone(snippet),
            exact,
        };
        self.add(In::Code, snippet.file, span, arm, origin, w);
    }

    fn writer<'b>(&self, block: &'b ResolvedBlock) -> Writer<'b>
    where
        'a: 'b,
    {
        let source = self
            .project
            .file_by_id(block.file)
            .map_or("", |f| &*f.source);
        Writer::new(source, &block.substitutions, &block.links)
    }

    /// The arm of a group in `file` that holds `span`.
    fn arm(&self, file: FileId, span: Span) -> Option<String> {
        self.project
            .file_by_id(file)
            .and_then(|f| arm_of(&f.document.blocks, span))
    }

    /// Adds a block of a resolved page, written in its file at `span`.
    fn push(&mut self, block: &ResolvedBlock, kind: In, span: Span, w: Writer<'_>) {
        let origin = if block.via.is_empty() && block.file == self.page.file {
            Origin::Page
        } else {
            Origin::Fragment
        };
        let arm = self.arm(block.file, span);
        self.add(kind, block.file, span, arm, origin, w);
    }

    fn add(
        &mut self,
        kind: In,
        file: FileId,
        span: Span,
        arm: Option<String>,
        origin: Origin,
        w: Writer<'_>,
    ) {
        let (text, pieces) = w.finish();
        if text.is_empty() {
            return;
        }
        self.blocks.push(ReadBlock {
            kind,
            text,
            pieces,
            file,
            span,
            under: self.under.clone(),
            arm,
            origin,
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

impl<'a> Writer<'a> {
    fn new(source: &'a str, subs: &'a [Substitution], links: &'a [ResolvedLink]) -> Writer<'a> {
        Writer {
            source,
            subs,
            links,
            text: String::new(),
            pieces: Vec::new(),
            pending: None,
        }
    }

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

    /// A code block's code, or an HTML block's: each line at its line in
    /// the source, the lines joined by spaces.
    fn code(&mut self, span: Span, literal: &str) {
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
        let literal = literal.strip_suffix('\n').unwrap_or(literal);
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
