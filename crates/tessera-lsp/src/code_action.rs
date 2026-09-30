//! Quick fixes for diagnostics and safe Ascribe-specific repairs.

use std::collections::HashMap;

use lsp_types::{
    CodeAction, CodeActionKind, CodeActionOrCommand, CodeActionParams, Diagnostic, TextEdit,
    WorkspaceEdit,
};
use tessera_core::{LineIndex, Span};

use crate::nav::{Ctx, directive_at};

pub(crate) fn actions(ctx: &Ctx, params: CodeActionParams) -> Vec<CodeActionOrCommand> {
    let Some(file) = ctx.file() else {
        return Vec::new();
    };
    let index = LineIndex::new(&file.source);
    let start = ctx
        .encoding
        .offset_lenient(&index, &file.source, params.range.start);
    let end = ctx
        .encoding
        .offset_lenient(&index, &file.source, params.range.end);
    let mut actions = Vec::new();

    for diagnostic in params.context.diagnostics {
        let Some(slug) = diagnostic
            .data
            .as_ref()
            .and_then(|data| data.get("slug"))
            .and_then(serde_json::Value::as_str)
        else {
            continue;
        };
        let fixes = diagnostic
            .data
            .as_ref()
            .and_then(|data| data.get("fixes"))
            .and_then(serde_json::Value::as_array);
        if let Some(fixes) = fixes {
            for fix in fixes {
                let Some(title) = fix.get("title").and_then(serde_json::Value::as_str) else {
                    continue;
                };
                let edits: Vec<TextEdit> = fix
                    .get("edits")
                    .and_then(serde_json::Value::as_array)
                    .into_iter()
                    .flatten()
                    .filter_map(|edit| serde_json::from_value(edit.clone()).ok())
                    .collect();
                if !edits.is_empty() {
                    actions.push(action(ctx, title, vec![diagnostic.clone()], edits));
                }
            }
        }

        let at = ctx
            .encoding
            .offset_lenient(&index, &file.source, diagnostic.range.start);
        let diagnostic_end =
            ctx.encoding
                .offset_lenient(&index, &file.source, diagnostic.range.end);
        if diagnostic_end < start || at > end {
            continue;
        }
        match slug {
            "attribute-unquoted-reserved" => {
                if let Some(raw) = file.source.get(at..diagnostic_end) {
                    let quoted = format!("\"{}\"", raw.replace('\\', "\\\\").replace('"', "\\\""));
                    actions.push(action(
                        ctx,
                        "Quote the attribute value",
                        vec![diagnostic.clone()],
                        vec![TextEdit {
                            range: diagnostic.range,
                            new_text: quoted,
                        }],
                    ));
                }
            }
            "container-colon-missing" | "container-colon-unexpected" => {
                if let Some(edit) =
                    container_colon_edit(&file.source, &file.document.blocks, at, slug)
                {
                    actions.push(action(
                        ctx,
                        if slug == "container-colon-missing" {
                            "Add a trailing colon"
                        } else {
                            "Remove the stray colon"
                        },
                        vec![diagnostic.clone()],
                        vec![text_edit(ctx, &file.source, edit)],
                    ));
                }
            }
            "binding-blank-line" => {
                if let Some(span) = blank_line_edit(&file.source, at) {
                    actions.push(action(
                        ctx,
                        "Remove the blank line before the bound block",
                        vec![diagnostic.clone()],
                        vec![text_edit(
                            ctx,
                            &file.source,
                            tessera_core::TextEdit::delete(span),
                        )],
                    ));
                }
            }
            "phrase-undeclared" => {
                if let Some(use_) = file
                    .phrases
                    .iter()
                    .find(|use_| !use_.declared && use_.phrase.span.contains(at))
                {
                    actions.push(action(
                        ctx,
                        "Escape this phrase as literal text",
                        vec![diagnostic.clone()],
                        vec![text_edit(
                            ctx,
                            &file.source,
                            tessera_core::TextEdit::insert(use_.phrase.span.start(), "\\"),
                        )],
                    ));
                    if let Some((at, new_text)) =
                        phrase_declaration(&ctx.model_text, &use_.phrase.key)
                    {
                        let model_index = LineIndex::new(&ctx.model_text);
                        actions.push(workspace_action(
                            ctx,
                            "Declare phrase in ascribe.toml",
                            vec![diagnostic.clone()],
                            Vec::new(),
                            Some(TextEdit {
                                range: ctx.encoding.range(&model_index, Span::empty(at)),
                                new_text,
                            }),
                        ));
                    }
                }
            }
            _ => {}
        }
    }
    for heading in &file.headings {
        let start = ctx
            .encoding
            .offset_lenient(&index, &file.source, params.range.start);
        let end = ctx
            .encoding
            .offset_lenient(&index, &file.source, params.range.end);
        if heading.explicit_id.is_some()
            || heading.source_id.is_empty()
            || heading.span.end() < start
            || heading.span.start() > end
        {
            continue;
        }
        let line_end = file.source[heading.span.start()..]
            .find(['\r', '\n'])
            .map_or(file.source.len(), |i| heading.span.start() + i);
        let edit =
            tessera_core::TextEdit::insert(line_end, format!("\n@id: {}", heading.source_id));
        actions.push(action(
            ctx,
            "Add a stable @id for this heading",
            Vec::new(),
            vec![text_edit(ctx, &file.source, edit)],
        ));
    }
    actions
}

fn action(
    ctx: &Ctx,
    title: &str,
    diagnostics: Vec<Diagnostic>,
    edits: Vec<TextEdit>,
) -> CodeActionOrCommand {
    workspace_action(ctx, title, diagnostics, edits, None)
}

#[allow(clippy::mutable_key_type)]
fn workspace_action(
    ctx: &Ctx,
    title: &str,
    diagnostics: Vec<Diagnostic>,
    source_edits: Vec<TextEdit>,
    config_edit: Option<TextEdit>,
) -> CodeActionOrCommand {
    let mut changes = HashMap::new();
    if !source_edits.is_empty()
        && let Some(uri) = ctx.uri_of(&ctx.path)
    {
        changes.insert(uri, source_edits);
    }
    if let (Some(edit), Some(uri)) = (config_edit, crate::uri::path_to_uri(&ctx.config)) {
        changes.insert(uri, vec![edit]);
    }
    CodeActionOrCommand::CodeAction(CodeAction {
        title: title.to_owned(),
        kind: Some(CodeActionKind::QUICKFIX),
        diagnostics: Some(diagnostics),
        edit: Some(WorkspaceEdit {
            changes: Some(changes),
            ..WorkspaceEdit::default()
        }),
        command: None,
        is_preferred: None,
        disabled: None,
        data: None,
    })
}

fn text_edit(ctx: &Ctx, source: &str, edit: tessera_core::TextEdit) -> TextEdit {
    TextEdit {
        range: ctx.encoding.range(&LineIndex::new(source), edit.span),
        new_text: edit.new_text,
    }
}

fn container_colon_edit(
    source: &str,
    blocks: &[tessera_syntax::Block],
    offset: usize,
    slug: &str,
) -> Option<tessera_core::TextEdit> {
    let line = directive_at(blocks, offset)?;
    let line_end = source[line.span.start()..]
        .find(['\r', '\n'])
        .map_or(source.len(), |i| line.span.start() + i);
    if slug == "container-colon-missing" {
        if line.colon.is_some() {
            return None;
        }
        let segment = source.get(line.span.start()..line_end)?;
        let trim = segment.trim_end_matches([' ', '\t']).len();
        Some(tessera_core::TextEdit::insert(
            line.span.start() + trim,
            ":",
        ))
    } else {
        Some(tessera_core::TextEdit::delete(line.colon?))
    }
}

fn blank_line_edit(source: &str, offset: usize) -> Option<Span> {
    let line_start = source[..offset].rfind('\n').map_or(0, |i| i + 1);
    let line_end = source[offset..]
        .find(['\r', '\n'])
        .map_or(source.len(), |i| offset + i);
    let (start, end) = if source.get(line_start..line_end)?.trim().is_empty() {
        (line_start, line_end)
    } else {
        let next_start = if source[line_end..].starts_with("\r\n") {
            line_end + 2
        } else if line_end < source.len() {
            line_end + 1
        } else {
            return None;
        };
        let next_end = source[next_start..]
            .find(['\r', '\n'])
            .map_or(source.len(), |i| next_start + i);
        if !source.get(next_start..next_end)?.trim().is_empty() {
            return None;
        }
        (next_start, next_end)
    };
    let after = if source[end..].starts_with("\r\n") {
        end + 2
    } else if end < source.len() {
        end + 1
    } else {
        end
    };
    (after > start).then_some(Span::new(start, after))
}

fn phrase_declaration(model: &str, key: &str) -> Option<(usize, String)> {
    if key.is_empty()
        || !key
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
    {
        return None;
    }

    let quoted_key = format!("\"{}\"", key.replace('\\', "\\\\").replace('"', "\\\""));
    let declaration = format!("{quoted_key} = \"\"\n");
    let mut section_start = None;
    let mut section_end = model.len();
    let mut at = 0;
    for line in model.split_inclusive('\n') {
        let trimmed = line.trim();
        if trimmed.starts_with('[') && !trimmed.starts_with("[[") {
            let name = trimmed.strip_prefix('[')?.strip_suffix(']')?.trim();
            if section_start.is_some() {
                section_end = at;
                break;
            }
            if name == "phrases" {
                section_start = Some(at + line.len());
            }
        }
        at += line.len();
    }
    let range = if let Some(start) = section_start {
        let content = model.get(start..section_end)?;
        if content.lines().any(|line| {
            line.split_once('=')
                .is_some_and(|(name, _)| name.trim().trim_matches(['"', '\'']) == key)
        }) {
            return None;
        }
        let insert = if start > 0 && model[..start].ends_with('\n') {
            start + content.trim_end_matches(['\r', '\n']).len()
        } else {
            section_end
        };
        let prefix = if !model[..insert].ends_with('\n') {
            "\n"
        } else {
            ""
        };
        (insert, format!("{prefix}{declaration}"))
    } else {
        if model.contains(&format!("{quoted_key} =")) {
            return None;
        }
        let prefix = if model.is_empty() || model.ends_with("\n\n") {
            ""
        } else if model.ends_with('\n') {
            "\n"
        } else {
            "\n\n"
        };
        let new_text = format!("{prefix}[phrases]\n{declaration}");
        (model.len(), new_text)
    };
    Some(range)
}
