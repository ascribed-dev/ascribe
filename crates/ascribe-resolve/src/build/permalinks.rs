//! Permalinks (SPEC §4.1): the headings a build publishes whose `@id` is
//! marked `{permalink=true}`, each with the page and anchor it leads to.

use std::sync::Arc;

use ascribe_core::{Location, RelPath};

use super::tree::{ResolvedBuild, ResolvedPage};
use crate::expand::IncludeSite;

/// A permalink a build publishes: a marked heading on one page.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Permalink {
    /// The name: the heading's `@id`.
    pub name: String,
    /// The content path of the page the heading is published on.
    pub page: RelPath,
    /// That page's route.
    pub route: String,
    /// The heading's anchor on the page: its page id, which is the name.
    pub anchor: String,
    /// The `@id` line, in the file it's written in: the page, or a fragment
    /// the page includes.
    pub at: Location,
    /// The includes the heading came through, outermost first. Empty for a
    /// heading in the page's own file.
    pub via: Arc<[IncludeSite]>,
}

impl ResolvedBuild {
    /// Every permalink the build publishes, in page order and then in
    /// document order. A name the build publishes twice is listed twice;
    /// `permalink-duplicate` reports it.
    pub fn permalinks(&self) -> Vec<Permalink> {
        permalinks(self.pages.iter())
    }
}

/// Every permalink on these pages, in their order and then in document
/// order.
pub fn permalinks<'a>(pages: impl IntoIterator<Item = &'a ResolvedPage>) -> Vec<Permalink> {
    let mut out = Vec::new();
    for page in pages {
        for (block, heading) in page.headings() {
            let Some(at) = heading.explicit_at.filter(|_| heading.permalink) else {
                continue;
            };
            out.push(Permalink {
                name: heading.page_id.clone(),
                page: page.path.clone(),
                route: page.route.clone(),
                anchor: heading.page_id.clone(),
                at,
                via: block.via.clone(),
            });
        }
    }
    out
}
