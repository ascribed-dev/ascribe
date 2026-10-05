// The review overlay: a pull request's threads beside the blocks of a rendered
// page. It reads the page's source anchors to find each thread's block, gets
// everything else from its host (`OverlayHost`), and draws in shadow roots of
// its own, so the page's styles and the overlay's stay apart:
//
// - above the page, the threads with no block on it (detached);
// - over the page, in a layer: at wide widths, a column of threads beside the
//   page, each aligned with its block, with a line between them on hover and
//   focus; at narrow widths, the panel a block's count marker opens; and the
//   dialogs (submit, all comments);
// - below the page, a bar while there are unsent comments;
// - in the page, on the block under the pointer or focus, a Comment button,
//   and at narrow widths a count marker on each block with threads. These are
//   elements with `data-ascribe-ui`, as the marks' are, each holding a shadow
//   root, so they sit in the page's reading and tab order.
//
// Comment bodies are rendered by `markdown.ts`, as DOM nodes.
import { findBlock, goTo, reveal } from "../marks/index.js";
import { anchorKey, formatSource, type Anchor } from "../place/anchor.js";
import type { LocatedThread } from "../place/place.js";
import { renderMarkdown } from "./markdown.js";
import { OVERLAY_CSS } from "./style.js";
import {
  authorName,
  hasUnsent,
  initials,
  linesLabel,
  listLabel,
  onRemovedText,
  plural,
  relativeTime,
  whereLabel,
} from "./text.js";
import type {
  CommentTarget,
  HostError,
  OverlayData,
  OverlayHost,
  ReviewEvent,
  ThreadSummary,
} from "./types.js";

/** Options for `createOverlay`. */
export interface OverlayOptions {
  /** The rendered page, with source anchors. */
  root: HTMLElement;
  host: OverlayHost;
  /**
   * The least width, in pixels, of the page's container at which threads
   * show in a column beside the page. Narrower, each block with threads gets
   * a count marker that opens a panel. Default 600.
   */
  columnAt?: number;
  /** Called after the overlay redraws: for a host showing the thread count. */
  onUpdate?: () => void;
  /**
   * Whether to draw the bar of unsent comments below the page. Default true.
   * A host that shows the count itself (`unsentCount`) turns it off and opens
   * the submit dialog with `submitReview`.
   */
  unsentBar?: boolean;
}

/** The overlay on one page. */
export interface Overlay {
  /** Reads the threads again from the host and redraws. */
  refresh(): Promise<void>;
  /** Places the threads again, after the page changed size or what it shows. */
  layout(): void;
  /** Goes to a thread on this page, now or once the threads are read. Whether it's on the page. */
  goToThread(threadId: string): boolean;
  /** Opens the list of every thread on the pull request's pages. */
  showAllComments(): void;
  /** How many threads the pull request has on its pages, once read. */
  threadCount(): number | undefined;
  /** How many comments are unsent, once read. */
  unsentCount(): number | undefined;
  /** Opens the submit dialog, when there's something to submit. */
  submitReview(): void;
  /** Removes the overlay and everything it put on the page. */
  dispose(): void;
}

const COLUMN_WIDTH = 250;
const COLUMN_GAP = 12;
const MARKER_ROOM = "44px";

/** A block with threads, or one being commented on, in the page's order. */
interface Entry {
  key: string;
  anchor: Anchor;
  threads: LocatedThread[];
  /** On removed text: the block is the marks' stand-in for what was removed. */
  removed: boolean;
  element: HTMLElement | null;
}

interface Composer {
  key: string;
  anchor: Anchor;
  /** `undefined` while the host works it out. */
  target: CommentTarget | undefined;
  busy: boolean;
  error: string | undefined;
}

type Filter = "open" | "resolved" | "detached" | "unsent";

const EVENTS: [ReviewEvent, string][] = [
  ["COMMENT", "Comment"],
  ["APPROVE", "Approve"],
  ["REQUEST_CHANGES", "Request changes"],
];

/** Puts the review overlay on a rendered page. */
export function createOverlay(options: OverlayOptions): Overlay {
  return new ReviewOverlay(options);
}

class ReviewOverlay implements Overlay {
  private readonly root: HTMLElement;
  private readonly host: OverlayHost;
  private readonly doc: Document;
  private readonly columnAt: number;
  private readonly onUpdate: (() => void) | undefined;
  private readonly unsentBar: boolean;

  private readonly beforeHost: HTMLElement;
  private readonly afterHost: HTMLElement;
  private readonly layerHost: HTMLElement;
  private readonly toolsHost: HTMLElement;
  private readonly before: ShadowRoot;
  private readonly after: ShadowRoot;
  private readonly layer: ShadowRoot;
  private readonly tools: ShadowRoot;
  private readonly live: HTMLElement;

  private data: OverlayData | undefined;
  private all: ThreadSummary[] | undefined;
  private entries: Entry[] = [];
  private wide = true;
  private loadSeq = 0;
  private composer: Composer | undefined;
  private panel: string | undefined;
  /** The block the panel is for, which may have no threads yet. */
  private panelAnchor: Anchor | undefined;
  private dialog: "submit" | "all" | undefined;
  private discardAsk = false;
  private submitting = false;
  private dialogError: string | undefined;
  private filter: Filter = "open";
  private event: ReviewEvent = "COMMENT";
  private readonly expanded = new Set<string>();
  private readonly original = new Set<string>();
  private readonly drafts = new Map<string, string>();
  private readonly busy = new Set<string>();
  private readonly errors = new Map<string, string>();
  private linked: string | undefined;
  /** Which entries were hidden in the page when last drawn. */
  private drawnHidden = "";
  private linkTimer: ReturnType<typeof setTimeout> | undefined;
  private pendingGoTo: string | undefined;
  /** The read whose threads are drawn. */
  private loaded = 0;
  /** The block the Comment button is on. */
  private toolsOn: HTMLElement | undefined;
  /** What had focus before a dialog or the panel opened, to give it back. */
  private opener: HTMLElement | undefined;
  /** The page's blocks this overlay made focusable, and the blocks it made room on. */
  private focusable: HTMLElement[] = [];
  private padded: { element: HTMLElement; before: string }[] = [];
  private markers: { host: HTMLElement; block: HTMLElement }[] = [];
  private containerPosition: string | undefined;
  /** The padding below the page for the column to end in, in pixels. */
  private room = 0;
  private readonly cleanups: (() => void)[] = [];
  private disposed = false;

  constructor(options: OverlayOptions) {
    this.root = options.root;
    this.host = options.host;
    this.doc = this.root.ownerDocument;
    this.columnAt = options.columnAt ?? 600;
    this.onUpdate = options.onUpdate;
    this.unsentBar = options.unsentBar ?? true;

    this.beforeHost = this.uiHost("div");
    this.afterHost = this.uiHost("div");
    this.layerHost = this.uiHost("div");
    this.toolsHost = this.uiHost("span");
    this.before = this.shadow(this.beforeHost);
    this.after = this.shadow(this.afterHost);
    this.layer = this.shadow(this.layerHost);
    this.tools = this.shadow(this.toolsHost);

    const container = this.container();
    if (this.doc.defaultView?.getComputedStyle(container).position === "static") {
      this.containerPosition = container.style.position;
      container.style.position = "relative";
    }
    this.root.before(this.beforeHost);
    this.root.after(this.afterHost);
    this.afterHost.after(this.layerHost);
    Object.assign(this.afterHost.style, { position: "sticky", bottom: "0", zIndex: "3" });
    Object.assign(this.layerHost.style, {
      position: "absolute",
      top: "0",
      left: "0",
      width: "100%",
      height: "100%",
      pointerEvents: "none",
    });
    Object.assign(this.toolsHost.style, { position: "absolute", zIndex: "2" });
    this.toolsHost.hidden = true;

    this.live = this.el("div", "sr-only");
    this.live.setAttribute("role", "status");
    this.live.setAttribute("aria-live", "polite");
    this.layer.append(this.live);

    this.listen();
    this.host.onDidChange(() => {
      if (!this.disposed) void this.refresh();
    });
    void this.refresh();
  }

  // --- Public ---

  async refresh(): Promise<void> {
    const seq = ++this.loadSeq;
    let data: OverlayData;
    let all: ThreadSummary[] | undefined;
    try {
      [data, all] = await Promise.all([
        this.host.load(),
        this.host.allThreads().catch(() => undefined),
      ]);
    } catch (error) {
      if (seq !== this.loadSeq || this.disposed) return;
      this.announce(messageOf(error));
      return;
    }
    if (seq !== this.loadSeq || this.disposed) return;
    this.data = data;
    this.loaded = seq;
    if (all !== undefined) this.all = all;
    this.render();
    const goal = this.pendingGoTo;
    if (goal !== undefined) {
      this.pendingGoTo = undefined;
      this.goToThread(goal);
    }
  }

  layout(): void {
    if (this.disposed || this.data === undefined) return;
    // Crossing the column's width, or a tab or `<details>` showing or hiding
    // a block with threads, changes what's drawn, not only where.
    if (this.measureWide() !== this.wide || this.hiddenState() !== this.drawnHidden) {
      this.render();
      return;
    }
    this.place();
  }

  goToThread(threadId: string): boolean {
    // Until the threads being read arrive: the page may have just changed.
    if (this.data === undefined || this.loaded !== this.loadSeq) {
      this.pendingGoTo = threadId;
      return true;
    }
    const entry = this.entries.find((e) => e.threads.some((t) => t.id === threadId));
    const detached = this.data.threads.detached.some((t) => t.id === threadId);
    if (entry === undefined && !detached) return false;
    this.dialog = undefined;
    this.expanded.add(threadId);
    if (entry?.element) {
      reveal(entry.element);
      if (!this.wide) {
        this.panel = entry.key;
        this.panelAnchor = entry.removed ? undefined : entry.anchor;
      }
      this.render();
      goTo(entry.element);
      if (this.wide) this.link(entry.key, 1800);
    } else {
      this.render();
    }
    const card = this.findCard(threadId);
    if (card) {
      if (typeof card.scrollIntoView === "function" && !entry?.element) {
        card.scrollIntoView({ block: "center" });
      }
      card.focus({ preventScroll: true });
    }
    return true;
  }

  showAllComments(): void {
    this.openDialog("all");
  }

  threadCount(): number | undefined {
    return this.all?.length;
  }

  unsentCount(): number | undefined {
    return this.data?.pending.count;
  }

  submitReview(): void {
    if ((this.data?.pending.count ?? 0) > 0) this.openDialog("submit");
  }

  dispose(): void {
    if (this.disposed) return;
    this.disposed = true;
    for (const cleanup of this.cleanups) cleanup();
    this.undecorate();
    this.beforeHost.remove();
    this.afterHost.remove();
    this.layerHost.remove();
    this.toolsHost.remove();
    if (this.containerPosition !== undefined) {
      this.container().style.position = this.containerPosition;
    }
    this.root.style.removeProperty("margin-inline-end");
    this.makeRoom(0);
    if (this.linkTimer) clearTimeout(this.linkTimer);
  }

  // --- Setup ---

  private container(): HTMLElement {
    return this.root.parentElement ?? this.doc.body;
  }

  private uiHost(tag: "div" | "span"): HTMLElement {
    const host = this.doc.createElement(tag);
    host.setAttribute("data-ascribe-ui", "");
    host.setAttribute("data-ascribe-overlay", "");
    return host;
  }

  private shadow(host: HTMLElement): ShadowRoot {
    const shadow = host.attachShadow({ mode: "open" });
    adoptStyles(shadow, this.doc);
    return shadow;
  }

  private listen(): void {
    const on = <K extends keyof HTMLElementEventMap>(
      target: HTMLElement | ShadowRoot | Window,
      type: K,
      handler: (event: HTMLElementEventMap[K]) => void,
      options?: AddEventListenerOptions,
    ): void => {
      target.addEventListener(type, handler as EventListener, options);
      this.cleanups.push(() => target.removeEventListener(type, handler as EventListener, options));
    };
    on(this.root, "mouseover", (event) => this.pointAt(event.target));
    on(this.root, "focusin", (event) => this.pointAt(event.target));
    on(this.root, "mouseout", (event) => {
      const to = event.relatedTarget;
      if (to instanceof Node && this.toolsOn?.contains(to)) return;
      if (to instanceof Node && (this.toolsHost.contains(to) || to === this.toolsHost)) return;
      if (!this.toolsOn?.contains(this.doc.activeElement)) this.hideTools();
      this.link(undefined);
    });
    on(this.root, "focusout", (event) => {
      const to = event.relatedTarget;
      if (to instanceof Node && this.toolsOn?.contains(to)) return;
      this.hideTools();
      this.link(undefined);
    });
    on(this.root, "click", (event) => this.clickBlock(event));
    on(this.root, "keydown", (event) => this.keyOnBlock(event));
    // A tab or a `<details>` in the page changed what shows: place again.
    on(this.root, "toggle", () => this.layout(), { capture: true });
    on(this.root, "click", () => requestFrame(this.doc, () => this.layout()));
    for (const shadow of [this.before, this.after, this.layer, this.tools]) {
      on(shadow, "click", (event) => this.clickLink(event));
      on(shadow, "keydown", (event) => this.keyInOverlay(event));
      on(shadow, "dblclick", (event) => event.stopPropagation());
    }
    const view = this.doc.defaultView;
    if (view) on(view, "resize", () => this.layout());
    if (view && "ResizeObserver" in view) {
      // Placed in the next frame: placing can pad the page, which resizes
      // what's observed, and doing that inside the observer's callback is a
      // "ResizeObserver loop" error on the page (a dev server reports it).
      let queued = false;
      const observer = new view.ResizeObserver(() => {
        if (queued) return;
        queued = true;
        requestFrame(this.doc, () => {
          queued = false;
          this.layout();
        });
      });
      observer.observe(this.root);
      observer.observe(this.container());
      this.cleanups.push(() => observer.disconnect());
    }
  }

  // --- Reading the page ---

  private measureWide(): boolean {
    return this.container().clientWidth >= this.columnAt;
  }

  private anchorOf(element: Element): Anchor | undefined {
    const source = element.getAttribute("data-ascribe-source");
    if (source === null) return undefined;
    const via = (element.getAttribute("data-ascribe-via") ?? "").split(" ").filter(Boolean);
    return { source, via };
  }

  /** The block an event's target is in: the innermost anchored element. */
  private blockOf(target: EventTarget | null): HTMLElement | undefined {
    if (!(target instanceof Element)) return undefined;
    if (target.closest("[data-ascribe-overlay]") && target !== this.toolsHost) {
      // A marker: the block it's on.
      const marker = this.markers.find((m) => m.host === target.closest("[data-ascribe-overlay]"));
      if (marker) return marker.block;
    }
    const block = target.closest<HTMLElement>("[data-ascribe-source]");
    return block && this.root.contains(block) ? block : undefined;
  }

  private removedElement(anchor: Anchor): HTMLElement | null {
    for (const el of Array.from(
      this.root.querySelectorAll<HTMLElement>("[data-ascribe-was-source]"),
    )) {
      if (el.getAttribute("data-ascribe-was-source") === anchor.source) return el;
    }
    return null;
  }

  private readEntries(): Entry[] {
    const data = this.data;
    if (!data) return [];
    const entries: Entry[] = [];
    for (const { anchor, threads } of data.threads.blocks) {
      entries.push({
        key: anchorKey(anchor),
        anchor,
        threads,
        removed: false,
        element: findBlock(this.root, anchor),
      });
    }
    for (const { anchor, threads } of data.threads.removed) {
      entries.push({
        key: `removed ${anchorKey(anchor)}`,
        anchor,
        threads,
        removed: true,
        element: this.removedElement(anchor),
      });
    }
    // A block being commented on, or whose panel is open, may have no threads yet.
    for (const anchor of [this.composer?.anchor, this.panelAnchor]) {
      if (anchor === undefined) continue;
      const key = anchorKey(anchor);
      if (entries.some((e) => e.key === key)) continue;
      entries.push({
        key,
        anchor,
        threads: [],
        removed: false,
        element: findBlock(this.root, anchor),
      });
    }
    // Page order.
    entries.sort((a, b) => {
      if (!a.element || !b.element || a.element === b.element) return 0;
      return a.element.compareDocumentPosition(b.element) & Node.DOCUMENT_POSITION_FOLLOWING
        ? -1
        : 1;
    });
    return entries;
  }

  /** Whether what the page shows hides a block: removed text in "as it will be", new text in "as it was". */
  private hiddenByShow(element: HTMLElement): boolean {
    const show = this.root.getAttribute("data-ascribe-show");
    const kind = element.closest("[data-ascribe-change]")?.getAttribute("data-ascribe-change");
    if (show === "will") return kind === "removed" || kind === "moved-from";
    if (show === "was") return kind === "added" || kind === "moved";
    return false;
  }

  /** Where a block is hidden in the page: an unselected tab or a closed `<details>`. */
  private hiddenIn(element: HTMLElement): string | undefined {
    for (
      let node: HTMLElement | null = element.parentElement;
      node !== null && node !== this.root.parentElement;
      node = node.parentElement
    ) {
      if (node.localName === "ascribe-tab" && node.hidden) {
        return `In the ${node.getAttribute("label") ?? ""} tab, which isn't showing.`;
      }
      if (node.localName === "details" && !(node as HTMLDetailsElement).open) {
        const summary = node.querySelector(":scope > summary");
        if (summary?.contains(element)) continue;
        return `Inside “${(summary?.textContent ?? "").trim()}”, which is closed.`;
      }
    }
    return undefined;
  }

  private hiddenState(): string {
    return this.entries
      .map((e) => (e.element && (this.hiddenIn(e.element) || this.hiddenByShow(e.element)) ? 1 : 0))
      .join("");
  }

  /**
   * The element to align a block's threads with: the block, or, while it's
   * in an unselected tab or a closed `<details>`, the tabs or the details.
   */
  private visibleFor(element: HTMLElement | null): HTMLElement | null {
    let shown = element;
    for (
      let node = element?.parentElement ?? null;
      node !== null && node !== this.root;
      node = node.parentElement
    ) {
      if (node.localName === "ascribe-tab" && node.hidden) shown = node.parentElement;
      if (node.localName === "details" && !(node as HTMLDetailsElement).open) {
        const summary = node.querySelector(":scope > summary");
        if (!(shown && summary?.contains(shown))) shown = node;
      }
    }
    return shown;
  }

  // --- Drawing ---

  private render(): void {
    if (this.disposed) return;
    const focus = this.focusKey();
    this.wide = this.measureWide();
    this.entries = this.readEntries();
    if (this.panel !== undefined && !this.entries.some((e) => e.key === this.panel)) {
      this.panel = undefined;
      this.panelAnchor = undefined;
    }
    this.drawnHidden = this.hiddenState();
    this.decorate();
    this.before.replaceChildren(...this.styleNodes(this.before), ...this.drawDetached());
    this.after.replaceChildren(...this.styleNodes(this.after), ...this.drawUnsentBar());
    const layer = this.el("div", "layer");
    if (this.wide) layer.append(...this.drawColumn());
    const sheet = this.wide ? null : this.drawSheet();
    if (sheet) layer.append(sheet);
    const dialog = this.drawDialog();
    if (dialog) layer.append(dialog);
    this.layer.replaceChildren(...this.styleNodes(this.layer), layer, this.live);
    this.drawTools();
    this.restoreFocus(focus);
    this.place();
    this.onUpdate?.();
  }

  /** A `<style>` element where constructed stylesheets aren't available. */
  private styleNodes(shadow: ShadowRoot): Node[] {
    return Array.from(shadow.children).filter(
      (child) => child.localName === "style" && child.hasAttribute("data-overlay"),
    );
  }

  /** Makes the page's blocks focusable, and at narrow widths puts count markers on them. */
  private decorate(): void {
    this.undecorate();
    const threadsOn = this.data !== undefined;
    if (!threadsOn) {
      this.root.style.removeProperty("margin-inline-end");
      return;
    }
    if (this.wide)
      this.root.style.setProperty("margin-inline-end", `${COLUMN_WIDTH + COLUMN_GAP}px`);
    else this.root.style.removeProperty("margin-inline-end");
    for (const el of Array.from(this.root.querySelectorAll<HTMLElement>("[data-ascribe-source]"))) {
      if (el.hasAttribute("tabindex")) continue;
      el.tabIndex = 0;
      this.focusable.push(el);
    }
    if (this.wide) return;
    for (const entry of this.entries) {
      const count = entry.threads.length;
      if (count === 0 || !entry.element || this.hiddenByShow(entry.element)) continue;
      const host = this.uiHost("span");
      Object.assign(host.style, { position: "absolute", zIndex: "1" });
      const shadow = this.shadow(host);
      const marker = this.button("marker", String(count), () => this.openEntry(entry));
      marker.setAttribute(
        "aria-label",
        `${plural(count, "comment")} on this block. Show ${count === 1 ? "it" : "them"}.`,
      );
      marker.setAttribute("aria-haspopup", "dialog");
      shadow.append(marker);
      // A block in a closed `<details>` or an unselected tab has its marker
      // on what shows in its place; its panel says where it is.
      const element = this.visibleFor(entry.element) ?? entry.element;
      if (this.markers.some((m) => m.block === element)) continue;
      if (isVoid(element)) element.after(host);
      else element.append(host);
      this.markers.push({ host, block: element });
      this.padded.push({ element, before: element.style.paddingInlineEnd });
      element.style.paddingInlineEnd = MARKER_ROOM;
    }
  }

  private undecorate(): void {
    for (const el of this.focusable) el.removeAttribute("tabindex");
    this.focusable = [];
    for (const { element, before } of this.padded) element.style.paddingInlineEnd = before;
    this.padded = [];
    for (const { host } of this.markers) host.remove();
    this.markers = [];
  }

  private drawDetached(): HTMLElement[] {
    const detached = this.data?.threads.detached ?? [];
    if (detached.length === 0) return [];
    const box = this.el("section", "detached");
    box.setAttribute("aria-label", "Comments with no block on this page");
    box.append(
      this.el(
        "div",
        "",
        detached.length === 1
          ? "1 comment has no block on this page any more:"
          : `${detached.length} comments have no block on this page any more:`,
      ),
      ...detached.map((thread) => this.card(thread, "detached", undefined)),
    );
    return [box];
  }

  private drawUnsentBar(): HTMLElement[] {
    const count = this.data?.pending.count ?? 0;
    if (count === 0 || !this.unsentBar) return [];
    const bar = this.el("div", "unsent-bar");
    bar.setAttribute("role", "region");
    bar.setAttribute("aria-label", "Unsent comments");
    bar.append(
      this.el("b", "", plural(count, "unsent comment")),
      this.el("span", "hint", `Only you can see ${count === 1 ? "it" : "them"} until you submit.`),
      this.el("span", "spacer"),
      this.button("primary", "Submit review…", () => this.openDialog("submit"), "submit-open"),
    );
    return [bar];
  }

  private drawColumn(): Element[] {
    const column = this.el("div", "column");
    for (const entry of this.entries) {
      if (entry.element && this.hiddenByShow(entry.element)) continue;
      const cards = this.cardsFor(entry, "column");
      if (cards.length === 0) continue;
      const slot = this.el("div", "slot");
      slot.dataset["key"] = entry.key;
      slot.append(...cards);
      slot.addEventListener("mouseenter", () => this.link(entry.key));
      slot.addEventListener("mouseleave", () => this.link(undefined));
      slot.addEventListener("focusin", () => this.link(entry.key));
      slot.addEventListener("focusout", (event) => {
        if (!(event.relatedTarget instanceof Node && slot.contains(event.relatedTarget))) {
          this.link(undefined);
        }
      });
      column.append(slot);
    }
    const highlight = this.el("div", "highlight");
    highlight.hidden = true;
    const svg = this.doc.createElementNS("http://www.w3.org/2000/svg", "svg");
    svg.setAttribute("class", "connector");
    svg.setAttribute("aria-hidden", "true");
    return [highlight, svg, column];
  }

  private cardsFor(entry: Entry, view: "column" | "panel"): HTMLElement[] {
    const cards = entry.threads.map((thread) => this.card(thread, view, entry));
    if (this.composer?.key === entry.key) cards.push(this.drawComposer(this.composer));
    return cards;
  }

  private drawSheet(): HTMLElement | null {
    const entry = this.entries.find((e) => e.key === this.panel);
    if (!entry) return null;
    const sheet = this.el("div", "sheet");
    sheet.setAttribute("role", "dialog");
    sheet.setAttribute("aria-label", "Comments on this block");
    const head = this.el("div", "head");
    head.append(
      this.el("b", "", entry.removed ? "Removed text" : sourceLabel(entry.anchor)),
      this.el("span", "spacer"),
    );
    if (!entry.removed) {
      head.append(this.button("ghost", "Source", () => this.host.openSource(entry.anchor)));
      if (this.composer?.key !== entry.key) {
        head.append(this.button("ghost", "Comment", () => this.startComment(entry.anchor)));
      }
    }
    head.append(this.button("ghost", "Close", () => this.closePanel(), "panel-close"));
    sheet.append(head);
    const cards = this.cardsFor(entry, "panel");
    if (cards.length === 0) sheet.append(this.el("p", "hint", "No comments on this block yet."));
    sheet.append(...cards);
    return sheet;
  }

  private card(
    thread: LocatedThread,
    view: "column" | "panel" | "detached",
    entry: Entry | undefined,
  ): HTMLElement {
    const data = this.data;
    const viewer = data?.viewer ?? "";
    const collapsed = thread.resolved && !this.expanded.has(thread.id);
    const card = this.el("article", `thread${collapsed ? " collapsed" : ""}`);
    card.dataset["thread"] = thread.id;
    card.dataset["focus"] = `thread:${thread.id}`;
    card.tabIndex = -1;
    const first = thread.comments[0];
    card.setAttribute(
      "aria-label",
      `Comment by ${first ? authorName(first.author, viewer) : "someone"} on ${whereLabel(thread)}`,
    );

    const head = this.el("div", "th");
    head.append(this.el("span", "", whereLabel(thread)));
    if (onRemovedText(thread)) head.append(this.badge("removed", "On removed text"));
    if (thread.resolved) head.append(this.badge("resolved", "Resolved"));
    if (thread.detached !== undefined) head.append(this.badge("detached", "Detached"));
    if (thread.outdated && thread.detached === undefined) {
      head.append(this.badge("outdated", "Outdated"));
    }
    if (thread.kind === "conversation") head.append(this.badge("summary", "In the review summary"));
    if (hasUnsent(thread)) head.append(this.badge("unsent", "Unsent"));
    card.append(head);

    // Actions sit on their own row, so the header never pushes one onto a line by itself.
    const acts = this.el("div", "acts");
    if (thread.resolved) {
      acts.append(
        this.button(
          "link",
          collapsed ? "Expand" : "Collapse",
          () => {
            if (collapsed) this.expanded.add(thread.id);
            else this.expanded.delete(thread.id);
            this.render();
          },
          `expand:${thread.id}`,
        ),
      );
    }
    if (thread.outdated && thread.detached === undefined && thread.quote !== undefined) {
      const open = this.original.has(thread.id);
      acts.append(
        this.button(
          "link",
          open ? "Hide original text" : "Original text",
          () => {
            if (open) this.original.delete(thread.id);
            else this.original.add(thread.id);
            this.render();
          },
          `original:${thread.id}`,
        ),
      );
    }
    if (entry && !entry.removed) {
      acts.append(
        this.button("link", "Open source", () => this.host.openSource(threadSource(thread, entry))),
      );
    }
    if (thread.kind === "review" && thread.detached === undefined) {
      const can = thread.resolved ? thread.canUnresolve : thread.canResolve;
      if (can) {
        const resolve = this.button(
          "link",
          thread.resolved ? "Reopen" : "Resolve",
          () => void this.resolve(thread),
          `resolve:${thread.id}`,
        );
        resolve.title = "Sent to GitHub right away, not held with your review";
        resolve.disabled = this.busy.has(thread.id);
        acts.append(resolve);
      }
    }
    if (acts.childElementCount > 0) card.append(acts);

    const hidden = view !== "detached" && entry?.element ? this.hiddenIn(entry.element) : undefined;
    if (hidden && entry?.element) {
      const element = entry.element;
      const inside = this.el("div", "inside", `${hidden} `);
      inside.append(
        this.button("link", "Show it", () => {
          reveal(element);
          this.render();
          goTo(element);
          this.link(entry.key, 1800);
        }),
      );
      card.append(inside);
    }
    if (collapsed) return card;

    if (
      thread.quote !== undefined &&
      thread.quote.trim() !== "" &&
      (thread.detached !== undefined || this.original.has(thread.id))
    ) {
      const orig = this.el("div", "orig");
      orig.append(
        this.el(
          "span",
          "",
          thread.detached !== undefined
            ? "The comment was on this text, which is no longer on the page: "
            : "The text when the comment was made: ",
        ),
        this.el("q", "", thread.quote),
      );
      card.append(orig);
    }

    const now = Date.now();
    for (const comment of thread.comments) {
      const row = this.el("div", "comment");
      const avatar = this.el("span", "avatar", initials(comment.author));
      avatar.setAttribute("aria-hidden", "true");
      const text = this.el("div", "");
      text.append(
        this.el("span", "who", authorName(comment.author, viewer)),
        this.el(
          "span",
          "when",
          comment.pending ? "not sent yet" : relativeTime(comment.createdAt, now),
        ),
      );
      const body = this.el("div", "body");
      body.append(renderMarkdown(this.doc, comment.body));
      text.append(body);
      row.append(avatar, text);
      card.append(row);
    }

    if (thread.kind === "review" && thread.detached === undefined && thread.canReply) {
      card.append(this.drawReply(thread));
    }
    return card;
  }

  private drawReply(thread: LocatedThread): HTMLElement {
    const key = `reply:${thread.id}`;
    const box = this.el("div", "reply");
    const input = this.doc.createElement("textarea");
    input.rows = 1;
    input.placeholder = "Reply…";
    input.setAttribute("aria-label", "Reply to thread");
    input.dataset["focus"] = key;
    input.value = this.drafts.get(key) ?? "";
    const busy = this.busy.has(thread.id);
    input.disabled = busy;
    input.addEventListener("input", () => this.drafts.set(key, input.value));
    input.addEventListener("keydown", (event) => {
      if (event.key === "Enter" && (event.ctrlKey || event.metaKey)) {
        event.preventDefault();
        void this.reply(thread, "withReview");
      }
    });
    const held = (this.data?.pending.count ?? 0) > 0;
    const now = this.button(
      "ghost",
      "Reply now",
      () => {
        if (held) {
          this.announce(HELD_REASON);
          return;
        }
        void this.reply(thread, "now");
      },
      `reply-now:${thread.id}`,
    );
    // GitHub adds every reply to the review while it has unsent comments, so
    // a reply can't go at once until it's submitted or discarded. The button
    // stays focusable, so the reason can be read.
    if (held) {
      now.setAttribute("aria-disabled", "true");
      now.title = HELD_REASON;
      const reason = this.el("span", "sr-only", HELD_REASON);
      reason.id = `held-${cssId(thread.id)}`;
      now.setAttribute("aria-describedby", reason.id);
      box.append(reason);
    } else {
      now.title = "Send this reply to GitHub right away";
    }
    now.disabled = busy;
    const add = this.button(
      "ghost",
      "Add to review",
      () => void this.reply(thread, "withReview"),
      `reply-add:${thread.id}`,
    );
    add.title = "Hold this reply until you submit the review";
    add.disabled = busy;
    box.prepend(input);
    box.append(now, add);
    const error = this.errors.get(key);
    if (error) {
      const line = this.el("div", "error", error);
      line.setAttribute("role", "alert");
      box.append(line);
    }
    return box;
  }

  private drawComposer(composer: Composer): HTMLElement {
    const key = `comment:${composer.key}`;
    const box = this.el("div", "composer");
    box.setAttribute("role", "group");
    box.setAttribute("aria-label", `Comment on ${sourceLabel(composer.anchor)}`);
    const target = composer.target;
    const refused = target?.kind === "push-first";
    if (target === undefined)
      box.append(this.el("div", "hint", "Checking where this comment can go…"));
    if (target?.kind === "summary") {
      box.append(
        this.el(
          "div",
          "hint",
          target.reason === "file"
            ? "This page's file isn't in the pull request, so GitHub can't put a comment on its lines. Your comment will go in your review's summary and still show here."
            : "GitHub only takes comments on the lines a pull request changes and the few around them. Your comment will go in your review's summary and still show here.",
        ),
      );
    }
    if (target?.kind === "push-first") {
      const refuse = this.el("div", "refuse", target.message);
      refuse.setAttribute("role", "alert");
      box.append(refuse);
    }
    const text = this.doc.createElement("textarea");
    text.setAttribute("aria-label", "Comment");
    text.placeholder = `Comment on ${sourceLabel(composer.anchor)}`;
    text.dataset["focus"] = key;
    text.value = this.drafts.get(key) ?? "";
    text.disabled = refused || composer.busy;
    text.addEventListener("input", () => this.drafts.set(key, text.value));
    text.addEventListener("keydown", (event) => {
      if (event.key === "Enter" && (event.ctrlKey || event.metaKey)) {
        event.preventDefault();
        void this.addComment();
      }
    });
    box.append(text);
    if (composer.error) {
      const error = this.el("div", "error", composer.error);
      error.setAttribute("role", "alert");
      box.append(error);
    }
    const row = this.el("div", "row");
    const add = this.button(
      "primary",
      "Add to review",
      () => void this.addComment(),
      "composer-add",
    );
    add.disabled = refused || composer.busy || target === undefined;
    row.append(
      add,
      this.button("ghost", "Cancel", () => this.cancelComment(), "composer-cancel"),
    );
    if (!refused)
      row.append(this.el("span", "hint", "Nothing is sent until you submit the review."));
    box.append(row);
    return box;
  }

  private drawDialog(): HTMLElement | null {
    const data = this.data;
    if (!data || this.dialog === undefined) return null;
    const backdrop = this.el("div", "backdrop");
    backdrop.addEventListener("click", (event) => {
      if (event.target === backdrop) this.closeDialog();
    });
    const dialog = this.el("div", "dialog");
    dialog.setAttribute("role", "dialog");
    dialog.setAttribute("aria-modal", "true");
    backdrop.append(dialog);
    if (this.dialog === "all") this.drawAll(dialog, data);
    else this.drawSubmit(dialog, data);
    return backdrop;
  }

  private drawAll(dialog: HTMLElement, data: OverlayData): void {
    dialog.setAttribute("aria-label", "All comments");
    const head = this.el("div", "row");
    head.append(
      this.el("h2", "", `All comments on #${data.pullRequest.number}`),
      this.el("span", "spacer"),
      this.button("ghost", "Close", () => this.closeDialog(), "dialog-close"),
    );
    dialog.append(head);
    const all = this.all;
    if (all === undefined) {
      dialog.append(this.el("p", "hint", "Reading the comments…"));
      return;
    }
    const groups: Record<Filter, ThreadSummary[]> = {
      open: all.filter((s) => !s.thread.resolved && s.thread.detached === undefined),
      resolved: all.filter((s) => s.thread.resolved),
      detached: all.filter((s) => s.thread.detached !== undefined),
      unsent: all.filter((s) => hasUnsent(s.thread)),
    };
    const filters = this.el("div", "segmented");
    filters.setAttribute("role", "group");
    filters.setAttribute("aria-label", "Filter");
    for (const [filter, label] of [
      ["open", "Open"],
      ["resolved", "Resolved"],
      ["detached", "Detached"],
      ["unsent", "Unsent"],
    ] as const) {
      const choice = this.button(
        "",
        `${label} ${groups[filter].length}`,
        () => {
          this.filter = filter;
          this.render();
        },
        `filter:${filter}`,
      );
      choice.setAttribute("aria-pressed", String(this.filter === filter));
      filters.append(choice);
    }
    dialog.append(filters);
    const list = this.el("ul", "thread-list");
    for (const summary of groups[this.filter]) {
      const { thread, pages } = summary;
      const first = thread.comments[0];
      const page =
        pages.length > 1 ? `on ${pages.length} pages` : (pages[0]?.title ?? pages[0]?.path ?? "");
      const where = [listLabel(thread), thread.outdated ? "outdated" : "", page]
        .filter(Boolean)
        .join(" · ");
      const more = thread.comments.length > 1 ? ` (+${thread.comments.length - 1})` : "";
      const item = this.el("li", "");
      const jump = this.button("", "", () => this.jump(summary), `jump:${thread.id}`);
      jump.append(
        this.el("span", "where", where),
        this.el(
          "span",
          "first",
          first ? `${authorName(first.author, data.viewer)}: ${oneLine(first.body)}${more}` : "",
        ),
      );
      item.append(jump);
      list.append(item);
    }
    if (groups[this.filter].length === 0) list.append(this.el("li", "hint", "None."));
    dialog.append(list);
  }

  private drawSubmit(dialog: HTMLElement, data: OverlayData): void {
    dialog.setAttribute("aria-label", "Submit review");
    const count = data.pending.count;
    dialog.append(this.el("h2", "", `Submit review to #${data.pullRequest.number}`));
    if (count > 0) {
      dialog.append(
        this.el(
          "span",
          "hint",
          count === 1 ? "This one will be sent:" : `These ${count} will be sent:`,
        ),
      );
      const list = this.el("ul", "unsent-list");
      for (const summary of this.all ?? []) {
        const { thread } = summary;
        thread.comments.forEach((comment, index) => {
          if (!comment.pending) return;
          const item = this.el("li", "");
          item.append(this.button("link", listLabel(thread), () => this.jump(summary)));
          const how =
            index > 0
              ? " (reply): "
              : thread.kind === "conversation"
                ? " (in the summary): "
                : ": ";
          item.append(this.el("span", "", `${how}${oneLine(comment.body)}`));
          list.append(item);
        });
      }
      dialog.append(list);
    }
    const choices = this.doc.createElement("fieldset");
    const legend = this.el("legend", "hint", "Submit as");
    choices.append(legend);
    for (const [event, label] of EVENTS) {
      const option = this.doc.createElement("label");
      const radio = this.doc.createElement("input");
      radio.type = "radio";
      radio.name = "event";
      radio.value = event;
      radio.checked = this.event === event;
      radio.dataset["focus"] = `event:${event}`;
      radio.addEventListener("change", () => {
        if (radio.checked) this.event = event;
      });
      option.append(radio, ` ${label}`);
      choices.append(option);
    }
    dialog.append(choices);
    const summary = this.doc.createElement("textarea");
    summary.placeholder = "Summary (optional)";
    summary.setAttribute("aria-label", "Summary (optional)");
    summary.dataset["focus"] = "summary";
    summary.value = this.drafts.get("summary") ?? "";
    summary.addEventListener("input", () => this.drafts.set("summary", summary.value));
    dialog.append(summary);
    if (this.dialogError) {
      const error = this.el("div", "error", this.dialogError);
      error.setAttribute("role", "alert");
      dialog.append(error);
    }
    if (this.discardAsk) {
      const confirm = this.el("div", "confirm");
      confirm.setAttribute("role", "alert");
      const row = this.el("div", "row");
      row.append(
        this.button("danger", "Discard comments", () => void this.discard(), "discard-yes"),
        this.button(
          "ghost",
          "Keep review",
          () => {
            this.discardAsk = false;
            this.render();
          },
          "discard-no",
        ),
      );
      confirm.append(
        this.el("b", "", `Discard ${plural(count, "unsent comment")}?`),
        this.el("span", "", " They will be deleted and can't be brought back."),
        row,
      );
      dialog.append(confirm);
      return;
    }
    const row = this.el("div", "row");
    const submit = this.button("primary", "Submit review", () => void this.submit(), "submit");
    submit.disabled = this.submitting;
    row.append(
      submit,
      this.button("ghost", "Cancel", () => this.closeDialog(), "dialog-close"),
      this.el("span", "spacer"),
    );
    if (count > 0) {
      row.append(
        this.button(
          "link",
          "Discard…",
          () => {
            this.discardAsk = true;
            this.render();
            this.focusOn("discard-no");
          },
          "discard",
        ),
      );
    }
    dialog.append(row);
  }

  /** The Comment button, on the block under the pointer or focus. */
  private drawTools(): void {
    const button = this.button(
      "tool",
      "Comment",
      () => {
        const block = this.toolsOn;
        const anchor = block && this.anchorOf(block);
        if (anchor) this.startComment(anchor);
      },
      "tools",
    );
    button.setAttribute("aria-label", "Comment on this block");
    this.tools.replaceChildren(...this.styleNodes(this.tools), button);
    if (!this.wide || this.data === undefined) this.hideTools();
  }

  // --- Placing ---

  private place(): void {
    if (this.disposed) return;
    const base = this.layerHost.getBoundingClientRect();
    const column = this.layer.querySelector<HTMLElement>(".column");
    if (column) {
      const page = this.root.getBoundingClientRect();
      column.style.left = `${page.right - base.left + COLUMN_GAP}px`;
      column.style.width = `${COLUMN_WIDTH}px`;
      let previous = 0;
      for (const slot of Array.from(column.querySelectorAll<HTMLElement>(":scope > .slot"))) {
        const entry = this.entries.find((e) => e.key === slot.dataset["key"]);
        const element = this.visibleFor(entry?.element ?? null);
        const at = element ? element.getBoundingClientRect().top - base.top : previous;
        const top = Math.max(at, previous);
        slot.style.top = `${top}px`;
        previous = top + slot.offsetHeight + 2;
      }
      column.style.height = `${previous}px`;
      // The column can run past the page: make room below it. The room
      // already made isn't the page's, or making it would undo it.
      const overrun = Math.ceil(previous - (page.bottom - this.room - base.top));
      this.makeRoom(overrun > 0 ? overrun : 0);
    } else {
      this.makeRoom(0);
    }
    for (const { host, block } of this.markers) this.pin(host, block);
    if (this.toolsOn && !this.toolsHost.hidden) this.pin(this.toolsHost, this.toolsOn);
    if (this.linked !== undefined) this.drawLink(this.linked);
  }

  /** Pads the page's bottom by `room` pixels, for the column to end in. */
  private makeRoom(room: number): void {
    if (room === this.room) return;
    this.room = room;
    if (room > 0) this.root.style.setProperty("padding-bottom", `${room}px`);
    else this.root.style.removeProperty("padding-bottom");
  }

  /** Positions an element of the page at a block's top right corner. */
  private pin(host: HTMLElement, block: HTMLElement): void {
    const parent = host.offsetParent ?? this.doc.body;
    const outer = parent.getBoundingClientRect();
    const box = block.getBoundingClientRect();
    host.style.top = `${box.top - outer.top - parent.clientTop}px`;
    host.style.left = `${box.right - outer.left - parent.clientLeft - host.offsetWidth}px`;
  }

  private pointAt(target: EventTarget | null): void {
    if (this.data === undefined) return;
    const block = this.blockOf(target);
    if (!block) return;
    if (this.wide) {
      const key = this.anchorKeyOf(block);
      if (key !== undefined && this.entries.some((e) => e.key === key)) this.link(key);
      if (block !== this.toolsOn || this.toolsHost.hidden) this.showTools(block);
    }
  }

  private anchorKeyOf(block: HTMLElement): string | undefined {
    const anchor = this.anchorOf(block);
    return anchor && anchorKey(anchor);
  }

  private showTools(block: HTMLElement): void {
    if (this.toolsHost.parentElement !== block) {
      if (isVoid(block)) block.after(this.toolsHost);
      else block.append(this.toolsHost);
    }
    this.toolsOn = block;
    this.toolsHost.hidden = false;
    this.pin(this.toolsHost, block);
  }

  private hideTools(): void {
    this.toolsHost.hidden = true;
  }

  /** Highlights a block and its threads and joins them with a line; `undefined` clears it. */
  private link(key: string | undefined, forMs?: number): void {
    if (this.linkTimer) clearTimeout(this.linkTimer);
    this.linkTimer = undefined;
    this.linked = key;
    this.drawLink(key);
    if (key !== undefined && forMs !== undefined) {
      this.linkTimer = setTimeout(() => this.link(undefined), forMs);
    }
  }

  private drawLink(key: string | undefined): void {
    const svg = this.layer.querySelector("svg.connector");
    const highlight = this.layer.querySelector<HTMLElement>(".highlight");
    for (const slot of Array.from(this.layer.querySelectorAll(".slot.linked"))) {
      slot.classList.remove("linked");
    }
    svg?.replaceChildren();
    if (highlight) highlight.hidden = true;
    if (key === undefined || !svg || !highlight) return;
    const entry = this.entries.find((e) => e.key === key);
    const element = this.visibleFor(entry?.element ?? null);
    const slot = Array.from(this.layer.querySelectorAll<HTMLElement>(".slot")).find(
      (s) => s.dataset["key"] === key,
    );
    if (!element || !slot) return;
    slot.classList.add("linked");
    const base = this.layerHost.getBoundingClientRect();
    const a = element.getBoundingClientRect();
    const s = slot.getBoundingClientRect();
    Object.assign(highlight.style, {
      top: `${a.top - base.top}px`,
      left: `${a.left - base.left}px`,
      width: `${a.width}px`,
      height: `${a.height}px`,
    });
    highlight.hidden = false;
    const x1 = a.right - base.left;
    const y1 = a.top - base.top + 12;
    const x2 = s.left - base.left;
    const y2 = s.top - base.top + 14;
    const mid = (x1 + x2) / 2;
    const path = this.doc.createElementNS("http://www.w3.org/2000/svg", "path");
    path.setAttribute("d", `M${x1} ${y1} H${mid} V${y2} H${x2}`);
    svg.append(path);
  }

  // --- What the reviewer does ---

  private clickBlock(event: MouseEvent): void {
    if (this.data === undefined || this.wide || event.defaultPrevented) return;
    const target = event.target instanceof Element ? event.target : null;
    if (!target || target.closest("a, button, input, textarea, select, summary, [role=tab]"))
      return;
    if (this.doc.getSelection()?.isCollapsed === false) return;
    const block = this.blockOf(target);
    const anchor = block && this.anchorOf(block);
    if (anchor) this.openPanel(anchor);
  }

  private keyOnBlock(event: KeyboardEvent): void {
    if (this.data === undefined) return;
    if (event.key === "Escape") {
      this.escape();
      return;
    }
    const block = event.target instanceof HTMLElement ? event.target : null;
    if (!block || !this.focusable.includes(block)) return;
    if (event.key !== "Enter") return;
    event.preventDefault();
    const anchor = this.anchorOf(block);
    if (!anchor) return;
    if (this.wide) this.startComment(anchor);
    else this.openPanel(anchor);
  }

  private keyInOverlay(event: KeyboardEvent): void {
    if (event.key === "Escape") {
      event.stopPropagation();
      this.escape();
      return;
    }
    if (event.key === "Tab" && this.dialog !== undefined) this.trapFocus(event);
  }

  private escape(): void {
    if (this.dialog !== undefined) this.closeDialog();
    else if (this.composer !== undefined) this.cancelComment();
    else if (this.panel !== undefined) this.closePanel();
  }

  private clickLink(event: MouseEvent): void {
    const target = event.composedPath()[0];
    const link = target instanceof Element ? target.closest("a[href]") : null;
    if (!link || !this.host.openLink) return;
    event.preventDefault();
    this.host.openLink(link.getAttribute("href") ?? "");
  }

  private startComment(anchor: Anchor): void {
    const key = anchorKey(anchor);
    if (this.composer?.key === key) {
      this.focusOn(`comment:${key}`);
      return;
    }
    this.rememberOpener();
    this.composer = { key, anchor, target: undefined, busy: false, error: undefined };
    if (!this.wide) {
      this.panel = key;
      this.panelAnchor = anchor;
    }
    this.render();
    this.focusOn(`comment:${key}`);
    const composer = this.composer;
    this.host.commentTarget(anchor).then(
      (target) => {
        if (this.composer !== composer) return;
        composer.target = target;
        this.render();
        if (target.kind !== "push-first") this.focusOn(`comment:${key}`);
        else this.focusOn("composer-cancel");
      },
      () => {
        if (this.composer !== composer) return;
        // Nothing to say up front: try it, and the session holds it if it must.
        composer.target = { kind: "thread" };
        this.render();
      },
    );
  }

  private cancelComment(): void {
    const composer = this.composer;
    if (!composer) return;
    this.composer = undefined;
    this.drafts.delete(`comment:${composer.key}`);
    this.render();
    this.giveBackFocus();
  }

  private async addComment(): Promise<void> {
    const composer = this.composer;
    const key = composer && `comment:${composer.key}`;
    const body = key ? (this.drafts.get(key) ?? "").trim() : "";
    if (!composer || !key || composer.busy || body === "") return;
    composer.busy = true;
    composer.error = undefined;
    this.render();
    try {
      const thread = await this.host.comment(composer.anchor, body);
      this.composer = undefined;
      this.drafts.delete(key);
      this.announce(
        thread.kind === "conversation"
          ? "Added to your review's summary. Nothing is sent until you submit the review."
          : "Added to your review. Nothing is sent until you submit the review.",
      );
      await this.refresh();
      if (!this.focusOn(`reply:${thread.id}`)) this.findCard(thread.id)?.focus();
    } catch (error) {
      composer.busy = false;
      composer.error = messageOf(error);
      if (codeOf(error) === "push-first")
        composer.target = { kind: "push-first", message: composer.error };
      this.announce(composer.error);
      this.render();
    }
  }

  private async reply(thread: LocatedThread, when: "now" | "withReview"): Promise<void> {
    const key = `reply:${thread.id}`;
    const body = (this.drafts.get(key) ?? "").trim();
    if (body === "" || this.busy.has(thread.id)) return;
    this.busy.add(thread.id);
    this.errors.delete(key);
    this.render();
    try {
      await this.host.reply(thread.id, body, when);
      this.drafts.delete(key);
      this.busy.delete(thread.id);
      const message = when === "now" ? "Reply sent to GitHub." : "Reply added to your review.";
      this.announce(message);
      if (when === "now") this.host.notify?.(message);
      await this.refresh();
      this.focusOn(key);
    } catch (error) {
      this.busy.delete(thread.id);
      this.errors.set(key, messageOf(error));
      this.announce(messageOf(error));
      // The session refused because the review has unsent comments: read again, so the bar shows them.
      if (codeOf(error) === "reply-held") await this.refresh();
      else this.render();
      this.focusOn(key);
    }
  }

  private async resolve(thread: LocatedThread): Promise<void> {
    if (this.busy.has(thread.id)) return;
    const resolved = !thread.resolved;
    this.busy.add(thread.id);
    this.render();
    try {
      await this.host.resolve(thread.id, resolved);
      this.busy.delete(thread.id);
      this.expanded.delete(thread.id);
      const message = resolved
        ? "Resolved on GitHub. Resolving is sent right away, not held with your review."
        : "Reopened on GitHub.";
      this.announce(message);
      this.host.notify?.(message);
      await this.refresh();
      if (!this.focusOn(`resolve:${thread.id}`)) this.focusOn(`expand:${thread.id}`);
    } catch (error) {
      this.busy.delete(thread.id);
      this.announce(messageOf(error));
      this.host.notify?.(messageOf(error));
      this.render();
    }
  }

  private async submit(): Promise<void> {
    const data = this.data;
    if (!data || this.submitting) return;
    this.submitting = true;
    this.dialogError = undefined;
    this.render();
    const summary = (this.drafts.get("summary") ?? "").trim();
    try {
      await this.host.submit(this.event, summary === "" ? undefined : summary);
      this.submitting = false;
      this.drafts.delete("summary");
      this.dialog = undefined;
      const number = data.pullRequest.number;
      const done =
        this.event === "APPROVE"
          ? `Approved #${number}.`
          : this.event === "REQUEST_CHANGES"
            ? `Requested changes on #${number}.`
            : `Review submitted to #${number}.`;
      const message = `${done} Your comments are now on GitHub.`;
      this.event = "COMMENT";
      this.announce(message);
      this.host.notify?.(message);
      await this.refresh();
      this.giveBackFocus();
    } catch (error) {
      this.submitting = false;
      this.dialogError = messageOf(error);
      this.announce(this.dialogError);
      this.render();
    }
  }

  private async discard(): Promise<void> {
    try {
      await this.host.discard();
      this.dialog = undefined;
      this.discardAsk = false;
      const message = "Discarded your unsent comments.";
      this.announce(message);
      this.host.notify?.(message);
      await this.refresh();
      this.giveBackFocus();
    } catch (error) {
      this.discardAsk = false;
      this.dialogError = messageOf(error);
      this.announce(this.dialogError);
      this.render();
    }
  }

  /** Goes to a thread from a list: on this page, or on the page the host opens. */
  private jump(summary: ThreadSummary): void {
    const { thread, pages } = summary;
    this.dialog = undefined;
    this.discardAsk = false;
    if (this.goToThread(thread.id)) return;
    this.render();
    const page = pages[0];
    if (page) {
      this.host.openThread(thread.id, page.path);
      return;
    }
    const lines = thread.lines ?? { first: 1, last: 1 };
    this.host.openSource({
      source: formatSource({ path: thread.path, first: lines.first, last: lines.last }),
      via: [],
    });
  }

  private openPanel(anchor: Anchor): void {
    this.rememberOpener();
    this.panel = anchorKey(anchor);
    this.panelAnchor = anchor;
    this.render();
    const sheet = this.layer.querySelector<HTMLElement>(".sheet");
    (
      sheet?.querySelector<HTMLElement>(".thread") ?? sheet?.querySelector<HTMLElement>("button")
    )?.focus();
  }

  /** Opens the panel of a block with threads (which may be removed text). */
  private openEntry(entry: Entry): void {
    this.rememberOpener();
    this.panel = entry.key;
    this.panelAnchor = entry.removed ? undefined : entry.anchor;
    this.render();
    const sheet = this.layer.querySelector<HTMLElement>(".sheet");
    (
      sheet?.querySelector<HTMLElement>(".thread") ?? sheet?.querySelector<HTMLElement>("button")
    )?.focus();
  }

  private closePanel(): void {
    this.panel = undefined;
    this.panelAnchor = undefined;
    this.composer = undefined;
    this.render();
    this.giveBackFocus();
  }

  private openDialog(dialog: "submit" | "all"): void {
    if (this.data === undefined) return;
    this.rememberOpener();
    this.dialog = dialog;
    this.discardAsk = false;
    this.dialogError = undefined;
    this.render();
    this.focusDialog();
    // The lists need every thread; read them again, since they may have changed.
    void this.host.allThreads().then(
      (all) => {
        if (this.disposed) return;
        this.all = all;
        if (this.dialog === dialog) {
          const focus = this.focusKey();
          this.render();
          if (!this.restoreFocus(focus)) this.focusDialog();
        }
      },
      () => undefined,
    );
  }

  private closeDialog(): void {
    this.dialog = undefined;
    this.discardAsk = false;
    this.render();
    this.giveBackFocus();
  }

  private focusDialog(): void {
    const dialog = this.layer.querySelector<HTMLElement>(".dialog");
    const first = dialog?.querySelector<HTMLElement>(
      this.dialog === "submit" ? "input[type=radio]:checked" : "[aria-pressed=true]",
    );
    first?.focus();
  }

  private trapFocus(event: KeyboardEvent): void {
    const dialog = this.layer.querySelector<HTMLElement>(".dialog");
    if (!dialog) return;
    const focusable = Array.from(
      dialog.querySelectorAll<HTMLElement>("button, textarea, input, [tabindex]"),
    ).filter((el) => !(el as HTMLButtonElement).disabled && el.tabIndex >= 0);
    const first = focusable[0];
    const last = focusable.at(-1);
    const active = this.layer.activeElement;
    if (event.shiftKey && active === first) {
      event.preventDefault();
      last?.focus();
    } else if (!event.shiftKey && active === last) {
      event.preventDefault();
      first?.focus();
    }
  }

  // --- Focus ---

  private rememberOpener(): void {
    if (this.opener !== undefined) return;
    const active = this.activeElement();
    this.opener = active instanceof HTMLElement ? active : undefined;
  }

  private giveBackFocus(): void {
    const opener = this.opener;
    this.opener = undefined;
    if (opener?.isConnected) opener.focus({ preventScroll: true });
  }

  /** The focused element, inside the overlay's shadow roots or out. */
  private activeElement(): Element | null {
    let active = this.doc.activeElement;
    while (active?.shadowRoot?.activeElement) active = active.shadowRoot.activeElement;
    return active;
  }

  private focusKey(): { key: string; start: number; end: number } | undefined {
    const active = this.activeElement();
    const key = active instanceof HTMLElement ? active.dataset["focus"] : undefined;
    if (key === undefined || !active) return undefined;
    const field = active as HTMLTextAreaElement;
    return {
      key,
      start: typeof field.selectionStart === "number" ? field.selectionStart : 0,
      end: typeof field.selectionEnd === "number" ? field.selectionEnd : 0,
    };
  }

  private restoreFocus(focus: { key: string; start: number; end: number } | undefined): boolean {
    if (!focus) return false;
    const el = this.byFocusKey(focus.key);
    if (!el || (el as HTMLButtonElement).disabled) return false;
    el.focus({ preventScroll: true });
    if (el instanceof HTMLTextAreaElement) el.setSelectionRange(focus.start, focus.end);
    return true;
  }

  private byFocusKey(key: string): HTMLElement | undefined {
    for (const shadow of [this.layer, this.before, this.after, this.tools]) {
      for (const el of Array.from(shadow.querySelectorAll<HTMLElement>("[data-focus]"))) {
        if (el.dataset["focus"] === key) return el;
      }
    }
    return undefined;
  }

  private focusOn(key: string): boolean {
    const el = this.byFocusKey(key);
    if (!el || (el as HTMLButtonElement).disabled) return false;
    el.focus({ preventScroll: true });
    return true;
  }

  private findCard(threadId: string): HTMLElement | undefined {
    for (const shadow of [this.layer, this.before]) {
      for (const el of Array.from(shadow.querySelectorAll<HTMLElement>(".thread"))) {
        if (el.dataset["thread"] === threadId) return el;
      }
    }
    return undefined;
  }

  private announce(message: string): void {
    // A change of text is what's announced: clear it first, so the same message is said again.
    this.live.textContent = "";
    requestFrame(this.doc, () => {
      this.live.textContent = message;
    });
  }

  // --- Elements ---

  private el<K extends keyof HTMLElementTagNameMap>(
    tag: K,
    className: string,
    text?: string,
  ): HTMLElementTagNameMap[K] {
    const el = this.doc.createElement(tag);
    if (className) el.className = className;
    if (text !== undefined) el.textContent = text;
    return el;
  }

  private button(
    className: string,
    text: string,
    onClick: () => void,
    focusKey?: string,
  ): HTMLButtonElement {
    const button = this.el("button", className, text);
    button.type = "button";
    if (focusKey !== undefined) button.dataset["focus"] = focusKey;
    button.addEventListener("click", (event) => {
      event.stopPropagation();
      onClick();
    });
    return button;
  }

  private badge(kind: string, text: string): HTMLElement {
    return this.el("span", `badge ${kind}`, text);
  }
}

const HELD_REASON =
  "You have unsent comments, so GitHub adds replies to your review. Submit or discard it to reply at once.";

/** `rollouts.md:28`, from a block's anchor. */
function sourceLabel(anchor: Anchor): string {
  const colon = anchor.source.lastIndexOf(":");
  const match = /^(\d+)-(\d+)$/.exec(anchor.source.slice(colon + 1));
  if (colon < 0 || !match) return anchor.source;
  let path = anchor.source.slice(0, colon);
  try {
    path = decodeURIComponent(path);
  } catch {
    // Show it as it is.
  }
  return linesLabel(path, { first: Number(match[1]), last: Number(match[2]) });
}

/** A comment's first line, for a list. */
function oneLine(body: string): string {
  return body.replace(/\s+/g, " ").trim();
}

function messageOf(error: unknown): string {
  if (typeof error === "object" && error !== null && "message" in error) {
    return String((error as HostError).message);
  }
  return String(error);
}

function codeOf(error: unknown): string | undefined {
  if (typeof error === "object" && error !== null && "code" in error) {
    const code = (error as HostError).code;
    return typeof code === "string" ? code : undefined;
  }
  return undefined;
}

/** An element that can't hold children: the overlay's things go after it. */
function isVoid(element: Element): boolean {
  return /^(img|hr|br|input|video|audio|iframe|embed|object|svg|math|canvas)$/.test(
    element.localName,
  );
}

function cssId(text: string): string {
  return text.replace(/[^A-Za-z0-9_-]/g, "_");
}

function requestFrame(doc: Document, callback: () => void): void {
  const view = doc.defaultView;
  if (view && typeof view.requestAnimationFrame === "function")
    view.requestAnimationFrame(callback);
  else setTimeout(callback, 0);
}

let sheet: { doc: Document; sheet: CSSStyleSheet } | undefined;

/**
 * Gives a shadow root the overlay's stylesheet: one constructed stylesheet,
 * shared, where the browser has them (a content security policy that allows
 * no inline styles still allows these); a `<style>` element elsewhere.
 */
function adoptStyles(shadow: ShadowRoot, doc: Document): void {
  const view = doc.defaultView as (Window & { CSSStyleSheet?: typeof CSSStyleSheet }) | null;
  if (view?.CSSStyleSheet && "adoptedStyleSheets" in shadow) {
    try {
      if (sheet?.doc !== doc) {
        const constructed = new view.CSSStyleSheet();
        constructed.replaceSync(OVERLAY_CSS);
        sheet = { doc, sheet: constructed };
      }
      shadow.adoptedStyleSheets = [sheet.sheet];
      return;
    } catch {
      // Not constructable here: fall back to an element.
    }
  }
  const style = doc.createElement("style");
  style.setAttribute("data-overlay", "");
  style.textContent = OVERLAY_CSS;
  shadow.append(style);
}

/** Where a thread's Open source goes: its own lines when it has them, else its block's. */
function threadSource(thread: LocatedThread, entry: { anchor: Anchor }): Anchor {
  if (thread.lines === undefined || (thread.kind === "review" && thread.side === "LEFT")) {
    return entry.anchor;
  }
  return {
    source: formatSource({ path: thread.path, first: thread.lines.first, last: thread.lines.last }),
    via: entry.anchor.via,
  };
}
