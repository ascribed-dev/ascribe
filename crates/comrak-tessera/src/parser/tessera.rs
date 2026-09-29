//! TESSERA: the block parser's handling of Ascribe lines. This whole file is
//! Ascribe's; see `FORK.md` and [`crate::tessera`].
//!
//! How a Ascribe line fits comrak's block parser:
//!
//! - **Starting.** [`Parser::handle_tessera_line`] runs in `open_new_blocks`
//!   with the other block starts, so it's tried only at line start, after the
//!   open containers have consumed their markers and indentation, and only
//!   when the line isn't indented four or more columns (that's an indented
//!   code block, or paragraph continuation text). Like an ATX heading, it
//!   closes an open paragraph and can't be a lazy continuation line: an
//!   unindented Ascribe line after a list item or blockquote paragraph closes
//!   those containers, because `add_child` closes every container that can't
//!   hold it. The `\@` escape (SPEC §2.3) needs nothing: the line then
//!   starts with `\`.
//! - **Without a text primary**, the node takes its own line and is closed by
//!   the next one, as a heading is.
//! - **With a text primary**, the node gets a child paragraph that starts at
//!   the primary's first character. The paragraph then continues, is
//!   interrupted, and continues lazily exactly as any paragraph does, and
//!   comrak's inline parser parses it. `check_open_blocks` keeps the node open
//!   while its paragraph is.
//! - **The primary stays a paragraph.** It's inline content (SPEC §3.4), so a
//!   setext underline, a table delimiter row, or a leading link reference
//!   definition doesn't change it; see [`is_text_primary`].

use crate::nodes::{Node, NodeValue};
use crate::parser::Parser;
use crate::tessera::{NodeTesseraLine, scan_line};

impl<'a, 'o, 'c> Parser<'a, 'o, 'c>
where
    'c: 'o,
{
    /// Opens a Ascribe line if the line is one. On success, `container` is
    /// the new node, or its primary's paragraph when it has a text primary,
    /// and the caller adds the rest of the line to it.
    pub(super) fn handle_tessera_line(&mut self, container: &mut Node<'a>, line: &str) -> bool {
        let Some(options) = self.options.extension.tessera.as_deref() else {
            return false;
        };
        // Up to three spaces of indentation beyond the container's are
        // allowed, as for an ATX heading; `self.first_nonspace` skips them.
        // SPEC §1.5 and §3.9 rule 5 (resolved Q1).
        let start = self.first_nonspace;
        let Some(scanned) = scan_line(&line[start..], options) else {
            return false;
        };

        let raw = &line[start..start + scanned.content_end];
        let ntl = NodeTesseraLine {
            raw: raw.to_string(),
            name: raw[1..scanned.name_end].to_string(),
            text_primary: scanned.text_primary,
        };
        *container = self.add_child(container, NodeValue::TesseraLine(Box::new(ntl)), start + 1);

        if let Some(primary) = scanned.text_primary {
            let offset = self.offset;
            self.advance_offset(line, start + primary - offset, false);
            *container = self.add_child(container, NodeValue::Paragraph, start + primary + 1);
        }

        true
    }
}

/// Whether `node` is the paragraph holding a Ascribe line's text primary.
///
/// The primary is inline content (SPEC §3.4), never a block, so the parser
/// doesn't turn it into a setext heading or a table header, and doesn't take
/// link reference definitions from its start.
///
/// Only the parent is checked, since a Ascribe line's only possible child is
/// that paragraph. That keeps this safe to call while `node` itself is
/// borrowed, as it is during `finalize`. The parent may be borrowed too: some
/// containers, such as multiline blockquotes, finalize their children while
/// holding their own data. A parent that's borrowed isn't a Ascribe line,
/// because the parser never holds a Ascribe line's data while it works on
/// the line's paragraph, so a failed borrow means "no".
// SPEC §3.4: a text primary is always inline content (resolved Q2).
pub(super) fn is_text_primary(node: Node<'_>) -> bool {
    node.parent().is_some_and(|parent| {
        parent
            .data
            .try_borrow()
            .is_ok_and(|ast| matches!(ast.value, NodeValue::TesseraLine(..)))
    })
}
