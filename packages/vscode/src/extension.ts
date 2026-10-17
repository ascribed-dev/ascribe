import * as vscode from "vscode";
import type { ProjectServer, ServerState } from "./client.js";
import { ProjectRegistry } from "./registry.js";
import type { ResolvedBinary } from "./binary.js";
import { PreviewController, type PreviewApi } from "./preview/controller.js";
import { ActionsController, type ActionsApi } from "./actions/controller.js";
import { NewPage, type NewPageApi } from "./actions/newPage.js";
import { PromptAgent, type PromptAgentApi } from "./actions/promptAgent.js";
import { McpRegistration, type McpApi } from "./agents/mcp.js";
import { TerminalPath, type TerminalsApi } from "./agents/terminals.js";
import { AgentTools, type ToolsApi } from "./agents/tools.js";
import { BuildLens, type BuildLensApi } from "./ui/buildLens.js";
import { EveryBuildChecks, type EveryBuildApi } from "./ui/checkEveryBuild.js";
import { ChosenBuilds } from "./ui/chosenBuild.js";
import { STATE_NAMES } from "./ui/describe.js";
import { BuildLenses } from "./ui/lens.js";
import { ProjectBuilds } from "./ui/projectBuilds.js";
import { ProjectsView, type ProjectsViewApi } from "./ui/projectsView.js";
import { ReviewView, type ReviewViewApi } from "./ui/reviewView.js";
import { SidebarViews, type SidebarApi, type ThisPageApi } from "./ui/sidebarViews.js";
import { showRestricted, StatusBar, type StatusBarApi } from "./ui/statusBar.js";
import { Walkthrough } from "./ui/walkthrough.js";
import { showMemos } from "./ui/memos.js";
import { FormatterOffer } from "./ui/formatter.js";

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
  /** The folders of the projects whose servers registered their lenses for a file, for tests. */
  lensesFor(uri: vscode.Uri): Promise<string[]>;
  /** The preview, for tests. */
  preview: PreviewApi;
  /** The editor's actions, for tests. */
  actions: ActionsApi;
  /** New Page, for tests. */
  newPage: NewPageApi;
  /** Prompt agent, for tests. */
  promptAgent: PromptAgentApi;
  /** What agents in VS Code get: the MCP server, the tools, and the terminals' path, for tests. */
  agents: { mcp: McpApi; tools: ToolsApi; terminals: TerminalsApi };
  /** The status bar item and the sidebar's views, for tests. */
  ui: {
    statusBar: StatusBarApi;
    projects: ProjectsViewApi;
    lens: BuildLensApi;
    /** The Review view, and its Open Source Changes. */
    review: ReviewViewApi;
    /** The This page view. */
    thisPage: ThisPageApi;
    /** The Pages and Content model views. */
    sidebar: SidebarApi;
    /** Check Every Build's problems. */
    everyBuild: EveryBuildApi;
    /** Settles when every request for a project's builds has been answered. */
    whenBuildsKnown(): Promise<void>;
  };
}

let registry: ProjectRegistry | undefined;

export async function activate(context: vscode.ExtensionContext): Promise<AscribeApi> {
  // In Restricted Mode the extension runs nothing, since it would run the
  // project's own binary; it only says so, and starts once the folder is trusted.
  if (!vscode.workspace.isTrusted) {
    const notice = showRestricted();
    context.subscriptions.push(notice);
    await new Promise<void>((resolve) => {
      context.subscriptions.push(vscode.workspace.onDidGrantWorkspaceTrust(resolve));
    });
    notice.dispose();
  }
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
  const terminals = new TerminalPath(context, projects);

  const lenses = new BuildLenses();
  const checks = new EveryBuildChecks(projects);
  checks.register();
  const builds = new ProjectBuilds(projects, chosen, lenses, checks);
  builds.register();
  const statusBar = new StatusBar(projects, builds, chosen, lenses);
  statusBar.register();
  const lens = new BuildLens(projects, builds, chosen, lenses);
  lens.register();
  const projectsView = new ProjectsView(projects, builds);
  projectsView.register();
  const reviewView = new ReviewView(preview.review, projects);
  reviewView.register();
  const actions = new ActionsController(projects);
  actions.register();
  const sidebar = new SidebarViews(projects, preview.review, builds, actions);
  sidebar.register();

  const newPage = new NewPage(projects);
  newPage.register();
  const walkthrough = new Walkthrough(projects);
  walkthrough.register();
  const formatter = new FormatterOffer(context, projects);
  formatter.register();

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
    terminals,
    actions,
    newPage,
    formatter,
    promptAgent,
    chosen,
    lenses,
    checks,
    builds,
    statusBar,
    lens,
    projectsView,
    reviewView,
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
    // The server's quick fix on an example that's out of date: it puts the
    // command to run on the clipboard, for a terminal, where the person sees
    // what it starts. The extension runs nothing an example's config names.
    vscode.commands.registerCommand("ascribe.copyCommand", async (line: unknown) => {
      if (typeof line !== "string") return;
      await vscode.env.clipboard.writeText(line);
      void vscode.window.showInformationMessage(
        `Ascribe: copied \`${line}\`. Run it in a terminal.`,
      );
    }),
    // The command a lens in a code file carries: the pages showing it, in the
    // references peek, as Find All References lists places.
    vscode.commands.registerCommand("ascribe.showReferences", async (...args: unknown[]) => {
      const shown = referencesOf(args);
      if (!shown) return;
      await vscode.commands.executeCommand(
        "editor.action.showReferences",
        shown.uri,
        shown.position,
        shown.locations,
      );
    }),
    // A lens on several memos lists them to open one (crates/ascribe-lsp/src/memos.rs).
    vscode.commands.registerCommand("ascribe.showMemos", (memos: unknown) => showMemos(memos)),
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
  terminals.register();
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
    lensesFor: async (uri) => {
      const document = await vscode.workspace.openTextDocument(uri);
      return projects.servers
        .filter((server) => server.registeredLenses(document))
        .map((server) => server.project.folder);
    },
    preview: preview.api,
    actions: actions.api,
    newPage: newPage.api,
    promptAgent: promptAgent.api,
    agents: { mcp: mcp.api, tools: tools.api, terminals: terminals.api },
    ui: {
      statusBar: statusBar.api,
      projects: projectsView.api,
      lens: lens.api,
      review: reviewView.api,
      thisPage: sidebar.thisPageApi,
      sidebar: sidebar.api,
      everyBuild: checks.api,
      whenBuildsKnown: () => builds.whenSettled(),
    },
  };
}

/**
 * The arguments of `ascribe.showReferences` as the server writes them, in the
 * editor's types: a file's URI, a position in it, and locations, each a URI
 * and a range. `undefined` when they aren't that.
 */
function referencesOf(
  args: unknown[],
): { uri: vscode.Uri; position: vscode.Position; locations: vscode.Location[] } | undefined {
  const [uri, position, locations] = args as [unknown, unknown, unknown];
  const asPosition = (value: unknown): vscode.Position | undefined => {
    const p = value as { line?: unknown; character?: unknown } | null;
    return typeof p?.line === "number" && typeof p.character === "number"
      ? new vscode.Position(p.line, p.character)
      : undefined;
  };
  const at = asPosition(position);
  if (typeof uri !== "string" || !at || !Array.isArray(locations)) return undefined;
  const found: vscode.Location[] = [];
  for (const location of locations as {
    uri?: unknown;
    range?: { start?: unknown; end?: unknown };
  }[]) {
    const start = asPosition(location.range?.start);
    const end = asPosition(location.range?.end);
    if (typeof location.uri !== "string" || !start || !end) return undefined;
    found.push(new vscode.Location(vscode.Uri.parse(location.uri), new vscode.Range(start, end)));
  }
  return { uri: vscode.Uri.parse(uri), position: at, locations: found };
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
