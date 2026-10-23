//! Section summaries (SPEC §4.12): each `@summary` of a resolved page, and
//! the section it's the summary of, once includes are expanded and the
//! build's modes applied, so a summary in a fragment is the summary of the
//! section it lands in.
//!
//! A summary's section is the section of the nearest heading before it in
//! its own list of blocks, or, with none, the one its enclosing block is in;
//! with none at all, the page's opening.

use ascribe_syntax::{Block, BlockKind, Bound, Inline, PrimaryValue};

use super::tree::{ResolvedBlock, ResolvedKind, ResolvedPage};

/// The directive that gives a section its summary.
const SUMMARY: &str = "summary";

/// How a section's summary is given.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SummaryForm {
    /// `@summary: text`: its own text, which the page doesn't show.
    Text,
    /// `@summary` above a paragraph: that paragraph, shown where it is.
    Paragraph,
}

impl SummaryForm {
    /// The form's name, as the JSON output writes it.
    pub fn name(self) -> &'static str {
        match self {
            SummaryForm::Text => "text",
            SummaryForm::Paragraph => "paragraph",
        }
    }
}

/// A section's summary.
#[derive(Clone, Debug, PartialEq)]
pub struct SectionSummary {
    /// The page id of the section's heading; `None` for the page's opening,
    /// before its first heading.
    pub anchor: Option<String>,
    /// How it's given.
    pub form: SummaryForm,
    /// The block whose inline content is the summary: the `@summary` line
    /// for [`SummaryForm::Text`], the paragraph for
    /// [`SummaryForm::Paragraph`]. Its links and phrases are resolved.
    pub block: ResolvedBlock,
}

impl SectionSummary {
    /// The summary's inline content.
    pub fn inlines(&self) -> &[Inline] {
        summary_inlines(&self.block)
    }
}

/// One `@summary` found on a page.
#[derive(Clone, Copy, Debug)]
pub struct FoundSummary<'a> {
    /// The heading of the section it's in; `None` for the page's opening.
    pub heading: Option<&'a ResolvedBlock>,
    /// The `@summary` line.
    pub directive: &'a ResolvedBlock,
    /// How it's given.
    pub form: SummaryForm,
    /// The block whose inline content is the summary.
    pub block: &'a ResolvedBlock,
}

impl ResolvedPage {
    /// The summary of the section whose heading's page id is `anchor`, or,
    /// for `None`, of the page's opening.
    pub fn summary(&self, anchor: Option<&str>) -> Option<&SectionSummary> {
        self.summaries
            .iter()
            .find(|s| s.anchor.as_deref() == anchor)
    }
}

/// Every `@summary` of `blocks` that binds, in document order, each with its
/// section. A section may have more than one: the first is its summary, and
/// the checks report the rest (`summary-repeated`). A `@summary` that doesn't
/// bind, or whose block isn't a paragraph, is reported where it's written
/// and is no section's summary.
pub fn find_summaries(blocks: &[ResolvedBlock]) -> Vec<FoundSummary<'_>> {
    let mut out = Vec::new();
    walk(blocks, None, &mut out);
    out
}

/// The summary of each section of `blocks`: the first [`find_summaries`]
/// finds for it.
pub(super) fn section_summaries(blocks: &[ResolvedBlock]) -> Vec<SectionSummary> {
    let mut out: Vec<SectionSummary> = Vec::new();
    for found in find_summaries(blocks) {
        let anchor = found
            .heading
            .and_then(|h| h.heading.as_ref())
            .map(|ids| ids.page_id.clone());
        if out.iter().any(|s| s.anchor == anchor) {
            continue;
        }
        out.push(SectionSummary {
            anchor,
            form: found.form,
            block: found.block.clone(),
        });
    }
    out
}

fn walk<'a>(
    blocks: &'a [ResolvedBlock],
    enclosing: Option<&'a ResolvedBlock>,
    out: &mut Vec<FoundSummary<'a>>,
) {
    let mut heading = enclosing;
    for (i, block) in blocks.iter().enumerate() {
        if block.heading.is_some() {
            heading = Some(block);
        }
        if let Some((form, target)) = summary_of(blocks, i) {
            out.push(FoundSummary {
                heading,
                directive: block,
                form,
                block: target,
            });
        }
        block.for_each_child_list(&mut |children| walk(children, heading, out));
    }
}

/// When `blocks[i]` is a `@summary` that binds, its form and the block that
/// holds its text.
fn summary_of(blocks: &[ResolvedBlock], i: usize) -> Option<(SummaryForm, &ResolvedBlock)> {
    let block = &blocks[i];
    let line = match &block.kind {
        ResolvedKind::Leaf(Block {
            kind: BlockKind::Directive(line),
            ..
        }) if line.name == SUMMARY => line,
        _ => return None,
    };
    match line.binding? {
        Bound::Heading => Some((SummaryForm::Text, block)),
        Bound::FollowingBlock => {
            // Stacked following-block directives all bind the block the last
            // of them touches.
            let target = blocks[i + 1..].iter().find(|b| !is_following(b))?;
            matches!(
                &target.kind,
                ResolvedKind::Leaf(Block {
                    kind: BlockKind::Paragraph(_),
                    ..
                })
            )
            .then_some((SummaryForm::Paragraph, target))
        }
        Bound::Own | Bound::Unbound => None,
    }
}

/// The inline content of a summary's block: a paragraph's, or a `@summary`
/// line's text.
fn summary_inlines(block: &ResolvedBlock) -> &[Inline] {
    match &block.kind {
        ResolvedKind::Leaf(Block {
            kind: BlockKind::Paragraph(p),
            ..
        }) => &p.inlines,
        ResolvedKind::Leaf(Block {
            kind: BlockKind::Directive(line),
            ..
        }) => match &line.primary {
            Some(PrimaryValue::Text(text)) => &text.inlines,
            _ => &[],
        },
        _ => &[],
    }
}

/// Whether a block is a line-form directive that binds the next block.
fn is_following(block: &ResolvedBlock) -> bool {
    matches!(
        &block.kind,
        ResolvedKind::Leaf(Block {
            kind: BlockKind::Directive(line),
            ..
        }) if line.binding == Some(Bound::FollowingBlock)
    )
}
