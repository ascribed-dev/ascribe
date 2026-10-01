import * as vscode from "vscode";
import type { ServerState } from "./client.js";
import { ProjectRegistry } from "./registry.js";
import type { ResolvedBinary } from "./binary.js";
import { PreviewController, type PreviewApi } from "./preview/controller.js";

/** What the extension returns from `activate`, for tests and other extensions. */
export interface AscribeApi {
  /**
   * The binary in use, if one was found, for the project in `folder`: the only
   * or first project when it's omitted.
   */
  binary(folder?: string): ResolvedBinary | undefined;
  /** The server's state for the project in `folder` (`stopped` when it has none). */
  state(folder?: string): ServerState;
  /** Settles when every start or restart under way is over. */
  whenSettled(): Promise<void>;
  /** The preview, for tests. */
  preview: PreviewApi;
}

let registry: ProjectRegistry | undefined;

export async function activate(context: vscode.ExtensionContext): Promise<AscribeApi> {
  const projects = new ProjectRegistry(context);
  registry = projects;

  const preview = new PreviewController(context, projects);
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
        projects
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
            projects.current()?.reportFeatureError("format on save", error);
            return [];
          }),
      );
    }),
  );

  context.subscriptions.push(
    projects,
    preview,
    vscode.commands.registerCommand("ascribe.restartServer", async () => {
      await projects.refresh();
      if (projects.projects.length === 0) {
        void vscode.window.showInformationMessage("Ascribe: this workspace has no ascribe.toml.");
        return;
      }
      // Nothing running yet (servers start on demand): start the current project's.
      if (projects.servers.every((server) => server.state === "stopped")) {
        await projects.current()?.start();
      } else {
        await projects.restartRunning();
      }
    }),
    vscode.commands.registerCommand("ascribe.showOutput", () => projects.current()?.showOutput()),
    projects.onDidChangeProjects(() => void updateActive(projects)),
  );

  // A contributed command activates the extension in any workspace, so
  // discover projects here and start servers only where there are some.
  projects.register();
  await projects.refresh();
  await updateActive(projects);
  return {
    binary: (folder) => projects.serverAt(folder)?.binary,
    state: (folder) => projects.serverAt(folder)?.state ?? "stopped",
    whenSettled: () => projects.whenSettled(),
    preview: preview.api,
  };
}

/** Tells VS Code whether the workspace has a project, for the `ascribe.active` conditions. */
function updateActive(projects: ProjectRegistry): Thenable<unknown> {
  return vscode.commands.executeCommand(
    "setContext",
    "ascribe.active",
    projects.projects.length > 0,
  );
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

export async function deactivate(): Promise<void> {
  await registry?.dispose();
  registry = undefined;
}
