//! The `ascribe/edit` answers to the edits the Pages view makes to
//! `ascribe.nav` (SPEC §7.8). The edits themselves, and what a page or a
//! folder moving changes in the file, are `ascribe_edit::navigation`'s.

use std::collections::HashMap;

use ascribe_core::{FileId, LineIndex, RelPath};
use ascribe_edit::navigation::{NavChange, change, file_name, follows, text_edit};
use ascribe_model::{NAV_FILE, NavFile};
use ascribe_resolve::Navigation;
use lsp_types::{
    CreateFile, CreateFileOptions, DocumentChangeOperation, DocumentChanges, OneOf,
    OptionalVersionedTextDocumentIdentifier, Range, ResourceOp, TextDocumentEdit, WorkspaceEdit,
};
use serde_json::{Map, Value};

use super::{EditAction, EditResult};
use crate::nav::Ctx;

/// The `ascribe/edit` answer to a change the Pages view asks for: an edit
/// of `ascribe.nav`, or one that creates it with its text when the project
/// has none. The project's navigation with the file as it would be is the
/// one asked for, or the change is refused.
#[allow(clippy::mutable_key_type)]
pub(crate) fn edit(ctx: &Ctx, asked: &NavChange) -> Result<EditResult, String> {
    let path = RelPath::parse(NAV_FILE).map_err(|_| unnamed())?;
    let text = ascribe_resolve::FileSystem::read_file(&*ctx.fs, &path)
        .ok()
        .map(|bytes| String::from_utf8_lossy(&bytes).into_owned());
    let project = ctx.snapshot.project();
    // An unlisted page is in no sidebar (SPEC §7.8): it has no place to take.
    if let NavChange::PlacePage { page } = asked
        && project
            .file(page)
            .is_some_and(|f| f.listing(project.model()) == ascribe_model::Listing::Unlisted)
    {
        return Err(format!(
            "`{}` is `unlisted`, so no sidebar shows it and it has no place to take.",
            file_name(page.as_str())
        ));
    }
    let navigation = project.navigation();
    let after = change(text.as_deref(), &navigation, asked)?;
    let nav = NavFile::parse(&after, FileId::new(0))
        .map_err(|_| "The change would make `ascribe.nav` unreadable.".to_owned())?;
    let made = Navigation::of(project, Some(&nav));
    if !follows(&navigation, &made, asked) {
        return Err("The change wouldn't give the order asked for.".to_owned());
    }
    let root = ctx.config.parent().ok_or_else(unnamed)?;
    let uri = crate::uri::path_to_uri(&root.join(NAV_FILE)).ok_or_else(unnamed)?;
    let edit = match text {
        Some(text) => {
            let edit = text_edit(&text, &after);
            let index = LineIndex::new(&text);
            let mut changes = HashMap::new();
            changes.insert(
                uri,
                vec![lsp_types::TextEdit {
                    range: ctx.encoding.range(&index, edit.span),
                    new_text: edit.new_text,
                }],
            );
            WorkspaceEdit {
                changes: Some(changes),
                document_changes: None,
                change_annotations: None,
            }
        }
        None => WorkspaceEdit {
            changes: None,
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
                    text_document: OptionalVersionedTextDocumentIdentifier { uri, version: None },
                    edits: vec![OneOf::Left(lsp_types::TextEdit {
                        range: Range::default(),
                        new_text: after,
                    })],
                }),
            ])),
            change_annotations: None,
        },
    };
    Ok(EditResult::Edit { edit, select: None })
}

/// The change `args` names for `action`.
pub(crate) fn asked(action: EditAction, args: &Map<String, Value>) -> Result<NavChange, String> {
    let text = |key: &str| match args.get(key) {
        Some(Value::String(s)) => Ok(Some(s.clone())),
        None | Some(Value::Null) => Ok(None),
        Some(_) => Err(format!("`{key}` must be text.")),
    };
    let required = |key: &str| text(key)?.ok_or_else(|| format!("`{key}` is missing."));
    let page = |key: &str| -> Result<Option<RelPath>, String> {
        text(key)?
            .map(|p| {
                RelPath::parse(&p)
                    .ok()
                    .filter(RelPath::is_inside)
                    .ok_or_else(|| format!("`{key}` must be a page's path from the content root."))
            })
            .transpose()
    };
    Ok(match action {
        EditAction::MoveSection => NavChange::MoveSection {
            folder: required("folder")?,
            before: text("before")?,
        },
        EditAction::MovePage => NavChange::MovePage {
            page: page("page")?.ok_or_else(|| "`page` is missing.".to_owned())?,
            before: page("before")?,
        },
        EditAction::PlacePage => NavChange::PlacePage {
            page: page("page")?.ok_or_else(|| "`page` is missing.".to_owned())?,
        },
        _ => NavChange::SetLabel {
            folder: required("folder")?,
            label: text("label")?,
        },
    })
}

fn unnamed() -> String {
    "`ascribe.nav` can't be named by a URI.".to_owned()
}
