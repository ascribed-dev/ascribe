//! Permalinks (SPEC §2.1, §4.1): the pages a build publishes whose
//! frontmatter sets `permalink`, and the headings whose `@id` is marked
//! `{permalink=true}`, each with the page and anchor it leads to.

use std::sync::Arc;

use ascribe_core::{Location, RelPath};

use super::tree::{PagePermalink, ResolvedBuild, ResolvedPage};
use crate::expand::IncludeSite;
use crate::index::{FileIndex, frontmatter_key};

/// The frontmatter key that gives a page a permalink (SPEC §2.1).
const KEY: &str = "permalink";

/// A permalink a build publishes: a page, or a marked heading on one.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Permalink {
    /// The name: the page's `permalink`, or the heading's `@id`.
    pub name: String,
    /// The content path of the page it's on.
    pub page: RelPath,
    /// That page's route.
    pub route: String,
    /// A heading's anchor on the page: its page id, which is the name. `None`
    /// for a page's own permalink, which leads to the top of the page.
    pub anchor: Option<String>,
    /// Where it's declared, in the file it's written in: the page's
    /// `permalink` line, or the heading's `@id` line in the page or in a
    /// fragment the page includes.
    pub at: Location,
    /// The includes the heading came through, outermost first. Empty for a
    /// page's own permalink and for a heading in the page's own file.
    pub via: Arc<[IncludeSite]>,
}

impl ResolvedBuild {
    /// Every permalink the build publishes, in page order and then in
    /// document order, a page's own first. A name the build publishes twice
    /// is listed twice; `permalink-duplicate` reports it.
    pub fn permalinks(&self) -> Vec<Permalink> {
        permalinks(self.pages.iter())
    }
}

/// Every permalink on these pages, in their order and then in document
/// order, each page's own first.
pub fn permalinks<'a>(pages: impl IntoIterator<Item = &'a ResolvedPage>) -> Vec<Permalink> {
    let mut out = Vec::new();
    for page in pages {
        if let Some(own) = &page.permalink {
            out.push(Permalink {
                name: own.name.clone(),
                page: page.path.clone(),
                route: page.route.clone(),
                anchor: None,
                at: own.at,
                via: Arc::from([]),
            });
        }
        for (block, heading) in page.headings() {
            let Some(at) = heading.explicit_at.filter(|_| heading.permalink) else {
                continue;
            };
            out.push(Permalink {
                name: heading.page_id.clone(),
                page: page.path.clone(),
                route: page.route.clone(),
                anchor: Some(heading.page_id.clone()),
                at,
                via: block.via.clone(),
            });
        }
    }
    out
}

/// A page's own permalink: its frontmatter `permalink`, when that's a
/// string. A value of another type is `frontmatter-type-mismatch`'s, and a
/// name with characters a name can't have is `id-invalid`'s; either way it's
/// reported where the file is checked.
pub(super) fn of_page(index: &FileIndex) -> Option<PagePermalink> {
    let name = index.frontmatter.as_ref()?.get(KEY)?.as_str()?;
    let span = frontmatter_key(index, KEY)?;
    Some(PagePermalink {
        name: name.to_owned(),
        at: Location::new(index.file, span),
    })
}
