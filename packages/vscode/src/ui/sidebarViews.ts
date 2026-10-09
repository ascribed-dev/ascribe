import * as path from "node:path";
import * as vscode from "vscode";
import type { ProjectServer } from "../client.js";
import type { ProjectRegistry } from "../registry.js";
import type { ContextResult, InventoryResult } from "../shapes.js";
import {
  headingAt,
  headingTitle,
  modelChildren,
  modelLook,
  modelRoots,
  pagesChildren,
  pagesLook,
  pagesPath,
  pagesRoots,
  usedByChildren,
  usedByLook,
  usedByRoots,
  type ItemLook,
  type ModelNode,
  type PagesNode,
  type Place,
  type UsedByNode,
} from "./sidebar.js";

/** How long after the last save, or switch of project, the inventory is asked for again. */
const INVENTORY_DEBOUNCE_MS = 300;

/** How long after the cursor stops moving Used by asks again. */
const USED_BY_DEBOUNCE_MS = 250;

/** An item of a view, as the tests see it. */
export interface SidebarItem extends ItemLook {
  /** The file and line it opens, if it opens one. */
  opens: string | undefined;
  children: SidebarItem[];
}

/** The Used by, Pages, and Content model views, for tests. */
export interface SidebarApi {
  /** Every item of a view, with every node expanded. */
  items(view: "usedBy" | "pages" | "model"): SidebarItem[];
  /** The message above a view's items, if it shows one. */
  message(view: "usedBy" | "pages" | "model"): string | undefined;
  /** The project the views are showing, by folder. */
  project(): string | undefined;
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
  ) {}

  getTreeItem(node: N): vscode.TreeItem {
    const look = this.look(node);
    const item = new vscode.TreeItem(
      look.label,
      look.expandable
        ? vscode.TreeItemCollapsibleState.Collapsed
        : vscode.TreeItemCollapsibleState.None,
    );
    item.description = look.description;
    item.tooltip = look.tooltip;
    item.iconPath = new vscode.ThemeIcon(
      look.icon,
      look.dimmed ? new vscode.ThemeColor("disabledForeground") : undefined,
    );
    const target = this.opens(node);
    if (target) {
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
 * The Used by, Pages, and Content model views in the Ascribe sidebar, for the
 * active file's project. They ask only a server that is already running, and
 * never start one: a page being active means its project's server runs.
 * Pages and Content model refresh when a file of the project is saved and when
 * the active project changes; Used by follows the active editor and its cursor.
 */
export class SidebarViews implements vscode.Disposable {
  /** The server of the active file's project, running or not. */
  private server: ProjectServer | undefined;
  private inventory: InventoryResult | undefined;
  /** The server the inventory came from. */
  private inventoryFrom: ProjectServer | undefined;
  private inventoryAsked = 0;
  private places: Place[] = [];
  /** Counts the cursor's stops, so an older stop's answer is dropped. */
  private usedByLooked = 0;
  /** Counts the questions asked, so an older question's answer is dropped. */
  private usedByAsked = 0;
  /** What Used by last asked about: a page, and a heading's start or none. */
  private usedByQuestion: string | undefined;
  /** Whether Used by must ask again even about the same thing: a file was saved. */
  private usedByStale = false;
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

  private readonly usedBy = new Tree<UsedByNode>(
    () => usedByRoots(this.places),
    usedByChildren,
    usedByLook,
    (node) => {
      if (node.kind !== "place") return undefined;
      const { start, end } = node.place.range;
      return {
        uri: vscode.Uri.parse(node.place.uri),
        selection: new vscode.Range(start.line, start.character, end.line, end.character),
      };
    },
  );

  constructor(private readonly projects: ProjectRegistry) {}

  get api(): SidebarApi {
    const tree = (view: "usedBy" | "pages" | "model") =>
      view === "usedBy" ? this.usedBy : view === "pages" ? this.pages : this.model;
    return {
      items: (view) => tree(view).items(),
      message: (view) => tree(view).view?.message || undefined,
      project: () => this.inventoryFrom?.project.folder,
      whenSettled: async () => {
        while (this.timers.size > 0 || this.pending.size > 0) {
          await Promise.all(this.pending);
          if (this.timers.size > 0) await new Promise((resolve) => setTimeout(resolve, 50));
        }
      },
    };
  }

  register(): void {
    this.usedBy.view = vscode.window.createTreeView("ascribe.usedBy", {
      treeDataProvider: this.usedBy,
    });
    // A hidden or collapsed Used by asks nothing, and catches up when shown.
    this.disposables.push(
      this.usedBy.view.onDidChangeVisibility(({ visible }) => {
        if (visible) this.soon("usedBy");
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
      this.usedBy,
      this.pages,
      this.model,
      vscode.window.onDidChangeActiveTextEditor(() => this.follow()),
      vscode.window.onDidChangeTextEditorSelection(({ textEditor }) => {
        if (textEditor === vscode.window.activeTextEditor) this.soon("usedBy");
      }),
      vscode.workspace.onDidSaveTextDocument((document) => {
        if (this.server && this.projects.serverFor(document.uri) === this.server) {
          this.usedByStale = true;
          this.soon("inventory");
          this.soon("usedBy");
        }
      }),
      this.projects.onDidChangeState((server) => {
        if (server === this.server) this.follow(true);
      }),
      this.projects.onDidChangeProjects(() => this.follow(true)),
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
      this.usedByStale = true;
      this.soon("inventory");
    }
    this.soon("usedBy");
  }

  private soon(what: "inventory" | "usedBy"): void {
    clearTimeout(this.timers.get(what));
    this.timers.set(
      what,
      setTimeout(
        () => {
          this.timers.delete(what);
          const done = (what === "inventory" ? this.askInventory() : this.askUsedBy()).finally(() =>
            this.pending.delete(done),
          );
          this.pending.add(done);
        },
        what === "inventory" ? INVENTORY_DEBOUNCE_MS : USED_BY_DEBOUNCE_MS,
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

  private async askUsedBy(): Promise<void> {
    if (this.usedBy.view && !this.usedBy.view.visible) return;
    const looked = ++this.usedByLooked;
    const editor = vscode.window.activeTextEditor;
    const document = editor?.document;
    const server = document && this.projects.serverFor(document.uri);
    if (!editor || !document || document.languageId !== "markdown" || !server) {
      this.showUsedBy(undefined, [], "Open a page to see what links to it.");
      return;
    }
    if (server.state !== "running") {
      this.showUsedBy(undefined, [], "Waiting for the project's language server.");
      return;
    }
    const uri = document.uri.toString();
    const { line, character } = editor.selection.active;
    let context: ContextResult | null = null;
    try {
      context = (await server.request("ascribe/context", {
        textDocument: { uri },
        range: { start: { line, character }, end: { line, character } },
      })) as ContextResult | null;
    } catch {
      context = null;
    }
    if (looked !== this.usedByLooked) return;
    // The server parses the page, so a `#` line in a code block isn't a
    // heading and a setext heading is. Asking at the heading's start, not the
    // cursor, asks about the heading and not a phrase written in it.
    const heading = headingAt(context);
    let position = heading ? heading.range.start : { line: 0, character: 0 };
    // At the very start of the file, the server answers for the page.
    if (heading && position.line === 0 && position.character === 0) {
      position = { line: 0, character: 1 };
    }
    const question = `${uri}#${heading ? `${position.line}:${position.character}` : ""}`;
    // Moving within the same heading, or the same page, asks nothing again.
    if (question === this.usedByQuestion && !this.usedByStale) return;
    const asked = ++this.usedByAsked;
    this.usedByQuestion = question;
    this.usedByStale = false;
    let answer: unknown;
    try {
      answer = await server.request("textDocument/references", {
        textDocument: { uri },
        position,
        context: { includeDeclaration: false },
      });
    } catch {
      answer = null;
    }
    if (asked !== this.usedByAsked) return;
    const places = await this.placesOf(server, answer);
    if (asked !== this.usedByAsked) return;
    const subject = heading
      ? `“${headingTitle(document.getText(toRange(heading.range)))}”`
      : path.basename(document.uri.fsPath);
    this.showUsedBy(
      question,
      places,
      places.length === 0
        ? `Nothing links to or includes ${subject}.`
        : heading
          ? `What links to ${subject}`
          : undefined,
    );
  }

  /** Shows an answer in Used by; `question` is what it answers, if anything. */
  private showUsedBy(question: string | undefined, places: Place[], message: string | undefined) {
    if (question === undefined) this.usedByAsked++;
    this.usedByQuestion = question;
    this.places = places;
    if (this.usedBy.view) this.usedBy.view.message = message ?? "";
    this.usedBy.changed.fire();
  }

  /** The places a references answer lists, with the text of each one's line. */
  private async placesOf(server: ProjectServer, answer: unknown): Promise<Place[]> {
    if (!Array.isArray(answer)) return [];
    const texts = new Map<string, string[]>();
    const places: Place[] = [];
    for (const location of answer as { uri: string; range: Place["range"] }[]) {
      let lines = texts.get(location.uri);
      if (!lines) {
        lines = await linesOf(vscode.Uri.parse(location.uri));
        texts.set(location.uri, lines);
      }
      const file = vscode.Uri.parse(location.uri).fsPath;
      places.push({
        uri: location.uri,
        path: path.relative(server.project.folder, file).split(path.sep).join("/"),
        range: location.range,
        text: (lines[location.range.start.line] ?? "").trim(),
      });
    }
    return places;
  }
}

/** A file's lines: an open document's text, else the file on disk. */
async function linesOf(uri: vscode.Uri): Promise<string[]> {
  const open = vscode.workspace.textDocuments.find((d) => d.uri.toString() === uri.toString());
  if (open) return open.getText().split(/\r?\n/);
  try {
    return new TextDecoder().decode(await vscode.workspace.fs.readFile(uri)).split(/\r?\n/);
  } catch {
    return [];
  }
}

function toRange({ start, end }: Place["range"]): vscode.Range {
  return new vscode.Range(start.line, start.character, end.line, end.character);
}

/** The URI of a content path under the content root. */
function contentFile(content: string, file: string): vscode.Uri {
  return vscode.Uri.joinPath(vscode.Uri.parse(content), ...file.split("/"));
}
