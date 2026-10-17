import * as vscode from "vscode";
import type { ProjectServer } from "../client.js";
import type { ChangedPage, ChangesResult } from "../preview/protocol.js";
import type { ReviewController } from "../preview/review.js";
import type { ProjectRegistry } from "../registry.js";
import { fromContentPath } from "../preview/reviewText.js";
import { SourceChanges, type SourceChangesApi } from "../preview/sourceChanges.js";
import {
  badgeTooltip,
  reviewChildren,
  reviewLook,
  reviewRoots,
  sourceFiles,
  type PageProblem,
  type ProblemSeverity,
  type ReviewNode,
  type ReviewPage,
} from "./reviewTree.js";
import type { SidebarItem } from "./sidebarViews.js";

/** How long after the last change the view asks for the changed pages again. */
const REFRESH_DEBOUNCE_MS = 300;

/** The context key for whether Problems on these pages lists advice too. */
const ADVICE_KEY = "ascribe.review.advice";
/** The context key for whether the review has comments to submit. */
const UNSENT_KEY = "ascribe.review.unsent";

/** The Review view, for tests. */
export interface ReviewViewApi {
  /** Every item, with every node expanded. */
  items(): SidebarItem[];
  /** The message above its items, if it shows one. */
  message(): string | undefined;
  /** The badge's number and tooltip, when it shows one. */
  badge(): { value: number; tooltip: string } | undefined;
  /** The project the view shows, by folder: `undefined` while review is off, when the view is gone. */
  project(): string | undefined;
  /** Shows advice in Problems on these pages, or not, as the title bar's toggle does. */
  showAdvice(show: boolean): Promise<void>;
  /** Open Source Changes. */
  sourceChanges: SourceChangesApi;
  /** Settles when every refresh that is due has finished. */
  whenSettled(): Promise<void>;
}

/**
 * The Review view, first in the Ascribe sidebar while review is on: the
 * changed pages of the active project's review, under Added, Changed, and
 * Removed, with their counts and open threads, then the problems on them.
 * The pages are the server's (`ascribe/review/changes`), as Changed Pages
 * lists them; the problems are VS Code's diagnostics for the pages' files.
 * Its badge counts the comments not yet submitted.
 */
export class ReviewView implements vscode.Disposable, vscode.TreeDataProvider<ReviewNode> {
  private readonly changed = new vscode.EventEmitter<void>();
  readonly onDidChangeTreeData = this.changed.event;
  private view: vscode.TreeView<ReviewNode> | undefined;
  /** The project shown, and what it answered. */
  private server: ProjectServer | undefined;
  private changes: ChangesResult | undefined;
  private pages: ReviewPage[] = [];
  private advice = false;
  private asked = 0;
  private timer: ReturnType<typeof setTimeout> | undefined;
  private readonly pending = new Set<Promise<void>>();
  private readonly disposables: vscode.Disposable[] = [];
  private readonly sources: SourceChanges;

  constructor(
    private readonly review: ReviewController,
    projects: ProjectRegistry,
  ) {
    this.sources = new SourceChanges(projects);
  }

  get api(): ReviewViewApi {
    return {
      items: () => this.items(),
      message: () => this.view?.message || undefined,
      badge: () => {
        const badge = this.view?.badge;
        return badge ? { value: badge.value, tooltip: badge.tooltip } : undefined;
      },
      project: () => (this.changes ? this.server?.project.folder : undefined),
      showAdvice: async (show) => {
        await this.setAdvice(show);
      },
      sourceChanges: this.sources.api,
      whenSettled: () => this.whenSettled(),
    };
  }

  register(): void {
    this.sources.register();
    this.view = vscode.window.createTreeView("ascribe.review", { treeDataProvider: this });
    void vscode.commands.executeCommand("setContext", ADVICE_KEY, this.advice);
    this.disposables.push(
      this.view,
      this.changed,
      this.sources,
      vscode.commands.registerCommand("ascribe.review.openPage", (node?: ReviewNode) =>
        node?.kind === "page" ? this.openPage(node.page.page) : undefined,
      ),
      vscode.commands.registerCommand("ascribe.review.pageSourceChanges", (node?: ReviewNode) =>
        node?.kind === "page" ? this.openSources([node.page.page]) : undefined,
      ),
      vscode.commands.registerCommand("ascribe.openSourceChanges", () =>
        this.openSources(undefined),
      ),
      vscode.commands.registerCommand("ascribe.submitReview", () => this.submit()),
      vscode.commands.registerCommand("ascribe.review.showAdvice", () => this.setAdvice(true)),
      vscode.commands.registerCommand("ascribe.review.hideAdvice", () => this.setAdvice(false)),
      this.review.onDidUpdate(() => this.soon()),
      vscode.workspace.onDidChangeTextDocument(({ document }) => {
        if (this.server && this.review.current() === this.server && isSource(document)) {
          this.soon();
        }
      }),
      vscode.languages.onDidChangeDiagnostics(({ uris }) => {
        const files = new Set(this.pageFiles().map((uri) => uri.toString()));
        if (uris.some((uri) => files.has(uri.toString()))) this.changed.fire();
      }),
    );
    this.soon();
  }

  dispose(): void {
    clearTimeout(this.timer);
    this.timer = undefined;
    for (const disposable of this.disposables.splice(0)) disposable.dispose();
  }

  getTreeItem(node: ReviewNode): vscode.TreeItem {
    const look = reviewLook(node);
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
    if (node.kind === "page") {
      item.contextValue = "ascribe.review.page";
      item.command = { command: "ascribe.review.openPage", title: "Open", arguments: [node] };
    } else {
      const target = this.opens(node);
      if (target) {
        item.command = {
          command: "vscode.open",
          title: "Open",
          arguments: [target.uri, { selection: target.selection }],
        };
      }
    }
    return item;
  }

  getChildren(node?: ReviewNode): ReviewNode[] {
    if (node) return reviewChildren(node);
    return reviewRoots(this.pages, this.problems(), this.advice);
  }

  private items(node?: ReviewNode): SidebarItem[] {
    return this.getChildren(node).map((child) => {
      const target = this.opens(child);
      return {
        ...reviewLook(child),
        opens:
          child.kind === "page"
            ? this.pageUri(child.page.page)?.toString()
            : target && `${target.uri.toString()}:${target.selection.start.line + 1}`,
        children: this.items(child),
      };
    });
  }

  /** Where a problem item opens. */
  private opens(node: ReviewNode): { uri: vscode.Uri; selection: vscode.Range } | undefined {
    if (node.kind !== "problem" || !this.changes?.contentRoot) return undefined;
    const { problem } = node;
    const position = new vscode.Position(problem.line, problem.character);
    return {
      uri: vscode.Uri.file(fromContentPath(this.changes.contentRoot, problem.path)),
      selection: new vscode.Range(position, position),
    };
  }

  /** A changed page's file, while it has one. */
  private pageUri(page: ChangedPage): vscode.Uri | undefined {
    const root = this.changes?.contentRoot;
    if (!root || page.status === "removed") return undefined;
    return vscode.Uri.file(fromContentPath(root, page.path));
  }

  /** The files of the changed pages that have one. */
  private pageFiles(): vscode.Uri[] {
    return this.pages.flatMap(({ page }) => this.pageUri(page) ?? []);
  }

  /** VS Code's diagnostics for the changed pages' files. */
  private problems(): PageProblem[] {
    const out: PageProblem[] = [];
    for (const { page } of this.pages) {
      const uri = this.pageUri(page);
      if (!uri) continue;
      for (const diagnostic of vscode.languages.getDiagnostics(uri)) {
        out.push({
          path: page.path,
          severity: SEVERITIES[diagnostic.severity],
          message: diagnostic.message,
          line: diagnostic.range.start.line,
          character: diagnostic.range.start.character,
        });
      }
    }
    return out;
  }

  private async setAdvice(show: boolean): Promise<void> {
    this.advice = show;
    await vscode.commands.executeCommand("setContext", ADVICE_KEY, show);
    this.changed.fire();
  }

  /** Opens a changed page and its preview at its first change; a removed page, its source diff. */
  private async openPage(page: ChangedPage): Promise<void> {
    const uri = this.pageUri(page);
    if (uri) await this.review.showPage(uri);
    else await this.openSources([page]);
  }

  /** Opens the source changes of `pages`, or of every changed page, in one diff. */
  private async openSources(pages: readonly ChangedPage[] | undefined): Promise<void> {
    const server = this.review.current();
    const base = server && this.review.baseOf(server);
    if (!server || !base) {
      void vscode.window.showInformationMessage("Ascribe: review is off for this project.");
      return;
    }
    let changes = this.server === server ? this.changes : undefined;
    if (!pages || !changes) {
      try {
        changes = await this.review.changesOf(server);
      } catch (error) {
        server.reportFeatureError("review", error);
        return;
      }
    }
    const files = sourceFiles(pages ?? changes.pages, {
      projectRoot: server.project.folder,
      contentRoot: changes.contentRoot,
    });
    await this.sources.open(base, this.review.baseNameOf(server, base), files);
  }

  /** Submit Review…: the preview's submit dialog, on the page it shows or the first changed page. */
  private async submit(): Promise<void> {
    const server = this.review.current();
    if (!server) {
      void vscode.window.showInformationMessage("Ascribe: review is off for this project.");
      return;
    }
    const unsent = await this.review.threads.unsent(server);
    if (unsent === undefined) {
      void vscode.window.showInformationMessage(
        "Ascribe: this review has no pull request on GitHub to submit to.",
      );
      return;
    }
    if (unsent === 0) {
      void vscode.window.showInformationMessage(
        "Ascribe: there's nothing to submit yet. Comment on a block in the preview first.",
      );
      return;
    }
    const first = this.pages.find(({ page }) => page.status !== "removed");
    await this.review.submitReview(server, first && this.pageUri(first.page));
  }

  private soon(): void {
    clearTimeout(this.timer);
    this.timer = setTimeout(() => {
      this.timer = undefined;
      const done = this.refresh().finally(() => this.pending.delete(done));
      this.pending.add(done);
    }, REFRESH_DEBOUNCE_MS);
  }

  private async whenSettled(): Promise<void> {
    while (this.timer !== undefined || this.pending.size > 0) {
      await Promise.all(this.pending);
      if (this.timer !== undefined) await new Promise((resolve) => setTimeout(resolve, 50));
    }
  }

  /** Asks the project's server for its changed pages, and GitHub for their threads. */
  private async refresh(): Promise<void> {
    const asked = ++this.asked;
    const server = this.review.current();
    let changes: ChangesResult | undefined;
    let pages: ReviewPage[] = [];
    let unsent: number | undefined;
    if (server?.state === "running") {
      try {
        changes = await this.review.changesOf(server);
      } catch {
        changes = undefined;
      }
      if (changes) {
        const [threads, count] = await Promise.all([
          this.review.threads.openThreads(server, changes.pages),
          this.review.threads.unsent(server),
        ]);
        pages = changes.pages.map((page) => ({
          page,
          openThreads: threads && (threads.get(page.path) ?? 0),
        }));
        unsent = count;
      }
    }
    if (asked !== this.asked) return;
    this.server = server;
    this.changes = changes;
    this.pages = pages;
    if (this.view) {
      const base = server && this.review.baseOf(server);
      this.view.description =
        server && base ? `against ${this.review.baseNameOf(server, base)}` : "";
      this.view.message = !server
        ? ""
        : !changes
          ? "Open a page of the project to list its changed pages."
          : changes.problem !== null
            ? changes.problem
            : changes.pages.length === 0
              ? `No page changed in ${changes.build}.`
              : "";
      const tooltip = badgeTooltip(unsent);
      this.view.badge = tooltip && unsent ? { value: unsent, tooltip } : undefined;
    }
    void vscode.commands.executeCommand("setContext", UNSENT_KEY, (unsent ?? 0) > 0);
    this.changed.fire();
  }
}

/** VS Code's severities, by their number, as the view names them. */
const SEVERITIES: Record<vscode.DiagnosticSeverity, ProblemSeverity> = {
  [vscode.DiagnosticSeverity.Error]: "error",
  [vscode.DiagnosticSeverity.Warning]: "warning",
  [vscode.DiagnosticSeverity.Information]: "info",
  [vscode.DiagnosticSeverity.Hint]: "hint",
};

/** A document whose edit can change the changed pages: a page, a fragment, or the content model. */
function isSource(document: vscode.TextDocument): boolean {
  return (
    document.uri.scheme === "file" &&
    (document.languageId === "markdown" || document.fileName.endsWith("ascribe.toml"))
  );
}
