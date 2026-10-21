// The Project view, first in the Ascribe sidebar: the sidebar's project, its
// server's state and binary, Health's counts with when each is from, and
// Check Every Build. A webview view on This page's base (`webviewView.ts`);
// the answer is `projectHealth.ts`'s, and the webview's script is
// `../webview/project.ts`.
//
// It shows the project Pages and Content model show (`SidebarViews.project`),
// which follows the active file and keeps its project when the active file
// belongs to none, so a click in the view never moves it. Each count opens
// the list that already exists. It never runs Check Every Build by itself:
// that takes a second or more on a large project. The agent files' check is
// quicker, and runs the first time the view shows a project, then after
// each save while it shows it (`agentFiles.ts`).

import * as vscode from "vscode";
import type { ProjectServer } from "../client.js";
import { boolean, listOf, nullable, shape, text, type Check } from "../preview/messages.js";
import type { ProjectRegistry } from "../registry.js";
import { formatVersion } from "../version.js";
import type { ProjectAnswer, ProjectReport } from "../webview/viewProtocol.js";
import type { AgentFilesChecks } from "./agentFiles.js";
import type { EveryBuildChecks } from "./checkEveryBuild.js";
import {
  inventoryCounts,
  PROJECT_KEYS,
  projectAnswer,
  syncQuestion,
  type ProjectFacts,
} from "./projectHealth.js";
import type { SidebarViews } from "./sidebarViews.js";
import type { ProjectTasks } from "./tasks.js";
import { WebviewView, type Drawn } from "./webviewView.js";

/** The Project view, for tests. */
export interface ProjectViewApi {
  /** The newest answer: what the view draws next, or has drawn. */
  answer(): ProjectAnswer | undefined;
  /** Whether it's showing: neither collapsed nor in a hidden container. */
  visible(): boolean;
  /** What its webview drew last, if it has drawn. */
  drawn(): Drawn<ProjectReport> | undefined;
  /** Waits until what its webview drew satisfies `predicate`. */
  whenDrawn(
    description: string,
    predicate: (drawn: Drawn<ProjectReport>) => boolean,
    timeout?: number,
  ): Promise<Drawn<ProjectReport>>;
  /** Chooses a count or an action of what the webview drew last, by its key, as a click does. */
  choose(key: string): Promise<void>;
}

export class ProjectView implements vscode.Disposable {
  private readonly view: WebviewView<ProjectAnswer, ProjectReport>;
  private readonly disposables: vscode.Disposable[] = [];
  /** Draws the view again when the day changes, which overdue reviews are counted for. */
  private midnight: ReturnType<typeof setTimeout> | undefined;

  constructor(
    private readonly projects: ProjectRegistry,
    private readonly sidebar: SidebarViews,
    private readonly checks: EveryBuildChecks,
    private readonly agentFiles: AgentFilesChecks,
    private readonly tasks: ProjectTasks,
    extensionUri: vscode.Uri,
  ) {
    this.view = new WebviewView(
      "ascribe.project",
      extensionUri,
      { script: "project.js", title: "Project" },
      projectReport,
      {
        open: (seq, key) => {
          if (this.view.isLatest(seq)) void this.choose(key);
        },
        // The view has no page of its own: the focus goes back to the editor group.
        focusEditor: () =>
          void vscode.commands.executeCommand("workbench.action.focusActiveEditorGroup"),
      },
    );
  }

  get api(): ProjectViewApi {
    return {
      answer: () => this.view.latest,
      visible: () => this.view.visible,
      drawn: () => this.view.drawn,
      whenDrawn: (description, predicate, timeout) =>
        this.view.whenDrawn(description, predicate, timeout),
      choose: (key) => this.choose(key),
    };
  }

  register(): void {
    this.agentFiles.shown = (server) => this.view.visible && this.sidebar.project === server;
    this.disposables.push(
      this.view.register(),
      this.view,
      this.view.onDidShow(() => this.draw()),
      this.sidebar.onDidChangeProject(() => this.draw()),
      this.projects.onDidChangeState((server) => {
        if (server === this.sidebar.project) this.draw();
      }),
      this.projects.onDidChangeProjects(() => this.draw()),
      // A check starts, is made, or stops being current.
      this.checks.onDidChange(() => this.draw()),
      this.sidebar.onDidChangeInventory(() => this.draw()),
      this.agentFiles.onDidChange(() => this.draw()),
    );
    this.draw();
  }

  dispose(): void {
    clearTimeout(this.midnight);
    for (const disposable of this.disposables.splice(0)) disposable.dispose();
  }

  /** Sends the view the project as it stands. It's cheap: nothing is asked of the server. */
  private draw(): void {
    const server = this.sidebar.project;
    if (server && this.view.visible) this.agentFiles.ensure(server);
    this.view.show(projectAnswer(server && this.facts(server)));
    clearTimeout(this.midnight);
    const tomorrow = new Date();
    tomorrow.setHours(24, 0, 1, 0);
    this.midnight = setTimeout(() => this.draw(), tomorrow.getTime() - Date.now());
  }

  private facts(server: ProjectServer): ProjectFacts {
    const { project } = server;
    const binary = server.binary;
    const inventory = this.sidebar.projectInventory;
    return {
      name: this.projects.name(project),
      folder: vscode.workspace.asRelativePath(project.folder, true),
      state: server.state,
      version: binary && formatVersion(binary.version),
      everyBuild: this.checks.resultOf(project.folder),
      inventory: inventory && inventoryCounts(inventory),
      agentFiles: this.agentFiles.resultOf(project.folder),
      now: Date.now(),
      time: (at) => new Date(at).toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" }),
      day: (at) => new Date(at).toLocaleDateString([], { day: "numeric", month: "long" }),
    };
  }

  /**
   * Sync Agent Files, with the files it would change: the count's list. It
   * asks first, naming them, since it rewrites them.
   */
  private async offerSync(server: ProjectServer): Promise<void> {
    const result = this.agentFiles.resultOf(server.project.folder);
    if (result.kind !== "made" || result.stale.length === 0) return;
    const sync = "Sync Agent Files";
    const { message, detail } = syncQuestion(this.projects.name(server.project), result.stale);
    const choice = await vscode.window.showInformationMessage(
      message,
      { modal: true, detail },
      sync,
    );
    if (choice === sync) await this.tasks.run(server.project, "syncAgentFiles");
  }

  /** Does what a count or an action of the view does, for the project it shows. */
  private async choose(key: string): Promise<void> {
    const server = this.sidebar.project;
    if (!server) return;
    switch (key) {
      case PROJECT_KEYS.problems:
        await vscode.commands.executeCommand("workbench.actions.view.problems");
        return;
      case PROJECT_KEYS.examples:
      case PROJECT_KEYS.reviews:
      case PROJECT_KEYS.permalinks: {
        const result = this.checks.resultOf(server.project.folder);
        const first = result.kind === "made" ? result.counted[key].first : null;
        await vscode.commands.executeCommand("workbench.actions.view.problems");
        if (first) {
          const { start, end } = first.range;
          await vscode.window.showTextDocument(vscode.Uri.file(first.file), {
            selection: new vscode.Range(start.line, start.character, end.line, end.character),
          });
        }
        return;
      }
      case PROJECT_KEYS.orphans:
        await this.sidebar.showOnly("pages");
        return;
      case PROJECT_KEYS.unused:
        await this.sidebar.showOnly("model");
        return;
      case PROJECT_KEYS.agentFiles:
        await this.offerSync(server);
        return;
      case PROJECT_KEYS.checkEveryBuild:
        await this.checks.checkProject(server);
        return;
      case PROJECT_KEYS.checkAgentFiles:
        await this.agentFiles.checkProject(server);
        return;
      case PROJECT_KEYS.build:
      case PROJECT_KEYS.syncAgentFiles:
        await this.tasks.run(server.project, key);
        return;
      default:
        return;
    }
  }
}

/** A check of what the Project view's webview says it drew. */
const projectReport: Check<ProjectReport> = shape<ProjectReport>({
  message: nullable(text),
  head: listOf(text),
  sections: listOf(
    shape<ProjectReport["sections"][number]>({ key: text, label: text, open: boolean }),
  ),
  health: listOf(
    shape<ProjectReport["health"][number]>({
      key: text,
      value: text,
      note: text,
      state: text,
      dimmed: boolean,
    }),
  ),
  buttons: listOf(text),
  violations: listOf(text),
});
