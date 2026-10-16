//! Folding ranges: a container `@memo` folds to its opening line.
//!
//! A page's other folding is the client's own Markdown support (headings,
//! lists, code blocks), which knows nothing of a container. VS Code merges
//! the ranges of every provider, so these add to its.

use ascribe_core::{LineIndex, schema::Builtin};
use ascribe_syntax::{Block, BlockKind};
use lsp_types::{FoldingRange, FoldingRangeKind};

use crate::nav::Ctx;

/// The folding ranges of the requested file.
pub(crate) fn folding_ranges(ctx: &Ctx) -> Vec<FoldingRange> {
    let Some(file) = ctx.file() else {
        return Vec::new();
    };
    let index = LineIndex::new(&file.source);
    let mut out = Vec::new();
    memos(&file.document.blocks, &index, &mut out);
    out
}

/// A range from each closed container memo's opening line to its `@end`.
fn memos(blocks: &[Block], index: &LineIndex, out: &mut Vec<FoldingRange>) {
    for block in blocks {
        match &block.kind {
            BlockKind::Container(c) => {
                if c.opener.name == Builtin::Memo.name()
                    && let Some(end) = &c.end
                    && let Some(start) = index.line_col(c.opener.span.start())
                    && let Some(last) = index.line_col(end.span.start())
                    && last.line > start.line
                {
                    out.push(FoldingRange {
                        start_line: start.line,
                        end_line: last.line,
                        kind: Some(FoldingRangeKind::Comment),
                        ..FoldingRange::default()
                    });
                }
                memos(&c.children, index, out);
            }
            BlockKind::Group(g) => {
                for arm in &g.arms {
                    memos(&arm.children, index, out);
                }
            }
            BlockKind::BlockQuote(q) => memos(&q.children, index, out),
            BlockKind::List(l) => {
                for item in &l.items {
                    memos(&item.children, index, out);
                }
            }
            _ => {}
        }
    }
}
