// The pull request's review threads on source lines, in the editor: the same
// threads, from the same session, as the preview's overlay, so a reply in one
// shows in the other. Each thread can be replied to (at once, or with the
// review) and resolved; any line of a page can be commented on, into the same
// pending review.
//
// The GitHub Pull Requests extension shows these threads already, so with
// `ascribe.review.sourceComments` at `auto` they show only when it isn't active.

import * as path from "node:path";
import * as vscode from "vscode";
import { formatSource, type LocatedThread } from "@ascribed/review/place";
import type { ProjectServer } from "../client.js";
import { comparable, within } from "../projects.js";
import type { ProjectRegistry } from "../registry.js";
import { fromContentPath } from "./reviewText.js";
import { messageOf, type ThreadsController } from "./threads.js";
import { imagesAsLinks, showSourceComments } from "./threadsText.js";

/** The GitHub Pull Requests extension, which shows review threads on source lines itself. */
export const PULL_REQUESTS_EXTENSION = "GitHub.vscode-pull-request-github";

/** A thread in the editor, as the tests see it. */
export interface SourceThreadRecord {
  id: string;
  file: string;
  /** From 0. */
  line: number;
  label: string | undefined;
  bodies: string[];
  resolved: boolean;
}

interface Shown {
  server: ProjectServer;
  thread: vscode.CommentThread;
  data: LocatedThread;
}

export class SourceComments implements vscode.Disposable {
  private controller: vscode.CommentController | undefined;
  /** Each project's threads in the editor, by thread id. */
  private readonly shown = new Map<string, Map<string, Shown>>();
  /** The editor's threads, back to what they show. */
  private readonly byThread = new WeakMap<vscode.CommentThread, Shown>();
  private disposables: vscode.Disposable[] = [];

  constructor(
    private readonly projects: ProjectRegistry,
    private readonly threads: ThreadsController,
  ) {}

  register(): void {
    this.disposables.push(
      vscode.commands.registerCommand("ascribe.review.replyNow", (reply: vscode.CommentReply) =>
        this.reply(reply, "now"),
      ),
      vscode.commands.registerCommand("ascribe.review.addToReview", (reply: vscode.CommentReply) =>
        this.reply(reply, "withReview"),
      ),
      vscode.commands.registerCommand("ascribe.review.resolve", (thread: vscode.CommentThread) =>
        this.resolve(thread, true),
      ),
      vscode.commands.registerCommand("ascribe.review.reopen", (thread: vscode.CommentThread) =>
        this.resolve(thread, false),
      ),
      this.threads.onDidChange(({ server }) => void this.reload(server)),
      vscode.workspace.onDidChangeConfiguration((event) => {
        if (event.affectsConfiguration("ascribe.review.sourceComments")) this.reloadAll();
      }),
      vscode.extensions.onDidChange(() => this.reloadAll()),
    );
  }

  /** The threads shown in the editor, for tests. */
  records(): SourceThreadRecord[] {
    const out: SourceThreadRecord[] = [];
    for (const threads of this.shown.values()) {
      for (const { thread, data } of threads.values()) {
        out.push({
          id: data.id,
          file: thread.uri.fsPath,
          line: thread.range?.start.line ?? -1,
          label: thread.label || undefined,
          bodies: thread.comments.map((c) => (typeof c.body === "string" ? c.body : c.body.value)),
          resolved: thread.state === vscode.CommentThreadState.Resolved,
        });
      }
    }
    return out;
  }

  /** Whether the editor shows threads, by the setting and the GitHub Pull Requests extension. */
  private enabled(): boolean {
    const setting = vscode.workspace.getConfiguration("ascribe.review").get("sourceComments");
    const pullRequests = vscode.extensions.getExtension(PULL_REQUESTS_EXTENSION);
    return showSourceComments(setting, pullRequests?.isActive ?? false);
  }

  private reloadAll(): void {
    for (const server of this.projects.servers) void this.reload(server);
  }

  /** Shows the project's threads again: they changed, or review started or stopped. */
  private async reload(server: ProjectServer): Promise<void> {
    const id = comparable(server.project.folder);
    const connection = this.threads.connection(server);
    if (!this.enabled() || connection?.state !== "on") {
      this.clear(id);
      if (!this.enabled() && this.shown.size === 0) this.disposeController();
      return;
    }
    let located: LocatedThread[];
    try {
      located = await connection.session.allThreads();
    } catch {
      // The preview says why; the editor keeps what it shows.
      return;
    }
    // Stopped or reconnected meanwhile.
    if (this.threads.connection(server) !== connection) return;
    const controller = this.ensureController();
    const shown = this.shown.get(id) ?? new Map<string, Shown>();
    this.shown.set(id, shown);
    const keep = new Set<string>();
    for (const data of located) {
      // Threads on removed text are on lines of the base, not of the file.
      if (data.lines === undefined || (data.kind === "review" && data.side === "LEFT")) continue;
      keep.add(data.id);
      const uri = vscode.Uri.file(fromContentPath(connection.contentRoot, data.path));
      const range = new vscode.Range(data.lines.first - 1, 0, data.lines.last - 1, 0);
      let entry = shown.get(data.id);
      if (entry && entry.thread.uri.fsPath !== uri.fsPath) {
        entry.thread.dispose();
        entry = undefined;
      }
      if (!entry) {
        const thread = controller.createCommentThread(uri, range, []);
        entry = { server, thread, data };
        shown.set(data.id, entry);
        this.byThread.set(thread, entry);
      }
      entry.data = data;
      this.draw(entry, range);
    }
    for (const [threadId, entry] of shown) {
      if (keep.has(threadId)) continue;
      entry.thread.dispose();
      shown.delete(threadId);
    }
  }

  private draw(entry: Shown, range: vscode.Range): void {
    const { thread, data } = entry;
    thread.range = range;
    thread.comments = data.comments.map((comment) => ({
      body: untrusted(comment.body),
      mode: vscode.CommentMode.Preview,
      author: {
        name: comment.author?.login ?? "ghost",
        ...(comment.author?.avatarUrl
          ? { iconPath: vscode.Uri.parse(comment.author.avatarUrl) }
          : {}),
      },
      ...(comment.pending ? { label: "Unsent" } : {}),
      timestamp: new Date(comment.createdAt),
    }));
    // Who's in the thread, then its state.
    const who = [...new Set(data.comments.map((c) => c.author?.login ?? "ghost"))].join(", ");
    const labels: string[] = who ? [who] : [];
    if (data.kind === "conversation") labels.push("In the review summary");
    if (data.outdated) labels.push("Outdated");
    if (data.comments.some((c) => c.pending)) labels.push("Unsent");
    thread.label = labels.join(" · ");
    thread.state = data.resolved
      ? vscode.CommentThreadState.Resolved
      : vscode.CommentThreadState.Unresolved;
    thread.canReply = data.kind === "review" && data.canReply;
    const context = [
      data.kind === "review" && data.canReply ? "canReply" : "",
      data.kind === "review" && !data.resolved && data.canResolve ? "canResolve" : "",
      data.kind === "review" && data.resolved && data.canUnresolve ? "canUnresolve" : "",
    ];
    thread.contextValue = context.filter(Boolean).join(" ");
    thread.collapsibleState = data.resolved
      ? vscode.CommentThreadCollapsibleState.Collapsed
      : thread.collapsibleState;
  }

  private ensureController(): vscode.CommentController {
    if (this.controller) return this.controller;
    const controller = vscode.comments.createCommentController("ascribe.review", "Ascribe Review");
    controller.options = {
      prompt: "Comment on these lines…",
      placeHolder: "Nothing is sent until you submit the review.",
    };
    controller.commentingRangeProvider = {
      provideCommentingRanges: (document) => this.commentingRanges(document),
    };
    this.controller = controller;
    return controller;
  }

  /** Every line of a page in a project whose pull request is open: the session decides where each comment goes. */
  private commentingRanges(document: vscode.TextDocument): vscode.Range[] {
    if (document.uri.scheme !== "file" || document.languageId !== "markdown") return [];
    const located = this.locate(document.uri);
    if (!located) return [];
    return [new vscode.Range(0, 0, Math.max(0, document.lineCount - 1), 0)];
  }

  /** The project and content path of a file in a connected project's content root. */
  private locate(uri: vscode.Uri): { server: ProjectServer; contentPath: string } | undefined {
    const server = this.projects.serverFor(uri);
    const connection = server && this.threads.connection(server);
    if (!server || connection?.state !== "on") return undefined;
    if (!within(uri.fsPath, connection.contentRoot)) return undefined;
    const contentPath = path.relative(connection.contentRoot, uri.fsPath).split(path.sep).join("/");
    return { server, contentPath };
  }

  /** A reply to a thread, or the first comment of a new one (which goes into the review). */
  private async reply(reply: vscode.CommentReply, when: "now" | "withReview"): Promise<void> {
    const body = reply.text.trim();
    if (body === "") return;
    const entry = this.byThread.get(reply.thread);
    try {
      if (entry) {
        await this.threads.handle(
          entry.server,
          undefined,
          "reply",
          { threadId: entry.data.id, body, when },
          "source",
        );
        vscode.window.setStatusBarMessage(
          when === "now"
            ? "$(comment-discussion) Reply sent to GitHub."
            : "$(comment-discussion) Added to your review. Nothing is sent until you submit the review.",
          8000,
        );
        return;
      }
      const located = this.locate(reply.thread.uri);
      const range = reply.thread.range;
      if (!located || !range) throw new Error("This file isn't in the pull request's project.");
      const anchor = {
        source: formatSource({
          path: located.contentPath,
          first: range.start.line + 1,
          last: range.end.line + 1,
        }),
        via: [],
      };
      await this.threads.handle(located.server, undefined, "comment", { anchor, body }, "source");
      // The session's thread takes the new one's place.
      reply.thread.dispose();
      vscode.window.setStatusBarMessage(
        "$(comment-discussion) Added to your review. Nothing is sent until you submit the review.",
        8000,
      );
    } catch (error) {
      void vscode.window.showWarningMessage(`Ascribe: ${messageOf(error)}`);
    }
  }

  private async resolve(thread: vscode.CommentThread, resolved: boolean): Promise<void> {
    const entry = this.byThread.get(thread);
    if (!entry) return;
    try {
      await this.threads.handle(
        entry.server,
        undefined,
        "resolve",
        { threadId: entry.data.id, resolved },
        "source",
      );
      vscode.window.setStatusBarMessage(
        resolved
          ? "$(check) Resolved on GitHub. Resolving is sent right away, not held with your review."
          : "$(issue-reopened) Reopened on GitHub.",
        8000,
      );
    } catch (error) {
      void vscode.window.showWarningMessage(`Ascribe: ${messageOf(error)}`);
    }
  }

  private clear(id: string): void {
    for (const { thread } of this.shown.get(id)?.values() ?? []) thread.dispose();
    this.shown.delete(id);
  }

  private disposeController(): void {
    this.controller?.dispose();
    this.controller = undefined;
  }

  dispose(): void {
    for (const threads of this.shown.values()) {
      for (const { thread } of threads.values()) thread.dispose();
    }
    this.shown.clear();
    this.disposeController();
    for (const d of this.disposables) d.dispose();
    this.disposables = [];
  }
}

/**
 * A comment's body: Markdown from other people, so no commands, no HTML, no
 * theme icons, and images as links (as in the preview), so nothing loads
 * until the reader asks.
 */
function untrusted(body: string): vscode.MarkdownString {
  const markdown = new vscode.MarkdownString(imagesAsLinks(body), false);
  markdown.isTrusted = false;
  markdown.supportHtml = false;
  return markdown;
}
