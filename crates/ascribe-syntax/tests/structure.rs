//! The structure pass: containers, groups and arms, titles,
//! bindings, and the structural issues of SPEC §8.2. The conformance suite
//! checks the same rules against the spec's own examples; these tests cover
//! the tree's shape and the pass's invariants.

#![allow(clippy::panic, clippy::expect_used)]

mod support;

use ascribe_core::Span;
use ascribe_syntax::*;
use proptest::prelude::*;

fn doc(source: &str) -> ParsedDocument {
    parse(source, &ParseOptions::default())
}

fn slugs(d: &ParsedDocument) -> Vec<&'static str> {
    d.issues.iter().map(|i| i.slug.as_str()).collect()
}

fn cut(source: &str, span: Span) -> &str {
    &source[span.start()..span.end()]
}

fn directive(block: &Block) -> &DirectiveLine {
    match &block.kind {
        BlockKind::Directive(line) => line,
        other => panic!("not a line-form directive: {other:?}"),
    }
}

#[test]
fn a_container_holds_its_blocks_and_end_line() {
    let source = "@note {type=tip}:\nOne.\n\nTwo.\n@end\nAfter.\n";
    let d = doc(source);
    assert!(d.issues.is_empty(), "{:?}", d.issues);
    assert_eq!(d.blocks.len(), 2);
    let BlockKind::Container(c) = &d.blocks[0].kind else {
        panic!("a container: {:?}", d.blocks[0]);
    };
    assert_eq!(c.opener.name, "note");
    assert_eq!(c.children.len(), 2);
    assert_eq!(cut(source, c.end.as_ref().expect("closed").span), "@end");
    assert_eq!(
        cut(source, d.blocks[0].span),
        &source[..source.find("\nAfter").unwrap()]
    );
    assert!(matches!(d.blocks[1].kind, BlockKind::Paragraph(_)));
}

#[test]
fn end_closes_the_innermost_container() {
    let d = doc(".Outer\n@details:\nBefore.\n@note:\nInner.\n@end\nAfter.\n@end\n");
    assert!(d.issues.is_empty(), "{:?}", d.issues);
    let BlockKind::Container(outer) = &d.blocks[0].kind else {
        panic!()
    };
    assert_eq!(outer.children.len(), 3);
    assert!(matches!(&outer.children[1].kind, BlockKind::Container(c) if c.opener.name == "note"));
}

#[test]
fn a_group_is_one_node_and_its_end_line_is_the_groups() {
    let source = "@variant {pm=npm}:\nA.\n@variant {pm=yarn}:\nB.\n@end\n";
    let d = doc(source);
    assert!(d.issues.is_empty(), "{:?}", d.issues);
    assert_eq!(d.blocks.len(), 1);
    let BlockKind::Group(g) = &d.blocks[0].kind else {
        panic!()
    };
    assert_eq!(g.name, "variant");
    assert_eq!(g.arms.len(), 2);
    assert!(g.end.is_some());
    // An arm runs from its opener to its last block, not into the next arm.
    assert_eq!(cut(source, g.arms[0].span), "@variant {pm=npm}:\nA.");
    assert_eq!(cut(source, g.arms[1].span), "@variant {pm=yarn}:\nB.");
}

#[test]
fn a_container_open_at_the_next_arm_is_reported_there() {
    let source = "@variant {pm=npm}:\n@note:\nUnclosed.\n@variant {pm=yarn}:\nB.\n@end\n";
    let d = doc(source);
    assert_eq!(slugs(&d), ["container-open-at-arm"]);
    let issue = &d.issues[0];
    assert_eq!(cut(source, issue.location.span), "@variant");
    assert_eq!(issue.arg("name"), Some("note"));
    assert_eq!(issue.arg("line"), Some("2"));
    let BlockKind::Group(g) = &d.blocks[0].kind else {
        panic!()
    };
    // The note stays in the arm it was opened in, unclosed.
    assert!(matches!(&g.arms[0].children[0].kind, BlockKind::Container(c) if c.end.is_none()));
    assert_eq!(g.arms.len(), 2);
    assert!(g.end.is_some());
}

#[test]
fn an_opener_joins_the_nearest_group_of_its_own_directive() {
    // The inner `@variant` is a sibling arm, not a nested group (SPEC §3.6).
    let d = doc("@variant {pm=npm}:\n@variant {pm=yarn}:\nText.\n@end\n");
    assert!(d.issues.is_empty(), "{:?}", d.issues);
    let BlockKind::Group(g) = &d.blocks[0].kind else {
        panic!()
    };
    assert_eq!(g.arms.len(), 2);
    assert!(g.arms[0].children.is_empty());
}

#[test]
fn a_group_inside_another_directive_stays_in_it() {
    let d = doc("@note:\n@variant {pm=npm}:\nA.\n@variant {pm=yarn}:\nB.\n@end\n@end\n");
    assert!(d.issues.is_empty(), "{:?}", d.issues);
    let BlockKind::Container(c) = &d.blocks[0].kind else {
        panic!()
    };
    assert!(matches!(c.children[0].kind, BlockKind::Group(_)));
}

#[test]
fn unclosed_containers_are_reported_at_their_openers() {
    let source = "@note:\nOne.\n\n@details:\nTwo.\n";
    let d = doc(source);
    let unclosed: Vec<_> = d
        .issues
        .iter()
        .filter(|i| i.slug.as_str() == "container-unclosed")
        .map(|i| cut(source, i.location.span))
        .collect();
    assert_eq!(unclosed, ["@note", "@details"]);
    // Nothing is dropped: the tree keeps every block.
    let BlockKind::Container(outer) = &d.blocks[0].kind else {
        panic!()
    };
    assert!(outer.end.is_none());
    assert!(
        matches!(&outer.children.last().expect("child").kind, BlockKind::Container(c) if c.end.is_none())
    );
}

#[test]
fn containers_cant_straddle_list_items() {
    // The note is unclosed at the end of its item; the `@end` in the next
    // item is another container's, so it's reported too (SPEC §3.9).
    let d = doc("- One\n\n  @note:\n  Inside.\n- Two\n\n  @end\n");
    assert_eq!(slugs(&d), ["container-unclosed", "end-indent-mismatch"]);
    let d = doc("- One\n\n  @note:\n  Inside.\n  @end\n- Two\n");
    assert!(d.issues.is_empty(), "{:?}", d.issues);
    // An end line inside a list item can't close an opener outside it.
    let d = doc("@note:\n- item\n\n  @end\n");
    assert!(slugs(&d).contains(&"end-indent-mismatch"));
    assert!(slugs(&d).contains(&"container-unclosed"));
    // With nothing open anywhere, it's simply unmatched.
    assert_eq!(slugs(&doc("Text.\n\n@end\n")), ["end-unmatched"]);
    // An orphan is claimed by one end line only.
    let d = doc("- a\n\n  @note:\n- b\n\n  @end\n\n@end\n");
    assert_eq!(
        slugs(&d),
        ["container-unclosed", "end-indent-mismatch", "end-unmatched"]
    );
}

#[test]
fn extra_indentation_in_the_same_container_still_closes() {
    let d = doc("@note:\nInside.\n   @end\n");
    assert!(d.issues.is_empty(), "{:?}", d.issues);
    assert!(matches!(&d.blocks[0].kind, BlockKind::Container(c) if c.end.is_some()));
}

#[test]
fn form_errors_report_once_and_keep_the_structure() {
    // SPEC §3.5: the colon line opens a container even as an error.
    let d = doc("@steps:\n1. One.\n@end\n");
    assert_eq!(slugs(&d), ["container-colon-unexpected"]);
    assert!(matches!(&d.blocks[0].kind, BlockKind::Container(c) if c.end.is_some()));
    // A container-only directive without its colon is still an opener.
    let d = doc("@variant {pm=npm}\nText.\n@end\n");
    assert_eq!(slugs(&d), ["container-colon-missing"]);
    assert!(matches!(d.blocks[0].kind, BlockKind::Group(_)));
    // A directive with both forms is a container only with its colon.
    let d = doc("@note\nText.\n@end\n");
    assert_eq!(slugs(&d), ["end-unmatched"]);
    // Attributes on a container-only directive with a trailing colon: fine.
    assert!(doc("@variant {pm=npm}:\nA.\n@end\n").issues.is_empty());
}

#[test]
fn nesting_deeper_than_two_warns_at_the_innermost_opener() {
    let source = ".O\n@details:\n@note:\n@note:\nDeep.\n@end\n@end\n@end\n";
    let d = doc(source);
    assert_eq!(slugs(&d), ["container-nesting-deep"]);
    assert_eq!(d.issues[0].arg("depth"), Some("3"));
    assert_eq!(cut(source, d.issues[0].location.span), "@note");
    assert_eq!(
        d.issues[0].location.span.start(),
        source.find("@note:\nDeep").expect("inner")
    );
    // Containers around a list item count.
    let d = doc("@note:\n- a\n\n  @note:\n  @note:\n  x\n  @end\n  @end\n@end\n");
    assert!(
        slugs(&d).contains(&"container-nesting-deep"),
        "{:?}",
        d.issues
    );
    // A group is one level, and arms aren't more.
    let d = doc("@variant {a=1}:\n@variant {a=2}:\n@note:\nx\n@end\n@end\n");
    assert!(d.issues.is_empty(), "{:?}", d.issues);
}

#[test]
fn a_title_line_attaches_to_the_directive_below() {
    let source = ".Try **it**\n@note {type=tip}\nText.\n";
    let d = doc(source);
    assert!(d.issues.is_empty(), "{:?}", d.issues);
    assert_eq!(d.blocks.len(), 2);
    let line = directive(&d.blocks[0]);
    let title = line.title.as_ref().expect("title");
    assert_eq!(cut(source, title.dot), ".");
    assert_eq!(cut(source, title.content), "Try **it**");
    assert_eq!(cut(source, title.span), ".Try **it**");
    assert!(matches!(&title.inlines[0].kind, InlineKind::Text(t) if t == "Try "));
    assert!(matches!(title.inlines[1].kind, InlineKind::Strong(_)));
    // The block covers the title line, and the directive line still starts at `@`.
    assert_eq!(d.blocks[0].span.start(), 0);
    assert_eq!(cut(source, line.span), "@note {type=tip}");
}

#[test]
fn a_title_can_be_only_a_dot_and_markup() {
    let d = doc(".*x*\n@note: A.\n");
    let title = directive(&d.blocks[0]).title.as_ref().expect("title");
    assert_eq!(title.inlines.len(), 1);
    assert!(matches!(title.inlines[0].kind, InlineKind::Emphasis(_)));
}

#[test]
fn dot_lines_that_arent_titles_stay_paragraphs() {
    for (source, expected) in [
        ("Text\n.Not\n@note: A.\n", vec![]),
        (".Not\n\n@note: A.\n", vec![]),
        (". Space\n@note: A.\n", vec!["title-dot-space"]),
        (". Space\n\n@note: A.\n", vec![]),
        ("..Two\n@note: A.\n", vec![]),
        ("\\.Escaped\n@steps\n1. a\n", vec![]),
        (".Title\n@steps\n1. a\n", vec!["title-not-accepted"]),
        (". Title\n@steps\n1. a\n", vec![]),
        (".\n@note: A.\n", vec![]),
    ] {
        let d = doc(source);
        assert_eq!(slugs(&d), expected, "{source:?}");
        assert!(
            d.blocks
                .iter()
                .all(|b| !matches!(&b.kind, BlockKind::Directive(l) if l.title.is_some())),
            "{source:?}"
        );
    }
}

#[test]
fn titles_attach_to_openers_and_arms() {
    let d = doc(".Docker\n@variant:\nA.\n\n.Other\n@variant:\nB.\n@end\n");
    assert!(d.issues.is_empty(), "{:?}", d.issues);
    let BlockKind::Group(g) = &d.blocks[0].kind else {
        panic!()
    };
    assert!(
        g.arms
            .iter()
            .all(|a| a.title.is_some() && a.opener.title == a.title)
    );
    let d = doc(".Ref\n@details:\nX.\n@end\n");
    assert!(d.issues.is_empty(), "{:?}", d.issues);
    let BlockKind::Container(c) = &d.blocks[0].kind else {
        panic!()
    };
    assert!(c.opener.title.is_some());
}

#[test]
fn required_titles_and_variant_arm_rules() {
    assert_eq!(slugs(&doc("@details\nText.\n")), ["details-title-missing"]);
    assert_eq!(
        slugs(&doc("@details:\nText.\n@end\n")),
        ["details-title-missing"]
    );
    assert_eq!(
        slugs(&doc("@variant:\nText.\n@end\n")),
        ["variant-arm-kind"]
    );
    assert_eq!(
        slugs(&doc(".T\n@variant {a=b}:\nText.\n@end\n")),
        ["variant-arm-kind"]
    );
    assert_eq!(
        slugs(&doc(".T\n@variant:\nX.\n@variant {a=b}:\nY.\n@end\n")),
        ["variant-mixed-arms"]
    );
    assert_eq!(
        slugs(&doc("@variant {a=1}:\nX.\n@variant {b=2}:\nY.\n@end\n")),
        ["variant-no-shared-dimension"]
    );
    assert!(
        doc("@variant {a=1, b=1}:\nX.\n@variant {b=2}:\nY.\n@variant {b=3, c=1}:\nZ.\n@end\n")
            .issues
            .is_empty()
    );
}

#[test]
fn group_issues_are_reported_at_the_first_opener() {
    // SPEC §3.6.
    let source = ".T\n@variant:\nX.\n@variant {a=b}:\nY.\n";
    let d = doc(source);
    let mixed = d
        .issues
        .iter()
        .find(|i| i.slug.as_str() == "variant-mixed-arms")
        .expect("mixed");
    assert_eq!(cut(source, mixed.location.span), "@variant");
    assert_eq!(mixed.location.span.start(), 3);
    let unclosed = d
        .issues
        .iter()
        .find(|i| i.slug.as_str() == "container-unclosed")
        .expect("unclosed");
    assert_eq!(unclosed.location.span.start(), 3);
}

#[test]
fn bindings_by_schema_and_position() {
    let source = "\
@available: cloud
Before any heading.

## H
@id: h

@available: cloud
@include: x.md
@note: Own.
@note
Bound.

@steps
1. one
";
    let d = doc(source);
    // `@include` follows heading-bound directives, so nothing after it is at the top.
    let bindings: Vec<_> = d
        .blocks
        .iter()
        .filter_map(|b| match &b.kind {
            BlockKind::Directive(l) => Some((l.name.as_str(), l.binding)),
            _ => None,
        })
        .collect();
    assert_eq!(
        bindings,
        [
            ("available", Some(Bound::FollowingBlock)),
            ("id", Some(Bound::Heading)),
            ("available", Some(Bound::Heading)),
            ("include", Some(Bound::Own)),
            ("note", Some(Bound::Own)),
            ("note", Some(Bound::FollowingBlock)),
            ("steps", Some(Bound::FollowingBlock)),
        ]
    );
    assert!(d.issues.is_empty(), "{:?}", d.issues);
}

#[test]
fn heading_bound_errors() {
    // SPEC §3.8.
    assert_eq!(
        slugs(&doc("@id: orphan\n\nText.\n")),
        ["binding-not-section-top"]
    );
    assert_eq!(
        slugs(&doc("## H\nText.\n\n@id: late\n")),
        ["binding-not-section-top"]
    );
    // A directive after a subheading binds the subheading.
    assert!(doc("## A\n### B\n@id: b\n\nText.\n").issues.is_empty());
    // A heading-bound directive in a container has its own sections.
    assert_eq!(
        slugs(&doc("## H\n@note:\n@id: x\n@end\n")),
        ["binding-not-section-top"]
    );
}

#[test]
fn following_block_errors_and_warnings() {
    assert_eq!(slugs(&doc("Text.\n\n@note\n")), ["binding-no-block"]);
    assert_eq!(slugs(&doc("@note\n## Heading\n")), ["binding-heading"]);
    assert_eq!(slugs(&doc("@note\n\nText.\n")), ["binding-blank-line"]);
    // The next block must be in the same container.
    let d = doc("@note:\n@steps\n@end\nAfter.\n");
    assert_eq!(slugs(&d), ["binding-no-block"]);
    assert_eq!(d.issues[0].arg("container"), Some("container"));
    // A block quote and a list item are containers too.
    assert_eq!(slugs(&doc("> @note\n\nText.\n")), ["binding-no-block"]);
    assert_eq!(slugs(&doc("- a\n\n  @note\n- b\n")), ["binding-no-block"]);
    // What binds a block never binds another directive that stands alone.
    assert_eq!(slugs(&doc("@note\n@include: x.md\n")), ["binding-no-block"]);
    // Directives that bind the following block stack on the same block.
    let d = doc("@available: cloud\n@note\nText.\n");
    assert!(d.issues.is_empty(), "{:?}", d.issues);
    assert_eq!(bound_block(&d.blocks, 0), Some(2));
    assert_eq!(bound_block(&d.blocks, 1), Some(2));
}

#[test]
fn a_one_line_note_is_a_block_a_directive_can_bind() {
    // SPEC §3.8: the three spellings of a cloud-only note.
    for source in [
        "@available: cloud\n@note: Streaming sync is in preview.\n",
        "@available: cloud\n@note\nStreaming sync is in preview.\n",
        "@available: cloud\n@note:\nInside.\n@end\n",
    ] {
        let d = doc(source);
        assert!(d.issues.is_empty(), "{source:?}: {:?}", d.issues);
        assert_eq!(directive(&d.blocks[0]).binding, Some(Bound::FollowingBlock));
    }
    let d = doc("@available: cloud\n@note: Preview.\n");
    assert_eq!(bound_block(&d.blocks, 0), Some(1));
    // A stack ends at the text directive, which binds its own text.
    let d = doc("@available: cloud\n@steps\n@note: N.\n");
    assert_eq!(bound_block(&d.blocks, 0), Some(2));
    // Directives that stand alone still have no block.
    assert_eq!(
        slugs(&doc("@available: cloud\n@include: x.md\n")),
        ["binding-no-block"]
    );
    assert_eq!(
        slugs(&doc("@note\n@id: x\n")),
        ["binding-no-block", "binding-not-section-top"]
    );
}

#[test]
fn an_unreadable_head_is_reported_once() {
    let d = doc("@note {type=tip\n");
    assert_eq!(slugs(&d), ["attribute-syntax"]);
    assert_eq!(directive(&d.blocks[0]).binding, Some(Bound::Unbound));
}

#[test]
fn steps_must_bind_an_ordered_list() {
    assert!(
        doc("@steps\n1. a\n2. b\n\n@steps\n5. c\n")
            .issues
            .is_empty()
    );
    let d = doc("@steps\n- a\n");
    assert_eq!(slugs(&d), ["steps-not-ordered-list"]);
    assert_eq!(d.issues[0].arg("found"), Some("a bullet list"));
    assert_eq!(slugs(&doc("@steps\nText.\n")), ["steps-not-ordered-list"]);
    assert_eq!(slugs(&doc("@steps\n")), ["binding-no-block"]);
}

#[test]
fn bound_helpers_find_the_heading_and_the_block() {
    let d = doc("## H\n@id: h\n@available: cloud\n\nText.\n\n@note\nBound.\n");
    assert_eq!(bound_heading(&d.blocks, 1), Some(0));
    assert_eq!(bound_heading(&d.blocks, 2), Some(0));
    assert_eq!(bound_heading(&d.blocks, 0), None);
    assert_eq!(bound_block(&d.blocks, 1), None);
    assert_eq!(bound_block(&d.blocks, 4), Some(5));
}

#[test]
fn list_warnings() {
    let d = doc("1. a\n@note: N.\n");
    assert_eq!(slugs(&d), ["list-ended-by-directive"]);
    // A blank line, an end line, or a block quote isn't reported.
    assert!(doc("1. a\n\n@note: N.\n").issues.is_empty());
    assert!(doc("@note:\n- a\n@end\n").issues.is_empty());
    assert!(doc("> a\n@note: N.\n").issues.is_empty());
    // Indented to the content column: in the item.
    assert!(doc("1. a\n   @note: N.\n").issues.is_empty());
    // Over-indented into code, at the top level and in an item.
    let d = doc("Text.\n\n    @note: X.\n");
    assert_eq!(slugs(&d), ["directive-indented-code"]);
    assert_eq!(d.issues[0].arg("name"), Some("note"));
    assert_eq!(d.issues[0].location.span, Span::new(11, 16));
    assert_eq!(
        slugs(&doc("1. a\n\n       @end\n")),
        ["directive-indented-code"]
    );
    // Ordinary code and unknown names aren't.
    assert!(
        doc("Text.\n\n    let x = 1;\n\n    @astrojs/react\n")
            .issues
            .is_empty()
    );
    assert!(doc("```\n@note: X.\n```\n").issues.is_empty());
    // Numbering: a list that continues an `@steps` list after a directive.
    let d = doc("@steps\n1. a\n@note: N.\n\n2. b\n");
    assert_eq!(
        slugs(&d),
        ["list-ended-by-directive", "steps-numbering-continued"]
    );
    assert!(
        doc("@steps\n1. a\n@note: N.\n\n1. b\n")
            .issues
            .iter()
            .all(|i| i.slug.as_str() != "steps-numbering-continued")
    );
    assert!(doc("@steps\n1. a\n2. b\n").issues.is_empty());
}

#[test]
fn appendix_b_has_no_issues() {
    let source =
        std::fs::read_to_string(support::repo_root().join("examples/quill/docs/install-agent.md"))
            .expect("the Quill example page");
    let d = doc(&source);
    assert!(d.issues.is_empty(), "{:?}", d.issues);
    assert!(support::check_tree(&source, &d).is_empty());
    // The install section's group sits inside its list item.
    let BlockKind::List(list) = &d
        .blocks
        .iter()
        .find(|b| matches!(b.kind, BlockKind::List(_)))
        .expect("a list")
        .kind
    else {
        panic!()
    };
    assert!(
        matches!(list.items[0].children.iter().find(|b| matches!(b.kind, BlockKind::Group(_))).map(|b| &b.kind), Some(BlockKind::Group(g)) if g.arms.len() == 3)
    );
}

// ---------------------------------------------------------------------------
// Properties

/// What the pass leaves behind, counted over the whole tree.
#[derive(Default, Debug)]
struct Census {
    unclosed: usize,
    stray_ends: usize,
    flat_openers: usize,
}

fn census(blocks: &[Block], out: &mut Census) {
    for block in blocks {
        match &block.kind {
            BlockKind::Container(c) => {
                out.unclosed += usize::from(c.end.is_none());
                census(&c.children, out);
            }
            BlockKind::Group(g) => {
                out.unclosed += usize::from(g.end.is_none());
                for arm in &g.arms {
                    census(&arm.children, out);
                }
            }
            BlockKind::End(_) => out.stray_ends += 1,
            BlockKind::Directive(l) => out.flat_openers += usize::from(l.form == Form::Container),
            BlockKind::BlockQuote(q) => census(&q.children, out),
            BlockKind::List(l) => {
                for item in &l.items {
                    census(&item.children, out);
                }
            }
            _ => {}
        }
    }
}

fn line() -> impl Strategy<Value = &'static str> {
    prop_oneof![
        Just("@note:"),
        Just("@note"),
        Just("@note {type=tip}: text"),
        Just("@details:"),
        Just("@variant {pm=npm}:"),
        Just("@variant {deployment=cloud}:"),
        Just("@variant:"),
        Just("@variant"),
        Just("@end"),
        Just("@end"),
        Just("@steps"),
        Just("@steps:"),
        Just("@id: x"),
        Just("@available: cloud"),
        Just("@include: a.md"),
        Just(".Title"),
        Just(". Spaced"),
        Just("text"),
        Just("# Heading"),
        Just("## Sub"),
        Just("1. one"),
        Just("2. two"),
        Just("- item"),
        Just("```"),
        Just("    code"),
        Just(""),
        Just(""),
    ]
}

fn prefix() -> impl Strategy<Value = &'static str> {
    prop_oneof![
        Just(""),
        Just(""),
        Just(""),
        Just("  "),
        Just("> "),
        Just("   "),
        Just("    "),
        Just("- "),
    ]
}

proptest! {
    /// The pass never panics, every container it opens is closed or
    /// reported, and every end line is consumed or reported.
    #[test]
    fn every_container_is_closed_or_reported(
        lines in proptest::collection::vec((prefix(), line()), 1..30),
    ) {
        let source: String = lines.iter().map(|(p, l)| format!("{p}{l}\n")).collect();
        let d = doc(&source);
        let problems = support::check_tree(&source, &d);
        prop_assert!(problems.iter().all(|p| !p.contains("not a valid range")), "{:?}: {:?}", source, problems);
        let mut c = Census::default();
        census(&d.blocks, &mut c);
        let count = |names: &[&str]| d.issues.iter().filter(|i| names.contains(&i.slug.as_str())).count();
        prop_assert_eq!(c.flat_openers, 0, "{:?}", source);
        prop_assert_eq!(c.unclosed, count(&["container-unclosed", "container-open-at-arm"]), "{:?}", source);
        prop_assert_eq!(c.stray_ends, count(&["end-unmatched", "end-indent-mismatch"]), "{:?}", source);
        // Issues stay in source order.
        prop_assert!(d.issues.windows(2).all(|w| w[0].location.span.start() <= w[1].location.span.start()));
    }

    /// Arbitrary text never panics the whole pipeline either.
    #[test]
    fn arbitrary_text_never_panics(source in "[@a-z{}:.\\n> -]{0,200}") {
        let d = doc(&source);
        let mut c = Census::default();
        census(&d.blocks, &mut c);
        prop_assert_eq!(c.flat_openers, 0, "{:?}", source);
    }
}
