//! Behavior of `parse`: directive heads, primaries, issues, and lines that
//! only look like directives.

#![allow(clippy::panic)]

use tessera_core::{AttributeValue, Builtin, DirectiveSchema, Origin, Span};
use tessera_syntax::*;

fn doc(source: &str) -> ParsedDocument {
    parse(source, &ParseOptions::default())
}

fn directive(source: &str) -> (ParsedDocument, DirectiveLine) {
    let d = doc(source);
    let Some(Block {
        kind: BlockKind::Directive(line),
        ..
    }) = d.blocks.first().cloned()
    else {
        panic!("not a directive: {:?}", d.blocks);
    };
    (d, line)
}

fn slugs(d: &ParsedDocument) -> Vec<&'static str> {
    d.issues.iter().map(|i| i.slug.as_str()).collect()
}

fn cut(source: &str, span: Span) -> &str {
    &source[span.start()..span.end()]
}

#[test]
fn parses_every_part_of_a_head() {
    let source = "@note {type=caution}: Back up your database first.\n";
    let (d, line) = directive(source);
    assert!(d.issues.is_empty());
    assert_eq!(line.name, "note");
    assert_eq!(cut(source, line.name_span), "@note");
    let attrs = line.attributes.expect("attributes");
    assert_eq!(cut(source, attrs.span), "{type=caution}");
    assert_eq!(cut(source, attrs.attributes[0].key_span), "type");
    assert_eq!(cut(source, line.colon.expect("colon")), ":");
    let Some(PrimaryValue::Text(p)) = line.primary else {
        panic!("a text primary");
    };
    assert_eq!(cut(source, p.span), "Back up your database first.");
    assert_eq!(line.form, Form::Line);
    assert_eq!(cut(source, line.span), &source[..source.len() - 1]);
}

#[test]
fn spacing_tolerance() {
    for source in [
        "@note{type=tip}:Text",
        "@note {type=tip}: Text",
        "@note\t{ type = tip }\t:\tText",
        "@note   {type=tip}   :   Text   ",
    ] {
        let (d, line) = directive(source);
        assert!(d.issues.is_empty(), "{source:?}: {:?}", d.issues);
        let Some(PrimaryValue::Text(p)) = &line.primary else {
            panic!("{source:?}: a text primary");
        };
        assert_eq!(cut(source, p.span), "Text", "{source:?}");
        assert_eq!(line.form, Form::Line);
    }
}

#[test]
fn a_trailing_colon_opens_a_container_but_a_primary_ending_in_one_does_not() {
    let (_, line) = directive("@note:\n");
    assert_eq!(line.form, Form::Container);
    assert!(line.primary.is_none());
    let (_, line) = directive("@note {type=tip}:   \t\n");
    assert_eq!(line.form, Form::Container);
    let (d, line) = directive("@note: Important:\n");
    assert_eq!(line.form, Form::Line);
    assert!(d.issues.is_empty());
    let Some(PrimaryValue::Text(p)) = line.primary else {
        panic!("a text primary");
    };
    assert_eq!(p.lines.len(), 1);
    let (_, line) = directive("@note\n");
    assert_eq!(line.form, Form::Line);
    assert!(line.colon.is_none());
}

#[test]
fn a_text_primary_continues_like_a_paragraph() {
    let source = "@note {type=caution}: Back up your database\nbefore you upgrade.\n\nNext.\n";
    let d = doc(source);
    assert_eq!(d.blocks.len(), 2);
    let BlockKind::Directive(line) = &d.blocks[0].kind else {
        panic!("a directive");
    };
    let Some(PrimaryValue::Text(p)) = &line.primary else {
        panic!("a text primary");
    };
    assert_eq!(p.lines.len(), 2);
    assert_eq!(
        raw_text(source, p.span),
        "Back up your database\nbefore you upgrade."
    );
    assert!(matches!(d.blocks[1].kind, BlockKind::Paragraph(_)));
}

#[test]
fn a_text_primary_in_a_quote_or_list_has_clean_lines() {
    let source = "> @note: one\n> two\n>   three\n";
    let d = doc(source);
    let BlockKind::BlockQuote(q) = &d.blocks[0].kind else {
        panic!("a quote");
    };
    let BlockKind::Directive(line) = &q.children[0].kind else {
        panic!("a directive");
    };
    let Some(PrimaryValue::Text(p)) = &line.primary else {
        panic!("a text primary");
    };
    let lines: Vec<_> = p.lines.iter().map(|l| cut(source, *l)).collect();
    assert_eq!(lines, ["one", "two", "three"]);
}

#[test]
fn identifier_and_line_primaries() {
    let (d, line) = directive("@include {heading=false}: guides/setup.md#install\n");
    assert!(d.issues.is_empty());
    let Some(PrimaryValue::Identifier(p)) = line.primary else {
        panic!("an identifier");
    };
    assert_eq!(p.text, "guides/setup.md#install");
    assert!(p.trailing.is_none());

    // An identifier ends at whitespace; the rest is kept, not dropped.
    let source = "@include: my file.md\n";
    let (d, line) = directive(source);
    assert_eq!(slugs(&d), ["directive-primary"]);
    let Some(PrimaryValue::Identifier(p)) = line.primary else {
        panic!("an identifier");
    };
    assert_eq!(p.text, "my");
    assert_eq!(cut(source, p.trailing.expect("trailing")), "file.md");

    // A line primary keeps its spaces, and never continues.
    let source = "@available: cloud, self-managed preview 3.4\nA paragraph.\n";
    let d = doc(source);
    assert!(d.issues.is_empty());
    assert_eq!(d.blocks.len(), 2, "the paragraph is its own block");
    let BlockKind::Directive(line) = &d.blocks[0].kind else {
        panic!("a directive");
    };
    let Some(PrimaryValue::Line(p)) = &line.primary else {
        panic!("a line primary");
    };
    assert_eq!(p.text, "cloud, self-managed preview 3.4");
    assert!(matches!(d.blocks[1].kind, BlockKind::Paragraph(_)));
}

#[test]
fn a_line_primary_isnt_inline_content() {
    let (_, line) = directive("@available: cloud *not emphasis*\n");
    let Some(PrimaryValue::Line(p)) = line.primary else {
        panic!("a line primary");
    };
    assert_eq!(p.text, "cloud *not emphasis*");
}

#[test]
fn reports_a_primary_where_the_schema_takes_none() {
    let source = "@steps: now\n";
    let (d, line) = directive(source);
    assert_eq!(slugs(&d), ["directive-primary"]);
    assert_eq!(d.issues[0].variant, None);
    assert_eq!(d.issues[0].arg("name"), Some("steps"));
    assert_eq!(cut(source, d.issues[0].location.span), "now");
    assert!(matches!(line.primary, Some(PrimaryValue::Unexpected(_))));
}

#[test]
fn reports_a_missing_required_primary() {
    for (source, name) in [
        ("@include\n", "include"),
        ("@include:\n", "include"),
        ("@id {x=y}:   \n", "id"),
        ("@available\n", "available"),
    ] {
        let d = doc(source);
        let missing: Vec<_> = d
            .issues
            .iter()
            .filter(|i| i.slug.as_str() == "directive-primary")
            .collect();
        assert_eq!(missing.len(), 1, "{source:?}: {:?}", d.issues);
        assert_eq!(missing[0].variant, Some("missing"));
        assert_eq!(missing[0].arg("name"), Some(name));
        assert!(missing[0].arg("kind").is_some() && missing[0].arg("example").is_some());
    }
    // Optional primaries aren't missing.
    assert!(doc("@note\n@note:\n@end\n").issues.is_empty());
}

#[test]
fn reports_malformed_attributes_with_their_slugs() {
    let d = doc("@note {type=tip: text\n");
    assert_eq!(slugs(&d), ["attribute-syntax"]);
    let d = doc("@note {type}: text\n");
    assert_eq!(slugs(&d), ["attribute-bare-key"]);
    let d = doc("@note {type=a b}: text\n");
    assert_eq!(slugs(&d), ["attribute-unquoted-reserved"]);
    let d = doc("@note {type=tip, type=note}: text\n");
    assert_eq!(slugs(&d), ["attribute-duplicate-key"]);
    let d = doc("@note {type=}: text\n");
    assert_eq!(slugs(&d), ["attribute-syntax"]);
}

#[test]
fn keeps_and_reports_text_that_fits_nothing_in_the_head() {
    let source = "@note hello: text\n";
    let (d, line) = directive(source);
    assert_eq!(slugs(&d), ["attribute-syntax"]);
    assert_eq!(cut(source, line.unexpected.expect("kept")), "hello: text");
    assert!(line.primary.is_none() && line.colon.is_none());
    assert_eq!(line.form, Form::Line);
}

#[test]
fn attribute_values_of_every_form() {
    let source = "@variant {pm=npm, platform=cloud|on-prem, label=\"Using other images\"}:\n";
    let (d, line) = directive(source);
    assert!(d.issues.is_empty(), "{:?}", d.issues);
    let attrs = line.attributes.expect("attributes");
    assert!(matches!(
        attrs.get("pm").and_then(|a| a.value.as_ref()),
        Some(AttributeValue::Token(_))
    ));
    assert!(matches!(
        attrs.get("platform").and_then(|a| a.value.as_ref()),
        Some(AttributeValue::Set { members, .. }) if members.len() == 2
    ));
    assert_eq!(
        attrs.get("label").and_then(|a| a.value.as_ref()?.as_text()),
        Some("Using other images")
    );
    assert_eq!(line.form, Form::Container);
}

#[test]
fn end_lines() {
    let d = doc("@note:\nText.\n@end\n");
    assert!(d.issues.is_empty());
    assert_eq!(d.blocks.len(), 3);
    assert!(matches!(&d.blocks[2].kind, BlockKind::End(e) if e.extra.is_none()));
    let d = doc("@end   \n");
    assert!(matches!(&d.blocks[0].kind, BlockKind::End(_)));
    assert!(d.issues.is_empty());
    let d = doc("@end: nope\n");
    assert_eq!(slugs(&d), ["directive-primary"]);
    assert!(matches!(&d.blocks[0].kind, BlockKind::End(e) if e.extra.is_some()));
}

#[test]
fn recognition_follows_the_spec() {
    // Unknown keywords, mid-word `@`, escapes, and code stay text.
    for source in [
        "@astrojs/react is a package.\n",
        "Mail support@example.com.\n",
        "\\@note: escaped\n",
        "`@note: in a code span`\n",
        "```\n@note: in a fence\n```\n",
        "    @note: indented code\n",
        "<div>\n@note: in html\n</div>\n",
        "@Note: uppercase\n",
    ] {
        let d = doc(source);
        assert!(
            !d.blocks
                .iter()
                .any(|b| matches!(b.kind, BlockKind::Directive(_))),
            "{source:?}"
        );
    }
    // Known keywords at line start, in containers and after up to three spaces.
    for source in [
        "  @note: three spaces\n",
        "- item\n  @note: in the item\n",
        "> @note: in a quote\n",
    ] {
        let d = doc(source);
        let found = format!("{:?}", d.blocks).contains("Directive(");
        assert!(found, "{source:?}");
    }
}

#[test]
fn a_directive_interrupts_a_paragraph() {
    let d = doc("Some text\n@note: a note\nMore text after it.\n");
    assert_eq!(d.blocks.len(), 2);
    assert!(matches!(d.blocks[0].kind, BlockKind::Paragraph(_)));
    assert!(matches!(d.blocks[1].kind, BlockKind::Directive(_)));
}

#[test]
fn misspelled_directives_stay_text_and_are_reported() {
    let source = "@warning: Careful.\n";
    let d = doc(source);
    assert!(matches!(d.blocks[0].kind, BlockKind::Paragraph(_)));
    assert_eq!(slugs(&d), ["directive-unknown"]);
    let issue = &d.issues[0];
    assert_eq!(cut(source, issue.location.span), "@warning");
    assert_eq!(issue.variant, Some("suggestion"));
    assert_eq!(issue.arg("name"), Some("warning"));
    assert_eq!(issue.arg("suggestion"), Some("@note {type=warning}:"));

    let d = doc("@availible: cloud\n");
    assert_eq!(d.issues[0].arg("suggestion"), Some("@available"));

    // No close match: still reported, with the plain message.
    let d = doc("@zzzzzz\n");
    assert_eq!(d.issues[0].variant, None);
    assert_eq!(d.issues[0].arg("suggestion"), None);
}

#[test]
fn misspelled_directives_are_found_wherever_a_line_starts() {
    let d = doc("Prose.\n@warning: mid-paragraph.\n\n> @tip {x=y}: in a quote\n\n- @notes\n");
    assert_eq!(slugs(&d), ["directive-unknown"; 3]);
    let d = doc("@note: A note\n@warning: continues the note's primary.\n");
    assert_eq!(slugs(&d), ["directive-unknown"]);
}

#[test]
fn prose_that_only_looks_like_directives_isnt_reported() {
    for source in [
        "@astrojs/react\n",
        "@timestamp is a field.\n",
        "\\@warning: escaped\n",
        "support@example.com\n",
        "See `@warning:` here.\n",
        "```\n@warning:\n```\n",
        "text @warning: not at line start\n",
        "@Warning:\n",
    ] {
        assert!(doc(source).issues.is_empty(), "{source:?}");
    }
}

#[test]
fn project_widgets_are_directives() {
    let mut widget = Builtin::Note.schema();
    widget.name = "quill-demo".into();
    widget.origin = Origin::Widget;
    let mut schemas = tessera_core::builtin_schemas();
    schemas.push(widget);
    let source = "@quill-demo {type=tip}: A demo\ncontinues.\n";
    let d = parse(source, &ParseOptions::new(schemas));
    assert!(d.issues.is_empty());
    let BlockKind::Directive(line) = &d.blocks[0].kind else {
        panic!("a directive");
    };
    assert_eq!(line.name, "quill-demo");
    assert!(matches!(line.primary, Some(PrimaryValue::Text(_))));
    // Without the widget declared, it's a misspelled directive.
    assert_eq!(slugs(&doc(source)), ["directive-unknown"]);
}

#[test]
fn issues_carry_the_file_id() {
    let options = ParseOptions::default().with_file(tessera_core::FileId::new(7));
    let d = parse("@steps: x\n", &options);
    assert_eq!(d.file, tessera_core::FileId::new(7));
    assert_eq!(d.issues[0].location.file, tessera_core::FileId::new(7));
}

#[test]
fn blocks_and_inlines() {
    let source = "---\ntitle: T\n---\n\n# Heading *one*\n\nText with `code`, [a link](/x \"t\"), ![alt](s.png \"ti\"), ![ref][r], ![c][], ![s], <https://x.y>.\n\n[r]: /r\n[c]: /c\n[s]: /s\n\n- one\n- two\n\n1. first\n\n> quote\n\n```rust\nfn x() {}\n```\n\n| a | b |\n|---|---|\n| c | d |\n\n***\n";
    let d = doc(source);
    let fm = d.frontmatter.as_ref().expect("frontmatter");
    assert_eq!(cut(source, fm.content), "title: T\n");
    let BlockKind::Heading(h) = &d.blocks[0].kind else {
        panic!("a heading");
    };
    assert_eq!((h.level, h.setext), (1, false));
    assert_eq!(cut(source, h.content), "Heading *one*");
    let BlockKind::Paragraph(p) = &d.blocks[1].kind else {
        panic!("a paragraph");
    };
    let forms: Vec<_> = p
        .inlines
        .iter()
        .filter_map(|i| match &i.kind {
            InlineKind::Image(img) => Some((img.form, img.destination.clone())),
            _ => None,
        })
        .collect();
    assert_eq!(
        forms,
        [
            (LinkForm::Inline, "s.png".to_owned()),
            (LinkForm::Full, "/r".to_owned()),
            (LinkForm::Collapsed, "/c".to_owned()),
            (LinkForm::Shortcut, "/s".to_owned()),
        ]
    );
    let image = p
        .inlines
        .iter()
        .find_map(|i| match &i.kind {
            InlineKind::Image(img) if img.form == LinkForm::Inline => Some(img),
            _ => None,
        })
        .expect("an image");
    assert_eq!(cut(source, image.alt), "alt");
    assert_eq!(image.title.as_deref(), Some("ti"));
    assert!(image.attributes.is_none());
    let full = p
        .inlines
        .iter()
        .find_map(|i| match &i.kind {
            InlineKind::Image(img) if img.form == LinkForm::Full => Some(img),
            _ => None,
        })
        .expect("a full reference image");
    assert_eq!(cut(source, full.label.expect("label")), "r");
    assert!(
        p.inlines
            .iter()
            .any(|i| matches!(&i.kind, InlineKind::Link(l) if l.form == LinkForm::Autolink))
    );
    let kinds: Vec<_> = d
        .blocks
        .iter()
        .map(|b| std::mem::discriminant(&b.kind))
        .collect();
    assert!(kinds.len() >= 7);
}

#[test]
fn never_produces_nodes_phases_06_and_07_own() {
    let d = doc(".Title\n@note:\nx\n@end\n@variant {a=b}:\n{phrase} ![i](s){w=1}\n");
    let text = format!("{:?}", d.blocks);
    for owned in ["Container(", "Group(", "Title(", "Phrase("] {
        assert!(!text.contains(owned), "{owned}");
    }
}

#[test]
fn schema_type_is_reexported_in_options() {
    let schemas: Vec<DirectiveSchema> = ParseOptions::default().schemas;
    assert_eq!(schemas.len(), 7);
}
