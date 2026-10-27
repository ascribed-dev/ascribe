//! What the navigation requests share: the state one request works from, and
//! the helpers that read a file's syntax and index at a cursor.
//!
//! A request takes a [`Ctx`] under the server's lock (a snapshot, the model,
//! the negotiated position encoding) and computes without it, so every answer
//! comes from the project as it was when the request was handled, never from a
//! stale snapshot. What an edit needs of it is `ascribe_edit`'s
//! ([`ascribe_edit::nav::Ctx`]), which it derefs to, with the helpers that
//! read a file at a cursor.

use std::ops::{Deref, DerefMut};
use std::sync::Arc;

use ascribe_core::Span;
use ascribe_emit::labels::plain_text;
use ascribe_model::ContentModel;
use ascribe_resolve::references::{operation_target, reference_target};
use ascribe_resolve::{FileIndex, RefKind, Resolution};
use ascribe_syntax::{Block, BlockKind};

pub(crate) use ascribe_edit::nav::{
    Hit, Lines, directive_at, encode_destination, frontmatter_key, hit_at, identifier_primary,
    line_start, link_headings, named_headings, relative_path,
};

/// What a navigation request works from: what an edit does
/// ([`ascribe_edit::nav::Ctx`]), and what the editor has besides.
pub(crate) struct Ctx {
    /// What an edit works from.
    pub edit: ascribe_edit::nav::Ctx,
    /// The requested document's version, when it's open in the editor.
    pub version: Option<i32>,
    /// The day the checks run on.
    pub today: Option<ascribe_core::Date>,
    /// What `ascribe.baseline` holds back of the requested file's problems,
    /// when they're for its text as the request has it.
    pub held: Vec<lsp_types::Diagnostic>,
    /// `ascribe.baseline`, as the editor has it.
    pub baseline: Arc<crate::baseline::Now>,
}

impl Ctx {
    /// The path of a request about the whole project asked through a file
    /// that isn't a source ([`crate::core::Core::project_target`]).
    pub(crate) const PROJECT: &'static str = ascribe_edit::nav::Ctx::PROJECT;
}

impl Deref for Ctx {
    type Target = ascribe_edit::nav::Ctx;

    fn deref(&self) -> &Self::Target {
        &self.edit
    }
}

impl DerefMut for Ctx {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.edit
    }
}

/// The text of the line up to the cursor.
pub(crate) fn line_prefix(source: &str, offset: usize) -> &str {
    &source[line_start(source, offset)..offset]
}

/// Replaces each declared `{key}` in `text` with its value.
pub(crate) fn substitute_phrases(model: &ContentModel, text: &str) -> String {
    if !text.contains('{') {
        return text.to_owned();
    }
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(open) = rest.find('{') {
        out.push_str(&rest[..open]);
        let after = &rest[open + 1..];
        match after.find('}') {
            Some(close) if model.has_phrase(&after[..close]) => {
                out.push_str(model.phrase(&after[..close]).unwrap_or_default());
                rest = &after[close + 1..];
            }
            _ => {
                out.push('{');
                rest = after;
            }
        }
    }
    out.push_str(rest);
    out
}

/// Cuts `text` to at most `max` characters, at a character boundary.
pub(crate) fn truncate(text: &str, max: usize) -> String {
    match text.char_indices().nth(max) {
        Some((at, _)) => format!("{}…", text[..at].trim_end()),
        None => text.to_owned(),
    }
}

/// The first paragraph of a file, or of a heading's section, as plain text with
/// declared phrases replaced by their values: the preview a link's hover
/// shows.
pub(crate) fn first_paragraph(
    file: &FileIndex,
    model: &ContentModel,
    section: Option<Span>,
) -> Option<String> {
    let found = find_paragraph(&file.document.blocks, section)?;
    let text = substitute_phrases(model, &plain_text(found));
    let text = text.split_whitespace().collect::<Vec<_>>().join(" ");
    (!text.is_empty()).then(|| truncate(&text, 280))
}

fn find_paragraph(blocks: &[Block], section: Option<Span>) -> Option<&[ascribe_syntax::Inline]> {
    for block in blocks {
        let inside = section.is_none_or(|s| s.contains_span(block.span));
        match &block.kind {
            BlockKind::Paragraph(p) if inside => return Some(&p.inlines),
            BlockKind::BlockQuote(q) => {
                if let Some(found) = find_paragraph(&q.children, section) {
                    return Some(found);
                }
            }
            BlockKind::List(l) => {
                for item in &l.items {
                    if let Some(found) = find_paragraph(&item.children, section) {
                        return Some(found);
                    }
                }
            }
            BlockKind::Container(c) => {
                if let Some(found) = find_paragraph(&c.children, section) {
                    return Some(found);
                }
            }
            BlockKind::Group(g) => {
                for arm in &g.arms {
                    if let Some(found) = find_paragraph(&arm.children, section) {
                        return Some(found);
                    }
                }
            }
            _ => {}
        }
    }
    None
}

/// The operation an `@operation`'s link names (SPEC §4.13), resolved as the
/// build resolves it: `None` when it isn't a link to a declared spec with a
/// fragment, which the page's problems say.
pub(crate) fn operation_resolution(ctx: &Ctx, link: &str) -> Option<Resolution> {
    let target = reference_target(RefKind::Link, link, &[], &ctx.path, &ctx.model);
    operation_target(&target, &ctx.model, ctx.snapshot.layout())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn truncation_is_on_a_character_boundary() {
        assert_eq!(truncate("héllo wörld", 5), "héllo…");
        assert_eq!(truncate("short", 20), "short");
    }
}
