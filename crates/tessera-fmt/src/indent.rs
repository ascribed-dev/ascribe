//! Indentation of directive lines and end lines (SPEC §3.9, §8.3, Q1, Q19).
//!
//! A directive line, or an end line, belongs to the container that owns it:
//! the document, a list item, or a block quote. It may be indented up to
//! three columns beyond that container's content (SPEC §1.5, §3.9), which
//! doesn't change what it belongs to. Canonical form removes those extra
//! spaces: a directive line in a list item is indented exactly to the item's
//! content column, and one in the document isn't indented at all. In a block
//! quote it's the space or spaces after the last `>` that count: one is kept.
//!
//! The rule only removes spaces that follow the container's own prefix. A
//! line whose prefix holds anything else (a list marker: `- @note: x`, or a
//! tab) is left alone.

use tessera_core::{Span, TextEdit};

use crate::Ctx;

/// The container that owns a line.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Owner {
    /// The document: no indentation.
    Document,
    /// A block quote.
    Quote,
    /// A list item. The number is the width in bytes, from the start of the
    /// line, of everything before the item's content column (outer
    /// containers' prefixes, the marker, and the spaces after it), or `None`
    /// when a tab makes it unknown.
    Item(Option<usize>),
}

/// The width of the prefix of a list item whose marker is `marker`, up to its
/// content column (CommonMark: the marker and one to four spaces, or one space
/// when the marker is followed by no text or by five or more spaces).
pub(crate) fn content_width(source: &str, marker: Span) -> Option<usize> {
    let start = source[..marker.start()]
        .rfind(['\n', '\r'])
        .map_or(0, |i| i + 1);
    if source[start..marker.start()].contains('\t') {
        return None;
    }
    let rest = &source[marker.end()..];
    let rest = &rest[..rest.find(['\n', '\r']).unwrap_or(rest.len())];
    let spaces = rest.bytes().take_while(|b| *b == b' ').count();
    if rest[spaces..].starts_with('\t') {
        return None;
    }
    let blank = rest[spaces..].is_empty();
    let width = if blank || spaces == 0 || spaces >= 5 {
        1
    } else {
        spaces
    };
    Some(marker.end() - start + width)
}

// Resolved Q73: block quotes keep the marker and one space; a
// directive on a marker's line, and indentation with a tab, are left alone.
/// Removes the extra spaces before the directive or end line whose `@` is at
/// `at`.
pub(crate) fn rule(ctx: &mut Ctx<'_>, at: usize, owner: Owner) {
    let start = ctx.line_start(at);
    let prefix = &ctx.source[start..at];
    // Only spaces and quote markers: anything else (a list marker, text, a
    // tab) means the line isn't just indentation past its container.
    if !prefix.bytes().all(|b| b == b' ' || b == b'>') {
        return;
    }
    // Where the container's own prefix ends, in bytes from the line's start.
    let keep = match owner {
        Owner::Document => {
            if prefix.contains('>') {
                return;
            }
            0
        }
        Owner::Quote => match prefix.rfind('>') {
            // The quote marker, and the one space after it that belongs to it.
            Some(marker) => (marker + 2).min(prefix.len()),
            None => return,
        },
        Owner::Item(Some(width)) if width <= prefix.len() => width,
        Owner::Item(_) => return,
    };
    if prefix[keep..].bytes().any(|b| b != b' ') {
        return;
    }
    if keep < prefix.len() {
        ctx.edits
            .push(TextEdit::delete(Span::new(start + keep, at)));
    }
}
