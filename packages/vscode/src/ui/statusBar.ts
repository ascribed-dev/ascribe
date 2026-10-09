import * as vscode from "vscode";
import type { ProjectServer } from "../client.js";
import type { ProjectRegistry } from "../registry.js";
import type { ChosenBuilds } from "./chosenBuild.js";
import { statusFor, type StatusText } from "./describe.js";
import type { ProjectBuilds } from "./projectBuilds.js";

const MENU_COMMAND = "ascribe.projectMenu";
const SWITCH_BUILD_COMMAND = "ascribe.switchBuild";

/** What the status bar item shows, for tests. */
export interface StatusBarApi {
  /** Its text and tooltip while it's shown; `undefined` while it's hidden. */
  shown(): { text: string; tooltip: string } | undefined;
  /** Answers the next quick pick it shows with the item whose label starts with this, instead of asking. */
  answerNext(label: string): void;
}

/**
 * The status bar item: the active file's project, the build you're looking
 * at, and the server's state. Clicking it opens a small menu for the project.
 */
export class StatusBar implements vscode.Disposable {
  private readonly item = vscode.window.createStatusBarItem(
    "ascribe.project",
    vscode.StatusBarAlignment.Left,
    0,
  );
  private current: StatusText | undefined;
  private readonly answers: string[] = [];
  private readonly disposables: vscode.Disposable[] = [];

  constructor(
    private readonly projects: ProjectRegistry,
    private readonly builds: ProjectBuilds,
    private readonly chosen: ChosenBuilds,
  ) {
    this.item.name = "Ascribe Project";
    this.item.command = MENU_COMMAND;
  }

  get api(): StatusBarApi {
    return {
      shown: () => this.current && { text: this.current.text, tooltip: this.current.tooltip },
      answerNext: (label) => this.answers.push(label),
    };
  }

  register(): void {
    this.disposables.push(
      this.item,
      vscode.window.onDidChangeActiveTextEditor(() => this.update()),
      // A file's language can change after it opens, from plain text to Markdown.
      vscode.workspace.onDidOpenTextDocument(() => this.update()),
      this.builds.onDidChange(() => this.update()),
      vscode.commands.registerCommand(MENU_COMMAND, () => this.showMenu()),
      vscode.commands.registerCommand(SWITCH_BUILD_COMMAND, () => this.switchBuild()),
    );
    this.update();
  }

  dispose(): void {
    for (const disposable of this.disposables.splice(0)) disposable.dispose();
  }

  /** The active file's project's server, if the item is about one. */
  private server(): ProjectServer | undefined {
    const uri = vscode.window.activeTextEditor?.document.uri;
    return uri && this.projects.serverFor(uri);
  }

  private update(): void {
    const document = vscode.window.activeTextEditor?.document;
    this.current = statusFor(
      document && {
        scheme: document.uri.scheme,
        path: document.uri.fsPath,
        languageId: document.languageId,
      },
      this.projects.servers.map((server) => this.builds.info(server)),
    );
    if (!this.current) {
      this.item.hide();
      return;
    }
    this.item.text = this.current.text;
    this.item.tooltip = this.current.tooltip;
    this.item.accessibilityInformation = { label: this.current.label };
    this.item.show();
  }

  private async showMenu(): Promise<void> {
    const server = this.server();
    if (!server) return;
    const page = vscode.window.activeTextEditor?.document.languageId === "markdown";
    const items: (vscode.QuickPickItem & { run: () => unknown })[] = [
      {
        label: "$(package) Switch build",
        description: this.builds.info(server).build ?? "",
        run: () => this.switchBuild(),
      },
      { label: "$(output) Show output", run: () => server.showOutput() },
      { label: "$(debug-restart) Restart server", run: () => server.restart() },
    ];
    if (page) {
      items.push({
        label: "$(open-preview) Open preview",
        run: () => vscode.commands.executeCommand("ascribe.openPreview"),
      });
    }
    const picked = await this.pick(items, {
      title: this.builds.info(server).name,
      placeHolder: "Ascribe project",
    });
    await picked?.run();
  }

  /** Picks the build the active file's project is looking at. */
  private async switchBuild(): Promise<void> {
    const server = this.server();
    if (!server) {
      void vscode.window.showInformationMessage(
        "Ascribe: the active file isn't part of an Ascribe project.",
      );
      return;
    }
    const builds = this.builds.builds(server);
    if (builds.length === 0) {
      if (server.state === "running") await this.builds.refresh(server);
      if (this.builds.builds(server).length === 0) {
        void vscode.window.showInformationMessage(
          server.state === "running"
            ? "Ascribe: there are no builds to choose from."
            : "Ascribe: the project's language server isn't running, so its builds aren't known. Restart it, then try again.",
        );
        return;
      }
    }
    const info = this.builds.info(server);
    const picked = await this.pick(
      info.builds.map((build) => ({
        label: build.name,
        description: [build.editor ? "editor build" : "", build.name === info.build ? "shown" : ""]
          .filter(Boolean)
          .join(", "),
      })),
      { title: "Switch build", placeHolder: `Now looking at ${info.build ?? "the editor build"}` },
    );
    if (!picked) return;
    const editor = info.builds.find((b) => b.editor)?.name;
    this.chosen.choose(server.project.folder, picked.label, editor);
  }

  private async pick<T extends vscode.QuickPickItem>(
    items: T[],
    options: vscode.QuickPickOptions,
  ): Promise<T | undefined> {
    const answer = this.answers.shift();
    if (answer !== undefined) {
      const plain = (label: string) => label.replace(/^\$\([\w~-]+\)\s*/, "");
      return items.find((item) => plain(item.label).startsWith(answer));
    }
    return vscode.window.showQuickPick(items, options);
  }
}
