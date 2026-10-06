//! Reaching the inline content a resolved block holds itself.

use ascribe_core::Span;
use ascribe_syntax::{BlockKind, DirectiveLine, Inline, InlineKind, PrimaryValue};

use super::tree::{ResolvedBlock, ResolvedKind};

/// Every list of inlines a block holds directly, not counting the blocks
/// nested in it: a heading's or paragraph's inlines, the cells of a table, the
/// title and text primary of a directive line, and the titles of a
/// container's opener or a group's arms.
pub(crate) fn own_lists_mut(block: &mut ResolvedBlock) -> Vec<&mut Vec<Inline>> {
    match &mut block.kind {
        ResolvedKind::Leaf(b) => match &mut b.kind {
            BlockKind::Heading(h) => vec![&mut h.inlines],
            BlockKind::Paragraph(p) => vec![&mut p.inlines],
            BlockKind::Table(t) => t
                .rows
                .iter_mut()
                .flat_map(|r| r.cells.iter_mut().map(|c| &mut c.inlines))
                .collect(),
            BlockKind::Directive(line) => line_lists_mut(line),
            _ => Vec::new(),
        },
        ResolvedKind::Container { opener, .. } => title_lists_mut(opener),
        ResolvedKind::Group { arms, .. } => arms
            .iter_mut()
            .flat_map(|a| title_lists_mut(&mut a.opener))
            .collect(),
        ResolvedKind::BlockQuote { .. } | ResolvedKind::List { .. } => Vec::new(),
    }
}

/// The lists of inlines that are prose (SPEC §5.4): paragraphs, table cells,
/// and the text primary of a directive. Not headings or titles.
pub(crate) fn prose_lists_mut(block: &mut ResolvedBlock) -> Vec<&mut Vec<Inline>> {
    match &mut block.kind {
        ResolvedKind::Leaf(b) => match &mut b.kind {
            BlockKind::Paragraph(p) => vec![&mut p.inlines],
            BlockKind::Table(t) => t
                .rows
                .iter_mut()
                .flat_map(|r| r.cells.iter_mut().map(|c| &mut c.inlines))
                .collect(),
            BlockKind::Directive(line) => match &mut line.primary {
                Some(PrimaryValue::Text(t)) => vec![&mut t.inlines],
                _ => Vec::new(),
            },
            _ => Vec::new(),
        },
        _ => Vec::new(),
    }
}

fn line_lists_mut(line: &mut DirectiveLine) -> Vec<&mut Vec<Inline>> {
    let mut out: Vec<&mut Vec<Inline>> = Vec::new();
    if let Some(title) = &mut line.title {
        out.push(&mut title.inlines);
    }
    if let Some(PrimaryValue::Text(text)) = &mut line.primary {
        out.push(&mut text.inlines);
    }
    out
}

fn title_lists_mut(line: &mut DirectiveLine) -> Vec<&mut Vec<Inline>> {
    line.title.iter_mut().map(|t| &mut t.inlines).collect()
}

/// The source ranges that hold a block's own inline content, in the sense of
/// [`own_lists_mut`]: a reference inside one belongs to the block itself.
pub(crate) fn own_ranges(block: &ResolvedBlock) -> Vec<Span> {
    match &block.kind {
        ResolvedKind::Leaf(b) => crate::index::walk::own_ranges(b),
        ResolvedKind::Container { opener, .. } => opener.title.iter().map(|t| t.span).collect(),
        ResolvedKind::Group { arms, .. } => arms
            .iter()
            .filter_map(|a| a.opener.title.as_ref().map(|t| t.span))
            .collect(),
        ResolvedKind::BlockQuote { .. } | ResolvedKind::List { .. } => Vec::new(),
    }
}

/// The link or image node with this span, looking through emphasis, link
/// text, and alt text.
pub(crate) fn find_mut(list: &mut [Inline], span: Span) -> Option<&mut Inline> {
    for inline in list {
        if inline.span == span && matches!(inline.kind, InlineKind::Link(_) | InlineKind::Image(_))
        {
            return Some(inline);
        }
        let found = match &mut inline.kind {
            InlineKind::Emphasis(c) | InlineKind::Strong(c) => find_mut(c, span),
            InlineKind::Link(l) => find_mut(&mut l.children, span),
            InlineKind::Image(i) => find_mut(&mut i.children, span),
            _ => None,
        };
        if found.is_some() {
            return found;
        }
    }
    None
}
