//! A permalink's section: the blocks of its page that say what the
//! permalink names, for a reader shown that part of the page on its own
//! (`_ascribe/help.json`, output-layout contract §1.7).

use super::permalinks::Permalink;
use super::tree::{ResolvedArm, ResolvedBlock, ResolvedBuild, ResolvedKind, ResolvedPage};

/// A part of a resolved page, borrowed from it: a heading's section, or the
/// page's opening.
#[derive(Clone, Debug)]
pub struct PageSection<'a> {
    /// The heading, for a heading's section; `None` for a page's opening.
    pub heading: Option<&'a ResolvedBlock>,
    /// The blocks after the heading, up to the next heading at its level or
    /// above or the end of the blocks it's among, whichever comes first. A
    /// page's opening is its blocks before the first one that is or holds a
    /// heading.
    pub blocks: &'a [ResolvedBlock],
    /// The groups the heading is in, outermost first, each with the arm
    /// that holds it. Empty for a page's opening.
    pub arms: Vec<(&'a ResolvedBlock, &'a ResolvedArm)>,
}

impl ResolvedPage {
    /// The section of the heading whose page id is `anchor`, or, for
    /// `None`, the page's opening. `None` when the page has no such heading.
    ///
    /// A heading's section runs from the block after it to the next heading
    /// at its level or above, or to the end of the container the heading is
    /// in (a variant arm, a note), whichever comes first.
    pub fn section(&self, anchor: Option<&str>) -> Option<PageSection<'_>> {
        let Some(anchor) = anchor else {
            let end = self
                .blocks
                .iter()
                .position(holds_heading)
                .unwrap_or(self.blocks.len());
            return Some(PageSection {
                heading: None,
                blocks: &self.blocks[..end],
                arms: Vec::new(),
            });
        };
        let mut arms = Vec::new();
        find(&self.blocks, anchor, &mut arms)
    }
}

impl ResolvedBuild {
    /// A permalink's page and its section in it: the page
    /// [`ResolvedBuild::permalinks`] records for it, so a heading in a
    /// fragment is cut from that page.
    pub fn section(&self, permalink: &Permalink) -> Option<(&ResolvedPage, PageSection<'_>)> {
        let page = self.page(&permalink.page)?;
        Some((page, page.section(permalink.anchor.as_deref())?))
    }
}

fn find<'a>(
    blocks: &'a [ResolvedBlock],
    anchor: &str,
    arms: &mut Vec<(&'a ResolvedBlock, &'a ResolvedArm)>,
) -> Option<PageSection<'a>> {
    for (i, block) in blocks.iter().enumerate() {
        if let Some(ids) = &block.heading
            && ids.page_id == anchor
        {
            let rest = &blocks[i + 1..];
            let next = rest
                .iter()
                .position(|b| b.heading.as_ref().is_some_and(|h| h.level <= ids.level));
            let end = next.unwrap_or(rest.len());
            return Some(PageSection {
                heading: Some(block),
                blocks: &rest[..end],
                arms: std::mem::take(arms),
            });
        }
        let found = match &block.kind {
            ResolvedKind::Leaf(_) => None,
            ResolvedKind::BlockQuote { children } | ResolvedKind::Container { children, .. } => {
                find(children, anchor, arms)
            }
            ResolvedKind::List { items, .. } => items
                .iter()
                .find_map(|item| find(&item.children, anchor, arms)),
            ResolvedKind::Group { arms: group, .. } => group.iter().find_map(|arm| {
                arms.push((block, arm));
                let found = find(&arm.children, anchor, arms);
                if found.is_none() {
                    arms.pop();
                }
                found
            }),
        };
        if found.is_some() {
            return found;
        }
    }
    None
}

/// Whether a block is a heading or holds one.
fn holds_heading(block: &ResolvedBlock) -> bool {
    let mut found = false;
    block.visit(&mut |b| found |= b.heading.is_some());
    found
}
