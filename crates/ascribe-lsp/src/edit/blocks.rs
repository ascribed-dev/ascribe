//! Notes, details, steps, heading ids, and the arms of variant groups: the
//! operations that wrap blocks, take a wrapper away, or change one.
//!
//! A wrap and its unwrap are inverses. A wrap writes its directive line (and
//! title line) directly above the first block, each new line continuing the
//! container the block is in, and `@end` directly below the last; the unwrap
//! deletes exactly those bytes, from the directive to the start of what it
//! holds and from the end of what it holds through `@end`.

use ascribe_core::{Span, TextEdit};
use ascribe_syntax::{Block, BlockKind, DirectiveLine, PrimaryValue};

use super::{Outcome, Page, Placeholder, Plan, Unwrap, title_line};
use crate::context::{ContextNode, Form, SelectionKind};

/// What `wrap` puts around blocks.
struct Wrapper {
    /// The title line, for `@details`.
    title: Option<String>,
    /// The directive's head: `@note {type=tip}`, `@details`.
    head: String,
}

/// The blocks a wrap applies to: the paragraph (or, for `@details`, the
/// block) the cursor is in, or the whole blocks the selection covers.
fn wrapped<'a>(page: &Page<'a>, any_block: bool) -> Result<(&'a [Block], usize, usize), String> {
    let selection = (page.start < page.end)
        .then(|| crate::context::selection(page.file, page.start, page.end).0);
    if selection == Some(SelectionKind::Blocks) {
        let text = &page.source[page.start..page.end];
        let s = page.start + (text.len() - text.trim_start().len());
        let e = page.end - (text.len() - text.trim_end().len());
        return selected(&page.file.document.blocks, s, e)
            .ok_or_else(|| "Select whole blocks to wrap.".to_owned());
    }
    if selection.is_some_and(|kind| kind != SelectionKind::Prose) {
        return Err("Select whole blocks to wrap, or put the cursor in one.".to_owned());
    }
    let target = page.innermost(|node| match node {
        ContextNode::Paragraph { .. } => Some(()),
        ContextNode::CodeBlock { .. }
        | ContextNode::Table { .. }
        | ContextNode::Snippet { .. }
        | ContextNode::Include { .. }
            if any_block =>
        {
            Some(())
        }
        _ => None,
    });
    let what = if any_block { "a block" } else { "a paragraph" };
    let (span, ()) =
        target.ok_or_else(|| format!("Put the cursor in {what}, or select whole blocks."))?;
    let (siblings, i) = page
        .block_at(span.start())
        .ok_or_else(|| format!("Put the cursor in {what}, or select whole blocks."))?;
    Ok((siblings, i, i))
}

/// The sibling blocks `s..e` covers whole: the innermost list of siblings
/// where it does, with the first and last index.
fn selected(blocks: &[Block], s: usize, e: usize) -> Option<(&[Block], usize, usize)> {
    if let Some(block) = blocks
        .iter()
        .find(|b| b.span.start() <= s && e <= b.span.end())
        && !(s <= block.span.start() && block.span.end() <= e)
    {
        return match &block.kind {
            BlockKind::BlockQuote(q) => selected(&q.children, s, e),
            BlockKind::Container(c) => selected(&c.children, s, e),
            BlockKind::List(list) => list
                .items
                .iter()
                .find(|i| i.span.start() <= s && e <= i.span.end())
                .and_then(|i| selected(&i.children, s, e)),
            BlockKind::Group(g) => g
                .arms
                .iter()
                .find(|a| a.span.start() <= s && e <= a.span.end())
                .and_then(|a| selected(&a.children, s, e)),
            _ => None,
        };
    }
    let first = blocks
        .iter()
        .position(|b| b.span.start() < e && s < b.span.end())?;
    let last = blocks
        .iter()
        .rposition(|b| b.span.start() < e && s < b.span.end())?;
    let whole = blocks[first..=last]
        .iter()
        .all(|b| s <= b.span.start() && b.span.end() <= e);
    whole.then_some((blocks, first, last))
}

/// Wraps `blocks[first..=last]`: one paragraph (or, for a wrapper that takes
/// any block, one block) gets the directive line above it, and several get a
/// container.
fn wrap(
    page: &Page<'_>,
    wrapper: &Wrapper,
    (blocks, first, last): (&[Block], usize, usize),
    any_block: bool,
) -> Outcome {
    let mut start = blocks[first].span.start();
    let prefix = page.prefix(start);
    let nl = page.nl;
    let mut head = String::new();
    if needs_blank_before_title(page, blocks, first) && wrapper.title.is_some() {
        // The line's own prefix is written again, after the blank line.
        start = super::line_start(page.source, start);
        head.push_str(prefix.trim_end());
        head.push_str(nl);
        head.push_str(&prefix);
    }
    if let Some(title) = &wrapper.title {
        head.push_str(&title_line(title));
        head.push_str(nl);
        head.push_str(&prefix);
    }
    head.push_str(&wrapper.head);
    let one = first == last && (any_block || matches!(blocks[first].kind, BlockKind::Paragraph(_)));
    let at = Span::new(start, blocks[first].span.start());
    if one {
        head.push_str(nl);
        head.push_str(&prefix);
        return Ok(Plan::new(vec![TextEdit::replace(at, head)]));
    }
    head.push(':');
    head.push_str(nl);
    head.push_str(&prefix);
    let end = blocks[last].span.end();
    Ok(Plan::new(vec![
        TextEdit::replace(at, head),
        TextEdit::insert(end, format!("{nl}{prefix}@end")),
    ]))
}

/// Whether a title line written directly above `blocks[i]` would follow text
/// it can't follow, and so needs a blank line before it: a title line starts
/// a block only after a blank line, a heading, a directive line, or the start
/// of its container (SPEC §3.7).
fn needs_blank_before_title(page: &Page<'_>, blocks: &[Block], i: usize) -> bool {
    let start = super::line_start(page.source, blocks[i].span.start());
    if start == 0 || i == 0 {
        return false;
    }
    let previous = super::line_start(page.source, start - 1);
    if super::is_blank(&page.source[previous..start]) {
        return false;
    }
    !matches!(
        blocks[i - 1].kind,
        BlockKind::Heading(_) | BlockKind::Directive(_)
    )
}

/// `@note`, with its type unless it's `note`.
fn note_head(page: &Page<'_>, note_type: &str) -> String {
    let pairs = if note_type == "note" {
        Vec::new()
    } else {
        vec![("type".to_owned(), note_type.to_owned())]
    };
    match ascribe_fmt::directive_block("note", &pairs, &page.options(), &page.ctx.model) {
        Some(block) => format!("@note {block}"),
        None => "@note".to_owned(),
    }
}

/// The `type` argument, checked against the model's note types; `note` when
/// it isn't given.
pub(super) fn note_type_arg(page: &Page<'_>) -> Result<String, String> {
    let value = page.opt_str("type")?.unwrap_or_else(|| "note".to_owned());
    if page.ctx.model.note_type(&value).is_some() {
        return Ok(value);
    }
    let names: Vec<String> = page
        .ctx
        .model
        .notes
        .iter()
        .map(|n| format!("`{}`", n.name))
        .collect();
    Err(format!(
        "`{value}` isn't a note type. The note types are {}.",
        names.join(", ")
    ))
}

/// A title argument: one line, not empty.
fn title_arg(page: &Page<'_>) -> Result<String, String> {
    page.line_arg("title")
}

pub(crate) fn wrap_note(page: &Page<'_>) -> Outcome {
    let note_type = note_type_arg(page)?;
    let target = wrapped(page, false)?;
    let wrapper = Wrapper {
        title: None,
        head: note_head(page, &note_type),
    };
    wrap(page, &wrapper, target, false)
}

pub(crate) fn wrap_details(page: &Page<'_>) -> Outcome {
    let title = title_arg(page)?;
    let target = wrapped(page, true)?;
    let wrapper = Wrapper {
        title: Some(title),
        head: "@details".to_owned(),
    };
    wrap(page, &wrapper, target, true)
}

/// The directive line of a note, details, or steps found at `span`, with the
/// block it starts and its siblings.
fn directive_of<'a>(
    page: &Page<'a>,
    span: Span,
) -> Result<(&'a [Block], usize, &'a DirectiveLine), String> {
    let (siblings, i) = page
        .block_at(span.start())
        .ok_or_else(|| "The cursor isn't in one.".to_owned())?;
    let line = match &siblings[i].kind {
        BlockKind::Directive(line) => line,
        BlockKind::Container(c) => &c.opener,
        _ => return Err("The cursor isn't in one.".to_owned()),
    };
    Ok((siblings, i, line))
}

fn innermost_note(page: &Page<'_>) -> Result<(Span, Form), String> {
    page.innermost(|node| match node {
        ContextNode::Note { form, .. } => Some(*form),
        _ => None,
    })
    .ok_or_else(|| "Put the cursor in a note.".to_owned())
}

pub(crate) fn set_note_type(page: &Page<'_>) -> Outcome {
    let note_type = note_type_arg(page)?;
    let (span, _) = innermost_note(page)?;
    let (_, _, line) = directive_of(page, span)?;
    let mut pairs: Vec<(String, String)> = line
        .attributes
        .as_ref()
        .map(|b| {
            b.attributes
                .iter()
                .filter(|a| a.key != "type")
                .filter_map(|a| Some((a.key.clone(), a.value.as_ref()?.members().join("|"))))
                .collect()
        })
        .unwrap_or_default();
    if note_type != "note" {
        pairs.push(("type".to_owned(), note_type));
    }
    let block = ascribe_fmt::directive_block("note", &pairs, &page.options(), &page.ctx.model);
    let written = block.map(|b| format!(" {b}")).unwrap_or_default();
    let at = line.name_span.end();
    let edit = match &line.attributes {
        Some(existing) => TextEdit::replace(Span::new(at, existing.span.end()), written),
        None if written.is_empty() => return Err("The note already has that type.".to_owned()),
        None => TextEdit::insert(at, written),
    };
    if page.text(edit.span) == edit.new_text {
        return Err("The note already has that type.".to_owned());
    }
    Ok(Plan::new(vec![edit]))
}

/// Takes a note, a details, or a `@steps` away, leaving what it holds.
pub(crate) fn unwrap(page: &Page<'_>, what: Unwrap) -> Outcome {
    let found = match what {
        Unwrap::Note => page.innermost(|node| match node {
            ContextNode::Note { form, .. } => Some(*form),
            _ => None,
        }),
        Unwrap::Details => page.innermost(|node| match node {
            ContextNode::Details { form, .. } => Some(*form),
            _ => None,
        }),
        Unwrap::Steps => page.innermost(|node| match node {
            ContextNode::Steps { .. } => Some(Form::Block),
            _ => None,
        }),
    };
    let (span, form) = found.ok_or_else(|| {
        match what {
            Unwrap::Note => "Put the cursor in a note.",
            Unwrap::Details => "Put the cursor in a details block.",
            Unwrap::Steps => "Put the cursor in a list of steps.",
        }
        .to_owned()
    })?;
    let (siblings, i, line) = directive_of(page, span)?;
    let block = &siblings[i];
    match (&block.kind, form) {
        (BlockKind::Directive(_), Form::Block) => {
            let next = siblings
                .get(i + 1)
                .ok_or_else(|| "It holds nothing to keep.".to_owned())?;
            Ok(Plan::new(vec![TextEdit::delete(Span::new(
                block.span.start(),
                next.span.start(),
            ))]))
        }
        (BlockKind::Directive(_), _) => match &line.primary {
            Some(PrimaryValue::Text(text)) => Ok(Plan::new(vec![TextEdit::delete(Span::new(
                block.span.start(),
                text.span.start(),
            ))])),
            _ => Err("The note holds nothing to keep.".to_owned()),
        },
        (BlockKind::Container(c), _) => {
            let end = c
                .end
                .as_ref()
                .ok_or_else(|| "It has no `@end`, so it can't be taken apart.".to_owned())?;
            match (c.children.first(), c.children.last()) {
                (Some(first), Some(last)) => Ok(Plan::new(vec![
                    TextEdit::delete(Span::new(block.span.start(), first.span.start())),
                    TextEdit::delete(Span::new(last.span.end(), end.span.end())),
                ])),
                _ => Err("It holds nothing to keep.".to_owned()),
            }
        }
        _ => Err("The cursor isn't in one.".to_owned()),
    }
}

pub(crate) fn note_to_details(page: &Page<'_>) -> Outcome {
    let title = title_arg(page)?;
    let (span, _) = innermost_note(page)?;
    let (siblings, i, line) = directive_of(page, span)?;
    let block = &siblings[i];
    let prefix = page.prefix(block.span.start());
    let nl = page.nl;
    let head = format!("{}{nl}{prefix}@details", title_line(&title));
    let edit = match &block.kind {
        BlockKind::Container(c) => TextEdit::replace(
            Span::new(block.span.start(), c.opener.span.end()),
            format!("{head}:"),
        ),
        BlockKind::Directive(_) => match &line.primary {
            Some(PrimaryValue::Text(text)) => TextEdit::replace(
                Span::new(block.span.start(), text.span.start()),
                format!("{head}{nl}{prefix}"),
            ),
            None if line.binding == Some(ascribe_syntax::Bound::FollowingBlock) => {
                TextEdit::replace(Span::new(block.span.start(), line.span.end()), head)
            }
            _ => return Err("The note holds nothing to keep.".to_owned()),
        },
        _ => return Err("Put the cursor in a note.".to_owned()),
    };
    Ok(Plan::new(vec![edit]))
}

pub(crate) fn make_steps(page: &Page<'_>) -> Outcome {
    let (span, (ordered, steps)) = page
        .innermost(|node| match node {
            ContextNode::List { ordered, steps, .. } => Some((*ordered, *steps)),
            _ => None,
        })
        .ok_or_else(|| "Put the cursor in a numbered list.".to_owned())?;
    if !ordered {
        return Err("Only a numbered list can be steps.".to_owned());
    }
    if steps {
        return Err("The list is already steps.".to_owned());
    }
    let prefix = page.prefix(span.start());
    Ok(Plan::new(vec![TextEdit::insert(
        span.start(),
        format!("@steps{}{prefix}", page.nl),
    )]))
}

/// Whether `id` can be a heading id: letters, digits, `-`, `_`, and `.`.
pub(super) fn is_id(id: &str) -> bool {
    !id.is_empty()
        && id
            .chars()
            .all(|c| c.is_alphanumeric() || matches!(c, '-' | '_' | '.'))
}

pub(crate) fn add_heading_id(page: &Page<'_>) -> Outcome {
    let (span, (slug, explicit)) = page
        .innermost(|node| match node {
            ContextNode::Heading {
                id, explicit_id, ..
            } => Some((id.clone(), *explicit_id)),
            _ => None,
        })
        .ok_or_else(|| "Put the cursor in a heading.".to_owned())?;
    if explicit {
        return Err("The heading already has an `@id`.".to_owned());
    }
    let id = match page.opt_str("id")? {
        Some(id) if !id.trim().is_empty() => id.trim().to_owned(),
        _ if slug.is_empty() => {
            return Err("The heading has no slug to keep; give it an id.".to_owned());
        }
        _ => slug,
    };
    if !is_id(&id) {
        return Err(format!(
            "`{id}` can't be an id: use letters, digits, `-`, `_`, and `.`."
        ));
    }
    let snapshot = &page.ctx.snapshot;
    let taken = match snapshot.expansion(&page.ctx.path) {
        Some(expanded) => expanded
            .headings(snapshot.project())
            .into_iter()
            .any(|(path, h)| h.source_id == id && !(path == page.ctx.path && h.span == span)),
        None => page
            .file
            .headings
            .iter()
            .any(|h| h.source_id == id && h.span != span),
    };
    if taken {
        return Err(format!("Another heading on the page has the id `{id}`."));
    }
    let prefix = page.prefix(span.start());
    Ok(Plan::new(vec![TextEdit::insert(
        span.end(),
        format!("{}{prefix}@id: {id}", page.nl),
    )]))
}

/// A variant group, its span, its dimension, and the arm the cursor is in.
type Found<'a> = (
    &'a ascribe_syntax::Group,
    Span,
    Option<String>,
    Option<usize>,
);

/// The variant group the cursor is in.
fn variant_group<'a>(page: &Page<'a>) -> Result<Found<'a>, String> {
    let (span, (dimension, arm)) = page
        .innermost(|node| match node {
            ContextNode::VariantGroup { dimension, arm, .. } => Some((dimension.clone(), *arm)),
            _ => None,
        })
        .ok_or_else(|| "Put the cursor in a group of `@variant` arms.".to_owned())?;
    let (siblings, i) = page
        .block_at(span.start())
        .ok_or_else(|| "Put the cursor in a group of `@variant` arms.".to_owned())?;
    let BlockKind::Group(group) = &siblings[i].kind else {
        return Err("Put the cursor in a group of `@variant` arms.".to_owned());
    };
    Ok((group, span, dimension, arm))
}

/// The placeholder in a new arm, or in a new block of a group.
pub(super) fn arm_placeholder(label: &str) -> String {
    format!("Write what applies to {label} here.")
}

pub(crate) fn add_variant_arm(page: &Page<'_>) -> Outcome {
    let (group, span, dimension, _) = variant_group(page)?;
    let dimension = dimension.ok_or_else(|| {
        "The group's arms are labeled, not by a dimension, so there's no value to add.".to_owned()
    })?;
    let model = &page.ctx.model;
    let declared = model
        .dimension(&dimension)
        .ok_or_else(|| format!("The content model declares no dimension `{dimension}`."))?;
    let value = page.str_arg("value")?;
    let position = |v: &str| declared.values.iter().position(|d| d.value == v);
    let Some(at) = position(&value) else {
        let values: Vec<String> = declared
            .values
            .iter()
            .map(|v| format!("`{}`", v.value))
            .collect();
        return Err(format!(
            "`{value}` isn't a value of `{dimension}`. Its values are {}.",
            values.join(", ")
        ));
    };
    let arm_values = |arm: &ascribe_syntax::Arm| -> Vec<String> {
        arm.opener
            .attributes
            .as_ref()
            .and_then(|b| b.get(&dimension))
            .and_then(|a| a.value.as_ref())
            .map(|v| v.members().iter().map(|m| (*m).to_owned()).collect())
            .unwrap_or_default()
    };
    if group.arms.iter().any(|a| arm_values(a).contains(&value)) {
        return Err(format!("The group already has an arm for `{value}`."));
    }
    let pairs = vec![(dimension.clone(), value.clone())];
    let block = ascribe_fmt::directive_block("variant", &pairs, &page.options(), model)
        .ok_or_else(|| format!("`{value}` can't be written as an attribute value."))?;
    let label = declared
        .values
        .get(at)
        .map_or(value.as_str(), |v| v.label.as_str());
    let placeholder = arm_placeholder(label);
    let opener = format!("@variant {block}:");
    let prefix = page.prefix(span.start());
    let nl = page.nl;
    let before = group.arms.iter().find(|a| {
        arm_values(a)
            .iter()
            .filter_map(|v| position(v))
            .min()
            .is_some_and(|p| p > at)
    });
    let (edit, from) = match before {
        Some(arm) => {
            let text = format!("{opener}{nl}{prefix}{placeholder}{nl}{prefix}");
            let from = opener.len() + nl.len() + prefix.len();
            (TextEdit::insert(arm.span.start(), text), from)
        }
        None => {
            let last = group
                .arms
                .last()
                .ok_or_else(|| "The group has no arms.".to_owned())?;
            let text = format!("{nl}{prefix}{opener}{nl}{prefix}{placeholder}");
            let from = text.len() - placeholder.len();
            (TextEdit::insert(last.span.end(), text), from)
        }
    };
    Ok(Plan {
        edits: vec![edit],
        select: Some(Placeholder {
            edit: 0,
            from,
            len: placeholder.len(),
        }),
    })
}

pub(crate) fn remove_variant_arm(page: &Page<'_>) -> Outcome {
    let (group, _, _, arm) = variant_group(page)?;
    let i = arm.ok_or_else(|| "Put the cursor in the arm to remove.".to_owned())?;
    if group.arms.len() < 2 {
        return Err(
            "It's the group's only arm. Remove the group instead, or add another arm first."
                .to_owned(),
        );
    }
    let span = match group.arms.get(i + 1) {
        Some(next) => Span::new(group.arms[i].span.start(), next.span.start()),
        None => Span::new(group.arms[i - 1].span.end(), group.arms[i].span.end()),
    };
    Ok(Plan::new(vec![TextEdit::delete(span)]))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ids_take_letters_digits_and_three_marks() {
        assert!(is_id("ece_setup"));
        assert!(is_id("v1.2"));
        assert!(is_id("streaming-sync"));
        assert!(!is_id("two words"));
        assert!(!is_id(""));
        assert!(!is_id("a#b"));
    }
}
