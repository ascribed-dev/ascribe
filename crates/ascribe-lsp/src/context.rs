//! The `ascribe/context` request: what is at a position or a selection of a
//! page, so a client knows which actions apply there.
//!
//! The answer is the chain of what contains the start of the range, from the
//! innermost out (a phrase, the link it's in, the paragraph, the list item,
//! the list, the `@steps` that binds it, the section), what the selection is
//! when the range isn't empty, the token under the start of the range, and
//! whether a block can be inserted at it. It comes from the current snapshot,
//! so it includes unsaved edits. It holds nothing about the rest of the
//! project; that's `ascribe/targets`.

use ascribe_core::{AttributeBlock, LineIndex, Span};
use ascribe_resolve::{FileIndex, RefKind};
use ascribe_syntax::{
    Block, BlockKind, Bound, DirectiveLine, Inline, InlineKind, PrimaryValue, bound_block,
};
use lsp_types::{Range, TextDocumentIdentifier};
use serde::{Deserialize, Serialize};

use crate::nav::{Ctx, Hit, directive_at, hit_at};

/// The request's method name.
pub const METHOD: &str = "ascribe/context";

/// The parameters of `ascribe/context`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContextParams {
    /// The page.
    pub text_document: TextDocumentIdentifier,
    /// The selection; an empty range is the cursor.
    pub range: Range,
}

/// The answer to `ascribe/context`. A document that isn't a source file of
/// the project gets an empty answer: no project, nothing at the range.
#[derive(Debug, Default, Serialize, PartialEq)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct ContextResult {
    /// The project the page is in; `null` when it's in none.
    pub project: Option<ContextProject>,
    /// What contains the start of the range, innermost first.
    pub at: Vec<ContextNode>,
    /// What the selection is; `null` for an empty range.
    pub selection: Option<Selection>,
    /// The token under the start of the range, if there is one.
    pub token: Option<ContextToken>,
    /// Whether the line at the start of the range is blank and between
    /// blocks, where a block can be inserted: not in a code block or the
    /// frontmatter.
    pub insertable: bool,
}

/// The project a page is in.
#[derive(Debug, Serialize, PartialEq)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct ContextProject {
    /// The directory of its `ascribe.toml`, as a path.
    pub root: String,
    /// The editor's build (`[editor] build`), which decides the diagnostics.
    pub editor_build: String,
}

/// Something that contains a position: a block, a construct that groups
/// blocks, or an inline node. Every range is in the negotiated position
/// encoding.
#[derive(Debug, Clone, Serialize, PartialEq)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum ContextNode {
    /// The YAML frontmatter.
    Frontmatter {
        /// The frontmatter, delimiters included.
        #[cfg_attr(feature = "json-schema", schemars(with = "crate::schema::LspRange"))]
        range: Range,
        /// Its `variant` key's value, when it has one.
        variant: Option<FrontmatterValue>,
        /// Its `available` key's value, when it has one.
        available: Option<FrontmatterValue>,
    },
    /// A heading's section: the heading and what follows it up to the next
    /// heading of the same or a higher level.
    Section {
        /// The section.
        #[cfg_attr(feature = "json-schema", schemars(with = "crate::schema::LspRange"))]
        range: Range,
        /// The id of its heading; empty when the heading has none.
        heading_id: String,
    },
    /// A heading.
    Heading {
        /// The heading.
        #[cfg_attr(feature = "json-schema", schemars(with = "crate::schema::LspRange"))]
        range: Range,
        /// 1 to 6.
        level: u8,
        /// Its id: its `@id`, or its slug; empty when it has none.
        id: String,
        /// Whether the id is an `@id`.
        explicit_id: bool,
    },
    /// A paragraph.
    Paragraph {
        /// The paragraph.
        #[cfg_attr(feature = "json-schema", schemars(with = "crate::schema::LspRange"))]
        range: Range,
    },
    /// A list.
    List {
        /// The list.
        #[cfg_attr(feature = "json-schema", schemars(with = "crate::schema::LspRange"))]
        range: Range,
        /// Whether it's ordered.
        ordered: bool,
        /// Whether a `@steps` binds it.
        steps: bool,
    },
    /// An item of a list.
    ListItem {
        /// The item, from its marker.
        #[cfg_attr(feature = "json-schema", schemars(with = "crate::schema::LspRange"))]
        range: Range,
    },
    /// A block quote.
    BlockQuote {
        /// The block quote.
        #[cfg_attr(feature = "json-schema", schemars(with = "crate::schema::LspRange"))]
        range: Range,
    },
    /// A `@note`.
    Note {
        /// The note: its line, the block it binds, or the container, with
        /// its title line.
        #[cfg_attr(feature = "json-schema", schemars(with = "crate::schema::LspRange"))]
        range: Range,
        /// Its type (`note` when it gives none).
        #[serde(rename = "type")]
        note_type: String,
        /// Its form.
        form: Form,
    },
    /// A `@details`.
    Details {
        /// The details: the block it binds, or the container, with its title
        /// line.
        #[cfg_attr(feature = "json-schema", schemars(with = "crate::schema::LspRange"))]
        range: Range,
        /// Its title's text, as written.
        title: Option<String>,
        /// Its form.
        form: Form,
    },
    /// A `@steps` and the list it binds.
    Steps {
        /// From the `@steps` through the list.
        #[cfg_attr(feature = "json-schema", schemars(with = "crate::schema::LspRange"))]
        range: Range,
    },
    /// A group of `@variant` arms.
    VariantGroup {
        /// The group, through its `@end`.
        #[cfg_attr(feature = "json-schema", schemars(with = "crate::schema::LspRange"))]
        range: Range,
        /// The dimension every arm names; `null` for labeled arms.
        dimension: Option<String>,
        /// The arms, in order.
        arms: Vec<VariantArm>,
        /// The index of the arm the position is in; `null` on the group's
        /// `@end`.
        arm: Option<usize>,
    },
    /// An `@available` line: on a heading's section, just the line; binding
    /// a block, the line and the block.
    Availability {
        /// The line, or the line and the block it binds.
        #[cfg_attr(feature = "json-schema", schemars(with = "crate::schema::LspRange"))]
        range: Range,
        /// The spec or feature key, as written.
        spec: String,
    },
    /// An `@include`.
    Include {
        /// The directive line.
        #[cfg_attr(feature = "json-schema", schemars(with = "crate::schema::LspRange"))]
        range: Range,
        /// The path as written, without the `#id`.
        path: String,
        /// The id after `#`, when it includes one section.
        section: Option<String>,
    },
    /// A `@snippet`.
    Snippet {
        /// The directive line.
        #[cfg_attr(feature = "json-schema", schemars(with = "crate::schema::LspRange"))]
        range: Range,
        /// Its address as written: `<source>:<path>#<region>`.
        address: String,
    },
    /// A project widget.
    Widget {
        /// The widget: its line, the block it binds, its container, or its
        /// group of arms.
        #[cfg_attr(feature = "json-schema", schemars(with = "crate::schema::LspRange"))]
        range: Range,
        /// Its name.
        name: String,
        /// Its attributes, as written: of the group's arm the position is in,
        /// for a group.
        attributes: Vec<AttributePair>,
        /// Its form.
        form: Form,
    },
    /// A code block, fenced or indented.
    CodeBlock {
        /// The block, fences included.
        #[cfg_attr(feature = "json-schema", schemars(with = "crate::schema::LspRange"))]
        range: Range,
        /// The fence's info string; empty when it has none.
        info: String,
        /// Whether it's fenced.
        fenced: bool,
    },
    /// A table.
    Table {
        /// The table.
        #[cfg_attr(feature = "json-schema", schemars(with = "crate::schema::LspRange"))]
        range: Range,
    },
    /// A row of a table.
    TableRow {
        /// The row.
        #[cfg_attr(feature = "json-schema", schemars(with = "crate::schema::LspRange"))]
        range: Range,
        /// Whether it's the header row.
        header: bool,
        /// Its `available` attribute's value, when it has one.
        available: Option<String>,
    },
    /// A link.
    Link {
        /// The link, brackets included.
        #[cfg_attr(feature = "json-schema", schemars(with = "crate::schema::LspRange"))]
        range: Range,
        /// Its destination, escapes decoded.
        destination: String,
        /// Whether it has no text, so it takes its target's title.
        text_empty: bool,
    },
    /// An image.
    Image {
        /// The image, its attribute block included.
        #[cfg_attr(feature = "json-schema", schemars(with = "crate::schema::LspRange"))]
        range: Range,
        /// Its source, escapes decoded.
        src: String,
        /// Its alt text, as written.
        alt: String,
        /// Its attributes (`{width=600}`), as written.
        attributes: Vec<AttributePair>,
    },
    /// A phrase candidate, `{key}`.
    Phrase {
        /// The candidate, braces included.
        #[cfg_attr(feature = "json-schema", schemars(with = "crate::schema::LspRange"))]
        range: Range,
        /// The key.
        key: String,
        /// Whether the content model declares it.
        declared: bool,
    },
}

/// How a directive is written (SPEC §3.5, §3.6).
#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub enum Form {
    /// One line, whose primary is its content, or that stands alone.
    Line,
    /// One line that binds the block below it.
    Block,
    /// A container, through its `@end`.
    Container,
    /// A group of arms.
    Group,
}

/// A frontmatter key's value.
#[derive(Debug, Clone, Serialize, PartialEq)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct FrontmatterValue {
    /// The value, after the key's colon, through its last line.
    #[cfg_attr(feature = "json-schema", schemars(with = "crate::schema::LspRange"))]
    pub range: Range,
    /// The value's YAML, as written; a one-line value without its quotes.
    pub value: String,
}

/// An arm of a `@variant` group.
#[derive(Debug, Clone, Serialize, PartialEq)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct VariantArm {
    /// The arm's values of the group's dimension, as written
    /// (`cloud|self-managed`); `null` for a labeled arm.
    pub value: Option<String>,
    /// The arm's title, for a labeled arm.
    pub label: Option<String>,
    /// The arm, from its title line or opener through its last block.
    #[cfg_attr(feature = "json-schema", schemars(with = "crate::schema::LspRange"))]
    pub range: Range,
}

/// An attribute, as written.
#[derive(Debug, Clone, Serialize, PartialEq)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct AttributePair {
    /// The key.
    pub key: String,
    /// The value, without quotes (a value set's members joined by `|`);
    /// `null` for a key with no value.
    pub value: Option<String>,
}

/// What a selection is.
#[derive(Debug, Serialize, PartialEq)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct Selection {
    /// Its kind.
    pub kind: SelectionKind,
    /// The selected text.
    pub text: String,
    /// Whether it's inside one paragraph's or heading's text.
    pub inline: bool,
}

/// The kinds of selection. Whitespace at either end doesn't count.
#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub enum SelectionKind {
    /// Prose: inside one paragraph's or heading's text (`inline`), or
    /// across paragraphs and headings and covering only part of one.
    Prose,
    /// Whole blocks of any kind, side by side.
    Blocks,
    /// Inside one code block.
    Code,
    /// Across blocks and covering part of one that isn't prose.
    Mixed,
    /// Anything else: part of one table, directive line, or the frontmatter.
    Other,
}

/// The token under a position.
#[derive(Debug, Clone, Serialize, PartialEq)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum ContextToken {
    /// A link.
    Link {
        /// The link, brackets included.
        #[cfg_attr(feature = "json-schema", schemars(with = "crate::schema::LspRange"))]
        range: Range,
        /// Its destination, escapes decoded.
        destination: String,
        /// Whether it has no text.
        text_empty: bool,
    },
    /// An image.
    Image {
        /// The image, its attribute block included.
        #[cfg_attr(feature = "json-schema", schemars(with = "crate::schema::LspRange"))]
        range: Range,
        /// Its source, escapes decoded.
        src: String,
    },
    /// The path of an `@include`.
    Include {
        /// The path, and the `#id` if it has one.
        #[cfg_attr(feature = "json-schema", schemars(with = "crate::schema::LspRange"))]
        range: Range,
        /// The path as written, without the `#id`.
        path: String,
        /// The id after `#`.
        section: Option<String>,
    },
    /// A phrase candidate.
    Phrase {
        /// The candidate, braces included.
        #[cfg_attr(feature = "json-schema", schemars(with = "crate::schema::LspRange"))]
        range: Range,
        /// The key.
        key: String,
        /// Whether the content model declares it.
        declared: bool,
    },
    /// A directive's `@` and name.
    DirectiveName {
        /// The `@` and the name.
        #[cfg_attr(feature = "json-schema", schemars(with = "crate::schema::LspRange"))]
        range: Range,
        /// The name, without `@`.
        name: String,
    },
    /// An attribute of a directive.
    Attribute {
        /// From the key through the value.
        #[cfg_attr(feature = "json-schema", schemars(with = "crate::schema::LspRange"))]
        range: Range,
        /// The directive's name.
        directive: String,
        /// The key.
        key: String,
        /// The value, without quotes; `null` for a key with no value.
        value: Option<String>,
    },
}

/// What is at a range of the requested page.
pub(crate) fn context(ctx: &Ctx, range: Range) -> ContextResult {
    let Some(file) = ctx.file() else {
        return ContextResult::default();
    };
    let index = LineIndex::new(&file.source);
    let source: &str = &file.source;
    let start = ctx.encoding.offset_lenient(&index, source, range.start);
    let end = ctx
        .encoding
        .offset_lenient(&index, source, range.end)
        .max(start);
    let mut walk = Walk {
        ctx,
        file,
        index: &index,
        offset: start,
        found: Vec::new(),
    };
    walk.frontmatter();
    walk.blocks(&file.document.blocks);
    let found = walk.found;
    let token = token(ctx, file, &index, start, &found);
    let insertable = start == end
        && is_blank_line(source, start)
        && !found.iter().any(|(_, node)| {
            matches!(
                node,
                ContextNode::CodeBlock { .. } | ContextNode::Frontmatter { .. }
            )
        });
    let selection = (start < end).then(|| {
        let (kind, inline) = selection(file, start, end);
        Selection {
            kind,
            text: source[start..end].to_owned(),
            inline,
        }
    });
    ContextResult {
        project: Some(ContextProject {
            root: ctx
                .config
                .parent()
                .map(|root| root.to_string_lossy().into_owned())
                .unwrap_or_default(),
            editor_build: ctx.model.editor_build.clone(),
        }),
        at: found.into_iter().rev().map(|(_, node)| node).collect(),
        selection,
        token,
        insertable,
    }
}

fn touches(span: Span, offset: usize) -> bool {
    span.start() <= offset && offset <= span.end()
}

/// Whether the line holding `offset` is empty or only whitespace.
fn is_blank_line(source: &str, offset: usize) -> bool {
    let start = crate::nav::line_start(source, offset);
    let end = source[offset..]
        .find('\n')
        .map_or(source.len(), |i| offset + i);
    source[start..end].trim().is_empty()
}

/// The walk down the tree to the position: what it found, outermost first,
/// each with its span.
struct Walk<'a> {
    ctx: &'a Ctx,
    file: &'a FileIndex,
    index: &'a LineIndex,
    offset: usize,
    found: Vec<(Span, ContextNode)>,
}

impl Walk<'_> {
    fn range(&self, span: Span) -> Range {
        self.ctx.encoding.range(self.index, span)
    }

    fn text(&self, span: Span) -> String {
        self.file
            .source
            .get(span.range())
            .unwrap_or_default()
            .to_owned()
    }

    fn push(&mut self, span: Span, node: ContextNode) {
        self.found.push((span, node));
    }

    fn frontmatter(&mut self) {
        let Some(frontmatter) = &self.file.document.frontmatter else {
            return;
        };
        if !touches(frontmatter.span, self.offset) {
            return;
        }
        let value = |key| {
            frontmatter_value(&self.file.source, frontmatter.content, key).map(|(span, value)| {
                FrontmatterValue {
                    range: self.range(span),
                    value,
                }
            })
        };
        let node = ContextNode::Frontmatter {
            range: self.range(frontmatter.span),
            variant: value("variant"),
            available: value("available"),
        };
        self.push(frontmatter.span, node);
    }

    /// One list of sibling blocks: the sections there, the directives that
    /// bind a block, and the block the position is in.
    fn blocks(&mut self, blocks: &[Block]) {
        let offset = self.offset;
        let mut sections: Vec<_> = self
            .file
            .headings
            .iter()
            .filter(|h| touches(h.section, offset) && blocks.iter().any(|b| b.span == h.span))
            .collect();
        sections.sort_by_key(|h| (h.section.start(), std::cmp::Reverse(h.section.end())));
        for heading in sections {
            let node = ContextNode::Section {
                range: self.range(heading.section),
                heading_id: heading.source_id.clone(),
            };
            self.push(heading.section, node);
        }
        for i in 0..blocks.len() {
            let Some(bound) = bound_block(blocks, i) else {
                continue;
            };
            let BlockKind::Directive(line) = &blocks[i].kind else {
                continue;
            };
            let span = Span::new(blocks[i].span.start(), blocks[bound].span.end());
            if touches(span, offset) {
                self.bound(line, span);
            }
        }
        let steps_bound = |k: usize| {
            (0..k).any(|i| {
                bound_block(blocks, i) == Some(k)
                    && matches!(&blocks[i].kind, BlockKind::Directive(l) if l.name == "steps")
            })
        };
        if let Some(k) = blocks.iter().position(|b| touches(b.span, offset)) {
            self.block(&blocks[k], steps_bound(k));
        }
    }

    /// A line-form directive that binds the block below it: it holds the
    /// block.
    fn bound(&mut self, line: &DirectiveLine, span: Span) {
        let range = self.range(span);
        let node = match line.name.as_str() {
            "note" => ContextNode::Note {
                range,
                note_type: note_type(line),
                form: Form::Block,
            },
            "details" => ContextNode::Details {
                range,
                title: line.title.as_ref().map(|t| self.text(t.content)),
                form: Form::Block,
            },
            "steps" => ContextNode::Steps { range },
            "available" => ContextNode::Availability {
                range,
                spec: line_primary(line),
            },
            name if self.ctx.model.widget(name).is_some() => ContextNode::Widget {
                range,
                name: name.to_owned(),
                attributes: attributes(line.attributes.as_ref()),
                form: Form::Block,
            },
            _ => return,
        };
        self.push(span, node);
    }

    fn block(&mut self, block: &Block, steps: bool) {
        let range = self.range(block.span);
        let span = block.span;
        match &block.kind {
            BlockKind::Heading(h) => {
                let found = self.file.headings.iter().find(|x| x.span == span);
                let node = ContextNode::Heading {
                    range,
                    level: h.level,
                    id: found.map(|x| x.source_id.clone()).unwrap_or_default(),
                    explicit_id: found.is_some_and(|x| x.explicit_id.is_some()),
                };
                self.push(span, node);
                self.inlines(&h.inlines);
            }
            BlockKind::Paragraph(p) => {
                self.push(span, ContextNode::Paragraph { range });
                self.inlines(&p.inlines);
            }
            BlockKind::CodeBlock(code) => {
                let node = ContextNode::CodeBlock {
                    range,
                    info: code.info.clone(),
                    fenced: code.fenced,
                };
                self.push(span, node);
            }
            BlockKind::BlockQuote(q) => {
                self.push(span, ContextNode::BlockQuote { range });
                self.blocks(&q.children);
            }
            BlockKind::List(list) => {
                let node = ContextNode::List {
                    range,
                    ordered: list.ordered,
                    steps,
                };
                self.push(span, node);
                if let Some(item) = list.items.iter().find(|i| touches(i.span, self.offset)) {
                    let node = ContextNode::ListItem {
                        range: self.range(item.span),
                    };
                    self.push(item.span, node);
                    self.blocks(&item.children);
                }
            }
            BlockKind::Table(table) => {
                self.push(span, ContextNode::Table { range });
                if let Some(row) = table.rows.iter().find(|r| touches(r.span, self.offset)) {
                    let available = row
                        .attributes
                        .as_ref()
                        .and_then(|a| a.get("available"))
                        .and_then(|a| a.value.as_ref())
                        .and_then(|v| v.as_text())
                        .map(str::to_owned);
                    let node = ContextNode::TableRow {
                        range: self.range(row.span),
                        header: row.header,
                        available,
                    };
                    self.push(row.span, node);
                    if let Some(cell) = row.cells.iter().find(|c| touches(c.span, self.offset)) {
                        self.inlines(&cell.inlines);
                    }
                }
            }
            BlockKind::Directive(line) => self.directive(line, span),
            BlockKind::Container(c) => {
                let opener = &c.opener;
                let node = match opener.name.as_str() {
                    "note" => Some(ContextNode::Note {
                        range,
                        note_type: note_type(opener),
                        form: Form::Container,
                    }),
                    "details" => Some(ContextNode::Details {
                        range,
                        title: opener.title.as_ref().map(|t| self.text(t.content)),
                        form: Form::Container,
                    }),
                    name if self.ctx.model.widget(name).is_some() => Some(ContextNode::Widget {
                        range,
                        name: name.to_owned(),
                        attributes: attributes(opener.attributes.as_ref()),
                        form: Form::Container,
                    }),
                    _ => None,
                };
                if let Some(node) = node {
                    self.push(span, node);
                }
                self.title_and_primary(opener);
                self.blocks(&c.children);
            }
            BlockKind::Group(group) => {
                let arm = group.arms.iter().position(|a| touches(a.span, self.offset));
                if group.name == "variant" {
                    let dimension = group_dimension(group);
                    let arms = group
                        .arms
                        .iter()
                        .map(|a| VariantArm {
                            value: dimension.as_deref().and_then(|d| {
                                attributes(a.opener.attributes.as_ref())
                                    .into_iter()
                                    .find(|p| p.key == d)
                                    .and_then(|p| p.value)
                            }),
                            label: a.title.as_ref().map(|t| self.text(t.content)),
                            range: self.range(a.span),
                        })
                        .collect();
                    let node = ContextNode::VariantGroup {
                        range,
                        dimension,
                        arms,
                        arm,
                    };
                    self.push(span, node);
                } else if self.ctx.model.widget(&group.name).is_some() {
                    let node = ContextNode::Widget {
                        range,
                        name: group.name.clone(),
                        attributes: arm
                            .map(|i| attributes(group.arms[i].opener.attributes.as_ref()))
                            .unwrap_or_default(),
                        form: Form::Group,
                    };
                    self.push(span, node);
                }
                if let Some(i) = arm {
                    self.title_and_primary(&group.arms[i].opener);
                    self.blocks(&group.arms[i].children);
                }
            }
            BlockKind::HtmlBlock(_)
            | BlockKind::ThematicBreak
            | BlockKind::End(_)
            | BlockKind::Title(_) => {}
        }
    }

    /// A line-form directive. One that binds the block below it is found
    /// with that block ([`Walk::bound`]).
    fn directive(&mut self, line: &DirectiveLine, span: Span) {
        if line.binding != Some(Bound::FollowingBlock) {
            let range = self.range(span);
            let node = match line.name.as_str() {
                "note" => Some(ContextNode::Note {
                    range,
                    note_type: note_type(line),
                    form: Form::Line,
                }),
                "available" => Some(ContextNode::Availability {
                    range,
                    spec: line_primary(line),
                }),
                "include" => self
                    .file
                    .includes
                    .iter()
                    .find(|i| i.span == line.span)
                    .map(|i| ContextNode::Include {
                        range,
                        path: i.written.clone(),
                        section: i.section.clone(),
                    }),
                "snippet" => self
                    .file
                    .snippet_at(line.span)
                    .map(|s| ContextNode::Snippet {
                        range,
                        address: s.written.clone(),
                    }),
                name if self.ctx.model.widget(name).is_some() => Some(ContextNode::Widget {
                    range,
                    name: name.to_owned(),
                    attributes: attributes(line.attributes.as_ref()),
                    form: Form::Line,
                }),
                _ => None,
            };
            if let Some(node) = node {
                self.push(span, node);
            }
        }
        self.title_and_primary(line);
    }

    /// The inline content of a directive line's title and text primary.
    fn title_and_primary(&mut self, line: &DirectiveLine) {
        if let Some(title) = line.title.as_ref().filter(|t| touches(t.span, self.offset)) {
            self.inlines(&title.inlines);
        }
        if let Some(PrimaryValue::Text(text)) = &line.primary
            && touches(text.span, self.offset)
        {
            self.inlines(&text.inlines);
        }
    }

    /// The links, images, and phrases the position is in.
    fn inlines(&mut self, inlines: &[Inline]) {
        let offset = self.offset;
        let Some(inline) = inlines.iter().find(|i| touches(i.span, offset)) else {
            return;
        };
        let range = self.range(inline.span);
        match &inline.kind {
            InlineKind::Emphasis(children) | InlineKind::Strong(children) => {
                self.inlines(children);
            }
            InlineKind::Link(link) => {
                let text_empty = self
                    .file
                    .references
                    .iter()
                    .find(|r| r.span == inline.span)
                    .is_some_and(|r| r.text_empty);
                let node = ContextNode::Link {
                    range,
                    destination: link.destination.clone(),
                    text_empty,
                };
                self.push(inline.span, node);
                self.inlines(&link.children);
                self.phrases(&link.destination_phrases);
            }
            InlineKind::Image(image) => {
                let node = ContextNode::Image {
                    range,
                    src: image.destination.clone(),
                    alt: self.text(image.alt),
                    attributes: attributes(image.attributes.as_ref().map(|a| &a.block)),
                };
                self.push(inline.span, node);
                self.inlines(&image.children);
                self.phrases(&image.destination_phrases);
            }
            InlineKind::Phrase(phrase) => self.phrases(std::slice::from_ref(phrase)),
            _ => {}
        }
    }

    fn phrases(&mut self, phrases: &[ascribe_syntax::Phrase]) {
        if let Some(phrase) = phrases.iter().find(|p| touches(p.span, self.offset)) {
            let node = ContextNode::Phrase {
                range: self.range(phrase.span),
                key: phrase.key.clone(),
                declared: self.ctx.model.has_phrase(&phrase.key),
            };
            self.push(phrase.span, node);
        }
    }
}

/// A `@note`'s type: its `type` attribute, or `note`.
fn note_type(line: &DirectiveLine) -> String {
    line.attributes
        .as_ref()
        .and_then(|a| a.get("type"))
        .and_then(|a| a.value.as_ref())
        .and_then(|v| v.as_text())
        .unwrap_or("note")
        .to_owned()
}

/// A directive's line primary (an availability spec), as written.
fn line_primary(line: &DirectiveLine) -> String {
    match &line.primary {
        Some(PrimaryValue::Line(p)) => p.text.clone(),
        _ => String::new(),
    }
}

fn attributes(block: Option<&AttributeBlock>) -> Vec<AttributePair> {
    block
        .map(|b| {
            b.attributes
                .iter()
                .map(|a| AttributePair {
                    key: a.key.clone(),
                    value: a.value.as_ref().map(|v| v.members().join("|")),
                })
                .collect()
        })
        .unwrap_or_default()
}

/// The dimension a group's arms vary by: the first key of its first arm that
/// every arm has. `None` for labeled arms.
fn group_dimension(group: &ascribe_syntax::Group) -> Option<String> {
    let keys = |line: &DirectiveLine| -> Vec<String> {
        line.attributes
            .as_ref()
            .map(|b| b.attributes.iter().map(|a| a.key.clone()).collect())
            .unwrap_or_default()
    };
    let first = group.arms.first()?;
    keys(&first.opener)
        .into_iter()
        .find(|key| group.arms.iter().all(|a| keys(&a.opener).contains(key)))
}

/// The value of a top-level frontmatter key: from after its colon through
/// its last line (lines indented below it, or list items), with the text.
/// A one-line value is given without its quotes.
fn frontmatter_value(source: &str, content: Span, key: &str) -> Option<(Span, String)> {
    let text = source.get(content.range())?;
    let prefix = format!("{key}:");
    let mut at = content.start();
    let mut lines = text.split_inclusive('\n').peekable();
    while let Some(line) = lines.next() {
        let line_start = at;
        at += line.len();
        let Some(rest) = line.strip_prefix(&prefix) else {
            continue;
        };
        let lead = rest.len() - rest.trim_start_matches([' ', '\t']).len();
        let from = line_start + prefix.len() + lead;
        let mut to = line_start + line.trim_end().len();
        while let Some(next) = lines.peek() {
            let continues = next.starts_with([' ', '\t', '-']) || next.trim().is_empty();
            if !continues {
                break;
            }
            if !next.trim().is_empty() {
                to = at + next.trim_end().len();
            }
            at += next.len();
            lines.next();
        }
        let to = to.max(from);
        let value = source.get(from..to)?;
        let unquoted = value
            .strip_prefix('"')
            .and_then(|v| v.strip_suffix('"'))
            .or_else(|| value.strip_prefix('\'').and_then(|v| v.strip_suffix('\'')));
        return Some(match unquoted {
            Some(inner) if !value.contains('\n') => (Span::new(from + 1, to - 1), inner.to_owned()),
            _ => (Span::new(from, to), value.to_owned()),
        });
    }
    None
}

/// The token under `offset`.
fn token(
    ctx: &Ctx,
    file: &FileIndex,
    index: &LineIndex,
    offset: usize,
    found: &[(Span, ContextNode)],
) -> Option<ContextToken> {
    let range = |span: Span| ctx.encoding.range(index, span);
    let hit = hit_at(file, offset).and_then(|hit| match hit {
        Hit::Phrase(phrase) => Some(ContextToken::Phrase {
            range: range(phrase.phrase.span),
            key: phrase.phrase.key.clone(),
            declared: phrase.declared,
        }),
        Hit::Reference(_, reference) => Some(match reference.kind {
            RefKind::Link => ContextToken::Link {
                range: range(reference.span),
                destination: reference.destination.clone(),
                text_empty: reference.text_empty,
            },
            RefKind::Image => ContextToken::Image {
                range: range(reference.span),
                src: found
                    .iter()
                    .find_map(|(span, node)| match node {
                        ContextNode::Image { src, .. } if *span == reference.span => {
                            Some(src.clone())
                        }
                        _ => None,
                    })
                    .unwrap_or_else(|| reference.destination.clone()),
            },
        }),
        Hit::Include(include) => Some(ContextToken::Include {
            range: range(include.primary?),
            path: include.written.clone(),
            section: include.section.clone(),
        }),
        Hit::Directive(line) => directive_token(line, offset, &range),
        Hit::Availability(..) | Hit::Frontmatter(..) => None,
    });
    hit.or_else(|| {
        let line = directive_at(&file.document.blocks, offset)?;
        directive_token(line, offset, &range)
    })
}

/// A directive's name, or the attribute under `offset`.
fn directive_token(
    line: &DirectiveLine,
    offset: usize,
    range: &impl Fn(Span) -> Range,
) -> Option<ContextToken> {
    if touches(line.name_span, offset) {
        return Some(ContextToken::DirectiveName {
            range: range(line.name_span),
            name: line.name.clone(),
        });
    }
    let attribute = line
        .attributes
        .as_ref()?
        .attributes
        .iter()
        .find(|a| touches(a.span, offset))?;
    Some(ContextToken::Attribute {
        range: range(attribute.span),
        directive: line.name.clone(),
        key: attribute.key.clone(),
        value: attribute.value.as_ref().map(|v| v.members().join("|")),
    })
}

/// What a selection from `start` to `end` is.
fn selection(file: &FileIndex, start: usize, end: usize) -> (SelectionKind, bool) {
    let text = &file.source[start..end];
    let s = start + (text.len() - text.trim_start().len());
    let e = end - (text.len() - text.trim_end().len());
    if s >= e {
        return (SelectionKind::Other, false);
    }
    if file
        .document
        .frontmatter
        .as_ref()
        .is_some_and(|f| f.span.start() < e && s < f.span.end())
    {
        return (SelectionKind::Other, false);
    }
    within(&file.document.blocks, s, e)
}

/// The selection `s..e` among sibling blocks.
fn within(blocks: &[Block], s: usize, e: usize) -> (SelectionKind, bool) {
    let Some(block) = blocks
        .iter()
        .find(|b| b.span.start() <= s && e <= b.span.end())
    else {
        return across(blocks.iter().map(|b| (b.span, is_prose(b))), s, e);
    };
    let whole = s <= block.span.start() && block.span.end() <= e;
    let nested = |children: &[Block]| {
        if whole {
            (SelectionKind::Blocks, false)
        } else {
            within(children, s, e)
        }
    };
    match &block.kind {
        BlockKind::Paragraph(_) | BlockKind::Heading(_) => (SelectionKind::Prose, true),
        BlockKind::CodeBlock(_) if whole => (SelectionKind::Blocks, false),
        BlockKind::CodeBlock(_) => (SelectionKind::Code, false),
        BlockKind::BlockQuote(q) => nested(&q.children),
        BlockKind::Container(c) => nested(&c.children),
        BlockKind::List(list) => {
            if whole {
                return (SelectionKind::Blocks, false);
            }
            match list
                .items
                .iter()
                .find(|i| i.span.start() <= s && e <= i.span.end())
            {
                Some(item) => within(&item.children, s, e),
                None => across(list.items.iter().map(|i| (i.span, false)), s, e),
            }
        }
        BlockKind::Group(group) => {
            if whole {
                return (SelectionKind::Blocks, false);
            }
            match group
                .arms
                .iter()
                .find(|a| a.span.start() <= s && e <= a.span.end())
            {
                Some(arm) => within(&arm.children, s, e),
                None => across(group.arms.iter().map(|a| (a.span, false)), s, e),
            }
        }
        _ if whole => (SelectionKind::Blocks, false),
        _ => (SelectionKind::Other, false),
    }
}

/// A selection across sibling blocks, each given as its span and whether it
/// is prose.
fn across(blocks: impl Iterator<Item = (Span, bool)>, s: usize, e: usize) -> (SelectionKind, bool) {
    let mut touched = false;
    let mut all_whole = true;
    let mut partial_prose_only = true;
    for (span, prose) in blocks.filter(|(span, _)| span.start() < e && s < span.end()) {
        touched = true;
        if !(s <= span.start() && span.end() <= e) {
            all_whole = false;
            partial_prose_only &= prose;
        }
    }
    let kind = match (touched, all_whole, partial_prose_only) {
        (false, ..) => SelectionKind::Other,
        (true, true, _) => SelectionKind::Blocks,
        (true, false, true) => SelectionKind::Prose,
        (true, false, false) => SelectionKind::Mixed,
    };
    (kind, false)
}

fn is_prose(block: &Block) -> bool {
    matches!(block.kind, BlockKind::Paragraph(_) | BlockKind::Heading(_))
}
