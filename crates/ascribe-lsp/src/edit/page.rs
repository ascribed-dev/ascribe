//! Availability, and the page's frontmatter keys `variant` and `available`.
//!
//! The frontmatter is edited as text, so every other field, comment, and the
//! key order stay as written: a key's value is replaced, a missing key is
//! added at the end, and a page with no frontmatter gets one.

use ascribe_core::availability::parse_availability;
use ascribe_core::{Span, TextEdit};
use ascribe_syntax::{Block, BlockKind, Bound};

use super::{Outcome, Page, Plan, line_end};
use crate::context::ContextNode;

/// The `spec` argument: an availability spec, or a feature key.
fn spec_arg(page: &Page<'_>) -> Result<String, String> {
    let spec = page.line_arg("spec")?;
    let is_feature = page.ctx.model.feature(&spec).is_some();
    if !is_feature && let Err(e) = parse_availability(&spec, 0) {
        let features: Vec<String> = page
            .ctx
            .model
            .features
            .iter()
            .map(|f| format!("`{}`", f.key))
            .collect();
        let hint = if features.is_empty() {
            String::new()
        } else {
            format!(" The features are {}.", features.join(", "))
        };
        return Err(format!("`{spec}` isn't an availability spec: {e}.{hint}"));
    }
    Ok(spec)
}

/// What `markAvailable` marks.
enum Marked {
    Row(Span),
    Section(Span),
    Block(Span),
}

pub(crate) fn mark_available(page: &Page<'_>) -> Outcome {
    let spec = spec_arg(page)?;
    let marked = page
        .innermost(|node| match node {
            ContextNode::TableRow { header: true, .. } => Some(None),
            ContextNode::TableRow { .. } => Some(Some(true)),
            ContextNode::Heading { .. } => Some(Some(false)),
            ContextNode::Paragraph { .. }
            | ContextNode::CodeBlock { .. }
            | ContextNode::Table { .. }
            | ContextNode::List { .. }
            | ContextNode::BlockQuote { .. }
            | ContextNode::Note { .. }
            | ContextNode::Details { .. }
            | ContextNode::Steps { .. }
            | ContextNode::VariantGroup { .. }
            | ContextNode::Widget { .. }
            | ContextNode::Include { .. }
            | ContextNode::Snippet { .. } => Some(Some(false)),
            _ => None,
        })
        .ok_or_else(|| "Put the cursor in a heading, a block, or a table row.".to_owned())?;
    let marked = match marked {
        (_, None) => return Err("A table's header row can't have availability.".to_owned()),
        (span, Some(true)) => Marked::Row(span),
        (span, Some(false)) => {
            let heading = page
                .found
                .iter()
                .find(|(s, _)| *s == span)
                .is_some_and(|(_, node)| matches!(node, ContextNode::Heading { .. }));
            if heading {
                Marked::Section(span)
            } else {
                Marked::Block(span)
            }
        }
    };
    let nl = page.nl;
    match marked {
        Marked::Section(span) => {
            let prefix = page.prefix(span.start());
            Ok(Plan::new(vec![TextEdit::insert(
                span.end(),
                format!("{nl}{prefix}@available: {spec}"),
            )]))
        }
        Marked::Block(span) => {
            let (siblings, i) = page.block_at(span.start()).ok_or_else(|| {
                "Put the cursor in a heading, a block, or a table row.".to_owned()
            })?;
            if tops_a_section(siblings, i) {
                return Err(
                    "The block starts its section, where `@available` marks the whole section. Put the cursor in the heading to mark the section."
                        .to_owned(),
                );
            }
            let prefix = page.prefix(span.start());
            Ok(Plan::new(vec![TextEdit::insert(
                span.start(),
                format!("@available: {spec}{nl}{prefix}"),
            )]))
        }
        Marked::Row(span) => mark_row(page, span, &spec),
    }
}

/// Whether `blocks[i]` comes first in a heading's section, where a
/// directive above it would bind the heading (SPEC §3.8).
fn tops_a_section(blocks: &[Block], i: usize) -> bool {
    blocks[..i]
        .iter()
        .rev()
        .find(|b| !matches!(&b.kind, BlockKind::Directive(l) if l.binding == Some(Bound::Heading)))
        .is_some_and(|b| matches!(b.kind, BlockKind::Heading(_)))
}

/// `{available=…}` at the end of a table row's first cell.
fn mark_row(page: &Page<'_>, span: Span, spec: &str) -> Outcome {
    let row = find_row(&page.file.document.blocks, span)
        .ok_or_else(|| "Put the cursor in a table row.".to_owned())?;
    if row.attributes.is_some() {
        return Err("The row already has availability; change it by hand.".to_owned());
    }
    let cell = row
        .cells
        .first()
        .ok_or_else(|| "The row has no cells.".to_owned())?;
    let value = ascribe_fmt::written_value(spec);
    let at = page.source[..cell.span.end()].trim_end().len();
    Ok(Plan::new(vec![TextEdit::insert(
        at,
        format!(" {{available={value}}}"),
    )]))
}

fn find_row(blocks: &[Block], span: Span) -> Option<&ascribe_syntax::TableRow> {
    blocks.iter().find_map(|block| {
        if !block.span.contains_span(span) {
            return None;
        }
        match &block.kind {
            BlockKind::Table(t) => t.rows.iter().find(|r| r.span == span),
            BlockKind::BlockQuote(q) => find_row(&q.children, span),
            BlockKind::List(l) => l.items.iter().find_map(|i| find_row(&i.children, span)),
            BlockKind::Container(c) => find_row(&c.children, span),
            BlockKind::Group(g) => g.arms.iter().find_map(|a| find_row(&a.children, span)),
            _ => None,
        }
    })
}

/// A scalar as YAML writes it: plain when it reads back as the same string,
/// else double-quoted.
fn yaml_scalar(text: &str) -> String {
    let plain = serde_yaml_ng::from_str::<serde_yaml_ng::Value>(text)
        .ok()
        .is_some_and(|v| v.as_str() == Some(text));
    if plain && !text.contains(['#', '\n']) {
        text.to_owned()
    } else {
        format!("\"{}\"", text.replace('\\', "\\\\").replace('"', "\\\""))
    }
}

/// A top-level key of the frontmatter: its line, and its value from after
/// the colon through its last line (lines indented below it, or list items).
struct Key {
    /// From the start of the key's line through the end of its value.
    span: Span,
    /// The value on the key's own line, trimmed; empty when it's on the
    /// lines below.
    inline: Span,
    /// The lines below, each `(indent, start of text, end)`.
    children: Vec<(usize, usize, usize)>,
}

fn frontmatter_key(page: &Page<'_>, content: Span, key: &str) -> Option<Key> {
    let source = page.source;
    let prefix = format!("{key}:");
    let mut at = content.start();
    while at < content.end() {
        let end = line_end(source, at);
        let next = (end
            + if source[end..].starts_with("\r\n") {
                2
            } else {
                1
            })
        .min(content.end());
        let line = &source[at..end];
        if let Some(rest) = line.strip_prefix(&prefix)
            && (rest.is_empty() || rest.starts_with([' ', '\t']))
        {
            let lead = rest.len() - rest.trim_start().len();
            let from = at + prefix.len() + lead;
            let inline = Span::new(from, from + rest.trim().len());
            let mut children = Vec::new();
            let mut last = inline.end();
            let mut cursor = next;
            while cursor < content.end() {
                let e = line_end(source, cursor);
                let text = &source[cursor..e];
                let continues = text.starts_with([' ', '\t', '-']) || text.trim().is_empty();
                if !continues {
                    break;
                }
                if !text.trim().is_empty() {
                    let indent = text.len() - text.trim_start().len();
                    children.push((indent, cursor + indent, cursor + text.trim_end().len()));
                    last = cursor + text.trim_end().len();
                }
                cursor = (e + if source[e..].starts_with("\r\n") {
                    2
                } else {
                    1
                })
                .min(content.end());
            }
            return Some(Key {
                span: Span::new(at, last),
                inline,
                children,
            });
        }
        at = next;
    }
    None
}

/// The frontmatter's content, or an edit that adds a frontmatter holding
/// `lines` when the page has none.
fn frontmatter(page: &Page<'_>, lines: &str) -> Result<Result<Span, TextEdit>, String> {
    if page.file.kind != ascribe_resolve::FileKind::Page {
        return Err(
            "A fragment has no frontmatter keys of its own; set them on the page.".to_owned(),
        );
    }
    match &page.file.document.frontmatter {
        Some(f) => Ok(Ok(f.content)),
        None => {
            let nl = page.nl;
            Ok(Err(TextEdit::insert(
                0,
                format!("---{nl}{lines}{nl}---{nl}"),
            )))
        }
    }
}

pub(crate) fn set_available(page: &Page<'_>) -> Outcome {
    let spec = spec_arg(page)?;
    let value = yaml_scalar(&spec);
    let line = format!("available: {value}");
    let content = match frontmatter(page, &line)? {
        Ok(content) => content,
        Err(add) => return Ok(Plan::new(vec![add])),
    };
    let edit = match frontmatter_key(page, content, "available") {
        Some(key) => {
            let span = Span::new(key.inline.start(), key.span.end());
            if page.text(span) == value {
                return Err("The page already has that availability.".to_owned());
            }
            TextEdit::replace(span, value)
        }
        None => TextEdit::insert(content.end(), format!("{line}{}", page.nl)),
    };
    Ok(Plan::new(vec![edit]))
}

pub(crate) fn set_variant(page: &Page<'_>) -> Outcome {
    let model = &page.ctx.model;
    let name = page.str_arg("dimension")?;
    let Some(dimension) = model.dimension(&name) else {
        let names: Vec<String> = model
            .dimensions
            .iter()
            .map(|d| format!("`{}`", d.name))
            .collect();
        return Err(if names.is_empty() {
            "The content model declares no dimensions.".to_owned()
        } else {
            format!(
                "`{name}` isn't a dimension. The dimensions are {}.",
                names.join(", ")
            )
        });
    };
    let value = page.str_arg("value")?;
    if !dimension.values.iter().any(|v| v.value == value) {
        let known: Vec<String> = dimension
            .values
            .iter()
            .map(|v| format!("`{}`", v.value))
            .collect();
        return Err(format!(
            "`{value}` isn't a value of `{name}`. Its values are {}.",
            known.join(", ")
        ));
    }
    let nl = page.nl;
    let written = yaml_scalar(&value);
    let entry = format!("{name}: {written}");
    let content = match frontmatter(page, &format!("variant:{nl}  {entry}"))? {
        Ok(content) => content,
        Err(add) => return Ok(Plan::new(vec![add])),
    };
    let Some(key) = frontmatter_key(page, content, "variant") else {
        return Ok(Plan::new(vec![TextEdit::insert(
            content.end(),
            format!("variant:{nl}  {entry}{nl}"),
        )]));
    };
    let source = page.source;
    if !key.inline.is_empty() {
        // A flow mapping, `variant: {pm: npm}`: written again as one, with
        // the dimension set.
        let text = page.text(key.inline);
        let Ok(serde_yaml_ng::Value::Mapping(mut map)) =
            serde_yaml_ng::from_str::<serde_yaml_ng::Value>(text)
        else {
            return Err("The page's `variant` isn't a mapping; change it by hand.".to_owned());
        };
        map.insert(name.clone().into(), value.clone().into());
        let pairs: Vec<String> = map
            .iter()
            .filter_map(|(k, v)| Some(format!("{}: {}", k.as_str()?, yaml_scalar(v.as_str()?))))
            .collect();
        if pairs.len() != map.len() {
            return Err("The page's `variant` can't be read; change it by hand.".to_owned());
        }
        return Ok(Plan::new(vec![TextEdit::replace(
            key.inline,
            format!("{{{}}}", pairs.join(", ")),
        )]));
    }
    let indent = key.children.first().map_or(2, |(indent, _, _)| *indent);
    for (_, start, end) in &key.children {
        let line = &source[*start..*end];
        if let Some(rest) = line.strip_prefix(&format!("{name}:")) {
            let lead = rest.len() - rest.trim_start().len();
            let from = start + name.len() + 1 + lead;
            let span = Span::new(from, *end);
            if page.text(span) == written {
                return Err(format!("The page's `{name}` is already `{value}`."));
            }
            return Ok(Plan::new(vec![TextEdit::replace(span, written)]));
        }
    }
    Ok(Plan::new(vec![TextEdit::insert(
        key.span.end(),
        format!("{nl}{}{entry}", " ".repeat(indent)),
    )]))
}
