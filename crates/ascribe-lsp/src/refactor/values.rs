//! Renaming a dimension's value everywhere it's used: in `ascribe.toml`
//! (the dimension's `values`, `labels`, and `versionless`, features' specs,
//! and builds' `variants` and `filter`), and in pages (`@variant`
//! attributes, `variant` and `available` frontmatter, `@available` lines,
//! and table rows' `available`).
//!
//! A value belongs to one dimension and plays no other role (the content
//! model's `model-dimension-value-shared` and `model-name-multiple-roles`),
//! so a name equal to the value, where a target or a value is written, is
//! the value.

use std::collections::HashMap;

use ascribe_core::availability::parse_availability;
use ascribe_core::{AttributeValue, FileId, LineIndex, RelPath, Span, TextEdit as ByteEdit};
use ascribe_resolve::FileIndex;
use ascribe_syntax::{Block, BlockKind, DirectiveLine};
use lsp_types::{TextEdit, WorkspaceEdit};
use toml_edit::{Item, Value};

use crate::model_file::{self, ModelFile, Seg, path};
use crate::nav::Ctx;

/// A dimension's value, where a rename starts from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DimensionValue {
    pub dimension: String,
    pub value: String,
    /// Where it's written, without quotes.
    pub span: Span,
}

/// The value in a `@variant` attribute at `offset` of a page.
pub(crate) fn in_page(file: &FileIndex, offset: usize) -> Option<DimensionValue> {
    let mut found = None;
    variant_lines(&file.document.blocks, &mut |line| {
        for attribute in line.attributes.iter().flat_map(|b| &b.attributes) {
            for (value, span) in members(attribute.value.as_ref()) {
                if span.start() <= offset && offset <= span.end() {
                    found = Some(DimensionValue {
                        dimension: attribute.key.clone(),
                        value,
                        span,
                    });
                }
            }
        }
    });
    found
}

/// The value in a dimension's `values` at `offset` of `ascribe.toml`.
pub(crate) fn in_model(ctx: &Ctx, offset: usize) -> Option<DimensionValue> {
    let file = ModelFile::parse(&ctx.model_text).ok()?;
    for dimension in &ctx.model.dimensions {
        let values = path(&["dimensions", &dimension.name, "values"]);
        let Some(array) = file.get(&values).and_then(Item::as_array) else {
            continue;
        };
        for item in array.iter() {
            let (Some(text), Some(span)) = (item.as_str(), item.span()) else {
                continue;
            };
            if span.start <= offset && offset <= span.end {
                // Inside the quotes, when the text is written as it is.
                let inner = Span::new(span.start + 1, span.end.saturating_sub(1));
                let span = if inner.len() == text.len() {
                    inner
                } else {
                    Span::new(span.start, span.end)
                };
                return Some(DimensionValue {
                    dimension: dimension.name.clone(),
                    value: text.to_owned(),
                    span,
                });
            }
        }
    }
    None
}

/// Why `new` can't be the value's new name, or `None` when it can.
pub(crate) fn refusal(ctx: &Ctx, value: &DimensionValue, new: &str) -> Option<String> {
    let model = &ctx.model;
    if ctx.model_problem {
        return Some("`ascribe.toml` has a problem. Fix it, then try again.".to_owned());
    }
    if !model
        .dimension_values(&value.dimension)
        .is_some_and(|values| values.iter().any(|v| v.value == value.value))
    {
        return Some(format!(
            "`{}` isn't a value of the dimension `{}`.",
            value.value, value.dimension
        ));
    }
    if new == value.value {
        return Some("The new name is the same.".to_owned());
    }
    if !ascribe_model::is_name_word(new) {
        return Some(format!(
            "`{new}` can't be a dimension value: {}.",
            ascribe_model::NAME_WORD_RULE
        ));
    }
    if model.dimension(new).is_some()
        || model.dimension_of_value(new).is_some()
        || model.lifecycle_state(new).is_some()
        || model.feature(new).is_some()
    {
        return Some(format!(
            "`{new}` already names something in the content model."
        ));
    }
    None
}

/// The edit that renames the value everywhere; `None` when `new` can't be
/// its name, or `ascribe.toml` can't be edited in place.
#[allow(clippy::mutable_key_type)]
pub(crate) fn rename(ctx: &Ctx, value: &DimensionValue, new: &str) -> Option<WorkspaceEdit> {
    if refusal(ctx, value, new).is_some() {
        return None;
    }
    let file = ModelFile::parse(&ctx.model_text).ok()?;
    let (model_edits, model_text) = file.finish(model_edits(ctx, &file, value, new)?).ok()?;
    let root = ctx.config.parent()?;
    ascribe_model::load_str_in(&model_text, FileId::new(0), root).ok()?;

    let mut edits: HashMap<RelPath, Vec<ByteEdit>> = HashMap::new();
    for file in ctx.snapshot.files() {
        let page = page_edits(file, value, new);
        if !page.is_empty() {
            edits.insert(file.path.clone(), page);
        }
    }
    let mut result = super::workspace_edit(ctx, edits);
    let index = LineIndex::new(&ctx.model_text);
    let model_edits = model_edits
        .iter()
        .map(|e| TextEdit {
            range: ctx.encoding.range(&index, e.span),
            new_text: e.new_text.clone(),
        })
        .collect();
    result
        .changes
        .get_or_insert_with(HashMap::new)
        .insert(crate::uri::path_to_uri(&ctx.config)?, model_edits);
    Some(result)
}

/// The edits to `ascribe.toml`.
fn model_edits(
    ctx: &Ctx,
    file: &ModelFile<'_>,
    value: &DimensionValue,
    new: &str,
) -> Option<Vec<model_file::Edit>> {
    let old = value.value.as_str();
    let mut out = Vec::new();
    let dimension = path(&["dimensions", &value.dimension]);
    let at = |mut base: Vec<Seg>, more: &[Seg]| {
        base.extend_from_slice(more);
        base
    };
    // Its `values` and `versionless`: each item that is it.
    for key in ["values", "versionless"] {
        let list = at(dimension.clone(), &[Seg::Key(key.to_owned())]);
        if let Some(array) = file.get(&list).and_then(Item::as_array) {
            for (i, item) in array.iter().enumerate() {
                if item.as_str() == Some(old) {
                    out.push(
                        file.replace_value(&at(list.clone(), &[Seg::Index(i)]), Value::from(new))
                            .ok()?,
                    );
                }
            }
        }
    }
    // Its label.
    let labels = at(dimension.clone(), &[Seg::Key("labels".to_owned())]);
    if file
        .table_like(&labels)
        .is_some_and(|t| t.get(old).is_some())
    {
        out.push(
            file.rename_key(&at(labels, &[Seg::Key(old.to_owned())]), new)
                .ok()?,
        );
    }
    // Features' specs.
    for feature in &ctx.model.features {
        let spec = path(&["features", &feature.key, "available"]);
        if let Some(text) = file.get(&spec).and_then(Item::as_str)
            && let Some(renamed) = rename_in_spec(text, old, new)
        {
            out.push(file.replace_value(&spec, Value::from(renamed)).ok()?);
        }
    }
    // Builds' selections and filters.
    for build in &ctx.model.builds {
        let variants = path(&["builds", &build.name, "variants", &value.dimension]);
        match file.get(&variants) {
            Some(item) if item.as_str() == Some(old) => {
                out.push(file.replace_value(&variants, Value::from(new)).ok()?);
            }
            Some(item) => {
                for (i, member) in item.as_array().into_iter().flatten().enumerate() {
                    if member.as_str() == Some(old) {
                        out.push(
                            file.replace_value(
                                &at(variants.clone(), &[Seg::Index(i)]),
                                Value::from(new),
                            )
                            .ok()?,
                        );
                    }
                }
            }
            None => {}
        }
        let filter = path(&["builds", &build.name, "availability", "filter"]);
        if let Some(text) = file.get(&filter).and_then(Item::as_str)
            && let Some(renamed) = rename_in_spec(text, old, new)
        {
            out.push(file.replace_value(&filter, Value::from(renamed)).ok()?);
        }
    }
    Some(out)
}

/// A spec with each target that is `old` renamed; `None` when it names no
/// such target.
fn rename_in_spec(text: &str, old: &str, new: &str) -> Option<String> {
    let spans = spec_targets(text, 0, old);
    if spans.is_empty() {
        return None;
    }
    let edits: Vec<ByteEdit> = spans
        .into_iter()
        .map(|span| ByteEdit::replace(span, new))
        .collect();
    ascribe_core::apply_edits(text, &edits).ok()
}

/// The spans of the targets that are `value` in the spec `text`, written at
/// `offset`.
fn spec_targets(text: &str, offset: usize, value: &str) -> Vec<Span> {
    parse_availability(text, offset)
        .map(|spec| {
            spec.entries
                .into_iter()
                .filter(|e| e.target.text == value)
                .map(|e| e.target.span)
                .collect()
        })
        .unwrap_or_default()
}

/// The edits to one source file.
fn page_edits(file: &FileIndex, value: &DimensionValue, new: &str) -> Vec<ByteEdit> {
    let old = value.value.as_str();
    let mut spans = Vec::new();
    variant_lines(&file.document.blocks, &mut |line| {
        for attribute in line.attributes.iter().flat_map(|b| &b.attributes) {
            if attribute.key != value.dimension {
                continue;
            }
            for (member, span) in members(attribute.value.as_ref()) {
                if member == old {
                    spans.push(span);
                }
            }
        }
    });
    for marker in &file.availability {
        if let Some(Ok(spec)) = &marker.spec {
            spans.extend(
                spec.entries
                    .iter()
                    .filter(|e| e.target.text == old)
                    .map(|e| e.target.span),
            );
        }
    }
    spans.extend(frontmatter(file, old));
    spans.sort_by_key(|s| (s.start(), s.end()));
    spans.dedup();
    spans
        .into_iter()
        .map(|span| ByteEdit::replace(span, new))
        .collect()
}

/// The value in the frontmatter: in the spec of `available`, and in
/// `variant`, wherever it's written as a value and not a key. Comments are
/// left alone.
fn frontmatter(file: &FileIndex, old: &str) -> Vec<Span> {
    let Some(fm) = &file.document.frontmatter else {
        return Vec::new();
    };
    let source: &str = &file.source;
    let base = fm.content.start();
    let text = source.get(fm.content.range()).unwrap_or_default();
    let word = |c: char| c.is_ascii_alphanumeric() || c == '_' || c == '-';
    let mut out = Vec::new();
    let mut in_variant = false;
    for line in ascribe_syntax::frontmatter_lines(text) {
        if line.indent == 0 && line.item.is_none() && !line.block {
            in_variant = line.key == Some("variant");
            if line.key == Some("available")
                && let Some((spec, span)) =
                    crate::nav::frontmatter_available(file, base + line.start)
            {
                out.extend(spec_targets(&spec, span.start(), old));
            }
        }
        if !in_variant || line.block {
            continue;
        }
        let value = line.value;
        for (i, _) in value.match_indices(old) {
            let end = i + old.len();
            let before = value[..i].chars().next_back();
            let after = value[end..].trim_start().chars().next();
            if before.is_some_and(word)
                || value[end..].chars().next().is_some_and(word)
                || after == Some(':')
            {
                continue;
            }
            let at = base + line.value_start + i;
            out.push(Span::new(at, at + old.len()));
        }
    }
    out
}

/// An attribute value's members, each with where it's written: a set's
/// tokens, a token, or a quoted value's text when it's written as it is.
fn members(value: Option<&AttributeValue>) -> Vec<(String, Span)> {
    match value {
        Some(AttributeValue::Token(t)) => vec![(t.text.clone(), t.span)],
        Some(AttributeValue::Set { members, .. }) => {
            members.iter().map(|t| (t.text.clone(), t.span)).collect()
        }
        Some(AttributeValue::Quoted { text, span }) => {
            let inner = Span::new(span.start() + 1, span.end().saturating_sub(1));
            if inner.len() == text.len() {
                vec![(text.clone(), inner)]
            } else {
                Vec::new()
            }
        }
        None => Vec::new(),
    }
}

/// Calls `f` with each `@variant` opener among `blocks`.
fn variant_lines<'a>(blocks: &'a [Block], f: &mut dyn FnMut(&'a DirectiveLine)) {
    for block in blocks {
        match &block.kind {
            BlockKind::Directive(line) if line.name == "variant" => f(line),
            BlockKind::BlockQuote(q) => variant_lines(&q.children, f),
            BlockKind::List(l) => {
                for item in &l.items {
                    variant_lines(&item.children, f);
                }
            }
            BlockKind::Container(c) => {
                if c.opener.name == "variant" {
                    f(&c.opener);
                }
                variant_lines(&c.children, f);
            }
            BlockKind::Group(g) => {
                for arm in &g.arms {
                    if arm.opener.name == "variant" {
                        f(&arm.opener);
                    }
                    variant_lines(&arm.children, f);
                }
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn specs_rename_their_targets_only() {
        assert_eq!(
            rename_in_spec("cloud, self-managed beta 2.4", "self-managed", "on-prem").as_deref(),
            Some("cloud, on-prem beta 2.4")
        );
        assert_eq!(rename_in_spec("cloud (beta 1, ga 2)", "beta", "x"), None);
        assert_eq!(rename_in_spec("cloud", "self-managed", "x"), None);
    }
}
