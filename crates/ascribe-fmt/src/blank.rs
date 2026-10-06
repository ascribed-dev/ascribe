//! The blank line between a following-block directive and its block
//! (SPEC §3.8, §8.3).
//!
//! A following-block directive belongs directly above the block it
//! annotates, touching it. A blank line between them is allowed but hides
//! what the directive annotates, and canonical form removes it. Directives
//! that stack (SPEC §3.8) are all removed from the blank lines that separate
//! them from what follows, so a stack ends up touching its block.
//!
//! Only lines that are blank are removed. A line that holds anything else,
//! such as a link reference definition (comrak consumes definitions, so they
//! aren't blocks), is never touched; the rule then leaves the gap alone.
//!
//! A gap between two blocks of a list item is left alone too: it may be what
//! makes the list loose, and closing it would change how the list renders.
//!
//! A directive whose text primary would swallow the block once nothing
//! separates them is left alone too.

use ascribe_core::{Span, TextEdit};
use ascribe_syntax::{Block, BlockKind, Bound, PrimaryValue};

use crate::{Ctx, Owner};

// A gap with a text primary before it, with a line that
// isn't blank (a definition), or between blocks of a list item is left alone.
/// Applies the rule to the directive at `blocks[index]`, if it's a
/// following-block directive with something after it in the same container.
pub(crate) fn rule(ctx: &mut Ctx<'_>, blocks: &[Block], index: usize, owner: Owner) {
    let BlockKind::Directive(line) = &blocks[index].kind else {
        return;
    };
    // A blank line between two blocks of a list item is what makes its list
    // loose, and closing the gap could make the list tight, which changes how
    // the whole list renders. The gap stays, and `binding-blank-line` still
    // reports it.
    if matches!(owner, Owner::Item(_)) {
        return;
    }
    if line.binding != Some(Bound::FollowingBlock)
        // Its own text would go on into the block.
        || matches!(line.primary, Some(PrimaryValue::Text(_)))
    {
        return;
    }
    let Some(next) = blocks.get(index + 1) else {
        return;
    };
    let head = Span::new(blocks[index].span.start(), line.span.end());
    if ctx.has_error(head) {
        return;
    }
    // The lines strictly between the directive's last line and the line the
    // next block starts on.
    let from = ctx.next_line(line.span.end());
    let to = ctx.line_start(next.span.start());
    if from >= to {
        return;
    }
    let gap = Span::new(from, to);
    let lines = &ctx.source[gap.range()];
    let is_blank = |l: &str| {
        l.chars()
            .all(|c| matches!(c, ' ' | '\t' | '>' | '\r' | '\n'))
    };
    if !lines.split_inclusive(['\n', '\r']).all(is_blank) || touches_definition(ctx, gap) {
        return;
    }
    ctx.edits.push(TextEdit::delete(gap));
}

/// Whether a link reference definition has any of its bytes in `gap`.
fn touches_definition(ctx: &Ctx<'_>, gap: Span) -> bool {
    ctx.doc
        .definitions
        .iter()
        .any(|d| d.span.start() < gap.end() && gap.start() < d.span.end())
}
