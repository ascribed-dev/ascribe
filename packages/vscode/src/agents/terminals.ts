import * as path from "node:path";
import * as vscode from "vscode";
import { resolveProjectBinary, workspaceFolderOf } from "../client.js";
import { nodePathEnvironment } from "../environment.js";
import { samePath } from "../projects.js";
import type { ProjectRegistry } from "../registry.js";
import { ascribeElsewhere } from "./reach.js";

/** What the extension returns for tests. */
export interface TerminalsApi {
  /** The folder put first on each workspace folder's terminals' path, by workspace folder. */
  directories(): ReadonlyMap<string, string>;
  /** Settles when the terminals' path has been worked out again. */
  whenSettled(): Promise<void>;
}

/**
 * Puts the binary the editor runs first on the path of the window's
 * terminals, so an agent that runs `ascribe` in one runs the same binary.
 * Each workspace folder gets its first project's binary, as the MCP server
 * does; a folder where `ascribe` is already that binary gets nothing.
 */
export class TerminalPath implements vscode.Disposable {
  private readonly disposables: vscode.Disposable[] = [];
  private readonly applied = new Map<string, string>();
  private updating: Promise<void> = Promise.resolve();

  constructor(
    private readonly context: vscode.ExtensionContext,
    private readonly projects: ProjectRegistry,
  ) {}

  register(): void {
    this.disposables.push(
      this.projects.onDidChangeProjects(() => this.update()),
      // A server starting may have found a binary installed since.
      this.projects.onDidStart(() => this.update()),
      vscode.workspace.onDidChangeWorkspaceFolders(() => this.update()),
      vscode.workspace.onDidChangeConfiguration((event) => {
        if (event.affectsConfiguration("ascribe.path")) this.update();
      }),
    );
    this.update();
  }

  get api(): TerminalsApi {
    return { directories: () => this.applied, whenSettled: () => this.updating };
  }

  /** Works the path out again, after any update under way. */
  private update(): void {
    this.updating = this.updating
      .then(() => this.apply())
      .catch((error: unknown) => {
        this.projects.servers[0]?.log(`Setting the terminals' path failed: ${String(error)}`);
      });
  }

  private async apply(): Promise<void> {
    const wanted = new Map<string, string>();
    for (const folder of vscode.workspace.workspaceFolders ?? []) {
      const server = this.projects.servers.find((s) =>
        samePath(workspaceFolderOf(s.project), folder.uri.fsPath),
      );
      if (!server) continue;
      const resolution = await resolveProjectBinary(this.context, server.project);
      if (resolution.kind !== "found") continue;
      const elsewhere = await ascribeElsewhere(resolution.binary.path, nodePathEnvironment());
      if (elsewhere) wanted.set(folder.uri.toString(), elsewhere.directory);
    }

    const collection = this.context.environmentVariableCollection;
    collection.clear();
    this.applied.clear();
    for (const folder of vscode.workspace.workspaceFolders ?? []) {
      const directory = wanted.get(folder.uri.toString());
      if (directory === undefined) continue;
      const scoped = collection.getScoped({ workspaceFolder: folder });
      scoped.prepend("PATH", directory + path.delimiter);
      scoped.description = new vscode.MarkdownString(
        "Puts the Ascribe binary the editor runs first on the path, so `ascribe` in a terminal runs it.",
      );
      this.applied.set(folder.uri.fsPath, directory);
    }
  }

  dispose(): void {
    for (const disposable of this.disposables.splice(0)) disposable.dispose();
  }
}
