// The Project view, first in the Ascribe sidebar: the sidebar's project, its
// server's state and binary, how many problems every build has with when
// that count is from, and Check Every Build. A webview view on This page's
// base (`webviewView.ts`); the answer is `projectHealth.ts`'s, and the
// webview's script is `../webview/project.ts`.
//
// It shows the project Pages and Content model show (`SidebarViews.project`),
// which follows the active file and keeps its project when the active file
// belongs to none, so a click in the view never moves it. It never runs the
// check by itself: that takes a second or more on a large project.

import * as vscode from "vscode";
import type { ProjectServer } from "../client.js";
import { boolean, listOf, nullable, shape, text, type Check } from "../preview/messages.js";
import type { ProjectRegistry } from "../registry.js";
import { formatVersion } from "../version.js";
import type { ProjectAnswer, ProjectReport } from "../webview/viewProtocol.js";
import type { EveryBuildChecks } from "./checkEveryBuild.js";
import { PROJECT_KEYS, projectAnswer, type ProjectFacts } from "./projectHealth.js";
import type { SidebarViews } from "./sidebarViews.js";
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

  constructor(
    private readonly projects: ProjectRegistry,
    private readonly sidebar: SidebarViews,
    private readonly checks: EveryBuildChecks,
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
    );
    this.draw();
  }

  dispose(): void {
    for (const disposable of this.disposables.splice(0)) disposable.dispose();
  }

  /** Sends the view the project as it stands. It's cheap: nothing is asked of the server. */
  private draw(): void {
    const server = this.sidebar.project;
    this.view.show(projectAnswer(server && this.facts(server)));
  }

  private facts(server: ProjectServer): ProjectFacts {
    const { project } = server;
    const binary = server.binary;
    return {
      name: this.projects.name(project),
      folder: vscode.workspace.asRelativePath(project.folder, true),
      state: server.state,
      version: binary && formatVersion(binary.version),
      everyBuild: this.checks.resultOf(project.folder),
      time: (at) => new Date(at).toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" }),
    };
  }

  /** Does what a count or an action of the view does, for the project it shows. */
  private async choose(key: string): Promise<void> {
    const server = this.sidebar.project;
    if (!server) return;
    switch (key) {
      case PROJECT_KEYS.problems:
        await vscode.commands.executeCommand("workbench.actions.view.problems");
        return;
      case PROJECT_KEYS.checkEveryBuild:
        await this.checks.checkProject(server);
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
