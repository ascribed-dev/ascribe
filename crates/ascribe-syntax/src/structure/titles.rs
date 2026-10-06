//! Title lines (SPEC §3.7).
//!
//! The block parser leaves a title line as a one-line paragraph that the
//! directive line below it interrupted. This attaches it.

use ascribe_core::{Issue, Span, TitleRule, diagnostics};

use super::Pass;
use crate::tree::*;

/// What a one-line paragraph could be, as a title.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Shape {
    /// A `.` and then a character that is neither whitespace nor `.`.
    Title,
    /// A `.` and then whitespace: never a title, but probably meant as one.
    DotSpace,
}

impl Pass<'_> {
    /// Attaches each title line to the directive directly below it, and
    /// reports the `.` lines that look like titles but can't be.
    ///
    /// A title line must be alone in its paragraph (a `.` line that continues
    /// a longer paragraph is text), directly above the directive line.
    pub(super) fn attach_titles(&mut self, blocks: Vec<Block>) -> Vec<Block> {
        let mut out = Vec::with_capacity(blocks.len());
        let mut blocks = blocks.into_iter().peekable();
        while let Some(block) = blocks.next() {
            if let BlockKind::Paragraph(paragraph) = &block.kind
                && let Some(next) = blocks.peek()
                && let BlockKind::Directive(line) = &next.kind
                && self.touches(block.span.end(), line.span.start())
                && let Some(shape) = self.shape(block.span)
            {
                let accepts = self
                    .options
                    .schema(&line.name)
                    .is_some_and(|s| s.title != TitleRule::None);
                match (shape, accepts) {
                    (Shape::Title, true) => {
                        let title = self.title_line(&block, paragraph);
                        let Some(Block {
                            kind: BlockKind::Directive(mut line),
                            ..
                        }) = blocks.next()
                        else {
                            continue;
                        };
                        let span = Span::new(title.span.start(), line.span.end());
                        line.title = Some(title);
                        out.push(Block {
                            span,
                            kind: BlockKind::Directive(line),
                        });
                        continue;
                    }
                    // The line stays a paragraph, and the warning is
                    // reported.
                    (Shape::Title, false) => {
                        let issue =
                            Issue::new(diagnostics::TITLE_NOT_ACCEPTED, self.location(block.span))
                                .with_arg("name", line.name.clone());
                        self.report(issue);
                    }
                    (Shape::DotSpace, true) => {
                        let text = self.source.get(block.span.start() + 1..block.span.end());
                        let issue =
                            Issue::new(diagnostics::TITLE_DOT_SPACE, self.location(block.span))
                                .with_arg("name", line.name.clone())
                                .with_arg("title", text.unwrap_or("").trim());
                        self.report(issue);
                    }
                    (Shape::DotSpace, false) => {}
                }
            }
            out.push(block);
        }
        out
    }

    /// Whether the paragraph is one line that starts like a title line.
    fn shape(&self, span: Span) -> Option<Shape> {
        let text = self.source.get(span.start()..span.end())?;
        if text.contains(['\n', '\r']) {
            return None;
        }
        let mut chars = text.chars();
        if chars.next()? != '.' {
            return None;
        }
        match chars.next()? {
            '.' => None,
            c if c.is_whitespace() => Some(Shape::DotSpace),
            _ => Some(Shape::Title),
        }
    }

    /// The title line a one-line paragraph makes: everything after the dot,
    /// as the paragraph's own inlines.
    fn title_line(&self, block: &Block, paragraph: &Paragraph) -> TitleLine {
        let start = block.span.start();
        let mut inlines = paragraph.inlines.clone();
        let mut drop_first = false;
        if let Some(first) = inlines.first_mut()
            && first.span.start() == start
            && let InlineKind::Text(value) = &mut first.kind
            && value.starts_with('.')
        {
            value.remove(0);
            first.span = Span::new(first.span.start() + 1, first.span.end().max(start + 1));
            drop_first = value.is_empty() && first.span.is_empty();
        }
        if drop_first {
            inlines.remove(0);
        }
        TitleLine {
            span: block.span,
            dot: Span::new(start, start + 1),
            content: Span::new(start + 1, block.span.end()),
            inlines,
        }
    }
}
