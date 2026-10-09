// Review in the preview: the header (the base, the page's change count and
// its breakdown, what to show, and next and previous change) and the marks on
// the page, applied with @ascribed/review's marks to the page the server
// rendered with source anchors. Past the last change, the header offers the
// next changed page instead of wrapping round.

import {
  clearMarks,
  describeSource,
  disarm,
  goTo,
  markChanges,
  setShow,
  showSources,
  type Mark,
  type Show,
} from "@ascribed/review/marks";
import { countsText as breakdown, errorsText } from "../preview/counts.js";
import type { FromWebview, ReviewView } from "../preview/protocol.js";
import type { HostError, PromptRequest } from "@ascribed/review/overlay";
import { againstText, threadsNotice } from "../preview/threadsText.js";
import type { Threads } from "./threads.js";
import {
  CLASS_LABEL,
  CLASS_LABEL_BEFORE,
  CLASS_MARKS,
  DATA_SHOW,
  DATA_SOURCE,
  DATA_VIA,
} from "../names.js";

/** "10 changes on this page", or "3 of 10 on this page" while stepping through them. */
function position(total: number, at: number): string {
  if (total === 0) return "No changes on this page";
  if (at >= 0 && at < total) return `${at + 1} of ${total} on this page`;
  return total === 1 ? "1 change on this page" : `${total} changes on this page`;
}

/** The model file, which a page can change through but isn't a file of content. */
const MODEL_FILE = "ascribe.toml";

const SHOWS: [Show, string][] = [
  ["changes", "Changes"],
  ["will", "As it will be"],
  ["was", "As it was"],
];

export class Review {
  private view: ReviewView | null = null;
  private path: string | null = null;
  private marks: Mark[] = [];
  private show: Show = "changes";
  /** The change last gone to, from 0; -1 before the first. */
  private at = -1;
  private atEnd = false;
  private open = false;
  /** Whether the More review actions menu is open. */
  private menuOpen = false;
  /** The next changed page: `undefined` while asking. */
  private next: { page: { path: string; title: string } | null; first: boolean } | undefined;
  private stopSources: (() => void) | undefined;

  constructor(
    private readonly bar: HTMLElement,
    private readonly content: HTMLElement,
    private readonly post: (message: FromWebview) => void,
    private readonly threads?: Threads,
  ) {
    content.addEventListener("click", (event) => this.clickLabel(event));
    // A click anywhere but the menu closes it.
    document.addEventListener("click", (event) => {
      if (!this.menuOpen) return;
      if (event.target instanceof Element && event.target.closest(".more")) return;
      this.menuOpen = false;
      this.drawBar();
    });
    content.addEventListener("keydown", (event) => {
      if (event.key === "Enter") this.clickLabel(event);
    });
  }

  /**
   * Draws the header for `view` and marks the page just put in `content`.
   * `prepare` points the page as it was at its assets, as the page's own are.
   */
  draw(view: ReviewView | null, path: string | null, prepare?: (root: ParentNode) => void): void {
    if (path !== this.path) {
      this.at = -1;
      this.atEnd = false;
      this.open = false;
      this.menuOpen = false;
      this.next = undefined;
    }
    this.view = view;
    this.path = path;
    this.mark(prepare);
    if (view?.goToFirst && this.marks.length > 0) this.step(1);
    this.drawBar();
  }

  /** Draws the header again: the thread count changed. */
  redraw(): void {
    if (this.view) this.drawBar();
  }

  /** The marks on the page, by kind. */
  counts(): Record<string, number> {
    const counts: Record<string, number> = {};
    for (const mark of this.marks) counts[mark.change.kind] = (counts[mark.change.kind] ?? 0) + 1;
    return counts;
  }

  /** The header's text, or `null` when it isn't shown. */
  headerText(): string | null {
    return this.bar.hidden ? null : (this.bar.textContent ?? "");
  }

  /** The extension's answer to `atEnd`. */
  nextPage(page: { path: string; title: string } | null, first: boolean): void {
    this.next = { page, first };
    this.drawBar();
  }

  private mark(prepare?: (root: ParentNode) => void): void {
    const view = this.view;
    if (!view) {
      this.marks = [];
      this.stopSources?.();
      this.stopSources = undefined;
      clearMarks(this.content);
      this.content.classList.remove(CLASS_MARKS);
      this.content.removeAttribute(DATA_SHOW);
      return;
    }
    let was: ParentNode | null = null;
    if (view.wasHtml !== null) {
      const template = document.createElement("template");
      template.innerHTML = view.wasHtml;
      disarm(template.content);
      prepare?.(template.content);
      was = template.content;
    }
    this.marks = markChanges(this.content, view.page?.changes ?? [], { was });
    setShow(this.content, this.show);
    for (const mark of this.marks) {
      const source = mark.element.getAttribute(DATA_SOURCE);
      const label = labelOf(mark.element);
      if (source === null || !label) continue;
      label.title = `Open ${describeSource(source, mark.element.getAttribute(DATA_VIA))}`;
      label.tabIndex = 0;
      label.setAttribute("role", "link");
    }
    this.stopSources ??= showSources(this.content);
  }

  /** A click on a mark's label opens the block's source. */
  private clickLabel(event: Event): void {
    const label = event.target instanceof Element ? event.target.closest(`.${CLASS_LABEL}`) : null;
    if (!label || !this.content.contains(label)) return;
    const mark = this.marks.find((m) => labelOf(m.element) === label);
    const source = mark?.element.getAttribute(DATA_SOURCE);
    if (!source) return;
    event.preventDefault();
    event.stopPropagation();
    this.post({ type: "openSource", source });
  }

  private step(by: 1 | -1): void {
    const total = this.marks.length;
    if (by === 1 && this.at + 1 >= total) {
      this.at = total;
      if (!this.atEnd) {
        this.atEnd = true;
        this.next = undefined;
        this.post({ type: "atEnd" });
      }
      this.drawBar();
      return;
    }
    this.atEnd = false;
    this.at = Math.max(0, Math.min(total - 1, this.at + by));
    if (this.show !== "changes") this.setShow("changes");
    const mark = this.marks[this.at];
    if (mark) goTo(mark.element);
    this.drawBar();
  }

  private setShow(show: Show): void {
    this.show = show;
    setShow(this.content, show);
    // Removed or new blocks showed or hid: the threads beside them move.
    this.threads?.layout();
  }

  /**
   * More review actions (⋯): Prompt agent about the page's changes, and about
   * each file it changed through. `undefined` when the page didn't change.
   */
  private moreMenu(view: ReviewView): HTMLElement | undefined {
    const page = view.page;
    if (!this.threads || !page || page.status === "removed") return undefined;
    const items: [string, PromptRequest][] = [
      ["Prompt agent: review this page", { kind: "page-changes" }],
    ];
    for (const fragment of page.because) {
      if (fragment === MODEL_FILE) continue;
      items.push([
        page.because.length === 1
          ? "Prompt agent: check this fragment's pages"
          : `Prompt agent: check the pages that show ${fragment}`,
        { kind: "fragment-reach", fragment },
      ]);
    }
    const wrap = span("more", "");
    const toggle = button("⋯", "square", () => {
      this.menuOpen = !this.menuOpen;
      this.drawBar();
      if (this.menuOpen) this.bar.querySelector<HTMLElement>(".menu button")?.focus();
    });
    toggle.setAttribute("aria-label", "More review actions");
    toggle.title = "More review actions";
    toggle.setAttribute("aria-haspopup", "menu");
    toggle.setAttribute("aria-expanded", String(this.menuOpen));
    wrap.append(toggle);
    if (!this.menuOpen) return wrap;
    const menu = document.createElement("div");
    menu.className = "menu";
    menu.setAttribute("role", "menu");
    menu.setAttribute("aria-label", "More review actions");
    const close = (): void => {
      this.menuOpen = false;
      this.drawBar();
      this.bar.querySelector<HTMLElement>('[aria-label="More review actions"]')?.focus();
    };
    for (const [label, request] of items) {
      const item = button(label, "", () => {
        close();
        this.threads?.prompt(request).catch((error: HostError) => {
          this.post({ type: "notify", message: `The prompt couldn't be built: ${error.message}` });
        });
      });
      item.setAttribute("role", "menuitem");
      menu.append(item);
    }
    menu.addEventListener("keydown", (event) => {
      const buttons = [...menu.querySelectorAll<HTMLElement>("button")];
      const at = buttons.indexOf(document.activeElement as HTMLElement);
      if (event.key === "Escape") close();
      else if (event.key === "ArrowDown") buttons[(at + 1) % buttons.length]?.focus();
      else if (event.key === "ArrowUp")
        buttons[(at - 1 + buttons.length) % buttons.length]?.focus();
      else return;
      event.preventDefault();
    });
    wrap.append(menu);
    return wrap;
  }

  private drawBar(): void {
    const view = this.view;
    this.bar.hidden = view === null;
    if (view === null) {
      this.bar.replaceChildren();
      return;
    }
    const threads = view.threads;
    const pr = threads?.pullRequest;
    const info = span("grow", "");
    if (pr) {
      const url = pr.url;
      const number = button(`#${pr.number}`, "link", () => this.post({ type: "open", href: url }));
      number.title = `Open pull request #${pr.number} on GitHub`;
      info.append(number, againstText(threads).slice(`#${pr.number}`.length));
    } else {
      info.append(againstText(null));
    }
    const base = document.createElement("b");
    base.textContent = view.base;
    base.title = `Compared with ${view.commit}, where this branch left ${view.base}`;
    info.append(base);
    if (view.page?.status === "added") info.append(" · a new page");
    if (view.causes.length > 0) {
      info.append(" · changed only through ");
      view.causes.forEach((cause, i) => {
        if (i > 0) info.append(", ");
        info.append(
          button(cause.label, "link", () => this.post({ type: "openFile", path: cause.path })),
        );
      });
    }
    const total = this.marks.length;
    const counts = view.page?.counts;
    const stepping = this.at >= 0 && this.at < total && this.show === "changes";
    const count = button(position(total, stepping ? this.at : -1), "link", () => {
      this.open = !this.open;
      this.drawBar();
    });
    count.classList.add("position");
    count.setAttribute("aria-expanded", String(this.open));
    if (counts) count.title = breakdown(counts);
    const shows = document.createElement("div");
    shows.className = "segmented";
    shows.setAttribute("role", "group");
    shows.setAttribute("aria-label", "Show");
    for (const [show, label] of SHOWS) {
      const choice = button(label, "", () => {
        this.setShow(show);
        this.drawBar();
      });
      choice.setAttribute("aria-pressed", String(this.show === show));
      shows.append(choice);
    }
    const previous = button("↑", "square", () => this.step(-1));
    previous.setAttribute("aria-label", "Previous change");
    previous.title = "Previous change";
    previous.disabled = total === 0;
    const next = button("↓", "square", () => this.step(1));
    next.setAttribute("aria-label", "Next change");
    next.title = "Next change";
    const cells = [info, count, shows, previous, next];
    if (this.threads?.active) {
      const n = this.threads.count();
      const all = button(n === undefined ? "Comments" : `Comments (${n})`, "", () =>
        this.threads?.showAll(),
      );
      all.title = "Every comment on the pull request's pages";
      all.setAttribute("aria-haspopup", "dialog");
      cells.splice(1, 0, all);
    }
    const more = this.moreMenu(view);
    if (more) cells.push(more);
    const header = row(cells);
    if (this.open && counts) header.append(span("legend", breakdown(counts) || "No changes"));
    const parts = [header];
    const errors = errorsText(view.errors);
    if (errors) {
      const box = document.createElement("div");
      box.className = "notice";
      box.append(
        span("", errors),
        span("spacer", ""),
        button("Show problems", "", () => this.post({ type: "showProblems" })),
      );
      parts.push(box);
    }
    const notice = threads ? threadsNotice(threads) : undefined;
    if (notice) {
      const box = document.createElement("div");
      box.className = "notice";
      box.append(span("", notice.text), span("spacer", ""));
      notice.actions.forEach((action, i) => {
        box.append(
          button(action.label, i === 0 ? "primary" : "", () => {
            const m = action.message;
            if (m === "pull" || m === "push" || m === "fetch")
              this.post({ type: "git", command: m });
            else if (m === "refresh") this.post({ type: "refreshThreads" });
            else this.post({ type: m });
          }),
        );
      });
      parts.push(box);
    }
    if (this.atEnd) {
      const notice = document.createElement("div");
      notice.className = "notice";
      notice.append(
        span(
          "",
          total === 0 ? "This page has no changes." : "That was the last change on this page.",
        ),
        span("spacer", ""),
      );
      const next = this.next;
      if (next === undefined) notice.append(span("", "Looking for the next changed page…"));
      else if (next.page === null) notice.append(span("", "No other page changed."));
      else {
        const target = next.page;
        notice.append(
          button(`${next.first ? "First" : "Next"} changed page: ${target.title}`, "primary", () =>
            this.post({ type: "openPage", path: target.path }),
          ),
        );
      }
      parts.push(notice);
    }
    this.bar.replaceChildren(...parts);
  }
}

/** The label the marks put on a block: inside it at its start, or just before it. */
function labelOf(element: HTMLElement): HTMLElement | undefined {
  const inside = element.querySelector<HTMLElement>(`:scope > .${CLASS_LABEL}`);
  if (inside) return inside;
  const before = element.previousElementSibling;
  return before instanceof HTMLElement && before.classList.contains(CLASS_LABEL_BEFORE)
    ? before
    : undefined;
}

function row(children: HTMLElement[]): HTMLElement {
  const div = document.createElement("div");
  div.className = "review-row";
  div.append(...children);
  return div;
}

function span(className: string, text: string): HTMLElement {
  const el = document.createElement("span");
  if (className) el.className = className;
  el.textContent = text;
  return el;
}

function button(text: string, kind: string, onClick: () => void): HTMLButtonElement {
  const el = document.createElement("button");
  el.type = "button";
  if (kind) el.className = kind;
  el.textContent = text;
  el.addEventListener("click", onClick);
  return el;
}
