//! The Ascribe formatter, which rewrites Ascribe constructs into canonical
//! form (SPEC §8.3) and returns minimal text edits.
//!
//! [`format`] never re-renders the author's markdown. It works from spans in
//! the tree `ascribe_syntax::parse` returns, and edits only bytes that belong
//! to an Ascribe construct:
//!
//! - a directive line's head: the space before its attribute block, the
//!   attribute block itself, the `:` and the space after it, and whitespace
//!   after a container's `:`;
//! - the attribute block after an image;
//! - the indentation of directive lines and end lines;
//! - the blank lines between a following-block directive and its block.
//!
//! Prose, tables, code, and every other line stay byte for byte as written.
//! Link reference definitions aren't Ascribe constructs, so the formatter
//! never changes one: it never deletes a line that holds a definition.
//!
//! Each rule of SPEC §8.3 is its own function, in `head` (name, colon,
//! primary), `attributes` (spacing, order, quoting, empty blocks), `indent`,
//! and `blank`.
//!
//! # Constructs with errors are left alone
//!
//! A directive line, end line, or image attribute block that the parser or
//! the structure pass reported an error on is not touched, because the
//! formatter can't know what the author meant. Warnings don't stop it
//! (`binding-blank-line` is the one it fixes). The check passes that need the
//! content model (unknown attribute keys) aren't run here; a block with a key
//! the schema doesn't declare is normalized but its order is kept.
//!
//! [`format_files`] formats the files of a project on disk, as `ascribe
//! fmt` does.
//!
//! # Guarantees
//!
//! Formatting is idempotent, and the formatted file parses to the same
//! outline (the same blocks, directives, attribute values, and text). The
//! tests in this crate check both over every conformance input.
//!
//! ```
//! use ascribe_fmt::format_source;
//! use ascribe_model::load_str;
//! use ascribe_core::FileId;
//!
//! let model = load_str("spec = \"0.1\"\n", FileId::new(0)).expect("a valid model");
//! let options = ascribe_fmt::options_from_model(&model);
//! let out = format_source("@note{ type = tip }:Careful.\n", &options, &model);
//! assert_eq!(out, "@note {type=tip}: Careful.\n");
//! ```

mod attributes;
mod blank;
mod files;
mod head;
mod indent;
mod skip;

use ascribe_core::{Span, TextEdit, apply_edits};
use ascribe_model::ContentModel;
use ascribe_syntax::{
    Block, BlockKind, DirectiveLine, EndLine, Inline, InlineKind, ParseOptions, ParsedDocument,
    PrimaryValue, parse,
};

pub use attributes::{directive_block, image_block, written_value};
pub use files::{FormatFilesError, Formatted, Refused, format_files};

use indent::Owner;
pub use skip::NON_BLOCKING;

/// The parse options a project's content model gives: its directive schemas
/// (the built-ins, then the widgets) and its note types. Formatting with
/// options built any other way works, but the widgets' attribute order and
/// what counts as a directive come from these.
pub fn options_from_model(model: &ContentModel) -> ParseOptions {
    ParseOptions::new(model.directive_schemas())
        .with_note_types(model.notes.iter().map(|n| n.name.clone()).collect())
}

/// Formats `source` into canonical form (SPEC §8.3).
///
/// Returns the edits, which don't overlap and are sorted by position; apply
/// them with [`ascribe_core::apply_edits`]. An already canonical file gets no
/// edits. `options` decides which lines are directives (built-ins and the
/// project's widgets, and each schema's attribute order); `model` supplies
/// the order of `@variant`'s dimensions and of image attributes.
pub fn format(source: &str, options: &ParseOptions, model: &ContentModel) -> Vec<TextEdit> {
    let doc = parse(source, options);
    format_parsed(source, &doc, options, model)
}

/// [`format`] for a source already parsed with `options`.
pub fn format_parsed(
    source: &str,
    doc: &ParsedDocument,
    options: &ParseOptions,
    model: &ContentModel,
) -> Vec<TextEdit> {
    let mut ctx = Ctx {
        source,
        options,
        model,
        doc,
        blocking: skip::blocking_spans(doc),
        edits: Vec::new(),
    };
    ctx.blocks(&doc.blocks, Owner::Document);
    let mut edits = ctx.edits;
    edits.sort_by_key(|e| (e.span.start(), e.span.end()));
    // The rules edit disjoint bytes. If two ever met, keep the first rather
    // than return edits that can't be applied.
    let mut end = 0;
    edits.retain(|e| {
        let keep = e.span.start() >= end;
        if keep {
            end = e.span.end();
        }
        keep
    });
    edits
}

/// Formats `source` and returns the result: [`format`], applied.
pub fn format_source(source: &str, options: &ParseOptions, model: &ContentModel) -> String {
    let edits = format(source, options, model);
    apply_edits(source, &edits).unwrap_or_else(|_| source.to_owned())
}

/// What every rule needs, and where their edits go.
pub(crate) struct Ctx<'a> {
    pub(crate) source: &'a str,
    pub(crate) options: &'a ParseOptions,
    pub(crate) model: &'a ContentModel,
    pub(crate) doc: &'a ParsedDocument,
    /// Where an error was reported: what overlaps one is left alone.
    blocking: Vec<Span>,
    pub(crate) edits: Vec<TextEdit>,
}

impl Ctx<'_> {
    /// Whether an error was reported somewhere in `span` (both ends
    /// included, since a diagnostic can be an empty span at the end of a
    /// construct).
    pub(crate) fn has_error(&self, span: Span) -> bool {
        self.blocking
            .iter()
            .any(|at| at.end() >= span.start() && at.start() <= span.end())
    }

    /// The offset of the start of the line `offset` is on.
    pub(crate) fn line_start(&self, offset: usize) -> usize {
        self.source[..offset]
            .rfind(['\n', '\r'])
            .map_or(0, |i| i + 1)
    }

    /// The offset of the end of the line `offset` is on, before its ending.
    pub(crate) fn line_end(&self, offset: usize) -> usize {
        self.source[offset..]
            .find(['\n', '\r'])
            .map_or(self.source.len(), |i| offset + i)
    }

    /// The offset just after the line ending of the line `offset` is on.
    pub(crate) fn next_line(&self, offset: usize) -> usize {
        let end = self.line_end(offset);
        let rest = &self.source[end..];
        end + if rest.starts_with("\r\n") {
            2
        } else {
            usize::from(!rest.is_empty())
        }
    }

    /// Replaces `span` with `text`, unless that's already what it holds.
    pub(crate) fn replace(&mut self, span: Span, text: &str) {
        if self.source.get(span.range()) != Some(text) {
            self.edits.push(TextEdit::replace(span, text));
        }
    }

    // -- the walk ----------------------------------------------------------

    fn blocks(&mut self, blocks: &[Block], owner: Owner) {
        for (i, block) in blocks.iter().enumerate() {
            match &block.kind {
                BlockKind::Directive(line) => {
                    self.directive(line, owner);
                    blank::rule(self, blocks, i, owner);
                }
                BlockKind::Container(container) => {
                    self.directive(&container.opener, owner);
                    self.blocks(&container.children, owner);
                    if let Some(end) = &container.end {
                        self.end_line(end, owner);
                    }
                }
                BlockKind::Group(group) => {
                    for arm in &group.arms {
                        self.directive(&arm.opener, owner);
                        self.blocks(&arm.children, owner);
                    }
                    if let Some(end) = &group.end {
                        self.end_line(end, owner);
                    }
                }
                // An end line the structure pass left flat closes nothing,
                // and was reported.
                BlockKind::End(_) | BlockKind::Title(_) => {}
                BlockKind::BlockQuote(quote) => self.blocks(&quote.children, Owner::Quote),
                BlockKind::List(list) => {
                    for item in &list.items {
                        let owner = Owner::Item(indent::content_width(self.source, item.marker));
                        self.blocks(&item.children, owner);
                    }
                }
                BlockKind::Heading(h) => self.inlines(&h.inlines, false),
                BlockKind::Paragraph(p) => self.inlines(&p.inlines, false),
                BlockKind::Table(table) => {
                    for cell in table.rows.iter().flat_map(|r| &r.cells) {
                        self.inlines(&cell.inlines, true);
                    }
                }
                BlockKind::CodeBlock(_) | BlockKind::HtmlBlock(_) | BlockKind::ThematicBreak => {}
            }
        }
    }

    /// A directive line or container opener: its indentation and its head.
    fn directive(&mut self, line: &DirectiveLine, owner: Owner) {
        // The title line itself is left as written.
        // Titles hold inlines too, and an image in one has an attribute block.
        if let Some(title) = &line.title {
            self.inlines(&title.inlines, false);
        }
        if let Some(PrimaryValue::Text(primary)) = &line.primary {
            self.inlines(&primary.inlines, false);
        }
        let mut span = line.span;
        if let Some(title) = &line.title {
            span = span.cover(title.span);
        }
        // An unclosed attribute block runs to the end of its line.
        let span = Span::new(
            span.start(),
            span.end().max(self.line_end(line.span.start())),
        );
        if self.has_error(span) {
            return;
        }
        indent::rule(self, line.name_span.start(), owner);
        head::rules(self, line);
    }

    fn end_line(&mut self, end: &EndLine, owner: Owner) {
        let line = Span::new(end.span.start(), self.line_end(end.span.start()));
        if end.extra.is_some() || self.has_error(line) {
            return;
        }
        indent::rule(self, end.span.start(), owner);
    }

    /// The attribute block after each image in `inlines`.
    fn inlines(&mut self, inlines: &[Inline], in_table: bool) {
        for inline in inlines {
            match &inline.kind {
                InlineKind::Emphasis(children) | InlineKind::Strong(children) => {
                    self.inlines(children, in_table);
                }
                InlineKind::Link(link) => self.inlines(&link.children, in_table),
                InlineKind::Image(image) => {
                    self.inlines(&image.children, in_table);
                    if let Some(attributes) = &image.attributes {
                        // In a table cell a `|` is written `\|`, which the
                        // block's canonical spelling doesn't know.
                        if !in_table {
                            attributes::image_rule(self, &attributes.block);
                        }
                    }
                }
                InlineKind::Text(_)
                | InlineKind::Code(_)
                | InlineKind::SoftBreak
                | InlineKind::HardBreak
                | InlineKind::Html(_)
                | InlineKind::Phrase(_) => {}
            }
        }
    }
}
