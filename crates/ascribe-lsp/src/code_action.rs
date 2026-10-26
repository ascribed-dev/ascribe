//! Quick fixes for diagnostics and safe Ascribe-specific repairs.

use std::collections::HashMap;

use ascribe_core::{FileId, LineIndex, RelPath, Span};
use ascribe_model::{
    ADDRESSES_FILE, AddressFile, Current, NAV_FILE, PERMALINKS_FILE, PermalinkList,
    RETIRED_PLACEHOLDER, permalink_key,
};
use ascribe_resolve::{DeclaredPermalink, FileSystem};
use ascribe_syntax::{BlockKind, Bound};
use lsp_types::{
    CodeAction, CodeActionKind, CodeActionOrCommand, CodeActionParams, CreateFile,
    CreateFileOptions, Diagnostic, DocumentChangeOperation, DocumentChanges, OneOf,
    OptionalVersionedTextDocumentIdentifier, Range, ResourceOp, TextDocumentEdit, TextEdit,
    WorkspaceEdit,
};

use crate::intended;
use crate::nav::{Ctx, directive_at};
use crate::position::Encoding;

// Longer than the lint allows from before it was on. Split it only while
// changing it for another reason.
#[allow(clippy::too_many_lines)]
pub(crate) fn actions(
    ctx: &Ctx,
    params: CodeActionParams,
    published: &(Vec<Diagnostic>, Option<i32>),
) -> Vec<CodeActionOrCommand> {
    let Some(file) = ctx.file() else {
        return model_actions(ctx, params);
    };
    let index = LineIndex::new(&file.source);
    let start = ctx
        .encoding
        .offset_lenient(&index, &file.source, params.range.start);
    let end = ctx
        .encoding
        .offset_lenient(&index, &file.source, params.range.end);
    let mut actions = Vec::new();
    if asks_for(params.context.only.as_deref(), FIX_ALL_KIND)
        && let Some(fix_all) = fix_all(ctx, file, published)
    {
        actions.push(fix_all);
    }
    // The advice `ascribe.baseline` holds back isn't a diagnostic the
    // editor sends back: its quick fix is offered where it is.
    if asks_for(
        params.context.only.as_deref(),
        CodeActionKind::QUICKFIX.as_str(),
    ) {
        actions.extend(crate::baseline::actions(ctx, params.range));
    }

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
        actions.extend(fixes_elsewhere(&diagnostic));

        let at = ctx
            .encoding
            .offset_lenient(&index, &file.source, diagnostic.range.start);
        let diagnostic_end =
            ctx.encoding
                .offset_lenient(&index, &file.source, diagnostic.range.end);
        if diagnostic_end < start || at > end {
            continue;
        }
        // A review check's problem may be fine as it is: the author says so
        // with an acknowledgement, and writes why.
        let place = ascribe_check::Registry::global()
            .entries()
            .find(|e| e.slug.as_str() == slug)
            .and_then(|e| e.place);
        if let Some(place) = place {
            let subject = diagnostic
                .data
                .as_ref()
                .and_then(|data| data.get("subject"))
                .and_then(serde_json::Value::as_str);
            match intended::mark(
                place,
                slug,
                &file.source,
                &file.document,
                at,
                subject,
                &ctx.model_text,
            ) {
                Some(intended::Mark::Source(edit)) => actions.push(action(
                    ctx,
                    intended::TITLE,
                    vec![diagnostic.clone()],
                    vec![text_edit(ctx, &file.source, edit)],
                )),
                Some(intended::Mark::Model(edit)) => {
                    let model_index = LineIndex::new(&ctx.model_text);
                    actions.push(workspace_action(
                        ctx,
                        intended::TITLE,
                        vec![diagnostic.clone()],
                        Vec::new(),
                        Some(TextEdit {
                            range: ctx.encoding.range(&model_index, edit.span),
                            new_text: edit.new_text,
                        }),
                    ));
                }
                None => {}
            }
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
                            ascribe_core::TextEdit::delete(span),
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
                            ascribe_core::TextEdit::insert(use_.phrase.span.start(), "\\"),
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
            "example-stale" | "example-unobserved" => {
                if let Some(copy) = crate::examples::copy_command(ctx, file, at, diagnostic.clone())
                {
                    actions.push(copy);
                }
            }
            "glossary-definition-missing" => {
                if let Some(fix) = definition_fix(ctx, file, at, diagnostic.clone()) {
                    actions.push(fix);
                }
            }
            "permalinks-list-behind" => {
                if let Some(sync) = sync_permalinks(ctx, diagnostic.clone()) {
                    actions.push(sync);
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
        let line_ending = if file.source.contains("\r\n") {
            "\r\n"
        } else {
            "\n"
        };
        let edit = ascribe_core::TextEdit::insert(
            heading.span.end(),
            format!("{line_ending}@id: {}", heading.source_id),
        );
        actions.push(action(
            ctx,
            "Add a stable @id for this heading",
            Vec::new(),
            vec![text_edit(ctx, &file.source, edit)],
        ));
    }
    // A generated file is rewritten the next time its generator runs, so
    // nothing offers to edit it: its memo says where to make the change.
    if ascribe_check::generated::generated_memo(&file.source, &file.document.blocks).is_some()
        && let Some(uri) = ctx.uri_of(&ctx.path)
    {
        actions.retain(|action| !edits(action, &uri));
    }
    actions
}

/// Whether `action` edits the document `uri`.
#[allow(clippy::mutable_key_type)]
fn edits(action: &CodeActionOrCommand, uri: &lsp_types::Uri) -> bool {
    let CodeActionOrCommand::CodeAction(action) = action else {
        return false;
    };
    let Some(edit) = &action.edit else {
        return false;
    };
    let in_changes = edit
        .changes
        .as_ref()
        .is_some_and(|changes| changes.get(uri).is_some_and(|e| !e.is_empty()));
    let in_document_changes = match &edit.document_changes {
        Some(lsp_types::DocumentChanges::Edits(edits)) => {
            edits.iter().any(|e| &e.text_document.uri == uri)
        }
        Some(lsp_types::DocumentChanges::Operations(ops)) => ops.iter().any(|op| {
            matches!(op, lsp_types::DocumentChangeOperation::Edit(e) if &e.text_document.uri == uri)
        }),
        None => false,
    };
    in_changes || in_document_changes
}

/// The kind of the action that applies a page's safe fixes.
pub(crate) const FIX_ALL_KIND: &str = "source.fixAll.ascribe";

/// Its title.
pub(crate) const FIX_ALL_TITLE: &str = "Apply the safe fixes in this page";

/// Whether a request for the kinds `only` (every kind, for none) asks for
/// actions of `kind`: one of them is `kind`, or a kind it's under.
fn asks_for(only: Option<&[CodeActionKind]>, kind: &str) -> bool {
    only.is_none_or(|only| {
        only.iter().any(|asked| {
            kind == asked.as_str()
                || kind
                    .strip_prefix(asked.as_str())
                    .is_some_and(|rest| rest.starts_with('.'))
        })
    })
}

/// "Apply the safe fixes in this page": every safe fix of the problems last
/// published for `file`, in one edit, as [`safe_fixes`] takes them. None
/// when it has no safe fix, or when what was published is for another
/// version of the document than the editor's.
fn fix_all(
    ctx: &Ctx,
    file: &ascribe_resolve::FileIndex,
    (published, version): &(Vec<Diagnostic>, Option<i32>),
) -> Option<CodeActionOrCommand> {
    if *version != ctx.version {
        return None;
    }
    let (edits, fixed) = safe_fixes(ctx.encoding, &file.source, published)?;
    let CodeActionOrCommand::CodeAction(mut action) = action(ctx, FIX_ALL_TITLE, fixed, edits)
    else {
        return None;
    };
    action.kind = Some(CodeActionKind::new(FIX_ALL_KIND));
    Some(CodeActionOrCommand::CodeAction(action))
}

/// Every safe fix of the problems `published` for a file whose text is
/// `source`, taken as `ascribe check --fix` takes them, in its order (advice
/// last) and with two that edit the same text settled the same way: the
/// first is kept, and the other's problem stays for the next run. The edits,
/// and the problems they fix; `None` when there's no safe fix.
pub(crate) fn safe_fixes(
    encoding: Encoding,
    source: &str,
    published: &[Diagnostic],
) -> Option<(Vec<TextEdit>, Vec<Diagnostic>)> {
    let index = LineIndex::new(source);
    let mut diagnostics: Vec<&Diagnostic> = published.iter().collect();
    diagnostics.sort_by_key(|d| {
        (
            d.severity == Some(lsp_types::DiagnosticSeverity::INFORMATION),
            d.range.start,
            d.range.end,
        )
    });
    let mut settled = ascribe_check::fix::Settled::default();
    let mut fixed: Vec<Diagnostic> = Vec::new();
    for diagnostic in diagnostics {
        let fixes = diagnostic
            .data
            .as_ref()
            .and_then(|data| data.get("fixes"))
            .and_then(serde_json::Value::as_array);
        for fix in fixes.into_iter().flatten() {
            if fix.get("applicability").and_then(serde_json::Value::as_str) != Some("safe") {
                continue;
            }
            let edits: Vec<ascribe_core::TextEdit> = fix
                .get("edits")
                .and_then(serde_json::Value::as_array)
                .into_iter()
                .flatten()
                .filter_map(|edit| serde_json::from_value::<TextEdit>(edit.clone()).ok())
                .map(|edit| {
                    let start = encoding.offset_lenient(&index, source, edit.range.start);
                    let end = encoding.offset_lenient(&index, source, edit.range.end);
                    ascribe_core::TextEdit::replace(start..end.max(start), edit.new_text)
                })
                .collect();
            if !edits.is_empty() && settled.take(source, &edits) && !fixed.contains(diagnostic) {
                fixed.push(diagnostic.clone());
            }
        }
    }
    if settled.edits().is_empty() {
        return None;
    }
    let edits = settled
        .edits()
        .iter()
        .map(|edit| TextEdit {
            range: encoding.range(&index, edit.span),
            new_text: edit.new_text.clone(),
        })
        .collect();
    Some((edits, fixed))
}

fn action(
    ctx: &Ctx,
    title: &str,
    diagnostics: Vec<Diagnostic>,
    edits: Vec<TextEdit>,
) -> CodeActionOrCommand {
    workspace_action(ctx, title, diagnostics, edits, None)
}

/// The command a quick fix carries to select what it wrote, for the author
/// to type over: the document's URI, and the range in it after the edit.
pub(crate) const SELECT_RANGE: &str = "ascribe.selectRange";

/// The quick fix on `glossary-definition-missing`: a paragraph under the
/// term's heading and the directives that bind it, selected, for the author
/// to write the definition in.
fn definition_fix(
    ctx: &Ctx,
    file: &ascribe_resolve::FileIndex,
    at: usize,
    diagnostic: Diagnostic,
) -> Option<CodeActionOrCommand> {
    let glossary = ctx.snapshot.glossary();
    let term = glossary
        .terms
        .iter()
        .find(|t| t.written_in == ctx.path && t.heading.start() <= at && at <= t.heading.end())?;
    let blocks = &file.document.blocks;
    let heading = blocks.iter().position(|b| b.span == term.heading)?;
    let end = blocks[heading + 1..]
        .iter()
        .take_while(
            |b| matches!(&b.kind, BlockKind::Directive(d) if d.binding == Some(Bound::Heading)),
        )
        .last()
        .map_or(blocks[heading].span.end(), |b| b.span.end());
    let source: &str = &file.source;
    let nl = if source.contains("\r\n") {
        "\r\n"
    } else {
        "\n"
    };
    // At the end of the last line's text.
    let end = source.get(..end)?.trim_end_matches(['\n', '\r']).len();
    let text = format!("What {} means, in a sentence or two.", term.term);
    let rest = source.get(end..)?;
    let next = rest
        .strip_prefix("\r\n")
        .or_else(|| rest.strip_prefix('\n'))
        .unwrap_or(rest);
    let after = if next.is_empty() || next.starts_with(['\n', '\r']) {
        ""
    } else {
        nl
    };
    let edit = ascribe_core::TextEdit::insert(end, format!("{nl}{nl}{text}{after}"));
    let mut edited = source.to_owned();
    edited.insert_str(end, &edit.new_text);
    let start = end + 2 * nl.len();
    let selected = ctx.encoding.range(
        &LineIndex::new(&edited),
        Span::new(start, start + text.len()),
    );
    let uri = ctx.uri_of(&ctx.path)?;
    let edit = text_edit(ctx, source, edit);
    Some(CodeActionOrCommand::CodeAction(CodeAction {
        title: format!("Write a definition of {}", term.term),
        kind: Some(CodeActionKind::QUICKFIX),
        diagnostics: Some(vec![diagnostic]),
        edit: Some(WorkspaceEdit {
            changes: Some(HashMap::from([(uri.clone(), vec![edit])])),
            ..WorkspaceEdit::default()
        }),
        command: Some(lsp_types::Command {
            title: "Select the definition".to_owned(),
            command: SELECT_RANGE.to_owned(),
            arguments: Some(vec![
                serde_json::Value::String(uri.to_string()),
                serde_json::to_value(selected).ok()?,
            ]),
        }),
        is_preferred: Some(true),
        disabled: None,
        data: None,
    }))
}

/// The quick fix on `permalinks-list-behind`: `ascribe.permalinks` as
/// `ascribe permalinks sync` writes it for the project as the editor has
/// it, as one edit of the whole file. None when the list can't be read, or
/// is up to date already.
fn sync_permalinks(ctx: &Ctx, diagnostic: Diagnostic) -> Option<CodeActionOrCommand> {
    let path = RelPath::parse(PERMALINKS_FILE).ok()?;
    let text = String::from_utf8(ctx.fs.read_file(&path).ok()?).ok()?;
    let list = PermalinkList::parse(&text, FileId::new(0)).ok()?;
    let declared = ctx.snapshot.project().declared_permalinks();
    let current = Current::new(
        DeclaredPermalink::places(&declared),
        &ctx.snapshot.project().recorded_permalinks(),
    );
    let synced = list.synced(&current).list.to_toml();
    if synced == text {
        return None;
    }
    let uri = crate::uri::path_to_uri(&ctx.config.with_file_name(PERMALINKS_FILE))?;
    let edit = TextEdit {
        range: ctx
            .encoding
            .range(&LineIndex::new(&text), Span::new(0, text.len())),
        new_text: synced,
    };
    Some(CodeActionOrCommand::CodeAction(CodeAction {
        title: format!("Bring {PERMALINKS_FILE} up to date"),
        kind: Some(CodeActionKind::QUICKFIX),
        diagnostics: Some(vec![diagnostic]),
        edit: Some(WorkspaceEdit {
            changes: Some(HashMap::from([(uri, vec![edit])])),
            ..WorkspaceEdit::default()
        }),
        command: None,
        is_preferred: Some(true),
        disabled: None,
        data: None,
    }))
}

/// The title of the quick fix on `permalink-lost`.
pub(crate) const RETIRE_TITLE: &str = "Retire this permalink, and write why";

/// The quick fix on `permalink-lost`, in `ascribe.permalinks`: an entry in
/// `ascribe.addresses`' `[retired]` for the name, `go:<name>`, with a
/// placeholder reason, which it then selects for the author to replace,
/// making the file when there's none. The placeholder is reported until
/// it is, so the fix alone quiets nothing, and it's never the preferred
/// fix.
fn retire_permalink(ctx: &Ctx, diagnostic: Diagnostic) -> Option<CodeActionOrCommand> {
    let path = RelPath::parse(PERMALINKS_FILE).ok()?;
    let text = String::from_utf8(ctx.fs.read_file(&path).ok()?).ok()?;
    let list = PermalinkList::parse(&text, FileId::new(0)).ok()?;
    let at = ctx
        .encoding
        .offset_lenient(&LineIndex::new(&text), &text, diagnostic.range.start);
    let name = list
        .permalinks
        .keys()
        .chain(list.aliases.keys())
        .chain(list.moved.keys())
        .chain(list.retired.keys())
        .find(|name| list.span_of(name).is_some_and(|span| span.start() == at))?;
    let before = RelPath::parse(ADDRESSES_FILE)
        .ok()
        .and_then(|path| ctx.fs.read_file(&path).ok())
        .and_then(|bytes| String::from_utf8(bytes).ok());
    let mut file = match &before {
        Some(text) => AddressFile::parse(text, FileId::new(0)).ok()?,
        None => AddressFile::default(),
    };
    let key = permalink_key(name);
    file.record_retired(&key, RETIRED_PLACEHOLDER);
    let after = file.to_toml();
    let line = format!("{} = \"{RETIRED_PLACEHOLDER}\"", toml_quoted(&key));
    let start = after.find(&line)? + line.len() - RETIRED_PLACEHOLDER.len() - 1;
    let selection = ctx.encoding.range(
        &LineIndex::new(&after),
        Span::new(start, start + RETIRED_PLACEHOLDER.len()),
    );
    let uri = crate::uri::path_to_uri(&ctx.config.with_file_name(ADDRESSES_FILE))?;
    let edit = match &before {
        Some(text) => {
            let edit = file.edit_from(text)?;
            WorkspaceEdit {
                changes: Some(HashMap::from([(
                    uri.clone(),
                    vec![TextEdit {
                        range: ctx.encoding.range(&LineIndex::new(text), edit.span),
                        new_text: edit.new_text,
                    }],
                )])),
                ..WorkspaceEdit::default()
            }
        }
        None => WorkspaceEdit {
            document_changes: Some(DocumentChanges::Operations(vec![
                DocumentChangeOperation::Op(ResourceOp::Create(CreateFile {
                    uri: uri.clone(),
                    options: Some(CreateFileOptions {
                        overwrite: Some(false),
                        ignore_if_exists: Some(false),
                    }),
                    annotation_id: None,
                })),
                DocumentChangeOperation::Edit(TextDocumentEdit {
                    text_document: OptionalVersionedTextDocumentIdentifier {
                        uri: uri.clone(),
                        version: None,
                    },
                    edits: vec![OneOf::Left(TextEdit {
                        range: Range::default(),
                        new_text: after,
                    })],
                }),
            ])),
            ..WorkspaceEdit::default()
        },
    };
    Some(CodeActionOrCommand::CodeAction(CodeAction {
        title: RETIRE_TITLE.to_owned(),
        kind: Some(CodeActionKind::QUICKFIX),
        diagnostics: Some(vec![diagnostic]),
        edit: Some(edit),
        command: Some(lsp_types::Command {
            title: RETIRE_TITLE.to_owned(),
            command: crate::links::OPEN_FILE.to_owned(),
            arguments: Some(vec![
                serde_json::Value::String(uri.as_str().to_owned()),
                serde_json::to_value(selection).ok()?,
            ]),
        }),
        is_preferred: Some(false),
        disabled: None,
        data: None,
    }))
}

/// `text` as `ascribe.addresses` writes a key: a TOML basic string. A
/// permalink's name needs no escapes.
fn toml_quoted(text: &str) -> String {
    format!("\"{text}\"")
}

/// The actions on `ascribe.toml`'s and `ascribe.permalinks`'s diagnostics:
/// the fixes `ascribe.toml`'s carry, marking a problem about a content model entry or an image as intended,
/// in `[[intended]]`, retiring a removed permalink, and bringing the list
/// up to date with an alias or a retired permalink.
fn model_actions(ctx: &Ctx, params: CodeActionParams) -> Vec<CodeActionOrCommand> {
    let model_index = LineIndex::new(&ctx.model_text);
    let mut actions = Vec::new();
    // `ascribe.nav`'s problems carry their fixes, which edit the file.
    if crate::uri::uri_to_path(&params.text_document.uri)
        .is_some_and(|p| ascribe_core::path::normalize(&p) == ctx.config.with_file_name(NAV_FILE))
    {
        let uri = params.text_document.uri;
        for diagnostic in params.context.diagnostics {
            actions.extend(fixes_in(&diagnostic, "fixes", |_| Some(uri.clone())));
        }
        return actions;
    }
    // `ascribe.toml`'s problems carry their fixes too, such as the one that
    // deletes a type's copy of a `[frontmatter]` field.
    let in_model = crate::uri::uri_to_path(&params.text_document.uri)
        .is_some_and(|p| ascribe_core::path::normalize(&p) == ctx.config);
    for diagnostic in params.context.diagnostics {
        if in_model {
            let uri = params.text_document.uri.clone();
            actions.extend(fixes_in(&diagnostic, "fixes", |_| Some(uri.clone())));
        }
        let data = diagnostic.data.as_ref();
        let field = |key: &str| {
            data.and_then(|d| d.get(key))
                .and_then(serde_json::Value::as_str)
        };
        match field("slug") {
            Some("permalink-lost") => {
                actions.extend(retire_permalink(ctx, diagnostic.clone()));
                continue;
            }
            Some("permalinks-list-behind") => {
                actions.extend(sync_permalinks(ctx, diagnostic.clone()));
                continue;
            }
            _ => {}
        }
        let (Some(slug), Some(subject)) = (field("slug"), field("subject")) else {
            continue;
        };
        let place = ascribe_check::Registry::global()
            .entries()
            .find(|e| e.slug.as_str() == slug)
            .and_then(|e| e.place);
        if place != Some(ascribe_core::Place::Entry) {
            continue;
        }
        let Some(edit) = intended::entry(slug, subject, &ctx.model_text) else {
            continue;
        };
        actions.push(workspace_action(
            ctx,
            intended::TITLE,
            vec![diagnostic.clone()],
            Vec::new(),
            Some(TextEdit {
                range: ctx.encoding.range(&model_index, edit.span),
                new_text: edit.new_text,
            }),
        ));
    }
    actions
}

/// The fixes of a diagnostic that edit another file, as its data's
/// `elsewhere` carries them: a page's `nav-page-unplaced` is fixed in
/// `ascribe.nav`.
fn fixes_elsewhere(diagnostic: &Diagnostic) -> Vec<CodeActionOrCommand> {
    fixes_in(diagnostic, "elsewhere", |fix| {
        fix.get("uri")
            .and_then(serde_json::Value::as_str)
            .and_then(|uri| uri.parse().ok())
    })
}

/// A quick fix for each fix in a diagnostic's data under `key`, each editing
/// the file `uri` gives for it, safe ones preferred.
#[allow(clippy::mutable_key_type)]
fn fixes_in(
    diagnostic: &Diagnostic,
    key: &str,
    uri: impl Fn(&serde_json::Value) -> Option<lsp_types::Uri>,
) -> Vec<CodeActionOrCommand> {
    let fixes = diagnostic
        .data
        .as_ref()
        .and_then(|data| data.get(key))
        .and_then(serde_json::Value::as_array);
    let mut actions = Vec::new();
    for fix in fixes.into_iter().flatten() {
        let (Some(title), Some(uri)) = (
            fix.get("title").and_then(serde_json::Value::as_str),
            uri(fix),
        ) else {
            continue;
        };
        let edits: Vec<TextEdit> = fix
            .get("edits")
            .and_then(serde_json::Value::as_array)
            .into_iter()
            .flatten()
            .filter_map(|edit| serde_json::from_value(edit.clone()).ok())
            .collect();
        if edits.is_empty() {
            continue;
        }
        let safe = fix.get("applicability").and_then(serde_json::Value::as_str) == Some("safe");
        actions.push(CodeActionOrCommand::CodeAction(CodeAction {
            title: title.to_owned(),
            kind: Some(CodeActionKind::QUICKFIX),
            diagnostics: Some(vec![diagnostic.clone()]),
            edit: Some(WorkspaceEdit {
                changes: Some(HashMap::from([(uri, edits)])),
                ..WorkspaceEdit::default()
            }),
            command: None,
            is_preferred: Some(safe),
            disabled: None,
            data: None,
        }));
    }
    actions
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

fn text_edit(ctx: &Ctx, source: &str, edit: ascribe_core::TextEdit) -> TextEdit {
    TextEdit {
        range: ctx.encoding.range(&LineIndex::new(source), edit.span),
        new_text: edit.new_text,
    }
}

fn container_colon_edit(
    source: &str,
    blocks: &[ascribe_syntax::Block],
    offset: usize,
    slug: &str,
) -> Option<ascribe_core::TextEdit> {
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
        Some(ascribe_core::TextEdit::insert(
            line.span.start() + trim,
            ":",
        ))
    } else {
        Some(ascribe_core::TextEdit::delete(line.colon?))
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
