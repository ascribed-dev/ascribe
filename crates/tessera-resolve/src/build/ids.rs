//! Heading ids (SPEC §5.5, §9.2 step 5): every heading's page id.
//!
//! A heading's page id is its `@id`, or its slug, numbered across the whole
//! expanded page after includes and build modes, in one scope per page
//! (phase 09). Source ids are per file and were computed with the index.

use tessera_core::{Location, Slugger};
use tessera_syntax::BlockKind;

use super::tree::{HeadingIds, ResolvedBlock, ResolvedKind};
use crate::project::Project;
use tessera_core::SlugScope;

/// Assigns every heading of the page its ids, in document order.
pub(crate) fn assign(project: &Project, blocks: &mut [ResolvedBlock], slugger: &dyn Slugger) {
    let mut scope = slugger.new_scope();
    walk(project, blocks, scope.as_mut());
}

fn walk(project: &Project, blocks: &mut [ResolvedBlock], scope: &mut dyn SlugScope) {
    for block in blocks {
        if let ResolvedKind::Leaf(b) = &block.kind
            && matches!(b.kind, BlockKind::Heading(_))
            && let Some(index) = project.file_by_id(block.file)
            && let Some(heading) = index.headings.iter().find(|h| h.span == block.span)
        {
            // Only headings without `@id` go through the scope: explicit ids
            // don't take part in numbering (Q7).
            let page_id = match &heading.explicit_id {
                Some(explicit) => explicit.id.clone(),
                None => scope.slug(&heading.text),
            };
            block.heading = Some(HeadingIds {
                level: heading.level,
                text: heading.text.clone(),
                source_id: heading.source_id.clone(),
                page_id,
                explicit: heading.explicit_id.is_some(),
                explicit_at: heading
                    .explicit_id
                    .as_ref()
                    .map(|e| Location::new(block.file, e.span)),
            });
        }
        for children in block.child_lists_mut() {
            walk(project, children, scope);
        }
    }
}
