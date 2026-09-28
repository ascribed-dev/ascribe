//! Attribute blocks (SPEC §3.3, §8.3): spacing, order, and quoting.
//!
//! The canonical spelling of a block:
//!
//! - no space inside the braces, none around `=` or `|`, and `, ` between
//!   pairs (`render`);
//! - the pairs in the order the schema declares its attributes (`order`):
//!   for a directive its schema's (built-ins from `tessera-core`, widgets from
//!   the content model), for `@variant` the model's dimensions in declaration
//!   order, and for an image the model's image attributes;
//! - a value quoted only when it has to be (`value`): `label="Setup"` becomes
//!   `label=Setup`, while `label="Using other images"` keeps its quotes.
//!
//! A block is rewritten as one edit, and only when its canonical spelling
//! differs. Before an edit is offered, the canonical text is parsed again
//! and must mean exactly what the original did; if it doesn't (or the block
//! has a bare key, a repeated key, or anything the attribute parser
//! reported), the block is left alone.
//!
//! A key the schema doesn't declare can't be put in order, so a block that has
//! one keeps its order: it is still respaced and unquoted.

use tessera_core::attributes::parse_attribute_block;
use tessera_core::{Attribute, AttributeBlock, AttributeValue, Attributes, FileId, TextEdit};
use tessera_syntax::DirectiveLine;

use crate::Ctx;

/// The attribute block of a directive line (SPEC §3.3): the rules for
/// spacing, order, and quoting. The empty block is `crate::head::empty_block`.
pub(crate) fn directive_rule(ctx: &mut Ctx<'_>, line: &DirectiveLine) {
    let Some(block) = &line.attributes else {
        return;
    };
    let keys = declared_keys(ctx, &line.name);
    rule(ctx, block, keys.as_deref());
}

/// The attribute block after an image (SPEC §5.3): the same rules, in the
/// order of the model's image attributes; an empty block is removed unless
/// what follows it would then read as an attribute block.
pub(crate) fn image_rule(ctx: &mut Ctx<'_>, block: &AttributeBlock) {
    if ctx.has_error(block.span) {
        return;
    }
    if block.attributes.is_empty() {
        // Resolved Q76.
        // `![a](b){}{cloud}`: without its `{}`, `{cloud}` would be the
        // image's attribute block.
        if !ctx.source[block.span.end()..].starts_with('{') {
            ctx.edits.push(TextEdit::delete(block.span));
        }
        return;
    }
    let keys: Vec<String> = ctx
        .model
        .image_attributes
        .iter()
        .map(|a| a.key.clone())
        .collect();
    rule(ctx, block, Some(&keys));
}

/// The keys of `name`'s schema, in declaration order, if it has one.
fn declared_keys(ctx: &Ctx<'_>, name: &str) -> Option<Vec<String>> {
    let schema = ctx.options.schemas.iter().find(|s| s.name == name)?;
    Some(match &schema.attributes {
        Attributes::Declared(attributes) => attributes.iter().map(|a| a.key.clone()).collect(),
        // `@variant`'s attributes are dimension names, and the model's order
        // is the canonical one (content-model.md §1.1).
        Attributes::Dimensions => ctx
            .model
            .dimensions
            .iter()
            .map(|d| d.name.clone())
            .collect(),
    })
}

// Resolved Q75: a block with an undeclared key keeps its order; a bare
// or repeated key leaves the block alone.
/// Rewrites a non-empty `block` into canonical form, if that's safe.
fn rule(ctx: &mut Ctx<'_>, block: &AttributeBlock, keys: Option<&[String]>) {
    if let Some(text) = canonical(ctx.source, block, keys) {
        ctx.replace(block.span, &text);
    }
}

/// The canonical spelling of `block`, or `None` when it can't be rewritten
/// safely. `keys` is the schema's order.
pub(crate) fn canonical(
    source: &str,
    block: &AttributeBlock,
    keys: Option<&[String]>,
) -> Option<String> {
    let mut pairs: Vec<&Attribute> = block.attributes.iter().collect();
    if pairs.iter().any(|a| a.value.is_none()) || has_repeats(&pairs) {
        return None;
    }
    order(&mut pairs, keys);
    let rendered: Vec<String> = pairs
        .iter()
        .map(|a| pair(source, a))
        .collect::<Option<_>>()?;
    let text = render(&rendered);
    means_the_same(&text, &pairs).then_some(text)
}

/// Puts `pairs` in the order of `keys`, when every pair's key is one of them.
/// The sort is stable, and a block with a key the schema doesn't declare
/// keeps its order.
pub(crate) fn order(pairs: &mut [&Attribute], keys: Option<&[String]>) {
    let Some(keys) = keys else {
        return;
    };
    let position = |a: &Attribute| keys.iter().position(|k| *k == a.key);
    if pairs.iter().all(|a| position(a).is_some()) {
        pairs.sort_by_key(|a| position(a));
    }
}

/// `key=value` for one pair.
fn pair(source: &str, attribute: &Attribute) -> Option<String> {
    let value = attribute.value.as_ref()?;
    Some(format!("{}={}", attribute.key, self::value(source, value)))
}

/// A value, quoted only when it has to be. A quoted string whose text could
/// be a token loses its quotes; one that needs them keeps its spelling. A
/// value set's members are tokens, joined by `|`.
pub(crate) fn value(source: &str, value: &AttributeValue) -> String {
    match value {
        AttributeValue::Token(token) => token.text.clone(),
        AttributeValue::Quoted { text, .. } if is_token(text) => text.clone(),
        AttributeValue::Quoted { span, .. } => source[span.range()].to_owned(),
        AttributeValue::Set { members, .. } => members
            .iter()
            .map(|m| m.text.as_str())
            .collect::<Vec<_>>()
            .join("|"),
    }
}

/// Whether `text` can be written as a token: not empty, and without
/// whitespace or any of `,` `|` `{` `}` `=` `"` (SPEC §3.3).
fn is_token(text: &str) -> bool {
    !text.is_empty()
        && !text
            .chars()
            .any(|c| c.is_whitespace() || c.is_control() || ",|{}=\"".contains(c))
}

/// The block: `{`, the pairs separated by `, `, and `}`.
pub(crate) fn render(pairs: &[String]) -> String {
    format!("{{{}}}", pairs.join(", "))
}

fn has_repeats(pairs: &[&Attribute]) -> bool {
    pairs
        .iter()
        .enumerate()
        .any(|(i, a)| pairs[..i].iter().any(|b| b.key == a.key))
}

/// Whether parsing `text` gives back exactly `pairs`: the same keys, in
/// order, with the same values, of the same form (a value set stays a set),
/// and nothing to report.
fn means_the_same(text: &str, pairs: &[&Attribute]) -> bool {
    let Some(parsed) = parse_attribute_block(text, 0, FileId::new(0)) else {
        return false;
    };
    if !parsed.closed
        || !parsed.issues.is_empty()
        || parsed.len != text.len()
        || parsed.block.attributes.len() != pairs.len()
    {
        return false;
    }
    let is_set = |v: &AttributeValue| matches!(v, AttributeValue::Set { .. });
    parsed.block.attributes.iter().zip(pairs).all(|(new, old)| {
        new.key == old.key
            && match (&new.value, &old.value) {
                (Some(new), Some(old)) => {
                    is_set(new) == is_set(old) && new.members() == old.members()
                }
                _ => false,
            }
    })
}
