import * as path from "node:path";
import * as vscode from "vscode";
import type { ProjectServer } from "../client.js";
import type { ProjectRegistry } from "../registry.js";
import type { TargetBuild, TargetsResult } from "../shapes.js";
import type { ChosenBuilds } from "./chosenBuild.js";
import type { ProjectInfo } from "./describe.js";
import type { BuildLenses } from "./lens.js";

/** How long after the last change to an `ascribe.toml` its builds are asked for again. */
const MODEL_DEBOUNCE_MS = 300;

/**
 * Each running project's builds, from `ascribe/targets`, for the status bar
 * and the Projects view. It asks only a server that is already running, and
 * never starts one: a view that lists projects mustn't start their servers.
 */
export class ProjectBuilds implements vscode.Disposable {
  private readonly known = new Map<ProjectServer, readonly TargetBuild[]>();
  private readonly asking = new Map<ProjectServer, Promise<void>>();
  private readonly again = new Set<ProjectServer>();
  private readonly timers = new Map<ProjectServer, ReturnType<typeof setTimeout>>();
  private readonly changed = new vscode.EventEmitter<void>();
  private readonly disposables: vscode.Disposable[] = [];

  constructor(
    private readonly projects: ProjectRegistry,
    private readonly chosen: ChosenBuilds,
    private readonly lenses: BuildLenses,
  ) {}

  /**
   * Fires when what `info` says may have changed: a project, a state, its
   * builds, the choice, or the lens.
   */
  get onDidChange(): vscode.Event<void> {
    return this.changed.event;
  }

  register(): void {
    this.disposables.push(
      this.projects.onDidChangeProjects(() => {
        for (const server of this.known.keys()) {
          if (!this.projects.servers.includes(server)) this.known.delete(server);
        }
        this.changed.fire();
      }),
      this.projects.onDidChangeState((server) => {
        if (server.state === "running") void this.refresh(server);
        else this.known.delete(server);
        this.changed.fire();
      }),
      this.chosen.onDidChange(() => this.changed.fire()),
      this.lenses.onDidChange(() => this.changed.fire()),
      // The server reads an open `ascribe.toml` as it's edited.
      vscode.workspace.onDidChangeTextDocument(({ document }) => {
        if (path.basename(document.uri.fsPath) !== "ascribe.toml") return;
        const server = this.projects.serverFor(document.uri);
        if (server) this.refreshSoon(server);
      }),
      vscode.workspace.onDidSaveTextDocument((document) => {
        if (path.basename(document.uri.fsPath) !== "ascribe.toml") return;
        const server = this.projects.serverFor(document.uri);
        if (server) this.refreshSoon(server);
      }),
    );
    for (const server of this.projects.servers) {
      if (server.state === "running") void this.refresh(server);
    }
  }

  /** What the status bar and the Projects view say about a project. */
  info(server: ProjectServer): ProjectInfo {
    const builds = this.known.get(server) ?? [];
    return {
      ...server.project,
      name: this.projects.name(server.project),
      state: server.state,
      binary: server.binary,
      builds,
      build: this.chosen.shown(server.project.folder, builds),
      lens: this.lenses.isOn(server.project.folder),
    };
  }

  /** A project's builds, once its server has said; empty before. */
  builds(server: ProjectServer): readonly TargetBuild[] {
    return this.known.get(server) ?? [];
  }

  /** Asks a running server for its builds again. A server that isn't running is left alone. */
  refresh(server: ProjectServer): Promise<void> {
    const pending = this.asking.get(server);
    if (pending) {
      this.again.add(server);
      return pending;
    }
    const asking = (async () => {
      try {
        do {
          this.again.delete(server);
          await this.ask(server);
        } while (this.again.has(server));
      } finally {
        this.asking.delete(server);
      }
    })();
    this.asking.set(server, asking);
    return asking;
  }

  /** Settles when every request under way has been answered. */
  async whenSettled(): Promise<void> {
    await Promise.all(this.asking.values());
  }

  dispose(): void {
    for (const timer of this.timers.values()) clearTimeout(timer);
    this.timers.clear();
    for (const disposable of this.disposables.splice(0)) disposable.dispose();
    this.changed.dispose();
  }

  private refreshSoon(server: ProjectServer): void {
    clearTimeout(this.timers.get(server));
    this.timers.set(
      server,
      setTimeout(() => {
        this.timers.delete(server);
        void this.refresh(server);
      }, MODEL_DEBOUNCE_MS),
    );
  }

  /** Asks for the builds through the project's `ascribe.toml`: `ascribe/targets` answers for any of its files. */
  private async ask(server: ProjectServer): Promise<void> {
    if (server.state !== "running") return;
    let result: TargetsResult;
    try {
      result = (await server.request("ascribe/targets", {
        textDocument: { uri: vscode.Uri.file(server.project.config).toString() },
        kinds: ["builds"],
      })) as TargetsResult;
    } catch {
      return;
    }
    if (result.builds === undefined) return;
    if (!this.projects.servers.includes(server)) return;
    this.known.set(server, result.builds);
    // A build the content model no longer has: back to the editor's.
    const folder = server.project.folder;
    const chosen = this.chosen.get(folder);
    if (chosen !== undefined && !result.builds.some((b) => b.name === chosen)) {
      this.chosen.set(folder, undefined);
    }
    this.changed.fire();
  }
}
