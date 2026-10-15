import * as vscode from "vscode";
import { resolveProjectBinary, workspaceFolderOf } from "../client.js";
import type { ProjectRegistry } from "../registry.js";
import { mcpServerSpec, type McpServerSpec } from "./mcpServer.js";

/** The id of the provider, as `contributes.mcpServerDefinitionProviders` declares it. */
const MCP_PROVIDER = "ascribe.mcp";

/** What the extension returns for tests. */
export interface McpApi {
  /** Whether the provider is registered: VS Code has the API. */
  registered(): boolean;
  /** The server VS Code is offered now; `undefined` when there's none. */
  spec(): Promise<McpServerSpec | undefined>;
}

/**
 * Offers `ascribe mcp` to VS Code's agents, in a workspace with an Ascribe
 * project, so they get its tools with nothing to set up. Starting the
 * server is VS Code's: it runs when an agent first needs it.
 */
export class McpRegistration implements vscode.Disposable {
  private readonly changed = new vscode.EventEmitter<void>();
  private readonly disposables: vscode.Disposable[] = [];
  private isRegistered = false;
  private lastNote: string | undefined;
  /** The server last worked out, until a project or `ascribe.path` changes. */
  private cached: Promise<McpServerSpec | undefined> | undefined;

  constructor(
    private readonly context: vscode.ExtensionContext,
    private readonly projects: ProjectRegistry,
  ) {}

  /** Registers the provider, when this VS Code has the API; without it, does nothing. */
  register(): void {
    // A VS Code older than the manifest asks for, or an editor built on it,
    // may not have the API: the rest of the extension works without it.
    const lm = (vscode as { lm?: Partial<typeof vscode.lm> }).lm;
    if (
      typeof lm?.registerMcpServerDefinitionProvider !== "function" ||
      typeof vscode.McpStdioServerDefinition !== "function"
    ) {
      return;
    }
    this.disposables.push(
      lm.registerMcpServerDefinitionProvider(MCP_PROVIDER, {
        onDidChangeMcpServerDefinitions: this.changed.event,
        provideMcpServerDefinitions: () => this.definitions(),
      }),
      this.projects.onDidChangeProjects(() => this.forget()),
      // A server starting may have found a binary installed since.
      this.projects.onDidStart(() => this.forget()),
      vscode.workspace.onDidChangeConfiguration((event) => {
        if (event.affectsConfiguration("ascribe.path")) this.forget();
      }),
    );
    this.isRegistered = true;
  }

  get api(): McpApi {
    return { registered: () => this.isRegistered, spec: () => this.spec() };
  }

  /** Works the server out again when VS Code next asks, and tells it to ask. */
  private forget(): void {
    this.cached = undefined;
    this.changed.fire();
  }

  private async definitions(): Promise<vscode.McpStdioServerDefinition[]> {
    this.cached ??= this.spec();
    const spec = await this.cached;
    if (!spec) return [];
    if (spec.note !== undefined && spec.note !== this.lastNote) {
      this.projects.servers[0]?.log(spec.note);
    }
    this.lastNote = spec.note;
    const definition = new vscode.McpStdioServerDefinition(
      spec.label,
      spec.command,
      spec.args,
      {},
      spec.version,
    );
    definition.cwd = vscode.Uri.file(spec.cwd);
    return [definition];
  }

  /** The server for the workspace's projects, with the first one's binary; none without a project. */
  private async spec(): Promise<McpServerSpec | undefined> {
    const servers = this.projects.servers;
    const first = servers[0];
    if (!first) return undefined;
    const candidates = await Promise.all(
      servers.map(async (server) => {
        const resolution = await resolveProjectBinary(this.context, server.project);
        return {
          name: this.projects.name(server.project),
          binary: resolution.kind === "found" ? resolution.binary : undefined,
        };
      }),
    );
    return mcpServerSpec(candidates, workspaceFolderOf(first.project));
  }

  dispose(): void {
    for (const disposable of this.disposables.splice(0)) disposable.dispose();
    this.changed.dispose();
  }
}
