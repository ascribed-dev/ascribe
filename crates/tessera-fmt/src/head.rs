//! The head of a directive line (SPEC §3.1, §8.3): `@name {attributes}: primary`.
//!
//! The rules here are the ones about spacing and the colon:
//!
//! - one space between the name and `{` (`name_gap`);
//! - no empty attribute block (`empty_block`);
//! - the attribute block itself, which `crate::attributes` handles;
//! - `:` directly after the name or attribute block (`colon`);
//! - one space between the colon and a primary (`primary_gap`);
//! - nothing after a container's `:`, not even whitespace
//!   (`container_trailing`).
//!
//! Each works from the sub-spans the parser records, so it edits only the
//! whitespace between two parts of the head, or deletes the part it names.
//! Whitespace at the end of a line-form directive is left alone: a text
//! primary's trailing spaces can be a hard break.

use tessera_core::{AttributeBlock, Span, TextEdit};
use tessera_syntax::{DirectiveLine, Form, PrimaryValue};

use crate::{Ctx, attributes};

/// Applies every head rule to `line`, unless the head can't be read as a
/// whole (text that fits no part, a primary the directive doesn't take, an
/// unclosed block, a container-only directive without its colon).
pub(crate) fn rules(ctx: &mut Ctx<'_>, line: &DirectiveLine) {
    if !readable(line) {
        return;
    }
    if empty(line) {
        empty_block(ctx, line);
    } else {
        name_gap(ctx, line);
        attributes::directive_rule(ctx, line);
        colon(ctx, line);
    }
    primary_gap(ctx, line);
    container_trailing(ctx, line);
}

/// Whether every part of the head is where the grammar puts it.
fn readable(line: &DirectiveLine) -> bool {
    let primary_ok = match &line.primary {
        None | Some(PrimaryValue::Text(_) | PrimaryValue::Line(_)) => true,
        Some(PrimaryValue::Identifier(id)) => id.trailing.is_none(),
        Some(PrimaryValue::Unexpected(_)) => false,
    };
    line.attributes_closed
        && line.unexpected.is_none()
        && primary_ok
        // A container opener has a colon and no primary; a container-only
        // directive without its colon is an error.
        && (line.form == Form::Line || (line.colon.is_some() && line.primary.is_none()))
}

fn empty(line: &DirectiveLine) -> bool {
    line.attributes
        .as_ref()
        .is_some_and(|b| b.attributes.is_empty())
}

fn block(line: &DirectiveLine) -> Option<&AttributeBlock> {
    line.attributes.as_ref()
}

/// One space between a directive name and `{`: `@note {type=tip}`.
pub(crate) fn name_gap(ctx: &mut Ctx<'_>, line: &DirectiveLine) {
    if let Some(block) = block(line) {
        ctx.replace(Span::new(line.name_span.end(), block.span.start()), " ");
    }
}

/// No empty attribute block: `@note`, not `@note {}`. Removes the block and
/// the whitespace around it, up to the colon if there is one.
pub(crate) fn empty_block(ctx: &mut Ctx<'_>, line: &DirectiveLine) {
    let Some(block) = block(line) else {
        return;
    };
    let end = line.colon.map_or(block.span.end(), |c| c.start());
    ctx.edits
        .push(TextEdit::delete(Span::new(line.name_span.end(), end)));
}

/// `:` directly after the name or the attribute block: `@note {type=tip}:`,
/// not `@note {type=tip} :`.
pub(crate) fn colon(ctx: &mut Ctx<'_>, line: &DirectiveLine) {
    let Some(colon) = line.colon else {
        return;
    };
    let before = block(line).map_or(line.name_span.end(), |b| b.span.end());
    ctx.replace(Span::new(before, colon.start()), "");
}

/// One space between the colon and a primary: `@note: Text`. A container's
/// colon has no primary after it.
pub(crate) fn primary_gap(ctx: &mut Ctx<'_>, line: &DirectiveLine) {
    if let (Some(colon), Some(primary)) = (line.colon, &line.primary) {
        ctx.replace(Span::new(colon.end(), primary.span().start()), " ");
    }
}

/// Nothing after a container's `:`, not even whitespace.
pub(crate) fn container_trailing(ctx: &mut Ctx<'_>, line: &DirectiveLine) {
    if line.form != Form::Container {
        return;
    }
    let Some(colon) = line.colon else {
        return;
    };
    let end = ctx.line_end(colon.end());
    ctx.replace(Span::new(colon.end(), end), "");
}
