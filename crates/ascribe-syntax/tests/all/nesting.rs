//! Nesting is bounded: past `MAX_NESTING` levels, a block or an inline is its
//! source text, so a page that nests thousands of levels deep parses instead
//! of overflowing the stack of the thread parsing it.

#![allow(clippy::expect_used, clippy::panic)]

use crate::support;

use ascribe_syntax::*;

fn parse_text(source: &str) -> ParsedDocument {
    parse(source, &ParseOptions::default())
}

/// How deep a tree of blocks goes, counting as the converter counts: a block
/// or an inline is a level, a list's items aren't. Past the limit, a block is
/// a paragraph of text, two levels more.
fn block_depth(blocks: &[Block]) -> usize {
    blocks
        .iter()
        .map(|block| match &block.kind {
            BlockKind::BlockQuote(quote) => 1 + block_depth(&quote.children),
            BlockKind::List(list) => {
                1 + list
                    .items
                    .iter()
                    .map(|item| block_depth(&item.children))
                    .max()
                    .unwrap_or(0)
            }
            BlockKind::Paragraph(p) => 1 + inline_depth(&p.inlines),
            _ => 1,
        })
        .max()
        .unwrap_or(0)
}

/// How deep a tree of inlines goes.
fn inline_depth(inlines: &[Inline]) -> usize {
    inlines
        .iter()
        .map(|inline| match &inline.kind {
            InlineKind::Emphasis(children) | InlineKind::Strong(children) => {
                1 + inline_depth(children)
            }
            InlineKind::Link(link) => 1 + inline_depth(&link.children),
            InlineKind::Image(image) => 1 + inline_depth(&image.children),
            _ => 1,
        })
        .max()
        .unwrap_or(0)
}

#[test]
fn nested_block_quotes_past_the_limit_are_text() {
    let source = format!("{} x\n", ">".repeat(20_000));
    let doc = parse_text(&source);
    assert!(support::check_tree(&source, &doc).is_empty());
    assert!(
        block_depth(&doc.blocks) <= MAX_NESTING + 2,
        "{}",
        block_depth(&doc.blocks)
    );
}

#[test]
fn nested_images_past_the_limit_are_text() {
    let n = 20_000;
    let source = format!("{}x{}\n", "![".repeat(n), "]()".repeat(n));
    let doc = parse_text(&source);
    assert!(support::check_tree(&source, &doc).is_empty());
    assert!(
        block_depth(&doc.blocks) <= MAX_NESTING + 2,
        "{}",
        block_depth(&doc.blocks)
    );
}

#[test]
fn nested_emphasis_and_lists_past_the_limit_are_text() {
    let n = 20_000;
    let emphasis = format!("{}x{}\n", "*".repeat(n), "*".repeat(n));
    let doc = parse_text(&emphasis);
    assert!(support::check_tree(&emphasis, &doc).is_empty());
    assert!(block_depth(&doc.blocks) <= MAX_NESTING + 2);

    let lists: String = (0..2_000)
        .map(|level| format!("{}- item\n", "  ".repeat(level)))
        .collect();
    let doc = parse_text(&lists);
    assert!(support::check_tree(&lists, &doc).is_empty());
    assert!(block_depth(&doc.blocks) <= MAX_NESTING + 2);
}

#[test]
fn nesting_within_the_limit_is_kept() {
    let source = format!("{} x\n", ">".repeat(MAX_NESTING - 2));
    let doc = parse_text(&source);
    assert!(support::check_tree(&source, &doc).is_empty());
    // The quotes, the paragraph, and its text.
    assert_eq!(block_depth(&doc.blocks), MAX_NESTING);
}
