//! Hover (SPEC §10): what a link or include points at, a phrase's value, a
//! glossary term's definition (on its heading, with how many pages use it),
//! the availability a spec resolves to, the rule of `[listing]` a page is
//! under, what a directive is, which table of `ascribe.toml` declares a
//! frontmatter field, on its key in a page or in `ascribe.toml`, and a
//! section's summary, on its heading and its `@id` line.

use ascribe_core::availability::{AvailabilitySpec, Detail, Version, parse_availability};
use ascribe_core::schema::{
    AttributeSchema, AttributeType, Attributes, Builtin, DefaultValue, DirectiveSchema, Origin,
    Primary, SetMember,
};
use ascribe_core::{LineIndex, RelPath, Span};
use ascribe_emit::labels::{availability_display, availability_display_of_text};
use ascribe_model::{
    ContentModel, ContentType, Field, FieldSource, LineStatus, Placement, TypeFile,
};
use ascribe_resolve::openapi::Operation;
use ascribe_resolve::{DefaultRouter, FileIndex, Heading, Resolution, Snapshot, Usable};
use lsp_types::{Hover, HoverContents, MarkupContent, MarkupKind, Position, Range};

use crate::model_file::ModelFile;
use crate::nav::{Ctx, Hit, Lines, first_paragraph, hit_at};

/// On an occurrence of a glossary term in prose, the term and its
/// definition in the editor's build (SPEC §5.4).
fn term_hover(ctx: &Ctx, offset: usize) -> Option<Hover> {
    let (id, span) = ctx
        .snapshot
        .uses_in(&ctx.path)
        .into_iter()
        .find_map(|(used, place)| match used {
            Usable::Term(id) if place.span.start() <= offset && offset <= place.span.end() => {
                Some((id, place.span))
            }
            _ => None,
        })?;
    let project = ctx.snapshot.project();
    let term = project.glossary().term(&id)?.term.clone();
    let text = format!("**{term}**\n\n{}", term_definition(ctx, &id));
    markdown_hover(ctx, span, text)
}

/// On a glossary term's heading, its definition in the editor's build, and
/// how many pages use the term.
fn term_heading_hover(ctx: &Ctx, offset: usize) -> Option<Hover> {
    let file = ctx.file()?;
    let glossary = ctx.snapshot.glossary();
    let term = glossary.terms.iter().find(|t| {
        t.written_in == ctx.path && t.heading.start() <= offset && offset <= t.heading.end()
    })?;
    let mut pages: Vec<RelPath> = ctx
        .snapshot
        .uses(&Usable::Term(term.id.clone()))
        .into_iter()
        .map(|place| place.file)
        .collect();
    pages.dedup();
    let used = match pages.len() {
        0 => "No page uses it.".to_owned(),
        1 => "Used on 1 page.".to_owned(),
        n => format!("Used on {n} pages."),
    };
    let text = format!(
        "**{}**, a glossary term\n\n{}\n\n{used}",
        term.term,
        term_definition(ctx, &term.id)
    );
    let heading = file.heading_at(term.heading)?;
    markdown_hover(ctx, heading.span, text)
}

/// A term's definition in the editor's build, or that the build doesn't
/// link it.
fn term_definition(ctx: &Ctx, id: &str) -> String {
    let project = ctx.snapshot.project();
    let router = DefaultRouter::from_consumer(&ctx.model.consumer);
    project
        .resolver(ctx.model.editor_default_build(), &router)
        .term_definition(id)
        .filter(|d| !d.is_empty())
        .unwrap_or_else(|| "The editor's build doesn't link this glossary term.".to_owned())
}

/// On a heading, or on its `@id` line, the section's summary in the
/// editor's build (SPEC §4.12), as `_ascribe/help.json` writes it.
fn summary_hover(ctx: &Ctx, offset: usize) -> Option<Hover> {
    let file = ctx.file()?;
    let (heading, span) = section_at(file, offset)?;
    let summary = crate::summary::of_heading(ctx, heading.span)?;
    markdown_hover(ctx, span, crate::summary::markdown(&summary))
}

/// The heading `offset` is on, or whose `@id` line it's on, and the span
/// it's on: the heading's first line, or the `@id` line.
fn section_at(file: &FileIndex, offset: usize) -> Option<(&Heading, Span)> {
    let within = |span: Span| span.start() <= offset && offset <= span.end();
    file.headings.iter().find_map(|heading| {
        if let Some(id) = heading.explicit_id.as_ref().filter(|id| within(id.span)) {
            return Some((heading, id.span));
        }
        let text = file.source.get(heading.span.range())?;
        let first = text.find(['\n', '\r']).unwrap_or(text.len());
        let line = Span::new(heading.span.start(), heading.span.start() + first);
        within(line).then_some((heading, line))
    })
}

fn markdown_hover(ctx: &Ctx, span: Span, value: String) -> Option<Hover> {
    let range = Lines::new(ctx).range(&ctx.path, span)?;
    Some(Hover {
        contents: HoverContents::Markup(MarkupContent {
            kind: MarkupKind::Markdown,
            value,
        }),
        range: Some(range),
    })
}

/// The hover at a position of the requested file, and the advice
/// `ascribe.baseline` holds back on its line, which isn't a diagnostic, so
/// the hover is where the editor shows it (#790).
pub(crate) fn hover(ctx: &Ctx, position: Position) -> Option<Hover> {
    let held = crate::baseline::hover_text(ctx, position.line);
    let Some(held) = held else {
        return hover_at(ctx, position);
    };
    let Some(mut hover) = hover_at(ctx, position) else {
        let line = Range::new(
            Position::new(position.line, 0),
            Position::new(position.line + 1, 0),
        );
        let range = ctx
            .held
            .iter()
            .find(|d| d.range.start.line <= position.line && position.line <= d.range.end.line)
            .map_or(line, |d| d.range);
        return Some(Hover {
            contents: HoverContents::Markup(MarkupContent {
                kind: MarkupKind::Markdown,
                value: held,
            }),
            range: Some(range),
        });
    };
    if let HoverContents::Markup(markup) = &mut hover.contents {
        markup.value = format!("{}\n\n---\n\n{held}", markup.value);
    }
    Some(hover)
}

/// What's at a position of the requested file.
fn hover_at(ctx: &Ctx, position: Position) -> Option<Hover> {
    let file = ctx.file()?;
    let index = LineIndex::new(&file.source);
    let offset = ctx.encoding.offset_lenient(&index, &file.source, position);
    if let Some((name, span)) = crate::nav::frontmatter_key(file, offset)
        && let Some(ty) = ctx.page_type()
        && let Some(text) = page_field_markdown(&ctx.model, ty, name)
    {
        return markdown_hover(ctx, span, text);
    }
    let Some(hit) = hit_at(file, offset) else {
        return term_hover(ctx, offset)
            .or_else(|| term_heading_hover(ctx, offset))
            .or_else(|| summary_hover(ctx, offset));
    };
    let (span, text) = match hit {
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
        Hit::Operation(link, span) => (
            span,
            describe_resolution(ctx, &crate::nav::operation_resolution(ctx, link)?, link),
        ),
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
                None => {
                    let text = describe_directive(&schema);
                    // On a heading's `@id`, its section's summary first.
                    let summary = (d.name == "id")
                        .then(|| section_at(file, offset))
                        .flatten()
                        .and_then(|(heading, _)| crate::summary::of_heading(ctx, heading.span));
                    let text = match summary {
                        Some(summary) => {
                            format!("{}\n\n---\n\n{text}", crate::summary::markdown(&summary))
                        }
                        None => text,
                    };
                    (d.name_span, text)
                }
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
        Resolution::Operation { api, id } => {
            let project = ctx.snapshot.project();
            match project.apis().spec(api) {
                Some(spec) => match spec.named(id) {
                    Some(named) => operation_markdown(api, named.operation()),
                    None => format!("**Not found:** operation `{id}` of API `{api}`"),
                },
                None => format!("Operation `{id}` of API `{api}`, whose spec can't be read"),
            }
        }
        Resolution::SpecOutside { api } => format!(
            "The spec of API `{api}`, outside the project's folder: link to one of its operations"
        ),
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

/// An operation's method, in capitals, and its path: `GET /users/{id}`, as
/// a reference shows it.
pub(crate) fn method_and_path(operation: &Operation) -> String {
    operation.method_and_path_shown()
}

/// What a link to an operation names: its method and path, its summary,
/// whether it's deprecated, and its id, API, and tag.
pub(crate) fn operation_markdown(api: &str, operation: &Operation) -> String {
    let mut out = format!(
        "**{}** `{}`",
        operation.method.to_uppercase(),
        operation.path
    );
    if let Some(summary) = &operation.summary {
        out.push_str(&format!("\n\n{summary}"));
    }
    if operation.deprecated {
        out.push_str("\n\n**Deprecated**");
    }
    let mut about = match &operation.operation_id {
        Some(id) => format!("Operation `{id}` of API `{api}`"),
        None => format!("Operation of API `{api}`, with no id"),
    };
    if let Some(tag) = &operation.tag {
        about.push_str(&format!(", tagged {tag}"));
    }
    out.push_str(&format!("\n\n{about}"));
    out
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
        let mut items = planned_lines(model, &spec);
        items.extend(release_items(model, &model.resolve_planned(&spec).spec));
        if !items.is_empty() {
            out.push_str(&format!("\n\n{}", items.join("\n")));
        }
    }
    Some(out)
}

/// For each `planned` a spec writes, the line it names, as list items.
fn planned_lines(model: &ContentModel, spec: &AvailabilitySpec) -> Vec<String> {
    let mut items: Vec<String> = Vec::new();
    for entry in &spec.entries {
        let target = &entry.target.text;
        let one = AvailabilitySpec {
            entries: vec![entry.clone()],
            span: entry.span,
        };
        if !one.has_planned() {
            continue;
        }
        let resolved = model.resolve_planned(&one);
        let item = match (model.planned_line(target), resolved.problems.first()) {
            (Some(line), _) => format!("- `planned`: `{target}`'s planned line, {}", line.text),
            (None, Some(p)) if p.lines.is_empty() => {
                format!("- `planned`: `{target}` has no planned line")
            }
            (None, Some(p)) => {
                let lines: Vec<&str> = p.lines.iter().map(|l| l.text.as_str()).collect();
                format!(
                    "- `planned`: `{target}` has more than one planned line, {}",
                    lines.join(", ")
                )
            }
            (None, None) => continue,
        };
        if !items.contains(&item) {
            items.push(item);
        }
    }
    items
}

/// For each version a spec names of a target with `[releases]`, the line
/// it's in and the line's status, as a list after a blank line; nothing
/// when it names none.
fn release_lines(model: &ContentModel, spec: &AvailabilitySpec) -> String {
    let items = release_items(model, spec);
    if items.is_empty() {
        return String::new();
    }
    format!("\n\n{}", items.join("\n"))
}

/// [`release_lines`]'s items.
fn release_items(model: &ContentModel, spec: &AvailabilitySpec) -> Vec<String> {
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
    items
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

/// On a frontmatter key of a page of the type `ty`, the field and which
/// table of `ascribe.toml` declares it. `None` when the type has no field
/// of that name.
fn page_field_markdown(model: &ContentModel, ty: &ContentType, name: &str) -> Option<String> {
    let field = ty.frontmatter.fields.iter().find(|f| f.name == name)?;
    let declared = match model.field_source(ty, name)? {
        FieldSource::Frontmatter => {
            "Declared in `[frontmatter]`, which every page type shares.".to_owned()
        }
        FieldSource::Type if ty.implicit => {
            format!("Declared by the type `{}`.", ty.name)
        }
        FieldSource::Type => format!(
            "Declared by the type `{0}`, its own, in `[types.{0}.frontmatter]`.",
            ty.name
        ),
        FieldSource::Implicit => {
            "Every page type has it, a required string, though neither `[frontmatter]` nor the page's type declares it.".to_owned()
        }
    };
    Some(format!("{}\n\n{declared}", field_heading(field)))
}

/// A field's name and type, as a hover's first line.
fn field_heading(field: &Field) -> String {
    let required = if field.required { ", required" } else { "" };
    format!("**`{}`**: {}{required}", field.name, field.ty.describe())
}

/// On a field's key in `ascribe.toml`'s `[frontmatter]`, or in a type's own
/// `frontmatter`, which table it is, and what it means for the types: a
/// shared field says which types replace it, and a type's own says whether
/// it replaces a shared one. `text` is `ascribe.toml`'s text, and `offset`
/// is in it.
pub(crate) fn model_field_markdown(
    model: &ContentModel,
    text: &str,
    offset: usize,
) -> Option<(Span, String)> {
    let file = ModelFile::parse(text).ok()?;
    if let Some((name, span)) = file.key_at(&["frontmatter"], offset) {
        let field = model.frontmatter.iter().find(|f| f.name == name)?;
        let replaced: Vec<String> = model
            .types
            .iter()
            .filter(|t| t.own.iter().any(|f| f.name == name))
            .map(|t| format!("`{}`", t.name))
            .collect();
        let mut value = format!(
            "{}\n\nDeclared in `[frontmatter]`: every page type has it",
            field_heading(field)
        );
        match replaced.as_slice() {
            [] => value.push('.'),
            [one] => value.push_str(&format!(
                ", but the type {one} declares its own, which replaces it."
            )),
            many => value.push_str(&format!(
                ", but the types {} declare their own, which replace it.",
                many.join(", ")
            )),
        }
        return Some((span, value));
    }
    model.types.iter().filter(|t| !t.implicit).find_map(|ty| {
        let (name, span) = file.key_at(&["types", &ty.name, "frontmatter"], offset)?;
        let field = ty.own.iter().find(|f| f.name == name)?;
        let mut value = format!(
            "{}\n\nDeclared by the type `{}`, its own.",
            field_heading(field),
            ty.name
        );
        match model.frontmatter.iter().find(|f| f.name == name) {
            Some(shared) if shared == field => value.push_str(
                " It's the same as `[frontmatter]`'s, key for key, so it can be deleted: the type has that one without it.",
            ),
            Some(_) => value.push_str(" It replaces `[frontmatter]`'s, for this type."),
            None => {}
        }
        Some((span, value))
    })
}

/// A content type, on its name in `ascribe.toml`: its purpose, and where its
/// template and guide come from, the keys of its table or the lock `ascribe
/// templates eject` wrote (decision 809).
/// A name in `[from]`'s `take` list: where what it takes is written.
pub(crate) fn taken_markdown(taken: &ascribe_model::Taken) -> String {
    format!(
        "**`{}`**, taken from `{}`, where it's written, at line {}. It's read as if written here, and edited there.",
        taken.key, taken.file, taken.line
    )
}

pub(crate) fn type_markdown(t: &ContentType) -> String {
    let mut out = format!("**Content type `{}`**", t.name);
    if let Some(purpose) = &t.purpose {
        out.push_str(&format!("\n\n{purpose}"));
    }
    let listed = |files: &[&TypeFile]| -> String {
        files
            .iter()
            .map(|f| format!("`{}`", f.path))
            .collect::<Vec<_>>()
            .join(", ")
    };
    let guide: Vec<&TypeFile> = t.guide.iter().collect();
    let lock = t
        .template
        .iter()
        .chain(&t.guide)
        .find_map(|f| f.lock.as_deref());
    let mut said = Vec::new();
    if let Some(template) = &t.template {
        said.push(format!("Its template is `{}`", template.path));
    }
    if !guide.is_empty() {
        said.push(format!("its guide {}", listed(&guide)));
    }
    let what = said.join(", and ");
    out.push_str("\n\n");
    match lock {
        Some(lock) => out.push_str(&format!(
            "{}, from `{lock}`: `ascribe templates eject` wrote them, and the lock records them under the type's name.",
            capitalized(&what)
        )),
        None if !what.is_empty() => out.push_str(&format!(
            "{}, named in this table.",
            capitalized(&what)
        )),
        None => out.push_str(
            "It has no template or guide. A type takes them from `template` and `guide` in its table, or, naming neither, from `templates/templates.lock` when `ascribe templates eject` wrote one for it.",
        ),
    }
    out
}

/// `s` with its first letter a capital.
fn capitalized(s: &str) -> String {
    let mut chars = s.chars();
    chars
        .next()
        .map(|c| c.to_uppercase().chain(chars).collect())
        .unwrap_or_default()
}
