//! Headings, their sections, and their source ids (SPEC §5.5).

use ascribe_core::{Slugger, Span};
use ascribe_model::ContentModel;
use ascribe_syntax::{Block, BlockKind, Inline, InlineKind, PrimaryValue, bound_heading};

/// A heading's explicit id: the `@id` directive under it (SPEC §4.1).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExplicitId {
    /// The id.
    pub id: String,
    /// The `@id` directive line.
    pub span: Span,
}

/// A heading in a file.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Heading {
    /// 1 to 6.
    pub level: u8,
    /// The heading block.
    pub span: Span,
    /// The heading's text as plain text, with each declared phrase replaced by
    /// its value: what the slug is computed from, and what a link with no
    /// text shows for `page.md#id` (SPEC §5.2, §5.5). Emphasis, links, and
    /// code contribute their text; an image contributes nothing.
    pub text: String,
    /// The explicit id, if the heading has an `@id`. The first one, if it has
    /// several.
    pub explicit_id: Option<ExplicitId>,
    /// The heading's **source id**: its `@id`, or its slug, numbered within
    /// the file by the slugger (SPEC §5.5). Only headings without `@id` take
    /// part in numbering. Empty for a heading whose slug is empty.
    pub source_id: String,
    /// The section: the heading and every block after it in the same list of
    /// blocks, up to the next heading of the same or a higher level. A
    /// heading in a container, list item, or block quote has a section that
    /// stops at the end of that container (SPEC §3.8).
    pub section: Span,
    /// Whether the heading's text contains a declared phrase, which makes a
    /// slug that changes when the phrase's value does (SPEC §5.5).
    pub has_phrase: bool,
    /// Whether the slug is empty: the heading has no `@id`, and its text is
    /// only characters the slugger removes (punctuation, emoji).
    /// `ascribe check` reports it (SPEC §5.5).
    pub empty_slug: bool,
}

/// Collects every heading in document order, in every list of blocks, and
/// gives each its source id.
pub(crate) fn collect(
    blocks: &[Block],
    model: &ContentModel,
    slugger: &dyn Slugger,
) -> Vec<Heading> {
    let mut headings = Vec::new();
    collect_list(blocks, model, &mut headings);
    let mut scope = slugger.new_scope();
    for heading in &mut headings {
        match &heading.explicit_id {
            Some(explicit) => heading.source_id = explicit.id.clone(),
            None => {
                heading.source_id = scope.slug(&heading.text);
                // A fresh scope never numbers, so this is the slug the text
                // alone gives; a repeat (`-1`) of an empty slug is still empty.
                heading.empty_slug = slugger.new_scope().slug(&heading.text).is_empty();
            }
        }
    }
    headings
}

fn collect_list(blocks: &[Block], model: &ContentModel, out: &mut Vec<Heading>) {
    // Where each heading of this list sits in `out`, by position in `blocks`.
    let mut at: Vec<Option<usize>> = vec![None; blocks.len()];
    for (i, block) in blocks.iter().enumerate() {
        match &block.kind {
            BlockKind::Heading(h) => {
                let end = blocks[i + 1..]
                    .iter()
                    .position(|b| matches!(&b.kind, BlockKind::Heading(n) if n.level <= h.level))
                    .map_or(blocks.len(), |p| i + 1 + p);
                let last = &blocks[end - 1];
                let text = plain_text(&h.inlines, model);
                at[i] = Some(out.len());
                out.push(Heading {
                    level: h.level,
                    span: block.span,
                    has_phrase: has_declared_phrase(&h.inlines, model),
                    text,
                    explicit_id: None,
                    source_id: String::new(),
                    section: Span::new(block.span.start(), last.span.end()),
                    empty_slug: false,
                });
            }
            BlockKind::BlockQuote(q) => collect_list(&q.children, model, out),
            BlockKind::List(l) => {
                for item in &l.items {
                    collect_list(&item.children, model, out);
                }
            }
            BlockKind::Container(c) => collect_list(&c.children, model, out),
            BlockKind::Group(g) => {
                for arm in &g.arms {
                    collect_list(&arm.children, model, out);
                }
            }
            _ => {}
        }
    }
    // Explicit ids: an `@id` binds the heading before it in this list.
    // SPEC §4.1: the first `@id` with a value wins, valid or not.
    for (i, block) in blocks.iter().enumerate() {
        let BlockKind::Directive(line) = &block.kind else {
            continue;
        };
        if line.name != "id" {
            continue;
        }
        let Some(heading_at) = bound_heading(blocks, i) else {
            continue;
        };
        let Some(PrimaryValue::Identifier(primary)) = &line.primary else {
            continue;
        };
        if let Some(idx) = at[heading_at]
            && out[idx].explicit_id.is_none()
        {
            out[idx].explicit_id = Some(ExplicitId {
                id: primary.text.clone(),
                span: line.span,
            });
        }
    }
}

/// The plain text of inline content, with declared phrases replaced by their
/// values and undeclared candidates left as the `{key}` they are: a heading's
/// text, as its slug is computed from it (SPEC §5.5).
pub fn plain_text(inlines: &[Inline], model: &ContentModel) -> String {
    let mut out = String::new();
    push_text(inlines, model, &mut out);
    out
}

fn push_text(inlines: &[Inline], model: &ContentModel, out: &mut String) {
    for inline in inlines {
        match &inline.kind {
            // SPEC §5.5: the text content of the rendered
            // heading; an image and raw HTML contribute nothing.
            InlineKind::Text(t) | InlineKind::Code(t) => out.push_str(t),
            InlineKind::SoftBreak | InlineKind::HardBreak => out.push('\n'),
            InlineKind::Emphasis(children) | InlineKind::Strong(children) => {
                push_text(children, model, out);
            }
            InlineKind::Link(link) => push_text(&link.children, model, out),
            InlineKind::Phrase(p) => match model.phrase(&p.key) {
                Some(value) => out.push_str(value),
                None => {
                    out.push('{');
                    out.push_str(&p.key);
                    out.push('}');
                }
            },
            InlineKind::Html(_) | InlineKind::Image(_) => {}
        }
    }
}

fn has_declared_phrase(inlines: &[Inline], model: &ContentModel) -> bool {
    let mut found = false;
    super::walk::walk_inlines(inlines, &mut |inline| {
        if let InlineKind::Phrase(p) = &inline.kind
            && model.has_phrase(&p.key)
        {
            found = true;
        }
    });
    found
}
