//! Starting a page, and stepping through what it leaves to fill in: the
//! `newPage` and `nextPlaceholder` operations of `ascribe/edit`.
//!
//! `newPage` isn't about a place in a page: it's asked through any of the
//! project's files, and its edit creates the page, with its text, as
//! [`ascribe_templates::start`] gives it. The page is never written over a
//! file. `nextPlaceholder` changes nothing: its answer is an empty edit and
//! the place to select. Both find what's left to fill in as
//! `template-placeholder-left` does ([`ascribe_check::Project::template_places`]).

use std::collections::HashMap;

use ascribe_core::{LineIndex, RelPath, Span};
use lsp_types::{
    CreateFile, CreateFileOptions, DocumentChangeOperation, DocumentChanges, OneOf,
    OptionalVersionedTextDocumentIdentifier, Position, Range, ResourceOp, TextDocumentEdit,
    WorkspaceEdit,
};
use serde_json::{Map, Value};

use super::EditResult;
use crate::nav::Ctx;

/// The page a `newPage` asks for: `type`, `title`, and optionally `folder`,
/// a `file:` URI of a folder in the content root. Without a folder, the page
/// goes in the content root.
pub(crate) fn new_page(ctx: &Ctx, args: &Map<String, Value>) -> Result<EditResult, String> {
    let text_arg = |key: &str| match args.get(key) {
        Some(Value::String(s)) if !s.trim().is_empty() => Ok(s.trim().to_owned()),
        Some(Value::String(_)) | None | Some(Value::Null) => Err(format!("`{key}` is missing.")),
        Some(_) => Err(format!("`{key}` must be text.")),
    };
    let type_name = text_arg("type")?;
    let title = text_arg("title")?;
    if title.contains(['\n', '\r']) {
        return Err("`title` must be one line.".to_owned());
    }
    let folder = match args.get("folder") {
        None | Some(Value::Null) => None,
        Some(Value::String(uri)) => Some(content_folder(ctx, uri)?),
        Some(_) => return Err("`folder` must be a folder's URI.".to_owned()),
    };
    let project = project(ctx);
    let page = ascribe_templates::start(&project, &type_name, &title, folder.as_ref())
        .map_err(|e| sentence(&e.to_string()))?;
    let uri = ctx
        .uri_of(&page.path)
        .ok_or_else(|| format!("`{}` can't be named by a URI.", page.path))?;
    let select = project
        .template_places(&type_name, &page.text)
        .first()
        .map(|span| ctx.encoding.range(&LineIndex::new(&page.text), *span));
    let at_start = Range::new(Position::new(0, 0), Position::new(0, 0));
    let operations = vec![
        DocumentChangeOperation::Op(ResourceOp::Create(CreateFile {
            uri: uri.clone(),
            options: Some(CreateFileOptions {
                overwrite: Some(false),
                ignore_if_exists: Some(false),
            }),
            annotation_id: None,
        })),
        DocumentChangeOperation::Edit(TextDocumentEdit {
            text_document: OptionalVersionedTextDocumentIdentifier { uri, version: None },
            edits: vec![OneOf::Left(lsp_types::TextEdit {
                range: at_start,
                new_text: page.text,
            })],
        }),
    ];
    Ok(EditResult::Edit {
        edit: WorkspaceEdit {
            changes: None,
            document_changes: Some(DocumentChanges::Operations(operations)),
            change_annotations: None,
        },
        select,
    })
}

/// The next place in the page, after `end`, that its template left to fill
/// in, or the first when none is after it.
pub(crate) fn next_placeholder(ctx: &Ctx, source: &str, end: usize) -> Result<EditResult, String> {
    let Some(page_type) = ctx.page_type().filter(|t| t.template.is_some()) else {
        return Err(
            "This page's type has no template, so nothing on it is left to fill in.".to_owned(),
        );
    };
    let places = project(ctx).template_places(&page_type.name, source);
    let Some(next) = places
        .iter()
        .find(|span| span.start() >= end)
        .or_else(|| places.first())
    else {
        return Err("Nothing the template left to fill in is on the page.".to_owned());
    };
    Ok(EditResult::Edit {
        edit: WorkspaceEdit {
            changes: Some(HashMap::new()),
            document_changes: None,
            change_annotations: None,
        },
        select: Some(
            ctx.encoding
                .range(&LineIndex::new(source), Span::new(next.start(), next.end())),
        ),
    })
}

/// The content path of `uri`, a folder in the content root.
pub(crate) fn content_folder(ctx: &Ctx, uri: &str) -> Result<RelPath, String> {
    let outside = || "The folder isn't in the project's content root.".to_owned();
    let uri: lsp_types::Uri = uri.parse().map_err(|_| outside())?;
    let path = crate::uri::uri_to_path(&uri).ok_or_else(outside)?;
    let path = ascribe_core::path::normalize(&path);
    ascribe_core::path::relative_path(&ctx.content_dir, &path)
        .filter(RelPath::is_inside)
        .ok_or_else(outside)
}

/// The project as a page starts in it: its content model and its files,
/// with no sources, which a page's start doesn't read.
pub(crate) fn project(ctx: &Ctx) -> ascribe_check::Project {
    let root = ctx
        .config
        .parent()
        .map(std::path::Path::to_path_buf)
        .unwrap_or_default();
    ascribe_check::Project::from_parts_with_fs(
        root,
        ctx.snapshot.layout().content_root.clone(),
        (*ctx.model).clone(),
        ctx.model_text.clone(),
        Vec::new(),
        ctx.fs.clone(),
    )
}

/// An error's message as a sentence: its first letter a capital, with a
/// full stop.
fn sentence(message: &str) -> String {
    let mut chars = message.chars();
    let Some(first) = chars.next() else {
        return String::new();
    };
    let mut out: String = first.to_uppercase().chain(chars).collect();
    if !out.ends_with('.') {
        out.push('.');
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_message_becomes_a_sentence() {
        assert_eq!(
            sentence("`a.md` is there already"),
            "`a.md` is there already."
        );
        assert_eq!(sentence("no path. give one."), "No path. give one.");
    }
}
