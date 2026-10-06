import * as vscode from "vscode";
import type { ProjectServer } from "../client.js";
import { comparable } from "../projects.js";
import type { ProjectRegistry } from "../registry.js";
import type { BaseInfo, ChangedPage, ChangesResult, SetBaseResult } from "./protocol.js";
import {
  baseCommit,
  baseName,
  fromContentPath,
  nextChangedPage,
  pageDetail,
  sameBase,
} from "./reviewText.js";
import { ThreadsController, type ThreadsApi } from "./threads.js";

/** The language server's review requests (`crates/tessera-lsp/README.md`). */
const SET_BASE_REQUEST = "ascribe/review/setBase";
const CHANGES_REQUEST = "ascribe/review/changes";

/** What review needs from the preview. */
export interface ReviewHost {
  /** The document the preview shows, if it's open. */
  previewedDocument(): vscode.TextDocument | undefined;
  /** Whether the preview is the active editor. */
  previewActive(): boolean;
  /** The build the preview shows for a project, by folder. */
  previewBuild(folder: string): string | undefined;
  /** Renders the preview again: review was turned on or off. */
  refresh(): void;
  /** Opens a page in the editor and the preview, and goes to its first change. */
  showPage(uri: vscode.Uri): Promise<void>;
}

/** What the extension returns for tests. */
export interface ReviewApi {
  /** The base of the project in `folder`, while review is on. */
  base(folder: string): BaseInfo | undefined;
  /** Starts review of the project in `folder` against `base` (the default branch when omitted), without asking. */
  start(folder: string, base?: string): Promise<SetBaseResult>;
  /** Stops review of the project in `folder`. */
  stop(folder: string): Promise<void>;
  /** The changed pages of the project in `folder`, as the list shows them. */
  changedPages(folder: string): Promise<ChangedPage[]>;
  /** The status bar item's text, and whether it's shown. */
  status(): { text: string; visible: boolean };
  /** The pull request's threads. */
  threads: ThreadsApi;
  /** The offers to start review made so far, as shown. */
  offers(): string[];
  /** Forgets the offers made, and the branches reviewed, this session. */
  forgetOffers(): void;
}

/** The workspace state key for whether review was ever turned on in the workspace. */
const STARTED_ONCE = "ascribe.review.startedOnce";
/** The workspace state key for whether the reviewer turned the offer off. */
const NO_OFFER = "ascribe.review.noOffer";

/**
 * Review: comparing a project with a git revision, its base, so the preview
 * marks what changed. The state is per project and lasts for the session;
 * the base lives in the project's language server, and is set there again
 * after the server restarts.
 */
export class ReviewController implements vscode.Disposable {
  /** The base of each project with review on, by comparable folder. */
  private readonly bases = new Map<string, BaseInfo>();
  private readonly item: vscode.StatusBarItem;
  private disposables: vscode.Disposable[] = [];
  /** The pull request's threads, for the projects with review on. */
  readonly threads: ThreadsController;

  constructor(
    private readonly projects: ProjectRegistry,
    private readonly host: ReviewHost,
    /** The workspace's state. */
    private readonly state: vscode.Memento,
  ) {
    this.item = vscode.window.createStatusBarItem("ascribe.review", vscode.StatusBarAlignment.Left);
    this.item.name = "Ascribe Review";
    this.threads = new ThreadsController(
      {
        changedPages: async (server) => (await this.changes(server))?.pages ?? [],
      },
      state,
    );
  }

  register(): void {
    this.disposables.push(
      this.item,
      vscode.commands.registerCommand("ascribe.startReview", () =>
        this.startCommand(this.runningTarget()),
      ),
      vscode.commands.registerCommand("ascribe.stopReview", () => this.stopCommand()),
      vscode.commands.registerCommand("ascribe.changedPages", () => this.changedPagesCommand()),
      vscode.commands.registerCommand("ascribe.refreshComments", () => this.refreshCommand()),
      this.threads,
      this.threads.onDidChange(() => this.update()),
      vscode.window.onDidChangeActiveTextEditor(() => this.update()),
      vscode.window.onDidChangeWindowState((state) => {
        if (state.focused && this.host.previewActive()) void this.recheck();
      }),
      this.projects.onDidChangeProjects(() => this.update()),
      // A page's server starts after the page opens: offer review once it's up.
      this.projects.onDidStart(() => this.update()),
    );
    this.update();
  }

  get api(): ReviewApi {
    return {
      base: (folder) => this.bases.get(comparable(folder)),
      start: async (folder, base) => {
        const server = this.projects.serverAt(folder);
        if (!server) throw new Error(`no project in ${folder}`);
        const threads = await this.threads.connect(server, { interactive: false });
        return this.start(server, base ?? (threads.state === "on" ? threads.base : undefined));
      },
      stop: async (folder) => {
        const server = this.projects.serverAt(folder);
        if (server) await this.stop(server);
      },
      changedPages: async (folder) => {
        const server = this.projects.serverAt(folder);
        return server ? ((await this.changes(server))?.pages ?? []) : [];
      },
      status: () => ({ text: this.item.text, visible: this.visible }),
      threads: this.threads.api,
      offers: () => [...this.offerLog],
      forgetOffers: () => this.offered.clear(),
    };
  }

  /** The base of a project, while review is on for it. */
  baseOf(server: ProjectServer): BaseInfo | undefined {
    return this.bases.get(comparable(server.project.folder));
  }

  /**
   * Sets a project's base in its server again, after the server lost it (it
   * restarted). Whether it's set; when it can't be, review is turned off and
   * the reader is told why.
   */
  async restore(server: ProjectServer): Promise<boolean> {
    const base = this.baseOf(server);
    if (!base) return false;
    const result = await this.setBase(server, base.requested);
    if (result.base) {
      this.bases.set(comparable(server.project.folder), result.base);
      return true;
    }
    this.bases.delete(comparable(server.project.folder));
    this.update();
    void vscode.window.showWarningMessage(
      `Ascribe: review of ${this.projects.name(server.project)} stopped. ${result.problem ?? ""}`.trim(),
    );
    return false;
  }

  /**
   * Resolves the base of the previewed page's project again, and follows it
   * when it moved: a pull, rebase, or fetch since it was set changes where
   * the branch left it. Cheap when it didn't move; the server keeps the base
   * it read. Called when the preview regains focus.
   */
  async recheck(): Promise<void> {
    const document = this.host.previewedDocument();
    const server = document && this.projects.serverFor(document.uri);
    const base = server && this.baseOf(server);
    if (!server || !base || server.state !== "running" || this.rechecking.has(server)) return;
    this.rechecking.add(server);
    try {
      const result = await this.setBase(server, base.requested);
      // Stopped, or started again, meanwhile.
      if (this.baseOf(server) !== base || !result.base || sameBase(result.base, base)) return;
      this.bases.set(comparable(server.project.folder), result.base);
      this.update();
      this.host.refresh();
      void vscode.window.showInformationMessage(
        `Ascribe: this branch now leaves ${this.baseNameOf(server, result.base)} at ${baseCommit(result.base)}, so review compares with that.`,
      );
    } finally {
      this.rechecking.delete(server);
    }
  }

  private readonly rechecking = new Set<ProjectServer>();

  /** The changed page after `current` (a content path) in the project, for the preview's header. */
  async next(
    server: ProjectServer,
    current: string,
  ): Promise<{ page: { path: string; title: string } | null; first: boolean }> {
    const result = await this.changes(server);
    const next = result && nextChangedPage(result.pages, current);
    if (!next) return { page: null, first: false };
    return {
      page: { path: next.page.path, title: next.page.title ?? next.page.path },
      first: next.first,
    };
  }

  /** Shows the status bar item for the active Ascribe page's or the preview's project. */
  update(): void {
    const server = this.target();
    const base = server && this.baseOf(server);
    void vscode.commands.executeCommand("setContext", "ascribe.reviewOn", base !== undefined);
    if (!server || !this.isAscribeContext()) {
      this.visible = false;
      this.item.hide();
      return;
    }
    if (base) {
      const connection = this.threads.connection(server);
      const pr = connection?.state === "on" ? connection.session.pullRequest : undefined;
      const against = pr
        ? `#${pr.number} against ${baseName(base, pr.baseRefName)}`
        : baseName(base);
      this.item.text = `$(git-compare) Review: ${against}`;
      this.item.tooltip = `Reviewing ${pr ? `pull request #${pr.number} of ` : ""}${this.projects.name(server.project)} against ${baseName(base, pr?.baseRefName)}, from ${baseCommit(base)}, where this branch left it. Click for the changed pages.`;
      this.item.command = "ascribe.changedPages";
    } else {
      this.item.text = "$(git-compare) Review: off";
      this.item.tooltip = `Review is off for ${this.projects.name(server.project)}. Click to start it.`;
      this.item.command = "ascribe.startReview";
      void this.offer(server);
    }
    this.visible = true;
    this.item.show();
  }

  private visible = false;
  /** The branches offered review this session, or reviewed, by checkout: offered once at most. */
  private readonly offered = new Set<string>();
  /** The projects whose branch is being looked at. */
  private readonly looking = new Set<string>();
  private readonly offerLog: string[] = [];

  /**
   * Offers to start review when the project's branch has an open pull
   * request: once a session for each branch of a checkout, and only in a
   * workspace where review was turned on before. Until then, GitHub isn't
   * asked. Never asks for a sign-in.
   */
  private async offer(server: ProjectServer): Promise<void> {
    if (!this.state.get<boolean>(STARTED_ONCE, false) || this.state.get<boolean>(NO_OFFER, false)) {
      return;
    }
    const id = comparable(server.project.folder);
    if (server.state !== "running" || this.baseOf(server) || this.looking.has(id)) return;
    this.looking.add(id);
    try {
      const branch = await this.threads.branchKey(server);
      if (branch === undefined || this.offered.has(branch)) return;
      this.offered.add(branch);
      const number = await this.threads.quietPullRequest(server);
      // Started meanwhile, or no pull request.
      if (number === undefined || this.baseOf(server)) return;
      const message = `Ascribe: this branch has pull request #${number}. Start Review?`;
      this.offerLog.push(message);
      void vscode.window
        .showInformationMessage(message, "Start Review", "Don't Offer Again")
        .then((choice) => {
          if (choice === "Don't Offer Again") return this.state.update(NO_OFFER, true);
          // The notification can wait in the notification center: check again.
          if (choice && !this.baseOf(server)) return this.startCommand(this.running(server));
          return undefined;
        });
    } finally {
      this.looking.delete(id);
    }
  }

  /** The base's name for a project: the pull request's base branch by its name, when it's that. */
  baseNameOf(server: ProjectServer, base: BaseInfo): string {
    const connection = this.threads.connection(server);
    return baseName(
      base,
      connection?.state === "on" ? connection.session.pullRequest.baseRefName : undefined,
    );
  }

  /** Whether an Ascribe page or the preview is active: where the status bar item shows. */
  private isAscribeContext(): boolean {
    if (this.host.previewActive()) return true;
    const document = vscode.window.activeTextEditor?.document;
    return (
      document !== undefined &&
      document.uri.scheme === "file" &&
      document.languageId === "markdown" &&
      this.projects.serverFor(document.uri) !== undefined
    );
  }

  /** The project review acts on: the preview's, when it's active, else the active file's. */
  private target(): ProjectServer | undefined {
    const previewed = this.host.previewedDocument();
    if (this.host.previewActive() && previewed) return this.projects.serverFor(previewed.uri);
    const active = vscode.window.activeTextEditor?.document;
    if (active?.uri.scheme === "file") {
      const server = this.projects.serverFor(active.uri);
      if (server) return server;
    }
    return previewed && this.projects.serverFor(previewed.uri);
  }

  /** The project a command acts on, with its server running; says why not when there's none. */
  private runningTarget(): ProjectServer | undefined {
    const server = this.target();
    if (!server) {
      void vscode.window.showInformationMessage(
        "Ascribe: open a page of the project you want to review first.",
      );
      return undefined;
    }
    return this.running(server);
  }

  /** The server, when it's running; says why not when it isn't. */
  private running(server: ProjectServer): ProjectServer | undefined {
    if (server.state !== "running") {
      void vscode.window.showInformationMessage(
        `Ascribe: the language server for ${this.projects.name(server.project)} isn't running. Open one of its pages first, then start the review.`,
      );
      return undefined;
    }
    return server;
  }

  private async startCommand(server: ProjectServer | undefined): Promise<void> {
    if (!server) return;
    // The pull request decides the default base, and its threads show once
    // review is on. Signing in to GitHub is asked for here, never sooner.
    const threads = await vscode.window.withProgress(
      { location: vscode.ProgressLocation.Window, title: "Ascribe: looking for the pull request" },
      () => this.threads.connect(server, { interactive: true }),
    );
    const pr = threads.state === "on" ? threads.session.pullRequest : undefined;
    const choice = await vscode.window.showQuickPick(
      [
        ...(pr && threads.state === "on"
          ? [
              {
                label: `The base of pull request #${pr.number}`,
                detail: `${threads.base}, from where this branch left it`,
                base: threads.base as string | undefined,
              },
            ]
          : []),
        {
          label: "The default branch",
          detail: "origin/HEAD, origin/main, or main, from where this branch left it",
          base: undefined as string | undefined,
        },
        { label: "A revision you type…", detail: "a branch, tag, or commit", base: "" },
      ],
      {
        title: "Start Review",
        placeHolder: `Review ${this.projects.name(server.project)}'s changes against…`,
      },
    );
    if (!choice) {
      // Not started after all: no threads without review.
      if (!this.baseOf(server)) this.threads.disconnect(server);
      return;
    }
    let base = choice.base;
    if (base === "") {
      base = await vscode.window.showInputBox({
        title: "Start Review",
        prompt: "A branch, tag, or commit to compare with, from where this branch left it",
        placeHolder: "main",
        validateInput: (value) => (value.trim() === "" ? "Type a branch, tag, or commit." : null),
      });
      if (base === undefined) {
        if (!this.baseOf(server)) this.threads.disconnect(server);
        return;
      }
      base = base.trim();
    }
    const result = await vscode.window.withProgress(
      { location: vscode.ProgressLocation.Window, title: "Ascribe: reading the review base" },
      () => this.start(server, base),
    );
    if (!result.base) {
      if (!this.baseOf(server)) this.threads.disconnect(server);
      void vscode.window.showErrorMessage(`Ascribe: ${result.problem ?? "review couldn't start."}`);
      return;
    }
    const changes = await this.changes(server);
    const count = changes?.pages.length ?? 0;
    const pages = count === 1 ? "1 page changed" : `${count} pages changed`;
    void vscode.window.showInformationMessage(
      `Ascribe: review started against ${this.baseNameOf(server, result.base)}. ${count === 0 ? "No page changed" : pages}.`,
    );
  }

  private async stopCommand(): Promise<void> {
    const server = this.target();
    if (!server || !this.baseOf(server)) {
      void vscode.window.showInformationMessage("Ascribe: review is off for this project.");
      return;
    }
    await this.stop(server);
  }

  private async refreshCommand(): Promise<void> {
    const server = this.runningTarget();
    if (!server) return;
    if (!this.baseOf(server)) {
      void vscode.window.showInformationMessage("Ascribe: review is off for this project.");
      return;
    }
    const connection = await vscode.window.withProgress(
      { location: vscode.ProgressLocation.Window, title: "Ascribe: reading the review comments" },
      () => this.threads.refresh(server),
    );
    this.host.refresh();
    if (connection.state === "none") {
      void vscode.window.showInformationMessage(
        "Ascribe: there's no open pull request on GitHub for this branch, so there are no comments to show.",
      );
    }
  }

  private async changedPagesCommand(): Promise<void> {
    const server = this.runningTarget();
    if (!server) return;
    const base = this.baseOf(server);
    if (!base) {
      const start = await vscode.window.showInformationMessage(
        `Ascribe: review is off for ${this.projects.name(server.project)}. Start it to see the changed pages.`,
        "Start Review",
      );
      if (start) await this.startCommand(server);
      return;
    }
    const result = await this.changes(server);
    if (!result) return;
    type Item = vscode.QuickPickItem & { page?: ChangedPage; stop?: true };
    const items: Item[] = result.pages.map((page) => ({
      label: page.title ?? page.path,
      description: page.title === null ? "" : page.path,
      detail: pageDetail(page),
      page,
    }));
    if (items.length === 0) items.push({ label: "No page changed", detail: "" });
    items.push(
      { label: "", kind: vscode.QuickPickItemKind.Separator },
      { label: "Ascribe: Stop Review", stop: true },
    );
    const picked = await vscode.window.showQuickPick(items, {
      title: `Changed pages against ${this.baseNameOf(server, base)} (build: ${result.build})`,
      placeHolder: "Open a changed page and its preview",
      matchOnDescription: true,
    });
    if (!picked) return;
    if (picked.stop) {
      await this.stop(server);
      return;
    }
    const page = picked.page;
    if (!page || result.contentRoot === null) return;
    if (page.status === "removed") {
      void vscode.window.showInformationMessage(
        `Ascribe: ${page.path} was removed, so there is no page to open.`,
      );
      return;
    }
    await this.host.showPage(vscode.Uri.file(fromContentPath(result.contentRoot, page.path)));
  }

  private async start(server: ProjectServer, base: string | undefined): Promise<SetBaseResult> {
    const result = await this.setBase(server, base);
    if (result.base) {
      this.bases.set(comparable(server.project.folder), result.base);
      // Reviewed: from now on, a branch with a pull request is offered
      // review, but not this one again this session.
      const branch = await this.threads.branchKey(server);
      if (branch !== undefined) this.offered.add(branch);
      if (!this.state.get<boolean>(STARTED_ONCE, false)) void this.state.update(STARTED_ONCE, true);
      this.update();
      this.host.refresh();
    }
    return result;
  }

  private async stop(server: ProjectServer): Promise<void> {
    this.bases.delete(comparable(server.project.folder));
    this.threads.disconnect(server);
    this.update();
    this.host.refresh();
    if (server.state === "running") {
      // Frees the base in the server.
      await server.request(SET_BASE_REQUEST, { base: null }).catch(() => undefined);
    }
  }

  private async setBase(server: ProjectServer, base: string | undefined): Promise<SetBaseResult> {
    try {
      return (await server.request(
        SET_BASE_REQUEST,
        base === undefined ? {} : { base },
      )) as SetBaseResult;
    } catch (error) {
      return { base: null, problem: error instanceof Error ? error.message : String(error) };
    }
  }

  private async changes(server: ProjectServer): Promise<ChangesResult | undefined> {
    const build = this.host.previewBuild(server.project.folder);
    try {
      return (await server.request(
        CHANGES_REQUEST,
        build === undefined ? {} : { build },
      )) as ChangesResult;
    } catch (error) {
      server.reportFeatureError("review", error);
      return undefined;
    }
  }

  dispose(): void {
    for (const d of this.disposables) d.dispose();
    this.disposables = [];
  }
}
