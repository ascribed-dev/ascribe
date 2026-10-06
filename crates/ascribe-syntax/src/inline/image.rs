//! Attribute blocks after images (SPEC §5.3).
//!
//! The fork's inline parser skips the block right after an image, using
//! `comrak_ascribe::ascribe::image_attributes_len`; this finds the same block
//! with the same function and reads it with
//! `ascribe_core::parse_attribute_block`. Every form of image is one node, so
//! this needs no other knowledge of how the image was written.

use ascribe_core::{Span, parse_attribute_block};
use comrak_ascribe::ascribe::image_attributes_len;

use super::Pass;
use crate::tree::*;

impl Pass<'_> {
    /// Attaches the attribute block directly after `inline`, an image, and
    /// grows the image's span to cover it.
    ///
    /// SPEC §5.1, §5.3: any `{…}` closed on the same line and directly
    /// after an image is an attribute block, including `{key}`, which is a
    /// bare key there and not a phrase.
    pub(super) fn image_attributes(&mut self, inline: &mut Inline) {
        let InlineKind::Image(image) = &mut inline.kind else {
            return;
        };
        let at = inline.span.end();
        let Some(rest) = self.source.get(at..) else {
            return;
        };
        if let Some(len) = image_attributes_len(rest) {
            let Some(parsed) = parse_attribute_block(&rest[..len], at, self.file) else {
                return;
            };
            self.issues.extend(parsed.issues);
            image.attributes = Some(ImageAttributes {
                block: parsed.block,
            });
            inline.span = Span::new(inline.span.start(), at + len);
        } else if rest.starts_with('{') {
            // No `}` on the line, so the fork left it as text. It's still
            // meant as an attribute block: report it, and keep the text.
            let line = rest.split(['\n', '\r']).next().unwrap_or(rest);
            if let Some(parsed) = parse_attribute_block(line, at, self.file) {
                self.issues.extend(parsed.issues);
            }
        }
    }
}
