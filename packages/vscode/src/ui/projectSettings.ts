// Project Settings: an editor tab, a webview panel, that shows a project's
// `ascribe.toml` settings with their values, defaults, descriptions, and
// lines, and changes a simple value in place. Opened from the Command
// Palette (Ascribe: Open Project Settings), the Project view's gear, and the
// button in `ascribe.toml`'s title bar, which is offered only on the
// `ascribe.toml` of a project the extension found: not on one
// `ascribe.projects.exclude` leaves out, such as a test fixture's. One tab
// per project, found by the path of its `ascribe.toml`; when that file is
// deleted or renamed, the tab says the project is gone, and closes on the
// next click.
//
// It keeps the rules of the sidebar's webview views (decision #651a): the
// shell and its content security policy (`../webview/viewShell.ts`), the
// handshake with the newest answer sent (`handshake.ts`), numbered answers,
// and text set as text. Its webview isn't kept alive while its tab is hidden
// (`retainContextWhenHidden`): shown again, it says it's ready, and gets the
// newest answer.
//
// What it draws is the server's `ascribe/settings`, read from the text the
// editor has, and nothing else: `settingsModel.ts` has the rule. A change, a
// value added, and a reset to the default are each `ascribe/edit`'s
// `setSetting`, applied as a workspace edit, so `ascribe.toml` changes
// unsaved, Undo puts it back, and the tab follows the document as it follows
// typing. A check's level and limit are set the same way.

import { randomBytes } from "node:crypto";
import * as vscode from "vscode";
import type { ProjectServer } from "../client.js";
import { samePath } from "../projects.js";
import type { ProjectRegistry } from "../registry.js";
import type { SettingsResult } from "../shapes.js";
import type { SettingsAnswer, SettingsReport } from "../webview/settingsProtocol.js";
import { viewHtml } from "../webview/viewShell.js";
import { STATE_NAMES } from "./describe.js";
import { Handshake } from "./handshake.js";
import {
  allSettings,
  checkSettingsMessage,
  checkedValue,
  GONE,
  inStep,
  setInStep,
  settingTarget,
  settingsAnswer,
  settingsMessage,
} from "./settingsModel.js";
import type { Drawn } from "./webviewView.js";
import { changedElsewhere, otherModels } from "../takenFiles.js";

/** How many times an answer for other text is asked for again before the tab waits for the next change. */
const TRIES = 5;

/**
 * The context key that lists the `ascribe.toml` files the title bar's button
 * is offered on, each as VS Code may name it in `resourcePath`.
 */
const CONFIGS_KEY = "ascribe.projectConfigs";

/** Project Settings, for tests. */
export interface ProjectSettingsApi {
  /** Opens the tab of the project in `folder`, or of the current project, as the command does. */
  open(folder?: string): Promise<void>;
  /** What the open tab's webview drew last. */
  drawn(): Drawn<SettingsReport> | undefined;
  /** Waits until the open tab's webview has the focus. */
  whenFocused(): Promise<void>;
  /** Waits until what the open tab's webview drew satisfies `predicate`. */
  whenDrawn(
    description: string,
    predicate: (drawn: Drawn<SettingsReport>) => boolean,
    timeout?: number,
  ): Promise<Drawn<SettingsReport>>;
  /** Sets a setting of what the webview drew last, as its control does. */
  set(id: string, value: string | number | boolean): Promise<void>;
  /** Resets a setting of what the webview drew last to its default, as its button does. */
  reset(id: string): Promise<void>;
  /** Opens `ascribe.toml` at a setting's line, as its link does. */
  openLine(id: string): Promise<void>;
  /** How many tabs are open. */
  tabs(): number;
  /** Whether the title bar's button is offered on the file `uri`, as its `when` clause reads it. */
  offeredOn(uri: vscode.Uri): boolean;
  /** Clicks the open tab, as a reader does to close a tab whose `ascribe.toml` is gone. */
  click(): void;
  /** Settles when every request for an answer has been answered. */
  whenSettled(): Promise<void>;
}

export class ProjectSettings implements vscode.Disposable {
  private readonly panels = new Map<string, SettingsPanel>();
  /** The tab opened or shown last, which the test API acts on. */
  private last: SettingsPanel | undefined;
  private readonly disposables: vscode.Disposable[] = [];

  constructor(
    private readonly projects: ProjectRegistry,
    private readonly extensionUri: vscode.Uri,
    private readonly extensionId: string,
  ) {}

  register(): void {
    this.disposables.push(
      // The Project view's gear names its project's folder, and the title
      // bar's button the `ascribe.toml` it's on; the Command Palette, none.
      vscode.commands.registerCommand("ascribe.openProjectSettings", (target?: unknown) => {
        if (target instanceof vscode.Uri) {
          const project = this.projects.projects.find((p) => samePath(p.config, target.fsPath));
          return project ? this.open(project.folder) : undefined;
        }
        return this.open(typeof target === "string" ? target : undefined);
      }),
      this.projects.onDidChangeProjects(() => void this.updateConfigs()),
    );
    void this.updateConfigs();
  }

  /**
   * The `ascribe.toml` files the title bar's button is offered on: each
   * project's, as a path and as a URI's path, since the `resourcePath` its
   * `when` clause compares is one or the other by the version of VS Code. A
   * file it doesn't list, such as a fixture's `ascribe.toml` that
   * `ascribe.projects.exclude` leaves out, isn't a project the extension
   * found, and has no button.
   */
  private configs(): string[] {
    return this.projects.projects.flatMap((project) => {
      const uri = vscode.Uri.file(project.config);
      return [uri.fsPath, uri.path];
    });
  }

  private async updateConfigs(): Promise<void> {
    await vscode.commands.executeCommand("setContext", CONFIGS_KEY, this.configs());
  }

  get api(): ProjectSettingsApi {
    const panel = (): SettingsPanel => {
      if (!this.last) throw new Error("Project Settings isn't open");
      return this.last;
    };
    return {
      open: (folder) => this.open(folder),
      drawn: () => this.last?.drawn,
      whenFocused: () => panel().whenFocused(),
      whenDrawn: (description, predicate, timeout) =>
        panel().whenDrawn(description, predicate, timeout),
      set: (id, value) => panel().set(panel().seq, id, value),
      reset: (id) => panel().set(panel().seq, id, null),
      openLine: (id) => panel().openLine(panel().seq, id),
      tabs: () => this.panels.size,
      offeredOn: (uri) => {
        const configs = this.configs();
        return (
          uri.scheme === "file" && (configs.includes(uri.fsPath) || configs.includes(uri.path))
        );
      },
      click: () => panel().click(),
      whenSettled: () => this.last?.whenSettled() ?? Promise.resolve(),
    };
  }

  /** Opens the tab of a project, or shows the one already open. */
  private async open(folder?: string): Promise<void> {
    const server = folder === undefined ? this.projects.current() : this.projects.serverAt(folder);
    if (!server) {
      void vscode.window.showInformationMessage("Ascribe: this workspace has no ascribe.toml.");
      return;
    }
    const config = server.project.config;
    const open = [...this.panels.entries()].find(([path]) => samePath(path, config))?.[1];
    if (open) {
      this.last = open;
      open.reveal();
      return;
    }
    const panel = new SettingsPanel(
      server,
      this.projects.name(server.project) || "Project Settings",
      this.extensionUri,
      this.extensionId,
      () => {
        this.panels.delete(config);
        if (this.last === panel) this.last = undefined;
      },
    );
    this.panels.set(config, panel);
    this.last = panel;
    await server.start();
    panel.refresh();
  }

  dispose(): void {
    for (const panel of this.panels.values()) panel.dispose();
    for (const disposable of this.disposables.splice(0)) disposable.dispose();
  }
}

/** One project's tab. */
class SettingsPanel implements vscode.Disposable {
  private readonly panel: vscode.WebviewPanel;
  private readonly handshake = new Handshake<SettingsAnswer>();
  /** The server's answer the newest answer was made from. */
  private result: SettingsResult | undefined;
  private drawnLast: Drawn<SettingsReport> | undefined;
  /** Whether the webview's document has the focus, as it said last. */
  private focused = false;
  private waiters: (() => void)[] = [];
  private readonly disposables: vscode.Disposable[] = [];
  private readonly config: vscode.Uri;
  /** The refresh under way, and whether another is due after it. */
  private running: Promise<void> | undefined;
  private again = false;
  /** Whether `ascribe.toml` is gone: deleted, or renamed. */
  private gone = false;

  constructor(
    private readonly server: ProjectServer,
    title: string,
    extensionUri: vscode.Uri,
    private readonly extensionId: string,
    private readonly closed: () => void,
  ) {
    this.config = vscode.Uri.file(server.project.config);
    const root = vscode.Uri.joinPath(extensionUri, "dist", "webview");
    this.panel = vscode.window.createWebviewPanel(
      "ascribe.projectSettings",
      title,
      vscode.ViewColumn.Active,
      { enableScripts: true, localResourceRoots: [root] },
    );
    const webview = this.panel.webview;
    const file = (name: string) => webview.asWebviewUri(vscode.Uri.joinPath(root, name));
    webview.html = viewHtml({
      cspSource: webview.cspSource,
      nonce: randomBytes(16).toString("base64"),
      script: file("settings.js").toString(),
      style: file("settings.css").toString(),
      title: `${title}: Project Settings`,
    });
    const isConfig = (uri: vscode.Uri): boolean =>
      uri.scheme === "file" && samePath(uri.fsPath, this.config.fsPath);
    const watcher = vscode.workspace.createFileSystemWatcher(
      new vscode.RelativePattern(vscode.Uri.file(server.project.folder), "ascribe.toml"),
    );
    this.disposables.push(
      webview.onDidReceiveMessage((message: unknown) => this.receive(message)),
      // Its document is thrown away while the tab is hidden, and made again
      // when it's shown: the ready that follows gets the newest answer.
      this.panel.onDidChangeViewState(() => {
        if (!this.panel.visible) {
          this.handshake.reset();
          this.focused = false;
        }
      }),
      this.panel.onDidDispose(() => this.dispose()),
      // Typing, an edit from the tab, Undo, opening and closing the file, and
      // the server's problems: each is asked about again.
      vscode.workspace.onDidChangeTextDocument((event) => {
        if (isConfig(event.document.uri) && event.contentChanges.length > 0) this.refresh();
      }),
      vscode.workspace.onDidOpenTextDocument((document) => {
        if (isConfig(document.uri)) this.refresh();
      }),
      vscode.workspace.onDidCloseTextDocument((document) => {
        if (isConfig(document.uri)) this.refresh();
      }),
      vscode.languages.onDidChangeDiagnostics((event) => {
        if (event.uris.some(isConfig)) this.refresh();
      }),
      server.onDidChangeState(() => this.refresh()),
      watcher,
      watcher.onDidChange(() => this.refresh()),
      watcher.onDidCreate(() => this.refresh()),
      watcher.onDidDelete(() => this.refresh()),
      // Deleting or renaming `ascribe.toml` stops its server too, and the
      // registry forgets the project.
      vscode.workspace.onDidDeleteFiles(() => this.refresh()),
      vscode.workspace.onDidRenameFiles(() => this.refresh()),
    );
  }

  /** The newest answer's number: what a choice names. */
  get seq(): number {
    return this.handshake.latest?.seq ?? 0;
  }

  get drawn(): Drawn<SettingsReport> | undefined {
    return this.drawnLast;
  }

  reveal(): void {
    this.panel.reveal();
  }

  /** Asks the server again, after the refresh under way if there is one. */
  refresh(): void {
    if (this.running) {
      this.again = true;
      return;
    }
    this.running = (async () => {
      do {
        this.again = false;
        await this.refreshNow();
      } while (this.again);
    })().finally(() => {
      this.running = undefined;
    });
  }

  whenSettled(): Promise<void> {
    return this.running ?? Promise.resolve();
  }

  /** The version of `ascribe.toml`'s open document, or `undefined` when it isn't open. */
  private openVersion(): number | undefined {
    return vscode.workspace.textDocuments.find(
      (document) =>
        document.uri.scheme === "file" && samePath(document.uri.fsPath, this.config.fsPath),
    )?.version;
  }

  /** Whether `ascribe.toml` is on disk. An open document of a deleted file is gone too. */
  private async exists(): Promise<boolean> {
    try {
      await vscode.workspace.fs.stat(this.config);
      return true;
    } catch {
      return false;
    }
  }

  /** A click in the tab: it closes the tab when `ascribe.toml` is gone. */
  click(): void {
    if (this.gone) this.dispose();
  }

  private async refreshNow(): Promise<void> {
    this.gone = !(await this.exists());
    if (this.gone) {
      this.result = undefined;
      this.show(settingsMessage(GONE, true));
      return;
    }
    const state = this.server.state;
    if (state !== "running") {
      this.result = undefined;
      this.show(
        settingsMessage(
          `The Ascribe language server is ${STATE_NAMES[state]}. Project Settings shows the settings once it's running.`,
        ),
      );
      return;
    }
    for (let tries = 0; tries < TRIES; tries++) {
      let result: SettingsResult;
      try {
        result = (await this.server.request("ascribe/settings", {
          textDocument: { uri: this.config.toString() },
        })) as SettingsResult;
      } catch (error) {
        this.server.reportFeatureError("Project Settings", error);
        return;
      }
      // An answer for other text than the editor has is never drawn: the
      // text moved on while the server read it, so it's asked again.
      if (inStep(result.version, this.openVersion())) {
        this.result = result;
        this.show(settingsAnswer(result));
        return;
      }
      await new Promise((resolve) => setTimeout(resolve, 50 * (tries + 1)));
    }
    // Still out of step: the answer drawn is for older text, so it's put
    // away until the next change to the document asks again.
    this.result = undefined;
    this.show(settingsMessage("Reading ascribe.toml…"));
  }

  private show(answer: SettingsAnswer): void {
    const message = this.handshake.answer(answer);
    if (message) void this.panel.webview.postMessage(message);
  }

  private receive(value: unknown): void {
    const message = checkSettingsMessage(value);
    if (!message) return;
    switch (message.type) {
      case "ready": {
        const answer = this.handshake.readied();
        if (answer) void this.panel.webview.postMessage(answer);
        return;
      }
      case "drawn":
        this.drawnLast = { seq: message.seq, report: message.report, at: Date.now() };
        for (const waiter of this.waiters) waiter();
        return;
      case "set":
        void this.set(message.seq, message.id, message.value);
        return;
      case "reset":
        void this.set(message.seq, message.id, null);
        return;
      case "openLine":
        void this.openLine(message.seq, message.id);
        return;
      case "editorSettings":
        void vscode.commands.executeCommand(
          "workbench.action.openSettings",
          `@ext:${this.extensionId}`,
        );
        return;
      case "contentModel":
        void vscode.commands.executeCommand("ascribe.model.focus");
        return;
      case "close":
        this.click();
        return;
      case "focus":
        this.focused = message.focused;
        for (const waiter of this.waiters) waiter();
        return;
      // A tab has no page's editor to give the focus back to.
      case "focusEditor":
      case "open":
        return;
    }
  }

  /**
   * Sets the setting `id` of the answer `seq`, or with `null` resets it to
   * its default, when that answer is still the newest.
   */
  async set(seq: number, id: string, value: unknown): Promise<void> {
    const latest = this.handshake.latest;
    const result = this.result;
    if (!latest || latest.seq !== seq || !result) return;
    const shown = allSettings(latest.answer).find((s) => s.id === id);
    const target = settingTarget(result, id);
    const checked =
      value === null
        ? shown?.resettable
          ? null
          : undefined
        : shown?.editable
          ? checkedValue(shown, value)
          : undefined;
    if (!target || checked === undefined) return;
    try {
      const outcome = await setInStep(
        result.version,
        () => this.openVersion(),
        () =>
          this.server.requestEdit("ascribe/edit", {
            textDocument: { uri: this.config.toString() },
            range: { start: { line: 0, character: 0 }, end: { line: 0, character: 0 } },
            action: "setSetting",
            args: { table: target.table, key: target.key, value: checked, shown: result.version },
          }),
        async (edit) => {
          const applied = await vscode.workspace.applyEdit(edit);
          const elsewhere = changedElsewhere(
            otherModels(
              edit.entries().map(([file]) => vscode.workspace.asRelativePath(file, false)),
              vscode.workspace.asRelativePath(this.config, false),
            ),
          );
          if (applied && elsewhere)
            void vscode.window.showInformationMessage(`Ascribe: ${elsewhere}`);
          return applied;
        },
      );
      switch (outcome.type) {
        case "refused":
          void vscode.window.showErrorMessage(`Ascribe: ${outcome.error}`);
          this.refresh();
          return;
        case "stale":
          void vscode.window.showErrorMessage(
            "Ascribe: `ascribe.toml` changed while the setting was being set. Choose the value again.",
          );
          this.refresh();
          return;
        case "applied":
          if (!outcome.ok) {
            void vscode.window.showErrorMessage(`Ascribe: ascribe.toml couldn't be changed.`);
          }
          return;
      }
    } catch (error) {
      this.server.reportFeatureError("Project Settings", error);
    }
  }

  /** Opens `ascribe.toml` beside the tab, at the line that sets `id` of the answer `seq`, a setting's or a part's. */
  async openLine(seq: number, id: string): Promise<void> {
    const latest = this.handshake.latest;
    const set = this.result && settingTarget(this.result, id)?.set;
    if (!latest || latest.seq !== seq || !set) return;
    const { start, end } = set.range;
    const range = new vscode.Range(start.line, start.character, end.line, end.character);
    await vscode.window.showTextDocument(this.config, {
      viewColumn: vscode.ViewColumn.Beside,
      selection: range,
    });
  }

  whenDrawn(
    description: string,
    predicate: (drawn: Drawn<SettingsReport>) => boolean,
    timeout = 10_000,
  ): Promise<Drawn<SettingsReport>> {
    return this.until(`draw ${description}`, timeout, () => {
      const drawn = this.drawnLast;
      return drawn && predicate(drawn) ? drawn : undefined;
    });
  }

  async whenFocused(timeout = 10_000): Promise<void> {
    await this.until("have the focus", timeout, () => this.focused || undefined);
  }

  /** Settles with what `check` returns once it isn't `undefined`, checked at each message that could change it. */
  private until<T>(what: string, timeout: number, check: () => T | undefined): Promise<T> {
    return new Promise((resolve, reject) => {
      const deadline = setTimeout(() => {
        this.waiters = this.waiters.filter((w) => w !== waiter);
        reject(new Error(`Timed out waiting for Project Settings to ${what}`));
      }, timeout);
      const waiter = (): void => {
        const value = check();
        if (value !== undefined) {
          clearTimeout(deadline);
          this.waiters = this.waiters.filter((w) => w !== waiter);
          resolve(value);
        }
      };
      this.waiters.push(waiter);
      waiter();
    });
  }

  dispose(): void {
    for (const disposable of this.disposables.splice(0)) disposable.dispose();
    this.panel.dispose();
    this.closed();
  }
}
