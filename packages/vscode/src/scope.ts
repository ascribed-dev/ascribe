import * as vscode from "vscode";
import type { Middleware } from "vscode-languageclient/node";
import { followableScheme } from "./linkSchemes.js";

/** How a server's lenses are scoped: which files it's asked for, and what their titles say. */
export interface LensScope {
  /** Whether the server is asked for a document's lenses. */
  asks(document: vscode.TextDocument): boolean;
  /** A lens's title as the editor shows it in a document. */
  title(document: vscode.TextDocument, title: string): string;
}

/**
 * Middleware that keeps a server from seeing files that aren't its project's.
 * A document selector can only include a folder, so a parent project's
 * selector also matches a nested project's files, and the server's file
 * watcher and rename filter (`**` globs) cover the whole window. These guards
 * drop such files before they reach the server: document sync, every feature
 * request, pushed diagnostics, watched-file events, and renames.
 *
 * `excluded` says whether a file belongs to another project, or to none.
 * `lenses`, when given, decides the lens requests instead: a code file a page
 * shows is never sent to a server, and its lenses are asked for from every
 * server whose registration covers it (`codeLens.ts`). `toldOfChange`, when
 * given, decides the watched-file events instead: a server follows its
 * sources' folders on disk, which may be outside its project (`diskChanges.ts`).
 */
export function scopeMiddleware(
  excluded: (uri: vscode.Uri) => boolean,
  lenses?: LensScope,
  toldOfChange?: (uri: vscode.Uri) => boolean,
): Middleware {
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
    provideCodeLenses: async (d, t, next) => {
      if (lenses ? !lenses.asks(d) : skip(d)) return undefined;
      const found = await next(d, t);
      if (lenses) {
        for (const lens of found ?? []) {
          if (lens.command) lens.command.title = lenses.title(d, lens.command.title);
        }
      }
      return found;
    },
    provideDocumentFormattingEdits: (d, o, t, next) => (skip(d) ? undefined : next(d, o, t)),
    provideDocumentRangeFormattingEdits: (d, r, o, t, next) =>
      skip(d) ? undefined : next(d, r, o, t),
    provideDocumentRangesFormattingEdits: (d, r, o, t, next) =>
      skip(d) ? undefined : next(d, r, o, t),
    provideOnTypeFormattingEdits: (d, p, ch, o, t, next) =>
      skip(d) ? undefined : next(d, p, ch, o, t),
    provideRenameEdits: (d, p, n, t, next) => (skip(d) ? undefined : next(d, p, n, t)),
    prepareRename: (d, p, t, next) => (skip(d) ? undefined : next(d, p, t)),
    // A link with any other scheme from an older server is dropped (`linkSchemes.ts`).
    provideDocumentLinks: async (d, t, next) => {
      if (skip(d)) return undefined;
      const links = await next(d, t);
      return links?.filter((link) => !link.target || followableScheme(link.target.scheme));
    },
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
      didChangeWatchedFile: (event, next) => {
        const uri = vscode.Uri.parse(event.uri);
        return (toldOfChange ? toldOfChange(uri) : !excluded(uri))
          ? next(event)
          : Promise.resolve();
      },
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
