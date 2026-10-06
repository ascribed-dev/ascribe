//! ASCRIBE: the link reference definitions `parse_document_with_definitions`
//! returns, with their positions. `ascribe-syntax`'s tests cover
//! them in every container and check every span; this checks the fork's API.

use std::sync::Arc;

use comrak_ascribe::ascribe::AscribeOptions;
use comrak_ascribe::nodes::LineColumn;
use comrak_ascribe::{Arena, Options, parse_document, parse_document_with_definitions};

fn pos(line: usize, column: usize) -> LineColumn {
    LineColumn { line, column }
}

#[test]
fn definitions_with_positions() {
    let md = "Intro.\n\n[ref]: <a b> \"T\"\n[two]:\n  /x\n\n> - [q]: /y\n";
    let arena = Arena::new();
    let (root, defs) = parse_document_with_definitions(&arena, md, &Options::default());
    assert_eq!(defs.len(), 3);

    let d = &defs[0];
    assert_eq!(
        (d.sourcepos.start, d.sourcepos.end),
        (pos(3, 1), pos(3, 16))
    );
    assert_eq!((d.label.start, d.label.end), (pos(3, 2), pos(3, 4)));
    assert_eq!(
        (d.destination.start, d.destination.end),
        (pos(3, 8), pos(3, 12))
    );
    assert_eq!(d.url, "a b");
    let (title, text) = d.title.as_ref().unwrap();
    assert_eq!(
        (title.start, title.end, text.as_str()),
        (pos(3, 14), pos(3, 16), "T")
    );
    assert_eq!(d.normalized_label, "ref");

    // A definition across lines, and one in a list item in a block quote.
    let d = &defs[1];
    assert_eq!((d.sourcepos.start, d.sourcepos.end), (pos(4, 1), pos(5, 4)));
    assert_eq!(
        (d.destination.start, d.destination.end),
        (pos(5, 3), pos(5, 4))
    );
    assert!(d.title.is_none());
    let d = &defs[2];
    assert_eq!((d.label.start, d.label.end), (pos(7, 6), pos(7, 6)));
    assert_eq!(
        (d.destination.start, d.destination.end),
        (pos(7, 10), pos(7, 11))
    );

    // The tree is what `parse_document` gives.
    let other = Arena::new();
    let plain = parse_document(&other, md, &Options::default());
    assert_eq!(root.descendants().count(), plain.descendants().count());
}

#[test]
fn a_text_primary_has_none_and_the_option_is_not_needed() {
    let mut options = Options::default();
    options.extension.ascribe = Some(Arc::new(AscribeOptions::new().keyword("note", true)));
    let arena = Arena::new();
    let (_, defs) = parse_document_with_definitions(&arena, "@note: [a]: /x\n[b]: /y\n", &options);
    assert!(defs.is_empty());
    let (_, defs) = parse_document_with_definitions(&arena, "@note: text\n\n[b]: /y\n", &options);
    assert_eq!(defs.len(), 1);
}
