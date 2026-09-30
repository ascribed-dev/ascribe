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
          .catch(() => []),
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
          .catch(() => new vscode.WorkspaceEdit()),
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

interface ProtocolWorkspaceEdit {
  changes?: Record<string, ProtocolTextEdit[]>;
}

function asTextEdits(value: unknown): vscode.TextEdit[] {
  if (!Array.isArray(value)) return [];
  return value.flatMap((edit: ProtocolTextEdit) =>
    edit?.range?.start && edit.range.end && typeof edit.newText === "string"
      ? [vscode.TextEdit.replace(protocolRange(edit.range), edit.newText)]
      : [],
  );
}

function asWorkspaceEdit(value: unknown): vscode.WorkspaceEdit {
  const result = new vscode.WorkspaceEdit();
  const changes = (value as ProtocolWorkspaceEdit | null)?.changes;
  if (!changes || typeof changes !== "object") return result;
  for (const [uri, edits] of Object.entries(changes)) {
    for (const edit of edits) {
      if (edit?.range?.start && edit.range.end && typeof edit.newText === "string") {
        result.replace(vscode.Uri.parse(uri), protocolRange(edit.range), edit.newText);
      }
    }
  }
  return result;
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
