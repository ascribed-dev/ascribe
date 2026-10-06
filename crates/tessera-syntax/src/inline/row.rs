//! The attribute block at the end of a table row's first cell (SPEC §4.4).
//!
//! The fork's inline parser reads the block as text, so this finds it in the
//! cell's source, reads it with `tessera_core::parse_attribute_block`, and
//! takes it out of the cell's inlines. It runs before phrase candidates are
//! split out, so the cell's text is still whole.

use tessera_core::parse_attribute_block;

use super::Pass;
use super::phrase::push_text;
use crate::tree::*;

impl Pass<'_> {
    /// Attaches the attribute block that ends `row`'s first cell, if there is
    /// one: a `{` at the start of the cell or after whitespace, in text, whose
    /// block closes at the end of the cell. A block with no `=` in it is a
    /// phrase candidate or literal braces (SPEC §5.1), not a row's block.
    pub(super) fn row_attributes(&mut self, row: &mut TableRow) {
        let Some(cell) = row.cells.first_mut() else {
            return;
        };
        let Some(text) = self.source.get(cell.span.range()) else {
            return;
        };
        let end = text.trim_end().len();
        if !text[..end].ends_with('}') {
            return;
        }
        let base = cell.span.start();
        let in_text = |from: usize| {
            cell.inlines
                .iter()
                .filter(|i| i.span.end() > from && i.span.start() < base + end)
                .all(|i| matches!(i.kind, InlineKind::Text(_)))
        };
        let found = text[..end].match_indices('{').find_map(|(i, _)| {
            let after_space = i == 0 || text[..i].ends_with(char::is_whitespace);
            if !after_space || !text[i..end].contains('=') || !in_text(base + i) {
                return None;
            }
            let parsed = parse_attribute_block(&text[i..end], base + i, self.file)?;
            (parsed.closed && parsed.len == end - i).then_some((i, parsed))
        });
        let Some((at, parsed)) = found else {
            return;
        };
        self.issues.extend(parsed.issues);
        row.attributes = Some(parsed.block);

        // The cell's content ends before the block, without the space.
        let content_end = base + text[..at].trim_end().len();
        let old = std::mem::take(&mut cell.inlines);
        for inline in old {
            if inline.span.end() <= content_end {
                cell.inlines.push(inline);
            } else if inline.span.start() < content_end {
                // Only text reaches the block (`in_text`).
                let from = inline.span.start() - base;
                push_text(text, base, from, content_end - base, &mut cell.inlines);
            }
        }
    }
}
