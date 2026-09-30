//! Binding (SPEC §3.8, §4.4, §4.6).
//!
//! Runs on a final list of sibling blocks: the document's, a list item's, a
//! block quote's, a container's, or an arm's. A section, for heading-bound
//! directives, is a heading and what follows it in that same list.

use tessera_core::{Binding as SchemaBinding, Builtin, Issue, Origin, diagnostics};

use super::{Pass, Scope};
use crate::tree::*;

/// What a directive line's schema makes of it, given where it sits.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Class {
    /// Not a directive line.
    Other,
    /// The schema has no binding for the line.
    NoBinding,
    /// Its own primary, or nothing.
    Own,
    /// The heading it's at the top of the section of.
    Heading,
    /// A heading-bound directive that isn't at the top of its section.
    Misplaced,
    /// A line whose attribute block never closes. Its form and primary can't
    /// be known, so the head is what's reported, and nothing about its
    /// binding is.
    Unreadable,
    /// The following block.
    Block,
}

impl Pass<'_> {
    /// Sets the binding of every line-form directive in `children` and reports
    /// the ones that can't bind.
    pub(super) fn bind(&mut self, children: &mut [Block], scope: Scope) {
        let mut classes: Vec<Class> = Vec::with_capacity(children.len());
        for (i, block) in children.iter().enumerate() {
            let class = match &block.kind {
                BlockKind::Directive(line) => {
                    // SPEC §3.8, SPEC §3.8: a section starts at a heading in this
                    // same list of blocks. Before the first one there is no
                    // section, so nothing is at the top of one.
                    let top = i > 0
                        && match &children[i - 1].kind {
                            BlockKind::Heading(_) => true,
                            BlockKind::Directive(_) => classes[i - 1] == Class::Heading,
                            _ => false,
                        };
                    self.classify(line, top)
                }
                _ => Class::Other,
            };
            classes.push(class);
        }

        let mut bound: Vec<Option<Bound>> = vec![None; children.len()];
        for i in 0..children.len() {
            let BlockKind::Directive(line) = &children[i].kind else {
                continue;
            };
            bound[i] = match classes[i] {
                Class::Other | Class::NoBinding => None,
                Class::Unreadable => Some(Bound::Unbound),
                Class::Own => Some(Bound::Own),
                Class::Heading => Some(Bound::Heading),
                Class::Misplaced => {
                    let issue = Issue::new(
                        diagnostics::BINDING_NOT_SECTION_TOP,
                        self.location(line.name_span),
                    )
                    .with_arg("name", line.name.clone());
                    self.report(issue);
                    Some(Bound::Unbound)
                }
                Class::Block => Some(self.bind_block(children, &classes, i, scope)),
            };
        }
        for (block, bound) in children.iter_mut().zip(bound) {
            if let BlockKind::Directive(line) = &mut block.kind {
                line.binding = bound;
            }
        }
    }

    fn classify(&self, line: &DirectiveLine, top: bool) -> Class {
        if !line.attributes_closed {
            // SPEC §3.3.
            return Class::Unreadable;
        }
        let Some(schema) = self.options.schema(&line.name) else {
            return Class::NoBinding;
        };
        match schema.binding {
            None => Class::NoBinding,
            Some(SchemaBinding::SelfBound) => Class::Own,
            Some(SchemaBinding::Heading) if top => Class::Heading,
            Some(SchemaBinding::Heading) => Class::Misplaced,
            // With a primary, its own content; otherwise the block below.
            Some(SchemaBinding::Block) => match line.primary {
                Some(PrimaryValue::Text(_)) => Class::Own,
                _ => Class::Block,
            },
            // SPEC §4.4: the section at the top of one, else the block.
            Some(SchemaBinding::HeadingOrBlock) if top => Class::Heading,
            Some(SchemaBinding::HeadingOrBlock) => Class::Block,
        }
    }

    /// Binds a following-block directive, checking what it binds.
    fn bind_block(
        &mut self,
        children: &[Block],
        classes: &[Class],
        i: usize,
        scope: Scope,
    ) -> Bound {
        let BlockKind::Directive(line) = &children[i].kind else {
            return Bound::Unbound;
        };
        let at = self.location(line.name_span);
        // SPEC §3.8: directives that bind the following block stack:
        // they all describe the block the last of them touches. A line-form
        // directive that is its own text (`@note: text`) renders as a block,
        // so it can be bound; one that stands alone (`@include`, `@id`) or an
        // end line can't.
        let mut j = i + 1;
        while classes.get(j) == Some(&Class::Block) {
            j += 1;
        }
        let target = children.get(j).map(|b| &b.kind);
        let text_directive = matches!(
            target,
            Some(BlockKind::Directive(l))
                if classes.get(j) == Some(&Class::Own)
                    && matches!(l.primary, Some(PrimaryValue::Text(_)))
        );
        match target {
            None => {
                let issue = Issue::new(diagnostics::BINDING_NO_BLOCK, at)
                    .with_arg("name", line.name.clone())
                    .with_arg("container", scope.noun());
                self.report(issue);
                return Bound::Unbound;
            }
            Some(BlockKind::End(_) | BlockKind::Directive(_) | BlockKind::Title(_))
                if !text_directive =>
            {
                let issue = Issue::new(diagnostics::BINDING_NO_BLOCK, at)
                    .with_arg("name", line.name.clone())
                    .with_arg("container", scope.noun());
                self.report(issue);
                return Bound::Unbound;
            }
            Some(BlockKind::Heading(_)) => {
                let issue = Issue::new(diagnostics::BINDING_HEADING, at)
                    .with_arg("name", line.name.clone());
                self.report(issue);
                return Bound::Unbound;
            }
            Some(_) => {}
        }
        // A blank line hides what the directive annotates (SPEC §3.8).
        if let Some(next) = children.get(i + 1)
            && !self.touches(children[i].span.end(), next.span.start())
        {
            let issue = Issue::new(
                diagnostics::BINDING_BLANK_LINE,
                self.location(line.name_span),
            )
            .with_arg("name", line.name.clone());
            self.report(issue);
        }
        let is_steps = self
            .options
            .schema(&line.name)
            .is_some_and(|s| s.origin == Origin::Builtin(Builtin::Steps));
        if is_steps
            && let Some(kind) = target
            && !matches!(kind, BlockKind::List(list) if list.ordered)
        {
            let issue = Issue::new(diagnostics::STEPS_NOT_ORDERED_LIST, at)
                .with_arg("found", describe(kind));
            self.report(issue);
        }
        Bound::FollowingBlock
    }
}

/// A block, as a message names it.
fn describe(kind: &BlockKind) -> &'static str {
    match kind {
        BlockKind::Heading(_) => "a heading",
        BlockKind::Paragraph(_) => "a paragraph",
        BlockKind::CodeBlock(_) => "a code block",
        BlockKind::BlockQuote(_) => "a block quote",
        BlockKind::List(list) if list.ordered => "an ordered list",
        BlockKind::List(_) => "a bullet list",
        BlockKind::HtmlBlock(_) => "an HTML block",
        BlockKind::ThematicBreak => "a thematic break",
        BlockKind::Table(_) => "a table",
        BlockKind::Container(_) | BlockKind::Group(_) => "a container",
        BlockKind::Directive(_) | BlockKind::End(_) | BlockKind::Title(_) => "a directive",
    }
}

/// The index of the heading a heading-bound directive binds, in the same
/// list of blocks (`blocks[index]` is the directive).
///
/// Returns `None` when `blocks[index]` isn't a directive bound to a heading.
pub fn bound_heading(blocks: &[Block], index: usize) -> Option<usize> {
    let BlockKind::Directive(line) = &blocks.get(index)?.kind else {
        return None;
    };
    if line.binding != Some(Bound::Heading) {
        return None;
    }
    // Only heading-bound directives stand between it and its heading.
    (0..index)
        .rev()
        .find(|&k| !matches!(&blocks[k].kind, BlockKind::Directive(l) if l.binding == Some(Bound::Heading)))
        .filter(|&k| matches!(blocks[k].kind, BlockKind::Heading(_)))
}

/// The index of the block a following-block directive binds, in the same list
/// of blocks (`blocks[index]` is the directive). Directives that stack all
/// bind the same block.
///
/// Returns `None` when `blocks[index]` isn't a directive bound to a block.
pub fn bound_block(blocks: &[Block], index: usize) -> Option<usize> {
    let BlockKind::Directive(line) = &blocks.get(index)?.kind else {
        return None;
    };
    if line.binding != Some(Bound::FollowingBlock) {
        return None;
    }
    (index + 1..blocks.len()).find(|&k| {
        !matches!(&blocks[k].kind, BlockKind::Directive(l) if l.binding == Some(Bound::FollowingBlock))
    })
}
