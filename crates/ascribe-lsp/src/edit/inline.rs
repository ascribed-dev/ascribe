//! Links, phrases, and images: the operations on inline content.

use ascribe_core::{Destination, Span, TextEdit, classify_destination};
use ascribe_resolve::RefKind;
use ascribe_syntax::{Inline, InlineKind, LinkForm};

use super::insert::{check_image_keys, escaped_text, written_destination};
use super::{Outcome, Page, Plan};
use crate::context::{ContextNode, SelectionKind};
use crate::nav::link_headings;

/// The `destination` argument, checked: a URL, or a page of the project and,
/// after `#`, the id of a heading a link to it can name.
fn destination_arg(page: &Page<'_>, local_only: bool) -> Result<String, String> {
    let destination = page.line_arg("destination")?;
    if destination.contains(['<', '>']) {
        return Err("A destination can't have `<` or `>` in it.".to_owned());
    }
    let local = match classify_destination(&destination) {
        Destination::External if local_only => {
            return Err("A link to a URL needs text: select the text to link instead.".to_owned());
        }
        Destination::External => return Ok(destination),
        Destination::Local(local) => local,
    };
    let snapshot = &page.ctx.snapshot;
    let target = local
        .resolve(&page.ctx.path)
        .ok()
        .and_then(|t| snapshot.file(&t))
        .ok_or_else(|| format!("No page is at `{}`.", local.path))?;
    if target.kind != ascribe_resolve::FileKind::Page {
        return Err(format!(
            "`{}` is a fragment; link to a page that includes it.",
            local.path
        ));
    }
    if let Some(id) = local.fragment.as_deref().filter(|id| !id.is_empty())
        && !link_headings(snapshot, target)
            .iter()
            .any(|h| h.source_id == id)
    {
        return Err(format!(
            "`{}` has no heading with the id `{id}`.",
            target.path
        ));
    }
    Ok(destination)
}

/// The inline node at `offset` that isn't text a link can go in: a code
/// span, a link, an image, raw HTML, or a phrase.
fn blocking_inline(inlines: &[Inline], offset: usize, strict: bool) -> Option<&Inline> {
    inlines.iter().find_map(|inline| {
        let inside = if strict {
            inline.span.start() < offset && offset < inline.span.end()
        } else {
            inline.span.start() <= offset && offset <= inline.span.end()
        };
        if !(inline.span.start() <= offset && offset <= inline.span.end()) {
            return None;
        }
        match &inline.kind {
            InlineKind::Emphasis(children) | InlineKind::Strong(children) => {
                blocking_inline(children, offset, strict)
            }
            InlineKind::Code(_)
            | InlineKind::Link(_)
            | InlineKind::Image(_)
            | InlineKind::Html(_)
            | InlineKind::Phrase(_)
                if inside =>
            {
                Some(inline)
            }
            _ => None,
        }
    })
}

/// The inline content of the paragraph or heading at the start of the
/// range, if the cursor is in one.
fn prose<'a>(page: &Page<'a>) -> Option<&'a [Inline]> {
    let (span, ()) = page.innermost(|node| match node {
        ContextNode::Paragraph { .. } | ContextNode::Heading { .. } => Some(()),
        _ => None,
    })?;
    let (siblings, i) = page.block_at(span.start())?;
    match &siblings[i].kind {
        ascribe_syntax::BlockKind::Paragraph(p) => Some(&p.inlines),
        ascribe_syntax::BlockKind::Heading(h) => Some(&h.inlines),
        _ => None,
    }
}

/// Whether `s..e` cuts no inline node in two: each one is outside it, inside
/// it, or holds it in its own text (emphasis around the selection).
fn balanced(inlines: &[Inline], s: usize, e: usize) -> bool {
    inlines.iter().all(|inline| {
        let span = inline.span;
        if span.end() <= s || e <= span.start() || (s <= span.start() && span.end() <= e) {
            return true;
        }
        match &inline.kind {
            InlineKind::Emphasis(children) | InlineKind::Strong(children) => {
                let (Some(first), Some(last)) = (children.first(), children.last()) else {
                    return false;
                };
                first.span.start() <= s && e <= last.span.end() && balanced(children, s, e)
            }
            InlineKind::Text(_) | InlineKind::SoftBreak => true,
            _ => false,
        }
    })
}

/// Whether `inlines` holds a link or an image anywhere in `s..e`.
fn has_link(inlines: &[Inline], s: usize, e: usize) -> bool {
    inlines.iter().any(|inline| {
        let overlaps = inline.span.start() < e && s < inline.span.end();
        overlaps
            && match &inline.kind {
                InlineKind::Link(_) | InlineKind::Image(_) => true,
                InlineKind::Emphasis(children) | InlineKind::Strong(children) => {
                    has_link(children, s, e)
                }
                _ => false,
            }
    })
}

pub(crate) fn link_selection(page: &Page<'_>) -> Outcome {
    if page.start == page.end {
        return Err("Select the text to link.".to_owned());
    }
    let (kind, _) = crate::context::selection(page.file, page.start, page.end);
    if kind != SelectionKind::Prose {
        return Err("Select text inside one paragraph or heading to link.".to_owned());
    }
    let text = &page.source[page.start..page.end];
    let s = page.start + (text.len() - text.trim_start().len());
    let e = page.end - (text.len() - text.trim_end().len());
    let inlines = prose(page).ok_or_else(|| "Select text in a paragraph or heading.".to_owned())?;
    if has_link(inlines, s, e) || !balanced(inlines, s, e) {
        return Err(
            "The selection cuts across formatting or a link. Select plain text to link.".to_owned(),
        );
    }
    let destination = destination_arg(page, false)?;
    Ok(Plan::new(vec![
        TextEdit::insert(s, "["),
        TextEdit::insert(e, format!("]({})", written_destination(&destination))),
    ]))
}

/// The cursor, where inline text can be inserted: in a paragraph's or
/// heading's text, not inside a code span, link, image, or phrase.
fn cursor_in_prose(page: &Page<'_>, what: &str) -> Result<usize, String> {
    if page.start != page.end {
        return Err(format!(
            "Put the cursor where the {what} goes, with nothing selected."
        ));
    }
    let inlines = prose(page).ok_or_else(|| format!("Put the cursor in text for the {what}."))?;
    if blocking_inline(inlines, page.start, true).is_some() {
        return Err(format!(
            "The cursor is in a link, a phrase, or code; put the {what} outside it."
        ));
    }
    Ok(page.start)
}

pub(crate) fn insert_link(page: &Page<'_>) -> Outcome {
    let at = cursor_in_prose(page, "link")?;
    let destination = destination_arg(page, true)?;
    Ok(Plan::new(vec![TextEdit::insert(
        at,
        format!("[]({})", written_destination(&destination)),
    )]))
}

pub(crate) fn insert_phrase(page: &Page<'_>) -> Outcome {
    let key = page.line_arg("key")?;
    let model = &page.ctx.model;
    if !model.has_phrase(&key) {
        let keys: Vec<String> = model
            .phrases
            .iter()
            .map(|p| format!("`{}`", p.key))
            .collect();
        return Err(if keys.is_empty() {
            "The content model declares no phrases.".to_owned()
        } else {
            format!(
                "`{key}` isn't a phrase. The phrases are {}.",
                keys.join(", ")
            )
        });
    }
    let at = cursor_in_prose(page, "phrase")?;
    if page.source[..at].ends_with(['\\', '{']) {
        return Err("The character before the cursor would change the phrase.".to_owned());
    }
    Ok(Plan::new(vec![TextEdit::insert(at, format!("{{{key}}}"))]))
}

/// The link the cursor is in, and its index entry.
fn link<'a>(page: &Page<'a>) -> Result<(Span, &'a ascribe_resolve::Reference), String> {
    let (span, ()) = page
        .innermost(|node| matches!(node, ContextNode::Link { .. }).then_some(()))
        .ok_or_else(|| "Put the cursor in a link.".to_owned())?;
    let reference = page
        .file
        .references
        .iter()
        .find(|r| r.span == span && r.kind == RefKind::Link)
        .ok_or_else(|| "Put the cursor in a link.".to_owned())?;
    if reference.form != LinkForm::Inline {
        return Err("Only a link written `[text](destination)` can be changed here.".to_owned());
    }
    Ok((span, reference))
}

/// A destination's span as written, with its `<>` if it has them.
fn destination_span(page: &Page<'_>, span: Span) -> Span {
    let before = page.source[..span.start()].ends_with('<');
    let after = page.source[span.end()..].starts_with('>');
    if before && after {
        Span::new(span.start() - 1, span.end() + 1)
    } else {
        span
    }
}

pub(crate) fn set_link_target(page: &Page<'_>) -> Outcome {
    let (_, reference) = link(page)?;
    let destination = destination_arg(page, reference.text_empty)?;
    let written = written_destination(&destination);
    let edit = match reference.destination_span {
        Some(span) if !span.is_empty() => TextEdit::replace(destination_span(page, span), written),
        _ => {
            // `[text]()`: the destination goes between the parentheses.
            let at = page.source[..reference.span.end()]
                .rfind(')')
                .ok_or_else(|| "The link's destination can't be found.".to_owned())?;
            TextEdit::insert(at, written)
        }
    };
    if page.text(edit.span) == edit.new_text {
        return Err("The link already goes there.".to_owned());
    }
    Ok(Plan::new(vec![edit]))
}

pub(crate) fn use_target_title(page: &Page<'_>) -> Outcome {
    let (span, reference) = link(page)?;
    if reference.text_empty {
        return Err("The link already takes its target's title.".to_owned());
    }
    match &reference.target {
        ascribe_resolve::Target::Local(local) if local.source => {}
        _ => {
            return Err(
                "Only a link to a page takes its target's title; this one needs its text."
                    .to_owned(),
            );
        }
    }
    let inline = page
        .inline_at(span)
        .ok_or_else(|| "Put the cursor in a link.".to_owned())?;
    let InlineKind::Link(link) = &inline.kind else {
        return Err("Put the cursor in a link.".to_owned());
    };
    let text_end = link
        .children
        .last()
        .map(|c| c.span.end())
        .ok_or_else(|| "The link already takes its target's title.".to_owned())?;
    let close = page.source[text_end..span.end()]
        .find(']')
        .map(|i| text_end + i)
        .ok_or_else(|| "The link's text can't be found.".to_owned())?;
    Ok(Plan::new(vec![TextEdit::delete(Span::new(
        span.start() + 1,
        close,
    ))]))
}

/// The image the cursor is in.
fn image<'a>(page: &Page<'a>) -> Result<(Span, &'a ascribe_syntax::Image), String> {
    let (span, ()) = page
        .innermost(|node| matches!(node, ContextNode::Image { .. }).then_some(()))
        .ok_or_else(|| "Put the cursor in an image.".to_owned())?;
    let inline = page
        .inline_at(span)
        .ok_or_else(|| "Put the cursor in an image.".to_owned())?;
    match &inline.kind {
        InlineKind::Image(image) => Ok((span, image)),
        _ => Err("Put the cursor in an image.".to_owned()),
    }
}

pub(crate) fn set_image_width(page: &Page<'_>) -> Outcome {
    if !page
        .ctx
        .model
        .image_attributes
        .iter()
        .any(|a| a.key == "width")
    {
        return Err("The content model declares no `width` for images.".to_owned());
    }
    let width = page.line_arg("width")?;
    let (span, image) = image(page)?;
    let mut pairs: Vec<(String, String)> = image
        .attributes
        .as_ref()
        .map(|a| {
            a.block
                .attributes
                .iter()
                .filter(|a| a.key != "width")
                .filter_map(|a| Some((a.key.clone(), a.value.as_ref()?.members().join("|"))))
                .collect()
        })
        .unwrap_or_default();
    pairs.push(("width".to_owned(), width));
    check_image_keys(page, &pairs)?;
    let block = ascribe_fmt::image_block(&pairs, &page.ctx.model)
        .ok_or_else(|| "The width can't be written as an attribute value.".to_owned())?;
    let edit = match &image.attributes {
        Some(existing) => TextEdit::replace(existing.block.span, block),
        None => TextEdit::insert(span.end(), block),
    };
    if page.text(edit.span) == edit.new_text {
        return Err("The image already has that width.".to_owned());
    }
    Ok(Plan::new(vec![edit]))
}

pub(crate) fn set_image_alt(page: &Page<'_>) -> Outcome {
    let alt = page.line_arg("alt")?;
    let (_, image) = image(page)?;
    if !matches!(image.form, LinkForm::Inline | LinkForm::Full) {
        return Err("This image's alt text is its reference label; change it by hand.".to_owned());
    }
    let written = escaped_text(&alt);
    if page.text(image.alt) == written {
        return Err("The image already has that alt text.".to_owned());
    }
    Ok(Plan::new(vec![TextEdit::replace(image.alt, written)]))
}
