//! Walking the syntax tree: blocks in document order, and the inline
//! content each block holds itself.

use ascribe_core::Span;
use ascribe_syntax::{Block, BlockKind, DirectiveLine, Inline, InlineKind, PrimaryValue};

/// Calls `f` on every block, children after their parent, in document order.
/// It enters block quotes, list items, containers, and the arms of groups.
pub(crate) fn walk_blocks<'a>(blocks: &'a [Block], f: &mut impl FnMut(&'a Block)) {
    for block in blocks {
        f(block);
        match &block.kind {
            BlockKind::BlockQuote(q) => walk_blocks(&q.children, f),
            BlockKind::List(l) => {
                for item in &l.items {
                    walk_blocks(&item.children, f);
                }
            }
            BlockKind::Container(c) => walk_blocks(&c.children, f),
            BlockKind::Group(g) => {
                for arm in &g.arms {
                    walk_blocks(&arm.children, f);
                }
            }
            _ => {}
        }
    }
}

/// The inline content a block holds directly, not counting the blocks nested
/// inside it: a heading's or paragraph's inlines, the cells of a table, and
/// the title and text primary of a directive line (for a container or a
/// group, of its opener or arms' openers).
pub(crate) fn own_inlines(block: &Block) -> Vec<&[Inline]> {
    match &block.kind {
        BlockKind::Heading(h) => vec![&h.inlines],
        BlockKind::Paragraph(p) => vec![&p.inlines],
        BlockKind::Table(t) => t
            .rows
            .iter()
            .flat_map(|r| r.cells.iter().map(|c| c.inlines.as_slice()))
            .collect(),
        BlockKind::Directive(line) => line_inlines(line),
        BlockKind::Container(c) => line_inlines(&c.opener),
        BlockKind::Group(g) => g
            .arms
            .iter()
            .flat_map(|a| line_inlines(&a.opener))
            .collect(),
        BlockKind::Title(t) => vec![&t.inlines],
        _ => Vec::new(),
    }
}

fn line_inlines(line: &DirectiveLine) -> Vec<&[Inline]> {
    let mut out: Vec<&[Inline]> = Vec::new();
    if let Some(title) = &line.title {
        out.push(&title.inlines);
    }
    if let Some(PrimaryValue::Text(text)) = &line.primary {
        out.push(&text.inlines);
    }
    out
}

/// The source ranges that hold a block's own inline content, in the sense of
/// [`own_inlines`]. A reference inside one of these ranges belongs to the
/// block itself, not to something nested in it.
pub(crate) fn own_ranges(block: &Block) -> Vec<Span> {
    match &block.kind {
        BlockKind::BlockQuote(_) | BlockKind::List(_) => Vec::new(),
        BlockKind::Container(c) => c.opener.title.iter().map(|t| t.span).collect(),
        BlockKind::Group(g) => g
            .arms
            .iter()
            .filter_map(|a| a.opener.title.as_ref().map(|t| t.span))
            .collect(),
        _ => vec![block.span],
    }
}

/// Calls `f` on every inline, children after their parent. It enters
/// emphasis, strong text, link text, and image alt text.
pub(crate) fn walk_inlines<'a>(inlines: &'a [Inline], f: &mut impl FnMut(&'a Inline)) {
    for inline in inlines {
        f(inline);
        match &inline.kind {
            InlineKind::Emphasis(children) | InlineKind::Strong(children) => {
                walk_inlines(children, f);
            }
            InlineKind::Link(link) => walk_inlines(&link.children, f),
            InlineKind::Image(image) => walk_inlines(&image.children, f),
            _ => {}
        }
    }
}
