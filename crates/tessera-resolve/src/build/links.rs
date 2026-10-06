//! Links (SPEC §5.2, §9.2 step 6): resolving each file-path link from its
//! source id to the target heading's page id, filling empty link text from
//! the target's title, and computing each link's route.
//!
//! What a destination names comes from the source index
//! ([`crate::Resolution`], which is `tessera_resolve::references`, the same
//! rules `ascribe check` runs). This pass adds what only a build knows: which
//! pages the build publishes, and what a heading's page id is there. It
//! records the links whose target a build removes (`link-id-removed`,
//! `link-page-dropped`) for the page-level checks to report, and carries the
//! assets the
//! surviving content references.

use tessera_core::{Issue, Location, RelPath, Span, diagnostics};
use tessera_syntax::{Inline, InlineKind};

use super::BuildResolver;
use super::inlines::{find_mut, own_lists_mut, own_ranges};
use super::tree::{LinkTarget, ResolvedBlock, ResolvedLink, ResolvedPage};
use crate::expand::PageProblem;
use crate::index::{FileIndex, RefKind, Reference, Target};
use crate::project::{PageAsset, Resolution};

/// Resolves every link and image of a page, and collects its assets.
pub(crate) fn resolve(resolver: &BuildResolver<'_>, page: &mut ResolvedPage) {
    let mut cx = Links {
        resolver,
        page_path: page.path.clone(),
        assets: Vec::new(),
        problems: Vec::new(),
    };
    cx.blocks(&mut page.blocks);
    page.assets = cx.assets;
    page.problems.extend(cx.problems);
}

struct Links<'a, 'p> {
    resolver: &'a BuildResolver<'p>,
    /// The page being resolved.
    page_path: RelPath,
    assets: Vec<PageAsset>,
    problems: Vec<PageProblem>,
}

impl Links<'_, '_> {
    fn blocks(&mut self, blocks: &mut [ResolvedBlock]) {
        for block in blocks {
            self.block(block);
            for children in block.child_lists_mut() {
                self.blocks(children);
            }
        }
    }

    fn block(&mut self, block: &mut ResolvedBlock) {
        let project = self.resolver.project();
        let Some(index) = project.file_by_id(block.file) else {
            return;
        };
        let ranges = own_ranges(block);
        if ranges.is_empty() {
            return;
        }
        let resolutions = project.resolutions(&index.path);
        let mut links = Vec::new();
        for (at, reference) in index.references.iter().enumerate() {
            if !ranges.iter().any(|r| r.contains_span(reference.span)) {
                continue;
            }
            let Some(resolution) = resolutions.get(at) else {
                continue;
            };
            let (target, title) = self.target(index, reference, resolution, block);
            links.push((reference, target, title));
        }
        let mut resolved = Vec::with_capacity(links.len());
        for (reference, target, title) in links {
            let mut destination = reference.destination.clone();
            for list in own_lists_mut(block) {
                if let Some(node) = find_mut(list, reference.span) {
                    destination = patch(node, &target, title);
                    break;
                }
            }
            resolved.push(ResolvedLink {
                span: reference.span,
                kind: reference.kind,
                destination,
                target,
            });
        }
        block.links = resolved;
    }

    /// What one reference points at in this build, and the title a page link
    /// takes for its text if it has none.
    fn target(
        &mut self,
        index: &FileIndex,
        reference: &Reference,
        resolution: &Resolution,
        block: &ResolvedBlock,
    ) -> (LinkTarget, Option<String>) {
        let target = match resolution {
            Resolution::External => LinkTarget::External,
            Resolution::Asset { path, fragment } => {
                self.assets.push(PageAsset {
                    path: path.clone(),
                    kind: reference.kind,
                    fragment: fragment.clone(),
                    written_in: index.path.clone(),
                    location: Location::new(index.file, reference.span),
                    via: block.via.clone(),
                });
                LinkTarget::Asset {
                    path: path.clone(),
                    fragment: fragment.clone(),
                }
            }
            Resolution::Source {
                target,
                id,
                fragment,
            } => {
                return self.page_link(index, reference, block, target, id.as_deref(), *fragment);
            }
            // The file-level problem is the source index's.
            _ => LinkTarget::Unresolved,
        };
        (target, None)
    }

    fn page_link(
        &mut self,
        index: &FileIndex,
        reference: &Reference,
        block: &ResolvedBlock,
        target: &RelPath,
        id: Option<&str>,
        is_fragment: bool,
    ) -> (LinkTarget, Option<String>) {
        // SPEC §5.2: a `#id` alone in a fragment names a heading of the
        // fragment itself, which the page that includes it publishes. Any
        // other link to a fragment is an error.
        let names_itself = matches!(&reference.target, Target::Local(l) if l.written.is_empty());
        if is_fragment && !names_itself {
            return (LinkTarget::Unresolved, None);
        }
        let page = if is_fragment {
            self.page_path.clone()
        } else {
            target.clone()
        };
        let here = Location::new(
            index.file,
            reference.destination_span.unwrap_or(reference.span),
        );
        if !is_fragment && !self.resolver.is_published(target) {
            self.problem(
                Issue::new(diagnostics::LINK_PAGE_DROPPED, here)
                    .with_arg("build", self.resolver.build_name())
                    .with_arg("path", target.to_string()),
                block,
            );
            return (LinkTarget::Unresolved, None);
        }
        let project = self.resolver.project();
        if project.file(target).is_none() {
            return (LinkTarget::Unresolved, None);
        }
        let text_empty = reference.text_empty && reference.kind == RefKind::Link;
        match id {
            None => {
                let title = self.resolver.title_of(&page);
                let url = self.resolver.router().link(&page, None);
                let target = LinkTarget::Page {
                    page,
                    id: None,
                    url,
                    text_filled: text_empty && title.is_some(),
                };
                (target, title)
            }
            Some(id) => {
                // The heading is the target's own or comes from a fragment it
                // includes (SPEC §4.2). A missing id is reported by the source
                // index (`link-id-missing`).
                let Some((written_in, heading)) = project.page_heading(target, id) else {
                    return (LinkTarget::Unresolved, None);
                };
                let Some(written_in) = project.file(&written_in) else {
                    return (LinkTarget::Unresolved, None);
                };
                let page_id = self
                    .resolver
                    .page_id_of(&page, written_in.file, heading.span);
                let Some(page_id) = page_id else {
                    self.problem(
                        Issue::new(diagnostics::LINK_ID_REMOVED, here)
                            .with_arg("build", self.resolver.build_name())
                            .with_arg("id", id)
                            .with_arg("path", target.to_string()),
                        block,
                    );
                    return (LinkTarget::Unresolved, None);
                };
                let url = self.resolver.router().link(&page, Some(&page_id));
                let target = LinkTarget::Page {
                    page,
                    id: Some(page_id),
                    url,
                    text_filled: text_empty && !heading.text.is_empty(),
                };
                (target, Some(heading.text.clone()))
            }
        }
    }

    fn problem(&mut self, issue: Issue, block: &ResolvedBlock) {
        self.problems.push(PageProblem {
            issue,
            via: block.via.clone(),
        });
    }
}

/// Rewrites the link node for its resolved target: a page link's destination
/// becomes its URL, and empty text becomes the target's title. Returns the
/// destination as it was.
fn patch(node: &mut Inline, target: &LinkTarget, title: Option<String>) -> String {
    let span: Span = node.span;
    match &mut node.kind {
        InlineKind::Link(link) => {
            let before = link.destination.clone();
            if let LinkTarget::Page {
                url, text_filled, ..
            } = target
            {
                link.destination = url.clone();
                if *text_filled && let Some(title) = title {
                    link.children = vec![Inline {
                        span,
                        kind: InlineKind::Text(title),
                    }];
                }
            }
            before
        }
        InlineKind::Image(image) => image.destination.clone(),
        _ => String::new(),
    }
}
