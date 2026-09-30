import * as vscode from "vscode";
import { ServerController, type ServerState } from "./client.js";
import type { ResolvedBinary } from "./binary.js";
import { PreviewController, type PreviewApi } from "./preview/controller.js";

/** What the extension returns from `activate`, for tests and other extensions. */
export interface AscribeApi {
  /** The binary in use, if one was found. */
  binary(): ResolvedBinary | undefined;
  state(): ServerState;
  /** Settles when the current start or restart is over. */
  whenSettled(): Promise<void>;
  /** The preview, for tests. */
  preview: PreviewApi;
}

let controller: ServerController | undefined;

export async function activate(context: vscode.ExtensionContext): Promise<AscribeApi> {
  const server = new ServerController(context);
  controller = server;

  const preview = new PreviewController(context, server);
  preview.register();

  context.subscriptions.push(
    vscode.workspace.onWillSaveTextDocument((event) => {
      if (
        event.document.languageId !== "markdown" ||
        !vscode.workspace.getConfiguration("ascribe", event.document.uri).get("formatOnSave", false)
      ) {
        return;
      }
      event.waitUntil(
        server
          .request("textDocument/formatting", {
            textDocument: { uri: event.document.uri.toString() },
            options: {
              tabSize: vscode.workspace
                .getConfiguration("editor", event.document.uri)
                .get("tabSize", 2),
              insertSpaces: vscode.workspace
                .getConfiguration("editor", event.document.uri)
                .get("insertSpaces", true),
            },
          })
          .then((value) => asTextEdits(value))
          .catch((error: unknown) => {
            server.reportFeatureError("format on save", error);
            return [];
          }),
      );
    }),
    vscode.workspace.onWillRenameFiles((event) =>
      event.waitUntil(
        server
          .request("workspace/willRenameFiles", {
            files: event.files.map((file) => ({
              oldUri: file.oldUri.toString(),
              newUri: file.newUri.toString(),
            })),
          })
          .then((value) => asWorkspaceEdit(value))
          .catch((error: unknown) => {
            server.reportFeatureError("preparing workspace rename", error);
            throw error;
          }),
      ),
    ),
  );

  context.subscriptions.push(
    server,
    preview,
    vscode.commands.registerCommand("ascribe.restartServer", async () => {
      const present = await hasProject();
      await vscode.commands.executeCommand("setContext", "ascribe.active", present);
      if (present) await server.restart();
      else
        void vscode.window.showInformationMessage("Ascribe: this workspace has no ascribe.toml.");
    }),
    vscode.commands.registerCommand("ascribe.showOutput", () => server.showOutput()),
    vscode.workspace.onDidChangeConfiguration((event) => {
      if (event.affectsConfiguration("ascribe.maxCrashes")) server.readMaxCrashes();
      if (event.affectsConfiguration("ascribe.path")) void server.restart();
    }),
  );

  // A contributed command activates the extension in any workspace, so start
  // the server only where there's a project.
  const found = await hasProject();
  await vscode.commands.executeCommand("setContext", "ascribe.active", found);
  if (found) await server.start();
  return {
    binary: () => server.binary,
    state: () => server.state,
    whenSettled: () => server.whenSettled(),
    preview: preview.api,
  };
}

interface ProtocolPosition {
  line: number;
  character: number;
}

interface ProtocolTextEdit {
  range: { start: ProtocolPosition; end: ProtocolPosition };
  newText: string;
}

function asTextEdits(value: unknown): vscode.TextEdit[] {
  if (!Array.isArray(value)) {
    throw new Error("the language server returned invalid formatting edits");
  }
  return value.map((edit: unknown) => {
    if (!isProtocolTextEdit(edit)) {
      throw new Error("the language server returned an invalid formatting edit");
    }
    return vscode.TextEdit.replace(protocolRange(edit.range), edit.newText);
  });
}

function asWorkspaceEdit(value: unknown): vscode.WorkspaceEdit {
  if (!isRecord(value)) throw new Error("the language server returned an invalid workspace edit");
  const result = new vscode.WorkspaceEdit();
  const changes = value.changes;
  if (changes === undefined) {
    throw new Error("the language server returned a workspace edit without changes");
  }
  if (!isRecord(changes)) throw new Error("the language server returned invalid workspace changes");
  for (const [uri, edits] of Object.entries(changes)) {
    if (!Array.isArray(edits)) {
      throw new Error("the language server returned invalid workspace edits");
    }
    for (const edit of edits) {
      if (!isProtocolTextEdit(edit)) {
        throw new Error("the language server returned an invalid workspace edit");
      }
      result.replace(vscode.Uri.parse(uri), protocolRange(edit.range), edit.newText);
    }
  }
  return result;
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

function isProtocolPosition(value: unknown): value is ProtocolPosition {
  return (
    isRecord(value) &&
    typeof value.line === "number" &&
    Number.isInteger(value.line) &&
    value.line >= 0 &&
    typeof value.character === "number" &&
    Number.isInteger(value.character) &&
    value.character >= 0
  );
}

function isProtocolTextEdit(value: unknown): value is ProtocolTextEdit {
  return (
    isRecord(value) &&
    isRecord(value.range) &&
    isProtocolPosition(value.range.start) &&
    isProtocolPosition(value.range.end) &&
    typeof value.newText === "string"
  );
}

function protocolRange(range: { start: ProtocolPosition; end: ProtocolPosition }): vscode.Range {
  return new vscode.Range(
    range.start.line,
    range.start.character,
    range.end.line,
    range.end.character,
  );
}

/** Whether the workspace holds an `ascribe.toml` (not counting `node_modules`). */
async function hasProject(): Promise<boolean> {
  return (await vscode.workspace.findFiles("**/ascribe.toml", "**/node_modules/**", 1)).length > 0;
}

export async function deactivate(): Promise<void> {
  await controller?.dispose();
  controller = undefined;
}
