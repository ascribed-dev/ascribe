//! Project-wide source refactorings and file-operation edits.
//!
//! `textDocument/rename` renames a phrase key (from a `{key}` in a page or
//! its entry in `ascribe.toml`), an `@id`, a heading's text, and a
//! dimension's value (from a `@variant` attribute or the dimension's
//! `values`, in `refactor/values.rs`), each everywhere it's used.
//! `textDocument/prepareRename` says what's renamed at a position, or why
//! nothing is.

pub mod values;

use std::collections::HashMap;
use std::path::Path;
use std::str::FromStr;

use ascribe_core::path::{normalize, relative_path};
use ascribe_core::{Destination, LineIndex, RelPath, Span, TextEdit as ByteEdit};
use ascribe_resolve::{FileIndex, PhrasePlace, Target, Usable};
use ascribe_syntax::{Block, BlockKind, InlineKind};
use lsp_types::{
    FileOperationFilter, FileOperationPattern, FileOperationPatternKind,
    FileOperationRegistrationOptions, Position, PrepareRenameResponse, RenameFilesParams,
    RenameParams, TextEdit, Uri, WorkspaceEdit,
};

use crate::model_file;
use crate::nav::{Ctx, Lines, directive_at, encode_destination, hit_at, identifier_primary};

/// The file operations the server asks to hear of before they're made:
/// the renames of the files and folders whose moves it edits for.
pub fn file_operation_capability() -> FileOperationRegistrationOptions {
    FileOperationRegistrationOptions {
        // A folder too: what's in it moves with it, and `ascribe.nav` names
        // it.
        filters: [
            FileOperationPatternKind::File,
            FileOperationPatternKind::Folder,
        ]
        .into_iter()
        .map(|kind| FileOperationFilter {
            scheme: Some("file".to_owned()),
            pattern: FileOperationPattern {
                glob: "**/*".to_owned(),
                matches: Some(kind),
                options: None,
            },
        })
        .collect(),
    }
}

/// Whether something other than `old` is at `new`, where `old` moves to.
/// On a case-insensitive file system, as macOS's and Windows' are by
/// default, a name that differs from `old`'s only in case finds `old`
/// itself, which doesn't take it (issue 899). That's told by the folder's
/// listing, which has each name as it's written: `old`'s is there, and no
/// entry is named exactly as `new` is.
pub fn taken(old: &Path, new: &Path) -> bool {
    // Outside FileSystem: whether the place a file or folder moves to is
    // taken, by anything at all, for any path the editor or the command
    // names.
    if old == new || !new.exists() {
        return false;
    }
    let (Some(folder), Some(old_name), Some(new_name)) =
        (new.parent(), old.file_name(), new.file_name())
    else {
        return true;
    };
    let case_only = old.parent() == Some(folder)
        && old_name.to_string_lossy().to_lowercase() == new_name.to_string_lossy().to_lowercase();
    if !case_only {
        return true;
    }
    // Outside FileSystem: the names as they're written, to tell `old` from
    // another entry whose name differs from it only in case.
    let Ok(entries) = std::fs::read_dir(folder) else {
        return true;
    };
    let names: Vec<_> = entries.flatten().map(|e| e.file_name()).collect();
    names.iter().any(|n| n == new_name) || !names.iter().any(|n| n == old_name)
}

/// `workspace/willRenameFiles`: the edits a page, a fragment, or a folder
/// moving makes to what names it, and to what it names.
// `Uri` hashes by its text; the map is built once and never mutated through
// a key.
#[allow(clippy::mutable_key_type)]
pub fn will_rename(ctx: &Ctx, params: RenameFilesParams) -> Option<WorkspaceEdit> {
    let mut mappings = Vec::new();
    let root = ctx.config.parent()?;
    for file in params.files {
        let old_uri = Uri::from_str(&file.old_uri).ok()?;
        let new_uri = Uri::from_str(&file.new_uri).ok()?;
        let old_abs = normalize(&crate::uri::uri_to_path(&old_uri)?);
        let new_abs = normalize(&crate::uri::uri_to_path(&new_uri)?);
        if relative_path(root, &old_abs).is_none()
            || relative_path(root, &new_abs).is_none()
            || taken(&old_abs, &new_abs)
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
                    && let Some((home, text)) = phrase_home(ctx, &phrase.phrase.key)
                    && let Some(span) = phrase_value_span(text, &phrase.phrase.key)
                {
                    let old_value_path_end = value.find(['?', '#']).unwrap_or(value.len());
                    let phrase_value = format!("{new_path}{}", &value[old_value_path_end..]);
                    if phrase_value != value {
                        phrase_edits.insert(
                            phrase.phrase.key.clone(),
                            (
                                home,
                                TextEdit {
                                    range: ctx.encoding.range(&LineIndex::new(text), span),
                                    new_text: serde_json::to_string(&phrase_value).ok()?,
                                },
                            ),
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
    let mut result = with_memo_edits(
        workspace_edit(ctx, edits),
        crate::memos::move_edits(ctx, |path| remap(path, &mappings)),
    );
    let mut model_edits: Vec<(Uri, TextEdit)> = phrase_edits.into_values().collect();
    model_edits.extend(model_path_edits(ctx, &mappings)?);
    let changes = result.changes.get_or_insert_with(HashMap::new);
    for (uri, edit) in model_edits {
        changes.entry(uri).or_default().push(edit);
    }
    add_project_file_edits(ctx, root, &mappings, &mut result);
    Some(result)
}

/// Adds to `result` the edits the move makes to the project's own files:
/// `ascribe.nav`, and `ascribe.addresses`.
fn add_project_file_edits(
    ctx: &Ctx,
    root: &Path,
    mappings: &[(RelPath, RelPath)],
    result: &mut WorkspaceEdit,
) {
    add_nav_edit(ctx, root, mappings, result);
    add_addresses_edit(ctx, root, mappings, result);
}

/// Adds to `result` the edit to `ascribe.addresses` that records where each
/// page that moves went (SPEC §7.10), when the project has the file and it
/// reads: a page whose address changes is `[moved]` from its old path to
/// its new one, and a page moved back to an entry's key takes the entry
/// away. A fragment has no address of its own, so its move records nothing.
/// A file that can't be read or edited leaves the move's other edits as
/// they are.
#[allow(clippy::mutable_key_type)]
fn add_addresses_edit(
    ctx: &Ctx,
    root: &Path,
    mappings: &[(RelPath, RelPath)],
    result: &mut WorkspaceEdit,
) {
    let edit = || -> Option<(Uri, TextEdit)> {
        let path = RelPath::parse(ascribe_model::ADDRESSES_FILE).ok()?;
        let bytes = ascribe_resolve::FileSystem::read_file(&*ctx.fs, &path).ok()?;
        let text = String::from_utf8(bytes).ok()?;
        let after = recorded_moves(&text, ctx, mappings)?;
        let edit = crate::navigation::text_edit(&text, &after);
        let uri = crate::uri::path_to_uri(&root.join(ascribe_model::ADDRESSES_FILE))?;
        Some((
            uri,
            TextEdit {
                range: ctx.encoding.range(&LineIndex::new(&text), edit.span),
                new_text: edit.new_text,
            },
        ))
    };
    if let Some((uri, edit)) = edit() {
        result
            .changes
            .get_or_insert_with(HashMap::new)
            .insert(uri, vec![edit]);
    }
}

/// Adds to `result` the edit to `ascribe.addresses` that records an `@id`
/// renamed from `old` to `new` on the requested page (SPEC §7.10), when the
/// project has the file and it reads: `<page>#<old>` is `[moved]` to
/// `<page>#<new>`, as a page's move is recorded. An id in a fragment is
/// published on each page that includes it, none of them its own, so its
/// rename records nothing, as a fragment's move doesn't.
#[allow(clippy::mutable_key_type)]
fn add_id_record(ctx: &Ctx, old: &str, new: &str, result: &mut WorkspaceEdit) {
    let edit = || -> Option<(Uri, TextEdit)> {
        let page = ctx.file()?;
        if page.kind != ascribe_resolve::FileKind::Page {
            return None;
        }
        let root = ctx.config.parent()?;
        let path = RelPath::parse(ascribe_model::ADDRESSES_FILE).ok()?;
        let bytes = ascribe_resolve::FileSystem::read_file(&*ctx.fs, &path).ok()?;
        let text = String::from_utf8(bytes).ok()?;
        let mut file =
            ascribe_model::AddressFile::parse(&text, ascribe_core::FileId::new(0)).ok()?;
        file.record_move(
            &format!("{}#{old}", ctx.path),
            &format!("{}#{new}", ctx.path),
        );
        let edit = file.edit_from(&text)?;
        let uri = crate::uri::path_to_uri(&root.join(ascribe_model::ADDRESSES_FILE))?;
        Some((
            uri,
            TextEdit {
                range: ctx.encoding.range(&LineIndex::new(&text), edit.span),
                new_text: edit.new_text,
            },
        ))
    };
    if let Some((uri, edit)) = edit() {
        result
            .changes
            .get_or_insert_with(HashMap::new)
            .insert(uri, vec![edit]);
    }
}

/// `text`, an `ascribe.addresses`, with each page of the project that
/// `mappings` moves recorded, written as Ascribe writes the file; `None`
/// when it doesn't read, or nothing changes.
fn recorded_moves(text: &str, ctx: &Ctx, mappings: &[(RelPath, RelPath)]) -> Option<String> {
    let mut file = ascribe_model::AddressFile::parse(text, ascribe_core::FileId::new(0)).ok()?;
    let before = file.to_toml();
    for page in ctx.snapshot.pages() {
        let moved = remap(&page.path, mappings);
        if moved != page.path {
            file.record_move(page.path.as_str(), moved.as_str());
        }
    }
    let after = file.to_toml();
    (after != before).then_some(after)
}

/// Adds to `result` the edit to `ascribe.nav` that keeps what it names
/// following the pages and folders that move (`edit/navigation.rs`), when it
/// has one that reads and names any of them.
#[allow(clippy::mutable_key_type)]
fn add_nav_edit(
    ctx: &Ctx,
    root: &Path,
    mappings: &[(RelPath, RelPath)],
    result: &mut WorkspaceEdit,
) {
    let edit = || -> Option<(Uri, TextEdit)> {
        let path = RelPath::parse(ascribe_model::NAV_FILE).ok()?;
        let bytes = ascribe_resolve::FileSystem::read_file(&*ctx.fs, &path).ok()?;
        let text = String::from_utf8(bytes).ok()?;
        let after = crate::navigation::moved(&text, &|path| remap(path, mappings))?;
        let edit = crate::navigation::text_edit(&text, &after);
        let uri = crate::uri::path_to_uri(&root.join(ascribe_model::NAV_FILE))?;
        Some((
            uri,
            TextEdit {
                range: ctx.encoding.range(&LineIndex::new(&text), edit.span),
                new_text: edit.new_text,
            },
        ))
    };
    if let Some((uri, edit)) = edit() {
        result
            .changes
            .get_or_insert_with(HashMap::new)
            .insert(uri, vec![edit]);
    }
}

/// The edits to `ascribe.toml` that keep what it names by path following
/// the files that move: a glossary page in `[glossary] pages`. `None` when
/// they can't be made in place. A moved permalink is an entry of
/// `ascribe.addresses`, which the move's own entry keeps.
fn model_path_edits(ctx: &Ctx, mappings: &[(RelPath, RelPath)]) -> Option<Vec<(Uri, TextEdit)>> {
    model_link_edits(ctx, |written| moved_path(written, mappings))
}

/// The edits to `ascribe.toml` that change what it names by a path from the
/// content root, a glossary page, to what
/// `change` gives for each as it's written, leaving those it gives `None`
/// for, each with its file's URI: `ascribe.toml`'s, or that of the file
/// `[from]` takes it from. `None` when they can't be made in place.
fn model_link_edits(
    ctx: &Ctx,
    mut change: impl FnMut(&str) -> Option<String>,
) -> Option<Vec<(Uri, TextEdit)>> {
    // A file that isn't TOML loads no model, so names no path to follow.
    let Ok(file) = ctx.model_file() else {
        return Some(Vec::new());
    };
    let mut named: Vec<Vec<model_file::Seg>> = Vec::new();
    let pages = model_file::path(&["glossary", "pages"]);
    if let Some(list) = file.get(&pages).and_then(toml_edit::Item::as_array) {
        for i in 0..list.len() {
            let mut at = pages.clone();
            at.push(model_file::Seg::Index(i));
            named.push(at);
        }
    }
    let mut edits = Vec::new();
    for at in named {
        let Some(written) = file.get(&at).and_then(toml_edit::Item::as_str) else {
            continue;
        };
        if let Some(changed) = change(written) {
            edits.push(
                file.replace_value(&at, toml_edit::Value::from(changed))
                    .ok()?,
            );
        }
    }
    if edits.is_empty() {
        return Some(Vec::new());
    }
    let finished = file.finish(edits).ok()?;
    let index = LineIndex::new(&ctx.model_text);
    let uri = crate::uri::path_to_uri(&ctx.config)?;
    let mut out: Vec<(Uri, TextEdit)> = finished
        .edits
        .iter()
        .map(|edit| {
            (
                uri.clone(),
                TextEdit {
                    range: ctx.encoding.range(&index, edit.span),
                    new_text: edit.new_text.clone(),
                },
            )
        })
        .collect();
    for (uri, edits) in finished.taken_changes(ctx.encoding)? {
        out.extend(edits.into_iter().map(|edit| (uri.clone(), edit)));
    }
    Some(out)
}

/// Where the phrase `key` is written, by URI, and that file's text:
/// `ascribe.toml`, or the file `[from]` takes it from (SPEC §7.9).
fn phrase_home<'c>(ctx: &'c Ctx, key: &str) -> Option<(Uri, &'c str)> {
    match ascribe_model::taken_from(&ctx.model_text, &["phrases", key]) {
        None => Some((crate::uri::path_to_uri(&ctx.config)?, &ctx.model_text)),
        Some(taken) => {
            let file = ctx.taken().iter().find(|t| t.file == taken.file)?;
            Some((crate::uri::path_to_uri(&file.path)?, &file.text))
        }
    }
}

/// `written`, a path from the content root with an optional `/` before it
/// and `#id` after it, as it's written once `mappings` move its file; `None`
/// when its file doesn't move.
fn moved_path(written: &str, mappings: &[(RelPath, RelPath)]) -> Option<String> {
    let (path, rest) = written.split_at(written.find(['?', '#']).unwrap_or(written.len()));
    let slash = if path.starts_with('/') { "/" } else { "" };
    let old = RelPath::parse(path.trim_start_matches('/')).ok()?;
    let new = remap(&old, mappings);
    (new != old).then(|| format!("{slash}{new}{rest}"))
}

/// `textDocument/rename` in a page: what's renamed at the position,
/// everywhere it's used.
pub fn rename(ctx: &Ctx, params: RenameParams) -> Option<WorkspaceEdit> {
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
        let memo_edits = crate::memos::id_edits(ctx, file, old, &params.new_name);
        let mut result = workspace_edit_with_config(
            ctx,
            with_memo_edits(workspace_edit(ctx, edits), memo_edits),
            phrase_edits,
        )?;
        add_id_record(ctx, old, &params.new_name, &mut result);
        return Some(result);
    }
    if let Some(value) = values::in_page(file, offset) {
        return values::rename(ctx, &value, &params.new_name);
    }
    rename_heading(ctx, offset, &params.new_name)
}

/// What a rename at `position` of a page renames: its range and its text,
/// or why nothing there can be renamed.
pub fn prepare(ctx: &Ctx, position: Position) -> Result<PrepareRenameResponse, String> {
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
pub fn prepare_model(ctx: &Ctx, position: Position) -> Result<PrepareRenameResponse, String> {
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
        "Put the cursor on a phrase's key under `[phrases]`, a value in a dimension's `values`, or the target in a `[releases.<target>]` header, to rename it."
            .to_owned(),
    )
}

fn nothing_here() -> String {
    "Put the cursor on a phrase, a heading, an `@id`, or a value in a `@variant` attribute to rename it."
        .to_owned()
}

/// `textDocument/rename` in `ascribe.toml`: a phrase's key or a
/// dimension's value at the position, everywhere it's used.
pub fn rename_model_key(
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

/// [`rename_model_key`] in this project only, without the projects of the
/// files `[from]` takes from.
pub fn rename_model_key_here(
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
        return rename_phrase_here(ctx, &old, new_name, Some(span));
    }
    let value = values::in_model(ctx, offset)?;
    values::rename_here(ctx, &value, new_name)
}

/// The edit that renames the phrase `old` to `new`: its key where it's
/// written, and every use, in this project and in the project of the file
/// `[from]` takes it from.
pub fn rename_phrase(
    ctx: &Ctx,
    old: &str,
    new: &str,
    config_key: Option<Span>,
) -> Option<WorkspaceEdit> {
    let edit = rename_phrase_here(ctx, old, new, config_key)?;
    let renaming = crate::rename::Renaming::Phrase {
        key: old.to_owned(),
        new: new.to_owned(),
    };
    crate::rename::in_holders(ctx, &renaming, edit, &mut Default::default()).ok()
}

/// [`rename_phrase`] in this project only.
#[allow(clippy::mutable_key_type)]
pub fn rename_phrase_here(
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
    // The key where it's written: in `ascribe.toml`, at the cursor or found,
    // or in the file `[from]` takes it from.
    let (home, text) = match config_key {
        Some(_) => (
            crate::uri::path_to_uri(&ctx.config)?,
            ctx.model_text.as_str(),
        ),
        None => phrase_home(ctx, old)?,
    };
    let model_span = config_key.or_else(|| phrase_entry_span(text, old))?;
    let model_edit = TextEdit {
        range: ctx.encoding.range(&LineIndex::new(text), model_span),
        new_text: new.to_owned(),
    };
    changes.insert(home, vec![model_edit]);
    // A phrase `take` names is taken by its new name.
    let at = model_file::path(&["phrases", old]);
    if let Ok(file) = model_file::ModelFile::parse(&ctx.model_text)
        && let Some(retake) = file.retake(&at, new)
    {
        let finished = file.finish(vec![retake]).ok()?;
        let index = LineIndex::new(&ctx.model_text);
        let edits = finished.edits.iter().map(|e| TextEdit {
            range: ctx.encoding.range(&index, e.span),
            new_text: e.new_text.clone(),
        });
        changes
            .entry(crate::uri::path_to_uri(&ctx.config)?)
            .or_default()
            .extend(edits);
    }
    result.changes = Some(changes);
    let memo_edits = crate::memos::entry_edits(ctx, ascribe_resolve::EntryKind::Phrase, old, new);
    Some(with_memo_edits(result, memo_edits))
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
    with_id_changes(
        ctx,
        file,
        old_heading,
        ByteEdit::replace(span, new_text.to_owned()),
    )
}

/// What the editor says when a rename at `position` rewords a glossary
/// term's heading that has no `@id`: the term's id is the heading's, so it
/// changes with the words. `None` for any other rename.
pub fn term_id_note(ctx: &Ctx, position: Position, new_text: &str) -> Option<String> {
    let file = ctx.file()?;
    let index = LineIndex::new(&file.source);
    let offset = ctx.encoding.offset_lenient(&index, &file.source, position);
    let (heading_span, span) = heading_text_at(&file.document.blocks, offset, &file.source)?;
    let at = file.headings.iter().position(|h| h.span == heading_span)?;
    let heading = &file.headings[at];
    if heading.explicit_id.is_some() {
        return None;
    }
    ctx.snapshot.glossary().term_at(file.file, heading.span)?;
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
    let new_id = &indexed.headings.get(at)?.source_id;
    (*new_id != heading.source_id).then(|| {
        format!(
            "“{}” is a glossary term, and its heading has no `@id`, so the term's id changes too, from `{}` to `{new_id}`. The links to it are changed. To keep an id when the words change, give the heading an `@id`.",
            heading.text, heading.source_id
        )
    })
}

/// Gives the heading at `heading` of the requested page the `@id` `new_id`,
/// on a line of its own under it, and changes every link and include that
/// names an id it changes: its own, and those of the headings after it whose
/// slugs were numbered after its. `None` when it can't: the heading isn't on
/// a line of its own at the start of the line, or already has an `@id`.
pub fn add_id(ctx: &Ctx, heading: usize, new_id: &str) -> Option<WorkspaceEdit> {
    let file = ctx.file()?;
    let found = file.headings.get(heading)?;
    if found.explicit_id.is_some() {
        return None;
    }
    let source = &*file.source;
    let line_start = source[..found.span.start()]
        .rfind('\n')
        .map_or(0, |at| at + 1);
    if line_start != found.span.start() {
        return None;
    }
    let end = source
        .get(..found.span.end())?
        .trim_end_matches(['\n', '\r'])
        .len();
    let newline = if source.contains("\r\n") {
        "\r\n"
    } else {
        "\n"
    };
    let edit = ByteEdit::replace(Span::new(end, end), format!("{newline}@id: {new_id}"));
    with_id_changes(ctx, file, heading, edit)
}

/// `edit` to the requested page, which changes the id of its heading at
/// `heading`, with every link and include that names an id the edit
/// changes changed to match. `None` when the heading's new id is another
/// heading's, or the edit adds or removes a heading.
fn with_id_changes(
    ctx: &Ctx,
    file: &FileIndex,
    heading: usize,
    edit: ByteEdit,
) -> Option<WorkspaceEdit> {
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
    let new_id = indexed.headings.get(heading)?.source_id.clone();
    let old_id = &file.headings.get(heading)?.source_id;
    if new_id != *old_id
        && file
            .headings
            .iter()
            .enumerate()
            .any(|(i, h)| i != heading && h.source_id == new_id)
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
    let mut memo_edits = Vec::new();
    for (old_id, new_id) in id_changes {
        add_id_reference_edits(
            ctx,
            &ctx.path,
            &old_id,
            &new_id,
            &mut edits,
            &mut phrase_edits,
        )?;
        memo_edits.extend(crate::memos::id_edits(ctx, file, &old_id, &new_id));
    }
    workspace_edit_with_config(
        ctx,
        with_memo_edits(workspace_edit(ctx, edits), memo_edits),
        phrase_edits,
    )
}

fn add_id_reference_edits(
    ctx: &Ctx,
    target: &RelPath,
    old_id: &str,
    new_id: &str,
    edits: &mut HashMap<RelPath, Vec<ByteEdit>>,
    phrase_edits: &mut HashMap<String, (Uri, TextEdit)>,
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
                        && let Some((home, text)) = phrase_home(ctx, &phrase.phrase.key)
                        && let Some(model_span) = phrase_value_span(text, &phrase.phrase.key)
                    {
                        phrase_edits.insert(
                            phrase.phrase.key.clone(),
                            (
                                home,
                                TextEdit {
                                    range: ctx.encoding.range(&LineIndex::new(text), model_span),
                                    new_text: serde_json::to_string(&updated).ok()?,
                                },
                            ),
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

/// `result` with the edits to memo files (SPEC §7.7) that keep their
/// `about` following the rename, each under the memo file's URI.
#[allow(clippy::mutable_key_type)]
fn with_memo_edits(mut result: WorkspaceEdit, memo_edits: Vec<(Uri, TextEdit)>) -> WorkspaceEdit {
    if memo_edits.is_empty() {
        return result;
    }
    let changes = result.changes.get_or_insert_with(HashMap::new);
    for (uri, edit) in memo_edits {
        changes.entry(uri).or_default().push(edit);
    }
    result
}

#[allow(clippy::mutable_key_type)]
fn workspace_edit_with_config(
    ctx: &Ctx,
    mut result: WorkspaceEdit,
    phrase_edits: HashMap<String, (Uri, TextEdit)>,
) -> Option<WorkspaceEdit> {
    if phrase_edits.is_empty() {
        return Some(result);
    }
    let mut changes = result.changes.take().unwrap_or_default();
    // `ascribe.toml`'s edits replace any it had, as they always have.
    if let Some(uri) = crate::uri::path_to_uri(&ctx.config) {
        changes.remove(&uri);
    }
    for (uri, edit) in phrase_edits.into_values() {
        changes.entry(uri).or_default().push(edit);
    }
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
    ctx.snapshot.uses(&Usable::Phrase(key.to_owned())).len()
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

/// Whether `name` can be an id: letters, digits, hyphens, underscores, and
/// periods.
pub fn valid_id_or_phrase(name: &str) -> bool {
    !name.is_empty()
        && name
            .chars()
            .all(|c| c.is_alphanumeric() || matches!(c, '-' | '_' | '.'))
}

/// Whether `name` can be a phrase's key: a lowercase letter, then
/// lowercase letters, digits, and hyphens.
pub fn valid_phrase_key(name: &str) -> bool {
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

/// The span of the key `key` as it's written under `[phrases]` in `source`,
/// the text of `ascribe.toml`, quotes and all.
pub fn phrase_entry_span(source: &str, key: &str) -> Option<Span> {
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
