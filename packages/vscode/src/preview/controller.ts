import * as path from "node:path";
import { statSync } from "node:fs";
import * as vscode from "vscode";
import type { ProjectServer } from "../client.js";
import type { ProjectRegistry } from "../registry.js";
import { shellHtml } from "./html.js";
import type {
  FromWebview,
  ImageReport,
  PreviewParams,
  PreviewResult,
  RenderReport,
  ReviewView,
  ToWebview,
  WebviewAsset,
} from "./protocol.js";
import { canonicalReference, isExternal, splitFragment } from "./refs.js";
import { ReviewController, type ReviewApi } from "./review.js";
import { baseCommit, baseName, causes, fromContentPath, parseSource } from "./reviewText.js";
import { BuildChoices, previewProblems } from "./routing.js";

/** The custom request the language server answers (`crates/tessera-lsp/README.md`). */
export const PREVIEW_REQUEST = "ascribe/preview";

export const VIEW_TYPE = "ascribe.preview";

/** How long after the last change the preview asks for a new render. */
export const DEBOUNCE_MS = 100;

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
  private readonly builds = new BuildChoices();
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
  private readonly review: ReviewController;

  constructor(
    private readonly context: vscode.ExtensionContext,
    private readonly projects: ProjectRegistry,
  ) {
    this.review = new ReviewController(projects, {
      previewedDocument: () => (this.panel ? this.document : undefined),
      previewActive: () => this.panel?.active ?? false,
      previewBuild: (folder) => this.builds.get(folder),
      refresh: () => {
        if (this.panel) this.schedule(0);
      },
      showPage: (uri) => this.showPage(uri),
    });
  }

  /** Registers the commands, the listeners, and the panel serializer. */
  register(): void {
    this.review.register();
    this.disposables.push(
      vscode.commands.registerCommand("ascribe.openPreview", () => this.open()),
      vscode.commands.registerCommand("ascribe.selectPreviewBuild", () => this.pickBuild()),
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
      // A project appeared or went away: the file may have another owner.
      this.projects.onDidChangeProjects(() => {
        if (this.panel) this.schedule(0);
      }),
    );
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
    };
  }

  /** Opens the preview beside the editor, for the active Ascribe document. */
  async open(): Promise<void> {
    if (this.panel) {
      this.panel.reveal(undefined, true);
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
      { viewColumn: vscode.ViewColumn.Beside, preserveFocus: true },
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
  private async query(document: vscode.TextDocument): Promise<PreviewResult | undefined> {
    const server = await this.projects.ensureStartedFor(document.uri);
    if (!server) return undefined;
    const params: PreviewParams = { textDocument: { uri: document.uri.toString() } };
    const build = this.builds.get(server.project.folder);
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
        base: baseName(result.review.base),
        commit: baseCommit(result.review.base),
        page: result.review.changes,
        wasHtml: result.review.wasHtml,
        causes: causes(result.review.changes, result),
        goToFirst,
      };
    }
    const message: Extract<ToWebview, { type: "render" }> = {
      type: "render",
      seq: ++this.seq,
      path: result.page?.path ?? null,
      build: result.build,
      builds: result.builds,
      title: result.page?.title ?? null,
      available: result.page?.frontmatter.available ?? [],
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
    }
  }

  /** Chooses a build in the project of the last render, whose builds the picker lists. */
  private chooseBuild(name: string): void {
    const folder = this.latest?.folder;
    if (folder === undefined) return;
    // Choosing the editor's build is choosing the default, so a later change
    // of `[editor] build` is followed.
    this.builds.set(folder, name === this.editorBuild() ? undefined : name);
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

  /** Opens a changed page and its preview, and goes to its first change once it's drawn. */
  private async showPage(uri: vscode.Uri): Promise<void> {
    this.firstChangeOf = uri.fsPath;
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

export { isExternal };

/** Whether the preview and the editor scroll together, in a direction, for a document. */
function scrollSetting(
  document: vscode.TextDocument,
  name: "scrollPreviewWithEditor" | "scrollEditorWithPreview",
): boolean {
  return vscode.workspace.getConfiguration("ascribe.preview", document.uri).get(name, true);
}
