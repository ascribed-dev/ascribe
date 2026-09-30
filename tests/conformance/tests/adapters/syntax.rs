//! The adapter for `tessera-syntax`: directive lines and the structure pass.
//!
//! It handles the `parser` tag (Ascribe-line recognition, directive heads,
//! attributes, and primaries, SPEC §3.1–§3.4) and the `structure` tag
//! (containers, groups, titles, binding, SPEC §3.5–§3.10, §4). Its outline
//! is `tessera_syntax::parse`'s tree, with the Ascribe nodes written by
//! [`super::structure`]. Its diagnostics are the parser's own issues.
//!
//! Directive schemas come from the built-ins plus the widgets in the case's
//! `ascribe.toml`, read with only what parsing needs, so a case can test the
//! parser without a model the loader would accept.

use tessera_conformance::outline::normalize_ws;
use tessera_conformance::{
    AdapterError, AdapterResult, Case, ConformanceAdapter, Diagnostic, Node,
};
use tessera_core::{
    self as core, Binding as SchemaBinding, DirectiveSchema, Forms, LineIndex, Origin, Primary,
    TitleRule, WideEncoding,
};
use tessera_syntax::{Block, BlockKind, ParseOptions, parse, raw_text};

use super::structure;

/// Handles the tags whose cases `tessera-syntax` alone can answer.
pub struct SyntaxAdapter;

impl ConformanceAdapter for SyntaxAdapter {
    fn name(&self) -> &str {
        "syntax"
    }

    fn handles_tag(&self, tag: &str) -> bool {
        matches!(tag, "parser" | "structure" | "inline")
    }

    fn outline(&self, case: &Case) -> AdapterResult<Vec<Node>> {
        let Some(source) = case.input().map_err(err)? else {
            return Ok(None);
        };
        let options = options(case)?;
        let doc = parse(&source, &options);
        Ok(Some(outline(&source, &doc.blocks)))
    }

    fn diagnostics(&self, case: &Case) -> AdapterResult<Vec<Diagnostic>> {
        // A case tagged `check` gets the whole file-level check,
        // which includes these issues and the checks that need the model.
        if case.expect.tags.iter().any(|t| t == "check") {
            return Ok(None);
        }
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

/// A widget's schema from its `[widgets.<name>]` table. Only what parsing
/// needs: forms, primary, binding, title, groupable.
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

fn outline(source: &str, blocks: &[Block]) -> Vec<Node> {
    let mut out: Vec<Node> = Vec::new();
    for block in blocks {
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
                children: outline(source, &q.children),
            }),
            BlockKind::List(l) => Some(Node::List {
                ordered: l.ordered,
                start: l.start,
                children: l
                    .items
                    .iter()
                    .map(|item| Node::Item {
                        children: outline(source, &item.children),
                    })
                    .collect(),
            }),
            BlockKind::HtmlBlock(h) => Some(Node::Html {
                text: Some(h.literal.trim_end().to_owned()),
            }),
            BlockKind::ThematicBreak => Some(Node::ThematicBreak),
            BlockKind::Table(_) => Some(Node::Table),
            // Directives, containers, groups, and the end lines that close
            // nothing.
            _ => structure::node(source, block, &|blocks| outline(source, blocks)),
        };
        out.extend(node);
    }
    out
}

pub(super) fn text(source: &str, span: core::Span) -> String {
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
