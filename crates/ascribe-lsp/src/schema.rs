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

/// Changes to documents.
#[derive(schemars::JsonSchema)]
pub(crate) struct LspWorkspaceEdit {
    /// The edits to each document, by its URI.
    changes: std::collections::BTreeMap<String, Vec<LspTextEdit>>,
}
