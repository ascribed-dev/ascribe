import * as vscode from "vscode";
import { ServerController, type ServerState } from "./client.js";
import type { ResolvedBinary } from "./binary.js";

/** What the extension returns from `activate`, for tests and other extensions. */
export interface TesseraApi {
  /** The binary in use, if one was found. */
  binary(): ResolvedBinary | undefined;
  state(): ServerState;
  /** Settles when the current start or restart is over. */
  whenSettled(): Promise<void>;
}

let controller: ServerController | undefined;

export async function activate(context: vscode.ExtensionContext): Promise<TesseraApi> {
  const server = new ServerController(context);
  controller = server;

  context.subscriptions.push(
    server,
    vscode.commands.registerCommand("tessera.restartServer", async () => {
      const present = await hasProject();
      await vscode.commands.executeCommand("setContext", "tessera.active", present);
      if (present) await server.restart();
      else
        void vscode.window.showInformationMessage("Tessera: this workspace has no tessera.toml.");
    }),
    vscode.commands.registerCommand("tessera.showOutput", () => server.showOutput()),
    vscode.workspace.onDidChangeConfiguration((event) => {
      if (event.affectsConfiguration("tessera.maxCrashes")) server.readMaxCrashes();
      if (event.affectsConfiguration("tessera.path")) void server.restart();
    }),
  );

  // A contributed command activates the extension in any workspace, so start
  // the server only where there's a project.
  const found = await hasProject();
  await vscode.commands.executeCommand("setContext", "tessera.active", found);
  if (found) await server.start();
  return {
    binary: () => server.binary,
    state: () => server.state,
    whenSettled: () => server.whenSettled(),
  };
}

/** Whether the workspace holds a `tessera.toml` (not counting `node_modules`). */
async function hasProject(): Promise<boolean> {
  return (await vscode.workspace.findFiles("**/tessera.toml", "**/node_modules/**", 1)).length > 0;
}

export async function deactivate(): Promise<void> {
  await controller?.dispose();
  controller = undefined;
}
