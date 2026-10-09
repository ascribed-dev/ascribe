//! Project-wide source refactorings and file-operation edits.
//!
//! `textDocument/rename` renames a phrase key (from a `{key}` in a page or
//! its entry in `ascribe.toml`), an `@id`, a heading's text, and a
//! dimension's value (from a `@variant` attribute or the dimension's
//! `values`, in `refactor/values.rs`), each everywhere it's used.
//! `textDocument/prepareRename` says what's renamed at a position, or why
//! nothing is.

mod values;

use std::collections::HashMap;
use std::str::FromStr;

use ascribe_core::path::{normalize, relative_path};
use ascribe_core::{Destination, LineIndex, RelPath, Span, TextEdit as ByteEdit};
use ascribe_resolve::{FileIndex, PhrasePlace, Target};
use ascribe_syntax::{Block, BlockKind, InlineKind};
use lsp_types::{
    FileOperationFilter, FileOperationPattern, FileOperationPatternKind,
    FileOperationRegistrationOptions, Position, PrepareRenameResponse, RenameFilesParams,
    RenameParams, TextEdit, Uri, WorkspaceEdit,
};

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
        let old_abs = normalize(&crate::uri::uri_to_path(&old_uri)?);
        let new_abs = normalize(&crate::uri::uri_to_path(&new_uri)?);
        // Outside FileSystem: whether a rename's target is taken, for any path
        // the editor names.
        if relative_path(root, &old_abs).is_none()
            || relative_path(root, &new_abs).is_none()
            || (new_abs.exists() && new_abs != old_abs)
        {
            return Some(WorkspaceEdit::default());
        }
        let old = relative_path(&ctx.content_dir, &old_abs)?;
        let new = relative_path(&ctx.content_dir, &new_abs)?;
        if !old.is_inside() || !new.is_inside() {
            return Some(WorkspaceEdit::default());
        }
        mappings.push((old, new));
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
            let local = match ascribe_core::classify_destination(&include.written) {
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
        let mut phrase_edits = HashMap::new();
        edits
            .entry(ctx.path.clone())
            .or_insert_with(Vec::new)
            .push(ByteEdit::replace(span, params.new_name.clone()));
        add_id_reference_edits(
            ctx,
            &ctx.path,
            old,
            &params.new_name,
            &mut edits,
            &mut phrase_edits,
        )?;
        return workspace_edit_with_config(ctx, workspace_edit(ctx, edits), phrase_edits);
    }
    if let Some(value) = values::in_page(file, offset) {
        return values::rename(ctx, &value, &params.new_name);
    }
    rename_heading(ctx, offset, &params.new_name)
}

/// What a rename at `position` of a page renames: its range and its text,
/// or why nothing there can be renamed.
pub(crate) fn prepare(ctx: &Ctx, position: Position) -> Result<PrepareRenameResponse, String> {
    let file = ctx.file().ok_or_else(nothing_here)?;
    let index = LineIndex::new(&file.source);
    let offset = ctx.encoding.offset_lenient(&index, &file.source, position);
    let answer = |span: Span, placeholder: &str| PrepareRenameResponse::RangeWithPlaceholder {
        range: ctx.encoding.range(&index, span),
        placeholder: placeholder.to_owned(),
    };
    if let Some(crate::nav::Hit::Phrase(phrase)) = hit_at(file, offset) {
        let key = &phrase.phrase.key;
        if !ctx.model.has_phrase(key) {
            return Err(format!(
                "`{{{key}}}` isn't a phrase: the content model doesn't declare `{key}`."
            ));
        }
        snippet_phrase_edits(ctx, key, key)?;
        return Ok(answer(phrase.phrase.key_span, key));
    }
    if let Some(directive) = directive_at(&file.document.blocks, offset)
        && directive.name == "id"
        && let Some((id, span)) = identifier_primary(directive)
        && span.start() <= offset
        && offset <= span.end()
    {
        return Ok(answer(span, id));
    }
    if let Some(value) = values::in_page(file, offset) {
        if !ctx
            .model
            .dimension_values(&value.dimension)
            .is_some_and(|values| values.iter().any(|v| v.value == value.value))
        {
            return Err(format!(
                "`{}` isn't a value of the dimension `{}`.",
                value.value, value.dimension
            ));
        }
        return Ok(answer(value.span, &value.value));
    }
    if let Some((_, span)) = heading_text_at(&file.document.blocks, offset, &file.source) {
        return Ok(answer(
            span,
            file.source.get(span.range()).unwrap_or_default(),
        ));
    }
    Err(nothing_here())
}

/// What a rename at `position` of `ascribe.toml` renames.
pub(crate) fn prepare_model(
    ctx: &Ctx,
    position: Position,
) -> Result<PrepareRenameResponse, String> {
    let index = LineIndex::new(&ctx.model_text);
    let offset = ctx
        .encoding
        .offset_lenient(&index, &ctx.model_text, position);
    let answer = |span: Span, placeholder: &str| PrepareRenameResponse::RangeWithPlaceholder {
        range: ctx.encoding.range(&index, span),
        placeholder: placeholder.to_owned(),
    };
    if let Some((key, span)) = phrase_entry_at(&ctx.model_text, offset) {
        snippet_phrase_edits(ctx, &key, &key)?;
        return Ok(answer(span, &key));
    }
    if let Some(value) = values::in_model(ctx, offset) {
        return Ok(answer(value.span, &value.value));
    }
    Err(
        "Put the cursor on a phrase's key under `[phrases]`, or a value in a dimension's `values`, to rename it."
            .to_owned(),
    )
}

fn nothing_here() -> String {
    "Put the cursor on a phrase, a heading, an `@id`, or a value in a `@variant` attribute to rename it."
        .to_owned()
}

pub(crate) fn rename_model_key(
    ctx: &Ctx,
    position: lsp_types::Position,
    new_name: &str,
) -> Option<WorkspaceEdit> {
    let index = LineIndex::new(&ctx.model_text);
    let offset = ctx
        .encoding
        .offset_lenient(&index, &ctx.model_text, position);
    if let Some((old, span)) = phrase_entry_at(&ctx.model_text, offset) {
        if !valid_phrase_key(new_name) {
            return None;
        }
        return rename_phrase(ctx, &old, new_name, Some(span));
    }
    let value = values::in_model(ctx, offset)?;
    values::rename(ctx, &value, new_name)
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
    let code = snippet_phrase_edits(ctx, old, new).ok()?;
    let mut result = workspace_edit(ctx, edits);
    let mut changes = result.changes.take().unwrap_or_default();
    changes.extend(code);
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

/// The edits that rename the phrase `old` to `new` in the code `@snippet`s
/// with `phrases=true` take, by file: the lines each snippet covers. A
/// source in another repository can't be edited here, so a use there is an
/// error.
#[allow(clippy::mutable_key_type)]
fn snippet_phrase_edits(
    ctx: &Ctx,
    old: &str,
    new: &str,
) -> Result<HashMap<Uri, Vec<TextEdit>>, String> {
    let mut found: HashMap<RelPath, Vec<Span>> = HashMap::new();
    let mut texts: HashMap<RelPath, String> = HashMap::new();
    for file in ctx.snapshot.files() {
        for use_ in file.snippets.iter().filter(|u| u.phrases) {
            let Some(snippet) = ctx.snapshot.snippet_at(&file.path, use_.span) else {
                continue;
            };
            let Some((first, last)) = snippet.lines else {
                continue;
            };
            if !texts.contains_key(&snippet.path) {
                let Some(text) = ascribe_resolve::FileSystem::read_file(&*ctx.fs, &snippet.path)
                    .ok()
                    .and_then(|bytes| String::from_utf8(bytes).ok())
                else {
                    continue;
                };
                texts.insert(snippet.path.clone(), text);
            }
            let text = &texts[&snippet.path];
            let mut at = 0;
            for (n, line) in text.split_inclusive('\n').enumerate() {
                let start = at;
                at += line.len();
                let n = u32::try_from(n + 1).unwrap_or(u32::MAX);
                if n < first || n > last {
                    continue;
                }
                for phrase in ascribe_syntax::code_phrases(line) {
                    if phrase.key != old {
                        continue;
                    }
                    let remote = use_
                        .address
                        .as_ref()
                        .and_then(|a| a.as_ref().ok())
                        .and_then(|a| ctx.model.source(&a.source))
                        .is_some_and(|s| s.git.is_some());
                    if remote {
                        return Err(format!(
                            "`{{{old}}}` is used in `{}`, a copy of code in another repository, which a rename can't change.",
                            snippet.path
                        ));
                    }
                    found
                        .entry(snippet.path.clone())
                        .or_default()
                        .push(Span::new(
                            start + phrase.key_span.start(),
                            start + phrase.key_span.end(),
                        ));
                }
            }
        }
    }
    let root = ctx.config.parent().unwrap_or(std::path::Path::new("."));
    let mut out = HashMap::new();
    for (path, mut spans) in found {
        spans.sort_by_key(|s| (s.start(), s.end()));
        spans.dedup();
        let index = LineIndex::new(&texts[&path]);
        let uri = crate::uri::path_to_uri(&normalize(&root.join(path.as_str())))
            .ok_or_else(|| format!("`{path}` can't be named as a URI."))?;
        out.insert(
            uri,
            spans
                .into_iter()
                .map(|span| TextEdit {
                    range: ctx.encoding.range(&index, span),
                    new_text: new.to_owned(),
                })
                .collect(),
        );
    }
    Ok(out)
}

fn rename_heading(ctx: &Ctx, offset: usize, new_text: &str) -> Option<WorkspaceEdit> {
    let file = ctx.file()?;
    let (heading_span, span) = heading_text_at(&file.document.blocks, offset, &file.source)?;
    let old_heading = file.headings.iter().position(|h| h.span == heading_span)?;
    let edit = ByteEdit::replace(span, new_text.to_owned());
    let updated = ascribe_core::apply_edits(&file.source, std::slice::from_ref(&edit)).ok()?;
    let slugger = ascribe_resolve::slug::slugger_by_name(&ctx.model.consumer.slugger)
        .unwrap_or_else(ascribe_resolve::slug::default_slugger);
    let indexed = ascribe_resolve::index_file(
        file.file,
        &file.path,
        &updated,
        &ctx.model,
        slugger.as_ref(),
    );
    if indexed.headings.len() != file.headings.len() {
        return None;
    }
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
    let mut phrase_edits = HashMap::new();
    let mut id_changes = HashMap::new();
    for (old, new) in file.headings.iter().zip(&indexed.headings) {
        if old.source_id != new.source_id
            && id_changes
                .insert(old.source_id.clone(), new.source_id.clone())
                .is_some_and(|previous| previous != new.source_id)
        {
            return None;
        }
    }
    for (old_id, new_id) in id_changes {
        add_id_reference_edits(
            ctx,
            &ctx.path,
            &old_id,
            &new_id,
            &mut edits,
            &mut phrase_edits,
        )?;
    }
    workspace_edit_with_config(ctx, workspace_edit(ctx, edits), phrase_edits)
}

fn add_id_reference_edits(
    ctx: &Ctx,
    target: &RelPath,
    old_id: &str,
    new_id: &str,
    edits: &mut HashMap<RelPath, Vec<ByteEdit>>,
    phrase_edits: &mut HashMap<String, TextEdit>,
) -> Option<()> {
    // A link names the heading through the page it's on: the file itself, or
    // a page that includes it (SPEC §4.2).
    let mut names_heading = HashMap::new();
    let mut links_here = |path: &RelPath| {
        *names_heading.entry(path.clone()).or_insert_with(|| {
            path == target
                || ctx
                    .snapshot
                    .page_heading(path, old_id)
                    .is_some_and(|(written_in, _)| &written_in == target)
        })
    };
    for file in ctx.snapshot.files() {
        for include in &file.includes {
            if include.target.as_ref() != Some(target) || include.section.as_deref() != Some(old_id)
            {
                continue;
            }
            if let Some(span) = include.primary
                && let Some(raw) = file.source.get(span.range())
            {
                let replacement = replace_fragment(raw, new_id);
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
            if local.fragment.as_deref() != Some(old_id)
                || !local.path.as_ref().is_some_and(&mut links_here)
            {
                continue;
            }
            for span in destination_spans(file, reference.destination_span, &reference.destination)
            {
                if let Some(raw) = file.source.get(span.range()) {
                    if let Some(hash) = raw.find('#') {
                        let mut replacement = raw[..=hash].to_owned();
                        replacement.push_str(&encode_destination(new_id));
                        if raw.ends_with('>') && !replacement.ends_with('>') {
                            replacement.push('>');
                        }
                        edits
                            .entry(file.path.clone())
                            .or_default()
                            .push(ByteEdit::replace(span, replacement));
                        continue;
                    }
                    let raw_path_end = raw.find('?').unwrap_or(raw.len());
                    let phrase = file.phrases.iter().find(|phrase| {
                        phrase.place == PhrasePlace::Destination
                            && span.start() <= phrase.phrase.key_span.start()
                            && phrase.phrase.key_span.end() <= span.end()
                            && raw[..raw_path_end] == format!("{{{}}}", phrase.phrase.key)
                    });
                    if let Some(phrase) = phrase
                        && phrase_uses(ctx, &phrase.phrase.key) == 1
                        && let Some(value) = ctx.model.phrase(&phrase.phrase.key)
                        && let Some(updated) = replace_phrase_fragment(value, old_id, new_id)
                        && let Some(model_span) =
                            phrase_value_span(&ctx.model_text, &phrase.phrase.key)
                    {
                        phrase_edits.insert(
                            phrase.phrase.key.clone(),
                            TextEdit {
                                range: ctx
                                    .encoding
                                    .range(&LineIndex::new(&ctx.model_text), model_span),
                                new_text: serde_json::to_string(&updated).ok()?,
                            },
                        );
                        continue;
                    }
                    // The page the link names, which may include `target`.
                    let page = local.path.as_ref().unwrap_or(target);
                    let new_path =
                        destination_path(local.written.starts_with('/'), &file.path, page);
                    let query_start = local.written.find('?').unwrap_or(local.written.len());
                    let mut replacement =
                        format!("{new_path}{}#{new_id}", &local.written[query_start..]);
                    if raw.starts_with('<') && raw.ends_with('>') {
                        replacement = format!("<{replacement}>");
                    }
                    edits
                        .entry(file.path.clone())
                        .or_default()
                        .push(ByteEdit::replace(span, replacement));
                }
            }
        }
    }
    Some(())
}

fn replace_fragment(raw: &str, new_id: &str) -> String {
    if let Some(hash) = raw.find('#') {
        format!("{}#{}", &raw[..hash], encode_destination(new_id))
    } else {
        format!("{raw}#{}", encode_destination(new_id))
    }
}

fn replace_phrase_fragment(value: &str, old_id: &str, new_id: &str) -> Option<String> {
    let Destination::Local(local) = ascribe_core::classify_destination(value) else {
        return None;
    };
    if local.fragment.as_deref() != Some(old_id) {
        return None;
    }
    let hash = value.find('#')?;
    Some(format!("{}#{}", &value[..hash], encode_destination(new_id)))
}

#[allow(clippy::mutable_key_type)]
fn workspace_edit_with_config(
    ctx: &Ctx,
    mut result: WorkspaceEdit,
    phrase_edits: HashMap<String, TextEdit>,
) -> Option<WorkspaceEdit> {
    if phrase_edits.is_empty() {
        return Some(result);
    }
    let uri = crate::uri::path_to_uri(&ctx.config)?;
    let mut changes = result.changes.take().unwrap_or_default();
    changes.insert(uri, phrase_edits.into_values().collect());
    result.changes = Some(changes);
    Some(result)
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
