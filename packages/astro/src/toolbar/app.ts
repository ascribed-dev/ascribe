// The Ascribe review app in Astro's dev toolbar: the site preview's review.
// It marks the page's changes with @ascribed/review's marks, hosts the
// overlay for the pull request's threads, and holds the review controls in a
// panel that opens from its toolbar button. Every request goes to the dev
// server over the toolbar's channel; the page never sees a token and makes
// no request to GitHub.
//
// Review stays on across pages and reloads (the dev server holds it), and
// the panel stays as the reviewer left it in this tab.
import {
  clearMarks,
  describeSource,
  goTo,
  markChanges,
  setShow,
  showSources,
  type Mark,
  type Show,
} from "@ascribed/review/marks";
import marksCss from "@ascribed/review/marks.css?inline";
import {
  createOverlay,
  renderMarkdown,
  type HostError,
  type LocatedThread,
  type Overlay,
  type OverlayData,
  type OverlayHost,
} from "@ascribed/review/overlay";
import { defineToolbarApp } from "astro/toolbar";
import {
  APP_ID,
  CHANGED_EVENT,
  REQUEST_EVENT,
  RESULT_EVENT,
  type Change,
  type PageView,
  type Result,
  type Status,
} from "../review/protocol.js";
import {
  anchorOf,
  anchoredBlocks,
  anchorsArrived,
  contentRoot,
  pageFile,
  pageScheme,
  sourceLocation,
} from "./page.js";
import { PANEL_CSS } from "./style.js";
import {
  againstText,
  buttonLabel,
  countsText,
  nextChangedPage,
  nextPageText,
  pageDetail,
  position,
  THREAD_HASH,
  threadLocation,
  threadsNotice,
  unsentText,
} from "./text.js";

type ServerHelpers = Parameters<NonNullable<Parameters<typeof defineToolbarApp>[0]["init"]>>[2];
type AppEvents = Parameters<NonNullable<Parameters<typeof defineToolbarApp>[0]["init"]>>[1];

/** The width of the page's container from which threads show in a column beside it. */
const COLUMN_AT = 720;
/** The room between the panel and the window's edges, or the column of threads. */
const PANEL_MARGIN = 10;
const PANEL_WIDTH = 960;
/** How long a request may take before the app gives up on the dev server. */
const TIMEOUT_MS = 120_000;
/** This tab's memory: whether the panel is open, and a page to step into. */
const OPEN_KEY = "ascribe-review-open";
const FIRST_KEY = "ascribe-review-first";

const SHOWS: [Show, string][] = [
  ["changes", "Changes"],
  ["will", "As it will be"],
  ["was", "As it was"],
];

export default defineToolbarApp({
  init(canvas, app, server) {
    void new ReviewApp(canvas, app, server, document).init();
  },
});

/** Requests to the dev server, each answered by its `id`. */
class Channel {
  private seq = 0;
  private readonly waiting = new Map<
    number,
    { resolve: (value: unknown) => void; reject: (error: HostError) => void }
  >();
  readonly tab = Math.random().toString(36).slice(2);

  constructor(private readonly server: ServerHelpers) {
    server.on<Result>(RESULT_EVENT, (result) => {
      if (result.tab !== this.tab) return;
      const waiter = this.waiting.get(result.id);
      if (!waiter) return;
      this.waiting.delete(result.id);
      if (result.error) waiter.reject(result.error);
      else waiter.resolve(result.result);
    });
  }

  request<T>(method: string, params: Record<string, unknown> = {}): Promise<T> {
    const id = ++this.seq;
    return new Promise<T>((resolve, reject) => {
      const timer = setTimeout(() => {
        this.waiting.delete(id);
        reject({ message: "The dev server didn't answer. Is `astro dev` still running?" });
      }, TIMEOUT_MS);
      this.waiting.set(id, {
        resolve: (value) => {
          clearTimeout(timer);
          resolve(value as T);
        },
        reject: (error) => {
          clearTimeout(timer);
          reject(error);
        },
      });
      this.server.send(REQUEST_EVENT, { tab: this.tab, id, method, params });
    });
  }
}

type Stage =
  | { kind: "off" }
  | { kind: "starting" }
  | { kind: "failed"; message: string }
  | { kind: "on"; view: PageView };

class ReviewApp {
  private readonly channel: Channel;
  private readonly panel: HTMLElement;
  private readonly live: HTMLElement;
  private stage: Stage = { kind: "off" };
  private open = false;
  private root: HTMLElement | undefined;
  /** Whether the page's anchors arrived: else everything is listed in the panel. */
  private arrived = true;
  private marked: string | undefined;
  private marks: Mark[] = [];
  private show: Show = "changes";
  private at = -1;
  private atEnd = false;
  private breakdown = false;
  private overlay: Overlay | undefined;
  private overlayFor: number | undefined;
  private listeners: (() => void)[] = [];
  /** The threads, read for a page whose anchors didn't arrive. */
  private listed: OverlayData | undefined;
  private stopSources: (() => void) | undefined;
  private style: HTMLStyleElement | undefined;
  private padding: { scroll: string; body: string } | undefined;
  private message: string | undefined;
  private messageTimer: ReturnType<typeof setTimeout> | undefined;
  /** The scheme the site set for review's colors itself, if it did. */
  private readonly siteScheme: string | null;

  constructor(
    canvas: ShadowRoot,
    private readonly app: AppEvents,
    server: ServerHelpers,
    private readonly doc: Document,
  ) {
    this.channel = new Channel(server);
    const style = doc.createElement("style");
    style.textContent = PANEL_CSS;
    this.panel = doc.createElement("div");
    this.panel.className = "panel";
    this.panel.setAttribute("role", "region");
    this.panel.setAttribute("aria-label", "Ascribe review");
    this.live = doc.createElement("div");
    this.live.className = "sr-only";
    this.live.setAttribute("role", "status");
    this.live.setAttribute("aria-live", "polite");
    canvas.append(style, this.panel, this.live);
    this.siteScheme = doc.documentElement.getAttribute("data-ascribe-scheme");
    const win = doc.defaultView;
    win
      ?.matchMedia?.("(prefers-color-scheme: dark)")
      .addEventListener("change", () => this.render());
    win?.addEventListener("resize", () => this.placePanel());
    this.panel.addEventListener("keydown", (event) => event.stopPropagation());
    server.on<{ from: string | null }>(CHANGED_EVENT, (event) => {
      if (event.from !== this.channel.tab) void this.changedElsewhere();
    });
    app.onToggled(({ state }) => {
      this.open = state;
      remember(OPEN_KEY, state ? "1" : "0");
      if (state && this.stage.kind === "off") void this.start();
      this.render();
    });
  }

  async init(): Promise<void> {
    let status: Status;
    try {
      status = await this.channel.request<Status>("status");
    } catch {
      return;
    }
    if (!status.on) return;
    await this.load();
    if (recall(OPEN_KEY) === "1") this.app.toggleState({ state: true });
  }

  // --- Review on and off ---

  private async start(): Promise<void> {
    this.stage = { kind: "starting" };
    this.render();
    try {
      await this.channel.request<Status>("start", {});
    } catch (error) {
      this.stage = { kind: "failed", message: messageOf(error) };
      this.render();
      return;
    }
    await this.load();
  }

  private async stop(): Promise<void> {
    try {
      await this.channel.request("stop");
    } catch (error) {
      this.notify(messageOf(error));
      return;
    }
    this.clearPage();
    this.stage = { kind: "off" };
    this.render();
    this.announce("Review stopped.");
  }

  /** Another page changed the threads, or stopped review. */
  private async changedElsewhere(): Promise<void> {
    let status: Status;
    try {
      status = await this.channel.request<Status>("status");
    } catch {
      return;
    }
    if (!status.on) {
      this.clearPage();
      this.stage = { kind: "off" };
      this.render();
    } else if (this.stage.kind === "on") {
      this.didChange();
    }
  }

  /** Reads the page's changes and threads from the dev server, and draws them. */
  private async load(): Promise<void> {
    const blocks = anchoredBlocks(this.doc);
    let view: PageView;
    try {
      view = await this.channel.request<PageView>("page", {
        route: this.doc.location.pathname,
        path: pageFile(blocks),
      });
    } catch (error) {
      this.stage = { kind: "failed", message: messageOf(error) };
      this.render();
      return;
    }
    this.stage = { kind: "on", view };
    this.draw(view, blocks);
  }

  private async refresh(): Promise<void> {
    this.notify("Refreshing…");
    try {
      await this.channel.request("refresh");
    } catch (error) {
      this.notify(messageOf(error));
      return;
    }
    await this.load();
    this.didChange();
    this.notify("Refreshed.");
  }

  // --- The page ---

  private draw(view: PageView, blocks: HTMLElement[]): void {
    if (view.kind === "not-page") {
      this.clearPage();
      this.render();
      return;
    }
    const root = contentRoot(blocks);
    const changes = view.page?.changes ?? [];
    this.arrived = root !== undefined && anchorsArrived(root, blocks.length, changes);
    if (!root || !this.arrived) {
      this.clearPage();
      this.render();
      if (view.threads.state === "on") void this.listThreads(view);
      return;
    }
    this.listed = undefined;
    if (root !== this.root) this.clearPage();
    this.root = root;
    this.ensureStyle();
    const key = JSON.stringify(changes);
    if (key !== this.marked) {
      this.marked = key;
      this.at = -1;
      this.atEnd = false;
      this.marks = markChanges(root, changes);
      setShow(root, this.show);
      this.linkLabels();
    }
    this.stopSources ??= showSources(root);
    this.drawOverlay(view, root);
    this.render();
    this.goToAsked();
  }

  private drawOverlay(view: PageView, root: HTMLElement): void {
    const threads = view.threads;
    if (threads.state !== "on") {
      this.disposeOverlay();
      return;
    }
    if (this.overlay && this.overlayFor === threads.pullRequest.number) {
      this.didChange();
      return;
    }
    this.disposeOverlay();
    this.overlayFor = threads.pullRequest.number;
    this.overlay = createOverlay({
      root,
      host: this.host(),
      columnAt: COLUMN_AT,
      unsentBar: false,
      onUpdate: () => this.updated(),
    });
  }

  /**
   * The threads of a page whose anchors didn't arrive. The changes' anchors
   * stand in for the page's, so threads on the files it includes are read too.
   */
  private async listThreads(view: PageView): Promise<void> {
    const anchors = (view.page?.changes ?? []).flatMap((change) =>
      change.now
        ? [change.now, ...change.now.via.map((at) => ({ source: lineSource(at), via: [] }))]
        : [],
    );
    try {
      this.listed = await this.channel.request<OverlayData>("load", {
        path: view.path,
        anchors,
        removed: this.removed(),
      });
    } catch (error) {
      this.notify(messageOf(error));
      return;
    }
    this.updated();
  }

  /** Goes where the page was opened for: a thread, or its first change. */
  private goToAsked(): void {
    const hash = this.doc.location.hash;
    if (hash.startsWith(THREAD_HASH) && this.overlay) {
      const id = decodeURIComponent(hash.slice(THREAD_HASH.length));
      const { pathname, search } = this.doc.location;
      this.doc.defaultView?.history.replaceState(null, "", `${pathname}${search}`);
      this.overlay.goToThread(id);
    } else if (recall(FIRST_KEY) === "1") {
      remember(FIRST_KEY, "0");
      if (this.marks.length > 0) this.step(1);
    }
  }

  /** Each mark's label opens its block's source. */
  private linkLabels(): void {
    for (const mark of this.marks) {
      const source = mark.element.getAttribute("data-ascribe-source");
      const label = labelOf(mark.element);
      if (source === null || !label) continue;
      label.title = `Open ${describeSource(source, mark.element.getAttribute("data-ascribe-via"))}`;
      label.tabIndex = 0;
      label.setAttribute("role", "link");
      const open = (event: Event) => {
        event.preventDefault();
        event.stopPropagation();
        this.openSource(source);
      };
      label.addEventListener("click", open);
      label.addEventListener("keydown", (event) => {
        if (event.key === "Enter") open(event);
      });
    }
  }

  private ensureStyle(): void {
    if (this.style?.isConnected) return;
    this.style = this.doc.createElement("style");
    this.style.setAttribute("data-ascribe-review", "");
    this.style.textContent = marksCss;
    this.doc.head.append(this.style);
  }

  private clearPage(): void {
    this.disposeOverlay();
    this.stopSources?.();
    this.stopSources = undefined;
    if (this.root) {
      clearMarks(this.root);
      this.root.classList.remove("ascribe-marks");
      this.root.removeAttribute("data-ascribe-show");
    }
    this.root = undefined;
    this.marked = undefined;
    this.marks = [];
    this.listed = undefined;
    this.at = -1;
    this.atEnd = false;
    this.style?.remove();
    this.style = undefined;
    this.updated();
  }

  private disposeOverlay(): void {
    this.overlay?.dispose();
    this.overlay = undefined;
    this.overlayFor = undefined;
    this.listeners = [];
  }

  private didChange(): void {
    for (const listener of this.listeners) listener();
  }

  /** The overlay redrew, or the threads were read: the counts may have changed. */
  private updated(): void {
    const unsent = this.unsent();
    this.app.toggleNotification(unsent > 0 ? { state: true, level: "info" } : { state: false });
    labelButton(this.doc, buttonLabel(unsent));
    this.render();
  }

  private unsent(): number {
    return this.overlay?.unsentCount() ?? this.listed?.pending.count ?? 0;
  }

  // --- The overlay's host ---

  private host(): OverlayHost {
    const path = () => (this.stage.kind === "on" ? this.stage.view.path : null);
    const ask = <T>(method: string, params: Record<string, unknown> = {}) =>
      this.channel.request<T>(method, { ...params, path: path() });
    return {
      load: () => ask<OverlayData>("load", { anchors: this.anchors(), removed: this.removed() }),
      commentTarget: (anchor) => ask("commentTarget", { anchor }),
      comment: (anchor, body) => ask("comment", { anchor, body }),
      reply: (threadId, body, when) => ask("reply", { threadId, body, when }),
      allThreads: () => ask("allThreads"),
      resolve: (threadId, resolved) => ask("resolve", { threadId, resolved }),
      submit: (event, body) => ask("submit", body === undefined ? { event } : { event, body }),
      discard: () => ask("discard"),
      openSource: (anchor) => this.openSource(anchor.source),
      openThread: (threadId, pagePath) => this.openThread(threadId, pagePath),
      notify: (message) => this.notify(message),
      onDidChange: (listener) => {
        this.listeners.push(listener);
      },
    };
  }

  /** Every anchored block on the page, in its order. */
  private anchors(): { source: string; via: string[] }[] {
    return anchoredBlocks(this.doc)
      .filter((element) => this.root?.contains(element))
      .map(anchorOf);
  }

  /** Where the page's removed, moved, and changed blocks were, for threads on their old text. */
  private removed(): { source: string; via: string[] }[] {
    if (this.stage.kind !== "on") return [];
    return (this.stage.view.page?.changes ?? [])
      .filter((c: Change) => c.kind !== "added" && c.was)
      .map((c) => ({ source: c.was?.source ?? "", via: c.was?.via ?? [] }));
  }

  // --- Going places ---

  /** Opens a block's source file at its first line, in the editor Astro opens files in. */
  private openSource(source: string): void {
    if (this.stage.kind !== "on") return;
    const where = sourceLocation(source);
    if (!where) return;
    const { contentRoot: root, separator } = this.stage.view;
    const file = [root, ...where.path.split("/")].join(separator);
    void fetch(`/__open-in-editor?file=${encodeURIComponent(`${file}:${where.line}`)}`).catch(() =>
      this.notify("Couldn't open the editor."),
    );
  }

  private openThread(threadId: string, pagePath: string): void {
    const page = this.routeOf(pagePath);
    if (page === undefined) {
      this.notify("That page isn't one this site shows.");
      return;
    }
    this.doc.location.assign(`${page}${THREAD_HASH}${encodeURIComponent(threadId)}`);
  }

  private routeOf(pagePath: string): string | undefined {
    if (this.stage.kind !== "on") return undefined;
    return this.stage.view.changedPages.find((p) => p.path === pagePath && p.status !== "removed")
      ?.route;
  }

  private step(by: 1 | -1): void {
    const total = this.marks.length;
    if (by === 1 && this.at + 1 >= total) {
      this.at = total;
      this.atEnd = true;
      this.render();
      return;
    }
    this.atEnd = false;
    this.at = Math.max(0, Math.min(total - 1, this.at + by));
    if (this.show !== "changes") this.setShow("changes");
    const mark = this.marks[this.at];
    if (mark) goTo(mark.element);
    this.render();
  }

  private setShow(show: Show): void {
    this.show = show;
    if (this.root) setShow(this.root, show);
    this.overlay?.layout();
  }

  // --- Telling the reviewer ---

  private notify(message: string): void {
    this.message = message;
    if (this.messageTimer) clearTimeout(this.messageTimer);
    this.messageTimer = setTimeout(() => {
      this.message = undefined;
      this.render();
    }, 6000);
    this.announce(message);
    this.render();
  }

  private announce(message: string): void {
    this.live.textContent = "";
    this.live.textContent = message;
  }

  // --- The panel ---

  private render(): void {
    this.applyScheme();
    this.panel.hidden = !this.open;
    this.pad();
    if (!this.open) return;
    const focus = this.focused();
    const title = this.el("div", "ttl");
    title.append(this.el("span", "grow", "Ascribe review"));
    if (this.stage.kind === "on") {
      title.append(this.button("Stop Review", "", () => void this.stop()));
    }
    title.append(this.button("Close", "", () => this.app.toggleState({ state: false })));
    const parts: HTMLElement[] = [title, ...this.content()];
    if (this.message !== undefined) parts.push(this.notice(this.message));
    this.panel.replaceChildren(...parts);
    this.refocus(focus);
    this.placePanel();
    this.pad();
  }

  /**
   * Review's colors follow the page's, not the reader's system: a light-only
   * site stays light. The marks and the overlay read `data-ascribe-scheme`
   * on the page's root; a site that sets it itself keeps its own.
   */
  private applyScheme(): void {
    const site = this.siteScheme;
    const prefersDark =
      this.doc.defaultView?.matchMedia?.("(prefers-color-scheme: dark)").matches ?? false;
    const scheme =
      site === "light" || site === "dark"
        ? site
        : pageScheme(this.root ?? this.doc.body, prefersDark);
    this.panel.dataset["scheme"] = scheme;
    if (site !== null) return;
    const html = this.doc.documentElement;
    if (this.root) html.setAttribute("data-ascribe-scheme", scheme);
    else html.removeAttribute("data-ascribe-scheme");
  }

  /** Keeps the panel clear of the column of threads, over the page beside it. */
  private placePanel(): void {
    const style = this.panel.style;
    const start = this.overlay?.columnStart();
    const viewport = this.doc.documentElement.clientWidth;
    const width = Math.min(PANEL_WIDTH, viewport - 2 * PANEL_MARGIN);
    if (start === undefined || (viewport + width) / 2 <= start - PANEL_MARGIN) {
      style.removeProperty("left");
      style.removeProperty("width");
      style.removeProperty("transform");
      return;
    }
    const room = Math.max(0, start - 2 * PANEL_MARGIN);
    const fits = Math.min(PANEL_WIDTH, room);
    style.left = `${PANEL_MARGIN + (room - fits) / 2}px`;
    style.width = `${fits}px`;
    style.transform = "none";
  }

  private content(): HTMLElement[] {
    const stage = this.stage;
    switch (stage.kind) {
      case "off": {
        const row = this.el("div", "row");
        row.append(
          this.el("span", "grow", "Review is off for this project."),
          this.button("Start Review", "primary", () => void this.start()),
        );
        return [row];
      }
      case "starting":
        return [
          this.row(
            "Starting review: comparing the pages with their base and looking for the pull request…",
          ),
        ];
      case "failed": {
        const notice = this.notice(stage.message);
        notice.append(this.button("Try again", "primary", () => void this.start()));
        return [notice];
      }
      case "on":
        return stage.view.kind === "not-page" ? this.notPage(stage.view) : this.onPage(stage.view);
    }
  }

  /** The site's own route: nothing to review, and the pages that changed. */
  private notPage(view: PageView): HTMLElement[] {
    const body = this.el("div", "body");
    const { pullRequest } = againstText(view);
    const pages = view.changedPages.filter((p) => p.status !== "removed");
    body.append(
      this.el(
        "span",
        "",
        pages.length === 0
          ? "Nothing to review here: this page doesn't come from the Ascribe project, and no page changed."
          : `Nothing to review here: this page doesn't come from the Ascribe project. Pages ${pullRequest ?? "this branch"} changes:`,
      ),
    );
    if (pages.length > 0) {
      const list = this.el("ul", "");
      for (const page of pages) {
        const item = this.el("li", "");
        const link = this.el("a", "", page.route) as HTMLAnchorElement;
        link.href = page.route;
        item.append(link, ` ${pageDetail(page)}`);
        list.append(item);
      }
      body.append(list);
    }
    const parts = [body];
    if (view.problem !== null)
      parts.push(this.notice(`Couldn't compare the pages: ${view.problem}`));
    return parts;
  }

  private onPage(view: PageView): HTMLElement[] {
    const parts: HTMLElement[] = [this.controls(view)];
    if (this.breakdown && view.page) {
      parts.push(this.el("div", "legend", countsText(view.page.counts) || "No changes"));
    }
    if (view.problem !== null)
      parts.push(this.notice(`Couldn't compare the pages: ${view.problem}`));
    const threads = threadsNotice(view.threads);
    if (threads !== undefined) parts.push(this.notice(threads));
    if (this.atEnd) parts.push(this.endNotice(view));
    if (!this.arrived) parts.push(this.notArrived(view));
    const unsent = this.unsent();
    if (unsent > 0) {
      const bar = this.el("div", "unsent");
      bar.append(
        this.el("b", "", unsentText(unsent)),
        this.el(
          "span",
          "hint",
          `Only you can see ${unsent === 1 ? "it" : "them"} until you submit.`,
        ),
        this.el("span", "spacer"),
      );
      if (this.overlay) {
        bar.append(this.button("Submit review…", "primary", () => this.overlay?.submitReview()));
      }
      parts.push(bar);
    }
    return parts;
  }

  /** The review controls, in one row. */
  private controls(view: PageView): HTMLElement {
    const row = this.el("div", "row");
    const info = this.el("span", "grow");
    const { pullRequest, base } = againstText(view);
    if (pullRequest !== null && view.threads.state === "on") {
      const link = this.el("a", "", pullRequest) as HTMLAnchorElement;
      link.href = view.threads.pullRequest.url;
      link.target = "_blank";
      link.rel = "noopener noreferrer";
      link.title = `Open pull request ${pullRequest} on GitHub`;
      info.append(link, " against ");
    } else {
      info.append("Against ");
    }
    const baseName = this.el("b", "", base ?? "the base");
    if (view.base) {
      baseName.title = `Compared with ${(view.base.merge_base ?? view.base.commit).slice(0, 7)}, where this branch left ${view.base.requested}`;
    }
    info.append(baseName);
    const page = view.page;
    if (page?.status === "added") info.append(" · a new page");
    if (page && !page.own_file_changed && page.status === "changed" && page.because.length > 0) {
      info.append(" · changed only through ");
      page.because.forEach((cause, i) => {
        if (i > 0) info.append(", ");
        info.append(this.el("code", "", cause));
      });
    }
    row.append(info);
    if (this.arrived) {
      const total = this.marks.length;
      const stepping = this.at >= 0 && this.at < total && this.show === "changes";
      const count = this.button(position(total, stepping ? this.at : -1), "link position", () => {
        this.breakdown = !this.breakdown;
        this.render();
      });
      count.setAttribute("aria-expanded", String(this.breakdown));
      if (page) count.title = countsText(page.counts);
      const shows = this.el("div", "segmented");
      shows.setAttribute("role", "group");
      shows.setAttribute("aria-label", "Show");
      for (const [show, label] of SHOWS) {
        const choice = this.button(label, "", () => {
          this.setShow(show);
          this.render();
        });
        choice.setAttribute("aria-pressed", String(this.show === show));
        shows.append(choice);
      }
      const previous = this.button("↑", "ghost square", () => this.step(-1));
      previous.setAttribute("aria-label", "Previous change");
      previous.title = "Previous change";
      previous.disabled = total === 0;
      const next = this.button("↓", "ghost square", () => this.step(1));
      next.setAttribute("aria-label", "Next change");
      next.title = "Next change";
      row.append(count, shows, previous, next);
    }
    if (this.overlay) {
      const n = this.overlay.threadCount();
      const all = this.button(n === undefined ? "Comments" : `Comments (${n})`, "ghost", () =>
        this.overlay?.showAllComments(),
      );
      all.setAttribute("aria-label", "All comments on this pull request");
      all.setAttribute("aria-haspopup", "dialog");
      row.append(all);
    }
    const refresh = this.button("Refresh", "ghost", () => void this.refresh());
    refresh.setAttribute("aria-label", "Refresh comments and changes");
    row.append(refresh);
    return row;
  }

  /** Past the last change: the next changed page. */
  private endNotice(view: PageView): HTMLElement {
    const notice = this.notice(
      this.marks.length === 0
        ? "This page has no changes."
        : "That was the last change on this page.",
    );
    const next = nextChangedPage(view.changedPages, view.path);
    if (next === null) notice.append(this.el("span", "", "No other page changed."));
    else {
      const target = next.page;
      notice.append(
        this.button(nextPageText(next), "primary", () => {
          remember(FIRST_KEY, "1");
          this.doc.location.assign(target.route);
        }),
      );
    }
    return notice;
  }

  /** The page's changes and threads, listed, since nothing could be placed on it. */
  private notArrived(view: PageView): HTMLElement {
    const body = this.el("div", "body");
    const changes = view.page?.changes ?? [];
    const threads = listedThreads(this.listed);
    const what = [
      view.threads.state === "on" ? plural(threads.length, "comment") : undefined,
      plural(changes.length, "change"),
    ]
      .filter((x) => x !== undefined)
      .join(" and ");
    body.append(
      this.el(
        "b",
        "",
        anchoredBlocks(this.doc).length === 0
          ? `This page has ${what}, but its blocks carry no source anchors.`
          : `This page has ${what}, but most of its changed blocks carry no source anchors.`,
      ),
      this.el(
        "span",
        "",
        "A layout or component is probably dropping the data-ascribe-source attributes, so nothing can be placed on the page. Everything is listed here instead.",
      ),
    );
    if (changes.length > 0) {
      body.append(this.el("span", "hint", "Changes"));
      const list = this.el("ul", "");
      for (const change of changes) {
        const item = this.el("li", "", `${change.kind} `);
        const source = change.now?.source;
        if (source !== undefined) {
          item.append(this.button(describeSource(source), "link", () => this.openSource(source)));
        } else {
          item.append(this.el("span", "", change.text ?? "text"));
        }
        list.append(item);
      }
      body.append(list);
    }
    if (threads.length > 0) {
      body.append(this.el("span", "hint", "Comments"));
      for (const thread of threads) body.append(this.threadCard(thread));
    }
    return body;
  }

  /** A thread, read only: open it on GitHub to reply. */
  private threadCard(thread: LocatedThread): HTMLElement {
    const card = this.el("div", "thread");
    const where = this.el("div", "where");
    const lines = thread.lines;
    const { label, removed } = threadLocation(thread);
    // A thread on removed text is on the base's lines, which the working tree doesn't have.
    if (lines !== undefined && !removed) {
      const source = `${thread.path.split("/").map(encodeURIComponent).join("/")}:${lines.first}-${lines.last}`;
      const link = this.button(label, "link", () => this.openSource(source));
      link.title = `Open ${thread.path}, line ${lines.first}`;
      where.append(link);
    } else {
      const name = this.el("span", "", label);
      name.title = thread.path;
      where.append(name);
    }
    if (removed) where.append(" · On removed text");
    if (thread.resolved) where.append(" · Resolved");
    if (thread.outdated) where.append(" · Outdated");
    card.append(where);
    for (const comment of thread.comments) {
      const body = this.el("div", "comment");
      body.append(
        this.el("b", "", comment.author?.login ?? "ghost"),
        renderMarkdown(this.doc, comment.body),
      );
      card.append(body);
    }
    const url = thread.comments[0]?.url;
    if (url !== undefined && url !== "") {
      const link = this.el("a", "", "Open on GitHub") as HTMLAnchorElement;
      link.href = url;
      link.target = "_blank";
      link.rel = "noopener noreferrer";
      card.append(link);
    }
    return card;
  }

  /** Room below the page, so the panel never hides what's focused. */
  private pad(): void {
    const html = this.doc.documentElement;
    const body = this.doc.body;
    if (this.open && this.stage.kind === "on") {
      this.padding ??= { scroll: html.style.scrollPaddingBottom, body: body.style.paddingBottom };
      const room = `${this.panel.offsetHeight + 90}px`;
      html.style.scrollPaddingBottom = room;
      body.style.paddingBottom = room;
    } else if (this.padding) {
      html.style.scrollPaddingBottom = this.padding.scroll;
      body.style.paddingBottom = this.padding.body;
      this.padding = undefined;
    }
  }

  // --- DOM ---

  private row(text: string): HTMLElement {
    const row = this.el("div", "row");
    row.append(this.el("span", "grow", text));
    return row;
  }

  private notice(text: string): HTMLElement {
    const notice = this.el("div", "notice");
    const words = this.el("span", "");
    // A command or a setting is written in backticks: show it as code.
    text.split("`").forEach((part, i) => {
      if (part !== "") words.append(i % 2 === 1 ? this.el("code", "", part) : part);
    });
    notice.append(words, this.el("span", "spacer"));
    return notice;
  }

  private el(tag: string, className: string, text?: string): HTMLElement {
    const el = this.doc.createElement(tag);
    if (className) el.className = className;
    if (text !== undefined) el.textContent = text;
    return el;
  }

  private button(text: string, className: string, onClick: () => void): HTMLButtonElement {
    const button = this.doc.createElement("button");
    button.type = "button";
    if (className) button.className = className;
    button.textContent = text;
    button.addEventListener("click", onClick);
    return button;
  }

  /** The focused control's label, to focus it again after a redraw. */
  private focused(): string | undefined {
    const active = (this.panel.getRootNode() as ShadowRoot).activeElement;
    return active instanceof HTMLElement && this.panel.contains(active)
      ? (active.getAttribute("aria-label") ?? active.textContent ?? undefined)
      : undefined;
  }

  private refocus(label: string | undefined): void {
    if (label === undefined) return;
    for (const button of Array.from(this.panel.querySelectorAll<HTMLElement>("button, a"))) {
      if ((button.getAttribute("aria-label") ?? button.textContent) === label) {
        button.focus();
        return;
      }
    }
  }
}

/** The label the marks put on a block: inside it at its start, or just before it. */
function labelOf(element: HTMLElement): HTMLElement | undefined {
  const inside = element.querySelector<HTMLElement>(":scope > .ascribe-label");
  if (inside) return inside;
  const before = element.previousElementSibling;
  return before instanceof HTMLElement && before.classList.contains("ascribe-label-before")
    ? before
    : undefined;
}

/**
 * Puts the unsent count in the toolbar button's tooltip. The toolbar has no
 * API for a button's text, so this reads its markup and does nothing if
 * that changed.
 */
function labelButton(doc: Document, text: string): void {
  const toolbar = doc.querySelector("astro-dev-toolbar")?.shadowRoot;
  const button = toolbar?.querySelector(`button.item[data-app-id="${APP_ID}"]`);
  const tip = button?.querySelector(".item-tooltip");
  if (tip) tip.textContent = text;
  button?.setAttribute("aria-label", text);
}

/**
 * Every thread read for a page whose anchors didn't arrive, once each: on
 * the changes' blocks, on removed text, or on no block.
 */
function listedThreads(data: OverlayData | undefined): LocatedThread[] {
  if (!data) return [];
  const { blocks, removed, detached } = data.threads;
  const seen = new Set<string>();
  return [...blocks, ...removed]
    .flatMap((entry) => entry.threads)
    .concat(detached)
    .filter((thread) => !seen.has(thread.id) && seen.add(thread.id));
}

/** An include's place (`<path>:<line>`) as a block's source (`<path>:<line>-<line>`). */
function lineSource(at: string): string {
  const colon = at.lastIndexOf(":");
  const line = at.slice(colon + 1);
  return colon > 0 && /^\d+$/.test(line) ? `${at}-${line}` : at;
}

function plural(count: number, noun: string): string {
  return count === 1 ? `1 ${noun}` : `${count} ${noun}s`;
}

function messageOf(error: unknown): string {
  if (typeof error === "object" && error !== null && "message" in error) {
    return String((error as { message: unknown }).message);
  }
  return String(error);
}

function remember(key: string, value: string): void {
  try {
    sessionStorage.setItem(key, value);
  } catch {
    // Storage is off: the panel just starts closed on the next page.
  }
}

function recall(key: string): string | null {
  try {
    return sessionStorage.getItem(key);
  } catch {
    return null;
  }
}
