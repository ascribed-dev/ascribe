//! Summaries inside a `@variant` arm (SPEC §4.12).
//!
//! Runs on the finished tree, since an arm's blocks hold list items, block
//! quotes, and containers whose own structure was finished first.

use ascribe_core::{Builtin, Issue, diagnostics};

use super::Pass;
use crate::tree::*;

impl Pass<'_> {
    /// Reports every `@summary` inside a `@variant` arm, at any depth.
    pub(super) fn summaries_in_arms(&mut self, blocks: &[Block], in_arm: bool) {
        for block in blocks {
            match &block.kind {
                BlockKind::Directive(line) if in_arm && line.name == Builtin::Summary.name() => {
                    let issue = Issue::new(
                        diagnostics::SUMMARY_IN_VARIANT,
                        self.location(line.name_span),
                    );
                    self.report(issue);
                }
                BlockKind::BlockQuote(quote) => self.summaries_in_arms(&quote.children, in_arm),
                BlockKind::List(list) => {
                    for item in &list.items {
                        self.summaries_in_arms(&item.children, in_arm);
                    }
                }
                BlockKind::Container(container) => {
                    self.summaries_in_arms(&container.children, in_arm);
                }
                BlockKind::Group(group) => {
                    let arm = in_arm || group.name == Builtin::Variant.name();
                    for each in &group.arms {
                        self.summaries_in_arms(&each.children, arm);
                    }
                }
                _ => {}
            }
        }
    }
}
