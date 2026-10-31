//! ASCRIBE: positions after link reference definitions.
//!
//! Upstream comrak takes link reference definitions from the start of a
//! paragraph (or a setext heading's text) but leaves the node's start
//! position on the definitions, so every inline position in it was a few
//! lines too early. The fork moves the start down to the first line's text, and drops the
//! definitions' lines from the node's line offsets, which the inline parser
//! indexes from that line (two `// ASCRIBE:` hunks in `src/parser/mod.rs`,
//! `finalize_borrowed` and `handle_setext_heading`). These tests fail if a
//! merge from upstream drops that. Both hunks are candidates to send upstream; see `FORK.md`.

use comrak_ascribe::nodes::{Node, NodeValue};
use comrak_ascribe::{Arena, Options, parse_document};

/// The start line of the first node whose value matches `pick`, and of its
/// first text descendant.
fn start_lines(md: &str, pick: impl Fn(&NodeValue) -> bool) -> (usize, usize) {
    let arena = Arena::new();
    let root = parse_document(&arena, md, &Options::default());
    let node: Node<'_> = root
        .descendants()
        .find(|n| pick(&n.data().value))
        .expect("the document has the node");
    let text = node
        .descendants()
        .find(|n| matches!(n.data().value, NodeValue::Text(_)))
        .expect("the node has text");
    (
        node.data().sourcepos.start.line,
        text.data().sourcepos.start.line,
    )
}

fn is_paragraph(v: &NodeValue) -> bool {
    matches!(v, NodeValue::Paragraph)
}

fn is_heading(v: &NodeValue) -> bool {
    matches!(v, NodeValue::Heading(_))
}

#[test]
fn a_paragraph_after_a_one_line_definition_starts_after_it() {
    let md = "[foo]: /url\nbar\n";
    assert_eq!(start_lines(md, is_paragraph), (2, 2));
}

#[test]
fn a_paragraph_after_a_multi_line_definition_starts_after_it() {
    let md = "[foo]:\n  /url\n  \"a title\nthat spans lines\"\nbar\nbaz\n";
    assert_eq!(start_lines(md, is_paragraph), (5, 5));
}

#[test]
fn a_paragraph_after_several_definitions_starts_after_all_of_them() {
    let md = "[a]: /a\n[b]: /b\n[c]: /c\ntext\n";
    assert_eq!(start_lines(md, is_paragraph), (4, 4));
}

#[test]
fn a_setext_heading_after_a_definition_starts_after_it() {
    let md = "[foo]: /url\nbar\n===\n";
    assert_eq!(start_lines(md, is_heading), (2, 2));
}

#[test]
fn a_paragraph_without_definitions_is_unchanged() {
    let md = "\nfirst\nsecond\n";
    assert_eq!(start_lines(md, is_paragraph), (2, 2));
}

/// The start of each text node under the first node `pick` matches.
fn text_starts(md: &str, pick: impl Fn(&NodeValue) -> bool) -> Vec<(usize, usize)> {
    let arena = Arena::new();
    let root = parse_document(&arena, md, &Options::default());
    let node: Node<'_> = root
        .descendants()
        .find(|n| pick(&n.data().value))
        .expect("the document has the node");
    node.descendants()
        .filter(|n| matches!(n.data().value, NodeValue::Text(_)))
        .map(|n| {
            let start = n.data().sourcepos.start;
            (start.line, start.column)
        })
        .collect()
}

#[test]
fn a_paragraph_takes_no_indent_from_a_definitions_lines() {
    let md = "[a]:\n   /x\nb\nc\nd\n";
    assert_eq!(text_starts(md, is_paragraph), [(3, 1), (4, 1), (5, 1)]);
}

#[test]
fn a_setext_heading_takes_no_indent_from_a_definitions_lines() {
    let md = "[a]:\n   /x\nb\nc\n===\n";
    assert_eq!(text_starts(md, is_heading), [(3, 1), (4, 1)]);
}

/// The start (line, column) of the first node `pick` matches.
fn start(md: &str, pick: impl Fn(&NodeValue) -> bool) -> (usize, usize) {
    let arena = Arena::new();
    let root = parse_document(&arena, md, &Options::default());
    let node: Node<'_> = root
        .descendants()
        .find(|n| pick(&n.data().value))
        .expect("the document has the node");
    let start = node.data().sourcepos.start;
    (start.line, start.column)
}

#[test]
fn a_paragraph_after_a_definition_starts_at_its_first_lines_text() {
    // The new first line is indented less, or more, than the definition (#1230).
    assert_eq!(start(" [foo]: /url\nbar\n", is_paragraph), (2, 1));
    assert_eq!(start("[foo]: /url\n   bar\n", is_paragraph), (2, 4));
    assert_eq!(start("> [foo]: /url\n>  bar\n", is_paragraph), (2, 4));
}

#[test]
fn a_setext_heading_after_a_definition_starts_at_its_first_lines_text() {
    assert_eq!(start("[foo]: /url\n  bar\n===\n", is_heading), (2, 3));
}
