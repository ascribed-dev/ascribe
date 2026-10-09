import * as vscode from "vscode";
import type { ProjectServer, ServerState } from "./client.js";
import { ProjectRegistry } from "./registry.js";
import type { ResolvedBinary } from "./binary.js";
import { PreviewController, type PreviewApi } from "./preview/controller.js";
import { ActionsController, type ActionsApi } from "./actions/controller.js";
import { PromptAgent, type PromptAgentApi } from "./actions/promptAgent.js";
import { McpRegistration, type McpApi } from "./agents/mcp.js";
import { AgentTools, type ToolsApi } from "./agents/tools.js";
import { BuildLens, type BuildLensApi } from "./ui/buildLens.js";
import { ChosenBuilds } from "./ui/chosenBuild.js";
import { STATE_NAMES } from "./ui/describe.js";
import { BuildLenses } from "./ui/lens.js";
import { ProjectBuilds } from "./ui/projectBuilds.js";
import { ProjectsView, type ProjectsViewApi } from "./ui/projectsView.js";
import { SidebarViews, type SidebarApi } from "./ui/sidebarViews.js";
import { StatusBar, type StatusBarApi } from "./ui/statusBar.js";
import { Walkthrough } from "./ui/walkthrough.js";

/** What the extension returns from `activate`, for tests and other extensions. */
export interface AscribeApi {
  /**
   * The binary in use, if one was found, for the project in `folder`: the only
   * or first project when it's omitted.
   */
  binary(folder?: string): ResolvedBinary | undefined;
  /** The server's state for the project in `folder` (`stopped` when it has none). */
  state(folder?: string): ServerState;
  /** Every project in the workspace, by folder, with its server's state. */
  projects(): { folder: string; state: ServerState }[];
  /** Fires with a project's folder each time its server starts or restarts. */
  onDidStartServer: vscode.Event<string>;
  /** Settles when every start or restart under way is over. */
  whenSettled(): Promise<void>;
  /** The preview, for tests. */
  preview: PreviewApi;
  /** The editor's actions, for tests. */
  actions: ActionsApi;
  /** Prompt agent, for tests. */
  promptAgent: PromptAgentApi;
  /** What agents in VS Code get: the MCP server and the tools, for tests. */
  agents: { mcp: McpApi; tools: ToolsApi };
  /** The status bar item and the sidebar's views, for tests. */
  ui: {
    statusBar: StatusBarApi;
    projects: ProjectsViewApi;
    lens: BuildLensApi;
    /** The Used by, Pages, and Content model views. */
    sidebar: SidebarApi;
    /** Settles when every request for a project's builds has been answered. */
    whenBuildsKnown(): Promise<void>;
  };
}

let registry: ProjectRegistry | undefined;

export async function activate(context: vscode.ExtensionContext): Promise<AscribeApi> {
  const projects = new ProjectRegistry(context);
  registry = projects;

  const chosen = new ChosenBuilds();
  const promptAgent = new PromptAgent(context, projects);
  promptAgent.register();
  const preview = new PreviewController(context, projects, chosen, promptAgent);
  preview.register();
  const mcp = new McpRegistration(context, projects);
  mcp.register();
  const tools = new AgentTools(projects, preview.review);
  tools.register();

  const lenses = new BuildLenses();
  const builds = new ProjectBuilds(projects, chosen, lenses);
  builds.register();
  const statusBar = new StatusBar(projects, builds, chosen, lenses);
  statusBar.register();
  const lens = new BuildLens(projects, builds, chosen, lenses);
  lens.register();
  const projectsView = new ProjectsView(projects, builds);
  projectsView.register();
  const sidebar = new SidebarViews(projects);
  sidebar.register();

  const actions = new ActionsController(projects);
  actions.register();
  const walkthrough = new Walkthrough(projects);
  walkthrough.register();

  context.subscriptions.push(
    vscode.workspace.onWillSaveTextDocument((event) => {
      if (
        event.document.languageId !== "markdown" ||
        !vscode.workspace.getConfiguration("ascribe", event.document.uri).get("formatOnSave", false)
      ) {
        return;
      }
      const server = projects.serverFor(event.document.uri);
      if (!server || server.state !== "running") return;
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
          .catch((error: unknown) => {
            server.reportFeatureError("format on save", error);
            return [];
          }),
      );
    }),
  );

  context.subscriptions.push(
    projects,
    preview,
    mcp,
    tools,
    actions,
    promptAgent,
    chosen,
    lenses,
    builds,
    statusBar,
    lens,
    projectsView,
    sidebar,
    walkthrough,
    vscode.commands.registerCommand("ascribe.restartServer", async () => {
      await projects.refresh();
      if (projects.projects.length === 0) {
        void vscode.window.showInformationMessage("Ascribe: this workspace has no ascribe.toml.");
        return;
      }
      // Restarts the servers that have started; the others start when they're needed.
      if (projects.servers.every((server) => server.state === "stopped")) {
        void vscode.window.showInformationMessage(
          "Ascribe: no language server is running. A project's server starts when you open one of its files.",
        );
        return;
      }
      await projects.restartRunning();
    }),
    // The server's own command (a code lens opens the file it names). One
    // registration for every project, answered by the active file's server.
    vscode.commands.registerCommand("ascribe.openFile", (...args: unknown[]) =>
      projects.request("workspace/executeCommand", {
        command: "ascribe.openFile",
        arguments: args,
      }),
    ),
    vscode.commands.registerCommand("ascribe.showOutput", async () => {
      const active = vscode.window.activeTextEditor?.document.uri;
      const server = (active && projects.serverFor(active)) ?? (await pickServer(projects));
      server?.showOutput();
    }),
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
    projects: () =>
      projects.servers.map((server) => ({ folder: server.project.folder, state: server.state })),
    onDidStartServer: (listener, thisArgs, disposables) =>
      projects.onDidStart(
        (server) => listener.call(thisArgs, server.project.folder),
        undefined,
        disposables,
      ),
    whenSettled: () => projects.whenSettled(),
    preview: preview.api,
    actions: actions.api,
    promptAgent: promptAgent.api,
    agents: { mcp: mcp.api, tools: tools.api },
    ui: {
      statusBar: statusBar.api,
      projects: projectsView.api,
      lens: lens.api,
      sidebar: sidebar.api,
      whenBuildsKnown: () => builds.whenSettled(),
    },
  };
}

/** The only project's server, or the one picked from a list when there are several. */
async function pickServer(projects: ProjectRegistry): Promise<ProjectServer | undefined> {
  const servers = projects.servers;
  if (servers.length <= 1) return servers[0];
  const picked = await vscode.window.showQuickPick(
    servers.map((server) => ({
      label: projects.name(server.project),
      description: STATE_NAMES[server.state],
      server,
    })),
    {
      title: "Show Server Output",
      placeHolder: "The project whose language server output to show",
    },
  );
  return picked?.server;
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
  // `null` is a valid answer: nothing to format (a file outside the content root).
  if (value === null) return [];
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
