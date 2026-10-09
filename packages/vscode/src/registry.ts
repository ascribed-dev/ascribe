import { stat } from "node:fs/promises";
import * as path from "node:path";
import * as vscode from "vscode";
import { ProjectServer, type ProjectHost } from "./client.js";
import {
  channelName,
  comparable,
  nestedProjects,
  ownedElsewhere,
  owningProject,
  projectName,
  samePath,
  within,
  type Project,
  type WorkspaceFolder,
} from "./projects.js";

/** `ascribe.startServers`: when a project's server starts. */
type StartServers = "onDemand" | "all";

/** The most projects found in one workspace. */
const MAX_PROJECTS = 50;

/** How long opened files with an unknown `ascribe.toml` are gathered before looking for projects again. */
const LOOK_AGAIN_DELAY_MS = 300;

/**
 * Finds every `ascribe.toml` in the workspace and runs one language server
 * for each. Discovery starts nothing: a server starts the first time one of
 * its files is needed (or at discovery, with `ascribe.startServers: "all"`).
 */
export class ProjectRegistry implements vscode.Disposable, ProjectHost {
  private readonly byConfig = new Map<string, ProjectServer>();
  /** The nested projects each server was started with, to restart it when they change. */
  private readonly nestedAtStart = new Map<ProjectServer, string>();
  private warnedOfCap = false;
  private refreshing: Promise<void> = Promise.resolve();
  /** `ascribe.toml` files found beside opened files, for the next refresh to pick up. */
  private unknownConfigs = new Set<string>();
  /** Those a refresh didn't pick up (under `node_modules`, or past the cap), so they aren't asked for again. */
  private readonly passedOver = new Set<string>();
  private lookAgainTimer: ReturnType<typeof setTimeout> | undefined;
  private readonly projectsChanged = new vscode.EventEmitter<void>();
  private readonly started = new vscode.EventEmitter<ProjectServer>();
  private readonly stateChanged = new vscode.EventEmitter<ProjectServer>();
  private readonly disposables: vscode.Disposable[] = [];

  constructor(private readonly context: vscode.ExtensionContext) {}

  /** The servers, one per project, in path order. */
  get servers(): ProjectServer[] {
    return [...this.byConfig.values()];
  }

  get projects(): Project[] {
    return this.servers.map((server) => server.project);
  }

  /** Fires when a project appears or disappears. */
  get onDidChangeProjects(): vscode.Event<void> {
    return this.projectsChanged.event;
  }

  /** Fires with a server each time it reaches the running state. */
  get onDidStart(): vscode.Event<ProjectServer> {
    return this.started.event;
  }

  /** Fires with a server each time its state changes. */
  get onDidChangeState(): vscode.Event<ProjectServer> {
    return this.stateChanged.event;
  }

  /** Starts following the workspace: its `ascribe.toml` files, open documents, and the settings. */
  register(): void {
    const watcher = vscode.workspace.createFileSystemWatcher("**/ascribe.toml");
    this.disposables.push(
      watcher,
      watcher.onDidCreate(() => void this.refresh()),
      watcher.onDidDelete(() => void this.refresh()),
      vscode.workspace.onDidChangeWorkspaceFolders(() => void this.refresh()),
      vscode.workspace.onDidOpenTextDocument((document) => void this.startFor(document)),
      vscode.window.onDidChangeActiveTextEditor((editor) => {
        if (editor) void this.startFor(editor.document);
      }),
      vscode.workspace.onDidChangeConfiguration((event) => {
        if (event.affectsConfiguration("ascribe.maxCrashes")) {
          for (const server of this.servers) server.readMaxCrashes();
        }
        if (event.affectsConfiguration("ascribe.path")) void this.restartRunning();
        if (event.affectsConfiguration("ascribe.startServers")) void this.startAllIfConfigured();
      }),
    );
  }

  /**
   * Looks for projects again, adding servers for new ones and stopping those
   * of deleted ones, then starts whatever should now be running. Calls queue,
   * and one that fails is logged without stopping the ones after it.
   */
  refresh(): Promise<void> {
    this.refreshing = this.refreshing
      .then(() => this.refreshNow())
      .catch((error: unknown) => {
        const message = `Looking for projects failed: ${error instanceof Error ? (error.stack ?? error.message) : String(error)}`;
        // The extension host's log, and the first project's output.
        console.error(`Ascribe: ${message}`);
        this.servers[0]?.log(message);
      });
    return this.refreshing;
  }

  /** The server of the project that owns a file. */
  serverFor(uri: vscode.Uri): ProjectServer | undefined {
    const owner = owningProject(uri.fsPath, this.projects);
    return owner && this.byConfig.get(owner.config);
  }

  /** The server for a project folder, or the only or first one when no folder is given. */
  serverAt(folder?: string): ProjectServer | undefined {
    if (folder === undefined) return this.servers[0];
    return this.servers.find((server) => samePath(server.project.folder, folder));
  }

  /**
   * The server a command without a file acts on: the one that owns the active
   * editor, else the first running one, else the first.
   */
  current(): ProjectServer | undefined {
    const active = vscode.window.activeTextEditor?.document.uri;
    return (
      (active && this.serverFor(active)) ??
      this.servers.find((server) => server.state === "running") ??
      this.servers[0]
    );
  }

  /** Starts the server that owns a file, if there is one. Resolves to it once it has settled. */
  async ensureStartedFor(uri: vscode.Uri): Promise<ProjectServer | undefined> {
    const server = this.serverFor(uri);
    await server?.start();
    return server;
  }

  /** Sends a request to the current server: for the server's own commands, which name no file. */
  request(method: string, params: unknown): Promise<unknown> {
    const server = this.current();
    if (!server) return Promise.reject(new Error("the Ascribe language server isn't running"));
    return server.request(method, params);
  }

  /** Settles when every server that is starting or restarting has finished. */
  async whenSettled(): Promise<void> {
    await this.refreshing;
    await Promise.all(this.servers.map((server) => server.whenSettled()));
  }

  /** Restarts the servers that have been started; the others stay off until needed. */
  async restartRunning(): Promise<void> {
    await Promise.all(
      this.servers.filter((server) => server.state !== "stopped").map((s) => s.restart()),
    );
  }

  channelName(project: Project): string {
    return channelName(project, workspaceFolderOf(project), this.byConfig.size <= 1);
  }

  /** A project's name for people: its folder relative to its workspace folder. */
  name(project: Project): string {
    return projectName(project, workspaceFolderOf(project));
  }

  ownedElsewhere(project: Project, uri: vscode.Uri): boolean {
    return ownedElsewhere(project, uri.fsPath, this.projects);
  }

  async dispose(): Promise<void> {
    clearTimeout(this.lookAgainTimer);
    this.lookAgainTimer = undefined;
    for (const disposable of this.disposables.splice(0)) disposable.dispose();
    await this.refreshing.catch(() => undefined);
    await Promise.all(this.servers.map((server) => server.dispose()));
    this.byConfig.clear();
    this.projectsChanged.dispose();
    this.started.dispose();
    this.stateChanged.dispose();
  }

  private async refreshNow(): Promise<void> {
    const asked = this.unknownConfigs;
    this.unknownConfigs = new Set();
    // One more than the cap, to know there were more.
    const found = await vscode.workspace.findFiles(
      "**/ascribe.toml",
      "**/node_modules/**",
      MAX_PROJECTS + 1,
    );
    const all = found.map((uri) => uri.fsPath).sort();
    const configs = all.slice(0, MAX_PROJECTS);
    let changed = false;

    for (const [config, server] of this.byConfig) {
      if (configs.includes(config)) continue;
      this.byConfig.delete(config);
      this.nestedAtStart.delete(server);
      void server.dispose();
      changed = true;
    }
    const next = new Map<string, ProjectServer>();
    for (const config of configs) {
      let server = this.byConfig.get(config);
      if (!server) {
        const created = new ProjectServer(
          this.context,
          { config, folder: path.dirname(config) },
          this,
        );
        created.onDidStart(() => this.started.fire(created));
        created.onDidChangeState(() => this.stateChanged.fire(created));
        server = created;
        changed = true;
      }
      next.set(config, server);
    }
    this.byConfig.clear();
    for (const [config, server] of next) this.byConfig.set(config, server);

    for (const config of asked) {
      if (!this.knows(config)) this.passedOver.add(config);
    }

    // After the servers exist, so the first project's output can say so.
    if (all.length > MAX_PROJECTS && !this.warnedOfCap) {
      this.warnedOfCap = true;
      this.servers[0]?.log(
        `This workspace has more than ${MAX_PROJECTS} ascribe.toml files; only the first ${MAX_PROJECTS} (in path order) get a language server.`,
      );
    }

    // A server started before a nested project appeared (or went away) has
    // already been sent that project's files; starting it over drops them.
    const restarts: Promise<void>[] = [];
    for (const server of this.servers) {
      const nested = nestedProjects(server.project, this.projects)
        .map((project) => project.config)
        .join("\n");
      const before = this.nestedAtStart.get(server);
      if (before !== undefined && before !== nested && server.state !== "stopped") {
        restarts.push(server.restart());
      }
      this.nestedAtStart.set(server, nested);
    }
    await Promise.all(restarts);

    if (changed) this.projectsChanged.fire();
    await this.startAllIfConfigured();
    await this.startForOpenDocuments();
  }

  private async startAllIfConfigured(): Promise<void> {
    if (readStartServers() !== "all") return;
    await Promise.all(this.servers.map((server) => server.start()));
  }

  /** Starts the servers of every document already open. */
  private async startForOpenDocuments(): Promise<void> {
    await Promise.all(
      vscode.workspace.textDocuments.map((document) => this.startFor(document, false)),
    );
  }

  /**
   * Starts the server of the project a Markdown or `ascribe.toml` file belongs
   * to. When the nearest `ascribe.toml` above the file isn't a known project,
   * projects are looked for again: the file watcher can miss a new one, on
   * Linux in a folder made while it runs.
   */
  private async startFor(document: vscode.TextDocument, lookAgain = true): Promise<void> {
    if (document.uri.scheme !== "file") return;
    if (document.languageId !== "markdown" && !/[\\/]ascribe\.toml$/.test(document.uri.fsPath)) {
      return;
    }
    if (lookAgain) {
      const config = await nearestConfig(document.uri);
      if (config && !this.knows(config) && !this.passedOver.has(comparable(config))) {
        this.unknownConfigs.add(comparable(config));
        this.lookAgainSoon();
      }
    }
    await this.ensureStartedFor(document.uri);
  }

  /** Whether a project has this `ascribe.toml`. */
  private knows(config: string): boolean {
    return this.servers.some((server) => samePath(server.project.config, config));
  }

  /** Looks for projects again shortly: files opened together share one search. */
  private lookAgainSoon(): void {
    if (this.lookAgainTimer) return;
    this.lookAgainTimer = setTimeout(() => {
      this.lookAgainTimer = undefined;
      void this.refresh();
    }, LOOK_AGAIN_DELAY_MS);
  }
}

/**
 * The nearest `ascribe.toml` above a file, up to its workspace folder; none
 * for a file outside every workspace folder. A few `stat` calls.
 */
async function nearestConfig(uri: vscode.Uri): Promise<string | undefined> {
  const root = vscode.workspace.getWorkspaceFolder(uri)?.uri.fsPath;
  if (!root) return undefined;
  let folder = path.dirname(uri.fsPath);
  for (;;) {
    const config = path.join(folder, "ascribe.toml");
    try {
      if ((await stat(config)).isFile()) return config;
    } catch {
      // None here.
    }
    const parent = path.dirname(folder);
    if (samePath(folder, root) || parent === folder || !within(parent, root)) return undefined;
    folder = parent;
  }
}

function workspaceFolderOf(project: Project): WorkspaceFolder | undefined {
  const folder = vscode.workspace.getWorkspaceFolder(vscode.Uri.file(project.folder));
  return folder && { path: folder.uri.fsPath, name: folder.name };
}

function readStartServers(): StartServers {
  const value = vscode.workspace.getConfiguration("ascribe").get<string>("startServers");
  return value === "all" ? "all" : "onDemand";
}
