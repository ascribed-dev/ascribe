//! Phrases (SPEC §5.1, §9.2 step 4): substituting the declared candidates in
//! prose, headings, link text and destinations, opted-in fences, and the
//! frontmatter fields the content model allows. Undeclared candidates stay
//! literal.
//!
//! A substituted phrase becomes plain text: the value is inserted as it is,
//! and never scanned for phrases or markup again. Each replacement is also
//! listed as a [`Substitution`] over the block's source text.

use serde_yaml_ng::Value;
use tessera_core::{RelPath, Span};
use tessera_model::{ContentModel, FieldType, TypeMatch, inline};
use tessera_syntax::{BlockKind, Inline, InlineKind, LinkForm, Phrase};

use super::inlines::own_lists_mut;
use super::tree::{FormattedField, ResolvedBlock, ResolvedKind, Substitution};
use crate::index::substitute;
use crate::project::Project;
use crate::references::destination_phrases;

/// Substitutes phrases in every block, and lists the substitutions made in
/// each block's own source text.
pub(crate) fn substitute_blocks(project: &Project, blocks: &mut [ResolvedBlock]) {
    for block in blocks {
        substitute_block(project, block);
        for children in block.child_lists_mut() {
            substitute_blocks(project, children);
        }
    }
}

fn substitute_block(project: &Project, block: &mut ResolvedBlock) {
    let Some(index) = project.file_by_id(block.file) else {
        return;
    };
    let model = project.model();
    let mut subs = Vec::new();
    if let ResolvedKind::Leaf(b) = &mut block.kind
        && let BlockKind::CodeBlock(code) = &mut b.kind
        && let Some(phrases) = code.phrases.take()
    {
        // A fence that opted in: candidates in source order, and the same
        // in the literal text (a backslash doesn't escape in code, §5.1).
        // A snippet's candidates are in its code file, not this block's
        // source, so they're substituted but not listed.
        if block.snippet.is_none() {
            subs.extend(listed(&phrases, model));
        }
        code.literal = substitute(&code.literal, &phrases, model);
        code.phrases = Some(Vec::new());
    }
    for list in own_lists_mut(block) {
        inlines(
            list,
            &index.source,
            &index.document.definitions,
            model,
            &mut subs,
        );
    }
    subs.sort_by_key(|s| s.span);
    block.substitutions = subs;
}

/// The declared candidates among `phrases`, as substitutions.
fn listed(phrases: &[Phrase], model: &ContentModel) -> Vec<Substitution> {
    phrases
        .iter()
        .filter_map(|p| {
            model.phrase(&p.key).map(|value| Substitution {
                span: p.span,
                key: p.key.clone(),
                value: value.to_owned(),
            })
        })
        .collect()
}

fn inlines(
    list: &mut Vec<Inline>,
    source: &str,
    definitions: &[tessera_syntax::LinkDefinition],
    model: &ContentModel,
    subs: &mut Vec<Substitution>,
) {
    for inline in list.iter_mut() {
        match &mut inline.kind {
            InlineKind::Phrase(p) => {
                let text = match model.phrase(&p.key) {
                    Some(value) => {
                        subs.push(Substitution {
                            span: p.span,
                            key: p.key.clone(),
                            value: value.to_owned(),
                        });
                        value.to_owned()
                    }
                    // Undeclared: literal text.
                    None => format!("{{{}}}", p.key),
                };
                inline.kind = InlineKind::Text(text);
            }
            InlineKind::Emphasis(children) | InlineKind::Strong(children) => {
                inlines(children, source, definitions, model, subs);
            }
            InlineKind::Link(link) => {
                inlines(&mut link.children, source, definitions, model, subs);
                let phrases = destination_phrases(
                    source,
                    link.form,
                    &link.destination_phrases,
                    link.label,
                    &link.children,
                    None,
                    &link.destination,
                    definitions,
                );
                let destination = substitute(&link.destination, phrases, model);
                if own_form(link.form) {
                    subs.extend(listed(&link.destination_phrases, model));
                }
                link.destination = destination;
                link.destination_phrases.clear();
            }
            InlineKind::Image(image) => {
                inlines(&mut image.children, source, definitions, model, subs);
                let phrases = destination_phrases(
                    source,
                    image.form,
                    &image.destination_phrases,
                    image.label,
                    &image.children,
                    Some(image.alt),
                    &image.destination,
                    definitions,
                );
                let destination = substitute(&image.destination, phrases, model);
                if own_form(image.form) {
                    subs.extend(listed(&image.destination_phrases, model));
                }
                image.destination = destination;
                image.destination_phrases.clear();
            }
            _ => {}
        }
    }
    merge_text(list);
}

/// Whether the destination is written in the node itself (not in a
/// definition).
fn own_form(form: LinkForm) -> bool {
    matches!(form, LinkForm::Inline | LinkForm::Autolink)
}

/// Joins text nodes that a substitution left next to each other.
pub(crate) fn merge_text(list: &mut Vec<Inline>) {
    let mut out: Vec<Inline> = Vec::with_capacity(list.len());
    for inline in list.drain(..) {
        if let InlineKind::Text(text) = &inline.kind
            && let Some(last) = out.last_mut()
            && let InlineKind::Text(previous) = &mut last.kind
        {
            previous.push_str(text);
            last.span = Span::new(last.span.start(), inline.span.end());
            continue;
        }
        out.push(inline);
    }
    *list = out;
}

// -- Frontmatter ------------------------------------------------------------

/// The frontmatter with phrases substituted in the fields the content model
/// says take them (`phrases = true`): a `string`, or
/// each item of a `list(string)`, however deep in objects. A field read with
/// inline markup (`inline = "code"`) becomes its plain text, and its pieces
/// are returned beside it, with those of the defaults of such fields the
/// page leaves out.
pub(crate) fn frontmatter(
    model: &ContentModel,
    path: &RelPath,
    value: &Value,
) -> (Value, Vec<FormattedField>) {
    let mut out = value.clone();
    let fields = match model.type_for(path.as_str()) {
        TypeMatch::One(t) => &t.frontmatter.fields,
        _ => return (out, Vec::new()),
    };
    let mut formatted = Vec::new();
    if let Value::Mapping(map) = &mut out {
        for field in fields {
            let Some(item) = map.get_mut(field.name.as_str()) else {
                // Phrases aren't substituted in a default, so they aren't
                // here either.
                if field.inline.is_some()
                    && let Some(Value::String(text)) = &field.default
                {
                    formatted.push(FormattedField {
                        name: field.name.clone(),
                        segments: inline::parse(text, &|_| None),
                    });
                }
                continue;
            };
            match item {
                Value::String(text) if field.inline.is_some() => {
                    let segments = if field.phrases {
                        inline::parse(text, &|key| model.phrase(key).map(str::to_owned))
                    } else {
                        inline::parse(text, &|_| None)
                    };
                    *text = inline::plain_text(&segments);
                    formatted.push(FormattedField {
                        name: field.name.clone(),
                        segments,
                    });
                }
                _ => typed(item, &field.ty, field.phrases, model),
            }
        }
    }
    (out, formatted)
}

fn typed(value: &mut Value, ty: &FieldType, phrases: bool, model: &ContentModel) {
    match (ty, value) {
        (FieldType::String, Value::String(text)) if phrases => {
            *text = substitute_text(text, model);
        }
        (FieldType::List(inner), Value::Sequence(items)) => {
            for item in items {
                typed(item, inner, phrases, model);
            }
        }
        (FieldType::Object(fields), Value::Mapping(map)) => {
            for (key, item) in map.iter_mut() {
                if let Some(field) = key
                    .as_str()
                    .and_then(|k| fields.iter().find(|f| f.name == k))
                {
                    typed(item, &field.ty, field.phrases, model);
                }
            }
        }
        _ => {}
    }
}

/// Replaces each `{key}` whose key is declared; the values aren't scanned
/// again.
fn substitute_text(text: &str, model: &ContentModel) -> String {
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(open) = rest.find('{') {
        out.push_str(&rest[..open]);
        let after = &rest[open + 1..];
        match after
            .find('}')
            .and_then(|close| model.phrase(&after[..close]).map(|v| (close, v)))
        {
            Some((close, value)) => {
                out.push_str(value);
                rest = &after[close + 1..];
            }
            None => {
                out.push('{');
                rest = after;
            }
        }
    }
    out.push_str(rest);
    out
}
