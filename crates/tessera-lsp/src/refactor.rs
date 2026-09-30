//! Project-wide source refactorings and file-operation edits.

use std::collections::HashMap;
use std::str::FromStr;

use lsp_types::{
    FileOperationFilter, FileOperationPattern, FileOperationPatternKind,
    FileOperationRegistrationOptions, RenameFilesParams, RenameParams, TextEdit, Uri,
    WorkspaceEdit,
};
use tessera_core::{Destination, LineIndex, RelPath, Span, TextEdit as ByteEdit};
use tessera_resolve::{FileIndex, PhrasePlace, Target};
use tessera_syntax::{Block, BlockKind, InlineKind};

use crate::nav::{Ctx, Lines, directive_at, encode_destination, hit_at, identifier_primary};

pub(crate) fn file_operation_capability() -> FileOperationRegistrationOptions {
    FileOperationRegistrationOptions {
        filters: vec![FileOperationFilter {
            scheme: Some("file".to_owned()),
            pattern: FileOperationPattern {
                glob: "**/*".to_owned(),
                matches: Some(FileOperationPatternKind::File),
                options: None,
            },
        }],
    }
}

pub(crate) fn will_rename(ctx: &Ctx, params: RenameFilesParams) -> Option<WorkspaceEdit> {
    let mut mappings = Vec::new();
    let root = ctx.config.parent()?;
    for file in params.files {
        let old_uri = Uri::from_str(&file.old_uri).ok()?;
        let new_uri = Uri::from_str(&file.new_uri).ok()?;
        let old_abs = crate::uri::normalize(&crate::uri::uri_to_path(&old_uri)?);
        let new_abs = crate::uri::normalize(&crate::uri::uri_to_path(&new_uri)?);
        if crate::uri::relative_to(root, &old_abs).is_none()
            || crate::uri::relative_to(root, &new_abs).is_none()
            || (new_abs.exists() && new_abs != old_abs)
        {
            return Some(WorkspaceEdit::default());
        }
        let old = crate::uri::relative_to(&ctx.content_dir, &old_abs)?;
        let new = crate::uri::relative_to(&ctx.content_dir, &new_abs)?;
        if old == ".." || old.starts_with("../") || new == ".." || new.starts_with("../") {
            return Some(WorkspaceEdit::default());
        }
        mappings.push((RelPath::parse(&old).ok()?, RelPath::parse(&new).ok()?));
    }
    if mappings.is_empty() {
        return Some(WorkspaceEdit::default());
    }

    let mut edits: HashMap<RelPath, Vec<ByteEdit>> = HashMap::new();
    let mut phrase_edits = HashMap::new();
    for file in ctx.snapshot.files() {
        let new_owner = remap(&file.path, &mappings);
        for reference in &file.references {
            let Target::Local(local) = &reference.target else {
                continue;
            };
            let Some(target) = local.path.as_ref() else {
                continue;
            };
            let new_target = remap(target, &mappings);
            if target == &new_target && file.path == new_owner {
                continue;
            }
            let spans = destination_spans(file, reference.destination_span, &reference.destination);
            for span in spans {
                let Some(raw) = file.source.get(span.range()) else {
                    continue;
                };
                let raw_path_end = raw.find(['?', '#']).unwrap_or(raw.len());
                let raw_path = &raw[..raw_path_end];
                let destination_phrase = file.phrases.iter().find(|phrase| {
                    phrase.place == PhrasePlace::Destination
                        && span.start() <= phrase.phrase.key_span.start()
                        && phrase.phrase.key_span.end() <= span.end()
                        && raw_path == format!("{{{}}}", phrase.phrase.key)
                });
                let new_path =
                    destination_path(local.written.starts_with('/'), &new_owner, &new_target);
                if destination_phrase.is_some()
                    && let Some(replacement) =
                        preserved_phrase_destination(raw, &local.written, &new_path)
                {
                    if raw != replacement {
                        edits
                            .entry(file.path.clone())
                            .or_default()
                            .push(ByteEdit::replace(span, replacement));
                    }
                    continue;
                }
                if let Some(phrase) = destination_phrase
                    && phrase_uses(ctx, &phrase.phrase.key) == 1
                    && let Some(value) = ctx.model.phrase(&phrase.phrase.key)
                    && let Some(span) = phrase_value_span(&ctx.model_text, &phrase.phrase.key)
                {
                    let old_value_path_end = value.find(['?', '#']).unwrap_or(value.len());
                    let phrase_value = format!("{new_path}{}", &value[old_value_path_end..]);
                    if phrase_value != value {
                        phrase_edits.insert(
                            phrase.phrase.key.clone(),
                            TextEdit {
                                range: ctx.encoding.range(&LineIndex::new(&ctx.model_text), span),
                                new_text: serde_json::to_string(&phrase_value).ok()?,
                            },
                        );
                    }
                    continue;
                }
                let replacement = rewrite_destination(
                    raw,
                    local.written.starts_with('/'),
                    &new_owner,
                    &new_target,
                    &local.written,
                    file.phrases.iter().any(|phrase| {
                        phrase.place == PhrasePlace::Destination
                            && span.start() <= phrase.phrase.key_span.start()
                            && phrase.phrase.key_span.end() <= span.end()
                    }),
                );
                if raw != replacement {
                    edits
                        .entry(file.path.clone())
                        .or_default()
                        .push(ByteEdit::replace(span, replacement));
                }
            }
        }
        for include in &file.includes {
            let (Some(target), Some(primary)) = (&include.target, include.primary) else {
                continue;
            };
            let new_target = remap(target, &mappings);
            if target == &new_target && file.path == new_owner {
                continue;
            }
            let Some(raw) = file.source.get(primary.range()) else {
                continue;
            };
            let local = match tessera_core::classify_destination(&include.written) {
                Destination::Local(local) => local,
                Destination::External => continue,
            };
            let replacement = rewrite_destination(
                raw,
                local.root_relative,
                &new_owner,
                &new_target,
                &include.written,
                false,
            );
            if raw != replacement {
                edits
                    .entry(file.path.clone())
                    .or_default()
                    .push(ByteEdit::replace(primary, replacement));
            }
        }
    }
    let mut result = workspace_edit(ctx, edits);
    if !phrase_edits.is_empty() {
        let uri = crate::uri::path_to_uri(&ctx.config)?;
        result
            .changes
            .get_or_insert_with(HashMap::new)
            .insert(uri, phrase_edits.into_values().collect());
    }
    Some(result)
}

pub(crate) fn rename(ctx: &Ctx, params: RenameParams) -> Option<WorkspaceEdit> {
    let file = ctx.file()?;
    let index = LineIndex::new(&file.source);
    let offset =
        ctx.encoding
            .offset_lenient(&index, &file.source, params.text_document_position.position);
    if let Some(crate::nav::Hit::Phrase(phrase)) = hit_at(file, offset) {
        if !valid_phrase_key(&params.new_name) {
            return None;
        }
        return rename_phrase(ctx, &phrase.phrase.key, &params.new_name, None);
    }
    if let Some(directive) = directive_at(&file.document.blocks, offset)
        && directive.name == "id"
        && let Some((old, span)) = identifier_primary(directive)
        && span.start() <= offset
        && offset <= span.end()
    {
        if !valid_id_or_phrase(&params.new_name)
            || old == params.new_name
            || file
                .headings
                .iter()
                .any(|heading| heading.source_id == params.new_name && heading.source_id != old)
        {
            return None;
        }
        let mut edits = HashMap::new();
        edits
            .entry(ctx.path.clone())
            .or_insert_with(Vec::new)
            .push(ByteEdit::replace(span, params.new_name.clone()));
        add_id_reference_edits(ctx, &ctx.path, old, &params.new_name, &mut edits);
        return Some(workspace_edit(ctx, edits));
    }
    rename_heading(ctx, offset, &params.new_name)
}

pub(crate) fn rename_model_key(
    ctx: &Ctx,
    position: lsp_types::Position,
    new_name: &str,
) -> Option<WorkspaceEdit> {
    if !valid_phrase_key(new_name) {
        return None;
    }
    let index = LineIndex::new(&ctx.model_text);
    let offset = ctx
        .encoding
        .offset_lenient(&index, &ctx.model_text, position);
    let (old, span) = phrase_entry_at(&ctx.model_text, offset)?;
    rename_phrase(ctx, &old, new_name, Some(span))
}

#[allow(clippy::mutable_key_type)]
fn rename_phrase(
    ctx: &Ctx,
    old: &str,
    new: &str,
    config_key: Option<Span>,
) -> Option<WorkspaceEdit> {
    if !valid_phrase_key(new)
        || old == new
        || !ctx.model.has_phrase(old)
        || ctx.model.has_phrase(new)
    {
        return None;
    }
    let mut edits: HashMap<RelPath, Vec<ByteEdit>> = HashMap::new();
    for file in ctx.snapshot.files() {
        for phrase in &file.phrases {
            if phrase.phrase.key == old {
                edits
                    .entry(file.path.clone())
                    .or_default()
                    .push(ByteEdit::replace(phrase.phrase.key_span, new.to_owned()));
            }
        }
    }
    let mut result = workspace_edit(ctx, edits);
    let mut changes = result.changes.take().unwrap_or_default();
    let model_span = config_key.or_else(|| phrase_entry_span(&ctx.model_text, old))?;
    let model_edit = TextEdit {
        range: ctx
            .encoding
            .range(&LineIndex::new(&ctx.model_text), model_span),
        new_text: new.to_owned(),
    };
    changes.insert(crate::uri::path_to_uri(&ctx.config)?, vec![model_edit]);
    result.changes = Some(changes);
    Some(result)
}

fn rename_heading(ctx: &Ctx, offset: usize, new_text: &str) -> Option<WorkspaceEdit> {
    let file = ctx.file()?;
    let (heading_span, span) = heading_text_at(&file.document.blocks, offset, &file.source)?;
    let old_heading = file.headings.iter().position(|h| h.span == heading_span)?;
    let edit = ByteEdit::replace(span, new_text.to_owned());
    let updated = tessera_core::apply_edits(&file.source, std::slice::from_ref(&edit)).ok()?;
    let slugger = tessera_resolve::slug::slugger_by_name(&ctx.model.consumer.slugger)
        .unwrap_or_else(tessera_resolve::slug::default_slugger);
    let indexed = tessera_resolve::index_file(
        file.file,
        &file.path,
        &updated,
        &ctx.model,
        slugger.as_ref(),
    );
    let new_id = indexed.headings.get(old_heading)?.source_id.clone();
    let old_id = &file.headings.get(old_heading)?.source_id;
    if new_id != *old_id
        && file
            .headings
            .iter()
            .enumerate()
            .any(|(i, h)| i != old_heading && h.source_id == new_id)
    {
        return None;
    }
    let mut edits = HashMap::new();
    edits
        .entry(ctx.path.clone())
        .or_insert_with(Vec::new)
        .push(edit);
    if new_id != *old_id {
        add_id_reference_edits(ctx, &ctx.path, old_id, &new_id, &mut edits);
    }
    Some(workspace_edit(ctx, edits))
}

fn add_id_reference_edits(
    ctx: &Ctx,
    target: &RelPath,
    old_id: &str,
    new_id: &str,
    edits: &mut HashMap<RelPath, Vec<ByteEdit>>,
) {
    for file in ctx.snapshot.files() {
        for include in &file.includes {
            if include.target.as_ref() != Some(target) || include.section.as_deref() != Some(old_id)
            {
                continue;
            }
            if let Some(span) = include.primary
                && let Some(raw) = file.source.get(span.range())
                && let Some(hash) = raw.find('#')
            {
                let mut replacement = raw[..=hash].to_owned();
                replacement.push_str(&encode_destination(new_id));
                edits
                    .entry(file.path.clone())
                    .or_default()
                    .push(ByteEdit::replace(span, replacement));
            }
        }
        for reference in &file.references {
            let Target::Local(local) = &reference.target else {
                continue;
            };
            if local.path.as_ref() != Some(target) || local.fragment.as_deref() != Some(old_id) {
                continue;
            }
            for span in destination_spans(file, reference.destination_span, &reference.destination)
            {
                if let Some(raw) = file.source.get(span.range())
                    && let Some(hash) = raw.find('#')
                {
                    let mut replacement = raw[..=hash].to_owned();
                    replacement.push_str(&encode_destination(new_id));
                    if raw.ends_with('>') && !replacement.ends_with('>') {
                        replacement.push('>');
                    }
                    edits
                        .entry(file.path.clone())
                        .or_default()
                        .push(ByteEdit::replace(span, replacement));
                }
            }
        }
    }
}

fn destination_spans(file: &FileIndex, inline: Option<Span>, url: &str) -> Vec<Span> {
    if let Some(span) = inline {
        return vec![span];
    }
    file.document
        .definitions
        .iter()
        .filter(|definition| definition.url == url)
        .map(|definition| definition.destination)
        .collect()
}

fn rewrite_destination(
    raw: &str,
    root_relative: bool,
    from: &RelPath,
    target: &RelPath,
    old_written: &str,
    has_phrase: bool,
) -> String {
    let angle = raw.starts_with('<') && raw.ends_with('>');
    let inner = raw
        .strip_prefix('<')
        .and_then(|r| r.strip_suffix('>'))
        .unwrap_or(raw);
    let suffix_at = inner.find(['?', '#']).unwrap_or(inner.len());
    let suffix = &inner[suffix_at..];
    let path = destination_path(root_relative, from, target);
    let path = if has_phrase {
        preserved_phrase_path(inner, old_written, &path).unwrap_or(path)
    } else {
        path
    };
    let mut result = format!("{path}{suffix}");
    if angle {
        result = format!("<{result}>");
    }
    result
}

fn preserved_phrase_destination(raw: &str, old_written: &str, new_path: &str) -> Option<String> {
    let angle = raw.starts_with('<') && raw.ends_with('>');
    let inner = raw
        .strip_prefix('<')
        .and_then(|r| r.strip_suffix('>'))
        .unwrap_or(raw);
    let suffix_at = inner.find(['?', '#']).unwrap_or(inner.len());
    let suffix = &inner[suffix_at..];
    let path = preserved_phrase_path(inner, old_written, new_path)?;
    let result = format!("{path}{suffix}");
    Some(if angle { format!("<{result}>") } else { result })
}

fn preserved_phrase_path(raw: &str, old_written: &str, new_path: &str) -> Option<String> {
    let raw_path_end = raw.find(['?', '#']).unwrap_or(raw.len());
    let raw_path = &raw[..raw_path_end];
    let old_path_end = old_written.find(['?', '#']).unwrap_or(old_written.len());
    let old_path = &old_written[..old_path_end];
    let prefix = new_path.strip_suffix(old_path)?;
    let is_boundary = prefix.is_empty()
        || prefix.ends_with('/')
        || (old_path.starts_with('/') && raw_path.starts_with('/'));
    if !is_boundary {
        return None;
    }
    Some(
        if !prefix.is_empty() && !prefix.ends_with('/') && !raw_path.starts_with('/') {
            format!("{prefix}/{raw_path}")
        } else {
            format!("{prefix}{raw_path}")
        },
    )
}

fn destination_path(root_relative: bool, from: &RelPath, target: &RelPath) -> String {
    let path = if root_relative {
        format!("/{target}")
    } else {
        crate::nav::relative_path(target, from)
    };
    encode_destination(&path)
}

fn phrase_uses(ctx: &Ctx, key: &str) -> usize {
    ctx.snapshot
        .files()
        .flat_map(|file| file.phrases.iter())
        .filter(|phrase| phrase.phrase.key == key)
        .count()
}

fn phrase_value_span(source: &str, key: &str) -> Option<Span> {
    let mut phrases = false;
    let mut at = 0;
    for line in source.split_inclusive('\n') {
        let start = at;
        at += line.len();
        let trimmed = line.trim();
        if trimmed.starts_with('[') && !trimmed.starts_with("[[") {
            phrases = trimmed == "[phrases]";
            continue;
        }
        if !phrases || trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let Some((name, _)) = trimmed.split_once('=') else {
            continue;
        };
        if name.trim().trim_matches(['"', '\'']) != key {
            continue;
        }
        let equals = line.find('=')? + 1;
        let lead = line[equals..].len() - line[equals..].trim_start().len();
        let value_start = equals + lead;
        let value = line.get(value_start..)?;
        if value.starts_with("\"\"\"") || value.starts_with("'''") {
            return None;
        }
        let quote = *value.as_bytes().first()?;
        let end = match quote {
            b'"' => {
                let mut escaped = false;
                let mut end = None;
                for (offset, byte) in value.as_bytes().iter().copied().enumerate().skip(1) {
                    if byte == b'"' && !escaped {
                        end = Some(offset + 1);
                        break;
                    }
                    escaped = byte == b'\\' && !escaped;
                    if byte != b'\\' {
                        escaped = false;
                    }
                }
                end?
            }
            b'\'' => value[1..].find('\'')? + 2,
            _ => return None,
        };
        let rest = value.get(end..)?.trim();
        if !rest.is_empty() && !rest.starts_with('#') {
            return None;
        }
        return Some(Span::new(start + value_start, start + value_start + end));
    }
    None
}

fn remap(path: &RelPath, mappings: &[(RelPath, RelPath)]) -> RelPath {
    for (old, new) in mappings {
        let old_text = old.as_str();
        let path_text = path.as_str();
        if path_text == old_text {
            return new.clone();
        }
        if let Some(rest) = path_text
            .strip_prefix(old_text)
            .and_then(|rest| rest.strip_prefix('/'))
            && let Ok(path) = RelPath::parse(&format!("{new}/{rest}"))
        {
            return path;
        }
    }
    path.clone()
}

#[allow(clippy::mutable_key_type)]
fn workspace_edit(ctx: &Ctx, mut edits: HashMap<RelPath, Vec<ByteEdit>>) -> WorkspaceEdit {
    let mut lines = Lines::new(ctx);
    let changes = edits
        .drain()
        .filter_map(|(path, mut file_edits)| {
            file_edits.sort_by_key(|edit| (edit.span.start(), edit.span.end()));
            file_edits.dedup_by(|a, b| a.span == b.span && a.new_text == b.new_text);
            let mut end = 0;
            if file_edits.iter().any(|edit| {
                let overlap = edit.span.start() < end;
                end = edit.span.end();
                overlap
            }) {
                return None;
            }
            let uri = ctx.uri_of(&path)?;
            let converted = file_edits
                .into_iter()
                .map(|edit| {
                    Some(TextEdit {
                        range: lines.range(&path, edit.span)?,
                        new_text: edit.new_text,
                    })
                })
                .collect::<Option<Vec<_>>>()?;
            Some((uri, converted))
        })
        .collect();
    WorkspaceEdit {
        changes: Some(changes),
        ..WorkspaceEdit::default()
    }
}

fn valid_id_or_phrase(name: &str) -> bool {
    !name.is_empty()
        && name
            .chars()
            .all(|c| c.is_alphanumeric() || matches!(c, '-' | '_' | '.'))
}

fn valid_phrase_key(name: &str) -> bool {
    let mut chars = name.chars();
    chars.next().is_some_and(|c| c.is_ascii_lowercase())
        && chars.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
}

fn heading_text_at(blocks: &[Block], offset: usize, source: &str) -> Option<(Span, Span)> {
    for block in blocks {
        match &block.kind {
            BlockKind::Heading(heading) if block.span.contains(offset) => {
                if heading.inlines.is_empty()
                    || heading
                        .inlines
                        .iter()
                        .any(|inline| !matches!(inline.kind, InlineKind::Text(_)))
                {
                    return None;
                }
                let span = Span::new(
                    heading.inlines.first()?.span.start(),
                    heading.inlines.last()?.span.end(),
                );
                source.get(span.range())?;
                return Some((block.span, span));
            }
            BlockKind::BlockQuote(quote) => {
                if let Some(found) = heading_text_at(&quote.children, offset, source) {
                    return Some(found);
                }
            }
            BlockKind::List(list) => {
                for item in &list.items {
                    if let Some(found) = heading_text_at(&item.children, offset, source) {
                        return Some(found);
                    }
                }
            }
            BlockKind::Container(container) => {
                if let Some(found) = heading_text_at(&container.children, offset, source) {
                    return Some(found);
                }
            }
            BlockKind::Group(group) => {
                for arm in &group.arms {
                    if let Some(found) = heading_text_at(&arm.children, offset, source) {
                        return Some(found);
                    }
                }
            }
            _ => {}
        }
    }
    None
}

fn phrase_entry_at(source: &str, offset: usize) -> Option<(String, Span)> {
    let span = phrase_entry_span_at(source, offset)?;
    Some((
        source
            .get(span.range())?
            .trim_matches(['"', '\''])
            .to_owned(),
        span,
    ))
}

fn phrase_entry_span(source: &str, key: &str) -> Option<Span> {
    let mut phrases = false;
    let mut at = 0;
    for line in source.split_inclusive('\n') {
        let start = at;
        at += line.len();
        let trimmed = line.trim();
        if trimmed.starts_with('[') && !trimmed.starts_with("[[") {
            phrases = trimmed == "[phrases]";
            continue;
        }
        if !phrases || trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let Some((name, _)) = trimmed.split_once('=') else {
            continue;
        };
        if name.trim().trim_matches(['"', '\'']) == key {
            let lead = line.len() - line.trim_start().len();
            let before_eq = line.find('=')?;
            let raw_name = line[lead..before_eq].trim_end();
            return Some(Span::new(start + lead, start + lead + raw_name.len()));
        }
    }
    None
}

fn phrase_entry_span_at(source: &str, offset: usize) -> Option<Span> {
    let mut at = 0;
    let mut phrases = false;
    for line in source.split_inclusive('\n') {
        let start = at;
        at += line.len();
        let trimmed = line.trim();
        if trimmed.starts_with('[') && !trimmed.starts_with("[[") {
            phrases = trimmed == "[phrases]";
            continue;
        }
        if !phrases || trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let Some((name, _)) = trimmed.split_once('=') else {
            continue;
        };
        let lead = line.len() - line.trim_start().len();
        let before_eq = line.find('=')?;
        let raw_name = line[lead..before_eq].trim_end();
        let span = Span::new(start + lead, start + lead + raw_name.len());
        if span.start() <= offset
            && offset <= span.end()
            && start <= offset
            && offset <= at
            && !name.trim().is_empty()
        {
            return Some(span);
        }
    }
    None
}
