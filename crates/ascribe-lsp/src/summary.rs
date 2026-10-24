//! A section's summary (SPEC §4.12), as the editor shows it: on the hover of
//! its heading and its `@id` line, and in This page's Permalink row. It's
//! [`ascribe_emit::section_summary`], which `_ascribe/help.json` writes, so
//! the editor and the file can't disagree.

use ascribe_core::Span;
use ascribe_emit::EmitContext;
use ascribe_resolve::{DefaultRouter, FileKind};

use crate::nav::Ctx;

/// The summary of the section whose heading is at `heading` in the
/// requested file, in the editor's build: on the page itself, or, for a
/// fragment, on the first page that includes it and that the build
/// publishes, since a summary in a fragment is the summary of the section it
/// lands in. `None` when the section has none, or no page the build
/// publishes has it.
pub(crate) fn of_heading(ctx: &Ctx, heading: Span) -> Option<String> {
    of_section(ctx, Some(heading))
}

/// The summary of the requested page's opening, which its own permalink
/// leads to: its description, else the opening's `@summary`.
pub(crate) fn of_page(ctx: &Ctx) -> Option<String> {
    of_section(ctx, None)
}

fn of_section(ctx: &Ctx, heading: Option<Span>) -> Option<String> {
    let file = ctx.file()?;
    let index = ctx.snapshot.project();
    let pages = match (file.kind, heading) {
        (FileKind::Page, _) => vec![file.path.clone()],
        (FileKind::Fragment, Some(_)) => index.including_pages(&file.path),
        (FileKind::Fragment, None) => return None,
    };
    let build = ctx.model.editor_default_build();
    let router = DefaultRouter::from_consumer(&ctx.model.consumer);
    let root = ctx.config.parent().unwrap_or(&ctx.config);
    pages.iter().find_map(|path| {
        let page = index.resolve_page(path, build, &router)?;
        let anchor = match heading {
            Some(heading) => Some(
                page.headings()
                    .into_iter()
                    .find(|(block, _)| block.file == file.file && block.span == heading)?
                    .1
                    .page_id
                    .clone(),
            ),
            None => None,
        };
        let cx = EmitContext::new(index, root, build);
        ascribe_emit::section_summary(&cx, &page, anchor.as_deref())
    })
}

/// The hover's text for a section's summary.
pub(crate) fn markdown(summary: &str) -> String {
    format!("**Summary:** {summary}")
}
