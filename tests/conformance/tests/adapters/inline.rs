//! The adapter's reading of inline extensions: what the inline pass adds to
//! the outline (SPEC §5.3). `syntax.rs` calls [`image`] for a paragraph that is
//! one image.

use ascribe_conformance::outline::normalize_ws;
use ascribe_conformance::{AttrValue, Attributes, Node, Row};
use ascribe_core::{AttributeBlock, AttributeValue, Span};
use ascribe_syntax::{Image, Table, raw_text};

/// The outline's `image` block for an image alone in its paragraph.
pub(super) fn image(source: &str, image: &Image) -> Node {
    Node::Image {
        // SPEC §5.3: a reference image's source is its definition's
        // destination. The tree's `destination` already is that; the label is
        // `Image::label` (full form) or the alt text (collapsed and shortcut).
        src: image.destination.clone(),
        alt: Some(normalize_ws(&raw_text(source, image.alt))),
        title: image.title.clone(),
        attributes: attributes(image),
    }
}

/// The image's attributes, the first use of a key winning.
pub(super) fn attributes(image: &Image) -> Attributes {
    block_attributes(image.attributes.as_ref().map(|a| &a.block))
}

/// A table's rows, each by its first cell and that cell's attribute block
/// (SPEC §4.4).
pub(super) fn rows(table: &Table, text: &dyn Fn(Span) -> String) -> Vec<Row> {
    table
        .rows
        .iter()
        .map(|row| Row {
            text: match row.cells.first().map(|c| c.inlines.as_slice()) {
                Some([first, .., last]) => text(Span::new(first.span.start(), last.span.end())),
                Some([only]) => text(only.span),
                _ => String::new(),
            },
            attributes: block_attributes(row.attributes.as_ref()),
        })
        .collect()
}

/// An attribute block's attributes, the first use of a key winning.
fn block_attributes(block: Option<&AttributeBlock>) -> Attributes {
    let mut attributes = Attributes::new();
    let Some(block) = block else {
        return attributes;
    };
    for a in &block.attributes {
        let Some(value) = &a.value else { continue };
        let value = match value {
            AttributeValue::Set { members, .. } => {
                AttrValue::Set(members.iter().map(|m| m.text.clone()).collect())
            }
            v => AttrValue::Single(v.as_text().unwrap_or_default().to_owned()),
        };
        attributes.entry(a.key.clone()).or_insert(value);
    }
    attributes
}
