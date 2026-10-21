import * as vscode from "vscode";
import { QuickInputPrompter } from "../actions/ask.js";
import type { Action } from "../actions/registry.js";
import { newNameStep } from "../actions/registry.js";
import { renameAt, type RunRecord } from "../actions/run.js";
import {
  runWizard,
  ScriptedPrompter,
  text,
  validKey,
  validValue,
  type Answers,
  type Prompter,
  type Scripted,
} from "../actions/steps.js";
import type { ProjectServer } from "../client.js";
import {
  boolean,
  listOf,
  nullable,
  shape,
  text as shortText,
  whole,
  type Check,
} from "../preview/messages.js";
import type { ChangedPage } from "../preview/protocol.js";
import type { ReviewController } from "../preview/review.js";
import type { ProjectRegistry } from "../registry.js";
import type { ContextResult, InventoryResult, LspRange, PageResult } from "../shapes.js";
import type { ThisPageAnswer, ThisPageProbe, ThisPageReport } from "../webview/viewProtocol.js";
import { insertPlan, LastEditors } from "./insertAt.js";
import type { ProjectBuilds } from "./projectBuilds.js";
import { WebviewView, type Drawn } from "./webviewView.js";
import {
  blocksMove,
  modelChildren,
  modelDeclaration,
  modelLook,
  modelRoots,
  pagesChildren,
  pagesDraggable,
  pagesDrop,
  contentPathOf,
  followedProject,
  healthBadge,
  labelRename,
  leftOut,
  NOTICE_ACTIONS,
  pagesFind,
  pagesLook,
  pagesParent,
  pagesPath,
  newPathProblem,
  pagesResource,
  pagesRoots,
  shownPage,
  thisPageChildren,
  thisPageHead,
  thisPageItems,
  thisPageLook,
  thisPageNotice,
  thisPageRoots,
  viewsNotice,
  type ItemLook,
  type ModelNode,
  type PagesNode,
  type ProblemCounts,
  type ThisPageNode,
  type ThisPageNotice,
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
  /** The file it is, which VS Code decorates: its problems' color, for one. */
  resource?: string;
  /** The color the views' own decoration of its file gives its label, by its theme color's id. */
  labelColor?: string;
  /** The command it runs in place of opening a file, if it runs one. */
  runs?: string;
  children: SidebarItem[];
}

/** A command of an item's menu in Pages or Content model, as the tests name it. */
export type ItemCommand =
  | "openPreview"
  | "copyLink"
  | "renameOrMove"
  | "findReferences"
  | "revealInExplorer"
  | "newPageHere"
  | "rename"
  | "insert";

/** What a command of an item's menu did, for tests. */
export interface ItemRecord {
  /** What it told the writer. */
  messages: string[];
  /** What it copied. */
  copied?: string;
  /** The references it showed, each as a file relative to the workspace and a line from 1. */
  references?: string[];
  /** For a rename: whether it asked first, in the refactor preview, and the files it changes. */
  previewed?: boolean;
  files?: string[];
  /** For Insert at the Cursor: what running the action did. */
  run?: RunRecord;
  /** Whether it did what it's for. */
  done: boolean;
}

/** The Pages and Content model views, for tests. */
export interface SidebarApi {
  /** Every item of a view, with every node expanded. */
  items(view: "pages" | "model"): SidebarItem[];
  /**
   * Drops one item of Pages on another, as a drag does, each named by a
   * page's content path or a section's folder (`""` for Start). `move`
   * answers the dialog a move to another section asks first. Settles when
   * the edit is applied, with what it says when it can't be.
   */
  drop(dragged: string, target: string, move?: boolean): Promise<string | undefined>;
  /** Runs an item's command from its menu: Place Here or Rename Label…, with the label typed. */
  command(
    command: "placeHere" | "renameLabel",
    item: string,
    label?: string,
  ): Promise<string | undefined>;
  /**
   * Runs a command from an item's menu, `answers` answering what it asks.
   * An item of Pages is named by a page's or fragment's content path, or a
   * section's folder; one of Content model by its kind and key,
   * `phrase:product`, or a dimension's value, `value:deployment=cloud`.
   */
  act(command: ItemCommand, item: string, answers?: Scripted[]): Promise<ItemRecord>;
  /** Turns Orphans Only in Pages, or Unused Only in Content model, on or off. */
  filter(view: "pages" | "model", on: boolean): Promise<void>;
  /** The content path of the page selected in Pages, if one is. */
  selected(): string | undefined;
  /** The message above a view's items, if it shows one. */
  message(view: "pages" | "model"): string | undefined;
  /** The project the views are showing, by folder. */
  project(): string | undefined;
  /** Settles when every refresh that is due has finished. */
  whenSettled(): Promise<void>;
}

/** The This page view, for tests. */
export interface ThisPageApi {
  /** Every item of the model the webview draws, with every node expanded. */
  items(): SidebarItem[];
  /** The message in place of its items, if it shows one. */
  message(): string | undefined;
  /** The badge's number and tooltip, when it shows one: Health's things to do. */
  badge(): { value: number; tooltip: string } | undefined;
  /** Whether it's showing: neither collapsed nor in a hidden container. */
  visible(): boolean;
  /** What its webview drew last, if it has drawn. */
  drawn(): Drawn<ThisPageReport> | undefined;
  /** Waits until what its webview drew satisfies `predicate`. */
  whenDrawn(
    description: string,
    predicate: (drawn: Drawn<ThisPageReport>) => boolean,
    timeout?: number,
  ): Promise<Drawn<ThisPageReport>>;
  /** Chooses an item of what the webview drew last, by its key, as a click does. */
  choose(key: string): void;
  /**
   * Does in the webview what a reader does, and resolves with its report of
   * the answer drawn: opens every section, puts the keyboard's focus on an
   * item or a section's header by its key, scrolls, and chooses what has the
   * focus, as Enter does, in that order.
   */
  probe(what: Omit<ThisPageProbe, "type" | "probe">): Promise<Drawn<ThisPageReport>>;
  /** Presses Escape in the view, as the reader does. */
  escape(): Promise<void>;
  /**
   * How many requests it has sent the server, which is one for each cursor
   * move it answers, review on or off.
   */
  asked(): number;
  /**
   * Stands in for `vscode.env.openExternal` from now on: what it would have
   * opened is added to the list returned, and nothing is opened.
   */
  stubExternal(): string[];
  /** Goes to an open thread its Review section lists, by id, as choosing it does. */
  openThread(id: string): Promise<void>;
  /** Settles when every refresh that is due has finished. */
  whenSettled(): Promise<void>;
}

/** One of the three views: its tree, and how its items look and open. */
/** What an item opens: a file at a place, or, for This page's includes, a file beside the page. */
interface Target {
  uri: vscode.Uri;
  selection?: vscode.Range;
  beside?: boolean;
}

class Tree<N> implements vscode.TreeDataProvider<N> {
  /** An item's parent, which revealing an item walks up. */
  parentOf: ((node: N) => N | undefined) | undefined;
  /** The file an item is, which VS Code decorates as the Explorer does. */
  resourceOf: ((node: N) => vscode.Uri | undefined) | undefined;
  /** The views' own decoration of an item's file, if they give it one. */
  decorationOf: ((uri: vscode.Uri) => vscode.FileDecoration | undefined) | undefined;

  readonly changed = new vscode.EventEmitter<void>();
  readonly onDidChangeTreeData = this.changed.event;
  view: vscode.TreeView<N> | undefined;

  constructor(
    readonly roots: () => N[],
    readonly children: (node: N) => N[],
    readonly look: (node: N) => ItemLook,
    readonly opens: (node: N) => Target | undefined,
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
    const resource = this.resourceOf?.(node);
    if (resource) item.resourceUri = resource;
    item.description = look.description;
    item.tooltip = look.tooltip;
    if (look.context !== undefined) item.contextValue = look.context;
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

  getParent(node: N): N | undefined {
    return this.parentOf?.(node);
  }

  /** Every item, with every node expanded. */
  items(node?: N): SidebarItem[] {
    return this.getChildren(node).map((child) => {
      const target = this.opens(child);
      const command = this.command(child);
      const resource = this.resourceOf?.(child);
      const color = resource && this.decorationOf?.(resource)?.color;
      return {
        ...this.look(child),
        ...(resource ? { resource: resource.toString() } : {}),
        ...(color ? { labelColor: color.id } : {}),
        opens: target
          ? `${target.uri.toString()}${target.selection ? `:${target.selection.start.line + 1}` : ""}`
          : undefined,
        ...(command ? { runs: command.command } : {}),
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
 * active file's project, or the one Switch Project chose. They ask only a
 * server that is already running, and start one only when Switch Project
 * chooses its project: a page being active means its project's server runs.
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
  /**
   * The page This page shows (`shownPage`), by its document, and the cursor
   * in it as it was last: its editor is gone while another tab covers it.
   */
  private pageAt: PageAt | undefined;
  /** What This page says in place of a page; `undefined` while it shows one. */
  private pageNotice: ThisPageNotice | undefined;
  /** The nodes each answer This page sent names, by its items' keys. */
  private readonly pageNodes = new WeakMap<ThisPageAnswer, Map<string, ThisPageNode>>();
  /** Counts the cursor's stops, so an older stop's answer is dropped. */
  private pageLooked = 0;
  /** How many times This page has asked the server. */
  private pageAsked = 0;
  /** How many probes the tests have sent This page's webview. */
  private pageProbed = 0;
  /**
   * Opens an address on the published site outside VS Code, which asks about
   * the domain first if it isn't trusted. The tests stand in for it.
   */
  private openExternal = (uri: vscode.Uri): Thenable<boolean> => vscode.env.openExternal(uri);
  private readonly timers = new Map<string, ReturnType<typeof setTimeout>>();
  private readonly pending = new Set<Promise<void>>();
  private readonly disposables: vscode.Disposable[] = [];
  private readonly projectChanged = new vscode.EventEmitter<ProjectServer | undefined>();
  /** The views moved to another project: the Project view follows them. */
  readonly onDidChangeProject = this.projectChanged.event;
  private readonly inventoryChanged = new vscode.EventEmitter<void>();
  /** The inventory was answered again, or let go: the Project view counts it. */
  readonly onDidChangeInventory = this.inventoryChanged.event;

  private readonly decorationsChanged = new vscode.EventEmitter<undefined>();
  /** The files of the pages the shown build leaves out, for the inventory and build they're of. */
  private leftOutFiles: { from: InventoryResult; build: string; files: Set<string> } | undefined;

  /** Pages' items, made once for each inventory, so revealing one finds it. */
  private pagesNodes:
    { from: InventoryResult; orphansOnly: boolean; roots: PagesNode[] } | undefined;

  private readonly pages = new Tree<PagesNode>(
    () => this.pagesRoots(),
    pagesChildren,
    (node) => pagesLook(node, this.shownBuild()),
    (node) => {
      const content = this.inventory?.contentUri;
      const file = pagesPath(node);
      return content && file ? { uri: contentFile(content, file) } : undefined;
    },
  );

  private readonly model = new Tree<ModelNode>(
    () => (this.inventory ? modelRoots(this.inventory, this.unusedOnly) : []),
    modelChildren,
    modelLook,
    (node) => {
      const uri = this.declarationUri(node);
      const declaration = modelDeclaration(node);
      if (!uri || !declaration) return undefined;
      const { start, end } = declaration;
      return {
        uri,
        selection: new vscode.Range(start.line, start.character, end.line, end.character),
      };
    },
  );

  /**
   * The model This page's webview draws: the same functions the tree had,
   * so the tests read its items as before. It's never shown as a tree.
   */
  private readonly thisPage = new Tree<ThisPageNode>(
    () =>
      this.page
        ? thisPageRoots(this.page, this.pageReview, {
            uri: this.pageFile?.toString(),
            problems: this.pageFile && problemsOf(this.pageFile),
            build: this.pageBuild(),
          })
        : [],
    thisPageChildren,
    thisPageLook,
    (node) => {
      const at = (uri: string, { start, end }: LspRange) => ({
        uri: vscode.Uri.parse(uri),
        selection: new vscode.Range(start.line, start.character, end.line, end.character),
      });
      switch (node.kind) {
        case "memo":
          return at(node.memo.uri, node.memo.body);
        case "place":
          return at(node.place.uri, node.place.range);
        case "include":
          return node.include.uri
            ? { uri: vscode.Uri.parse(node.include.uri), beside: true }
            : undefined;
        case "row":
          switch (node.row.opens?.kind) {
            case "place":
              return at(node.row.opens.uri, node.row.opens.range);
            case "beside":
              return { uri: vscode.Uri.parse(node.row.opens.uri), beside: true };
            default:
              return undefined;
          }
        default:
          return undefined;
      }
    },
    (node) => {
      if (node.kind === "thread") {
        return { command: OPEN_THREAD, title: "Open Thread in the Preview", arguments: [node] };
      }
      if (node.kind === "row" && node.row.opens?.kind === "command") {
        return { command: node.row.opens.command, title: node.row.opens.title };
      }
      return undefined;
    },
  );

  /** Whether Pages shows only the orphans, and Content model only the unused entries. */
  private orphansOnly = false;
  private unusedOnly = false;
  /** The last Markdown editor of each project, which Insert at the Cursor writes into. */
  private readonly lastEditors = new LastEditors();

  constructor(
    private readonly projects: ProjectRegistry,
    private readonly review: ReviewController,
    private readonly builds: ProjectBuilds,
    private readonly actions: Inserter,
    /** This page's webview, which draws the model's items: made before the folder was trusted. */
    private readonly thisPageView: WebviewView<ThisPageAnswer, ThisPageReport>,
  ) {
    thisPageView.handlers = {
      open: (seq, key) => void this.chooseItem(seq, key),
      focusEditor: () => void this.focusPageEditor(),
    };
    this.pages.parentOf = pagesParent;
    this.pages.resourceOf = (node) => {
      const content = this.inventory?.contentUri;
      const file = pagesResource(node);
      return content && file ? contentFile(content, file) : undefined;
    };
    this.pages.decorationOf = (uri) => this.leftOutDecoration(uri);
  }

  /**
   * Dims the label of a page the shown build leaves out, as `pagesLook` dims
   * its icon: a tree item's label takes a color only from a decoration of its
   * file. A problem's red or yellow still shows over it, since VS Code gives
   * the problems' decorations more weight than an extension's. The Explorer
   * and the editor's tabs show the same file, so they dim it too.
   */
  private leftOutDecoration(uri: vscode.Uri): vscode.FileDecoration | undefined {
    const inventory = this.inventory;
    const content = inventory?.contentUri;
    const build = this.shownBuild();
    if (!inventory || !content || build === undefined) return undefined;
    if (this.leftOutFiles?.from !== inventory || this.leftOutFiles.build !== build) {
      const files = new Set(
        inventory.pages
          .filter((page) => leftOut(page, build))
          .map((page) => contentFile(content, page.path).toString()),
      );
      this.leftOutFiles = { from: inventory, build, files };
    }
    if (!this.leftOutFiles.files.has(uri.toString())) return undefined;
    return {
      color: new vscode.ThemeColor("disabledForeground"),
      tooltip: `Not in ${build}`,
    };
  }

  private pagesRoots(): PagesNode[] {
    const inventory = this.inventory;
    if (!inventory) return [];
    if (this.pagesNodes?.from !== inventory || this.pagesNodes.orphansOnly !== this.orphansOnly) {
      this.pagesNodes = {
        from: inventory,
        orphansOnly: this.orphansOnly,
        roots: pagesRoots(inventory, this.orphansOnly),
      };
    }
    return this.pagesNodes.roots;
  }

  /**
   * The build the status bar shows for the project of the page This page
   * shows, which says whether the page is on the site there.
   */
  private pageBuild(): string | undefined {
    const server = this.pageFile && this.projects.serverFor(this.pageFile);
    return server ? this.builds.info(server).build : undefined;
  }

  /** The build the status bar shows for the views' project, which Pages dims the pages it leaves out of. */
  private shownBuild(): string | undefined {
    const server = this.inventoryFrom;
    return server ? this.builds.info(server).build : undefined;
  }

  /** An item of Pages by a page's content path, or a section's folder. */
  private pagesItem(name: string): PagesNode | undefined {
    const roots = this.pagesRoots();
    const section = (nodes: readonly PagesNode[]): PagesNode | undefined => {
      for (const node of nodes) {
        if (node.kind === "section" && node.section.folder === name) return node;
        const found = section(node.children);
        if (found) return found;
      }
      return undefined;
    };
    return name.endsWith("/") || name === "" ? section(roots) : pagesFind(roots, name);
  }

  get api(): SidebarApi {
    const tree = (view: "pages" | "model") => (view === "pages" ? this.pages : this.model);
    const item = (name: string) => {
      const node = this.pagesItem(name);
      if (!node) throw new Error(`Pages has no item ${name}`);
      return node;
    };
    const modelItem = (name: string): ModelNode => {
      const [kind = "", key = ""] = name.split(/:(.*)/s);
      for (const root of this.model.getChildren()) {
        for (const entry of modelChildren(root)) {
          if (entry.kind !== "entry") continue;
          if (entry.entry.kind === kind && entry.entry.key === key) return entry;
          for (const value of modelChildren(entry)) {
            if (
              value.kind === "value" &&
              kind === "value" &&
              `${value.dimension.key}=${value.value.value}` === key
            ) {
              return value;
            }
          }
        }
      }
      throw new Error(`Content model has no item ${name}`);
    };
    return {
      items: (view) => tree(view).items(),
      drop: (dragged, target, move) =>
        this.drop(item(dragged), item(target), async () => move ?? false),
      command: (command, name, label) =>
        command === "placeHere"
          ? this.placeHere(item(name))
          : this.renameLabel(item(name), async () => label),
      act: (command, name, answers = []) => {
        const prompter = new ScriptedPrompter([...answers]);
        return /^[a-z]+:/.test(name)
          ? this.modelCommand(command, modelItem(name), prompter)
          : this.pagesCommand(command, item(name), prompter);
      },
      filter: (view, on) => this.filter(view, on),
      selected: () => {
        const node = this.pages.view?.selection[0];
        return node && pagesPath(node);
      },
      message: (view) => tree(view).view?.message || undefined,
      project: () => this.inventoryFrom?.project.folder,
      whenSettled: () => this.whenSettled(),
    };
  }

  get thisPageApi(): ThisPageApi {
    return {
      items: () => this.thisPage.items(),
      message: () => this.thisPageView.latest?.message ?? undefined,
      badge: () => {
        const badge = this.thisPageView.badge;
        return badge?.tooltip === undefined
          ? undefined
          : { value: badge.value, tooltip: badge.tooltip };
      },
      visible: () => this.thisPageView.visible,
      drawn: () => this.thisPageView.drawn,
      whenDrawn: (description, predicate, timeout) =>
        this.thisPageView.whenDrawn(description, predicate, timeout),
      choose: (key) => {
        const drawn = this.thisPageView.drawn;
        if (drawn) void this.chooseItem(drawn.seq, key);
      },
      probe: (what) => {
        const probe = ++this.pageProbed;
        this.thisPageView.post({ ...what, type: "probe", probe } satisfies ThisPageProbe);
        return this.thisPageView.whenDrawn(
          `its answer to probe ${probe}`,
          (drawn) => drawn.report.probed >= probe,
        );
      },
      escape: () => this.focusPageEditor(),
      asked: () => this.pageAsked,
      stubExternal: () => {
        const opened: string[] = [];
        this.openExternal = (uri) => {
          opened.push(uri.toString(true));
          return Promise.resolve(true);
        };
        return opened;
      },
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
    // A hidden or collapsed This page asks nothing, and catches up when shown.
    this.disposables.push(
      this.thisPageView.onDidShow(() => {
        // Shown again on another page than it drew: the answer it kept is
        // out of date, so it waits for the new one instead of drawing it.
        if (this.shownPageNow()?.uri.toString() !== this.pageFile?.toString()) {
          this.thisPageView.forget();
        }
        this.soon("thisPage", 0);
      }),
    );
    this.pages.view = vscode.window.createTreeView("ascribe.pages", {
      treeDataProvider: this.pages,
      showCollapseAll: true,
      dragAndDropController: this.pagesDrag(),
    });
    this.model.view = vscode.window.createTreeView("ascribe.model", {
      treeDataProvider: this.model,
      showCollapseAll: true,
    });
    this.disposables.push(
      this.thisPage,
      this.pages,
      this.model,
      vscode.window.registerFileDecorationProvider({
        onDidChangeFileDecorations: this.decorationsChanged.event,
        provideFileDecoration: (uri) => this.leftOutDecoration(uri),
      }),
      this.onDidChangeInventory(() => this.decorationsChanged.fire(undefined)),
      vscode.window.onDidChangeActiveTextEditor(() => this.follow(false, true)),
      vscode.window.onDidChangeTextEditorSelection(({ textEditor, selections }) => {
        const selection = selections[0];
        if (selection && textEditor.document.uri.toString() === this.pageAt?.uri.toString()) {
          this.pageAt = { uri: this.pageAt.uri, selection };
        }
        if (textEditor === vscode.window.activeTextEditor) this.soon("thisPage");
      }),
      // The page's last tab closed: This page shows the active editor's, if it's a page's.
      vscode.window.tabGroups.onDidChangeTabs(({ closed }) => {
        if (closed.length > 0 && this.pageAt && !openInTab(this.pageAt.uri)) {
          this.soon("thisPage");
        }
      }),
      // Health's Problems row counts VS Code's diagnostics, which the
      // server doesn't answer with: the view is drawn again, not asked again.
      vscode.languages.onDidChangeDiagnostics(({ uris }) => {
        const shown = this.pageFile?.toString();
        if (this.page && uris.some((uri) => uri.toString() === shown)) {
          this.drawThisPage();
        }
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
      // The links of the views' welcome while their project's server has failed.
      vscode.commands.registerCommand(SHOW_OUTPUT, () => this.server?.showOutput()),
      vscode.commands.registerCommand(RESTART, () => this.server?.restart()),
      vscode.commands.registerCommand(PLACE_HERE, (node?: PagesNode) =>
        node ? this.placeHere(node).then((said) => said && this.say(said)) : undefined,
      ),
      vscode.commands.registerCommand(RENAME_LABEL, (node?: PagesNode) =>
        node
          ? this.renameLabel(node, (value, prompt) =>
              Promise.resolve(vscode.window.showInputBox({ title: "Rename Label", prompt, value })),
            ).then((said) => said && this.say(said))
          : undefined,
      ),
      // Pages draws `ascribe.nav` as it's typed, and as a drag edits it.
      vscode.workspace.onDidChangeTextDocument(({ document }) => {
        if (
          this.server &&
          isNavFile(document.uri) &&
          this.projects.serverFor(document.uri) === this.server
        ) {
          this.soon("inventory");
        }
      }),
      ...this.registerItemCommands(),
      // Insert at the Cursor writes into the project's last Markdown editor.
      vscode.window.onDidChangeActiveTextEditor((editor) => editor && this.noteEditor(editor)),
      vscode.window.onDidChangeTextEditorSelection(({ textEditor }) => this.noteEditor(textEditor)),
      vscode.workspace.onDidCloseTextDocument((document) =>
        this.lastEditors.forget(document.uri.toString()),
      ),
      // Pages dims what the build the status bar shows leaves out, and This
      // page's addresses open on the site only where the build has the page.
      this.builds.onDidChange(() => {
        this.pages.changed.fire();
        this.decorationsChanged.fire(undefined);
        if (this.page) this.drawThisPage();
      }),
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
    const editor = vscode.window.activeTextEditor;
    if (editor) this.noteEditor(editor);
    void this.setFilterKeys();
    this.follow(true);
  }

  /** The commands of Pages' and Content model's items and title bars. */
  private registerItemCommands(): vscode.Disposable[] {
    const prompter = new QuickInputPrompter();
    const report = (record: ItemRecord) => {
      for (const message of record.messages) this.say(message);
    };
    const pages = (id: string, command: ItemCommand) =>
      vscode.commands.registerCommand(id, (node?: PagesNode) =>
        node ? this.pagesCommand(command, node, prompter).then(report) : undefined,
      );
    const model = (id: string, command: ItemCommand) =>
      vscode.commands.registerCommand(id, (node?: ModelNode) =>
        node ? this.modelCommand(command, node, prompter).then(report) : undefined,
      );
    return [
      pages(PAGES_COMMANDS.openPreview, "openPreview"),
      pages(PAGES_COMMANDS.copyLink, "copyLink"),
      pages(PAGES_COMMANDS.renameOrMove, "renameOrMove"),
      pages(PAGES_COMMANDS.findReferences, "findReferences"),
      pages(PAGES_COMMANDS.revealInExplorer, "revealInExplorer"),
      pages(PAGES_COMMANDS.newPageHere, "newPageHere"),
      model(MODEL_COMMANDS.findReferences, "findReferences"),
      model(MODEL_COMMANDS.rename, "rename"),
      model(MODEL_COMMANDS.insert, "insert"),
      vscode.commands.registerCommand(PAGES_COMMANDS.orphansOnly, () => this.filter("pages", true)),
      vscode.commands.registerCommand(PAGES_COMMANDS.showAll, () => this.filter("pages", false)),
      vscode.commands.registerCommand(MODEL_COMMANDS.unusedOnly, () => this.filter("model", true)),
      vscode.commands.registerCommand(MODEL_COMMANDS.showAll, () => this.filter("model", false)),
    ];
  }

  /** Turns a view's filter on or off: Orphans Only in Pages, Unused Only in Content model. */
  private async filter(view: "pages" | "model", on: boolean): Promise<void> {
    if (view === "pages") this.orphansOnly = on;
    else this.unusedOnly = on;
    await this.setFilterKeys();
    this.showMessages();
    (view === "pages" ? this.pages : this.model).changed.fire();
  }

  /** The context keys the title bars' toggles are shown by. */
  private async setFilterKeys(): Promise<void> {
    await Promise.all([
      vscode.commands.executeCommand("setContext", ORPHANS_ONLY, this.orphansOnly),
      vscode.commands.executeCommand("setContext", UNUSED_ONLY, this.unusedOnly),
    ]);
  }

  /** Notes a Markdown editor of a project as that project's last, with its cursor. */
  private noteEditor(editor: vscode.TextEditor): void {
    const { document, selection } = editor;
    if (document.languageId !== "markdown" || document.uri.scheme !== "file") return;
    const server = this.projects.serverFor(document.uri);
    if (!server) return;
    this.lastEditors.note(server.project.folder, {
      uri: document.uri.toString(),
      selection: {
        start: { line: selection.start.line, character: selection.start.character },
        end: { line: selection.end.line, character: selection.end.character },
      },
    });
  }

  /** Runs a command of an item of Pages. */
  private async pagesCommand(
    command: ItemCommand,
    node: PagesNode,
    prompter: Prompter,
  ): Promise<ItemRecord> {
    const record: ItemRecord = { messages: [], done: false };
    const content = this.inventory?.contentUri;
    if (!content) return record;
    if (command === "newPageHere") {
      if (node.kind !== "section") return record;
      // The content root's section is the content root.
      const folder = node.section.folder
        ? contentFile(content, node.section.folder.replace(/\/$/, ""))
        : vscode.Uri.parse(content);
      await vscode.commands.executeCommand("ascribe.newPage", folder);
      record.done = true;
      return record;
    }
    const path = pagesResource(node);
    if (path === undefined) return record;
    const uri = contentFile(content, path);
    switch (command) {
      case "openPreview":
        await vscode.window.showTextDocument(uri, { preview: true });
        await vscode.commands.executeCommand("ascribe.openPreview");
        record.done = true;
        break;
      case "copyLink": {
        // What a link in a page writes, from the content root, as Copy a
        // link to this section writes a heading's.
        const link = `/${path}`;
        await vscode.env.clipboard.writeText(link);
        record.copied = link;
        record.messages.push(`Copied ${link}`);
        record.done = true;
        break;
      }
      case "renameOrMove":
        await this.renameOrMove(path, prompter, record);
        break;
      case "findReferences":
        await this.findReferences(uri, new vscode.Position(0, 0), record);
        break;
      case "revealInExplorer":
        await vscode.commands.executeCommand("revealInExplorer", uri);
        record.done = true;
        break;
      default:
        break;
    }
    return record;
  }

  /**
   * Rename or Move…: asks for the new path, then renames the file as the
   * Explorer does, and the server's edits for a moved file follow: its links,
   * the links to it, and `ascribe.nav`.
   */
  private async renameOrMove(path: string, prompter: Prompter, record: ItemRecord): Promise<void> {
    const content = this.inventory?.contentUri;
    if (!content) return;
    const asked = await runWizard(
      "Rename or Move",
      {
        steps: () => [
          {
            kind: "text",
            key: "path",
            prompt: "Its new path, from the content root.",
            value: path,
            validate: (value) => newPathProblem(path, value),
          },
        ],
        args: (answers) => ({ path: text(answers, "path")?.trim() }),
      },
      prompter,
    );
    if (!asked || "error" in asked || typeof asked.args.path !== "string") return;
    const said = await this.moveFile(path, asked.args.path);
    if (said) record.messages.push(said);
    else record.done = true;
  }

  /** Moves a page's or fragment's file, as the Explorer does; what it says when it can't. */
  private async moveFile(from: string, to: string): Promise<string | undefined> {
    const content = this.inventory?.contentUri;
    if (!content) return undefined;
    const target = contentFile(content, to);
    if (blocksMove(from, to)) {
      try {
        await vscode.workspace.fs.stat(target);
        return `${to} is there already.`;
      } catch {
        // Nothing's there: it can move.
      }
    }
    const edit = new vscode.WorkspaceEdit();
    edit.renameFile(contentFile(content, from), target, { overwrite: false });
    if (!(await vscode.workspace.applyEdit(edit))) return `${from} couldn't be moved.`;
    this.soon("inventory");
    return undefined;
  }

  /**
   * Find All References at `position` of `uri`: the server's answer, shown in
   * a peek there. A page's are at its start; an entry's, at its declaration.
   */
  private async findReferences(
    uri: vscode.Uri,
    position: vscode.Position,
    record: ItemRecord,
  ): Promise<void> {
    await vscode.window.showTextDocument(uri, {
      selection: new vscode.Range(position, position),
      preview: true,
    });
    const places =
      (await vscode.commands.executeCommand<vscode.Location[] | undefined>(
        "vscode.executeReferenceProvider",
        uri,
        position,
      )) ?? [];
    record.references = places.map(
      (place) =>
        `${vscode.workspace.asRelativePath(place.uri, false)}:${place.range.start.line + 1}`,
    );
    await vscode.commands.executeCommand("editor.action.showReferences", uri, position, places);
    record.done = true;
  }

  /**
   * The file an item of Content model is declared in: a glossary term's page,
   * or `ascribe.toml` for the rest.
   */
  private declarationUri(node: ModelNode): vscode.Uri | undefined {
    const page = node.kind === "entry" ? node.entry.page : null;
    if (page !== null) {
      const content = this.inventory?.contentUri;
      return content ? contentFile(content, page) : undefined;
    }
    const model = this.inventory?.modelUri;
    return model ? vscode.Uri.parse(model) : undefined;
  }

  /** Runs a command of an item of Content model. */
  private async modelCommand(
    command: ItemCommand,
    node: ModelNode,
    prompter: Prompter,
  ): Promise<ItemRecord> {
    const record: ItemRecord = { messages: [], done: false };
    const model = this.inventory?.modelUri;
    const declaration = modelDeclaration(node);
    const declared = this.declarationUri(node);
    switch (command) {
      case "findReferences":
        if (declared && declaration) {
          const { line, character } = declaration.start;
          await this.findReferences(declared, new vscode.Position(line, character), record);
        }
        break;
      case "rename":
        if (model && declaration) await this.rename(node, model, declaration, prompter, record);
        break;
      case "insert":
        if (node.kind === "entry") await this.insert(node, prompter, record);
        break;
      default:
        break;
    }
    return record;
  }

  /**
   * Rename…, on a phrase or a dimension's value: asks for the new name, then
   * runs the server's rename at its declaration, which `F2` on it in a page
   * runs too, and shows every change in the refactor preview.
   */
  private async rename(
    node: ModelNode,
    model: string,
    declaration: LspRange,
    prompter: Prompter,
    record: ItemRecord,
  ): Promise<void> {
    const server = this.inventoryFrom;
    if (server?.state !== "running") {
      record.messages.push("The project's language server isn't running.");
      return;
    }
    const entries = this.inventory?.model ?? [];
    let current: string;
    let validate: (value: string) => string | undefined;
    if (node.kind === "entry" && node.entry.kind === "phrase") {
      current = node.entry.key;
      validate = validKey(entries.filter((e) => e.kind === "phrase").map((e) => e.key));
    } else if (node.kind === "value") {
      current = node.value.value;
      validate = validValue((node.dimension.values ?? []).map((v) => v.value));
    } else {
      return;
    }
    const asked = await runWizard(
      `Rename ${current}`,
      {
        steps: () => [newNameStep(current, validate)],
        args: (answers) => ({ newName: text(answers, "newName")?.trim() }),
      },
      prompter,
    );
    if (!asked || "error" in asked || typeof asked.args.newName !== "string") return;
    record.done = await renameAt(
      server,
      { uri: model, position: declaration.start, newName: asked.args.newName },
      record,
      (message) => record.messages.push(message),
    );
    if (record.done) this.soon("inventory");
  }

  /**
   * Insert at the Cursor, on a phrase or a widget: runs Insert a phrase or
   * Insert a widget with it chosen, in the project's last Markdown editor at
   * its cursor, which is brought to the front. Where there's none, or the
   * action doesn't apply at its cursor, it shows the action's hint and
   * changes nothing.
   */
  private async insert(
    node: Extract<ModelNode, { kind: "entry" }>,
    prompter: Prompter,
    record: ItemRecord,
  ): Promise<void> {
    const server = this.inventoryFrom;
    const last = server && this.lastEditors.get(server.project.folder);
    let context: ContextResult | undefined;
    if (last) {
      const editor = await vscode.window.showTextDocument(vscode.Uri.parse(last.uri), {
        preview: false,
      });
      const { start, end } = last.selection;
      editor.selection = new vscode.Selection(start.line, start.character, end.line, end.character);
      try {
        context = await this.actions.contextAt(editor);
      } catch {
        context = undefined;
      }
    }
    const plan = insertPlan(node.entry, last, context);
    if (!plan) return;
    if (plan.kind === "hint") {
      record.messages.push(plan.hint);
      return;
    }
    const run = await this.actions.runWith(plan.action, plan.preset, prompter);
    if (run) record.run = run;
    record.done = run?.done ?? false;
  }

  dispose(): void {
    for (const timer of this.timers.values()) clearTimeout(timer);
    this.timers.clear();
    for (const disposable of this.disposables.splice(0)) disposable.dispose();
    this.projectChanged.dispose();
    this.inventoryChanged.dispose();
    this.decorationsChanged.dispose();
  }

  /** Drags in Pages: sections and pages, dropped on each other. */
  private pagesDrag(): vscode.TreeDragAndDropController<PagesNode> {
    const mime = "application/vnd.code.tree.ascribe.pages";
    return {
      dragMimeTypes: [],
      dropMimeTypes: [mime],
      handleDrag: (source, data) => {
        const dragged = source.filter(pagesDraggable);
        if (dragged.length === 1) data.set(mime, new vscode.DataTransferItem(dragged[0]));
      },
      handleDrop: async (target, data) => {
        const dragged = data.get(mime)?.value as PagesNode | undefined;
        if (!dragged || !target) return;
        const said = await this.drop(dragged, target, (question) => this.confirm(question));
        if (said) this.say(said);
      },
    };
  }

  /** Asks before a page's file moves. */
  private async confirm(question: string): Promise<boolean> {
    const move = "Move";
    return (await vscode.window.showWarningMessage(question, { modal: true }, move)) === move;
  }

  /** The message above each view: what it shows when there's nothing to list. */
  private showMessages(): void {
    const server = this.server;
    const result = this.inventory;
    // A server that failed to start, or that Switch Project started and
    // hasn't answered: the views name its project.
    const notice = server && viewsNotice(server.state, this.projects.name(server.project), result);
    if (notice) {
      for (const tree of [this.pages, this.model]) {
        if (tree.view) {
          tree.view.message = notice.message;
          tree.view.description = notice.description;
        }
      }
      return;
    }
    if (this.pages.view) {
      this.pages.view.message =
        server && !result
          ? "Open a page of the project to list its pages."
          : result && this.orphansOnly && this.pagesRoots().length === 0
            ? "No orphans: something links to or includes every page and fragment."
            : "";
    }
    if (this.model.view) {
      this.model.view.message =
        server && !result
          ? "Open a page of the project to list its content model."
          : result && this.unusedOnly && modelRoots(result, true).length === 0
            ? "Nothing unused: every entry is used."
            : "";
    }
  }

  private say(message: string): void {
    void vscode.window.showInformationMessage(`Ascribe: ${message}`);
  }

  /**
   * Does what dropping `dragged` on `target` does: an edit of
   * `ascribe.nav`, applied as typing is, unsaved and undoable; or, asked
   * first, a page's file moving to another section, which the server's
   * rename follows with its links and `ascribe.nav`. What it says when it
   * can't, for the caller to show.
   */
  private async drop(
    dragged: PagesNode,
    target: PagesNode,
    ask: (question: string) => Promise<boolean>,
  ): Promise<string | undefined> {
    const plan = pagesDrop(dragged, target);
    if (plan.kind === "none") return plan.why;
    if (plan.kind === "edit") return this.editNav(plan.action, plan.args);
    const content = this.inventory?.contentUri;
    if (!content) return undefined;
    const name = plan.page.slice(plan.page.lastIndexOf("/") + 1);
    if (!(await ask(`Moving \`${name}\` to ${plan.label} changes its address.`))) return undefined;
    return this.moveFile(plan.page, `${plan.folder}${name}`);
  }

  /** Place Here, on a page `ascribe.nav` leaves out: it's appended to its section's pages. */
  private placeHere(node: PagesNode): Promise<string | undefined> {
    if (node.kind !== "page" || node.placed) return Promise.resolve(undefined);
    return this.editNav("placePage", { page: node.path });
  }

  /**
   * Rename Label…, on a section or a page: the section's `label` in
   * `ascribe.nav`, or the page's `nav-label`, or none for its default.
   */
  private async renameLabel(
    node: PagesNode,
    ask: (value: string, prompt: string) => Promise<string | undefined>,
  ): Promise<string | undefined> {
    const rename = labelRename(node);
    if (!rename) return undefined;
    const label = await ask(rename.value, rename.prompt);
    if (label === undefined) return undefined;
    if (rename.action === "setSectionLabel") {
      return this.editNav("setSectionLabel", { folder: rename.folder, label: label || null });
    }
    const content = this.inventory?.contentUri;
    if (!content) return undefined;
    return this.requestEdit(contentFile(content, rename.page), rename.page, "setPageLabel", {
      label: label || null,
    });
  }

  /** Asks the server for an edit of `ascribe.nav`, and applies it; what it says when it can't. */
  private editNav(
    action: string,
    args: Record<string, string | null>,
  ): Promise<string | undefined> {
    const server = this.server;
    if (server?.state !== "running") {
      return Promise.resolve("the project's language server isn't running.");
    }
    // Asked through the project's ascribe.toml, as any of its files will do.
    return this.requestEdit(vscode.Uri.file(server.project.config), "ascribe.nav", action, args);
  }

  /**
   * Asks the server for an `ascribe/edit` operation through `uri`, and
   * applies its edit to `changed` unsaved, as typing is; what it says when
   * it can't.
   */
  private async requestEdit(
    uri: vscode.Uri,
    changed: string,
    action: string,
    args: Record<string, string | null>,
  ): Promise<string | undefined> {
    const server = this.server;
    if (server?.state !== "running") return "the project's language server isn't running.";
    const at = { line: 0, character: 0 };
    try {
      const result = await server.requestEdit("ascribe/edit", {
        textDocument: { uri: uri.toString() },
        range: { start: at, end: at },
        action,
        args,
      });
      if ("error" in result) return result.error;
      if (!(await vscode.workspace.applyEdit(result.edit))) {
        return `${changed} couldn't be changed.`;
      }
    } catch (error) {
      server.reportFeatureError("Pages", error);
      return undefined;
    }
    this.soon("inventory");
    return undefined;
  }

  /** Shows the active page in Pages, its sections expanded, without taking focus. */
  private reveal(): void {
    const view = this.pages.view;
    const uri = vscode.window.activeTextEditor?.document.uri;
    const content = this.inventory?.contentUri;
    if (!view?.visible || !uri || !content || this.inventoryFrom !== this.server) return;
    const page = contentPathOf(uri.fsPath, vscode.Uri.parse(content).fsPath);
    if (page === undefined) return;
    const node = pagesFind(this.pagesRoots(), page);
    if (!node || view.selection[0] === node) return;
    view
      .reveal(node, { select: true, focus: false, expand: true })
      .then(undefined, () => undefined);
  }

  /** The project the views show, which Switch Project marks. */
  get project(): ProjectServer | undefined {
    return this.server;
  }

  /** The inventory of the project the views show, once its server has answered. */
  get projectInventory(): InventoryResult | undefined {
    return this.inventoryFrom === this.server ? this.inventory : undefined;
  }

  /**
   * Shows Pages with Orphans Only on, or Content model with Unused Only on:
   * the lists the Project view's counts open.
   */
  async showOnly(view: "pages" | "model"): Promise<void> {
    await this.filter(view, true);
    await vscode.commands.executeCommand(`ascribe.${view}.focus`);
  }

  /**
   * Shows a project chosen with Switch Project, starting its server if it
   * hasn't started. It's a jump, not a pin: the views stay on it until a
   * file of another project becomes active.
   */
  choose(server: ProjectServer): void {
    if (server !== this.server) {
      this.server = server;
      this.projectChanged.fire(server);
    }
    this.follow(true);
    if (server.state === "stopped") void server.start();
  }

  /**
   * Follows the active editor: when it changes to a file of another project,
   * the views switch to it, and a file of no project leaves them on the last
   * one. Drawing them again otherwise keeps the project Switch Project chose.
   */
  private follow(force = false, editorChanged = false): void {
    const uri = vscode.window.activeTextEditor?.document.uri;
    const active = uri?.scheme === "file" ? this.projects.serverFor(uri) : undefined;
    const server = followedProject(this.server, active, editorChanged, this.projects.servers);
    if (server !== this.server) {
      this.server = server;
      this.projectChanged.fire(server);
      force = true;
    }
    void vscode.commands.executeCommand("setContext", FAILED, this.server?.state === "failed");
    if (force) {
      if (this.server?.state !== "running" || !this.projects.servers.includes(this.server)) {
        this.inventory = undefined;
        this.inventoryFrom = undefined;
        this.inventoryAsked++;
        this.inventoryChanged.fire();
      }
      this.soon("inventory");
    }
    this.soon("thisPage");
    this.reveal();
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
    this.inventoryChanged.fire();
    const name = server && result ? this.projects.name(server.project) : undefined;
    for (const tree of [this.pages, this.model]) {
      if (tree.view) tree.view.description = name ?? "";
    }
    this.showMessages();
    this.pages.changed.fire();
    this.model.changed.fire();
    this.reveal();
  }

  /** Whether an editor holds a page of a project, which This page can show. */
  private isPageEditor(editor: vscode.TextEditor): boolean {
    const { document } = editor;
    return (
      document.languageId === "markdown" && this.projects.serverFor(document.uri) !== undefined
    );
  }

  /** The page This page shows now: the active editor's, or the one it shows while it's open. */
  private shownPageNow(): PageAt | undefined {
    const active = vscode.window.activeTextEditor;
    const page = shownPage(
      this.pageAt?.uri.toString(),
      active && this.isPageEditor(active) ? active.document.uri.toString() : undefined,
      this.pageAt && openInTab(this.pageAt.uri) ? [this.pageAt.uri.toString()] : [],
      () => true,
    );
    if (page === undefined) return undefined;
    if (active && active.document.uri.toString() === page) {
      return { uri: active.document.uri, selection: active.selection };
    }
    return this.pageAt;
  }

  private async askThisPage(): Promise<void> {
    if (!this.thisPageView.visible) {
      // Hidden, it asks nothing: a badge for a page it no longer shows goes.
      if (!this.showsPageNow()) this.thisPageView.badge = undefined;
      return;
    }
    const looked = ++this.pageLooked;
    // The page's document, not the active editor: focus in a webview, the
    // preview's for one, leaves VS Code reporting no active editor, and a tab
    // that covers the page's leaves it with no editor at all.
    this.pageAt = this.shownPageNow();
    const shown = this.pageAt;
    const server = shown && this.projects.serverFor(shown.uri);
    if (!shown || !server) {
      this.showThisPage(undefined, { kind: "noPage" });
      return;
    }
    if (server.state === "failed") {
      this.showThisPage(undefined, { kind: "failed", project: this.projects.name(server.project) });
      return;
    }
    if (server.state !== "running") {
      this.showThisPage(undefined, { kind: "starting" });
      return;
    }
    const { line, character } = shown.selection.active;
    this.pageAsked++;
    let page: PageResult | null = null;
    try {
      page = (await server.request("ascribe/page", {
        textDocument: { uri: shown.uri.toString() },
        position: { line, character },
      })) as PageResult | null;
    } catch {
      page = null;
    }
    const before = this.review.changesRequests;
    const review = page?.about ? await this.reviewOf(server, page.about.path) : undefined;
    this.pageAsked += this.review.changesRequests - before;
    if (looked !== this.pageLooked) return;
    this.pageFile = shown.uri;
    this.showThisPage(
      page?.about ? page : undefined,
      page?.about ? undefined : { kind: "noPage" },
      review,
    );
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

  /** Shows an answer in This page, or a notice in place of one. */
  private showThisPage(
    page: PageResult | undefined,
    notice: ThisPageNotice | undefined,
    review?: ThisPageReview,
  ): void {
    if (page === undefined) this.pageLooked++;
    this.page = page;
    this.pageReview = page && review;
    this.pageNotice = notice;
    this.drawThisPage();
  }

  /** Whether This page's answer is about the page it would show now. */
  private showsPageNow(): boolean {
    return this.shownPageNow()?.uri.toString() === this.pageFile?.toString();
  }

  /**
   * Sends This page's webview the model's items as they stand, under the
   * page's head, or its notice; and sets its badge to Health's things to do.
   */
  private drawThisPage(): void {
    const roots = this.thisPage.roots();
    const { items, nodes } = thisPageItems(
      roots,
      (node) =>
        this.thisPage.opens(node) !== undefined ||
        this.thisPage.command(node) !== undefined ||
        siteOf(node) !== undefined,
    );
    const about = this.page?.about;
    this.thisPageView.badge = this.showsPageNow() ? healthBadge(roots) : undefined;
    const answer: ThisPageAnswer =
      this.pageNotice || !about || items.length === 0
        ? thisPageNotice(this.pageNotice ?? { kind: "noPage" })
        : { message: null, actions: [], head: thisPageHead(about), sections: items };
    this.pageNodes.set(answer, nodes);
    this.thisPageView.show(answer);
  }

  /**
   * Opens what an item This page drew names: a file at its place, an
   * address on the published site, or a command. Only an item of the answer
   * the webview drew, which a newer one may be on its way to replace, whose
   * key names a node of that answer that opens something; anything else the
   * webview sends is dropped.
   */
  private async chooseItem(seq: number, key: string): Promise<void> {
    const answer = this.thisPageView.answerOf(seq);
    if (answer?.actions.some((action) => action.key === key)) {
      await this.runNoticeAction(key);
      return;
    }
    const node = answer && this.pageNodes.get(answer)?.get(key);
    if (!node) return;
    const site = siteOf(node);
    if (site !== undefined) {
      await this.openOnSite(site);
      return;
    }
    const command = this.thisPage.command(node);
    if (command) {
      await vscode.commands.executeCommand(command.command, ...(command.arguments ?? []));
      return;
    }
    const target = this.thisPage.opens(node);
    if (target?.beside) {
      await this.openBeside(target.uri);
    } else if (target) {
      await vscode.commands.executeCommand(
        "vscode.open",
        target.uri,
        target.selection ? { selection: target.selection } : {},
      );
    }
  }

  /**
   * Opens an address on the published site outside VS Code: only an `http:`
   * or `https:` one, which the model built from the project's `site`.
   */
  private async openOnSite(url: string): Promise<void> {
    const uri = vscode.Uri.parse(url, true);
    if (uri.scheme !== "http" && uri.scheme !== "https") return;
    await this.openExternal(uri);
  }

  /** Runs a button under This page's notice, for the page's server. */
  private async runNoticeAction(key: string): Promise<void> {
    const server = this.pageAt && this.projects.serverFor(this.pageAt.uri);
    switch (key) {
      case NOTICE_ACTIONS.showOutput:
        server?.showOutput();
        return;
      case NOTICE_ACTIONS.restart:
        await server?.restart();
        return;
      case NOTICE_ACTIONS.manageTrust:
        await vscode.commands.executeCommand(MANAGE_TRUST);
        return;
    }
  }

  /**
   * Opens an include, a snippet's file, or a tested example's in the editor
   * group beside the page's, leaving the focus where the choice left it. It's
   * beside the page's group, not the active one: the focus is in the view, and
   * the page's tab may be in any group.
   */
  private async openBeside(uri: vscode.Uri): Promise<void> {
    const group = this.pageAt && groupOf(this.pageAt.uri);
    await vscode.window.showTextDocument(uri, {
      viewColumn: group ? group.viewColumn + 1 : vscode.ViewColumn.Beside,
      preserveFocus: true,
      preview: true,
    });
  }

  /** Escape in This page: the focus goes back to the page's editor, in the group of its tab. */
  private async focusPageEditor(): Promise<void> {
    const page = this.pageAt;
    const group = page && groupOf(page.uri);
    if (!page || !group) return;
    await vscode.window.showTextDocument(page.uri, {
      viewColumn: group.viewColumn,
      preserveFocus: false,
    });
  }
}

/** The page This page shows: its document, and its cursor as it was last. */
interface PageAt {
  uri: vscode.Uri;
  selection: vscode.Selection;
}

/** Whether a tab is the text editor of `uri`. */
function isTabOf(tab: vscode.Tab, uri: vscode.Uri): boolean {
  return tab.input instanceof vscode.TabInputText && tab.input.uri.toString() === uri.toString();
}

/** The group of `uri`'s tab: one where it's on top, or else the first that has it. */
function groupOf(uri: vscode.Uri): vscode.TabGroup | undefined {
  const { all } = vscode.window.tabGroups;
  return (
    all.find((group) => group.activeTab !== undefined && isTabOf(group.activeTab, uri)) ??
    all.find((group) => group.tabs.some((tab) => isTabOf(tab, uri)))
  );
}

/** Whether `uri` is open in a tab of some group, on top or covered by another. */
function openInTab(uri: vscode.Uri): boolean {
  return vscode.window.tabGroups.all.some((group) => group.tabs.some((tab) => isTabOf(tab, uri)));
}

/**
 * This page's webview. It's made before the folder is trusted, so in
 * Restricted Mode it can say so, with Manage Workspace Trust; once the folder
 * is trusted, `SidebarViews` draws it and takes what it asks.
 */
export function thisPageWebview(
  extensionUri: vscode.Uri,
): WebviewView<ThisPageAnswer, ThisPageReport> {
  const view = new WebviewView<ThisPageAnswer, ThisPageReport>(
    "ascribe.thisPage",
    extensionUri,
    { script: "thisPage.js", title: "This page" },
    thisPageReport,
    {
      open: (seq, key) => {
        if (view.answerOf(seq) && key === NOTICE_ACTIONS.manageTrust) {
          void vscode.commands.executeCommand(MANAGE_TRUST);
        }
      },
      focusEditor: () =>
        void vscode.commands.executeCommand("workbench.action.focusActiveEditorGroup"),
    },
  );
  return view;
}

/** The command of Manage Workspace Trust, under This page's notice in Restricted Mode. */
const MANAGE_TRUST = "workbench.trust.manage";

/** A check of what This page's webview says it drew. */
const thisPageReport: Check<ThisPageReport> = shape<ThisPageReport>({
  message: nullable(shortText),
  actions: listOf(shortText),
  head: listOf(shortText),
  code: listOf(shortText),
  sections: listOf(
    shape<ThisPageReport["sections"][number]>({
      key: shortText,
      label: shortText,
      open: boolean,
      items: listOf(shortText),
    }),
  ),
  wraps: boolean,
  focused: nullable(shortText),
  hasFocus: boolean,
  scroll: whole,
  probed: whole,
  severities: listOf(shortText),
  violations: listOf(shortText),
});

/** What Content model's Insert at the Cursor needs of the editor's actions. */
export interface Inserter {
  /** The context at an editor's selection, as an action asks for it. */
  contextAt(editor: vscode.TextEditor): Promise<ContextResult>;
  /** Runs an action in the active editor with some of its answers given, the rest asked by `prompter`'s. */
  runWith(action: Action, preset: Answers, prompter?: Prompter): Promise<RunRecord | undefined>;
}

/** The commands of Pages' items and title bar. */
const PAGES_COMMANDS = {
  openPreview: "ascribe.pages.openPreview",
  copyLink: "ascribe.pages.copyLink",
  renameOrMove: "ascribe.pages.renameOrMove",
  findReferences: "ascribe.pages.findReferences",
  revealInExplorer: "ascribe.pages.revealInExplorer",
  newPageHere: "ascribe.pages.newPageHere",
  orphansOnly: "ascribe.pages.orphansOnly",
  showAll: "ascribe.pages.showAll",
} as const;

/** The commands of Content model's items and title bar. */
const MODEL_COMMANDS = {
  findReferences: "ascribe.model.findReferences",
  rename: "ascribe.model.rename",
  insert: "ascribe.model.insert",
  unusedOnly: "ascribe.model.unusedOnly",
  showAll: "ascribe.model.showAll",
} as const;

/** The context keys the title bars' toggles are shown by. */
const ORPHANS_ONLY = "ascribe.pages.orphansOnly";
const UNUSED_ONLY = "ascribe.model.unusedOnly";

/** Place Here, in the menu of a page `ascribe.nav` leaves out. */
const PLACE_HERE = "ascribe.pages.placeHere";

/** Rename Label…, in a section's menu. */
const RENAME_LABEL = "ascribe.pages.renameLabel";

/** Whether a file is a project's `ascribe.nav`. */
function isNavFile(uri: vscode.Uri): boolean {
  return uri.scheme === "file" && uri.path.endsWith("/ascribe.nav");
}

/** The address on the published site a row of This page opens, if it opens one. */
function siteOf(node: ThisPageNode): string | undefined {
  return node.kind === "row" && node.row.opens?.kind === "site" ? node.row.opens.url : undefined;
}

/** The command a thread in This page's Review section runs. */
const OPEN_THREAD = "ascribe.review.openThread";

/** The context key set while the views' project's server has failed, for their welcome. */
const FAILED = "ascribe.sidebar.failed";

/** The commands of the views' welcome while their project's server has failed. */
const SHOW_OUTPUT = "ascribe.sidebar.showOutput";
const RESTART = "ascribe.sidebar.restart";

/** The page's diagnostics, counted as the Problems panel shows them. */
function problemsOf(uri: vscode.Uri): ProblemCounts {
  const counts: ProblemCounts = { errors: 0, warnings: 0, advice: 0 };
  for (const diagnostic of vscode.languages.getDiagnostics(uri)) {
    if (diagnostic.severity === vscode.DiagnosticSeverity.Error) counts.errors++;
    else if (diagnostic.severity === vscode.DiagnosticSeverity.Warning) counts.warnings++;
    // A hint isn't in the Problems panel.
    else if (diagnostic.severity === vscode.DiagnosticSeverity.Information) counts.advice++;
  }
  return counts;
}

/** The URI of a content path under the content root. */
function contentFile(content: string, file: string): vscode.Uri {
  return vscode.Uri.joinPath(vscode.Uri.parse(content), ...file.split("/"));
}
