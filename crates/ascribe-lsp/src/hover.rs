//! Hover (SPEC §10): what a link or include points at, a phrase's value, the
//! availability a spec resolves to, and what a directive is.

use ascribe_core::schema::{
    AttributeSchema, AttributeType, Attributes, DefaultValue, DirectiveSchema, Origin, Primary,
    SetMember,
};
use ascribe_core::{LineIndex, RelPath};
use ascribe_emit::labels::{availability_display, availability_display_of_text};
use ascribe_model::ContentModel;
use ascribe_resolve::{Resolution, Snapshot};
use lsp_types::{Hover, HoverContents, MarkupContent, MarkupKind, Position};

use crate::nav::{Ctx, Hit, Lines, first_paragraph, hit_at};

/// The hover at a position of the requested file.
pub(crate) fn hover(ctx: &Ctx, position: Position) -> Option<Hover> {
    let file = ctx.file()?;
    let index = LineIndex::new(&file.source);
    let offset = ctx.encoding.offset_lenient(&index, &file.source, position);
    let (span, text) = match hit_at(file, offset)? {
        Hit::Phrase(use_) => {
            let key = &use_.phrase.key;
            let text = match ctx.model.phrase(key) {
                Some(value) => format!("**{{{key}}}**\n\n{value}"),
                None => format!(
                    "`{{{key}}}` isn't a declared phrase, so it stays literal text. \
                     Declare it under `[phrases]` in `ascribe.toml` to replace it."
                ),
            };
            (use_.phrase.span, text)
        }
        Hit::Reference(i, reference) => {
            let resolution = ctx.snapshot.resolutions(&ctx.path).get(i)?;
            (
                reference.span,
                describe_resolution(ctx, resolution, &reference.destination),
            )
        }
        Hit::Include(include) => {
            let text = match include.target.as_ref() {
                Some(target) if ctx.snapshot.file(target).is_some() => describe_source(
                    ctx,
                    target,
                    include.section.as_deref(),
                    ctx.snapshot.model().is_fragment(target.as_str()),
                ),
                Some(_) => format!("**Not found:** `{}`", include.written),
                None => return None,
            };
            (include.primary?, text)
        }
        Hit::Availability(text, span) => (span, availability_markdown(&ctx.model, text)?),
        Hit::Frontmatter(text, span) => (span, availability_markdown(&ctx.model, &text)?),
        Hit::Directive(d) => {
            let schema = ctx
                .model
                .directive_schemas()
                .into_iter()
                .find(|s| s.name == d.name)?;
            // On an attribute key, describe the key.
            let key = d.attributes.as_ref().and_then(|b| {
                b.attributes
                    .iter()
                    .find(|a| a.key_span.start() <= offset && offset <= a.key_span.end())
            });
            match key {
                Some(attribute) => (
                    attribute.key_span,
                    describe_attribute(&ctx.model, &schema, &attribute.key)?,
                ),
                None => (d.name_span, describe_directive(&schema)),
            }
        }
    };
    let range = Lines::new(ctx).range(&ctx.path, span)?;
    Some(Hover {
        contents: HoverContents::Markup(MarkupContent {
            kind: MarkupKind::Markdown,
            value: text,
        }),
        range: Some(range),
    })
}

/// What a link's or image's destination names, in words, with a preview of a
/// source file.
fn describe_resolution(ctx: &Ctx, resolution: &Resolution, destination: &str) -> String {
    let layout = ctx.snapshot.layout();
    match resolution {
        Resolution::External => format!("External link\n\n`{destination}`"),
        Resolution::Source {
            target,
            id,
            fragment,
        } => describe_source(ctx, target, id.as_deref(), *fragment),
        Resolution::SourceMissing { actual } => match actual {
            Some(actual) => format!(
                "**Not found:** `{destination}`\n\nA file differs only in case: `{actual}`. \
                 Names are case-sensitive."
            ),
            None => format!("**Not found:** `{destination}`"),
        },
        Resolution::Asset { path, fragment } => {
            let shown = layout.project_path(path);
            match fragment {
                Some(fragment) => format!("File `{shown}#{fragment}`"),
                None => format!("File `{shown}`"),
            }
        }
        Resolution::AssetMissing(_) => format!("**Not found:** `{destination}`"),
        Resolution::Route {
            suggestion,
            page_exists,
            ..
        } => {
            if *page_exists {
                format!(
                    "`{destination}` looks like a published route. Link to the file instead: `{suggestion}`."
                )
            } else {
                format!("`{destination}` looks like a published route, and names no file.")
            }
        }
    }
}

/// A source file, or a heading in it: its full path and a preview.
fn describe_source(ctx: &Ctx, target: &RelPath, id: Option<&str>, fragment: bool) -> String {
    let snapshot: &Snapshot = &ctx.snapshot;
    let Some(file) = snapshot.file(target) else {
        return format!("**Not found:** `{target}`");
    };
    let mut shown = snapshot.layout().project_path(target).to_string();
    // A heading of the page itself, or one from a fragment it includes
    // (SPEC §4.2), and the file it's written in.
    let found = id.and_then(|id| snapshot.page_heading(target, id));
    let heading = found.as_ref().map(|(_, h)| *h);
    let written_in = found
        .as_ref()
        .and_then(|(path, _)| snapshot.file(path))
        .unwrap_or(file);
    if let Some(id) = id {
        shown.push('#');
        shown.push_str(id);
    }
    let title = match (id, heading) {
        (Some(_), Some(h)) => Some(h.text.clone()),
        (Some(_), None) => None,
        (None, _) => file.title.clone(),
    };
    let mut out = String::new();
    match &title {
        Some(title) => out.push_str(&format!(
            "**{}**\n\n",
            crate::nav::substitute_phrases(&ctx.model, title)
        )),
        None if id.is_some() => out.push_str(&format!(
            "**No heading with the id `{}`** in this file\n\n",
            id.unwrap_or_default()
        )),
        None => {}
    }
    out.push_str(&format!("`{shown}`"));
    if fragment {
        out.push_str(" (a fragment: it's published only inside the pages that include it)");
    }
    if let Some(preview) = first_paragraph(written_in, &ctx.model, heading.map(|h| h.section)) {
        out.push_str("\n\n---\n\n");
        out.push_str(&preview);
    }
    out
}

/// A spec or feature key in words (SPEC §4.4, §9.4). `None` when the text isn't
/// a valid spec: the diagnostics say so.
fn availability_markdown(model: &ContentModel, text: &str) -> Option<String> {
    let text = text.trim();
    if let Some(feature) = model.feature(text) {
        return Some(format!(
            "**{}** (feature `{}`)\n\nAvailable: {}",
            feature.name,
            feature.key,
            availability_display(model, &feature.available)
        ));
    }
    let display = availability_display_of_text(model, text)?;
    Some(format!("Available: {display}"))
}

pub(crate) fn describe_directive(schema: &DirectiveSchema) -> String {
    let origin = match schema.origin {
        Origin::Builtin(_) => "built-in directive",
        Origin::Widget => "project widget",
    };
    let mut out = format!("**@{}**, a {origin}", schema.name);
    if let Some(description) = &schema.description {
        out.push_str(&format!("\n\n{description}"));
    }
    let mut forms = Vec::new();
    if schema.forms.line {
        forms.push("line");
    }
    if schema.forms.container {
        forms.push("container (`@end`)");
    }
    out.push_str(&format!("\n\nForms: {}", forms.join(", ")));
    let primary = match schema.primary {
        Primary::None => None,
        Primary::Identifier { .. } => Some("an identifier (a path, id, or key)"),
        Primary::Text { .. } => Some("text"),
        Primary::Availability { .. } => Some("an availability spec or feature key"),
    };
    if let Some(primary) = primary {
        let required = if schema.primary.is_required() {
            ", required"
        } else {
            ""
        };
        out.push_str(&format!("  \nPrimary: {primary}{required}"));
    }
    match &schema.attributes {
        Attributes::Declared(keys) if !keys.is_empty() => {
            out.push_str("\n\nAttributes:");
            for key in keys {
                out.push_str(&format!("\n- {}", attribute_line(key)));
            }
        }
        Attributes::Dimensions => {
            out.push_str("\n\nAttributes: any dimension of the content model, such as `pm=npm`.");
        }
        Attributes::Declared(_) => {}
    }
    out
}

fn describe_attribute(model: &ContentModel, schema: &DirectiveSchema, key: &str) -> Option<String> {
    match &schema.attributes {
        Attributes::Declared(keys) => keys.iter().find(|k| k.key == key).map(|k| {
            format!(
                "**{}** on `@{}`\n\n{}",
                k.key,
                schema.name,
                attribute_line(k)
            )
        }),
        Attributes::Dimensions => model.dimension(key).map(|d| {
            let values: Vec<String> = d
                .values
                .iter()
                .map(|v| format!("`{}` ({})", v.value, v.label))
                .collect();
            format!(
                "**{}**, a dimension of the content model\n\nValues: {}",
                d.label,
                values.join(", ")
            )
        }),
    }
}

/// `` `key` (type, default): description ``.
pub(crate) fn attribute_line(attribute: &AttributeSchema) -> String {
    let mut out = format!("`{}` ({}", attribute.key, type_name(&attribute.ty));
    if attribute.required {
        out.push_str(", required");
    }
    match &attribute.default {
        Some(DefaultValue::Text(t)) => out.push_str(&format!(", default `{t}`")),
        Some(DefaultValue::Boolean(b)) => out.push_str(&format!(", default `{b}`")),
        Some(DefaultValue::Set(members)) => {
            out.push_str(&format!(", default `{}`", members.join("|")));
        }
        None => {}
    }
    out.push(')');
    if let Some(description) = &attribute.description {
        out.push_str(&format!(": {description}"));
    }
    out
}

/// The name of an attribute type, for a person.
pub(crate) fn type_name(ty: &AttributeType) -> String {
    match ty {
        AttributeType::String => "string".to_owned(),
        AttributeType::Number => "number".to_owned(),
        AttributeType::Boolean => "boolean".to_owned(),
        AttributeType::Enum(values) => values.join(" | "),
        AttributeType::Set(SetMember::String) => "set of strings".to_owned(),
        AttributeType::Set(SetMember::Enum(values)) => format!("set of {}", values.join(" | ")),
        AttributeType::NoteType => "note type".to_owned(),
    }
}
