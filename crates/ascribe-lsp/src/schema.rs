//! The JSON Schemas of the LSP types the custom requests answer with, which
//! `lsp-types` doesn't derive. A field of an LSP type names one of these with
//! `schemars(with = …)`; they're never built.

#![allow(dead_code)]

/// A position in a document: a zero-based line, and a zero-based column in
/// the position encoding the client and server agreed on.
#[derive(schemars::JsonSchema)]
pub(crate) struct LspPosition {
    /// The line, from 0.
    line: u32,
    /// The column, from 0, in the negotiated position encoding.
    character: u32,
}

/// A range in a document, from `start` up to (not including) `end`.
#[derive(schemars::JsonSchema)]
pub(crate) struct LspRange {
    /// Where it starts.
    start: LspPosition,
    /// Where it ends.
    end: LspPosition,
}

/// A text edit: replace `range` with `newText`.
#[derive(schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub(crate) struct LspTextEdit {
    /// The range to replace, in the document as it is before the edit.
    range: LspRange,
    /// The text that replaces it.
    new_text: String,
}

/// Changes to documents: `changes`, or, for an edit that creates a file,
/// `documentChanges`.
#[derive(schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub(crate) struct LspWorkspaceEdit {
    /// The edits to each document, by its URI.
    #[serde(skip_serializing_if = "Option::is_none")]
    changes: Option<std::collections::BTreeMap<String, Vec<LspTextEdit>>>,
    /// Files to create and edits to make, in order: each file is created
    /// before the edit that gives it its text.
    #[serde(skip_serializing_if = "Option::is_none")]
    document_changes: Option<Vec<LspDocumentChange>>,
}

/// A file to create, or edits to one document.
#[derive(schemars::JsonSchema)]
#[serde(untagged)]
pub(crate) enum LspDocumentChange {
    /// A file to create.
    Create(LspCreateFile),
    /// Edits to one document.
    Edit(LspTextDocumentEdit),
}

/// A file to create, empty.
#[derive(schemars::JsonSchema)]
pub(crate) struct LspCreateFile {
    /// `create`.
    kind: LspCreateKind,
    /// The file's URI.
    uri: String,
    /// What to do when it's there already.
    #[serde(skip_serializing_if = "Option::is_none")]
    options: Option<LspCreateFileOptions>,
}

/// The kind of a file to create.
#[derive(schemars::JsonSchema)]
pub(crate) enum LspCreateKind {
    /// A file to create.
    #[serde(rename = "create")]
    Create,
}

/// What to do when a file to create is there already.
#[derive(schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub(crate) struct LspCreateFileOptions {
    /// Whether to write over it.
    #[serde(skip_serializing_if = "Option::is_none")]
    overwrite: Option<bool>,
    /// Whether to leave it as it is, and go on.
    #[serde(skip_serializing_if = "Option::is_none")]
    ignore_if_exists: Option<bool>,
}

/// Edits to one document.
#[derive(schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub(crate) struct LspTextDocumentEdit {
    /// The document.
    text_document: LspVersionedDocument,
    /// Its edits, each to the document as it is before any of them.
    edits: Vec<LspTextEdit>,
}

/// A document, and the version an edit is for; `null` for any.
#[derive(schemars::JsonSchema)]
pub(crate) struct LspVersionedDocument {
    /// Its URI.
    uri: String,
    /// The version.
    version: Option<i32>,
}
