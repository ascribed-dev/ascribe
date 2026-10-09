//! The head parser and the block parser (the comrak fork's scanner) must
//! agree on where a text primary starts, and `parse` must never panic.

use crate::support;

use std::sync::Arc;

use ascribe_core::FileId;
use ascribe_syntax::{BlockKind, ParseOptions, PrimaryValue, parse};
use comrak_ascribe::ascribe::AscribeOptions;
use comrak_ascribe::nodes::NodeValue;
use comrak_ascribe::{Arena, Options, parse_document};
use proptest::prelude::*;

/// Lines built from the pieces of a directive head, so most are close to
/// valid and the interesting edges come up often.
fn head_like() -> impl Strategy<Value = String> {
    let piece = prop_oneof![
        Just("@note".to_owned()),
        Just("@quill-demo".to_owned()),
        Just("@include".to_owned()),
        Just("@end".to_owned()),
        Just("{".to_owned()),
        Just("}".to_owned()),
        Just("type=tip".to_owned()),
        Just("a=\"x } y\"".to_owned()),
        Just("\"".to_owned()),
        Just("\\".to_owned()),
        Just(",".to_owned()),
        Just("|".to_owned()),
        Just("=".to_owned()),
        Just(":".to_owned()),
        Just(" ".to_owned()),
        Just("\t".to_owned()),
        Just("text".to_owned()),
        Just("é".to_owned()),
        any::<char>()
            .prop_filter("no line breaks", |c| !matches!(c, '\n' | '\r'))
            .prop_map(String::from),
    ];
    proptest::collection::vec(piece, 1..14).prop_map(|p| {
        let joined = p.concat();
        if joined.starts_with('@') {
            joined
        } else {
            format!("@note{joined}")
        }
    })
}

fn options() -> ParseOptions {
    let mut widget = ascribe_core::Builtin::Note.schema();
    widget.name = "quill-demo".into();
    widget.origin = ascribe_core::Origin::Widget;
    let mut schemas = ascribe_core::builtin_schemas();
    schemas.push(widget);
    ParseOptions::new(schemas)
}

/// The fork's view: where each Ascribe line's text primary starts, in the
/// raw line, from the block parser alone.
fn fork_view(line: &str, options: &ParseOptions) -> Vec<(String, Option<usize>)> {
    let mut comrak = Options::default();
    let mut keywords = AscribeOptions::new().keyword("end", false);
    for schema in &options.schemas {
        keywords.insert(
            schema.name.clone(),
            matches!(schema.primary, ascribe_core::Primary::Text { .. }),
        );
    }
    comrak.extension.ascribe = Some(Arc::new(keywords));
    let arena = Arena::new();
    let root = parse_document(&arena, line, &comrak);
    root.descendants()
        .filter_map(|n| match &n.data().value {
            NodeValue::AscribeLine(l) => Some((l.name.clone(), l.text_primary)),
            _ => None,
        })
        .collect()
}

proptest! {
    #[test]
    fn the_head_parser_agrees_with_the_block_parser(line in head_like()) {
        let options = options();
        let doc = parse(&line, &options);
        let ours = doc.blocks.iter().find_map(|b| match &b.kind {
            BlockKind::Directive(d) => Some(d),
            BlockKind::Container(c) => Some(&c.opener),
            BlockKind::Group(g) => g.arms.first().map(|a| &a.opener),
            _ => None,
        }).map(|d| (d.name.clone(), d.span.start(), d.primary.clone()));
        let forks = fork_view(&line, &options);
        let Some((name, start, primary)) = ours else {
            // Not a directive, or an end line: the fork sees an end line too.
            let ends = doc.blocks.iter().map(|b| match &b.kind {
                BlockKind::End(_) => 1,
                BlockKind::Container(c) => usize::from(c.end.is_some()),
                BlockKind::Group(g) => usize::from(g.end.is_some()),
                _ => 0,
            }).sum::<usize>();
            prop_assert_eq!(forks.len(), ends, "{:?}", line);
            return Ok(());
        };
        prop_assert_eq!(forks.len(), 1);
        prop_assert_eq!(&forks[0].0, &name);
        match (&primary, forks[0].1) {
            (Some(PrimaryValue::Text(p)), Some(offset)) => {
                prop_assert_eq!(p.span.start(), start + offset, "{:?}", line);
            }
            (Some(PrimaryValue::Text(_)), None) => {
                prop_assert!(false, "we found a text primary the fork didn't: {:?}", line);
            }
            (_, Some(_)) => {
                prop_assert!(false, "the fork found a text primary we didn't: {:?}", line);
            }
            (_, None) => {}
        }
        prop_assert!(support::check_tree(&line, &doc).is_empty(), "{:?}: {:?}", line, support::check_tree(&line, &doc));
    }

    #[test]
    fn parse_never_panics_and_keeps_its_spans_valid(source in "\\PC{0,120}") {
        let doc = parse(&source, &options());
        let problems = support::check_tree(&source, &doc);
        // Only exactness of Ascribe-line spans and validity are guaranteed
        // for arbitrary input; comrak's own positions can be off in odd
        // corners, so only range and boundary problems fail here.
        prop_assert!(
            problems.iter().all(|p| !p.contains("not a valid range")),
            "{:?}: {:?}", source, problems
        );
    }

    #[test]
    fn parse_never_panics_on_directive_soup(
        lines in proptest::collection::vec(head_like(), 1..6),
        indent in proptest::collection::vec(prop_oneof![Just(""), Just("  "), Just("> "), Just("- "), Just("    ")], 1..6),
    ) {
        let source: String = lines
            .iter()
            .zip(indent.iter().cycle())
            .map(|(l, i)| format!("{i}{l}\n"))
            .collect();
        let doc = parse(&source, &options().with_file(FileId::new(1)));
        let problems = support::check_tree(&source, &doc);
        prop_assert!(problems.iter().all(|p| !p.contains("not a valid range")), "{:?}: {:?}", source, problems);
    }
}
