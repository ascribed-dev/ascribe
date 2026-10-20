// The Project view's script, run in its webview view. It draws the answer the
// extension sends, which `../ui/projectHealth.ts` builds, and says which
// count or action the reader chose; the extension does it.
//
// The head is the project's name as a heading, with its folder, its server's
// state, and its binary's version under it. Health and Actions are sections
// as This page draws them: a `<details>` and its `<summary>` each, open at
// first, which stay as the reader leaves them (the view's state). Health's
// counts are a description list, each count's value and the action that
// brings it current buttons when choosing them does something, so a screen
// reader and the keyboard get them from the HTML itself. A count that isn't
// current says so in words, never by color alone.
//
// The project's name and folder are the author's text: all of it is set as
// text, never as HTML.

import { seenViolations, startView, type ViewChannel } from "./view.js";
import type { ProjectAnswer, ProjectCount, ProjectReport } from "./viewProtocol.js";

const main = document.querySelector<HTMLElement>('[data-role="view"]');
if (!main) throw new Error("The Project view's shell has no view");
const view: HTMLElement = main;

type Channel = ViewChannel<ProjectReport>;

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

/** A button that chooses `key` of the answer drawn. */
function chooser(key: string, channel: Channel): HTMLButtonElement {
  const button = element("button");
  button.type = "button";
  button.dataset["key"] = key;
  button.addEventListener("click", () => channel.post({ type: "open", seq: channel.seq, key }));
  return button;
}

/** The view's state: whether each section is open, by its key, once the reader has chosen. */
function openSections(channel: Channel): Record<string, boolean> {
  const sections = channel.state()["sections"];
  if (typeof sections !== "object" || sections === null) return {};
  return Object.fromEntries(
    Object.entries(sections).filter((entry): entry is [string, boolean] => {
      return typeof entry[1] === "boolean";
    }),
  );
}

/** A section, open at first, with its header's label and description. */
function section(
  key: string,
  label: string,
  description: string,
  body: HTMLElement,
  channel: Channel,
): HTMLDetailsElement {
  const details = element("details", "section");
  details.dataset["key"] = key;
  details.open = openSections(channel)[key] ?? true;
  const summary = element("summary");
  summary.append(
    icon("chevron-right", "twistie closed"),
    icon("chevron-down", "twistie open"),
    element("span", "label", label),
  );
  if (description) summary.append(element("span", "description", description));
  details.append(summary, body);
  // Setting `open` above queues a toggle too: only the reader's choice is kept.
  details.addEventListener("toggle", () => {
    if (details.open === (openSections(channel)[key] ?? true)) return;
    channel.setState({
      ...channel.state(),
      sections: { ...openSections(channel), [key]: details.open },
    });
  });
  return details;
}

function head(answer: ProjectAnswer): HTMLElement {
  const made = element("header", "head");
  if (!answer.head) return made;
  const { name, folder, state, version } = answer.head;
  const title = element("h2", "project-name", name);
  title.title = folder;
  const facts = element("dl");
  const fact = (label: string, value: string) => {
    const row = element("div", "row");
    const term = element("dt");
    term.append(element("span", "label", label));
    row.append(term, element("dd", undefined, value));
    facts.append(row);
  };
  fact("Folder", folder);
  fact("Server", state);
  if (version !== null) fact("Version", version);
  made.append(title, facts);
  return made;
}

/** A count: its label, then its value, what it's from, and the action that brings it current. */
function count(item: ProjectCount, channel: Channel): HTMLElement {
  const made = element("div", `row count ${item.state}`);
  made.dataset["key"] = item.key;
  const term = element("dt");
  term.append(element("span", "label", item.label));
  const details = element("dd");
  let value: HTMLElement;
  if (item.opens) {
    value = chooser(item.key, channel);
    value.textContent = item.value;
    value.setAttribute("aria-label", `${item.label}: ${item.value}. ${item.note}`);
  } else {
    value = element("span", undefined, item.value);
    // A dash is read as nothing: the note says why there's no count.
    if (item.value === "—") value.setAttribute("aria-hidden", "true");
  }
  value.classList.add("value");
  const note = element("span", "note");
  // Out of date is a warning, in words as well as its icon.
  if (item.state === "outOfDate") note.append(icon("warning"));
  note.append(element("span", undefined, item.note));
  details.append(value, note);
  if (item.action) {
    const action = chooser(item.action.key, channel);
    action.classList.add("action");
    action.append(icon("play"), element("span", "label", item.action.label));
    details.append(action);
  }
  made.append(term, details);
  return made;
}

function draw(answer: ProjectAnswer, channel: Channel): ProjectReport {
  if (answer.message !== null) {
    view.replaceChildren(element("p", "message", answer.message));
    return {
      message: answer.message,
      head: [],
      sections: [],
      health: [],
      buttons: [],
      violations: seenViolations(),
    };
  }
  const counts = element("dl");
  for (const item of answer.health) counts.append(count(item, channel));
  const actions = element("ul");
  for (const action of answer.actions) {
    const item = element("li", "item");
    const button = chooser(action.key, channel);
    button.append(icon(action.icon), element("span", "label", action.label));
    button.title = action.caption;
    item.append(button);
    actions.append(item);
  }
  const toDo =
    answer.toDo === 0 ? "" : `${answer.toDo} ${answer.toDo === 1 ? "thing" : "things"} to do`;
  const healthBody = element("div", "section-body");
  healthBody.append(counts);
  const actionsBody = element("div", "section-body");
  actionsBody.append(actions);
  const sections = [
    section("health", "Health", toDo, healthBody, channel),
    section("actions", "Actions", "", actionsBody, channel),
  ];
  const top = head(answer);
  view.replaceChildren(top, ...sections);
  return {
    message: null,
    head: [...top.querySelectorAll("h2, .row")].map((line) =>
      line.tagName === "H2"
        ? (line.textContent ?? "")
        : [...line.children].map((part) => part.textContent ?? "").join(": "),
    ),
    sections: sections.map((details) => ({
      key: details.dataset["key"] ?? "",
      label: details.querySelector("summary")?.textContent ?? "",
      open: details.open,
    })),
    health: answer.health.map((item) => {
      const row = view.querySelector(`.count[data-key="${item.key}"]`);
      const value = row?.querySelector(".value");
      const label = row?.querySelector("dt .label");
      return {
        key: item.key,
        value: value?.textContent ?? "",
        note: row?.querySelector(".note")?.textContent ?? "",
        state: item.state,
        // Drawn in another color than its label's, as the stylesheet dims a zero.
        dimmed:
          value && label ? getComputedStyle(value).color !== getComputedStyle(label).color : false,
      };
    }),
    buttons: [...view.querySelectorAll("button")].map((button) => button.textContent ?? ""),
    violations: seenViolations(),
  };
}

startView<ProjectAnswer, ProjectReport>(draw);
