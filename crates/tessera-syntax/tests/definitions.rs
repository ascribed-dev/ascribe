//! Link reference definitions (CommonMark), which comrak consumes: they are
//! in `ParsedDocument::definitions` with exact spans, and phrases apply in
//! their destinations (SPEC §5.1, resolved Q43).

#![allow(clippy::panic, clippy::expect_used)]

mod support;

use support::check_tree;
use tessera_core::Span;
use tessera_syntax::*;

fn doc(source: &str) -> ParsedDocument {
    let d = parse(source, &ParseOptions::default());
    let problems = check_tree(source, &d);
    assert!(problems.is_empty(), "{problems:?}\n{source:?}");
    d
}

fn cut(source: &str, span: Span) -> &str {
    &source[span.start()..span.end()]
}

/// The text of a definition's parts: whole, label, destination, title.
fn parts<'a>(source: &'a str, d: &LinkDefinition) -> (&'a str, &'a str, &'a str, Option<&'a str>) {
    (
        cut(source, d.span),
        cut(source, d.label),
        cut(source, d.destination),
        d.title.as_ref().map(|t| cut(source, t.span)),
    )
}

#[test]
fn a_definition_without_a_title() {
    let source = "[api]: /docs/api.md\n";
    let d = doc(source);
    assert_eq!(d.definitions.len(), 1);
    let def = &d.definitions[0];
    assert_eq!(
        parts(source, def),
        ("[api]: /docs/api.md", "api", "/docs/api.md", None)
    );
    assert_eq!(def.label_text, "api");
    assert_eq!(def.normalized_label, "api");
    assert_eq!(def.url, "/docs/api.md");
    assert!(def.destination_phrases.is_empty());
}

#[test]
fn every_kind_of_title() {
    let source = "[a]: /x \"double\"\n\n[b]: /y 'single'\n\n[c]: /z (parens)\n";
    let d = doc(source);
    let titles: Vec<_> = d
        .definitions
        .iter()
        .map(|d| {
            let t = d.title.as_ref().expect("has a title");
            (cut(source, t.span), t.text.as_str())
        })
        .collect();
    assert_eq!(
        titles,
        [
            ("\"double\"", "double"),
            ("'single'", "single"),
            ("(parens)", "parens")
        ]
    );
}

#[test]
fn a_definition_with_phrases_and_a_title() {
    let source = "[ref]: {api}streaming \"Streaming {name}\"\n";
    let d = doc(source);
    let def = &d.definitions[0];
    assert_eq!(cut(source, def.destination), "{api}streaming");
    assert_eq!(def.url, "{api}streaming");
    let keys: Vec<_> = def
        .destination_phrases
        .iter()
        .map(|p| (p.key.as_str(), cut(source, p.span), cut(source, p.key_span)))
        .collect();
    assert_eq!(keys, [("api", "{api}", "api")]);
    // Only the destination has candidates: the title is text.
    let title = def.title.as_ref().expect("a title");
    assert_eq!(title.text, "Streaming {name}");
    assert!(d.escaped_phrases.is_empty());
}

#[test]
fn several_phrases_and_a_pointy_destination() {
    let source = "[a]: https://{host}/{path}/x\n\n[b]: <{api}my file>\n";
    let d = doc(source);
    let keys = |i: usize| -> Vec<String> {
        d.definitions[i]
            .destination_phrases
            .iter()
            .map(|p| p.key.clone())
            .collect()
    };
    assert_eq!(keys(0), ["host", "path"]);
    // The angle brackets are part of the destination as written.
    assert_eq!(cut(source, d.definitions[1].destination), "<{api}my file>");
    assert_eq!(d.definitions[1].url, "{api}my file");
    assert_eq!(keys(1), ["api"]);
}

#[test]
fn an_escaped_phrase_is_text() {
    let source = "[ref]: \\{api}streaming\n\n[two]: {api}\\{other}x\n";
    let d = doc(source);
    assert!(d.definitions[0].destination_phrases.is_empty());
    assert_eq!(d.definitions[0].url, "{api}streaming");
    assert_eq!(
        cut(source, d.definitions[0].destination),
        "\\{api}streaming"
    );
    let keys: Vec<_> = d.definitions[1]
        .destination_phrases
        .iter()
        .map(|p| p.key.as_str())
        .collect();
    assert_eq!(keys, ["api"]);
    // Both escapes are listed with the escapes elsewhere, from the backslash.
    let escaped: Vec<_> = d
        .escaped_phrases
        .iter()
        .map(|p| (p.key.as_str(), cut(source, p.span)))
        .collect();
    assert_eq!(escaped, [("api", "\\{api}"), ("other", "\\{other}")]);
}

#[test]
fn an_entity_is_not_a_phrase() {
    let d = doc("[ref]: &#123;api}x\n");
    assert!(d.definitions[0].destination_phrases.is_empty());
}

#[test]
fn a_definition_across_lines() {
    let source = "[a long\n  label]:\n  /url\n  \"a\n  title\"\n\nText.\n";
    let d = doc(source);
    assert_eq!(d.definitions.len(), 1);
    let def = &d.definitions[0];
    assert_eq!(
        parts(source, def),
        (
            "[a long\n  label]:\n  /url\n  \"a\n  title\"",
            "a long\n  label",
            "/url",
            Some("\"a\n  title\"")
        )
    );
    assert_eq!(def.label_text, "a long\n  label");
    assert_eq!(def.normalized_label, "a long label");
    assert_eq!(
        def.title.as_ref().map(|t| t.text.as_str()),
        Some("a\ntitle")
    );
    // The paragraph after it is a block; the definitions aren't.
    assert_eq!(d.blocks.len(), 1);
    assert!(matches!(d.blocks[0].kind, BlockKind::Paragraph(_)));
}

#[test]
fn a_title_on_the_next_line_and_a_title_that_isnt_one() {
    // The title is on its own line.
    let source = "[a]: /url\n\"title\"\n";
    let d = doc(source);
    assert_eq!(parts(source, &d.definitions[0]).3, Some("\"title\""));
    // Text after the title makes it text, not a title, and the definition
    // ends at the destination.
    let source = "[a]: /url\n\"title\" trailing\n";
    let d = doc(source);
    let def = &d.definitions[0];
    assert_eq!(cut(source, def.span), "[a]: /url");
    assert!(def.title.is_none());
    match &d.blocks[0].kind {
        BlockKind::Paragraph(_) => {}
        other => panic!("{other:?}"),
    }
}

#[test]
fn definitions_in_a_row_and_before_a_paragraph() {
    let source = "[a]: /1\n[b]: /2 \"two\"\n[c]: /3\ntext after\n";
    let d = doc(source);
    let all: Vec<_> = d
        .definitions
        .iter()
        .map(|d| (d.label_text.as_str(), d.url.as_str()))
        .collect();
    assert_eq!(all, [("a", "/1"), ("b", "/2"), ("c", "/3")]);
    // The paragraph keeps its own span, after the definitions.
    let BlockKind::Paragraph(_) = &d.blocks[0].kind else {
        panic!("a paragraph");
    };
    assert_eq!(cut(source, d.blocks[0].span), "text after");
}

#[test]
fn indented_up_to_three_spaces() {
    let source = "   [a]: /x\n\n    [b]: /y\n";
    let d = doc(source);
    // Four spaces make an indented code block.
    assert_eq!(d.definitions.len(), 1);
    assert_eq!(cut(source, d.definitions[0].span), "[a]: /x");
}

#[test]
fn definitions_in_list_items() {
    let source = "- item\n\n  [a]: {api}one \"One\"\n\n  [b]:\n  /two\n- next\n\n  [c]: /three\n";
    let d = doc(source);
    let all: Vec<_> = d
        .definitions
        .iter()
        .map(|d| (cut(source, d.span), d.destination_phrases.len()))
        .collect();
    assert_eq!(
        all,
        [
            ("[a]: {api}one \"One\"", 1),
            ("[b]:\n  /two", 0),
            ("[c]: /three", 0)
        ]
    );
    // A definition that starts a list item.
    let source = "1. [a]: /x\n   [b]: {k}y\n";
    let d = doc(source);
    assert_eq!(d.definitions.len(), 2);
    assert_eq!(cut(source, d.definitions[1].destination), "{k}y");
    assert_eq!(d.definitions[1].destination_phrases[0].key, "k");
}

#[test]
fn definitions_in_block_quotes() {
    let source = "> [a]: {api}one\n> [b]: /two\n> \"Two\"\n>\n> > [c]: \\{x}\n";
    let d = doc(source);
    let all: Vec<_> = d.definitions.iter().map(|d| parts(source, d)).collect();
    assert_eq!(
        all,
        [
            ("[a]: {api}one", "a", "{api}one", None),
            ("[b]: /two\n> \"Two\"", "b", "/two", Some("\"Two\"")),
            ("[c]: \\{x}", "c", "\\{x}", None),
        ]
    );
    assert_eq!(d.definitions[0].destination_phrases[0].key, "api");
    assert_eq!(d.escaped_phrases.len(), 1);
    // A lazy continuation line of a quoted definition.
    let source = "> [a]:\n/lazy\n";
    let d = doc(source);
    assert_eq!(d.definitions.len(), 1);
    assert_eq!(cut(source, d.definitions[0].destination), "/lazy");
}

#[test]
fn definitions_in_directive_containers_and_after_directive_lines() {
    let source = "@note:\n[a]: {api}x\n\nText.\n@end\n\n@available: cloud\n[b]: /y\n";
    let d = doc(source);
    assert_eq!(d.definitions.len(), 2);
    assert_eq!(d.definitions[0].destination_phrases[0].key, "api");
    assert_eq!(cut(source, d.definitions[1].span), "[b]: /y");
}

#[test]
fn a_text_primary_has_no_definitions() {
    // SPEC §3.4: a leading `[label]: /url` in a text primary is text.
    let d = doc("@note: [a]: {api}x\n");
    assert!(d.definitions.is_empty());
}

#[test]
fn code_and_html_have_no_definitions() {
    let d = doc("```\n[a]: /x\n```\n\n    [b]: /y\n\n<div>\n[c]: /z\n</div>\n");
    assert!(d.definitions.is_empty());
}

#[test]
fn what_is_not_a_definition() {
    // No destination, an empty label, or text after the destination.
    for source in [
        "[a]:\n",
        "[]: /x\n",
        "[a]: /x y\n",
        "[a] /x\n",
        "[a]:/x\u{a0}z y\n",
    ] {
        let d = doc(source);
        assert!(d.definitions.is_empty(), "{source:?}");
    }
    // A paragraph that isn't only definitions keeps the rest.
    let d = doc("[a]: /x\nnot a definition\n[b]: /y\n");
    assert_eq!(d.definitions.len(), 1);
}

#[test]
fn a_later_definition_of_a_label_is_kept() {
    let source = "[Foo]: /first\n\n[FOO]: /second\n\n[foo]\n";
    let d = doc(source);
    assert_eq!(d.definitions.len(), 2);
    assert_eq!(
        d.definitions[0].normalized_label,
        d.definitions[1].normalized_label
    );
    // A link uses the first, as CommonMark says.
    let BlockKind::Paragraph(p) = &d.blocks[0].kind else {
        panic!("a paragraph");
    };
    let InlineKind::Link(link) = &p.inlines[0].kind else {
        panic!("a link");
    };
    assert_eq!(link.destination, "/first");
}

#[test]
fn definitions_are_in_source_order_and_utf8_is_intact() {
    let source = "é\n\n> - [é →]: /π \"😀\"\n\n[z]: /z\n";
    let d = doc(source);
    let all: Vec<_> = d.definitions.iter().map(|d| parts(source, d)).collect();
    assert_eq!(
        all,
        [
            ("[é →]: /π \"😀\"", "é →", "/π", Some("\"😀\"")),
            ("[z]: /z", "z", "/z", None)
        ]
    );
}

#[test]
fn crlf_and_cr_line_endings() {
    for eol in ["\r\n", "\r"] {
        let source = format!("[a]:{eol}{{api}}x{eol}\"t\"{eol}{eol}[b]: /y{eol}");
        let d = doc(&source);
        assert_eq!(d.definitions.len(), 2, "{eol:?}");
        assert_eq!(
            cut(&source, d.definitions[0].destination),
            "{api}x",
            "{eol:?}"
        );
        assert_eq!(d.definitions[0].destination_phrases.len(), 1);
    }
}

#[test]
fn definitions_after_frontmatter() {
    let source = "---\ntitle: T\n---\n\n[a]: {api}x\n";
    let d = doc(source);
    assert_eq!(cut(source, d.definitions[0].span), "[a]: {api}x");
}

#[test]
fn blocks_have_no_definition_kind() {
    // Definitions are a side list: a document of only definitions has no blocks.
    let d = doc("[a]: /x\n\n[b]: /y\n");
    assert!(d.blocks.is_empty());
    assert_eq!(d.definitions.len(), 2);
}
