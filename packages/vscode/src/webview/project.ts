// The Project view's script, run in its webview view. It draws the answer the
// extension sends, which `../ui/projectHealth.ts` builds, and says which
// count or action the reader chose; the extension does it.
//
// The head is the project's name as a heading, with its two settings
// buttons beside it, and its folder, its server's state with Show Output and
// Restart, its binary's version, and the build switch under it. The switch
// is a `<select>`, so a screen reader and the keyboard get it from the HTML,
// and choosing a build sends it back to the extension, which sets the one
// choice the status bar and the preview follow. Actions, Health, and About
// are sections as This page draws them: a `<details>` and its `<summary>`
// each, Actions and Health open at first and About closed, which stay as the
// reader leaves them (the view's state). Actions come first, so they stay
// where they are as Health's counts change. Actions are in their groups,
// each under its heading; in Restricted Mode they're drawn off, under the
// sentence that says why. Health's counts are in groups by when each is
// from: the group's head says it once, with the action that brings its
// counts current while they aren't, and each count is a line of a
// description list, its label then its value, the value a button when
// choosing it does something. A count that isn't current says so in words,
// never by color alone: a dash is named by its group's note.
//
// The project's name and folder are the author's text: all of it is set as
// text, never as HTML.

import { seenViolations, startView, type ViewChannel } from "./view.js";
import type {
  ProjectActionGroup,
  ProjectAnswer,
  ProjectButton,
  ProjectCheck,
  ProjectCount,
  ProjectHead,
  ProjectHealthGroup,
  ProjectReport,
} from "./viewProtocol.js";

/** What the build switch sends, before the build's name: `projectHealth.ts`'s `CHOOSE_BUILD`. */
const CHOOSE_BUILD = "chooseBuild/";

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

/** A section, open at first unless `closed`, with its header's label and description. */
function section(
  key: string,
  label: string,
  description: string,
  body: HTMLElement,
  channel: Channel,
  closed = false,
): HTMLDetailsElement {
  const details = element("details", "section");
  details.dataset["key"] = key;
  const first = !closed;
  details.open = openSections(channel)[key] ?? first;
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
    if (details.open === (openSections(channel)[key] ?? first)) return;
    channel.setState({
      ...channel.state(),
      sections: { ...openSections(channel), [key]: details.open },
    });
  });
  return details;
}

/** A head's button: its icon, with its label for a screen reader and its tooltip on hover. */
function iconButton(item: ProjectButton, channel: Channel): HTMLButtonElement {
  const button = chooser(item.key, channel);
  button.classList.add("icon-button");
  button.setAttribute("aria-label", item.label);
  button.title = item.tooltip;
  button.append(icon(item.icon));
  return button;
}

/** The build switch: the build you're looking at, among the project's. */
function buildSwitch(build: NonNullable<ProjectHead["build"]>, channel: Channel): HTMLElement {
  const select = element("select", "value build-switch");
  select.id = "build-switch";
  for (const choice of build.choices) {
    const option = element(
      "option",
      undefined,
      choice.description ? `${choice.name}, ${choice.description}` : choice.name,
    );
    option.value = choice.name;
    option.selected = choice.name === build.shown;
    select.append(option);
  }
  select.addEventListener("change", () =>
    channel.post({ type: "open", seq: channel.seq, key: `${CHOOSE_BUILD}${select.value}` }),
  );
  return select;
}

function head(answer: ProjectAnswer, channel: Channel): HTMLElement {
  const made = element("header", "head");
  if (!answer.head) return made;
  const { name, folder, state, version, server, settings, build } = answer.head;
  const top = element("div", "project-top");
  const title = element("h2", "project-name", name);
  title.title = folder;
  const buttons = element("span", "head-buttons");
  for (const item of settings) buttons.append(iconButton(item, channel));
  top.append(title, buttons);
  const facts = element("dl");
  const fact = (label: string, value: HTMLElement, after: HTMLElement[] = [], id?: string) => {
    const row = element("div", "row");
    const term = element("dt");
    if (id) {
      const named = element("label", "label", label);
      named.htmlFor = id;
      term.append(named);
    } else {
      term.append(element("span", "label", label));
    }
    const details = element("dd");
    value.classList.add("value");
    details.append(value, ...after);
    row.append(term, details);
    facts.append(row);
  };
  fact("Folder", element("span", undefined, folder));
  fact(
    "Server",
    element("span", undefined, state),
    server.map((item) => iconButton(item, channel)),
  );
  if (version !== null) fact("Version", element("span", undefined, version));
  if (build) fact("Build", buildSwitch(build, channel), [], "build-switch");
  made.append(top, facts);
  return made;
}

/** The actions, each group under its heading; an action that's off is drawn disabled. */
function actionList(groups: readonly ProjectActionGroup[], channel: Channel): HTMLElement {
  const body = element("div", "section-body");
  for (const group of groups) {
    const made = element("div", "group");
    made.append(element("h3", "group-head", group.label));
    const list = element("ul");
    for (const action of group.actions) {
      const item = element("li", "item");
      const button = chooser(action.key, channel);
      button.append(icon(action.icon), element("span", "label", action.label));
      button.title = action.caption;
      button.disabled = action.off;
      item.append(button);
      list.append(item);
    }
    made.append(list);
    body.append(made);
  }
  return body;
}

/** The action at a group's head, or beside a count's own note: what brings it current. */
function check(item: ProjectCheck, channel: Channel): HTMLButtonElement {
  const action = chooser(item.key, channel);
  action.classList.add("action");
  action.append(icon("play"), element("span", "label", item.label));
  return action;
}

/** Out of date is a warning, in words as well as its icon. */
function said(note: string, warns: boolean): HTMLElement {
  const made = element("span", "note");
  if (warns) made.append(icon("warning"));
  made.append(element("span", undefined, note));
  return made;
}

/**
 * A count, on one line: its label, then its value. A count whose own note
 * differs from its group's says it under them, with its action.
 */
function count(item: ProjectCount, group: ProjectHealthGroup, channel: Channel): HTMLElement {
  const made = element("div", `count ${item.state}`);
  made.dataset["key"] = item.key;
  const term = element("dt");
  term.append(element("span", "label", item.label));
  const details = element("dd", "count-value");
  // What the count's from, as its group's head says it.
  const when = item.note ?? [group.label, group.note].filter(Boolean).join(", ");
  let value: HTMLElement;
  if (item.opens) {
    value = chooser(item.key, channel);
    value.textContent = item.value;
    value.setAttribute("aria-label", `${item.label}: ${item.value}. ${when}`);
  } else {
    value = element("span", undefined, item.value);
    // A dash is read as why there's no count: its own note, or its group's.
    if (item.value === "—") {
      value.setAttribute("role", "img");
      value.setAttribute("aria-label", item.note ?? group.note);
    }
  }
  value.classList.add("value");
  details.append(value);
  made.append(term, details);
  if (item.note !== null) {
    const own = element("dd", "count-note");
    own.append(said(item.note, item.state === "outOfDate"));
    if (item.action) own.append(check(item.action, channel));
    made.append(own);
  }
  return made;
}

/** Health: each group under its head, which says when its counts are from, with its action while they aren't current. */
function healthList(groups: readonly ProjectHealthGroup[], channel: Channel): HTMLElement {
  const body = element("div", "section-body");
  for (const group of groups) {
    const made = element("div", `group health-group ${group.state}`);
    made.dataset["key"] = group.key;
    const top = element("div", "group-top");
    const heading = element("h3", "group-head");
    heading.append(element("span", "label", group.label));
    if (group.note) {
      const separator = element("span", "separator", "·");
      separator.setAttribute("aria-hidden", "true");
      heading.append(separator, said(group.note, group.state === "outOfDate"));
    }
    top.append(heading);
    if (group.action) top.append(check(group.action, channel));
    const counts = element("dl");
    for (const item of group.counts) counts.append(count(item, group, channel));
    made.append(top, counts);
    body.append(made);
  }
  return body;
}

/** The buttons' labels: their text, or an icon's label. */
function labels(buttons: Iterable<HTMLButtonElement>): string[] {
  return [...buttons].map(
    (button) => button.textContent || button.getAttribute("aria-label") || "",
  );
}

/** What the sections drew, for the report. */
function drawnSections(sections: readonly HTMLDetailsElement[]): ProjectReport["sections"] {
  return sections.map((details) => ({
    key: details.dataset["key"] ?? "",
    label: details.querySelector("summary")?.textContent ?? "",
    open: details.open,
  }));
}

function draw(answer: ProjectAnswer, channel: Channel): ProjectReport {
  if (answer.message !== null) {
    const said: HTMLElement[] = [element("p", "message", answer.message)];
    for (const action of answer.notice) {
      const button = chooser(action.key, channel);
      button.className = "notice-action";
      button.textContent = action.label;
      said.push(button);
    }
    // In Restricted Mode the actions are drawn off, so the reader sees what trusting the folder turns on.
    const sections =
      answer.actions.length === 0
        ? []
        : [section("actions", "Actions", "", actionList(answer.actions, channel), channel)];
    view.replaceChildren(...said, ...sections);
    const buttons = [...view.querySelectorAll("button")];
    return {
      message: answer.message,
      head: [],
      build: null,
      sections: drawnSections(sections),
      groups: [],
      health: [],
      buttons: labels(buttons),
      off: labels(buttons.filter((button) => button.disabled)),
      about: [],
      violations: seenViolations(),
    };
  }
  const toDo = answer.toDo === 0 ? "" : `${answer.toDo} to do`;
  const aboutRows = element("dl");
  for (const row of answer.about) {
    const made = element("div", "row");
    const term = element("dt");
    term.append(element("span", "label", row.label));
    made.append(term, element("dd", "value", row.value));
    aboutRows.append(made);
  }
  const aboutBody = element("div", "section-body");
  aboutBody.append(aboutRows);
  const sections = [
    section("actions", "Actions", "", actionList(answer.actions, channel), channel),
    section("health", "Health", toDo, healthList(answer.health, channel), channel),
    section("about", "About", "", aboutBody, channel, true),
  ];
  const top = head(answer, channel);
  view.replaceChildren(top, ...sections);
  const select = top.querySelector("select");
  const buttons = [...view.querySelectorAll("button")];
  // A row as it reads: the switch's by the build it shows.
  const line = (row: Element) => {
    const value = row.querySelector(".value");
    const said = value instanceof HTMLSelectElement ? value.value : (value?.textContent ?? "");
    return `${row.querySelector("dt")?.textContent ?? ""}: ${said}`;
  };
  return {
    message: null,
    head: [...top.querySelectorAll("h2, .row")].map((part) =>
      part.tagName === "H2" ? (part.textContent ?? "") : line(part),
    ),
    build: select
      ? {
          shown: select.value,
          choices: [...select.options].map((option) => option.value),
        }
      : null,
    sections: drawnSections(sections),
    groups: answer.health.map((group) => {
      const made = view.querySelector(`.health-group[data-key="${group.key}"]`);
      return {
        key: group.key,
        head: made?.querySelector(".group-head")?.textContent ?? "",
        action: made?.querySelector(".group-top > .action")?.textContent ?? null,
      };
    }),
    health: answer.health
      .flatMap((group) => group.counts.map((item) => ({ item, group })))
      .map(({ item, group }) => {
        const row = view.querySelector(`.count[data-key="${item.key}"]`);
        const value = row?.querySelector(".value");
        const label = row?.querySelector("dt .label");
        return {
          key: item.key,
          value: value?.textContent ?? "",
          note:
            row?.querySelector(".count-note .note")?.textContent ??
            view.querySelector(`.health-group[data-key="${group.key}"] .group-head .note`)
              ?.textContent ??
            "",
          state: item.state,
          // Drawn in another color than its label's, as the stylesheet dims a zero.
          dimmed:
            value && label
              ? getComputedStyle(value).color !== getComputedStyle(label).color
              : false,
        };
      }),
    buttons: labels(buttons),
    off: labels(buttons.filter((button) => button.disabled)),
    about: [...aboutRows.querySelectorAll(".row")].map(line),
    violations: seenViolations(),
  };
}

startView<ProjectAnswer, ProjectReport>(draw);
