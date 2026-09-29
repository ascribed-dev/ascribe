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
    vscode.commands.registerCommand("tessera.restartServer", () => server.restart()),
    vscode.commands.registerCommand("tessera.showOutput", () => server.showOutput()),
    vscode.workspace.onDidChangeConfiguration((event) => {
      if (event.affectsConfiguration("tessera.maxCrashes")) server.readMaxCrashes();
      if (event.affectsConfiguration("tessera.path")) void server.restart();
    }),
  );

  await server.start();
  return {
    binary: () => server.binary,
    state: () => server.state,
    whenSettled: () => server.whenSettled(),
  };
}

export async function deactivate(): Promise<void> {
  await controller?.dispose();
  controller = undefined;
}
