// The Project view, first in the Ascribe sidebar: the sidebar's project, its
// server's state and binary with Show Output and Restart, the build you're
// looking at, the two settings buttons, Health's counts with when each is
// from, its actions: Check Every Build, the edits to the whole project
// (`projectEdits.ts`), the tasks (`tasks.ts`), Start Review, and Prompt
// Agent; and About. A webview view on This page's base (`webviewView.ts`);
// the answer is `projectHealth.ts`'s, and the webview's script is
// `../webview/project.ts`.
//
// Its build switch sets the one choice of the build you're looking at
// (`ChosenBuilds`), which the status bar and the preview follow; its buttons
// run what the status bar's menu runs, for the view's own project. It's made
// before the folder is trusted, so in Restricted Mode it says so, with every
// action off (`restrictedAnswer`).
//
// It shows the project Pages and Content model show (`SidebarViews.project`),
// which follows the active file and keeps its project when the active file
// belongs to none, so a click in the view never moves it. Each count opens
// the list that already exists. It never runs Check Every Build by itself:
// that takes a second or more on a large project. The agent files' check is
// quicker, and runs the first time the view shows a project, then after
// each save while it shows it (`agentFiles.ts`).

import * as path from "node:path";
import * as vscode from "vscode";
import type { ProjectServer } from "../client.js";
import { boolean, listOf, nullable, shape, text, type Check } from "../preview/messages.js";
import { comparable } from "../projects.js";
import type { ProjectRegistry } from "../registry.js";
import { formatVersion } from "../version.js";
import type { ProjectAnswer, ProjectReport } from "../webview/viewProtocol.js";
import type { AgentFilesChecks } from "./agentFiles.js";
import type { EveryBuildChecks } from "./checkEveryBuild.js";
import { choices, type ChosenBuilds } from "./chosenBuild.js";
import type { ProjectBuilds } from "./projectBuilds.js";
import type { ProjectEdits } from "./projectEdits.js";
import {
  CHOOSE_BUILD,
  inventoryCounts,
  PROJECT_KEYS,
  projectAnswer,
  restrictedAnswer,
  syncQuestion,
  type AboutFacts,
  type ProjectFacts,
} from "./projectHealth.js";
import type { SidebarViews } from "./sidebarViews.js";
import type { ProjectTasks } from "./tasks.js";
import { WebviewView, type Drawn } from "./webviewView.js";

/** The command of Manage Workspace Trust, under the view's sentence in Restricted Mode. */
const MANAGE_TRUST = "workbench.trust.manage";

/**
 * The Project view's webview, made first, before the folder is trusted: in
 * Restricted Mode it says so, with every action off, and Manage Workspace
 * Trust. `ProjectView` takes it over once the extension runs.
 */
export function projectWebview(
  extensionUri: vscode.Uri,
): WebviewView<ProjectAnswer, ProjectReport> {
  const view = new WebviewView<ProjectAnswer, ProjectReport>(
    "ascribe.project",
    extensionUri,
    { script: "project.js", title: "Project" },
    projectReport,
    {
      open: (seq, key) => {
        if (view.isLatest(seq) && key === PROJECT_KEYS.manageTrust) {
          void vscode.commands.executeCommand(MANAGE_TRUST);
        }
      },
      focusEditor: () =>
        void vscode.commands.executeCommand("workbench.action.focusActiveEditorGroup"),
    },
  );
  return view;
}

/** Shows the Project view's Restricted Mode: a sentence in place of the counts, and every action off. */
export function showRestrictedProject(view: WebviewView<ProjectAnswer, ProjectReport>): void {
  view.show(restrictedAnswer());
}

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
  /**
   * Chooses a count, an action, or a button of what the webview drew last,
   * by its key, as a click does; a build, by `chooseBuild/` and its name, as
   * the switch does.
   */
  choose(key: string): Promise<void>;
}

/** The services the Project view acts through. */
export interface ProjectViewServices {
  projects: ProjectRegistry;
  sidebar: SidebarViews;
  checks: EveryBuildChecks;
  agentFiles: AgentFilesChecks;
  tasks: ProjectTasks;
  edits: ProjectEdits;
  builds: ProjectBuilds;
  chosen: ChosenBuilds;
  /** The extension's id, which VS Code's Settings are filtered by. */
  extensionId: string;
}

export class ProjectView implements vscode.Disposable {
  private readonly disposables: vscode.Disposable[] = [];
  /** Draws the view again when the day changes, which overdue reviews are counted for. */
  private midnight: ReturnType<typeof setTimeout> | undefined;
  /** Each project's sources, for About, by its folder: asked once, and again when they may have changed. */
  private readonly sources = new Map<string, AboutFacts["sources"]>();
  private readonly projects: ProjectRegistry;
  private readonly sidebar: SidebarViews;
  private readonly checks: EveryBuildChecks;
  private readonly agentFiles: AgentFilesChecks;
  private readonly tasks: ProjectTasks;
  private readonly edits: ProjectEdits;
  private readonly builds: ProjectBuilds;
  private readonly chosen: ChosenBuilds;
  private readonly extensionId: string;

  constructor(
    private readonly view: WebviewView<ProjectAnswer, ProjectReport>,
    services: ProjectViewServices,
  ) {
    this.projects = services.projects;
    this.sidebar = services.sidebar;
    this.checks = services.checks;
    this.agentFiles = services.agentFiles;
    this.tasks = services.tasks;
    this.edits = services.edits;
    this.builds = services.builds;
    this.chosen = services.chosen;
    this.extensionId = services.extensionId;
    view.handlers = {
      open: (seq, key) => {
        if (this.view.isLatest(seq)) void this.choose(key);
      },
      // The view has no page of its own: the focus goes back to the editor group.
      focusEditor: () =>
        void vscode.commands.executeCommand("workbench.action.focusActiveEditorGroup"),
    };
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
      // Its builds are known, or the build you're looking at changed.
      this.builds.onDidChange(() => this.draw()),
      this.tasks.onDidForgetSources(() => {
        this.sources.clear();
        this.draw();
      }),
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
    if (server && this.view.visible) {
      this.agentFiles.ensure(server);
      // The last check, from an earlier session: read back once the view first shows the project.
      this.checks.recall(server.project);
    }
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
    const builds = this.builds.builds(server);
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
      builds,
      build: this.chosen.shown(project.folder, builds),
      about: {
        contentRoot: inventory?.contentUri && contentRootOf(project.folder, inventory.contentUri),
        types: inventory?.model
          .filter((entry) => entry.kind === "type")
          .map((entry) => ({ key: entry.key, pages: entry.uses })),
        sources: this.sourcesOf(server),
      },
    };
  }

  /**
   * The project's sources, for About: asked of the binary the first time the
   * view shows the project, and drawn again once it answers. A hidden view
   * starts no process, as the agent files' check doesn't.
   */
  private sourcesOf(server: ProjectServer): AboutFacts["sources"] {
    const key = comparable(server.project.folder);
    const known = this.sources.get(key);
    if (known !== undefined) return known;
    if (!this.view.visible) return "asking";
    this.sources.set(key, "asking");
    void this.tasks.sourcesFor(server.project).then((sources) => {
      // Forgotten while it was asked: it's asked again.
      if (this.sources.get(key) !== "asking") return;
      this.sources.set(
        key,
        sources === undefined
          ? "failed"
          : sources.map((source) => ({
              name: source.name,
              path: source.path,
              tested: source.examples !== undefined,
            })),
      );
      this.draw();
    });
    return "asking";
  }

  /** Chooses the build you're looking at, as the status bar's Switch Build and the preview's picker do. */
  private chooseBuild(server: ProjectServer, build: string): void {
    const builds = this.builds.builds(server);
    if (!choices(builds).some((choice) => choice.name === build)) return;
    const editor = builds.find((each) => each.editor)?.name;
    this.chosen.choose(server.project.folder, build, editor);
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
    if (key.startsWith(CHOOSE_BUILD)) {
      this.chooseBuild(server, key.slice(CHOOSE_BUILD.length));
      return;
    }
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
      case PROJECT_KEYS.applyEverySafeFix:
        await this.edits.run(server, "fixAll");
        return;
      case PROJECT_KEYS.formatEveryPage:
        await this.edits.run(server, "format");
        return;
      case PROJECT_KEYS.checkAgentFiles:
        await this.agentFiles.checkProject(server);
        return;
      case PROJECT_KEYS.build:
      case PROJECT_KEYS.syncAgentFiles:
      case PROJECT_KEYS.observeExamples:
        await this.tasks.run(server.project, key);
        return;
      // The Command Palette's commands, given the view's project.
      case PROJECT_KEYS.startReview:
        await vscode.commands.executeCommand("ascribe.startReview", server.project.folder);
        return;
      case PROJECT_KEYS.promptAgent:
        await vscode.commands.executeCommand("ascribe.promptAgentProject", server.project.folder);
        return;
      // What the status bar's menu runs for the active file's project.
      case PROJECT_KEYS.showOutput:
        server.showOutput();
        return;
      case PROJECT_KEYS.restart:
        await server.restart();
        return;
      case PROJECT_KEYS.projectSettings:
        await vscode.commands.executeCommand("ascribe.openProjectSettings", server.project.folder);
        return;
      case PROJECT_KEYS.editorSettings:
        await vscode.commands.executeCommand(
          "workbench.action.openSettings",
          `@ext:${this.extensionId}`,
        );
        return;
      default:
        return;
    }
  }
}

/** The content root, from the project's folder, with forward slashes: `.` for the folder itself. */
function contentRootOf(folder: string, contentUri: string): string | undefined {
  let root: string;
  try {
    root = vscode.Uri.parse(contentUri, true).fsPath;
  } catch {
    return undefined;
  }
  const relative = path.relative(folder, root);
  if (relative === "") return ".";
  return path.isAbsolute(relative) ? root : relative.split(path.sep).join("/");
}

/** A check of what the Project view's webview says it drew. */
const projectReport: Check<ProjectReport> = shape<ProjectReport>({
  message: nullable(text),
  head: listOf(text),
  build: nullable(
    shape<NonNullable<ProjectReport["build"]>>({ shown: text, choices: listOf(text) }),
  ),
  sections: listOf(
    shape<ProjectReport["sections"][number]>({ key: text, label: text, open: boolean }),
  ),
  groups: listOf(
    shape<ProjectReport["groups"][number]>({ key: text, head: text, action: nullable(text) }),
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
  off: listOf(text),
  about: listOf(text),
  violations: listOf(text),
});
