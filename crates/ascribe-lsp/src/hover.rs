//! Hover (SPEC §10): what a link or include points at, a phrase's value, the
//! availability a spec resolves to, the rule of `[listing]` a page is
//! under, and what a directive is.

use ascribe_core::availability::{AvailabilitySpec, Detail, Version, parse_availability};
use ascribe_core::schema::{
    AttributeSchema, AttributeType, Attributes, Builtin, DefaultValue, DirectiveSchema, Origin,
    Primary, SetMember,
};
use ascribe_core::{LineIndex, RelPath};
use ascribe_emit::labels::{availability_display, availability_display_of_text};
use ascribe_model::{ContentModel, LineStatus, Placement};
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
        Hit::Snippet(snippet) => (
            snippet.primary?,
            crate::examples::snippet_hover(ctx, snippet)?,
        ),
        Hit::Availability(text, span) => (span, availability_markdown(&ctx.model, text)?),
        Hit::Frontmatter(text, span) => (span, availability_markdown(&ctx.model, &text)?),
        Hit::Listing(span) => (
            span,
            listing_markdown(&ctx.model, ctx.path.as_str(), ctx.page_type_name()),
        ),
        Hit::Replaces(span) => (span, replaces_markdown(ctx)?),
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

/// The page a page's `replaces` names, and which of the two each line of
/// a versioned build publishes. `None` when it names no page.
fn replaces_markdown(ctx: &Ctx) -> Option<String> {
    let replaced = ctx.snapshot.replaces(&ctx.path)?;
    let mut out = format!(
        "**Replaces** `{replaced}`\n\n{}",
        describe_source(ctx, &replaced, None, false)
    );
    let mut lines: Vec<String> = Vec::new();
    for build in ctx.model.builds.iter() {
        let Some(line) = &build.line else {
            continue;
        };
        let publishes = if ctx.snapshot.dropped(&ctx.path, build).is_none() {
            "this page".to_owned()
        } else if ctx.snapshot.dropped(&replaced, build).is_none() {
            format!("`{replaced}`")
        } else {
            "neither".to_owned()
        };
        lines.push(format!(
            "- {} (`{}`): {publishes}",
            line.line.text, build.name
        ));
    }
    if !lines.is_empty() {
        out.push_str("\n\nEach line publishes:\n\n");
        out.push_str(&lines.join("\n"));
    }
    Some(out)
}

/// A spec or feature key in words (SPEC §4.4, §9.4). `None` when the text isn't
/// a valid spec: the diagnostics say so.
fn availability_markdown(model: &ContentModel, text: &str) -> Option<String> {
    let text = text.trim();
    if let Some(feature) = model.feature(text) {
        let mut out = format!(
            "**{}** (feature `{}`)\n\nAvailable: {}",
            feature.name,
            feature.key,
            availability_display(model, &feature.available)
        );
        out.push_str(&release_lines(model, &feature.available));
        return Some(out);
    }
    let display = availability_display_of_text(model, text)?;
    let mut out = format!("Available: {display}");
    if let Ok(spec) = parse_availability(text, 0) {
        out.push_str(&release_lines(model, &spec));
    }
    Some(out)
}

/// For each version a spec names of a target with `[releases]`, the line
/// it's in and the line's status, as a list after a blank line; nothing
/// when it names none.
fn release_lines(model: &ContentModel, spec: &AvailabilitySpec) -> String {
    let mut items: Vec<String> = Vec::new();
    for entry in &spec.entries {
        let Some(releases) = model.releases_of(&entry.target.text) else {
            continue;
        };
        let versions: Vec<&Version> = match &entry.detail {
            Detail::None => Vec::new(),
            Detail::Version(v) => vec![v],
            Detail::State { version, .. } => version.iter().collect(),
            Detail::History(steps) => steps.iter().map(|s| &s.version).collect(),
        };
        for version in versions {
            let Some(place) = releases.place(version) else {
                continue;
            };
            let what = match place {
                Placement::In(line, LineStatus::Current) => {
                    format!("in line {}, the current line", line.text)
                }
                Placement::In(line, status) => {
                    format!("in line {}, {}", line.text, status.as_str())
                }
                Placement::Ahead { newest } => {
                    format!(
                        "newer than every declared line; the newest is {}",
                        newest.text
                    )
                }
                Placement::Between { line, older, newer } => format!(
                    "in line {line}, which isn't declared: between {} and {}",
                    older.text, newer.text
                ),
                // Older than every line: in every line, so nothing to say.
                Placement::Before => continue,
            };
            let item = format!("- `{} {}`: {what}", entry.target.text, version.text);
            if !items.contains(&item) {
                items.push(item);
            }
        }
    }
    if items.is_empty() {
        return String::new();
    }
    format!("\n\n{}", items.join("\n"))
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
        Primary::Reason { .. } if schema.origin == Origin::Builtin(Builtin::Memo) => {
            Some("the memo's text, as plain text")
        }
        Primary::Reason { .. } => Some("the reason, as plain text"),
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

/// The rule of `[listing]` a page at `path` of the type named `type_name` is
/// under, for its `listing:` line.
fn listing_markdown(model: &ContentModel, path: &str, type_name: Option<&str>) -> String {
    let values = "`listed`, then `unquoted`, then `unlisted`";
    match model.listing_rule(path, type_name) {
        Some(rule) => format!(
            "**Under `{}`**, line {} of `ascribe.toml`.\n\n\
             This page is at least `{}`. It can set that or a stricter value, \
             from {values}, and not a looser one.",
            rule.text,
            rule.line,
            rule.listing.name()
        ),
        None => format!(
            "**No rule of `[listing]` applies to this page.**\n\n\
             Its own value says how it's listed, from {values}."
        ),
    }
}
