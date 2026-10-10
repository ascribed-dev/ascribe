import * as path from "node:path";
import { statSync } from "node:fs";
import * as vscode from "vscode";
import type { PromptRequest } from "@ascribed/review/github";
import type { Prompts } from "../actions/promptAgent.js";
import type { ProjectServer } from "../client.js";
import { samePath } from "../projects.js";
import type { ProjectRegistry } from "../registry.js";
import type { ChosenBuilds } from "../ui/chosenBuild.js";
import { shellHtml } from "./html.js";
import type {
  FromWebview,
  ImageReport,
  PageFrontmatter,
  PreviewParams,
  PreviewResult,
  RenderReport,
  ReviewView,
  ThreadsReport,
  ToWebview,
  WebviewAsset,
} from "./protocol.js";
import { canonicalReference, isExternal, splitFragment } from "./refs.js";
import { ReviewController, type ReviewApi } from "./review.js";
import { SourceComments, type SourceThreadRecord } from "./sourceComments.js";
import { baseCommit, baseName, causes, fromContentPath, parseSource } from "./reviewText.js";
import { previewProblems } from "./routing.js";
import { findDevServer, noDevServerMessage, pageUrl, sectionAt } from "./site.js";

/** The custom request the language server answers (`crates/ascribe-lsp/README.md`). */
const PREVIEW_REQUEST = "ascribe/preview";

const VIEW_TYPE = "ascribe.preview";

/** How long after the last change the preview asks for a new render. */
const DEBOUNCE_MS = 100;

/** How long after the last change on disk (an image replaced, a file created). */
const DISK_DEBOUNCE_MS = 250;

/** The schemes a link in the preview may open outside VS Code. */
const EXTERNAL_SCHEMES = new Set(["http:", "https:", "mailto:"]);

/** One render, as the tests see it. */
export interface RenderRecord {
  seq: number;
  /** The document and its version the render is for. */
  document: string;
  version: number;
  /** When the extension sent it and when the webview said it had drawn it (`Date.now()`). */
  sentAt: number;
  drawnAt?: number;
  result: PreviewResult;
  assets: WebviewAsset[];
  report?: RenderReport;
  images?: ImageReport[];
}

/** What the extension returns for tests. */
export interface PreviewApi {
  isOpen(): boolean;
  /** The build the picker shows. */
  build(): string | undefined;
  /** The webview's HTML shell. */
  shell(): string | undefined;
  /** The directories the webview may read. */
  localResourceRoots(): string[];
  /** Every render sent so far. */
  renders(): readonly RenderRecord[];
  /** The ids of the headings the preview was told to scroll to (a link to one), in order. */
  reveals(): readonly string[];
  /** The blocks the preview scrolled to for the editor: the line asked for and the block's anchor. */
  lineReveals(): readonly { line: number; source: string }[];
  /** Waits until a render that has been drawn satisfies `predicate`. */
  whenDrawn(
    description: string,
    predicate: (render: RenderRecord) => boolean,
    timeout?: number,
  ): Promise<RenderRecord>;
  /** Handles a message as if the webview had posted it. */
  receive(message: FromWebview): Promise<void>;
  /** Selects a build as the picker does. */
  selectBuild(name: string): void;
  /** Review. */
  review: ReviewApi;
  /** What the overlay drew, each time it drew the threads. */
  threadsDrawn(): readonly ThreadsReport[];
  /** The review threads the source editor shows. */
  sourceThreads(): SourceThreadRecord[];
  /** The site preview. */
  site: SiteApi;
}

/** The site preview, for tests. */
interface SiteApi {
  /** From now on, records the addresses it would open in the browser instead of opening them. */
  captureExternal(): void;
  /** The addresses opened in the browser, in order. */
  opened(): readonly string[];
  /** The messages it showed, in order. */
  messages(): readonly string[];
  /** Which view the preview panel shows. */
  surface(): "page" | "site";
  /** Chooses Page or Site, as the panel's switch does. */
  selectSurface(surface: "page" | "site"): Promise<void>;
  /** What the panel's Site view showed: an address, or a problem. */
  shown(): readonly ({ url: string } | { problem: string })[];
  /** The addresses the panel's frame loaded, in order. */
  framed(): readonly string[];
}

/**
 * The preview panel: one webview beside the editor that shows the page of the
 * active Ascribe document, rendered by the language server of the project
 * that owns it (`ascribe/preview`) through the site emitter, and drawn with
 * the element library.
 */
export class PreviewController implements vscode.Disposable {
  private panel: vscode.WebviewPanel | undefined;
  private disposables: vscode.Disposable[] = [];
  private panelDisposables: vscode.Disposable[] = [];
  private ready = false;
  private document: vscode.TextDocument | undefined;
  private timer: NodeJS.Timeout | undefined;
  private refreshing = false;
  private again = false;
  private seq = 0;
  private roots: string[] = [];
  private contentRoot: string | undefined;
  private assetRoots: string[] = [];
  private watchers: vscode.FileSystemWatcher[] = [];
  /** The last render, and the folder of the project it is from (none for a file outside every project). */
  private latest:
    | {
        message: Extract<ToWebview, { type: "render" }>;
        result: PreviewResult;
        folder: string | undefined;
      }
    | undefined;
  private log: RenderRecord[] = [];
  private revealLog: string[] = [];
  private lineRevealLog: { line: number; source: string }[] = [];
  /** Until when the editor's scrolling is the preview's doing, not the author's. */
  private editorQuietUntil = 0;
  private waiters: (() => void)[] = [];
  /** The page to go to the first change of once it's drawn: one opened from review's next-page offer. */
  private firstChangeOf: string | undefined;
  /** The thread to go to once its page is drawn: one opened from another page's list. */
  private threadToGo: { file: string; threadId: string } | undefined;
  private threadsLog: ThreadsReport[] = [];
  private captureExternal = false;
  private externalLog: string[] = [];
  private siteMessages: string[] = [];
  /** Which view the panel shows. */
  private surface: "page" | "site" = "page";
  /** The origin the panel may frame: the site preview's, once it's shown. */
  private frameOrigin: string | undefined;
  /** What the Site view shows, sent again when the webview reloads. */
  private siteMessage: Extract<ToWebview, { type: "surface" }> | undefined;
  /** The document the Site view last showed. */
  private siteShownFor: string | undefined;
  private siteLog: ({ url: string } | { problem: string })[] = [];
  private framedLog: string[] = [];
  /** Review: its base, its changed pages, and the pull request's threads. */
  readonly review: ReviewController;
  private readonly sourceComments: SourceComments;

  constructor(
    private readonly context: vscode.ExtensionContext,
    private readonly projects: ProjectRegistry,
    /** The build each project is looking at, which the status bar shows and sets too. */
    private readonly builds: ChosenBuilds,
    /** Prompt agent: where review's prompts are built and delivered. */
    private readonly prompts: Prompts,
  ) {
    this.review = new ReviewController(
      projects,
      {
        previewedDocument: () => (this.panel ? this.document : undefined),
        previewActive: () => this.panel?.active ?? false,
        previewBuild: (folder) => this.builds.get(folder),
        refresh: () => {
          if (this.panel) this.schedule(0);
        },
        showPage: (uri) => this.showPage(uri),
      },
      context.workspaceState,
    );
    this.sourceComments = new SourceComments(projects, this.review.threads, prompts);
  }

  /** Registers the commands, the listeners, and the panel serializer. */
  register(): void {
    this.review.register();
    this.sourceComments.register();
    this.disposables.push(
      this.sourceComments,
      this.review.threads.onDidChange(({ server, origin }) => {
        if (!this.panel || this.server() !== server) return;
        // The overlay reads the threads again after its own actions.
        if (origin === "source") this.post({ type: "threadsChanged" });
        else if (origin === "refresh") this.schedule(0);
      }),
      vscode.commands.registerCommand("ascribe.openPreview", () => this.open()),
      vscode.commands.registerCommand("ascribe.openPagePreview", () =>
        this.open(vscode.ViewColumn.Active),
      ),
      vscode.commands.registerCommand("ascribe.openSitePreview", () => this.openSite()),
      vscode.commands.registerCommand("ascribe.selectPreviewBuild", () => this.pickBuild()),
      ...this.watchDevFiles(),
      vscode.window.registerWebviewPanelSerializer(VIEW_TYPE, {
        deserializeWebviewPanel: async (panel) => {
          this.attach(panel);
          this.followActiveEditor();
          this.schedule(0);
        },
      }),
      vscode.window.onDidChangeActiveTextEditor(() => {
        if (this.panel && this.followActiveEditor()) this.schedule(0);
      }),
      vscode.workspace.onDidChangeTextDocument((event) => {
        if (this.panel && affectsProject(event.document)) this.schedule(DEBOUNCE_MS);
      }),
      vscode.workspace.onDidSaveTextDocument((document) => {
        if (this.panel && affectsProject(document)) this.schedule(0);
      }),
      vscode.window.onDidChangeTextEditorSelection((event) => {
        if (this.panel && event.textEditor.document === this.document)
          this.followCursor(event.textEditor);
      }),
      vscode.window.onDidChangeTextEditorVisibleRanges((event) => {
        if (this.panel && event.textEditor.document === this.document)
          this.followScroll(event.textEditor);
      }),
      // The previewed project's server started or restarted: it has the
      // project's current state now. Other projects' servers don't matter.
      this.projects.onDidStart((server) => {
        if (this.panel && this.server() === server) this.schedule(0);
      }),
      // Review says how many errors the project has: keep the count current.
      vscode.languages.onDidChangeDiagnostics(() => {
        const review = this.latest?.message.review;
        const server = this.server();
        if (this.panel && review && server && server.errorCount() !== review.errors)
          this.schedule(DEBOUNCE_MS);
      }),
      // The previewed project's build was chosen here or elsewhere.
      this.builds.onDidChange((folder) => {
        const server = this.server();
        if (this.panel && server && samePath(server.project.folder, folder)) this.schedule(0);
      }),
      // A project appeared or went away: the file may have another owner.
      this.projects.onDidChangeProjects(() => {
        if (this.panel) this.schedule(0);
      }),
    );
  }

  /**
   * Keeps `ascribe.devServer` (which shows Open Site Preview in a page's title
   * bar) true while some project has a `dev.json`. Whether its server answers
   * is checked when the command runs.
   */
  private watchDevFiles(): vscode.Disposable[] {
    const pattern = "**/.ascribe/dev.json";
    const known = new Set<string>();
    const update = (): void =>
      void vscode.commands.executeCommand("setContext", "ascribe.devServer", known.size > 0);
    const add = (uri: vscode.Uri): void => {
      known.add(uri.fsPath);
      update();
    };
    const watcher = vscode.workspace.createFileSystemWatcher(pattern);
    void vscode.workspace.findFiles(pattern, "**/node_modules/**", 50).then((uris) => {
      for (const uri of uris) known.add(uri.fsPath);
      update();
    });
    return [
      watcher,
      watcher.onDidCreate(add),
      watcher.onDidChange(add),
      watcher.onDidDelete((uri) => {
        known.delete(uri.fsPath);
        update();
      }),
    ];
  }

  /** The test surface. */
  get api(): PreviewApi {
    return {
      isOpen: () => this.panel !== undefined,
      build: () => this.latest?.message.build,
      shell: () => this.panel?.webview.html,
      localResourceRoots: () => [...this.roots],
      renders: () => this.log,
      reveals: () => this.revealLog,
      lineReveals: () => this.lineRevealLog,
      whenDrawn: (description, predicate, timeout = 30_000) =>
        this.waitForDrawn(description, predicate, timeout),
      receive: (message) => this.receive(message),
      selectBuild: (name) => this.chooseBuild(name),
      review: this.review.api,
      threadsDrawn: () => this.threadsLog,
      sourceThreads: () => this.sourceComments.records(),
      site: {
        captureExternal: () => {
          this.captureExternal = true;
        },
        opened: () => this.externalLog,
        messages: () => this.siteMessages,
        surface: () => this.surface,
        selectSurface: (surface) => this.selectSurface(surface),
        shown: () => this.siteLog,
        framed: () => this.framedLog,
      },
    };
  }

  /** Opens the preview, beside the editor or in its place, for the active Ascribe document. */
  async open(column = vscode.ViewColumn.Beside): Promise<void> {
    const beside = column === vscode.ViewColumn.Beside;
    if (this.panel) {
      this.panel.reveal(beside ? undefined : column, beside);
      this.followActiveEditor();
      this.schedule(0);
      return;
    }
    this.followActiveEditor();
    // The content root is needed before the panel exists: it's one of the
    // directories the webview may read, and is fixed when the panel is made.
    const first = this.document && (await this.query(this.document));
    const panel = vscode.window.createWebviewPanel(
      VIEW_TYPE,
      panelTitle(this.document),
      { viewColumn: column, preserveFocus: beside },
      {
        ...this.webviewOptions(first?.contentRoot ?? undefined, first?.assetRoots ?? []),
        retainContextWhenHidden: true,
      },
    );
    this.attach(panel, first?.contentRoot ?? undefined, first?.assetRoots ?? []);
    this.schedule(0);
  }

  private attach(
    panel: vscode.WebviewPanel,
    contentRoot?: string,
    assetRoots: string[] = [],
  ): void {
    this.panel = panel;
    this.ready = false;
    this.latest = undefined;
    // A new panel shows the page first.
    this.surface = "page";
    this.siteMessage = undefined;
    this.siteShownFor = undefined;
    panel.iconPath = vscode.Uri.joinPath(this.context.extensionUri, "media", "preview.svg");
    panel.webview.options = this.webviewOptions(contentRoot, assetRoots);
    this.contentRoot = contentRoot;
    this.assetRoots = assetRoots;
    this.setShell();
    this.panelDisposables.push(
      panel.webview.onDidReceiveMessage((message: FromWebview) => void this.receive(message)),
      panel.onDidDispose(() => this.detach()),
      panel.onDidChangeViewState(() => {
        this.review.update();
        if (panel.active) void this.review.recheck();
      }),
    );
    void vscode.commands.executeCommand("setContext", "ascribe.previewOpen", true);
  }

  private detach(): void {
    if (this.timer) clearTimeout(this.timer);
    this.timer = undefined;
    for (const watcher of this.watchers) watcher.dispose();
    this.watchers = [];
    for (const d of this.panelDisposables) d.dispose();
    this.panelDisposables = [];
    this.panel = undefined;
    this.ready = false;
    this.latest = undefined;
    void vscode.commands.executeCommand("setContext", "ascribe.previewOpen", false);
    this.review.update();
  }

  /**
   * The webview's options. It may read the extension's webview files, the
   * project's content root, and the directory of each asset the page uses
   * that is outside the content root (a project may keep assets anywhere but
   * its output directory, and the server names them in `assetRoots`), and nothing else: never the workspace
   * or the project root.
   */
  // The content root and the directories of the assets outside it, never the project root.
  private webviewOptions(
    contentRoot: string | undefined,
    assetRoots: string[],
  ): vscode.WebviewOptions {
    const roots = [vscode.Uri.joinPath(this.context.extensionUri, "dist", "webview")];
    if (contentRoot) roots.push(vscode.Uri.file(contentRoot));
    for (const root of assetRoots) roots.push(vscode.Uri.file(root));
    this.roots = roots.map((uri) => uri.fsPath);
    return { enableScripts: true, localResourceRoots: roots };
  }

  private setShell(): void {
    if (!this.panel) return;
    const webview = this.panel.webview;
    const file = (name: string): string =>
      webview
        .asWebviewUri(vscode.Uri.joinPath(this.context.extensionUri, "dist", "webview", name))
        .toString();
    this.ready = false;
    webview.html = shellHtml({
      cspSource: webview.cspSource,
      elementsScript: file("elements.js"),
      elementsStyle: file("elements.css"),
      marksStyle: file("marks.css"),
      previewScript: file("preview.js"),
      previewStyle: file("preview.css"),
      ...(this.frameOrigin === undefined ? {} : { frameOrigin: this.frameOrigin }),
    });
  }

  /** Makes the active Ascribe document the one previewed. Whether it changed. */
  // The preview follows the active Ascribe editor.
  private followActiveEditor(): boolean {
    const editor = vscode.window.activeTextEditor;
    if (!editor || !isPreviewable(editor.document) || editor.document === this.document) {
      return false;
    }
    this.document = editor.document;
    if (this.panel) this.panel.title = panelTitle(this.document);
    return true;
  }

  private schedule(delay: number): void {
    if (this.timer) clearTimeout(this.timer);
    this.timer = setTimeout(() => {
      this.timer = undefined;
      void this.refresh();
    }, delay);
  }

  /** The server of the project that owns the previewed document. */
  private server(): ProjectServer | undefined {
    return this.document && this.projects.serverFor(this.document.uri);
  }

  /**
   * Asks the server of the project that owns a document for its page, in the
   * build chosen in that project, starting the server if it hasn't started.
   */
  private async query(
    document: vscode.TextDocument,
    buildOverride?: string,
  ): Promise<PreviewResult | undefined> {
    const server = await this.projects.ensureStartedFor(document.uri);
    if (!server) return undefined;
    const params: PreviewParams = { textDocument: { uri: document.uri.toString() } };
    const build = buildOverride ?? this.builds.get(server.project.folder);
    if (build !== undefined) params.build = build;
    if (this.review.baseOf(server)) params.review = true;
    try {
      return (await server.request(PREVIEW_REQUEST, params)) as PreviewResult;
    } catch {
      return undefined;
    }
  }

  /** Asks the server for the current render and sends it. One at a time; a change during a render asks again. */
  private async refresh(): Promise<void> {
    if (this.refreshing) {
      this.again = true;
      return;
    }
    this.refreshing = true;
    try {
      do {
        this.again = false;
        await this.renderOnce();
      } while (this.again);
    } finally {
      this.refreshing = false;
    }
  }

  private async renderOnce(): Promise<void> {
    const panel = this.panel;
    const document = this.document;
    if (!panel) return;
    let result: PreviewResult | undefined;
    if (document) {
      // The language client sends a change before a request that follows it,
      // but the server may answer from an older version if the notification
      // is still on its way; the answer says which version it used.
      for (let attempt = 0; attempt < 20; attempt++) {
        result = await this.query(document);
        if (
          !result ||
          result.documentVersion === null ||
          result.documentVersion >= document.version
        ) {
          break;
        }
        await new Promise((resolve) => setTimeout(resolve, 25));
      }
      // Review is on, and the server has no base: it restarted. Set it again.
      const server = this.projects.serverFor(document.uri);
      if (result?.page && !result.review && server && this.review.baseOf(server)) {
        if (await this.review.restore(server)) result = await this.query(document);
      }
    }
    if (!this.panel || panel !== this.panel) return;
    const server = document && this.projects.serverFor(document.uri);
    const folder = server?.project.folder;
    // A build the content model no longer has (its file changed): back to the editor's.
    const chosen = folder === undefined ? undefined : this.builds.get(folder);
    if (
      result &&
      folder !== undefined &&
      chosen !== undefined &&
      result.builds.length > 0 &&
      !result.builds.some((b) => b.name === chosen)
    ) {
      this.builds.set(folder, undefined);
      this.again = true;
      return;
    }
    const problems = previewProblems({
      file: document?.uri.fsPath,
      project: server && this.projects.name(server.project),
      state: server?.state,
      result,
      show: (file) => vscode.workspace.asRelativePath(file),
    });
    result = {
      ...(result ?? {
        build: "",
        builds: [],
        projectRoot: null,
        contentRoot: null,
        assetRoots: [],
        documentVersion: null,
        page: null,
        review: null,
      }),
      problems,
    };
    this.followRoots(result.contentRoot ?? undefined, result.assetRoots);
    const assets = this.assetUris(result);
    const base = server && this.review.baseOf(server);
    let review: ReviewView | null = null;
    if (result.page && document && base && result.review) {
      const goToFirst = this.firstChangeOf === document.uri.fsPath;
      if (goToFirst) this.firstChangeOf = undefined;
      review = {
        base: server
          ? this.review.baseNameOf(server, result.review.base)
          : baseName(result.review.base),
        commit: baseCommit(result.review.base),
        page: result.review.changes,
        wasHtml: result.review.wasHtml,
        causes: causes(result.review.changes, result),
        goToFirst,
        threads: null,
        errors: server?.errorCount() ?? 0,
      };
      if (server) {
        const goTo = this.threadToGo?.file === document.uri.fsPath ? this.threadToGo : undefined;
        if (goTo) this.threadToGo = undefined;
        review.threads = this.review.threads.view(server, goTo?.threadId ?? null);
      }
    }
    const message: Extract<ToWebview, { type: "render" }> = {
      type: "render",
      seq: ++this.seq,
      path: result.page?.path ?? null,
      build: result.build,
      builds: result.builds,
      title: result.page?.title ?? null,
      formattedTitle: result.page?.formattedTitle ?? null,
      available: (result.page?.frontmatter as PageFrontmatter | undefined)?.available ?? [],
      html: result.page?.html ?? null,
      assets,
      problems,
      review,
    };
    this.latest = { message, result, folder };
    this.log.push({
      seq: message.seq,
      document: document?.uri.toString() ?? "",
      version: document?.version ?? -1,
      sentAt: Date.now(),
      result,
      assets,
    });
    if (this.log.length > 200) this.log.splice(0, this.log.length - 200);
    if (this.ready) void panel.webview.postMessage(message);
    // The Site view follows the active file as the page does.
    if (this.surface === "site" && this.siteShownFor !== document?.uri.toString()) {
      void this.showSite();
    }
  }

  /** The content root changed (a new `ascribe.toml`, another project): the webview may read the new one. */
  private followRoots(contentRoot: string | undefined, assetRoots: string[]): void {
    if (!this.panel) return;
    const same =
      contentRoot === this.contentRoot &&
      assetRoots.length === this.assetRoots.length &&
      assetRoots.every((root, i) => root === this.assetRoots[i]);
    if (same) return;
    this.contentRoot = contentRoot;
    this.assetRoots = assetRoots;
    this.panel.webview.options = this.webviewOptions(contentRoot, assetRoots);
    // Reloading the document makes the new roots take effect; the webview
    // says `ready` again and gets the latest render.
    this.setShell();
    this.watchDisk();
  }

  private watchDisk(): void {
    for (const watcher of this.watchers) watcher.dispose();
    this.watchers = [];
    // Images and other files change on disk without an open document: watch
    // every directory the preview reads.
    const roots = [...(this.contentRoot ? [this.contentRoot] : []), ...this.assetRoots];
    const changed = (): void => {
      if (this.panel) this.schedule(DISK_DEBOUNCE_MS);
    };
    for (const root of roots) {
      const watcher = vscode.workspace.createFileSystemWatcher(
        new vscode.RelativePattern(vscode.Uri.file(root), "**/*"),
      );
      watcher.onDidCreate(changed);
      watcher.onDidChange(changed);
      watcher.onDidDelete(changed);
      this.watchers.push(watcher);
    }
  }

  /** The webview URL of each servable asset, with its modification time so a replaced image is fetched again. */
  private assetUris(result: PreviewResult): WebviewAsset[] {
    const webview = this.panel?.webview;
    if (!webview || !result.page) return [];
    const assets: WebviewAsset[] = [];
    for (const asset of result.page.assets) {
      if (!asset.servable || asset.kind !== "image") continue;
      let uri = webview.asWebviewUri(vscode.Uri.file(asset.path));
      try {
        uri = uri.with({ query: `v=${Math.trunc(statSync(asset.path).mtimeMs)}` });
      } catch {
        // A file that isn't there: the image shows as broken, and the diagnostics say why.
      }
      assets.push({ reference: asset.reference, uri: uri.toString() });
    }
    return assets;
  }

  private async receive(message: FromWebview): Promise<void> {
    switch (message.type) {
      case "ready": {
        this.ready = true;
        if (this.latest) void this.panel?.webview.postMessage(this.latest.message);
        else this.schedule(0);
        if (this.surface === "site" && this.siteMessage) this.post(this.siteMessage);
        if (this.contentRoot && this.watchers.length === 0) this.watchDisk();
        return;
      }
      case "build":
        this.chooseBuild(message.name);
        return;
      case "open":
        await this.openLink(message.href);
        return;
      case "showOutput":
        this.server()?.showOutput();
        return;
      case "showProblems":
        void vscode.commands.executeCommand("workbench.actions.view.problems");
        return;
      case "rendered": {
        const record = this.log.find((r) => r.seq === message.seq);
        if (record) {
          record.drawnAt = Date.now();
          record.report = message.report;
        }
        this.wake();
        if (record && this.latest?.message.seq === message.seq) {
          const editor = vscode.window.visibleTextEditors.find((e) => e.document === this.document);
          if (editor) this.followCursor(editor);
        }
        return;
      }
      case "images": {
        const record = this.log.find((r) => r.seq === message.seq);
        if (record) record.images = message.images;
        this.wake();
        return;
      }
      case "revealedLine":
        this.lineRevealLog.push({ line: message.line, source: message.source });
        this.wake();
        return;
      case "scrolled":
        this.scrollEditor(message.line);
        return;
      case "openLine":
        await this.openLine(message.line);
        return;
      case "openSource":
        await this.openSource(message.source);
        return;
      case "openFile":
        await this.showFile(vscode.Uri.file(message.path));
        return;
      case "atEnd": {
        const server = this.server();
        const page = this.latest?.result.page;
        if (!server || !page) return;
        const next = await this.review.next(server, page.path);
        const reply: ToWebview = { type: "nextPage", ...next };
        void this.panel?.webview.postMessage(reply);
        return;
      }
      case "openPage": {
        const contentRoot = this.latest?.result.contentRoot;
        if (contentRoot)
          await this.showPage(vscode.Uri.file(fromContentPath(contentRoot, message.path)));
        return;
      }
      case "threads":
        await this.threadsRequest(message);
        return;
      case "openThread": {
        const contentRoot = this.latest?.result.contentRoot;
        if (!contentRoot) return;
        const uri = vscode.Uri.file(fromContentPath(contentRoot, message.path));
        if (this.document && uri.fsPath === this.document.uri.fsPath) {
          this.post({ type: "goToThread", threadId: message.threadId });
          return;
        }
        this.threadToGo = { file: uri.fsPath, threadId: message.threadId };
        await this.showPage(uri, false);
        return;
      }
      case "notify":
        vscode.window.setStatusBarMessage(`$(comment-discussion) ${message.message}`, 8000);
        return;
      case "signIn":
      case "useGh":
      case "refreshThreads": {
        const server = this.server();
        if (!server || !this.review.baseOf(server)) return;
        const threads = this.review.threads;
        if (message.type === "signIn") await threads.signIn(server);
        else if (message.type === "useGh") await threads.useGh(server);
        else await threads.refresh(server);
        return;
      }
      case "git": {
        const server = this.server();
        await vscode.commands.executeCommand(`git.${message.command}`).then(
          () => undefined,
          (error: unknown) =>
            void vscode.window.showErrorMessage(
              `Ascribe: couldn't ${message.command}. ${error instanceof Error ? error.message : String(error)}`,
            ),
        );
        if (server && this.review.baseOf(server)) await this.review.threads.refresh(server);
        return;
      }
      case "surface":
        await this.selectSurface(message.surface);
        return;
      case "siteShown":
        this.framedLog.push(message.url);
        this.wake();
        return;
      case "threadsDrawn":
        this.threadsLog.push(message.report);
        if (this.threadsLog.length > 200) this.threadsLog.splice(0, this.threadsLog.length - 200);
        this.wake();
        return;
    }
  }

  /** Answers one of the overlay's requests, for the page the preview shows. */
  private async threadsRequest(message: Extract<FromWebview, { type: "threads" }>): Promise<void> {
    const server = this.server();
    const shown = this.latest?.message;
    const page =
      shown?.path === null || shown === undefined
        ? undefined
        : { build: shown.build, path: shown.path };
    let reply: ToWebview;
    try {
      if (!server) throw new Error("There's no project for this page.");
      const result =
        message.method === "promptAgent"
          ? await this.promptAgent(server, page, message.params["request"] as PromptRequest)
          : await this.review.threads.handle(
              server,
              page,
              message.method,
              message.params,
              "preview",
            );
      reply = { type: "threadsResult", id: message.id, result };
    } catch (error) {
      const code =
        typeof error === "object" && error !== null && "code" in error
          ? String((error as { code: unknown }).code)
          : undefined;
      reply = {
        type: "threadsResult",
        id: message.id,
        error: {
          message: error instanceof Error ? error.message : String(error),
          ...(code === undefined ? {} : { code }),
        },
      };
    }
    this.post(reply);
  }

  /**
   * Builds the prompt the overlay or the review header asked for and delivers
   * it: a thread's from the pull request, a page's changes or a fragment's
   * reach from the server.
   */
  private async promptAgent(
    server: ProjectServer,
    page: { build: string; path: string } | undefined,
    request: PromptRequest,
  ): Promise<null> {
    let answer: { prompt: string; aboutUnsaved: boolean } | undefined;
    if (request.kind === "thread" || request.kind === "open-threads") {
      answer = await this.review.threads.prompt(server, request);
    } else {
      const document = this.document;
      if (!document || !page) throw new Error("There's no page in the preview.");
      try {
        answer = await this.prompts.ask(
          server,
          request.kind === "page-changes"
            ? {
                kind: "pageChanges",
                textDocument: { uri: document.uri.toString() },
                build: page.build,
                unsaved: [],
              }
            : {
                kind: "fragmentReach",
                textDocument: { uri: document.uri.toString() },
                build: page.build,
                fragment: request.fragment,
                unsaved: [],
              },
        );
      } catch (error) {
        server.log(`Building the agent prompt failed: ${String(error)}`);
        throw new Error("The project's server couldn't build it. Its output has the details.");
      }
    }
    if (!answer) {
      throw new Error(
        request.kind === "page-changes"
          ? "Nothing on this page changed."
          : request.kind === "fragment-reach"
            ? "No page changed through that file."
            : "No review comment is open.",
      );
    }
    await this.prompts.deliver(answer.prompt, answer.aboutUnsaved);
    return null;
  }

  private post(message: ToWebview): void {
    if (this.panel && this.ready) void this.panel.webview.postMessage(message);
  }

  /** Chooses a build in the project of the last render, whose builds the picker lists. */
  private chooseBuild(name: string): void {
    const folder = this.latest?.folder;
    if (folder === undefined) return;
    this.builds.choose(folder, name, this.editorBuild());
    this.schedule(0);
  }

  private editorBuild(): string | undefined {
    return this.latest?.result.builds.find((b) => b.editor)?.name;
  }

  private async pickBuild(): Promise<void> {
    const result = this.latest?.result;
    if (!result || result.builds.length === 0) {
      void vscode.window.showInformationMessage("Ascribe: there are no builds to choose from.");
      return;
    }
    const picked = await vscode.window.showQuickPick(
      result.builds.map((b) => ({
        label: b.name,
        description: b.editor ? "editor build" : "",
        detail: b.description,
      })),
      { title: "Preview build", placeHolder: `Now previewing ${result.build}` },
    );
    if (picked) this.chooseBuild(picked.label);
  }

  /** Shows the block the cursor is in, when no part of it is in the preview's view. */
  private followCursor(editor: vscode.TextEditor): void {
    if (!scrollSetting(editor.document, "scrollPreviewWithEditor")) return;
    this.revealLine(editor.selection.active.line, true);
  }

  /** Scrolls the preview to the block at the top of the editor. */
  private followScroll(editor: vscode.TextEditor): void {
    if (Date.now() < this.editorQuietUntil) return;
    if (!scrollSetting(editor.document, "scrollPreviewWithEditor")) return;
    const top = editor.visibleRanges[0];
    if (top) this.revealLine(top.start.line, false);
  }

  private revealLine(line: number, ifHidden: boolean): void {
    if (!this.panel || !this.ready || !this.latest?.result.page) return;
    const message: ToWebview = { type: "revealLine", line, ifHidden };
    void this.panel.webview.postMessage(message);
  }

  /** The author scrolled the preview: scrolls the editor to the line of the block at its top. */
  private scrollEditor(line: number): void {
    const editor = vscode.window.visibleTextEditors.find((e) => e.document === this.document);
    if (!editor || !scrollSetting(editor.document, "scrollEditorWithPreview")) return;
    this.editorQuietUntil = Date.now() + 300;
    editor.revealRange(new vscode.Range(line, 0, line, 0), vscode.TextEditorRevealType.AtTop);
  }

  /** The author double-clicked a block: shows its line in the editor, with the cursor on it. */
  private async openLine(line: number): Promise<void> {
    if (!this.document) return;
    const viewColumn = this.editorColumn();
    const editor = await vscode.window.showTextDocument(this.document, {
      ...(viewColumn === undefined ? {} : { viewColumn }),
      selection: new vscode.Range(line, 0, line, 0),
    });
    editor.revealRange(
      new vscode.Range(line, 0, line, 0),
      vscode.TextEditorRevealType.InCenterIfOutsideViewport,
    );
  }

  /** A click on a mark's label: shows the block's source in the editor, its lines selected. */
  private async openSource(source: string): Promise<void> {
    const parsed = parseSource(source);
    const contentRoot = this.latest?.result.contentRoot;
    if (!parsed || !contentRoot) return;
    const uri = vscode.Uri.file(fromContentPath(contentRoot, parsed.path));
    const editor = await this.showFile(uri);
    const last = Math.min(parsed.last, editor.document.lineCount - 1);
    const range = new vscode.Range(parsed.first, 0, last, editor.document.lineAt(last).text.length);
    editor.selection = new vscode.Selection(range.start, range.end);
    editor.revealRange(range, vscode.TextEditorRevealType.InCenterIfOutsideViewport);
  }

  /** Opens a file in the editor column beside the preview. */
  private async showFile(uri: vscode.Uri): Promise<vscode.TextEditor> {
    const viewColumn = this.editorColumn();
    return vscode.window.showTextDocument(uri, {
      ...(viewColumn === undefined ? {} : { viewColumn }),
      preview: false,
    });
  }

  /** Opens a changed page and its preview, and (with `firstChange`) goes to its first change once it's drawn. */
  private async showPage(uri: vscode.Uri, firstChange = true): Promise<void> {
    if (firstChange) this.firstChangeOf = uri.fsPath;
    const viewColumn = this.panel ? this.editorColumn() : undefined;
    await vscode.window.showTextDocument(uri, {
      ...(viewColumn === undefined ? {} : { viewColumn }),
      preview: false,
    });
    await this.open();
  }

  private reveal(id: string): void {
    this.revealLog.push(id);
    const message: ToWebview = { type: "reveal", id };
    void this.panel?.webview.postMessage(message);
  }

  // Which links open a file.
  /** A click on a link in the preview: open the file it names. */
  private async openLink(href: string): Promise<void> {
    if (isExternal(href)) {
      try {
        const uri = vscode.Uri.parse(href, true);
        if (EXTERNAL_SCHEMES.has(`${uri.scheme}:`)) await vscode.env.openExternal(uri);
      } catch {
        // Not a URL VS Code can open.
      }
      return;
    }
    const page = this.latest?.result.page;
    if (!page) return;
    const { path: target } = splitFragment(href);
    const link =
      page.links.find((l) => l.href === href) ??
      page.links.find((l) => splitFragment(l.href).path === target);
    if (link) {
      const uri = vscode.Uri.file(link.path);
      if (this.document && uri.fsPath === this.document.uri.fsPath) {
        if (link.id) this.reveal(link.id);
        return;
      }
      const viewColumn = this.editorColumn();
      await vscode.window.showTextDocument(uri, {
        ...(viewColumn === undefined ? {} : { viewColumn }),
        preview: false,
      });
      return;
    }
    const asset = page.assets.find(
      (a) => canonicalReference(a.reference) === canonicalReference(href),
    );
    if (asset) {
      const viewColumn = this.editorColumn();
      await vscode.commands.executeCommand(
        "vscode.open",
        vscode.Uri.file(asset.path),
        viewColumn === undefined ? undefined : { viewColumn },
      );
    }
  }

  /**
   * Opens the page of the active Ascribe document (or of the preview, when
   * it's active) in the browser, from the project's dev server, at the heading
   * the editor shows.
   */
  private async openSite(): Promise<void> {
    const editor = vscode.window.activeTextEditor;
    const document =
      editor && isPreviewable(editor.document)
        ? editor.document
        : this.panel?.active
          ? this.document
          : undefined;
    if (!document) {
      this.tell("Ascribe: open a page first, then open its site preview.");
      return;
    }
    const server = await this.projects.ensureStartedFor(document.uri);
    if (!server) {
      this.tell("Ascribe: this file isn't part of an Ascribe project (no ascribe.toml above it).");
      return;
    }
    const found = await findDevServer(server.project.folder);
    if (found.state === "none") {
      this.tell(noDevServerMessage(this.projects.name(server.project)));
      return;
    }
    const page = (await this.query(document, found.server.build))?.page;
    if (!page) {
      this.tell(
        `Ascribe: ${vscode.workspace.asRelativePath(document.uri)} isn't a page of the ${found.server.build} build, which the dev server shows.`,
      );
      return;
    }
    const shown = vscode.window.visibleTextEditors.find((e) => e.document === document);
    const top = shown?.visibleRanges[0]?.start.line;
    const section = top === undefined ? undefined : sectionAt(page.sections, top);
    await this.openExternal(pageUrl(found.server, page.route, section));
  }

  /** Shows the page or the site in the panel. */
  private async selectSurface(surface: "page" | "site"): Promise<void> {
    this.surface = surface;
    this.siteShownFor = undefined;
    if (surface === "page") {
      this.siteMessage = { type: "surface", surface: "page" };
      this.post(this.siteMessage);
      return;
    }
    await this.showSite();
  }

  /**
   * Shows the previewed page's site preview in the panel: a frame on the
   * project's dev server, at the page's route. In VS Code for the Web, where
   * the dev server's forwarded address can't be framed, it opens the browser
   * instead and goes back to the page.
   */
  private async showSite(): Promise<void> {
    const document = this.document;
    this.siteShownFor = document?.uri.toString();
    const problem = (text: string): void => {
      this.siteLog.push({ problem: text });
      this.siteMessage = { type: "surface", surface: "site", problem: text };
      this.post(this.siteMessage);
      this.wake();
    };
    const server = this.server();
    if (!document || !server) {
      problem("Open an Ascribe page to see it on the site.");
      return;
    }
    if (vscode.env.uiKind === vscode.UIKind.Web) {
      await this.openSite();
      this.surface = "page";
      this.siteMessage = { type: "surface", surface: "page" };
      this.post(this.siteMessage);
      this.tell(
        "Ascribe: the preview panel can't show the site here, so it opened in the browser.",
      );
      return;
    }
    const found = await findDevServer(server.project.folder);
    if (this.surface !== "site" || document !== this.document) return;
    if (found.state === "none") {
      problem(noDevServerMessage(this.projects.name(server.project)).replace(/^Ascribe: t/, "T"));
      return;
    }
    const latest = this.latest?.result;
    const page =
      latest?.page &&
      latest.build === found.server.build &&
      this.latest?.folder === server.project.folder
        ? latest.page
        : (await this.query(document, found.server.build))?.page;
    if (this.surface !== "site" || document !== this.document) return;
    if (!page) {
      problem(
        `${vscode.workspace.asRelativePath(document.uri)} isn't a page of the ${found.server.build} build, which the dev server shows.`,
      );
      return;
    }
    const shown = vscode.window.visibleTextEditors.find((e) => e.document === document);
    const top = shown?.visibleRanges[0]?.start.line;
    const section = top === undefined ? undefined : sectionAt(page.sections, top);
    const url = pageUrl(found.server, page.route, section);
    const external = (await vscode.env.asExternalUri(vscode.Uri.parse(url, true))).toString(true);
    if (this.surface !== "site" || document !== this.document || !this.panel) return;
    this.siteLog.push({ url: external });
    this.siteMessage = { type: "surface", surface: "site", url: external };
    const origin = new URL(external).origin;
    if (origin !== this.frameOrigin) {
      // The shell's policy names the one origin it may frame: a new one reloads it.
      this.frameOrigin = origin;
      this.setShell();
      return;
    }
    this.post(this.siteMessage);
    this.wake();
  }

  /** Opens an address of the dev server in the browser, forwarded first in a remote workspace. */
  private async openExternal(url: string): Promise<void> {
    this.externalLog.push(url);
    if (this.captureExternal) return;
    const external = await vscode.env.asExternalUri(vscode.Uri.parse(url, true));
    await vscode.env.openExternal(external);
  }

  private tell(message: string): void {
    this.siteMessages.push(message);
    void vscode.window.showInformationMessage(message);
  }

  private editorColumn(): vscode.ViewColumn | undefined {
    return vscode.window.visibleTextEditors.find((e) => e.document === this.document)?.viewColumn;
  }

  private waitForDrawn(
    description: string,
    predicate: (render: RenderRecord) => boolean,
    timeout: number,
  ): Promise<RenderRecord> {
    return new Promise((resolve, reject) => {
      const deadline = setTimeout(() => {
        this.waiters = this.waiters.filter((w) => w !== check);
        reject(new Error(`Timed out waiting for the preview: ${description}`));
      }, timeout);
      const check = (): void => {
        const found = this.log.find((r) => r.drawnAt !== undefined && predicate(r));
        if (found) {
          clearTimeout(deadline);
          this.waiters = this.waiters.filter((w) => w !== check);
          resolve(found);
        }
      };
      this.waiters.push(check);
      check();
    });
  }

  private wake(): void {
    for (const waiter of this.waiters) waiter();
  }

  dispose(): void {
    this.panel?.dispose();
    this.review.dispose();
    for (const d of this.disposables) d.dispose();
    this.disposables = [];
  }
}

/** A document the preview can show a page of. */
function isPreviewable(document: vscode.TextDocument): boolean {
  return document.uri.scheme === "file" && document.languageId === "markdown";
}

/** A document whose change can change the preview: a source, or the content model. */
function affectsProject(document: vscode.TextDocument): boolean {
  return (
    document.uri.scheme === "file" &&
    (document.languageId === "markdown" || path.basename(document.uri.fsPath) === "ascribe.toml")
  );
}

function panelTitle(document: vscode.TextDocument | undefined): string {
  return document ? `Preview: ${path.basename(document.uri.fsPath)}` : "Ascribe preview";
}

/** Whether the preview and the editor scroll together, in a direction, for a document. */
function scrollSetting(
  document: vscode.TextDocument,
  name: "scrollPreviewWithEditor" | "scrollEditorWithPreview",
): boolean {
  return vscode.workspace.getConfiguration("ascribe.preview", document.uri).get(name, true);
}
