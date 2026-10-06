//! Semantic tokens (SPEC §10): what the editor colors, from the parse tree.
//!
//! The legend is a contract with the VS Code client and is documented in the
//! crate's README; entries are only appended, never reordered.

use std::collections::HashMap;

use ascribe_core::{AttributeBlock, AttributeValue, LineIndex, Span, schema::Builtin};
use ascribe_model::ContentModel;
use ascribe_resolve::FileIndex;
use ascribe_syntax::{Block, BlockKind, DirectiveLine, EndLine, Inline, InlineKind, PrimaryValue};
use lsp_types::{SemanticToken, SemanticTokenModifier, SemanticTokenType, SemanticTokensLegend};

use crate::position::Encoding;

/// The token types, in legend order.
pub const TYPES: [&str; 10] = [
    "ascribeDirective",
    "ascribeWidget",
    "ascribeAttributeKey",
    "ascribeAttributeValue",
    "ascribeColon",
    "ascribeEnd",
    "ascribeTitle",
    "ascribePhrase",
    "ascribePhraseUndeclared",
    "ascribeAvailability",
];

/// The token modifiers, in legend order.
pub const MODIFIERS: [&str; 1] = ["unknown"];

const DIRECTIVE: u32 = 0;
const WIDGET: u32 = 1;
const ATTRIBUTE_KEY: u32 = 2;
const ATTRIBUTE_VALUE: u32 = 3;
const COLON: u32 = 4;
const END: u32 = 5;
const TITLE: u32 = 6;
const PHRASE: u32 = 7;
const PHRASE_UNDECLARED: u32 = 8;
const AVAILABILITY: u32 = 9;

const UNKNOWN: u32 = 1;

/// The legend the server advertises.
pub fn legend() -> SemanticTokensLegend {
    SemanticTokensLegend {
        token_types: TYPES.iter().map(|t| SemanticTokenType::new(t)).collect(),
        token_modifiers: MODIFIERS
            .iter()
            .map(|m| SemanticTokenModifier::new(m))
            .collect(),
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Token {
    span: Span,
    ty: u32,
    modifiers: u32,
}

/// The tokens of a file, delta-encoded as the protocol wants, in `encoding`.
/// `only` limits them to those overlapping a byte range.
pub fn semantic_tokens(
    file: &FileIndex,
    model: &ContentModel,
    encoding: Encoding,
    only: Option<Span>,
) -> Vec<SemanticToken> {
    let mut tokens = collect(file, model);
    if let Some(range) = only {
        tokens.retain(|t| t.span.start() < range.end() && range.start() < t.span.end());
    }
    encode(&file.source, tokens, encoding)
}

fn collect(file: &FileIndex, model: &ContentModel) -> Vec<Token> {
    let schemas: HashMap<String, Vec<String>> = model
        .directive_schemas()
        .into_iter()
        .map(|s| {
            let keys = match s.attributes {
                ascribe_core::schema::Attributes::Declared(keys) => {
                    keys.into_iter().map(|k| k.key).collect()
                }
                // `@variant` takes any dimension name.
                ascribe_core::schema::Attributes::Dimensions => {
                    model.dimensions.iter().map(|d| d.name.clone()).collect()
                }
            };
            (s.name, keys)
        })
        .collect();
    let image_keys: Vec<String> = model
        .image_attributes
        .iter()
        .map(|a| a.key.clone())
        .collect();
    let mut walker = Walker {
        model,
        schemas,
        image_keys,
        out: Vec::new(),
    };
    walker.blocks(&file.document.blocks);
    for marker in &file.availability {
        if let Some((_, span)) = &marker.primary {
            walker.push(*span, AVAILABILITY, 0);
        }
    }
    let mut phrases = Vec::new();
    for use_ in &file.phrases {
        let ty = if use_.declared {
            PHRASE
        } else {
            PHRASE_UNDECLARED
        };
        phrases.push(Token {
            span: use_.phrase.span,
            ty,
            modifiers: 0,
        });
    }
    let mut out = carve(walker.out, &phrases);
    out.extend(phrases);
    out.sort_by_key(|t| (t.span.start(), t.span.end()));
    // A token never overlaps another; drop one that would (a defensive rule:
    // the collectors above don't produce any).
    let mut end = 0;
    out.retain(|t| {
        let keep = t.span.start() >= end && !t.span.is_empty();
        if keep {
            end = t.span.end();
        }
        keep
    });
    out
}

/// Removes the spans of `inner` tokens from the tokens that contain them (a
/// phrase inside a title line splits the title).
fn carve(outer: Vec<Token>, inner: &[Token]) -> Vec<Token> {
    let mut out = Vec::with_capacity(outer.len());
    for token in outer {
        let mut cuts: Vec<Span> = inner
            .iter()
            .map(|t| t.span)
            .filter(|s| s.start() < token.span.end() && token.span.start() < s.end())
            .collect();
        if cuts.is_empty() {
            out.push(token);
            continue;
        }
        cuts.sort_by_key(|s| s.start());
        let mut at = token.span.start();
        for cut in cuts {
            if cut.start() > at {
                out.push(Token {
                    span: Span::new(at, cut.start()),
                    ..token
                });
            }
            at = at.max(cut.end());
        }
        if at < token.span.end() {
            out.push(Token {
                span: Span::new(at, token.span.end()),
                ..token
            });
        }
    }
    out
}

struct Walker<'m> {
    model: &'m ContentModel,
    schemas: HashMap<String, Vec<String>>,
    image_keys: Vec<String>,
    out: Vec<Token>,
}

impl Walker<'_> {
    fn push(&mut self, span: Span, ty: u32, modifiers: u32) {
        self.out.push(Token {
            span,
            ty,
            modifiers,
        });
    }

    fn blocks(&mut self, blocks: &[Block]) {
        for block in blocks {
            self.block(block);
        }
    }

    fn block(&mut self, block: &Block) {
        match &block.kind {
            BlockKind::Heading(h) => self.inlines(&h.inlines),
            BlockKind::Paragraph(p) => self.inlines(&p.inlines),
            BlockKind::BlockQuote(q) => self.blocks(&q.children),
            BlockKind::List(l) => {
                for item in &l.items {
                    self.blocks(&item.children);
                }
            }
            BlockKind::Table(t) => {
                for row in &t.rows {
                    for cell in &row.cells {
                        self.inlines(&cell.inlines);
                    }
                }
            }
            BlockKind::Directive(d) => self.directive(d),
            BlockKind::End(e) => self.end(e),
            BlockKind::Container(c) => {
                self.directive(&c.opener);
                self.blocks(&c.children);
                if let Some(end) = &c.end {
                    self.end(end);
                }
            }
            BlockKind::Group(g) => {
                for arm in &g.arms {
                    self.directive(&arm.opener);
                    self.blocks(&arm.children);
                }
                if let Some(end) = &g.end {
                    self.end(end);
                }
            }
            BlockKind::CodeBlock(_)
            | BlockKind::HtmlBlock(_)
            | BlockKind::ThematicBreak
            | BlockKind::Title(_) => {}
        }
    }

    fn end(&mut self, end: &EndLine) {
        self.push(end.name_span, END, 0);
    }

    fn directive(&mut self, d: &DirectiveLine) {
        let ty = if self.model.widget(&d.name).is_some() {
            WIDGET
        } else {
            debug_assert!(
                Builtin::from_name(&d.name).is_some() || self.schemas.contains_key(&d.name)
            );
            DIRECTIVE
        };
        self.push(d.name_span, ty, 0);
        if let Some(block) = &d.attributes {
            let keys = self.schemas.get(&d.name).cloned();
            self.attributes(block, keys.as_deref());
        }
        if let Some(colon) = d.colon {
            self.push(colon, COLON, 0);
        }
        if let Some(title) = &d.title {
            self.push(title.span, TITLE, 0);
            self.inlines(&title.inlines);
        }
        if let Some(PrimaryValue::Text(text)) = &d.primary {
            self.inlines(&text.inlines);
        }
    }

    fn attributes(&mut self, block: &AttributeBlock, declared: Option<&[String]>) {
        for attribute in &block.attributes {
            let unknown = declared.is_some_and(|keys| !keys.contains(&attribute.key));
            self.push(
                attribute.key_span,
                ATTRIBUTE_KEY,
                if unknown { UNKNOWN } else { 0 },
            );
            match &attribute.value {
                Some(AttributeValue::Token(t)) => self.push(t.span, ATTRIBUTE_VALUE, 0),
                Some(AttributeValue::Quoted { span, .. }) => {
                    self.push(*span, ATTRIBUTE_VALUE, 0);
                }
                Some(AttributeValue::Set { members, .. }) => {
                    for member in members {
                        self.push(member.span, ATTRIBUTE_VALUE, 0);
                    }
                }
                None => {}
            }
        }
    }

    fn inlines(&mut self, inlines: &[Inline]) {
        for inline in inlines {
            match &inline.kind {
                InlineKind::Emphasis(children) | InlineKind::Strong(children) => {
                    self.inlines(children);
                }
                InlineKind::Link(link) => self.inlines(&link.children),
                InlineKind::Image(image) => {
                    self.inlines(&image.children);
                    if let Some(attributes) = &image.attributes {
                        let keys = self.image_keys.clone();
                        self.attributes(&attributes.block, Some(&keys));
                    }
                }
                _ => {}
            }
        }
    }
}

/// Splits tokens at line ends and delta-encodes them.
fn encode(text: &str, tokens: Vec<Token>, encoding: Encoding) -> Vec<SemanticToken> {
    let index = LineIndex::new(text);
    let mut out = Vec::with_capacity(tokens.len());
    let (mut prev_line, mut prev_start) = (0u32, 0u32);
    for token in tokens {
        for piece in split_lines(&index, token.span) {
            let (Some(start), Some(end)) = (
                encoding.position(&index, piece.start()),
                encoding.position(&index, piece.end()),
            ) else {
                continue;
            };
            if start.line != end.line || end.character <= start.character {
                continue;
            }
            let delta_line = start.line - prev_line;
            let delta_start = if delta_line == 0 {
                start.character - prev_start
            } else {
                start.character
            };
            out.push(SemanticToken {
                delta_line,
                delta_start,
                length: end.character - start.character,
                token_type: token.ty,
                token_modifiers_bitset: token.modifiers,
            });
            prev_line = start.line;
            prev_start = start.character;
        }
    }
    out
}

/// A span cut at line ends, one piece per line, without the line endings.
fn split_lines(index: &LineIndex, span: Span) -> Vec<Span> {
    let (Some(first), Some(last)) = (index.line_col(span.start()), index.line_col(span.end()))
    else {
        return Vec::new();
    };
    if first.line == last.line {
        return vec![span];
    }
    let mut pieces = Vec::new();
    for line in first.line..=last.line {
        let Some(text) = index.line_span(line) else {
            continue;
        };
        let start = text.start().max(span.start());
        let end = text.end().min(span.end());
        if start < end {
            pieces.push(Span::new(start, end));
        }
    }
    pieces
}
