//! TESSERA: the phase 04 spike tests for the Tessera-line block.
//!
//! Each test parses a document and compares an outline of the tree: one line
//! per node, indented by depth, with the text of leaves. The cases required
//! by the phase come first, in the phase file's order, followed by the edge
//! cases the spike turned up.

use std::fmt::Write as _;
use std::sync::Arc;

use comrak_tessera::nodes::{Node, NodeValue};
use comrak_tessera::tessera::TesseraOptions;
use comrak_tessera::{Arena, Options, format_commonmark, markdown_to_html, parse_document};

/// The built-in keywords (SPEC §4) plus `end`, and one project widget. Of the
/// built-ins, only `@note` takes a text primary. Later phases supply this set
/// from the content model.
fn keywords() -> TesseraOptions {
    TesseraOptions::new()
        .keyword("id", false)
        .keyword("include", false)
        .keyword("variant", false)
        .keyword("available", false)
        .keyword("note", true)
        .keyword("steps", false)
        .keyword("details", false)
        .keyword("end", false)
        .keyword("quill-demo", true)
}

fn options() -> Options<'static> {
    let mut options = Options::default();
    options.extension.tessera = Some(Arc::new(keywords()));
    options
}

fn outline_with(md: &str, options: &Options) -> String {
    fn walk(node: Node<'_>, depth: usize, out: &mut String) {
        let ast = node.data();
        let label = match &ast.value {
            NodeValue::TesseraLine(t) => format!("tessera_line {:?}", t.raw),
            NodeValue::Text(t) => format!("text {t:?}"),
            NodeValue::Code(c) => format!("code {:?}", c.literal),
            NodeValue::CodeBlock(c) => format!("code_block {:?}", c.literal),
            NodeValue::Heading(h) => format!("heading {}", h.level),
            NodeValue::List(l) => format!("list tight={}", l.tight),
            value => value.xml_node_name().to_string(),
        };
        let _ = writeln!(out, "{}{label}", "  ".repeat(depth));
        for child in node.children() {
            walk(child, depth + 1, out);
        }
    }

    let arena = Arena::new();
    let root = parse_document(&arena, md, options);
    let mut out = String::new();
    walk(root, 0, &mut out);
    out
}

fn outline(md: &str) -> String {
    outline_with(md, &options())
}

#[track_caller]
fn assert_outline(md: &str, expected: &str) {
    let expected = expected.trim_start_matches('\n');
    let actual = outline(md);
    assert_eq!(
        actual, expected,
        "\ninput:\n{md}\nactual:\n{actual}\nexpected:\n{expected}"
    );
}

fn tessera_lines<'a>(root: Node<'a>) -> Vec<Node<'a>> {
    root.descendants()
        .filter(|n| matches!(n.data().value, NodeValue::TesseraLine(..)))
        .collect()
}

// ---------------------------------------------------------------------------
// The phase's required cases (phase 04, task 4)
// ---------------------------------------------------------------------------

/// A directive line directly after a paragraph line starts a new block.
#[test]
fn directive_line_after_paragraph_line_starts_a_new_block() {
    assert_outline(
        "Some text\n@note {type=tip}\nYou can run Quill in the browser.\n",
        r#"
document
  paragraph
    text "Some text"
  tessera_line "@note {type=tip}"
  paragraph
    text "You can run Quill in the browser."
"#,
    );
    // A title line becomes a one-line paragraph the directive interrupts.
    assert_outline(
        ".Try it without installing\n@note {type=tip}\n",
        r#"
document
  paragraph
    text ".Try it without installing"
  tessera_line "@note {type=tip}"
"#,
    );
}

/// `1. Install\n@note` ends the list at `@note`: a directive line is never a
/// lazy continuation line.
#[test]
fn unindented_directive_line_ends_the_list() {
    assert_outline(
        "1. Install\n@note\n",
        r#"
document
  list tight=true
    item
      paragraph
        text "Install"
  tessera_line "@note"
"#,
    );
    // The same holds for a blockquote.
    assert_outline(
        "> Quoted\n@note\n",
        r#"
document
  block_quote
    paragraph
      text "Quoted"
  tessera_line "@note"
"#,
    );
}

/// A directive line indented to a list item's content column belongs to the
/// item.
#[test]
fn directive_line_at_content_column_belongs_to_the_item() {
    assert_outline(
        "1. Install\n   @note: Keep the key safe.\n2. Verify\n",
        r#"
document
  list tight=true
    item
      paragraph
        text "Install"
      tessera_line "@note: Keep the key safe."
        paragraph
          text "Keep the key safe."
    item
      paragraph
        text "Verify"
"#,
    );
    // Nested lists and blockquotes follow the same container rules.
    assert_outline(
        "- a\n  - b\n    @steps\n  @end\n> quote\n> @end\n",
        r#"
document
  list tight=true
    item
      paragraph
        text "a"
      list tight=true
        item
          paragraph
            text "b"
          tessera_line "@steps"
      tessera_line "@end"
  block_quote
    paragraph
      text "quote"
    tessera_line "@end"
"#,
    );
}

/// A directive line indented four or more spaces past the content column is
/// code.
#[test]
fn over_indented_directive_line_is_code() {
    assert_outline(
        "1. Install\n\n       @note: not a directive\n",
        r#"
document
  list tight=false
    item
      paragraph
        text "Install"
      code_block "@note: not a directive\n"
"#,
    );
    assert_outline(
        "    @note: not a directive\n",
        r#"
document
  code_block "@note: not a directive\n"
"#,
    );
    // Directly after a paragraph line, an over-indented line is paragraph
    // continuation text, as in CommonMark. Either way it's literal.
    assert_outline(
        "Some text\n    @note\n",
        r#"
document
  paragraph
    text "Some text"
    softbreak
    text "@note"
"#,
    );
}

/// `@unknown: text` and `@astrojs/react` stay paragraph text.
#[test]
fn unknown_names_stay_paragraph_text() {
    assert_outline(
        "@unknown: text\n",
        r#"
document
  paragraph
    text "@unknown: text"
"#,
    );
    assert_outline(
        "Install\n@astrojs/react\nand @timestamp\n",
        r#"
document
  paragraph
    text "Install"
    softbreak
    text "@astrojs/react"
    softbreak
    text "and @timestamp"
"#,
    );
}

/// `@note {type=caution}: Back up your database` followed by
/// `before you upgrade.` keeps both lines in the primary.
#[test]
fn text_primary_continues_like_a_paragraph() {
    assert_outline(
        "@note {type=caution}: Back up your database\nbefore you upgrade.\n",
        r#"
document
  tessera_line "@note {type=caution}: Back up your database"
    paragraph
      text "Back up your database"
      softbreak
      text "before you upgrade."
"#,
    );
}

/// A directive line inside a fenced code block is code.
#[test]
fn directive_line_in_fenced_code_is_code() {
    assert_outline(
        "```\n@note: not a directive\n@end\n```\n",
        r#"
document
  code_block "@note: not a directive\n@end\n"
"#,
    );
    // So is one in an HTML block, and in a code span.
    assert_outline(
        "<div>\n@note\n</div>\n\n`\n@note`\n",
        r#"
document
  html_block
  paragraph
    code " @note"
"#,
    );
}

// ---------------------------------------------------------------------------
// Text primaries
// ---------------------------------------------------------------------------

#[test]
fn text_primary_ends_where_a_paragraph_would() {
    // A blank line.
    assert_outline(
        "@note: one\ntwo\n\nthree\n",
        r#"
document
  tessera_line "@note: one"
    paragraph
      text "one"
      softbreak
      text "two"
  paragraph
    text "three"
"#,
    );
    // Another directive line, a heading, a fence, and a bullet list.
    assert_outline(
        "@note: one\n@end\n@note: two\n# Heading\n@note: three\n```\ncode\n```\n@note: four\n- item\n",
        r#"
document
  tessera_line "@note: one"
    paragraph
      text "one"
  tessera_line "@end"
  tessera_line "@note: two"
    paragraph
      text "two"
  heading 1
    text "Heading"
  tessera_line "@note: three"
    paragraph
      text "three"
  code_block "code\n"
  tessera_line "@note: four"
    paragraph
      text "four"
  list tight=true
    item
      paragraph
        text "item"
"#,
    );
    // A list that can't interrupt a paragraph can't interrupt a primary.
    assert_outline(
        "@note: The count is\n2. Not a list\n",
        r#"
document
  tessera_line "@note: The count is"
    paragraph
      text "The count is"
      softbreak
      text "2. Not a list"
"#,
    );
}

#[test]
fn text_primary_continues_lazily_like_a_paragraph() {
    assert_outline(
        "> @note: quoted\ncontinued\n\n- @note: in a list\ncontinued\n",
        r#"
document
  block_quote
    tessera_line "@note: quoted"
      paragraph
        text "quoted"
        softbreak
        text "continued"
  list tight=true
    item
      tessera_line "@note: in a list"
        paragraph
          text "in a list"
          softbreak
          text "continued"
"#,
    );
    // An indented line continues the primary instead of starting code.
    assert_outline(
        "@note: one\n        two\n",
        r#"
document
  tessera_line "@note: one"
    paragraph
      text "one"
      softbreak
      text "two"
"#,
    );
}

#[test]
fn text_primary_is_parsed_as_inline_content() {
    assert_outline(
        "@note: *Back up* your [database](db.md)\n",
        r#"
document
  tessera_line "@note: *Back up* your [database](db.md)"
    paragraph
      emph
        text "Back up"
      text " your "
      link
        text "database"
"#,
    );
}

/// SPEC §3.4 (resolved Q2): the primary stays inline content, so a setext
/// underline doesn't make it a heading (`===` continues it, and `---` is a
/// thematic break), and it holds no link reference definitions.
#[test]
fn text_primary_never_becomes_a_block() {
    assert_outline(
        "@note: Title\n===\n@note: Title\n---\n",
        r#"
document
  tessera_line "@note: Title"
    paragraph
      text "Title"
      softbreak
      text "==="
  tessera_line "@note: Title"
    paragraph
      text "Title"
  thematic_break
"#,
    );
    assert_outline(
        "@note: [a]: /url\n",
        r#"
document
  tessera_line "@note: [a]: /url"
    paragraph
      text "[a]: /url"
"#,
    );
    let mut options = options();
    options.extension.table = true;
    assert_eq!(
        outline_with("@note: a | b\n--|--\n", &options),
        r#"document
  tessera_line "@note: a | b"
    paragraph
      text "a | b"
      softbreak
      text "--|--"
"#
    );
}

#[test]
fn only_text_primaries_continue() {
    // A line with no primary, a container opener, and identifier and other
    // non-text primaries all take one line.
    assert_outline(
        "@note {type=tip}\nbound\n@note:\ninside\n@include: a.md\nnext\n@available: cloud, self-managed preview 3.4\nnext\n",
        r#"
document
  tessera_line "@note {type=tip}"
  paragraph
    text "bound"
  tessera_line "@note:"
  paragraph
    text "inside"
  tessera_line "@include: a.md"
  paragraph
    text "next"
  tessera_line "@available: cloud, self-managed preview 3.4"
  paragraph
    text "next"
"#,
    );
    // A project widget with a text primary continues like `@note`.
    assert_outline(
        "@quill-demo: one\ntwo\n",
        r#"
document
  tessera_line "@quill-demo: one"
    paragraph
      text "one"
      softbreak
      text "two"
"#,
    );
}

// ---------------------------------------------------------------------------
// Recognition
// ---------------------------------------------------------------------------

#[test]
fn recognition_needs_a_known_keyword_at_line_start() {
    assert_outline(
        "\\@note: escaped\n\nsupport@example.com\n\n@Note: capitalized\n\n@notes\n\n@note-x: text\n",
        r#"
document
  paragraph
    text "@note: escaped"
  paragraph
    text "support@example.com"
  paragraph
    text "@Note: capitalized"
  paragraph
    text "@notes"
  paragraph
    text "@note-x: text"
"#,
    );
}

/// SPEC §1.5 and §3.9 (resolved Q1): up to three spaces of indentation beyond the
/// container's, as for an ATX heading.
#[test]
fn up_to_three_spaces_of_extra_indentation_are_allowed() {
    assert_outline(
        "   @note\n1. Install\n  @end\n",
        r#"
document
  tessera_line "@note"
  list tight=true
    item
      paragraph
        text "Install"
  tessera_line "@end"
"#,
    );
}

#[test]
fn option_off_or_empty_leaves_commonmark_alone() {
    let md = "Some text\n@note: Back up\n1. Install\n@end\n";
    let plain = r#"document
  paragraph
    text "Some text"
    softbreak
    text "@note: Back up"
  list tight=true
    item
      paragraph
        text "Install"
        softbreak
        text "@end"
"#;
    assert_eq!(outline_with(md, &Options::default()), plain);
    let mut empty = Options::default();
    empty.extension.tessera = Some(Arc::new(TesseraOptions::new()));
    assert_eq!(outline_with(md, &empty), plain);
}

#[test]
fn directive_lines_keep_lists_tight() {
    assert_outline(
        "- a\n  @note\n- b\n",
        r#"
document
  list tight=true
    item
      paragraph
        text "a"
      tessera_line "@note"
    item
      paragraph
        text "b"
"#,
    );
}

#[test]
fn line_endings_and_tabs() {
    assert_outline(
        "@note\t{type=tip}:\tone\r\ntwo\r\n@end\r\n",
        r#"
document
  tessera_line "@note\t{type=tip}:\tone"
    paragraph
      text "one"
      softbreak
      text "two"
  tessera_line "@end"
"#,
    );
}

// ---------------------------------------------------------------------------
// The node
// ---------------------------------------------------------------------------

#[test]
fn node_records_raw_line_name_and_primary() {
    let arena = Arena::new();
    let md = "> @note {type=caution}:  Back up\n> first.\n";
    let root = parse_document(&arena, md, &options());
    let lines = tessera_lines(root);
    assert_eq!(lines.len(), 1);
    let NodeValue::TesseraLine(ref t) = lines[0].data().value else {
        unreachable!()
    };
    assert_eq!(t.raw, "@note {type=caution}:  Back up");
    assert_eq!(t.name, "note");
    assert_eq!(t.text_primary, Some(23));
    assert_eq!(&t.raw[23..], "Back up");
}

#[test]
fn sourcepos_covers_the_line_and_its_primary() {
    let arena = Arena::new();
    let md = "- item\n  @note: Back up\n  first.\n  @end\n";
    let root = parse_document(&arena, md, &options());
    let lines = tessera_lines(root);
    assert_eq!(lines.len(), 2);
    assert_eq!(lines[0].data().sourcepos.to_string(), "2:3-3:8");
    let primary = lines[0].first_child().unwrap();
    assert_eq!(primary.data().sourcepos.to_string(), "2:10-3:8");
    let first_text = primary.first_child().unwrap();
    assert_eq!(first_text.data().sourcepos.to_string(), "2:10-2:16");
    assert_eq!(lines[1].data().sourcepos.to_string(), "4:3-4:6");
}

#[test]
fn renders_to_html_and_commonmark() {
    let md = "Some text\n@note {type=caution}: Back up\nfirst.\n@steps\n";
    assert_eq!(
        markdown_to_html(md, &options()),
        "<p>Some text</p>\n\
         <div data-tessera-line=\"@note {type=caution}: Back up\">\n\
         <p>Back up\nfirst.</p>\n\
         </div>\n\
         <div data-tessera-line=\"@steps\">\n\
         </div>\n"
    );

    let arena = Arena::new();
    let root = parse_document(&arena, md, &options());
    let mut cm = String::new();
    format_commonmark(root, &options(), &mut cm).unwrap();
    assert_eq!(
        cm,
        "Some text\n\n@note {type=caution}: Back up\nfirst.\n\n@steps\n"
    );
    // Formatting is stable: the output parses to the same tree.
    assert_eq!(outline(&cm), outline(md));
}

/// The example in the `tessera` module's documentation (doctests are off in
/// this crate).
#[test]
fn module_example() {
    let mut options = Options::default();
    options.extension.tessera = Some(Arc::new(
        TesseraOptions::new()
            .keyword("note", true)
            .keyword("end", false),
    ));

    let arena = Arena::new();
    let root = parse_document(&arena, "Intro text\n@note: Careful.\n", &options);
    let second = root.last_child().unwrap();
    match &second.data().value {
        NodeValue::TesseraLine(line) => assert_eq!(line.raw, "@note: Careful."),
        other => panic!("expected a Tessera line, got {other:?}"),
    }
}

/// Tessera lines end GFM blocks as headings do, and follow front matter.
#[test]
fn works_alongside_gfm_and_front_matter() {
    let mut options = options();
    options.extension.table = true;
    options.extension.footnotes = true;
    options.extension.front_matter_delimiter = Some("---".into());
    let md = "---\ntitle: T\n---\n@id: intro\n| a |\n|---|\n| b |\n@note: after the table[^1]\n[^1]: A footnote\n@end\n";
    let actual = outline_with(md, &options);
    let expected = r#"document
  frontmatter
  tessera_line "@id: intro"
  table
    table_row
      table_cell
        text "a"
    table_row
      table_cell
        text "b"
  tessera_line "@note: after the table[^1]"
    paragraph
      text "after the table"
      footnote_reference
  tessera_line "@end"
  footnote_definition
    paragraph
      text "A footnote"
"#;
    assert_eq!(actual, expected, "\nactual:\n{actual}");
}
