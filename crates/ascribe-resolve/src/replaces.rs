//! A page's `replaces` (SPEC §2.1): the page it takes the place of, written
//! as a link destination is. A versioned build follows it when a link names
//! a page that isn't in a line (SPEC §9.3); in any other build the two are
//! different pages.

use std::collections::BTreeMap;

use ascribe_core::{RelPath, Span};
use ascribe_model::ContentModel;
use ascribe_syntax::frontmatter_lines;

use crate::index::{FileIndex, FileKind, RefKind, Target, target_of};
use crate::project::Project;

/// The reserved frontmatter key.
pub const REPLACES: &str = "replaces";

/// The source file a `replaces` value names, read from the page it's
/// written on as a link destination is: relative to the page, or to the
/// content root with a leading `/`. `None` when it can't name a source file:
/// an external URL, a file that isn't `.md`, or a destination with `#` and a
/// heading id, since it names a page and not a place on one.
pub fn replaces_target(written: &str, page: &RelPath, model: &ContentModel) -> Option<RelPath> {
    match target_of(RefKind::Link, written, &[], page, model) {
        Target::Local(local) if local.source && local.fragment.is_none() => local.path,
        _ => None,
    }
}

/// Where a page's `replaces` value is written in its file: its value on the
/// frontmatter's top-level `replaces` line. `None` when the page has none.
pub fn replaces_span(index: &FileIndex) -> Option<Span> {
    frontmatter_value_span(index, REPLACES)
}

/// Where a top-level frontmatter key's value is written in a page's file.
/// `None` when the page has no such line.
pub(crate) fn frontmatter_value_span(index: &FileIndex, key: &str) -> Option<Span> {
    let fm = index.document.frontmatter.as_ref()?;
    let text = index.source.get(fm.content.range())?;
    let line = frontmatter_lines(text)
        .into_iter()
        .find(|l| l.indent == 0 && l.item.is_none() && l.key == Some(key))?;
    let start = fm.content.start() + line.value_start;
    Some(Span::new(start, start + line.value.len()))
}

impl Project {
    /// The page a page's `replaces` names, when it's a page of the project.
    /// `None` for a file that isn't a page, one with no `replaces`, and one
    /// whose `replaces` names anything else (the file-level checks report
    /// that).
    pub fn replaces(&self, page: &RelPath) -> Option<RelPath> {
        let index = self.file(page)?;
        if index.kind != FileKind::Page {
            return None;
        }
        let written = index.frontmatter.as_ref()?.get(REPLACES)?.as_str()?;
        let target = replaces_target(written, page, self.model())?;
        (self.file(&target)?.kind == FileKind::Page).then_some(target)
    }

    /// Every page's `replaces` that names a page, by the page that says it.
    pub fn replacements(&self) -> BTreeMap<RelPath, RelPath> {
        self.pages()
            .filter_map(|p| Some((p.path.clone(), self.replaces(&p.path)?)))
            .collect()
    }

    /// The pages whose `replaces` names `page`, in path order.
    pub fn replaced_by(&self, page: &RelPath) -> Vec<RelPath> {
        self.replacements()
            .into_iter()
            .filter(|(_, replaced)| replaced == page)
            .map(|(by, _)| by)
            .collect()
    }
}
