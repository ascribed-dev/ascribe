// The pull request's review threads in the preview: @ascribed/review's
// overlay on the page, with its host (`OverlayHost`) answered by the extension
// over the webview's messages. The webview never sees a token and makes no
// requests: every read and write of GitHub happens in the extension.
//
// The overlay lasts across renders, so a draft survives the page re-rendering
// as the author types; each render tells it to read the threads again.

import { findBlock } from "@ascribed/review/marks";
import {
  createOverlay,
  type Anchor,
  type HostError,
  type Overlay,
  type OverlayData,
  type OverlayHost,
  type PromptRequest,
} from "@ascribed/review/overlay";
import type {
  Change,
  FromWebview,
  ThreadsMethod,
  ThreadsReport,
  ThreadsView,
  ToWebview,
} from "../preview/protocol.js";
import { DATA_SOURCE, DATA_UI, DATA_VIA } from "../names.js";

export class Threads {
  private overlay: Overlay | undefined;
  /** The pull request the overlay is for. */
  private number: number | undefined;
  private listeners: (() => void)[] = [];
  private readonly waiting = new Map<
    number,
    { resolve: (value: unknown) => void; reject: (error: HostError) => void }
  >();
  private seq = 0;
  private changes: Change[] = [];
  private data: OverlayData | undefined;
  private reported: OverlayData | undefined;
  /** The thread count the header last showed. */
  private shownCount: number | undefined;

  constructor(
    private readonly content: HTMLElement,
    private readonly post: (message: FromWebview) => void,
    /** Called after the overlay redraws: the header shows the thread count. */
    private readonly onUpdate: () => void,
  ) {}

  /**
   * Shows the threads for `view` on the page just put in `content`, whose
   * changes (`changes`) the marks drew; `null` takes the overlay away.
   */
  draw(view: ThreadsView | null, changes: Change[]): void {
    this.changes = changes;
    if (view?.state !== "on" || view.pullRequest === null) {
      this.dispose();
      return;
    }
    if (this.overlay && this.number === view.pullRequest.number) {
      // The page re-rendered: the overlay reads the threads again.
      for (const listener of this.listeners) listener();
    } else {
      this.dispose();
      this.number = view.pullRequest.number;
      this.overlay = createOverlay({
        root: this.content,
        host: this.host(),
        onUpdate: () => {
          this.report();
          const count = this.overlay?.threadCount();
          if (count !== this.shownCount) {
            this.shownCount = count;
            this.onUpdate();
          }
        },
      });
    }
    if (view.goTo !== null) this.overlay.goToThread(view.goTo);
  }

  /** Whether the overlay is on the page. */
  get active(): boolean {
    return this.overlay !== undefined;
  }

  /** How many threads the pull request has on its pages, once read. */
  count(): number | undefined {
    return this.overlay?.threadCount();
  }

  /** Opens the list of every thread. */
  showAll(): void {
    this.overlay?.showAllComments();
  }

  /** Asks the extension to build and deliver a prompt; rejects with what went wrong. */
  prompt(request: PromptRequest): Promise<void> {
    return this.request("promptAgent", { request });
  }

  /** Places the threads again: the page moved under them (the header grew, what shows changed). */
  layout(): void {
    this.overlay?.layout();
  }

  /** A message from the extension for the threads. */
  receive(message: ToWebview): void {
    if (message.type === "threadsResult") {
      const waiter = this.waiting.get(message.id);
      if (!waiter) return;
      this.waiting.delete(message.id);
      if (message.error) waiter.reject(message.error);
      else waiter.resolve(message.result);
    } else if (message.type === "threadsChanged") {
      for (const listener of this.listeners) listener();
    } else if (message.type === "goToThread") {
      this.overlay?.goToThread(message.threadId);
    }
  }

  private dispose(): void {
    this.overlay?.dispose();
    this.overlay = undefined;
    this.number = undefined;
    this.listeners = [];
    this.data = undefined;
    this.shownCount = undefined;
    for (const waiter of this.waiting.values()) waiter.reject({ message: "Review stopped." });
    this.waiting.clear();
  }

  private request<T>(method: ThreadsMethod, params: Record<string, unknown> = {}): Promise<T> {
    const id = ++this.seq;
    return new Promise<T>((resolve, reject) => {
      this.waiting.set(id, { resolve: resolve as (value: unknown) => void, reject });
      this.post({ type: "threads", id, method, params });
    });
  }

  private host(): OverlayHost {
    return {
      load: async () => {
        const data = await this.request<OverlayData>("load", {
          anchors: this.anchors(),
          removed: this.removed(),
        });
        this.data = data;
        return data;
      },
      commentTarget: (anchor) => this.request("commentTarget", { anchor }),
      comment: (anchor, body, quote) => this.request("comment", { anchor, body, quote }),
      reply: (threadId, body, when) => this.request("reply", { threadId, body, when }),
      allThreads: () => this.request("allThreads"),
      resolve: (threadId, resolved) => this.request("resolve", { threadId, resolved }),
      submit: (event, body) =>
        this.request("submit", body === undefined ? { event } : { event, body }),
      discard: () => this.request("discard"),
      promptAgent: (request) => this.prompt(request),
      openSource: (anchor) => this.post({ type: "openSource", source: anchor.source }),
      openThread: (threadId, path) => this.post({ type: "openThread", threadId, path }),
      openLink: (url) => this.post({ type: "open", href: url }),
      notify: (message) => this.post({ type: "notify", message }),
      onDidChange: (listener) => {
        this.listeners.push(listener);
      },
    };
  }

  /** Every anchored block on the page, in its order: the marks' and overlay's own elements aside. */
  private anchors(): Anchor[] {
    const anchors: Anchor[] = [];
    for (const element of this.content.querySelectorAll(`[${DATA_SOURCE}]`)) {
      if (element.closest(`[${DATA_UI}]`)) continue;
      anchors.push(anchorOf(element));
    }
    return anchors;
  }

  /** Where the page's removed, moved, and changed blocks were, for threads on their old text. */
  private removed(): Anchor[] {
    const removed: Anchor[] = [];
    for (const change of this.changes) {
      if (change.kind !== "added" && change.was) {
        removed.push({ source: change.was.source, via: change.was.via });
      }
    }
    return removed;
  }

  /** Tells the extension what the overlay drew, for tests. */
  private report(): void {
    const data = this.data;
    if (!data || data === this.reported) return;
    this.reported = data;
    const report: ThreadsReport = { blocks: {}, detached: [], unsent: data.pending.count };
    for (const { anchor, threads } of data.threads.blocks) {
      if (findBlock(this.content, anchor) === null) continue;
      report.blocks[anchor.source] = threads.map((t) => t.id);
    }
    report.detached = data.threads.detached.map((t) => t.id);
    this.post({ type: "threadsDrawn", report });
  }
}

function anchorOf(element: Element): Anchor {
  return {
    source: element.getAttribute(DATA_SOURCE) ?? "",
    via: (element.getAttribute(DATA_VIA) ?? "").split(" ").filter(Boolean),
  };
}
