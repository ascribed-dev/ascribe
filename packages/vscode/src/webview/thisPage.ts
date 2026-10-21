// This page's script, run in its webview view. It draws the sections the
// extension sends, which the model in `../ui/sidebar.ts` builds, and says
// which item the reader chose; the extension opens it.
//
// Each section is a `<details>` and its `<summary>`, each row that opens
// something a button, and the rows of a label and a value a description
// list, so a screen reader and the keyboard get them from the HTML itself:
// Tab between the headers and the buttons, and Enter or Space to open a
// section or choose a row. Which sections are open is the view's state, the
// same for every page, until the reader opens or closes one.
//
// The text is written by people and agents: titles, memos' first lines,
// threads' excerpts. All of it is set as text, never as HTML.
//
// An answer comes with each cursor move and each check, so drawing one keeps
// the reader's place: what's drawn already stays, each element with its key,
// and only what changed is drawn again, so the view keeps its scroll and the
// keyboard's focus. Up and Down move the focus between the headers and rows.

import { seenViolations, startView, type ViewChannel } from "./view.js";
import type { ThisPageAnswer, ThisPageProbe, ThisPageReport, ViewItem } from "./viewProtocol.js";

const main = document.querySelector<HTMLElement>('[data-role="view"]');
if (!main) throw new Error("This page's shell has no view");
const view: HTMLElement = main;

/** The view's state: whether each section is open, by its key, once the reader has chosen. */
function openSections(channel: ViewChannel<ThisPageReport>): Record<string, boolean> {
  const sections = channel.state()["sections"];
  if (typeof sections !== "object" || sections === null) return {};
  return Object.fromEntries(
    Object.entries(sections).filter((entry): entry is [string, boolean] => {
      return typeof entry[1] === "boolean";
    }),
  );
}

function element<K extends keyof HTMLElementTagNameMap>(
  tag: K,
  className?: string,
  text?: string,
): HTMLElementTagNameMap[K] {
  const made = document.createElement(tag);
  if (className) made.className = className;
  if (text !== undefined) made.textContent = text;
  return made;
}

function icon(name: string, className = ""): HTMLElement {
  const made = element(
    "span",
    `codicon codicon-${name.replace(/[^a-z0-9-]/g, "")} ${className}`.trimEnd(),
  );
  made.setAttribute("aria-hidden", "true");
  return made;
}

/** An item's label, with its hover text, and its description after it. */
function labelled(item: ViewItem): HTMLElement[] {
  const label = element("span", "label", item.label);
  if (item.tooltip) label.title = item.tooltip;
  return item.description ? [label, element("span", "description", item.description)] : [label];
}

function classes(item: ViewItem, base: string): string {
  return [base, item.dimmed ? "dimmed" : "", item.warn ? "warn" : ""].filter(Boolean).join(" ");
}

/** A button that chooses `item` of the answer drawn. */
function chooser(item: ViewItem, channel: ViewChannel<ThisPageReport>): HTMLButtonElement {
  const button = element("button");
  button.type = "button";
  button.dataset["key"] = item.key;
  button.addEventListener("click", () =>
    channel.post({ type: "open", seq: channel.seq, key: item.key }),
  );
  return button;
}

/** A row: its label as a description list's term, and its value as its details. */
function row(item: ViewItem, channel: ViewChannel<ThisPageReport>): HTMLElement {
  const made = element("div", classes(item, "row"));
  made.dataset["key"] = item.key;
  const term = element("dt");
  const label = element("span", "label", item.label);
  if (item.tooltip) label.title = item.tooltip;
  term.append(icon(item.icon), label);
  const details = element("dd");
  if (item.acts) {
    const button = chooser(item, channel);
    button.textContent = item.description;
    button.setAttribute("aria-label", `${item.label}: ${item.description}`);
    details.append(button);
  } else {
    details.textContent = item.description;
  }
  made.append(term, details);
  return made;
}

/** An item on its own line: a button when choosing it does something. */
function line(item: ViewItem, channel: ViewChannel<ThisPageReport>): HTMLElement {
  const made = element("li", classes(item, "item"));
  const body = item.acts ? chooser(item, channel) : element("span");
  if (!item.acts) body.dataset["key"] = item.key;
  body.append(icon(item.icon), ...labelled(item));
  made.append(body);
  return made;
}

/** A section's or group's items: rows in description lists, lines in lists, groups under headings. */
function drawItems(
  into: HTMLElement,
  items: readonly ViewItem[],
  depth: number,
  channel: ViewChannel<ThisPageReport>,
): void {
  let list: HTMLElement | undefined;
  for (const item of items) {
    if (item.kind === "group") {
      list = undefined;
      const group = element("div", classes(item, "group"));
      group.dataset["key"] = item.key;
      const head = element(depth === 0 ? "h3" : "h4", "group-head");
      head.append(icon(item.icon), ...labelled(item));
      group.append(head);
      drawItems(group, item.children, depth + 1, channel);
      into.append(group);
      continue;
    }
    const tag = item.kind === "row" ? "DL" : "UL";
    if (list?.tagName !== tag) {
      list = element(item.kind === "row" ? "dl" : "ul");
      into.append(list);
    }
    list.append(item.kind === "row" ? row(item, channel) : line(item, channel));
  }
}

function section(item: ViewItem, channel: ViewChannel<ThisPageReport>): HTMLDetailsElement {
  const details = element("details", classes(item, "section"));
  details.dataset["key"] = item.key;
  const chosen = openSections(channel)[item.key];
  const open = chosen ?? item.open;
  details.open = open;
  const summary = element("summary");
  // The twistie, as a tree draws it: the stylesheet shows the one for open or closed.
  summary.append(
    icon("chevron-right", "twistie closed"),
    icon("chevron-down", "twistie open"),
    icon(item.icon),
    ...labelled(item),
  );
  details.append(summary);
  const body = element("div", "section-body");
  drawItems(body, item.children, 0, channel);
  details.append(body);
  // Setting `open` above queues a toggle too: only the reader's choice is kept.
  details.addEventListener("toggle", () => {
    if (details.open === (openSections(channel)[item.key] ?? item.open)) return;
    channel.setState({
      ...channel.state(),
      sections: { ...openSections(channel), [item.key]: details.open },
    });
  });
  return details;
}

/** An item's text as the report gives it: its label, and its description in brackets. */
function text(item: ViewItem): string {
  return item.description ? `${item.label} (${item.description})` : item.label;
}

function flat(items: readonly ViewItem[]): string[] {
  return items.flatMap((item) => [text(item), ...flat(item.children)]);
}

/** Whether a node drawn already can stand for a new one: the same element with the same key, or text. */
function same(drawn: Node, fresh: Node): boolean {
  if (drawn.nodeType !== fresh.nodeType) return false;
  if (!(drawn instanceof Element) || !(fresh instanceof Element)) return true;
  return (
    drawn.tagName === fresh.tagName &&
    drawn.getAttribute("data-key") === fresh.getAttribute("data-key")
  );
}

/**
 * Makes what's drawn in `into` what `fresh` is, keeping each element that can
 * stand for its new one, with its listeners, its focus, and what's under it,
 * and changing only its attributes and text that differ.
 */
function patch(into: Element, fresh: readonly Node[]): void {
  const drawn = [...into.childNodes];
  fresh.forEach((node, i) => {
    const kept = drawn[i];
    if (!kept) {
      into.append(node);
    } else if (!same(kept, node)) {
      kept.replaceWith(node);
    } else if (kept instanceof Element && node instanceof Element) {
      for (const name of kept.getAttributeNames()) {
        if (!node.hasAttribute(name)) kept.removeAttribute(name);
      }
      for (const { name, value } of node.attributes) {
        if (kept.getAttribute(name) !== value) kept.setAttribute(name, value);
      }
      patch(kept, [...node.childNodes]);
    } else if (kept.nodeValue !== node.nodeValue) {
      kept.nodeValue = node.nodeValue;
    }
  });
  for (const extra of drawn.slice(fresh.length)) extra.remove();
}

const scroller = document.scrollingElement ?? document.documentElement;

/** The key of what has the focus: an item's, or its section's for a header. */
function focusedKey(): string | null {
  const focused = document.activeElement;
  if (!focused || !view.contains(focused)) return null;
  const keyed = focused.tagName === "SUMMARY" ? focused.parentElement : focused;
  return keyed?.getAttribute("data-key") ?? null;
}

/** Puts the focus on the item of `key`, or on a section's header by its key; whether it could. */
function focusKey(key: string): boolean {
  const selector = `[data-key="${CSS.escape(key)}"]`;
  const target = view.querySelector<HTMLElement>(`details${selector} > summary, button${selector}`);
  if (!target) return false;
  target.focus();
  return document.activeElement === target;
}

/** The headers and rows the keyboard goes between: those a closed section doesn't hide. */
function stops(): HTMLElement[] {
  return [...view.querySelectorAll<HTMLElement>("summary, button")].filter((stop) => {
    const section = (
      stop.tagName === "SUMMARY" ? stop.parentElement?.parentElement : stop
    )?.closest("details");
    return !section || section.open;
  });
}

// Up and Down: the focus moves to the header or row before or after, in the
// order Tab goes, skipping what a closed section hides.
document.addEventListener("keydown", (event) => {
  if (event.key !== "ArrowDown" && event.key !== "ArrowUp") return;
  if (event.altKey || event.ctrlKey || event.metaKey || event.shiftKey) return;
  const all = stops();
  const at = all.indexOf(document.activeElement as HTMLElement);
  const next =
    event.key === "ArrowDown"
      ? all[at < 0 ? 0 : Math.min(at + 1, all.length - 1)]
      : all[at < 0 ? all.length - 1 : Math.max(at - 1, 0)];
  event.preventDefault();
  next?.focus();
});

/** The answer drawn last, which a probe's report describes. */
let shown: ThisPageAnswer | undefined;
/** The newest probe answered. */
let probed = 0;

function report(answer: ThisPageAnswer): ThisPageReport {
  const sample = view.querySelector(".row dd, .item .label") ?? view;
  const { whiteSpace } = getComputedStyle(sample);
  return {
    message: answer.message,
    sections: (answer.message !== null ? [] : answer.sections).map((item) => ({
      key: item.key,
      label: item.label,
      open:
        view.querySelector<HTMLDetailsElement>(`details[data-key="${CSS.escape(item.key)}"]`)
          ?.open ?? false,
      items: flat(item.children),
    })),
    wraps: whiteSpace !== "nowrap" && whiteSpace !== "pre",
    focused: focusedKey(),
    hasFocus: document.hasFocus(),
    scroll: Math.round(scroller.scrollTop),
    probed,
    violations: seenViolations(),
  };
}

function draw(answer: ThisPageAnswer, channel: ViewChannel<ThisPageReport>): ThisPageReport {
  const focused = focusedKey();
  const scroll = scroller.scrollTop;
  patch(
    view,
    answer.message !== null
      ? [element("p", "message", answer.message)]
      : answer.sections.map((item) => section(item, channel)),
  );
  // What had the focus is gone: the focus goes to its section's header.
  if (focused !== null && focusedKey() === null && !focusKey(focused)) {
    focusKey(focused.split("/")[0] ?? focused);
  }
  scroller.scrollTop = scroll;
  shown = answer;
  return report(answer);
}

const channel = startView<ThisPageAnswer, ThisPageReport>(draw);

window.addEventListener("message", (event: MessageEvent<unknown>) => {
  const probe = event.data as Partial<ThisPageProbe> | null;
  if (probe?.type !== "probe" || typeof probe.probe !== "number" || !shown) return;
  if (probe.open) {
    for (const details of view.querySelectorAll("details")) details.open = true;
  }
  if (typeof probe.focus === "string") focusKey(probe.focus);
  if (typeof probe.scroll === "number") scroller.scrollTop = probe.scroll;
  if (probe.press && document.activeElement instanceof HTMLElement) {
    document.activeElement.click();
  }
  probed = Math.max(probed, probe.probe);
  channel.post({ type: "drawn", seq: channel.seq, report: report(shown) });
});
