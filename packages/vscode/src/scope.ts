import * as vscode from "vscode";
import type { Middleware } from "vscode-languageclient/node";

/**
 * Middleware that keeps a server from seeing the files of projects nested
 * inside its own folder. A document selector can only include a folder, so a
 * parent project's selector also matches a nested project's files; these
 * guards drop them before they reach the parent: document sync, every feature
 * request, pushed diagnostics, watched-file events, and renames.
 *
 * `excluded` says whether a file belongs to a nested project.
 */
export function scopeMiddleware(excluded: (uri: vscode.Uri) => boolean): Middleware {
  const skip = (document: vscode.TextDocument) => excluded(document.uri);
  return {
    didOpen: (document, next) => (skip(document) ? Promise.resolve() : next(document)),
    didChange: (event, next) => (skip(event.document) ? Promise.resolve() : next(event)),
    willSave: (event, next) => (skip(event.document) ? Promise.resolve() : next(event)),
    willSaveWaitUntil: (event, next) => (skip(event.document) ? Promise.resolve([]) : next(event)),
    didSave: (document, next) => (skip(document) ? Promise.resolve() : next(document)),
    didClose: (document, next) => (skip(document) ? Promise.resolve() : next(document)),

    provideCompletionItem: (d, p, c, t, next) => (skip(d) ? undefined : next(d, p, c, t)),
    provideHover: (d, p, t, next) => (skip(d) ? undefined : next(d, p, t)),
    provideDefinition: (d, p, t, next) => (skip(d) ? undefined : next(d, p, t)),
    provideSignatureHelp: (d, p, c, t, next) => (skip(d) ? undefined : next(d, p, c, t)),
    provideDocumentHighlights: (d, p, t, next) => (skip(d) ? undefined : next(d, p, t)),
    provideDocumentSymbols: (d, t, next) => (skip(d) ? undefined : next(d, t)),
    provideTypeDefinition: (d, p, t, next) => (skip(d) ? undefined : next(d, p, t)),
    provideImplementation: (d, p, t, next) => (skip(d) ? undefined : next(d, p, t)),
    provideDeclaration: (d, p, t, next) => (skip(d) ? undefined : next(d, p, t)),
    provideDocumentColors: (d, t, next) => (skip(d) ? undefined : next(d, t)),
    provideCodeActions: (d, r, c, t, next) => (skip(d) ? undefined : next(d, r, c, t)),
    provideCodeLenses: (d, t, next) => (skip(d) ? undefined : next(d, t)),
    provideDocumentFormattingEdits: (d, o, t, next) => (skip(d) ? undefined : next(d, o, t)),
    provideDocumentRangeFormattingEdits: (d, r, o, t, next) =>
      skip(d) ? undefined : next(d, r, o, t),
    provideDocumentRangesFormattingEdits: (d, r, o, t, next) =>
      skip(d) ? undefined : next(d, r, o, t),
    provideOnTypeFormattingEdits: (d, p, ch, o, t, next) =>
      skip(d) ? undefined : next(d, p, ch, o, t),
    provideRenameEdits: (d, p, n, t, next) => (skip(d) ? undefined : next(d, p, n, t)),
    prepareRename: (d, p, t, next) => (skip(d) ? undefined : next(d, p, t)),
    provideDocumentLinks: (d, t, next) => (skip(d) ? undefined : next(d, t)),
    provideFoldingRanges: (d, c, t, next) => (skip(d) ? undefined : next(d, c, t)),
    provideSelectionRanges: (d, p, t, next) => (skip(d) ? undefined : next(d, p, t)),
    prepareCallHierarchy: (d, p, t, next) => (skip(d) ? undefined : next(d, p, t)),
    prepareTypeHierarchy: (d, p, t, next) => (skip(d) ? undefined : next(d, p, t)),
    provideLinkedEditingRange: (d, p, t, next) => (skip(d) ? undefined : next(d, p, t)),
    provideInlineValues: (d, v, c, t, next) => (skip(d) ? undefined : next(d, v, c, t)),
    provideInlayHints: (d, r, t, next) => (skip(d) ? undefined : next(d, r, t)),
    provideInlineCompletionItems: (d, p, c, t, next) => (skip(d) ? undefined : next(d, p, c, t)),
    provideDocumentSemanticTokens: (d, t, next) => (skip(d) ? undefined : next(d, t)),
    provideDocumentSemanticTokensEdits: (d, id, t, next) => (skip(d) ? undefined : next(d, id, t)),
    provideDocumentRangeSemanticTokens: (d, r, t, next) => (skip(d) ? undefined : next(d, r, t)),

    handleDiagnostics: (uri, diagnostics, next) => next(uri, excluded(uri) ? [] : diagnostics),
    workspace: {
      didChangeWatchedFile: (event, next) =>
        excluded(vscode.Uri.parse(event.uri)) ? Promise.resolve() : next(event),
      willRenameFiles: (event, next) => {
        const files = event.files.filter((file) => !excluded(file.oldUri));
        if (files.length === 0) return Promise.resolve(undefined);
        if (files.length === event.files.length) return next(event);
        // The client reads `files` and `token`, and calls `waitUntil`.
        return next({
          files,
          token: event.token,
          waitUntil: (value: Thenable<unknown> | vscode.WorkspaceEdit) =>
            event.waitUntil(value as never),
        } as vscode.FileWillRenameEvent);
      },
    },
  };
}
