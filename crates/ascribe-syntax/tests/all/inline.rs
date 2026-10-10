//! Phrase candidates and image attribute blocks (SPEC §2.3, §5.1,
//! §5.3), with exact spans, in every place the spec allows them.

#![allow(clippy::panic, clippy::expect_used)]

use crate::support;

use ascribe_core::Span;
use ascribe_syntax::*;
use support::check_tree;

fn doc(source: &str) -> ParsedDocument {
    let d = parse(source, &ParseOptions::default());
    let problems = check_tree(source, &d);
    assert!(problems.is_empty(), "{problems:?}\n{source:?}");
    d
}

fn cut(source: &str, span: Span) -> &str {
    &source[span.start()..span.end()]
}

fn slugs(d: &ParsedDocument) -> Vec<&'static str> {
    d.issues.iter().map(|i| i.slug.as_str()).collect()
}

/// The inlines of the first paragraph.
fn paragraph(d: &ParsedDocument) -> &[Inline] {
    match &d.blocks[0].kind {
        BlockKind::Paragraph(p) => &p.inlines,
        other => panic!("not a paragraph: {other:?}"),
    }
}

/// The keys of every phrase candidate in `inlines`, at any depth.
fn keys(inlines: &[Inline]) -> Vec<String> {
    let mut out = Vec::new();
    for inline in inlines {
        match &inline.kind {
            InlineKind::Phrase(p) => out.push(p.key.clone()),
            InlineKind::Emphasis(c) | InlineKind::Strong(c) => out.extend(keys(c)),
            InlineKind::Link(l) => out.extend(keys(&l.children)),
            InlineKind::Image(i) => out.extend(keys(&i.children)),
            _ => {}
        }
    }
    out
}

/// The text of `inlines`: text values, with each phrase as it was written.
fn flatten(source: &str, inlines: &[Inline]) -> String {
    inlines
        .iter()
        .map(|i| match &i.kind {
            InlineKind::Text(t) => t.clone(),
            InlineKind::Phrase(_) => cut(source, i.span).to_owned(),
            _ => String::new(),
        })
        .collect()
}

// -- phrases ------------------------------------------------------------------

#[test]
fn a_candidate_in_prose_is_its_own_node_with_exact_spans() {
    let source = "Sign in to {cloud} and copy {api-key2}'s value.\n";
    let d = doc(source);
    let inlines = paragraph(&d);
    assert_eq!(inlines.len(), 5);
    let InlineKind::Phrase(p) = &inlines[1].kind else {
        panic!("{inlines:?}");
    };
    assert_eq!(p.key, "cloud");
    assert_eq!(cut(source, p.span), "{cloud}");
    assert_eq!(cut(source, p.key_span), "cloud");
    assert_eq!(cut(source, inlines[1].span), "{cloud}");
    assert!(matches!(&inlines[0].kind, InlineKind::Text(t) if t == "Sign in to "));
    assert!(matches!(&inlines[2].kind, InlineKind::Text(t) if t == " and copy "));
    assert!(matches!(&inlines[4].kind, InlineKind::Text(t) if t == "'s value."));
    assert_eq!(keys(inlines), ["cloud", "api-key2"]);
    assert!(d.issues.is_empty());
}

#[test]
fn braces_delimit_the_phrase_against_punctuation_letters_and_hyphens() {
    let d = doc("a{b}c {d}-{e}. ({f})\n");
    assert_eq!(keys(paragraph(&d)), ["b", "d", "e", "f"]);
}

#[test]
fn only_the_key_syntax_makes_a_candidate() {
    // Appendix A: a lowercase letter, then lowercase letters, digits, `-`.
    let d = doc("{} {A} {1a} {-a} {a b} {a_b} {a.b} {a=b} { a } {a\n");
    assert!(keys(paragraph(&d)).is_empty());
    assert_eq!(
        flatten("", paragraph(&d)),
        "{} {A} {1a} {-a} {a b} {a_b} {a.b} {a=b} { a } {a"
    );
}

#[test]
fn candidates_in_headings_link_text_emphasis_and_alt_text() {
    let source = "# The {product} guide\n\n[The {api} docs](x.md) **{a}** ![{b} logo](l.png)\n";
    let d = doc(source);
    let BlockKind::Heading(h) = &d.blocks[0].kind else {
        panic!("{:?}", d.blocks);
    };
    assert_eq!(keys(&h.inlines), ["product"]);
    let BlockKind::Paragraph(p) = &d.blocks[1].kind else {
        panic!("{:?}", d.blocks);
    };
    assert_eq!(keys(&p.inlines), ["api", "a", "b"]);
}

#[test]
fn candidates_in_text_primaries_lists_quotes_and_table_cells() {
    let source = "@note: Use {a},\nthen {b}.\n\n- item {c}\n\n> quoted {d}\n> {e}\n\n\
                  | h {f} | g |\n|---|---|\n| {g} \\| {h} | x |\n";
    let d = doc(source);
    let mut all = Vec::new();
    fn walk(blocks: &[Block], out: &mut Vec<String>) {
        for block in blocks {
            match &block.kind {
                BlockKind::Directive(line) => {
                    if let Some(PrimaryValue::Text(p)) = &line.primary {
                        out.extend(keys(&p.inlines));
                    }
                }
                BlockKind::Paragraph(p) => out.extend(keys(&p.inlines)),
                BlockKind::BlockQuote(q) => walk(&q.children, out),
                BlockKind::List(l) => l.items.iter().for_each(|i| walk(&i.children, out)),
                BlockKind::Table(t) => {
                    for cell in t.rows.iter().flat_map(|r| &r.cells) {
                        out.extend(keys(&cell.inlines));
                    }
                }
                _ => {}
            }
        }
    }
    walk(&d.blocks, &mut all);
    assert_eq!(all, ["a", "b", "c", "d", "e", "f", "g", "h"]);
}

/// The keys of every candidate in every paragraph, heading, title line, and
/// text primary of `blocks`, in order, at any depth. A title line is
/// `DirectiveLine::title` (SPEC §3.7); an arm's `title` is its opener's.
fn all_keys(blocks: &[Block]) -> Vec<String> {
    fn line_keys(line: &DirectiveLine) -> Vec<String> {
        let mut out = Vec::new();
        if let Some(title) = &line.title {
            out.extend(keys(&title.inlines));
        }
        if let Some(PrimaryValue::Text(p)) = &line.primary {
            out.extend(keys(&p.inlines));
        }
        out
    }
    let mut out = Vec::new();
    for block in blocks {
        match &block.kind {
            BlockKind::Paragraph(p) => out.extend(keys(&p.inlines)),
            BlockKind::Heading(h) => out.extend(keys(&h.inlines)),
            BlockKind::Directive(line) => out.extend(line_keys(line)),
            BlockKind::Container(c) => {
                out.extend(line_keys(&c.opener));
                out.extend(all_keys(&c.children));
            }
            BlockKind::Group(g) => {
                for arm in &g.arms {
                    out.extend(line_keys(&arm.opener));
                    out.extend(all_keys(&arm.children));
                }
            }
            BlockKind::BlockQuote(q) => out.extend(all_keys(&q.children)),
            BlockKind::List(l) => l
                .items
                .iter()
                .for_each(|i| out.extend(all_keys(&i.children))),
            _ => {}
        }
    }
    out
}

#[test]
fn a_candidate_in_a_title_line_is_found() {
    let source = ".Install {product} now\n@note: Body with {body}.\n";
    let d = doc(source);
    assert_eq!(all_keys(&d.blocks), ["product", "body"]);
}

#[test]
fn a_candidate_in_an_arm_title_is_found() {
    let source = ".Using {product} on {os}\n@variant {os=linux}:\nLinux text.\n\n\
                  .Other {tool}\n@variant {os=macos}:\nmacOS text.\n@end\n";
    let d = doc(source);
    assert_eq!(all_keys(&d.blocks), ["product", "os", "tool"]);
}

#[test]
fn a_candidate_after_an_escaped_pipe_in_a_table_cell_has_the_right_span() {
    let source = "| a |\n|---|\n| x \\| {k} |\n";
    let d = doc(source);
    let BlockKind::Table(t) = &d.blocks[0].kind else {
        panic!("{:?}", d.blocks);
    };
    let inlines = &t.rows[1].cells[0].inlines;
    let phrase = inlines
        .iter()
        .find(|i| matches!(i.kind, InlineKind::Phrase(_)))
        .expect("a phrase");
    assert_eq!(cut(source, phrase.span), "{k}");
}

#[test]
fn an_escaped_brace_is_literal_text_and_is_recorded() {
    let source = "\\{key} and \\{other} and {real}\n";
    let d = doc(source);
    let inlines = paragraph(&d);
    assert_eq!(keys(inlines), ["real"]);
    assert!(matches!(&inlines[0].kind, InlineKind::Text(t) if t == "{key} and {other} and "));
    // The text's span includes the backslash.
    assert_eq!(inlines[0].span.start(), 0);
    let escaped: Vec<_> = d
        .escaped_phrases
        .iter()
        .map(|p| cut(source, p.span))
        .collect();
    assert_eq!(escaped, ["\\{key}", "\\{other}"]);
    assert_eq!(d.escaped_phrases[0].key, "key");
    assert_eq!(cut(source, d.escaped_phrases[0].key_span), "key");
}

#[test]
fn an_escaped_backslash_does_not_escape_the_brace() {
    let source = "\\\\{key}\n";
    let d = doc(source);
    assert_eq!(keys(paragraph(&d)), ["key"]);
    assert!(d.escaped_phrases.is_empty());
    let InlineKind::Text(t) = &paragraph(&d)[0].kind else {
        panic!()
    };
    assert_eq!(t, "\\");
}

#[test]
fn an_escaped_brace_stays_escaped_in_headings_and_link_text() {
    let source = "## \\{a}\n\n[\\{b}](x.md)\n";
    let d = doc(source);
    assert_eq!(d.escaped_phrases.len(), 2);
    let BlockKind::Heading(h) = &d.blocks[0].kind else {
        panic!()
    };
    assert!(keys(&h.inlines).is_empty());
}

#[test]
fn an_entity_that_decodes_to_a_brace_is_not_a_candidate() {
    let source = "&#123;key} &lbrace;key} {yes}\n";
    let d = doc(source);
    assert_eq!(keys(paragraph(&d)), ["yes"]);
    let InlineKind::Text(t) = &paragraph(&d)[0].kind else {
        panic!()
    };
    assert!(t.starts_with("{key} {key} "));
}

#[test]
fn text_around_a_candidate_keeps_its_decoded_value() {
    let source = "&copy; \\*x\\* &amp; {k} \\_ &#35;\n";
    let d = doc(source);
    let inlines = paragraph(&d);
    assert!(matches!(&inlines[0].kind, InlineKind::Text(t) if t == "© *x* & "));
    assert!(matches!(&inlines[2].kind, InlineKind::Text(t) if t == " _ #"));
    assert_eq!(cut(source, inlines[0].span), "&copy; \\*x\\* &amp; ");
}

#[test]
fn code_spans_and_html_never_hold_candidates() {
    let source = "`{a}` ``{b}`` <span title=\"{c}\">{d}</span> <a href=\"{e}\">\n";
    let d = doc(source);
    assert_eq!(keys(paragraph(&d)), ["d"]);
}

#[test]
fn indented_code_and_html_blocks_never_hold_candidates() {
    let d = doc("    {a}\n\n<div>\n{b}\n</div>\n");
    assert!(d.blocks.iter().all(|b| match &b.kind {
        BlockKind::CodeBlock(c) => c.phrases.is_none(),
        BlockKind::Paragraph(_) => false,
        _ => true,
    }));
}

#[test]
fn a_fence_holds_candidates_only_with_phrases_true() {
    let source = "```yaml\nurl: {api}\n```\n\n\
                  ```yaml phrases=true\nurl: {api}streaming\nname: {cloud}\n```\n\n\
                  ```phrases=false\n{x}\n```\n\n\
                  ~~~ phrases=true extra\n{tilde}\n~~~\n\n\
                  ```notphrases=true\n{y}\n```\n\n\
                  ```\n{z}\n```\n";
    let d = doc(source);
    let phrases: Vec<Option<Vec<&str>>> = d
        .blocks
        .iter()
        .map(|b| match &b.kind {
            BlockKind::CodeBlock(c) => c
                .phrases
                .as_ref()
                .map(|p| p.iter().map(|p| cut(source, p.span)).collect()),
            _ => None,
        })
        .collect();
    assert_eq!(
        phrases,
        [
            None,
            Some(vec!["{api}", "{cloud}"]),
            None,
            Some(vec!["{tilde}"]),
            None,
            None
        ]
    );
}

#[test]
fn a_fence_that_opts_in_can_hold_no_candidates_and_the_info_string_is_not_code() {
    let d = doc("```yaml phrases=true\nplain {Not} {a b}\n```\n");
    let BlockKind::CodeBlock(c) = &d.blocks[0].kind else {
        panic!()
    };
    assert_eq!(c.phrases, Some(Vec::new()));
    let d = doc("```{info} phrases=true\ncode\n```\n");
    let BlockKind::CodeBlock(c) = &d.blocks[0].kind else {
        panic!()
    };
    assert_eq!(c.phrases, Some(Vec::new()));
}

#[test]
fn a_fence_in_a_list_and_a_quote_has_exact_spans() {
    let source = "- item\n\n  ```yaml phrases=true\n  a: {one}\n  ```\n\n> ```phrases=true\n> {two}\n> ```\n";
    let d = doc(source);
    let mut found = Vec::new();
    fn walk<'a>(source: &'a str, blocks: &[Block], out: &mut Vec<&'a str>) {
        for block in blocks {
            match &block.kind {
                BlockKind::CodeBlock(c) => {
                    out.extend(c.phrases.iter().flatten().map(|p| cut(source, p.span)));
                }
                BlockKind::BlockQuote(q) => walk(source, &q.children, out),
                BlockKind::List(l) => l.items.iter().for_each(|i| walk(source, &i.children, out)),
                _ => {}
            }
        }
    }
    walk(source, &d.blocks, &mut found);
    assert_eq!(found, ["{one}", "{two}"]);
}

// SPEC §5.1: backslashes don't escape in code.
#[test]
fn a_backslash_does_not_escape_in_a_fence() {
    let d = doc("```phrases=true\n\\{a}\n```\n");
    let BlockKind::CodeBlock(c) = &d.blocks[0].kind else {
        panic!()
    };
    assert_eq!(c.phrases.as_ref().map(Vec::len), Some(1));
    assert!(d.escaped_phrases.is_empty());
}

// -- link destinations ----------------------------------------------------------

fn only_link(d: &ParsedDocument) -> &Link {
    paragraph(d)
        .iter()
        .find_map(|i| match &i.kind {
            InlineKind::Link(l) => Some(l),
            _ => None,
        })
        .expect("a link")
}

#[test]
fn candidates_in_a_destination_are_recorded_and_the_destination_is_unchanged() {
    let source = "See the [streaming API reference]({api}streaming).\n";
    let d = doc(source);
    let link = only_link(&d);
    assert_eq!(link.destination, "{api}streaming");
    assert_eq!(link.form, LinkForm::Inline);
    let [p] = link.destination_phrases.as_slice() else {
        panic!("{link:?}");
    };
    assert_eq!(p.key, "api");
    assert_eq!(cut(source, p.span), "{api}");
    assert_eq!(cut(source, p.key_span), "api");
    // The link text and the text around the link don't hold it.
    assert!(keys(paragraph(&d)).is_empty());
}

#[test]
fn destinations_of_every_shape() {
    for (source, expected) in [
        ("[t]({a}x \"title {b}\")\n", vec!["{a}"]),
        ("[t](<{a} x>)\n", vec!["{a}"]),
        ("[t](  {a}x{b}  )\n", vec!["{a}", "{b}"]),
        ("[t](x(y){z}) w\n", vec!["{z}"]),
        ("[t](\\{a}x)\n", vec![]),
        ("[t]()\n", vec![]),
        ("[t][r]\n\n[r]: {a}x\n", vec![]),
        ("[t]\n\n[t]: {a}x\n", vec![]),
        // SPEC §5.1: an autolink's destination holds
        // candidates, and a backslash doesn't escape there.
        ("<https://x.org/{a}>\n", vec!["{a}"]),
        ("<https://{host}/x/{b}>\n", vec!["{host}", "{b}"]),
        ("<https://x.org/\\{a}>\n", vec!["{a}"]),
    ] {
        let d = doc(source);
        let found: Vec<_> = paragraph(&d)
            .iter()
            .flat_map(|i| match &i.kind {
                InlineKind::Link(l) => l.destination_phrases.clone(),
                InlineKind::Image(i) => i.destination_phrases.clone(),
                _ => Vec::new(),
            })
            .map(|p| cut(source, p.span).to_owned())
            .collect();
        assert_eq!(found, expected, "{source:?}");
    }
}

#[test]
fn an_image_destination_and_an_escaped_destination_brace() {
    let source = "![alt]({assets}a.png) [t](\\{a}b)\n";
    let d = doc(source);
    let images: Vec<_> = paragraph(&d)
        .iter()
        .filter_map(|i| match &i.kind {
            InlineKind::Image(i) => Some(i),
            _ => None,
        })
        .collect();
    assert_eq!(images[0].destination_phrases[0].key, "assets");
    assert_eq!(images[0].destination, "{assets}a.png");
    assert_eq!(d.escaped_phrases.len(), 1);
    assert_eq!(cut(source, d.escaped_phrases[0].span), "\\{a}");
}

// -- image attributes -------------------------------------------------------------

fn only_image(d: &ParsedDocument) -> &Image {
    paragraph(d)
        .iter()
        .find_map(|i| match &i.kind {
            InlineKind::Image(i) => Some(i),
            _ => None,
        })
        .expect("an image")
}

#[test]
fn an_attribute_block_attaches_to_every_form_of_image() {
    for (source, form) in [
        ("![alt](s.png){width=600}\n", LinkForm::Inline),
        ("![alt][r]{width=600}\n\n[r]: s.png\n", LinkForm::Full),
        ("![alt][]{width=600}\n\n[alt]: s.png\n", LinkForm::Collapsed),
        ("![alt]{width=600}\n\n[alt]: s.png\n", LinkForm::Shortcut),
    ] {
        let d = doc(source);
        let image = only_image(&d);
        assert_eq!(image.form, form, "{source:?}");
        assert_eq!(image.destination, "s.png", "{source:?}");
        let attributes = image.attributes.as_ref().expect("attributes");
        assert_eq!(cut(source, attributes.block.span), "{width=600}");
        let pair = &attributes.block.attributes[0];
        assert_eq!(pair.key, "width");
        assert_eq!(cut(source, pair.key_span), "width");
        assert_eq!(
            cut(source, pair.value.as_ref().expect("value").span()),
            "600"
        );
        assert!(d.issues.is_empty(), "{source:?}");
        // The image covers its block, and nothing follows it.
        let inline = &paragraph(&d)[0];
        assert!(cut(source, inline.span).ends_with("{width=600}"));
        assert_eq!(paragraph(&d).len(), 1, "{source:?}");
    }
}

#[test]
fn attributes_use_the_directive_grammar() {
    let source = "![a](s.png \"Title\"){width=\"6 0\", height = 400 , loading=eager|lazy}\n";
    let d = doc(source);
    let image = only_image(&d);
    assert_eq!(image.title.as_deref(), Some("Title"));
    let block = &image.attributes.as_ref().expect("attributes").block;
    let keys: Vec<_> = block.attributes.iter().map(|a| a.key.as_str()).collect();
    assert_eq!(keys, ["width", "height", "loading"]);
    assert_eq!(
        block.attributes[0].value.as_ref().and_then(|v| v.as_text()),
        Some("6 0")
    );
    assert!(d.issues.is_empty());
}

#[test]
fn an_empty_block_is_an_attribute_block() {
    let d = doc("![a](s.png){} and ![b](t.png){ }\n");
    let images: Vec<_> = paragraph(&d)
        .iter()
        .filter_map(|i| match &i.kind {
            InlineKind::Image(i) => Some(i),
            _ => None,
        })
        .collect();
    assert!(images.iter().all(|i| {
        i.attributes
            .as_ref()
            .is_some_and(|a| a.block.attributes.is_empty())
    }));
    assert!(d.issues.is_empty());
}

#[test]
fn a_block_after_a_space_or_after_anything_but_an_image_is_text() {
    for source in [
        "![a](s.png) {width=600}\n",
        "[a](s.png){width=600}\n",
        "text{width=600}\n",
        "**bold**{width=600}\n",
        "`code`{width=600}\n",
        "![a](s.png)\n{width=600}\n",
        "![undefined]{width=600}\n",
    ] {
        let d = doc(source);
        assert!(d.issues.is_empty(), "{source:?}");
        let attached = paragraph(&d)
            .iter()
            .any(|i| matches!(&i.kind, InlineKind::Image(image) if image.attributes.is_some()));
        assert!(!attached, "{source:?}");
        assert!(keys(paragraph(&d)).is_empty(), "{source:?}");
    }
}

#[test]
fn the_block_is_not_parsed_as_markdown() {
    let source = "![a](s.png){title=\"*x\", alt=\"[y](z)\", k=`v`} *emphasis* here\n";
    let d = doc(source);
    let inlines = paragraph(&d);
    assert!(matches!(&inlines[0].kind, InlineKind::Image(_)));
    assert!(matches!(&inlines[2].kind, InlineKind::Emphasis(_)));
    assert!(d.issues.is_empty(), "{:?}", slugs(&d));
    assert_eq!(inlines.len(), 4);
}

#[test]
fn an_image_with_a_block_in_a_link_a_list_and_a_quote() {
    let source = "[![a](i.png){w=1}](x.md)\n\n- ![b](j.png){w=2}\n\n> ![c][r]{w=3}\n\n[r]: k.png\n";
    let d = doc(source);
    assert!(d.issues.is_empty());
    let BlockKind::Paragraph(p) = &d.blocks[0].kind else {
        panic!()
    };
    let InlineKind::Link(link) = &p.inlines[0].kind else {
        panic!()
    };
    let InlineKind::Image(image) = &link.children[0].kind else {
        panic!("{link:?}");
    };
    assert!(image.attributes.is_some());
    assert_eq!(cut(source, link.children[0].span), "![a](i.png){w=1}");
}

#[test]
fn errors_in_an_image_block_are_reported_and_the_block_is_kept() {
    let source = "![a](s.png){width}\n\n![b](s.png){width=}\n\n![c](s.png){w=1, w=2}\n\n\
                  ![d][r]{size=3, a=b c}\n\n[r]: s.png\n";
    let d = doc(source);
    assert_eq!(
        slugs(&d),
        [
            "attribute-bare-key",
            "attribute-syntax",
            "attribute-duplicate-key",
            "attribute-unquoted-reserved"
        ]
    );
    let lines: Vec<_> = d
        .issues
        .iter()
        .map(|i| source[..i.location.span.start()].matches('\n').count() + 1)
        .collect();
    assert_eq!(lines, [1, 3, 5, 7]);
    for block in &d.blocks {
        if let BlockKind::Paragraph(p) = &block.kind {
            assert!(only_image_of(&p.inlines).attributes.is_some());
        }
    }
}

fn only_image_of(inlines: &[Inline]) -> &Image {
    match &inlines[0].kind {
        InlineKind::Image(i) => i,
        other => panic!("{other:?}"),
    }
}

// SPEC §5.1, §5.3
#[test]
fn a_bare_key_directly_after_an_image_is_an_attribute_block() {
    let source = "![a](s.png){product}\n";
    let d = doc(source);
    assert!(keys(paragraph(&d)).is_empty());
    assert_eq!(slugs(&d), ["attribute-bare-key"]);
    assert!(only_image(&d).attributes.is_some());
}

#[test]
fn an_unclosed_block_is_reported_and_stays_text() {
    let source = "![a](s.png){width=600 and more\n";
    let d = doc(source);
    assert_eq!(slugs(&d)[0], "attribute-syntax");
    assert!(only_image(&d).attributes.is_none());
    let InlineKind::Text(t) = &paragraph(&d)[1].kind else {
        panic!()
    };
    assert_eq!(t, "{width=600 and more");
}

#[test]
fn a_block_never_continues_onto_the_next_line() {
    let source = "![a](s.png){width=600,\nheight=400}\n";
    let d = doc(source);
    assert_eq!(slugs(&d)[0], "attribute-syntax");
    assert!(only_image(&d).attributes.is_none());
}

#[test]
fn an_image_after_a_directive_line_and_in_a_text_primary() {
    let source = "@note: See ![a](s.png){w=1} and {p}.\n";
    let d = doc(source);
    let BlockKind::Directive(line) = &d.blocks[0].kind else {
        panic!()
    };
    let Some(PrimaryValue::Text(p)) = &line.primary else {
        panic!()
    };
    assert_eq!(keys(&p.inlines), ["p"]);
    assert!(
        p.inlines
            .iter()
            .any(|i| matches!(&i.kind, InlineKind::Image(i) if i.attributes.is_some()))
    );
}

// -- the fork agrees --------------------------------------------------------------

#[test]
fn the_fork_and_the_tree_agree_on_where_a_block_ends() {
    // If the fork skipped a different length than the tree reads, text would
    // be lost or repeated after the block.
    for tail in ["", " and more", "{p}", "\\{q}", "}"] {
        for block in [
            "{w=1}",
            "{a=\"}\"}",
            "{a=\"\\\"}\"}",
            "{a=b,c=d}",
            "{}",
            "{ a = b | c }",
        ] {
            let source = format!("x ![a](s.png){block}{tail}\n");
            let d = doc(&source);
            let flat = flatten(&source, paragraph(&d));
            let expect_tail = tail.replace("\\{", "{");
            assert_eq!(flat, format!("x {expect_tail}"), "{source:?}");
        }
    }
}

#[test]
fn an_arms_title_stays_the_same_as_its_openers_after_the_inline_pass() {
    let source = ".Using {product}\n@variant {os=linux}:\nText.\n\n.Other\n@variant {os=macos}:\nText.\n@end\n";
    let d = doc(source);
    let BlockKind::Group(g) = &d.blocks[0].kind else {
        panic!("{:?}", d.blocks);
    };
    assert!(g.arms.iter().all(|a| a.title == a.opener.title));
    assert_eq!(
        g.arms[0].title.as_ref().map(|t| keys(&t.inlines)),
        Some(vec!["product".to_owned()])
    );
}

/// The table's rows, after the inline pass.
fn rows(d: &ParsedDocument) -> &[TableRow] {
    match &d.blocks[0].kind {
        BlockKind::Table(t) => &t.rows,
        other => panic!("not a table: {other:?}"),
    }
}

#[test]
fn a_block_at_the_end_of_a_rows_first_cell_is_the_rows() {
    let source = "| Key | Meaning |\n|---|---|\n| `anchors` {available=\"cloud, self-managed preview 3.4\"} | Marks blocks. |\n";
    let d = doc(source);
    assert!(d.issues.is_empty(), "{:?}", d.issues);
    let row = &rows(&d)[1];
    let block = row.attributes.as_ref().expect("a block");
    assert_eq!(
        cut(source, block.span),
        "{available=\"cloud, self-managed preview 3.4\"}"
    );
    assert_eq!(
        block
            .get("available")
            .and_then(|a| a.value.as_ref()?.as_text()),
        Some("cloud, self-managed preview 3.4")
    );
    // The cell keeps its span, and its content ends before the block.
    let cell = &row.cells[0];
    assert!(cut(source, cell.span).contains("{available"));
    assert_eq!(cell.inlines.len(), 1);
    assert_eq!(cut(source, cell.inlines[0].span), "`anchors`");
}

#[test]
fn a_cell_that_is_only_a_block_is_left_empty() {
    let source = "| a | b |\n|---|---|\n| {available=next} | x |\n";
    let d = doc(source);
    let row = &rows(&d)[1];
    assert!(row.attributes.is_some());
    assert!(row.cells[0].inlines.is_empty());
}

#[test]
fn text_before_the_block_loses_only_its_trailing_space() {
    let source = "| a |\n|---|\n| The *new* key  {available=next} |\n";
    let d = doc(source);
    let cell = &rows(&d)[1].cells[0];
    let last = cell.inlines.last().expect("inlines");
    assert!(
        matches!(&last.kind, InlineKind::Text(t) if t == " key"),
        "{last:?}"
    );
    assert_eq!(cut(source, last.span), " key");
}

#[test]
fn braces_that_arent_a_rows_block_stay_in_the_cell() {
    for (cell, phrases) in [
        // A phrase candidate, not a block.
        ("x {next}", vec!["next"]),
        // Not at the end of the cell.
        ("{available=next} x", vec![]),
        // Not after a space.
        ("x{available=next}", vec![]),
        // In code.
        ("`{available=next}`", vec![]),
        // Not the first cell.
        ("x | {available=next}", vec![]),
    ] {
        let source = format!("| a | b |\n|---|---|\n| {cell} |\n");
        let d = doc(&source);
        let row = &rows(&d)[1];
        assert!(row.attributes.is_none(), "{cell:?}");
        assert_eq!(keys(&row.cells[0].inlines), phrases, "{cell:?}");
    }
}

#[test]
fn the_header_row_has_no_block() {
    let source = "| a {available=next} |\n|---|\n| x |\n";
    let d = doc(source);
    assert!(rows(&d)[0].attributes.is_none());
}

#[test]
fn an_images_block_in_a_first_cell_is_the_images() {
    let source = "| a |\n|---|\n| ![x](s.png){width=1} |\n| ![x](s.png) {available=next} |\n";
    let d = doc(source);
    let rows = rows(&d);
    assert!(rows[1].attributes.is_none());
    assert!(rows[2].attributes.is_some());
    let InlineKind::Image(image) = &rows[2].cells[0].inlines[0].kind else {
        panic!("{:?}", rows[2].cells[0].inlines);
    };
    assert!(image.attributes.is_none());
}

#[test]
fn a_rows_block_reports_what_is_wrong_with_it() {
    let source = "| a |\n|---|\n| x {available=cloud, self-managed preview 3.4} |\n";
    let d = doc(source);
    assert!(rows(&d)[1].attributes.is_some());
    assert_eq!(slugs(&d), ["attribute-syntax"]);
}
