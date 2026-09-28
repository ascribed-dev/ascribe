//! The adapter's reading of inline extensions: what phase 07 adds to the
//! outline (SPEC §5.3). `syntax.rs` calls [`image`] for a paragraph that is
//! one image.

use tessera_conformance::outline::normalize_ws;
use tessera_conformance::{AttrValue, Attributes, Node};
use tessera_core::AttributeValue;
use tessera_syntax::{Image, raw_text};

/// The outline's `image` block for an image alone in its paragraph.
pub(super) fn image(source: &str, image: &Image) -> Node {
    Node::Image {
        // SPEC §5.3 (resolved Q23): a reference image's source is its definition's
        // destination. The tree's `destination` already is that; the label is
        // `Image::label` (full form) or the alt text (collapsed and shortcut).
        src: image.destination.clone(),
        alt: Some(normalize_ws(&raw_text(source, image.alt))),
        title: image.title.clone(),
        attributes: attributes(image),
    }
}

/// The image's attributes, the first use of a key winning.
fn attributes(image: &Image) -> Attributes {
    let mut attributes = Attributes::new();
    let Some(block) = image.attributes.as_ref().map(|a| &a.block) else {
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
