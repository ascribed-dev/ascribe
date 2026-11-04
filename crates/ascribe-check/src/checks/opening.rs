//! `page-opening-missing`: a page whose first block after its title isn't a
//! paragraph written in the page, such as one that opens straight into an
//! `@include`, a heading, or a list. It's structural, so it holds in any
//! language: what the paragraph says is left to a project's prose rules.

use ascribe_core::{Issue, diagnostics};
use ascribe_syntax::{Block, BlockKind, Bound};

use super::Ctx;

impl Ctx<'_> {
    /// Reports the first block of a page's top level when it isn't a
    /// paragraph, after passing over what doesn't open the page: a level-1
    /// heading, which repeats the title; a memo and an HTML comment, which
    /// aren't published; and a directive that binds to the block below it.
    pub(super) fn check_opening(&mut self, blocks: &[Block]) {
        if self.model.checks.is_off(diagnostics::PAGE_OPENING_MISSING)
            || self.model.is_fragment(self.file.path.as_str())
            || self.page_type.is_none()
        {
            return;
        }
        let mut level_one = false;
        let Some(first) = blocks.iter().find(|b| match &b.kind {
            BlockKind::Heading(h) if h.level == 1 && !level_one => {
                level_one = true;
                false
            }
            kind => !self.passed_over(kind, b),
        }) else {
            return;
        };
        let Some(what) = self.what(first) else {
            return;
        };
        let issue = Issue::new(diagnostics::PAGE_OPENING_MISSING, self.location(first.span))
            .with_arg("first", what);
        self.report(issue);
    }

    /// Whether a block at the top of a page is passed over in looking for
    /// its opening.
    fn passed_over(&self, kind: &BlockKind, block: &Block) -> bool {
        match kind {
            BlockKind::Directive(line) => {
                line.name == "memo" || line.binding == Some(Bound::FollowingBlock)
            }
            BlockKind::Container(c) => c.opener.name == "memo",
            BlockKind::HtmlBlock(_) => self
                .source()
                .get(block.span.range())
                .is_some_and(|t| t.trim_start().starts_with("<!--")),
            // A title line that couldn't attach is reported on its own.
            BlockKind::Title(_) | BlockKind::End(_) => true,
            _ => false,
        }
    }

    /// What the page opens with, for the message; `None` for a paragraph.
    fn what(&self, block: &Block) -> Option<String> {
        let directive = |name: &str| format!("an `@{name}`");
        let what = match &block.kind {
            BlockKind::Paragraph(_) => return None,
            BlockKind::Heading(_) => "a heading".to_owned(),
            BlockKind::CodeBlock(_) => "a code block".to_owned(),
            BlockKind::BlockQuote(_) => "a block quote".to_owned(),
            BlockKind::List(_) => "a list".to_owned(),
            BlockKind::HtmlBlock(_) => "HTML".to_owned(),
            BlockKind::ThematicBreak => "a thematic break".to_owned(),
            BlockKind::Table(_) => "a table".to_owned(),
            BlockKind::Directive(line) => directive(&line.name),
            BlockKind::Container(c) => directive(&c.opener.name),
            BlockKind::Group(g) => directive(&g.name),
            BlockKind::End(_) | BlockKind::Title(_) => return None,
        };
        Some(what)
    }
}
