//! The `ascribe/projectEdit` request: one edit for the whole project, which
//! the editor's Project view asks for and applies, unsaved, after saying how
//! much it changes. Two are asked for:
//!
//! - **Every safe fix:** the edit "Apply the safe fixes in this page" makes
//!   (`code_action.rs`), for every source file at once, from the problems
//!   last published for each. Those are the editor's build's; a problem only
//!   every build can tell is `ascribe check --fix`'s alone (decision #544e).
//! - **Every page formatted:** the edit Format Document makes
//!   (`formatting.rs`), for every source file, with `ascribe_fmt::format`,
//!   the code `ascribe fmt` runs.
//!
//! It's worked out from the snapshot, so an open document's unsaved text is
//! what's edited, and each open document's edit names the version it's for:
//! the editor refuses it when the document changed since. The server answers
//! once the diagnostics it has queued are published (`server.rs`), so the
//! problems it fixes are those of the text it edits.

use ascribe_core::{LineIndex, RelPath};
use lsp_types::{
    DocumentChanges, OneOf, OptionalVersionedTextDocumentIdentifier, TextDocumentEdit,
    TextDocumentIdentifier, TextEdit, Uri, WorkspaceEdit,
};
use serde::{Deserialize, Serialize};

use crate::nav::Ctx;

/// The request's method name.
pub const METHOD: &str = "ascribe/projectEdit";

/// The parameters of `ascribe/projectEdit`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectEditParams {
    /// Any of the project's files: a source file, `ascribe.toml`, or another
    /// file in the project's folder.
    pub text_document: TextDocumentIdentifier,
    /// Which edit.
    pub kind: ProjectEditKind,
}

/// Which edit `ascribe/projectEdit` makes.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum ProjectEditKind {
    /// Every safe fix of the problems the editor shows, in every source file.
    FixAll,
    /// Every source file formatted, as `ascribe fmt` formats it.
    Format,
}

/// The answer to `ascribe/projectEdit`: the edit and how much it changes, or
/// why there's none.
#[derive(Debug, Serialize, PartialEq)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[serde(untagged, rename_all_fields = "camelCase")]
pub enum ProjectEditResult {
    /// The edit, which may change nothing.
    Edit {
        /// Edits to each file it changes, under `documentChanges`, in path
        /// order: an open document's names the version it's for, and any
        /// other's `null`.
        #[cfg_attr(
            feature = "json-schema",
            schemars(with = "crate::schema::LspWorkspaceEdit")
        )]
        edit: WorkspaceEdit,
        /// How many files it changes.
        files: usize,
        /// For the safe fixes, how many problems they fix; `null` for
        /// formatting.
        fixes: Option<usize>,
    },
    /// Why there's no edit: a plain-language sentence, for the client to
    /// show.
    Failed {
        /// The message.
        error: String,
    },
}

impl Default for ProjectEditResult {
    fn default() -> ProjectEditResult {
        ProjectEditResult::Failed {
            error: "This file isn't one of the project's.".to_owned(),
        }
    }
}

/// A source file of the project, as the request finds it.
pub(crate) struct Source {
    /// Its content path.
    pub path: RelPath,
    /// Its URI: the open document's, as the client wrote it, or else made
    /// from its path.
    pub uri: Uri,
    /// The open document's version; `None` when it isn't open.
    pub version: Option<i32>,
    /// The diagnostics last published for it, without Vale's, and the
    /// version of the document they're for.
    pub published: (Vec<lsp_types::Diagnostic>, Option<i32>),
}

/// The edit of `kind` for `sources`, the project's source files in path
/// order.
pub(crate) fn project_edit(
    ctx: &Ctx,
    sources: &[Source],
    kind: ProjectEditKind,
) -> ProjectEditResult {
    let options = ascribe_fmt::options_from_model(&ctx.model);
    let mut changes = Vec::new();
    let mut fixes = 0;
    for source in sources {
        let Some(file) = ctx.snapshot.file(&source.path) else {
            continue;
        };
        let edits = match kind {
            ProjectEditKind::FixAll => {
                let (published, version) = &source.published;
                if *version != source.version {
                    continue;
                }
                let Some((edits, fixed)) =
                    crate::code_action::safe_fixes(ctx.encoding, &file.source, published)
                else {
                    continue;
                };
                fixes += fixed.len();
                edits
            }
            ProjectEditKind::Format => {
                let index = LineIndex::new(&file.source);
                ascribe_fmt::format(&file.source, &options, &ctx.model)
                    .into_iter()
                    .map(|edit| TextEdit {
                        range: ctx.encoding.range(&index, edit.span),
                        new_text: edit.new_text,
                    })
                    .collect()
            }
        };
        if edits.is_empty() {
            continue;
        }
        changes.push(TextDocumentEdit {
            text_document: OptionalVersionedTextDocumentIdentifier {
                uri: source.uri.clone(),
                version: source.version,
            },
            edits: edits.into_iter().map(OneOf::Left).collect(),
        });
    }
    ProjectEditResult::Edit {
        files: changes.len(),
        fixes: (kind == ProjectEditKind::FixAll).then_some(fixes),
        edit: WorkspaceEdit {
            document_changes: Some(DocumentChanges::Edits(changes)),
            ..WorkspaceEdit::default()
        },
    }
}
