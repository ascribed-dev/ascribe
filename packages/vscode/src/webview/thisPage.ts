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

import { seenViolations, startView, type ViewChannel } from "./view.js";
import type { ThisPageAnswer, ThisPageReport, ViewItem } from "./viewProtocol.js";

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

function draw(answer: ThisPageAnswer, channel: ViewChannel<ThisPageReport>): ThisPageReport {
  const sections =
    answer.message !== null
      ? []
      : answer.sections.map((item) => ({ item, details: section(item, channel) }));
  view.replaceChildren(
    ...(answer.message !== null
      ? [element("p", "message", answer.message)]
      : sections.map(({ details }) => details)),
  );
  const sample = view.querySelector(".row dd, .item .label") ?? view;
  const { whiteSpace } = getComputedStyle(sample);
  return {
    message: answer.message,
    sections: sections.map(({ item, details }) => ({
      key: item.key,
      label: item.label,
      open: details.open,
      items: flat(item.children),
    })),
    wraps: whiteSpace !== "nowrap" && whiteSpace !== "pre",
    violations: seenViolations(),
  };
}

startView<ThisPageAnswer, ThisPageReport>(draw);
