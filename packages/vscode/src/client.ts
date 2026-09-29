import * as path from "node:path";
import * as vscode from "vscode";
import {
  CloseAction,
  ErrorAction,
  LanguageClient,
  State,
  type CloseHandlerResult,
  type ErrorHandler,
  type ErrorHandlerResult,
  type LanguageClientOptions,
  type ServerOptions,
} from "vscode-languageclient/node.js";
import { ancestorsWithin, resolveBinary, type ResolvedBinary } from "./binary.js";
import { CrashCounter } from "./crash.js";
import { nodeEnvironment, shellCommand, usesShell } from "./environment.js";
import { parseVersion } from "./version.js";

/** Where the server is in its life. */
export type ServerState = "stopped" | "starting" | "running" | "failed";

const OPEN_SETTINGS = "Open Settings";
const SHOW_OUTPUT = "Show Output";
const RESTART = "Restart Server";

/**
 * Owns the language client: finds the binary, starts `ascribe lsp`, restarts
 * it on request, and gives up after too many crashes.
 */
export class ServerController implements vscode.Disposable {
  private client: LanguageClient | undefined;
  private current: ResolvedBinary | undefined;
  private status: ServerState = "stopped";
  private readonly output = vscode.window.createOutputChannel("Ascribe");
  private readonly crashes: CrashCounter;
  private starting: Promise<void> = Promise.resolve();
  private readonly started = new vscode.EventEmitter<void>();

  constructor(private readonly context: vscode.ExtensionContext) {
    this.crashes = new CrashCounter(readMaxCrashes());
  }

  /** The binary the running (or last started) server uses. */
  get binary(): ResolvedBinary | undefined {
    return this.current;
  }

  get state(): ServerState {
    return this.status;
  }

  /** Fires each time the server reaches the running state, at the first start and after a restart. */
  get onDidStart(): vscode.Event<void> {
    return this.started.event;
  }

  /**
   * Sends a custom request to the running server (the preview's
   * `ascribe/preview`). Rejects when the server isn't running.
   */
  request(method: string, params: unknown): Promise<unknown> {
    const client = this.client;
    if (!client || this.status !== "running") {
      return Promise.reject(new Error("the Ascribe language server isn't running"));
    }
    return client.sendRequest(method, params);
  }

  /** Settles when the current start or restart has finished, successfully or not. */
  whenSettled(): Promise<void> {
    return this.starting;
  }

  /** Starts the server, unless it's already running. */
  start(): Promise<void> {
    this.starting = this.starting.then(() => this.startNow());
    return this.starting;
  }

  /** Stops the server and starts it again, forgetting earlier crashes. */
  restart(): Promise<void> {
    this.crashes.reset();
    this.starting = this.starting.then(async () => {
      await this.stopNow();
      await this.startNow();
    });
    return this.starting;
  }

  showOutput(): void {
    this.output.show(true);
  }

  readMaxCrashes(): void {
    this.crashes.setLimit(readMaxCrashes());
  }

  async dispose(): Promise<void> {
    await this.starting.catch(() => undefined);
    await this.stopNow();
    this.output.dispose();
    this.started.dispose();
  }

  private async startNow(): Promise<void> {
    if (this.client) return;
    this.status = "starting";

    const resolution = await resolveBinary({
      setting: vscode.workspace.getConfiguration("ascribe").get<string>("path", ""),
      projectRoots: await projectRoots(),
      extensionPath: this.context.extensionPath,
      minVersion: minServerVersion(this.context),
      env: nodeEnvironment,
    });

    if (resolution.kind === "missing") {
      this.status = "failed";
      this.current = undefined;
      const { message, tried } = resolution.error;
      this.output.appendLine(message);
      for (const line of tried) this.output.appendLine(`  tried ${line}`);
      void vscode.window
        .showErrorMessage(`Ascribe: ${message}`, OPEN_SETTINGS, SHOW_OUTPUT)
        .then((choice) => {
          if (choice === OPEN_SETTINGS) {
            void vscode.commands.executeCommand("workbench.action.openSettings", "ascribe.path");
          } else if (choice === SHOW_OUTPUT) {
            this.showOutput();
          }
        });
      return;
    }

    const { binary } = resolution;
    this.current = binary;
    this.output.appendLine(`Using ${binary.source} binary ${binary.path}`);
    if (binary.warning) {
      this.output.appendLine(binary.warning);
      void vscode.window.showWarningMessage(`Ascribe: ${binary.warning}`);
    }

    const client = new LanguageClient("ascribe", "Ascribe", serverOptions(binary), {
      ...clientOptions(this.output),
      errorHandler: this.errorHandler(),
    });
    client.onDidChangeState(({ newState }) => {
      if (newState === State.Running) {
        this.status = "running";
        this.started.fire();
      }
    });
    this.client = client;
    try {
      await client.start();
    } catch (error) {
      this.client = undefined;
      this.status = "failed";
      const message = error instanceof Error ? error.message : String(error);
      this.output.appendLine(`The language server didn't start: ${message}`);
      void vscode.window
        .showErrorMessage(`Ascribe: the language server didn't start: ${message}`, SHOW_OUTPUT)
        .then((choice) => {
          if (choice === SHOW_OUTPUT) this.showOutput();
        });
    }
  }

  private async stopNow(): Promise<void> {
    const client = this.client;
    this.client = undefined;
    this.status = "stopped";
    if (!client) return;
    try {
      await client.dispose();
    } catch (error) {
      // A server that already died can't be stopped politely; there's nothing to do.
      this.output.appendLine(`Stopping the language server: ${String(error)}`);
    }
  }

  private errorHandler(): ErrorHandler {
    return {
      error: (): ErrorHandlerResult => ({ action: ErrorAction.Continue }),
      closed: (): CloseHandlerResult => {
        if (this.crashes.recordCrash()) {
          this.output.appendLine(
            `The language server stopped unexpectedly (crash ${this.crashes.count}); restarting.`,
          );
          return { action: CloseAction.Restart, handled: true };
        }
        this.status = "failed";
        const message =
          `The Ascribe language server crashed ${this.crashes.count} times, so it won't be ` +
          `restarted again. See the output for details, then restart it when you've fixed the cause.`;
        this.output.appendLine(message);
        void vscode.window
          .showErrorMessage(`Ascribe: ${message}`, SHOW_OUTPUT, RESTART)
          .then((choice) => {
            if (choice === SHOW_OUTPUT) this.showOutput();
            else if (choice === RESTART) void this.restart();
          });
        return { action: CloseAction.DoNotRestart, handled: true };
      },
    };
  }
}

function serverOptions(binary: ResolvedBinary): ServerOptions {
  // Standard input and output carry the protocol; the server logs to stderr.
  return {
    command: shellCommand(binary.path),
    args: ["lsp"],
    options: { shell: usesShell(binary.path) },
  };
}

function clientOptions(outputChannel: vscode.OutputChannel): LanguageClientOptions {
  return {
    documentSelector: [
      { scheme: "file", language: "markdown" },
      { scheme: "file", pattern: "**/ascribe.toml" },
    ],
    outputChannel,
    // Resolved Q124: the server asks for the files it wants watched with dynamic
    // registrations (`workspace/didChangeWatchedFiles`), which the client
    // forwards, so files that aren't open are followed too. Watching them
    // here as well would deliver every event twice.
  };
}

function readMaxCrashes(): number {
  const value = vscode.workspace.getConfiguration("ascribe").get<number>("maxCrashes", 5);
  return Number.isInteger(value) && value >= 1 ? value : 5;
}

/** Resolved Q125: the oldest server this extension is written for (`ascribe.minServerVersion` in package.json). */
function minServerVersion(context: vscode.ExtensionContext) {
  const declared = (context.extension.packageJSON as { ascribe?: { minServerVersion?: string } })
    .ascribe?.minServerVersion;
  return parseVersion(declared ?? "") ?? { parts: [0, 0, 0] as const, prerelease: undefined };
}

/**
 * The directories to look in for the project's own binary: each folder that
 * holds an `ascribe.toml`, and its parents up to the workspace folder (a
 * monorepo keeps `node_modules` at the top); then the workspace folders.
 */
async function projectRoots(): Promise<string[]> {
  const roots = new Set<string>();
  const models = await vscode.workspace.findFiles("**/ascribe.toml", "**/node_modules/**", 50);
  for (const model of models) {
    const folder = vscode.workspace.getWorkspaceFolder(model);
    const dir = path.dirname(model.fsPath);
    for (const root of ancestorsWithin(dir, folder?.uri.fsPath ?? dir)) roots.add(root);
  }
  for (const folder of vscode.workspace.workspaceFolders ?? []) roots.add(folder.uri.fsPath);
  return [...roots];
}
