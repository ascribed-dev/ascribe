//! Warnings about lists and indented lines (SPEC §3.9, §4.6, §8.2 "Lists").

use tessera_core::{Builtin, Issue, Origin, Span, diagnostics};

use super::{Pass, bound_block};
use crate::tree::*;

impl Pass<'_> {
    /// Runs the list heuristics over a final list of sibling blocks.
    pub(super) fn list_heuristics(&mut self, children: &[Block]) {
        for (i, block) in children.iter().enumerate() {
            match &block.kind {
                BlockKind::List(_) => self.list_then_directive(block, children.get(i + 1)),
                BlockKind::CodeBlock(code) if !code.fenced => self.indented_code(block),
                BlockKind::Directive(line) if self.is_steps(&line.name) => {
                    self.steps_numbering(children, i);
                }
                _ => {}
            }
        }
    }

    fn is_steps(&self, name: &str) -> bool {
        self.options
            .schema(name)
            .is_some_and(|s| s.origin == Origin::Builtin(Builtin::Steps))
    }

    /// A directive line right after a list, with no blank line between, ended
    /// the list: it wasn't indented to the item's content (SPEC §3.9 rule 4).
    /// SPEC-QUESTION(Q34): only directive lines do, not end lines, and only
    /// after a list, not after a block quote.
    fn list_then_directive(&mut self, list: &Block, next: Option<&Block>) {
        let Some(next) = next else { return };
        let opener = match &next.kind {
            BlockKind::Directive(line) => line,
            BlockKind::Container(container) => &container.opener,
            BlockKind::Group(group) => match group.arms.first() {
                Some(arm) => &arm.opener,
                None => return,
            },
            _ => return,
        };
        if self.touches(list.span.end(), next.span.start()) {
            let issue = Issue::new(
                diagnostics::LIST_ENDED_BY_DIRECTIVE,
                self.location(opener.name_span),
            )
            .with_arg("name", opener.name.clone());
            self.report(issue);
        }
    }

    /// A line of an indented code block that would be a directive line if it
    /// weren't indented four or more columns past its container (SPEC §3.9
    /// rule 5).
    fn indented_code(&mut self, block: &Block) {
        let first = self.line(block.span.start());
        let last = self.line(block.span.end());
        for line in first..=last {
            let Some(span) = self.index.line_span(line) else {
                continue;
            };
            let text = self.source.get(span.start()..span.end()).unwrap_or("");
            let rest = crate::tree::strip_prefix(text);
            let Some(name) = self.directive_shape(rest) else {
                continue;
            };
            let at = span.start() + (text.len() - rest.len());
            let issue = Issue::new(
                diagnostics::DIRECTIVE_INDENTED_CODE,
                self.location(Span::new(at, at + 1 + name.len())),
            )
            .with_arg("name", name.to_owned());
            self.report(issue);
        }
    }

    /// The keyword, when `text` starts like a directive line: `@`, a known
    /// keyword, and then a space, tab, `{`, `:`, or the end.
    fn directive_shape<'t>(&self, text: &'t str) -> Option<&'t str> {
        let after = text.strip_prefix('@')?;
        let len = after
            .find(|c: char| !(c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-'))
            .unwrap_or(after.len());
        let (name, rest) = after.split_at(len);
        let ends = matches!(rest.chars().next(), None | Some(' ' | '\t' | '{' | ':'));
        let known = name == tessera_core::END_KEYWORD || self.options.schema(name).is_some();
        (!name.is_empty() && ends && known).then_some(name)
    }

    /// An ordered list that continues the numbering of the `@steps` list
    /// above it after only directive lines: usually one of them was an
    /// unindented line that split the list (SPEC §8.2 "Lists").
    fn steps_numbering(&mut self, children: &[Block], i: usize) {
        let Some(j) = bound_block(children, i) else {
            return;
        };
        let BlockKind::List(steps) = &children[j].kind else {
            return;
        };
        if !steps.ordered {
            return;
        }
        let mut k = j + 1;
        while matches!(
            children.get(k).map(|b| &b.kind),
            Some(BlockKind::Directive(_))
        ) {
            k += 1;
        }
        if k == j + 1 {
            return;
        }
        let Some(Block {
            span,
            kind: BlockKind::List(next),
        }) = children.get(k)
        else {
            return;
        };
        let expected = steps
            .start
            .unwrap_or(1)
            .saturating_add(u64::try_from(steps.items.len()).unwrap_or(u64::MAX));
        if next.ordered && next.start == Some(expected) {
            let at = next.items.first().map_or(*span, |item| item.marker);
            self.report(Issue::new(
                diagnostics::STEPS_NUMBERING_CONTINUED,
                self.location(at),
            ));
        }
    }
}
