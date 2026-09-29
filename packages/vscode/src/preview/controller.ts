import * as path from "node:path";
import { statSync } from "node:fs";
import * as vscode from "vscode";
import { shellHtml } from "./html.js";
import type {
  FromWebview,
  ImageReport,
  PreviewParams,
  PreviewResult,
  RenderReport,
  ToWebview,
  WebviewAsset,
} from "./protocol.js";
import { canonicalReference, isExternal, splitFragment } from "./refs.js";

/** The custom request the language server answers (`crates/tessera-lsp/README.md`). */
export const PREVIEW_REQUEST = "ascribe/preview";

export const VIEW_TYPE = "ascribe.preview";

/** How long after the last change the preview asks for a new render. */
export const DEBOUNCE_MS = 100;

/** How long after the last change on disk (an image replaced, a file created). */
const DISK_DEBOUNCE_MS = 250;

/** The schemes a link in the preview may open outside VS Code. */
const EXTERNAL_SCHEMES = new Set(["http:", "https:", "mailto:"]);

/** What the preview needs from the language client. */
export interface PreviewServer {
  request(method: string, params: unknown): Promise<unknown>;
  /** Fires each time the server has started (and restarted). */
  onDidStart: vscode.Event<void>;
}

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
  /** The ids of the sections the preview was told to scroll to, in order. */
  reveals(): readonly string[];
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
}

/**
 * The preview panel: one webview beside the editor that shows the page of the
 * active Ascribe document, rendered by the language server (`ascribe/preview`)
 * through the site emitter, and drawn with the element library.
 */
export class PreviewController implements vscode.Disposable {
  private panel: vscode.WebviewPanel | undefined;
  private disposables: vscode.Disposable[] = [];
  private panelDisposables: vscode.Disposable[] = [];
  private ready = false;
  private document: vscode.TextDocument | undefined;
  private selectedBuild: string | undefined;
  private timer: NodeJS.Timeout | undefined;
  private refreshing = false;
  private again = false;
  private seq = 0;
  private roots: string[] = [];
  private contentRoot: string | undefined;
  private watcher: vscode.FileSystemWatcher | undefined;
  private latest:
    { message: Extract<ToWebview, { type: "render" }>; result: PreviewResult } | undefined;
  private revealed: string | undefined;
  private log: RenderRecord[] = [];
  private revealLog: string[] = [];
  private waiters: (() => void)[] = [];

  constructor(
    private readonly context: vscode.ExtensionContext,
    private readonly server: PreviewServer,
  ) {}

  /** Registers the commands, the listeners, and the panel serializer. */
  register(): void {
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
      this.server.onDidStart(() => {
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
      whenDrawn: (description, predicate, timeout = 30_000) =>
        this.waitForDrawn(description, predicate, timeout),
      receive: (message) => this.receive(message),
      selectBuild: (name) => this.chooseBuild(name),
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
    const first = await this.query();
    const panel = vscode.window.createWebviewPanel(
      VIEW_TYPE,
      panelTitle(this.document),
      { viewColumn: vscode.ViewColumn.Beside, preserveFocus: true },
      { ...this.webviewOptions(first?.contentRoot ?? undefined), retainContextWhenHidden: true },
    );
    this.attach(panel, first?.contentRoot ?? undefined);
    this.schedule(0);
  }

  private attach(panel: vscode.WebviewPanel, contentRoot?: string): void {
    this.panel = panel;
    this.ready = false;
    this.revealed = undefined;
    this.latest = undefined;
    panel.iconPath = vscode.Uri.joinPath(this.context.extensionUri, "media", "preview.svg");
    panel.webview.options = this.webviewOptions(contentRoot);
    this.contentRoot = contentRoot;
    this.setShell();
    this.panelDisposables.push(
      panel.webview.onDidReceiveMessage((message: FromWebview) => void this.receive(message)),
      panel.onDidDispose(() => this.detach()),
    );
    void vscode.commands.executeCommand("setContext", "ascribe.previewOpen", true);
  }

  private detach(): void {
    if (this.timer) clearTimeout(this.timer);
    this.timer = undefined;
    this.watcher?.dispose();
    this.watcher = undefined;
    for (const d of this.panelDisposables) d.dispose();
    this.panelDisposables = [];
    this.panel = undefined;
    this.ready = false;
    this.latest = undefined;
    void vscode.commands.executeCommand("setContext", "ascribe.previewOpen", false);
  }

  /**
   * The webview's options. It may read the extension's webview files and the
   * project's content root, and nothing else: not the workspace, not the
   * project root (asset contract §2 allows an asset there, outside the
   * content root; the preview reports it instead of widening this).
   */
  private webviewOptions(contentRoot: string | undefined): vscode.WebviewOptions {
    const roots = [vscode.Uri.joinPath(this.context.extensionUri, "dist", "webview")];
    if (contentRoot) roots.push(vscode.Uri.file(contentRoot));
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
      previewScript: file("preview.js"),
      previewStyle: file("preview.css"),
    });
  }

  /** Makes the active Ascribe document the one previewed. Whether it changed. */
  private followActiveEditor(): boolean {
    const editor = vscode.window.activeTextEditor;
    if (!editor || !isPreviewable(editor.document) || editor.document === this.document) {
      return false;
    }
    this.document = editor.document;
    this.revealed = undefined;
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

  private async query(): Promise<PreviewResult | undefined> {
    const document = this.document;
    if (!document) return undefined;
    const params: PreviewParams = { textDocument: { uri: document.uri.toString() } };
    if (this.selectedBuild !== undefined) params.build = this.selectedBuild;
    try {
      return (await this.server.request(PREVIEW_REQUEST, params)) as PreviewResult;
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
        result = await this.query();
        if (
          !result ||
          result.documentVersion === null ||
          result.documentVersion >= document.version
        ) {
          break;
        }
        await new Promise((resolve) => setTimeout(resolve, 25));
      }
    }
    if (!this.panel || panel !== this.panel) return;
    if (!result) {
      const reason = document
        ? "The Ascribe language server isn't running, so there is nothing to preview."
        : "Open an Ascribe page to preview it.";
      result = {
        build: "",
        builds: [],
        projectRoot: null,
        contentRoot: null,
        documentVersion: null,
        page: null,
        problems: [{ severity: "info", message: reason }],
      };
    }
    // A build the content model no longer has (its file changed): back to the editor's.
    if (
      this.selectedBuild !== undefined &&
      !result.builds.some((b) => b.name === this.selectedBuild)
    ) {
      if (result.builds.length > 0) {
        this.selectedBuild = undefined;
        this.again = true;
        return;
      }
    }
    this.followRoots(result.contentRoot ?? undefined);
    const assets = this.assetUris(result);
    const message: Extract<ToWebview, { type: "render" }> = {
      type: "render",
      seq: ++this.seq,
      build: result.build,
      builds: result.builds,
      title: result.page?.title ?? null,
      available: result.page?.frontmatter.available ?? [],
      html: result.page?.html ?? null,
      assets,
      problems: result.problems,
    };
    this.latest = { message, result };
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
  private followRoots(contentRoot: string | undefined): void {
    if (!this.panel || contentRoot === this.contentRoot) return;
    this.contentRoot = contentRoot;
    this.panel.webview.options = this.webviewOptions(contentRoot);
    // Reloading the document makes the new roots take effect; the webview
    // says `ready` again and gets the latest render.
    this.setShell();
    this.watchDisk(contentRoot);
  }

  private watchDisk(contentRoot: string | undefined): void {
    this.watcher?.dispose();
    this.watcher = undefined;
    if (!contentRoot) return;
    // Images and other files change on disk without an open document.
    this.watcher = vscode.workspace.createFileSystemWatcher(
      new vscode.RelativePattern(vscode.Uri.file(contentRoot), "**/*"),
    );
    const changed = (): void => {
      if (this.panel) this.schedule(DISK_DEBOUNCE_MS);
    };
    this.watcher.onDidCreate(changed);
    this.watcher.onDidChange(changed);
    this.watcher.onDidDelete(changed);
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
        if (this.contentRoot && !this.watcher) this.watchDisk(this.contentRoot);
        return;
      }
      case "build":
        this.chooseBuild(message.name);
        return;
      case "open":
        await this.openLink(message.href);
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
    }
  }

  private chooseBuild(name: string): void {
    // Choosing the editor's build is choosing the default, so a later change
    // of `[editor] build` is followed.
    this.selectedBuild = name === this.editorBuild() ? undefined : name;
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

  /** Scrolls the preview to the section the cursor is in, when it moved into another. */
  private followCursor(editor: vscode.TextEditor): void {
    const sections = this.latest?.result.page?.sections;
    if (!this.panel || !this.ready || !sections || sections.length === 0) return;
    const line = editor.selection.active.line;
    let current: string | undefined;
    for (const section of sections) {
      if (section.line <= line) current = section.id;
      else break;
    }
    if (current === undefined || current === this.revealed) return;
    this.revealed = current;
    this.reveal(current);
  }

  private reveal(id: string): void {
    this.revealLog.push(id);
    const message: ToWebview = { type: "reveal", id };
    void this.panel?.webview.postMessage(message);
  }

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
    for (const waiter of [...this.waiters]) waiter();
  }

  dispose(): void {
    this.panel?.dispose();
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
