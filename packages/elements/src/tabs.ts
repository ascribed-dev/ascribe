// <ascribe-tabs> and <ascribe-tab> (CONTRACT.md §3): the library's only
// element with behavior. Without this script every tab shows with its label
// (CSS generated content); with it, the group follows the WAI-ARIA tabs
// pattern.

const STORAGE_PREFIX = "ascribe-tabs:";

/** Choices made this page load, for readers whose storage is unavailable. */
const pageChoices = new Map<string, string>();

/** Every connected, initialized group with a `sync` attribute. */
const syncedGroups = new Set<AscribeTabs>();

let idCounter = 0;

function readChoice(dimension: string): string | null {
  // This page's choices are always at least as fresh as storage, which a
  // failed write leaves behind.
  const chosen = pageChoices.get(dimension);
  if (chosen !== undefined) return chosen;
  try {
    return localStorage.getItem(STORAGE_PREFIX + dimension);
  } catch {
    // Storage can be blocked or unavailable.
    return null;
  }
}

function writeChoice(dimension: string, value: string): void {
  pageChoices.set(dimension, value);
  try {
    localStorage.setItem(STORAGE_PREFIX + dimension, value);
  } catch {
    // The choice lasts for the page only.
  }
}

function valuesOf(tab: Element): string[] {
  return (tab.getAttribute("value") ?? "").split(/\s+/).filter(Boolean);
}

/** A tab panel: one arm of a `@variant` group. Styled by CSS; no behavior. */
export class AscribeTab extends HTMLElement {}

export class AscribeTabs extends HTMLElement {
  #tablist: HTMLElement | null = null;
  #tabs: HTMLElement[] = [];
  #buttons: HTMLButtonElement[] = [];
  #selected = 0;
  #ready = false;

  connectedCallback(): void {
    // A parser-created element is connected before its children are parsed.
    if (document.readyState === "loading") {
      document.addEventListener("DOMContentLoaded", () => this.#setup(), { once: true });
    } else {
      this.#setup();
    }
  }

  disconnectedCallback(): void {
    this.#teardown();
  }

  #setup(): void {
    if (!this.isConnected || this.#ready) return;
    this.#tabs = Array.from(this.children).filter(
      (child): child is HTMLElement => child.localName === "ascribe-tab",
    );
    if (this.#tabs.length === 0) return;
    this.#ready = true;

    const tablist = document.createElement("div");
    tablist.setAttribute("role", "tablist");
    this.#tablist = tablist;

    this.#buttons = this.#tabs.map((tab) => {
      if (!tab.id) tab.id = `ascribe-tab-${++idCounter}`;
      const button = document.createElement("button");
      button.type = "button";
      button.id = `${tab.id}-button`;
      button.setAttribute("role", "tab");
      button.setAttribute("aria-controls", tab.id);
      button.textContent = tab.getAttribute("label") ?? "";
      button.addEventListener("click", () => this.#choose(this.#buttons.indexOf(button)));
      button.addEventListener("keydown", (event) => this.#onKeydown(event));
      tablist.append(button);
      tab.setAttribute("role", "tabpanel");
      tab.setAttribute("aria-labelledby", button.id);
      return button;
    });
    this.prepend(tablist);

    const sync = this.getAttribute("sync");
    let initial = 0;
    if (sync !== null) {
      syncedGroups.add(this);
      const remembered = readChoice(sync);
      const match = remembered === null ? -1 : this.#indexOfValue(remembered);
      if (match >= 0) initial = match;
    }
    this.#select(initial);
  }

  #teardown(): void {
    if (!this.#ready) return;
    syncedGroups.delete(this);
    this.#tablist?.remove();
    for (const tab of this.#tabs) {
      tab.removeAttribute("role");
      tab.removeAttribute("aria-labelledby");
      tab.removeAttribute("hidden");
    }
    this.#tablist = null;
    this.#tabs = [];
    this.#buttons = [];
    this.#ready = false;
  }

  #indexOfValue(value: string): number {
    return this.#tabs.findIndex((tab) => valuesOf(tab).includes(value));
  }

  /** Select a tab without recording or syncing the choice. */
  #select(index: number): void {
    this.#selected = index;
    this.#tabs.forEach((tab, i) => {
      tab.toggleAttribute("hidden", i !== index);
    });
    this.#buttons.forEach((button, i) => {
      button.setAttribute("aria-selected", String(i === index));
      button.tabIndex = i === index ? 0 : -1;
    });
  }

  /** The reader picked a tab: select it and, for a synced group, tell the rest. */
  #choose(index: number): void {
    this.#select(index);
    const sync = this.getAttribute("sync");
    const tab = this.#tabs[index];
    const choice = tab ? valuesOf(tab)[0] : undefined;
    if (sync === null || choice === undefined) return;
    writeChoice(sync, choice);
    for (const group of syncedGroups) {
      if (group !== this && group.getAttribute("sync") === sync) group.#follow(choice);
    }
  }

  /** Another group with the same `sync` made a choice; keep our selection if we have no match. */
  #follow(value: string): void {
    const index = this.#indexOfValue(value);
    if (index >= 0) this.#select(index);
  }

  #onKeydown(event: KeyboardEvent): void {
    const last = this.#tabs.length - 1;
    let next: number;
    switch (event.key) {
      case "ArrowRight":
        next = this.#selected === last ? 0 : this.#selected + 1;
        break;
      case "ArrowLeft":
        next = this.#selected === 0 ? last : this.#selected - 1;
        break;
      case "Home":
        next = 0;
        break;
      case "End":
        next = last;
        break;
      default:
        return;
    }
    event.preventDefault();
    this.#choose(next);
    this.#buttons[next]?.focus();
  }
}
