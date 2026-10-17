import * as vscode from "vscode";
import type { ProjectServer } from "../client.js";
import type { ChangedPage } from "../preview/protocol.js";
import type { ReviewController } from "../preview/review.js";
import type { ProjectRegistry } from "../registry.js";
import type { InventoryResult, PageResult } from "../shapes.js";
import {
  modelChildren,
  modelLook,
  modelRoots,
  pagesChildren,
  pagesLook,
  pagesPath,
  pagesRoots,
  thisPageChildren,
  thisPageLook,
  thisPageRoots,
  type ItemLook,
  type ModelNode,
  type PagesNode,
  type ThisPageNode,
  type ThisPageReview,
} from "./sidebar.js";

/** How long after the last save, or switch of project, the inventory is asked for again. */
const INVENTORY_DEBOUNCE_MS = 300;

/**
 * How long after a page is created or deleted on disk the views ask again:
 * longer than the language client waits (250ms) before it tells the server,
 * so the server has the change when it's asked.
 */
const FILES_DEBOUNCE_MS = 600;

/** How long after the cursor stops moving This page asks again. */
const THIS_PAGE_DEBOUNCE_MS = 250;

/** An item of a view, as the tests see it. */
export interface SidebarItem extends ItemLook {
  /** The file and line it opens, if it opens one. */
  opens: string | undefined;
  children: SidebarItem[];
}

/** The Pages and Content model views, for tests. */
export interface SidebarApi {
  /** Every item of a view, with every node expanded. */
  items(view: "pages" | "model"): SidebarItem[];
  /** The message above a view's items, if it shows one. */
  message(view: "pages" | "model"): string | undefined;
  /** The project the views are showing, by folder. */
  project(): string | undefined;
  /** Settles when every refresh that is due has finished. */
  whenSettled(): Promise<void>;
}

/** The This page view, for tests. */
export interface ThisPageApi {
  /** Every item, with every node expanded. */
  items(): SidebarItem[];
  /** The message above its items, if it shows one. */
  message(): string | undefined;
  /** How many times it has asked the server, which is once for each cursor move it answers. */
  asked(): number;
  /** Goes to an open thread its Review section lists, by id, as choosing it does. */
  openThread(id: string): Promise<void>;
  /** Settles when every refresh that is due has finished. */
  whenSettled(): Promise<void>;
}

/** One of the three views: its tree, and how its items look and open. */
class Tree<N> implements vscode.TreeDataProvider<N> {
  readonly changed = new vscode.EventEmitter<void>();
  readonly onDidChangeTreeData = this.changed.event;
  view: vscode.TreeView<N> | undefined;

  constructor(
    readonly roots: () => N[],
    readonly children: (node: N) => N[],
    readonly look: (node: N) => ItemLook,
    readonly opens: (node: N) => { uri: vscode.Uri; selection?: vscode.Range } | undefined,
    /** The command an item runs in place of opening a file, if it runs one. */
    readonly command: (node: N) => vscode.Command | undefined = () => undefined,
  ) {}

  getTreeItem(node: N): vscode.TreeItem {
    const look = this.look(node);
    const item = new vscode.TreeItem(
      look.label,
      !look.expandable
        ? vscode.TreeItemCollapsibleState.None
        : look.expanded
          ? vscode.TreeItemCollapsibleState.Expanded
          : vscode.TreeItemCollapsibleState.Collapsed,
    );
    item.description = look.description;
    item.tooltip = look.tooltip;
    item.iconPath = new vscode.ThemeIcon(
      look.icon,
      look.dimmed ? new vscode.ThemeColor("disabledForeground") : undefined,
    );
    const target = this.opens(node);
    const command = this.command(node);
    if (command) {
      item.command = command;
    } else if (target) {
      item.command = {
        command: "vscode.open",
        title: "Open",
        arguments: [target.uri, target.selection ? { selection: target.selection } : {}],
      };
    }
    return item;
  }

  getChildren(node?: N): N[] {
    return node === undefined ? this.roots() : this.children(node);
  }

  /** Every item, with every node expanded. */
  items(node?: N): SidebarItem[] {
    return this.getChildren(node).map((child) => {
      const target = this.opens(child);
      return {
        ...this.look(child),
        opens: target
          ? `${target.uri.toString()}${target.selection ? `:${target.selection.start.line + 1}` : ""}`
          : undefined,
        children: this.items(child),
      };
    });
  }

  dispose(): void {
    this.view?.dispose();
    this.changed.dispose();
  }
}

/**
 * The This page, Pages, and Content model views in the Ascribe sidebar, for the
 * active file's project. They ask only a server that is already running, and
 * never start one: a page being active means its project's server runs.
 * Pages and Content model refresh when a file of the project is saved, when a
 * page is created or deleted, and when the active project changes; This page
 * follows the active editor and its cursor, while it's showing.
 */
export class SidebarViews implements vscode.Disposable {
  /** The server of the active file's project, running or not. */
  private server: ProjectServer | undefined;
  private inventory: InventoryResult | undefined;
  /** The server the inventory came from. */
  private inventoryFrom: ProjectServer | undefined;
  private inventoryAsked = 0;
  private page: PageResult | undefined;
  /** What review says about the page, while review is on for its project. */
  private pageReview: ThisPageReview | undefined;
  /** The file This page shows. */
  private pageFile: vscode.Uri | undefined;
  /** Counts the cursor's stops, so an older stop's answer is dropped. */
  private pageLooked = 0;
  /** How many times This page has asked the server. */
  private pageAsked = 0;
  private readonly timers = new Map<string, ReturnType<typeof setTimeout>>();
  private readonly pending = new Set<Promise<void>>();
  private readonly disposables: vscode.Disposable[] = [];

  private readonly pages = new Tree<PagesNode>(
    () => (this.inventory ? pagesRoots(this.inventory) : []),
    pagesChildren,
    pagesLook,
    (node) => {
      const content = this.inventory?.contentUri;
      const file = pagesPath(node);
      return content && file ? { uri: contentFile(content, file) } : undefined;
    },
  );

  private readonly model = new Tree<ModelNode>(
    () => (this.inventory ? modelRoots(this.inventory) : []),
    modelChildren,
    modelLook,
    (node) => {
      const model = this.inventory?.modelUri;
      if (node.kind !== "entry" || !model || !node.entry.declaration) return undefined;
      const { start, end } = node.entry.declaration;
      return {
        uri: vscode.Uri.parse(model),
        selection: new vscode.Range(start.line, start.character, end.line, end.character),
      };
    },
  );

  private readonly thisPage = new Tree<ThisPageNode>(
    () => (this.page ? thisPageRoots(this.page, this.pageReview) : []),
    thisPageChildren,
    thisPageLook,
    (node) => {
      if (node.kind === "memo") {
        const { start, end } = node.memo.body;
        return {
          uri: vscode.Uri.parse(node.memo.uri),
          selection: new vscode.Range(start.line, start.character, end.line, end.character),
        };
      }
      if (node.kind !== "place") return undefined;
      const { start, end } = node.place.range;
      return {
        uri: vscode.Uri.parse(node.place.uri),
        selection: new vscode.Range(start.line, start.character, end.line, end.character),
      };
    },
    (node) =>
      node.kind === "thread"
        ? { command: OPEN_THREAD, title: "Open Thread in the Preview", arguments: [node] }
        : undefined,
  );

  constructor(
    private readonly projects: ProjectRegistry,
    private readonly review: ReviewController,
  ) {}

  get api(): SidebarApi {
    const tree = (view: "pages" | "model") => (view === "pages" ? this.pages : this.model);
    return {
      items: (view) => tree(view).items(),
      message: (view) => tree(view).view?.message || undefined,
      project: () => this.inventoryFrom?.project.folder,
      whenSettled: () => this.whenSettled(),
    };
  }

  get thisPageApi(): ThisPageApi {
    return {
      items: () => this.thisPage.items(),
      message: () => this.thisPage.view?.message || undefined,
      asked: () => this.pageAsked,
      openThread: (id) => this.openThread(id),
      whenSettled: () => this.whenSettled(),
    };
  }

  private async whenSettled(): Promise<void> {
    while (this.timers.size > 0 || this.pending.size > 0) {
      await Promise.all(this.pending);
      if (this.timers.size > 0) await new Promise((resolve) => setTimeout(resolve, 50));
    }
  }

  register(): void {
    this.thisPage.view = vscode.window.createTreeView("ascribe.thisPage", {
      treeDataProvider: this.thisPage,
    });
    // A hidden or collapsed This page asks nothing, and catches up when shown.
    this.disposables.push(
      this.thisPage.view.onDidChangeVisibility(({ visible }) => {
        if (visible) this.soon("thisPage");
      }),
    );
    this.pages.view = vscode.window.createTreeView("ascribe.pages", {
      treeDataProvider: this.pages,
      showCollapseAll: true,
    });
    this.model.view = vscode.window.createTreeView("ascribe.model", {
      treeDataProvider: this.model,
      showCollapseAll: true,
    });
    this.disposables.push(
      this.thisPage,
      this.pages,
      this.model,
      vscode.window.onDidChangeActiveTextEditor(() => this.follow()),
      vscode.window.onDidChangeTextEditorSelection(({ textEditor }) => {
        if (textEditor === vscode.window.activeTextEditor) this.soon("thisPage");
      }),
      ...this.watchPages(),
      vscode.workspace.onDidSaveTextDocument((document) => {
        if (this.server && this.projects.serverFor(document.uri) === this.server) {
          this.soon("inventory");
          this.soon("thisPage");
        }
      }),
      this.projects.onDidChangeState((server) => {
        if (server === this.server) this.follow(true);
      }),
      this.projects.onDidChangeProjects(() => this.follow(true)),
      vscode.commands.registerCommand(OPEN_THREAD, (node?: ThisPageNode) =>
        node?.kind === "thread" ? this.openThread(node.thread.id) : undefined,
      ),
      // Review turned on or off, or the pull request's threads changed: This
      // page asks again while it shows a review, or its project's is on.
      this.review.onDidUpdate(() => {
        const on = this.server !== undefined && this.review.baseOf(this.server) !== undefined;
        if (on || this.pageReview) this.soon("thisPage");
      }),
    );
    this.follow(true);
  }

  dispose(): void {
    for (const timer of this.timers.values()) clearTimeout(timer);
    this.timers.clear();
    for (const disposable of this.disposables.splice(0)) disposable.dispose();
  }

  /**
   * Follows the active editor: a file of another project switches the views
   * to it, and a file of no project leaves them on the last one.
   */
  private follow(force = false): void {
    const uri = vscode.window.activeTextEditor?.document.uri;
    const server = uri?.scheme === "file" ? this.projects.serverFor(uri) : undefined;
    if (server && server !== this.server) {
      this.server = server;
      force = true;
    }
    if (force) {
      if (this.server?.state !== "running" || !this.projects.servers.includes(this.server)) {
        this.inventory = undefined;
        this.inventoryFrom = undefined;
        this.inventoryAsked++;
      }
      this.soon("inventory");
    }
    this.soon("thisPage");
  }

  /** Pages created or deleted outside the editor change the lists too. */
  private watchPages(): vscode.Disposable[] {
    const watcher = vscode.workspace.createFileSystemWatcher("**/*.md", false, true, false);
    const changed = (uri: vscode.Uri): void => {
      if (this.server && this.projects.serverFor(uri) === this.server) {
        this.soon("inventory", FILES_DEBOUNCE_MS);
        this.soon("thisPage", FILES_DEBOUNCE_MS);
      }
    };
    return [watcher, watcher.onDidCreate(changed), watcher.onDidDelete(changed)];
  }

  private soon(what: "inventory" | "thisPage", delay?: number): void {
    clearTimeout(this.timers.get(what));
    this.timers.set(
      what,
      setTimeout(
        () => {
          this.timers.delete(what);
          const done = (what === "inventory" ? this.askInventory() : this.askThisPage()).finally(
            () => this.pending.delete(done),
          );
          this.pending.add(done);
        },
        delay ?? (what === "inventory" ? INVENTORY_DEBOUNCE_MS : THIS_PAGE_DEBOUNCE_MS),
      ),
    );
  }

  private async askInventory(): Promise<void> {
    const asked = ++this.inventoryAsked;
    const server = this.server;
    let result: InventoryResult | undefined;
    if (server?.state === "running") {
      try {
        result = (await server.request("ascribe/inventory", {
          textDocument: { uri: vscode.Uri.file(server.project.config).toString() },
        })) as InventoryResult;
      } catch {
        result = undefined;
      }
    }
    if (asked !== this.inventoryAsked) return;
    this.inventory = result;
    this.inventoryFrom = result ? server : undefined;
    const name = server && result ? this.projects.name(server.project) : undefined;
    for (const tree of [this.pages, this.model]) {
      if (tree.view) tree.view.description = name ?? "";
    }
    if (this.pages.view) {
      this.pages.view.message =
        server && !result ? "Open a page of the project to list its pages." : "";
    }
    if (this.model.view) {
      this.model.view.message =
        server && !result ? "Open a page of the project to list its content model." : "";
    }
    this.pages.changed.fire();
    this.model.changed.fire();
  }

  private async askThisPage(): Promise<void> {
    if (this.thisPage.view && !this.thisPage.view.visible) return;
    const looked = ++this.pageLooked;
    const editor = vscode.window.activeTextEditor;
    const document = editor?.document;
    const server = document && this.projects.serverFor(document.uri);
    if (!editor || !document || document.languageId !== "markdown" || !server) {
      this.showThisPage(undefined, NO_PAGE);
      return;
    }
    if (server.state !== "running") {
      this.showThisPage(undefined, "Waiting for the project's language server.");
      return;
    }
    const { line, character } = editor.selection.active;
    this.pageAsked++;
    let page: PageResult | null = null;
    try {
      page = (await server.request("ascribe/page", {
        textDocument: { uri: document.uri.toString() },
        position: { line, character },
      })) as PageResult | null;
    } catch {
      page = null;
    }
    const review = page?.about ? await this.reviewOf(server, page.about.path) : undefined;
    if (looked !== this.pageLooked) return;
    this.pageFile = document.uri;
    this.showThisPage(page?.about ? page : undefined, page?.about ? undefined : NO_PAGE, review);
  }

  /**
   * What review says about the page at `path`, while review is on for its
   * project: the Review view's data for one page, from the same requests.
   */
  private async reviewOf(server: ProjectServer, path: string): Promise<ThisPageReview | undefined> {
    const base = this.review.baseOf(server);
    if (!base) return undefined;
    let changed: ChangedPage | null = null;
    try {
      const changes = await this.review.changesOf(server);
      changed = changes.pages.find((page) => page.path === path) ?? null;
    } catch {
      return undefined;
    }
    return {
      base: this.review.baseNameOf(server, base),
      page: changed,
      threads: await this.review.threads.threadsOn(server, path, changed),
    };
  }

  /** Goes to a thread This page lists, in the preview of the page it shows. */
  private async openThread(id: string): Promise<void> {
    if (!this.pageFile || !this.pageReview?.threads?.some((thread) => thread.id === id)) return;
    await this.review.openThread(this.pageFile, id);
  }

  /** Shows an answer in This page, or a message in place of one. */
  private showThisPage(
    page: PageResult | undefined,
    message: string | undefined,
    review?: ThisPageReview,
  ): void {
    if (page === undefined) this.pageLooked++;
    this.page = page;
    this.pageReview = page && review;
    if (this.thisPage.view) this.thisPage.view.message = message ?? "";
    this.thisPage.changed.fire();
  }
}

/** The command a thread in This page's Review section runs. */
const OPEN_THREAD = "ascribe.review.openThread";

/** What This page says when the active file isn't a page of a project. */
const NO_PAGE = "Open a page to see what's known about it.";

/** The URI of a content path under the content root. */
function contentFile(content: string, file: string): vscode.Uri {
  return vscode.Uri.joinPath(vscode.Uri.parse(content), ...file.split("/"));
}
