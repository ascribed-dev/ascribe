//! The adapter for `tessera-syntax`: what phase 05 produces.
//!
//! It handles the `parser` tag (Tessera-line recognition, directive heads,
//! attributes, and primaries, SPEC §3.1–§3.4). Its outline is the flat one
//! `tessera_syntax::parse` gives: directive lines and CommonMark blocks, with
//! no containers, groups, or titles yet, because those arrive with phase 06.
//! Its diagnostics are the parser's own issues.
//!
//! Directive schemas come from the built-ins plus the widgets in the case's
//! `tessera.toml`. That reader is a stand-in for phase 08's content-model
//! loader, which will replace it.

use tessera_conformance::outline::normalize_ws;
use tessera_conformance::{
    AdapterError, AdapterResult, AttrValue, Attributes, Binding, Case, ConformanceAdapter,
    Diagnostic, Directive, Form, Node,
};
use tessera_core::{
    self as core, AttributeValue, Binding as SchemaBinding, DirectiveSchema, Forms, LineIndex,
    Origin, Primary, TitleRule, WideEncoding,
};
use tessera_syntax::{
    Block, BlockKind, DirectiveLine, ParseOptions, PrimaryValue, parse, raw_text,
};

/// Handles the tags whose cases `tessera-syntax` alone can answer.
pub struct SyntaxAdapter;

impl ConformanceAdapter for SyntaxAdapter {
    fn name(&self) -> &str {
        "syntax"
    }

    fn handles_tag(&self, tag: &str) -> bool {
        tag == "parser" || tag == "inline"
    }

    fn outline(&self, case: &Case) -> AdapterResult<Vec<Node>> {
        let Some(source) = case.input().map_err(err)? else {
            return Ok(None);
        };
        let options = options(case)?;
        let doc = parse(&source, &options);
        Ok(Some(outline(&source, &options, &doc.blocks)))
    }

    fn diagnostics(&self, case: &Case) -> AdapterResult<Vec<Diagnostic>> {
        let options = options(case)?;
        let mut out = Vec::new();
        for file in case.source_files().map_err(err)? {
            if !file.ends_with(".md") {
                continue;
            }
            let path = case.content_root().join(&file);
            let source = std::fs::read_to_string(&path)
                .map_err(|e| AdapterError(format!("couldn't read {}: {e}", path.display())))?;
            let doc = parse(&source, &options);
            let index = LineIndex::new(&source);
            for issue in &doc.issues {
                let pos = index
                    .wide_line_col(WideEncoding::Utf32, issue.location.span.start())
                    .ok_or_else(|| AdapterError(format!("bad location in {file}")))?;
                out.push(Diagnostic {
                    slug: issue.slug.as_str().to_owned(),
                    file: file.clone(),
                    line: pos.line + 1,
                    column: pos.col + 1,
                });
            }
        }
        Ok(Some(out))
    }
}

fn err(e: impl std::fmt::Display) -> AdapterError {
    AdapterError(e.to_string())
}

// ---------------------------------------------------------------------------
// Options: the built-ins plus the case model's widgets

fn options(case: &Case) -> Result<ParseOptions, AdapterError> {
    let mut schemas = core::builtin_schemas();
    let mut note_types = None;
    if let Ok(text) = std::fs::read_to_string(&case.model) {
        let model: toml::Table = text
            .parse()
            .map_err(|e| AdapterError(format!("couldn't read {}: {e}", case.model.display())))?;
        if let Some(widgets) = model.get("widgets").and_then(toml::Value::as_table) {
            for (name, widget) in widgets {
                if let Some(widget) = widget.as_table() {
                    schemas.push(widget_schema(name, widget));
                }
            }
        }
        if let Some(notes) = model.get("notes").and_then(toml::Value::as_table) {
            let mut types: Vec<String> = ["note", "tip", "important", "warning", "caution"]
                .map(String::from)
                .to_vec();
            let added: Vec<String> = notes
                .keys()
                .filter(|k| !types.contains(k))
                .cloned()
                .collect();
            types.extend(added);
            note_types = Some(types);
        }
    }
    let mut options = ParseOptions::new(schemas);
    if let Some(types) = note_types {
        options = options.with_note_types(types);
    }
    Ok(options)
}

/// A widget's schema from its `[widgets.<name>]` table (content-model.md
/// §15). Only what parsing needs: forms, primary, binding, title, groupable.
fn widget_schema(name: &str, widget: &toml::Table) -> DirectiveSchema {
    let forms: Vec<&str> = widget
        .get("forms")
        .and_then(toml::Value::as_array)
        .map(|a| a.iter().filter_map(toml::Value::as_str).collect())
        .unwrap_or_default();
    let forms = Forms {
        line: forms.contains(&"line"),
        container: forms.contains(&"container"),
    };
    let primary = match widget.get("primary").and_then(toml::Value::as_str) {
        Some("identifier") => Primary::Identifier { required: true },
        Some("identifier?") => Primary::Identifier { required: false },
        Some("text") => Primary::Text { required: true },
        Some("text?") => Primary::Text { required: false },
        _ => Primary::None,
    };
    let binding = match widget.get("binding").and_then(toml::Value::as_str) {
        Some("self") => Some(SchemaBinding::SelfBound),
        Some("heading") => Some(SchemaBinding::Heading),
        Some("block") => Some(SchemaBinding::Block),
        Some("heading-or-block") => Some(SchemaBinding::HeadingOrBlock),
        _ => None,
    };
    let title = match widget.get("title").and_then(toml::Value::as_str) {
        Some("accepted") => TitleRule::Accepted,
        Some("required") => TitleRule::Required,
        _ => TitleRule::None,
    };
    DirectiveSchema {
        name: name.to_owned(),
        origin: Origin::Widget,
        forms,
        primary,
        binding: if forms.line { binding } else { None },
        title,
        groupable: widget
            .get("groupable")
            .and_then(toml::Value::as_bool)
            .unwrap_or(false),
        attributes: core::Attributes::Declared(Vec::new()),
        description: None,
    }
}

// ---------------------------------------------------------------------------
// Tree to outline

fn outline(source: &str, options: &ParseOptions, blocks: &[Block]) -> Vec<Node> {
    let mut out: Vec<Node> = Vec::new();
    // What each earlier sibling is bound to, for the top-of-section rule.
    let mut heading_bound: Vec<bool> = Vec::new();
    for block in blocks {
        let mut bound_to_heading = false;
        let node = match &block.kind {
            BlockKind::Heading(h) => Some(Node::Heading {
                level: h.level,
                text: Some(text(source, h.content)),
            }),
            BlockKind::Paragraph(p) => Some(paragraph(source, block, p)),
            BlockKind::CodeBlock(c) => Some(Node::Code {
                text: Some(c.literal.clone()),
                info: Some(c.info.clone()),
                fenced: Some(c.fenced),
            }),
            BlockKind::BlockQuote(q) => Some(Node::Blockquote {
                children: outline(source, options, &q.children),
            }),
            BlockKind::List(l) => Some(Node::List {
                ordered: l.ordered,
                start: l.start,
                children: l
                    .items
                    .iter()
                    .map(|item| Node::Item {
                        children: outline(source, options, &item.children),
                    })
                    .collect(),
            }),
            BlockKind::HtmlBlock(h) => Some(Node::Html {
                text: Some(h.literal.trim_end().to_owned()),
            }),
            BlockKind::ThematicBreak => Some(Node::ThematicBreak),
            BlockKind::Table(_) => Some(Node::Table),
            BlockKind::Directive(line) => {
                // The binding is worked out from the schema and the siblings
                // before it; phase 06 does this properly, with containers.
                let binding = binding(options, line, &heading_bound, &out);
                bound_to_heading = binding == Some(Binding::Heading);
                Some(Node::Directive(directive(source, line, binding)))
            }
            // End lines close containers, which don't exist in this tree yet.
            BlockKind::End(_) => None,
            BlockKind::Container(_) | BlockKind::Group(_) | BlockKind::Title(_) => None,
        };
        if let Some(node) = node {
            let is_heading = matches!(node, Node::Heading { .. });
            heading_bound.push(bound_to_heading || is_heading);
            out.push(node);
        }
    }
    out
}

fn text(source: &str, span: core::Span) -> String {
    normalize_ws(&raw_text(source, span))
}

/// A paragraph, or an image when the paragraph is one image alone.
fn paragraph(source: &str, block: &Block, p: &tessera_syntax::Paragraph) -> Node {
    if let [only] = p.inlines.as_slice()
        && let tessera_syntax::InlineKind::Image(image) = &only.kind
    {
        return super::inline::image(source, image);
    }
    Node::Paragraph {
        text: Some(text(source, block.span)),
    }
}

fn binding(
    options: &ParseOptions,
    line: &DirectiveLine,
    heading_bound: &[bool],
    previous: &[Node],
) -> Option<Binding> {
    if line.form == tessera_syntax::Form::Container {
        return None;
    }
    let schema = options.schemas.iter().find(|s| s.name == line.name)?;
    Some(match schema.binding? {
        SchemaBinding::SelfBound => Binding::SelfBinding,
        SchemaBinding::Heading => Binding::Heading,
        SchemaBinding::Block => match line.primary {
            Some(PrimaryValue::Text(_)) => Binding::SelfBinding,
            _ => Binding::FollowingBlock,
        },
        SchemaBinding::HeadingOrBlock => {
            // At the top of a section: only heading-bound directives, back to
            // a heading (SPEC §3.8).
            let mut top = false;
            for (node, bound) in previous.iter().zip(heading_bound).rev() {
                if matches!(node, Node::Heading { .. }) {
                    top = true;
                    break;
                }
                if !bound {
                    break;
                }
            }
            if top {
                Binding::Heading
            } else {
                Binding::FollowingBlock
            }
        }
    })
}

fn directive(source: &str, line: &DirectiveLine, binding: Option<Binding>) -> Directive {
    let mut attributes = Attributes::new();
    if let Some(block) = &line.attributes {
        for a in &block.attributes {
            let Some(value) = &a.value else { continue };
            let value = match value {
                AttributeValue::Set { members, .. } => {
                    AttrValue::Set(members.iter().map(|m| m.text.clone()).collect())
                }
                v => AttrValue::Single(v.as_text().unwrap_or_default().to_owned()),
            };
            attributes.entry(a.key.clone()).or_insert(value);
        }
    }
    let primary = match &line.primary {
        Some(PrimaryValue::Text(p)) => Some(text(source, p.span)),
        Some(PrimaryValue::Identifier(p)) => Some(p.text.clone()),
        Some(PrimaryValue::Line(p)) => Some(p.text.clone()),
        Some(PrimaryValue::Unexpected(_)) | None => None,
    };
    Directive {
        name: line.name.clone(),
        form: match line.form {
            tessera_syntax::Form::Line => Form::Line,
            tessera_syntax::Form::Container => Form::Container,
        },
        attributes,
        primary,
        title: None,
        binding,
        children: Vec::new(),
    }
}
