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
} from "vscode-languageclient/node";
import { ancestorsWithin, resolveBinary, type ResolvedBinary } from "./binary.js";
import { CrashCounter } from "./crash.js";
import { nodeEnvironment, shellCommand, usesShell } from "./environment.js";
import { globFolder, type Project } from "./projects.js";
import { scopeMiddleware } from "./scope.js";
import { parseVersion } from "./version.js";

/** Where the server is in its life. */
export type ServerState = "stopped" | "starting" | "running" | "failed";

const OPEN_SETTINGS = "Open Settings";
const SHOW_OUTPUT = "Show Output";
const RESTART = "Restart Server";

/** What a project's server needs to know about the workspace around it. */
export interface ProjectHost {
  /** The name of the project's output channel. */
  channelName(project: Project): string;
  /** Whether a file belongs to some other project than `project`, or to none. */
  ownedElsewhere(project: Project, uri: vscode.Uri): boolean;
}

/**
 * Owns one project's language client: finds the binary, starts `ascribe lsp`
 * rooted at the project's folder, restarts it on request, and gives up after
 * too many crashes.
 */
export class ProjectServer implements vscode.Disposable {
  private client: LanguageClient | undefined;
  private current: ResolvedBinary | undefined;
  private status: ServerState = "stopped";
  private channel: vscode.LogOutputChannel | undefined;
  private readonly crashes: CrashCounter;
  private requested = false;
  private starting: Promise<void> = Promise.resolve();
  private readonly started = new vscode.EventEmitter<void>();

  constructor(
    private readonly context: vscode.ExtensionContext,
    readonly project: Project,
    private readonly host: ProjectHost,
  ) {
    this.crashes = new CrashCounter(readMaxCrashes());
  }

  /** The output channel, created when first needed so its name reflects the workspace then. */
  private get output(): vscode.LogOutputChannel {
    this.channel ??= vscode.window.createOutputChannel(this.host.channelName(this.project), {
      log: true,
    });
    return this.channel;
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

  /**
   * Starts the server, once. Later calls wait for that start; a server that
   * failed isn't tried again until it's restarted.
   */
  start(): Promise<void> {
    if (!this.requested) {
      this.requested = true;
      this.starting = this.starting.then(() => this.startNow());
    }
    return this.starting;
  }

  /** Stops the server and starts it again, forgetting earlier crashes. */
  restart(): Promise<void> {
    this.requested = true;
    this.crashes.reset();
    this.starting = this.starting.then(async () => {
      await this.stopNow();
      await this.startNow();
    });
    return this.starting;
  }

  /** Writes a line to the project's output channel. */
  log(message: string): void {
    this.output.appendLine(message);
  }

  showOutput(): void {
    this.output.show(true);
  }

  reportFeatureError(feature: string, error: unknown): void {
    const message = error instanceof Error ? error.message : String(error);
    const detail = `Ascribe: ${feature} failed: ${message}`;
    this.output.appendLine(detail);
    void vscode.window.showErrorMessage(detail, SHOW_OUTPUT).then((choice) => {
      if (choice === SHOW_OUTPUT) this.showOutput();
    });
  }

  readMaxCrashes(): void {
    this.crashes.setLimit(readMaxCrashes());
  }

  async dispose(): Promise<void> {
    await this.starting.catch(() => undefined);
    await this.stopNow();
    this.channel?.dispose();
    this.started.dispose();
  }

  private async startNow(): Promise<void> {
    if (this.client) return;
    this.status = "starting";

    const resolution = await resolveBinary({
      setting: vscode.workspace.getConfiguration("ascribe").get<string>("path", ""),
      projectRoots: ancestorsWithin(
        this.project.folder,
        vscode.workspace.getWorkspaceFolder(vscode.Uri.file(this.project.folder))?.uri.fsPath ??
          this.project.folder,
      ),
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

    const client = new ProjectClient("ascribe", this.output.name, serverOptions(binary), {
      ...clientOptions(
        this.project,
        this.output,
        (uri) => this.host.ownedElsewhere(this.project, uri),
        (error) => this.reportFeatureError("preparing workspace rename", error),
      ),
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

/**
 * A language client that leaves the server's `workspace/executeCommand`
 * commands to the extension. The client would register each one as a VS Code
 * command, and a second project's server (which offers the same ones) would
 * fail to start with "command already exists".
 *
 * `ascribe.openFile` (`OPEN_FILE` in crates/tessera-lsp/src/links.rs) is the
 * server's only command; the extension registers it in `extension.ts`. A
 * command the server adds needs the same treatment.
 */
class ProjectClient extends LanguageClient {
  override registerFeature(feature: Parameters<LanguageClient["registerFeature"]>[0]): void {
    if (
      "registrationType" in feature &&
      feature.registrationType.method === "workspace/executeCommand"
    ) {
      return;
    }
    super.registerFeature(feature);
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

function clientOptions(
  project: Project,
  outputChannel: vscode.LogOutputChannel,
  ownedElsewhere: (uri: vscode.Uri) => boolean,
  reportRenameError: (error: unknown) => void,
): LanguageClientOptions {
  const scope = scopeMiddleware(ownedElsewhere);
  const folder = globFolder(project.folder);
  const folderUri = vscode.Uri.file(project.folder);
  return {
    // Rooting the server at the project's folder is what makes it load this
    // project: it looks for `ascribe.toml` upward from its workspace folder.
    workspaceFolder: { uri: folderUri, name: path.basename(project.folder), index: 0 },
    documentSelector: [
      { scheme: "file", language: "markdown", pattern: `${folder}/**` },
      { scheme: "file", pattern: `${folder}/ascribe.toml` },
    ],
    outputChannel,
    middleware: {
      ...scope,
      workspace: {
        ...scope.workspace,
        willRenameFiles: (event, next) =>
          (scope.workspace?.willRenameFiles ?? ((e, n) => n(e)))(event, (filtered) =>
            next(filtered).then(undefined, (error: unknown) => {
              reportRenameError(error);
              throw error;
            }),
          ),
      },
    },
    // The server asks for the files it wants watched with dynamic
    // registrations (`workspace/didChangeWatchedFiles`), which the client
    // forwards, so files that aren't open are followed too. Watching them
    // here as well would deliver every event twice.
  };
}

function readMaxCrashes(): number {
  const value = vscode.workspace.getConfiguration("ascribe").get<number>("maxCrashes", 5);
  return Number.isInteger(value) && value >= 1 ? value : 5;
}

/** The oldest server this extension is written for (`ascribe.minServerVersion` in package.json). */
function minServerVersion(context: vscode.ExtensionContext) {
  const declared = (context.extension.packageJSON as { ascribe?: { minServerVersion?: string } })
    .ascribe?.minServerVersion;
  return parseVersion(declared ?? "") ?? { parts: [0, 0, 0] as const, prerelease: undefined };
}
