// Project Settings' script, run in its webview panel. It draws the settings
// the extension sends (`../ui/settingsModel.ts` makes them from the server's
// `ascribe/settings`), and says which value the reader chose; the extension
// asks the server for the edit and applies it, and the next answer shows the
// file as it then is. The webview changes nothing itself: a control shows the
// value of the answer it drew until the next answer comes.
//
// The settings are under a heading for each table, and the checks under one
// of their own. Each setting, and each check, is a section with its name as a
// heading, so a screen reader moves between them by heading, and its control
// is labelled by the name. A choice or a level is a `<select>`, a switch a
// checkbox, and a number, a size, or a text an `<input>` that sends its value
// when it's changed: on Enter, or when the focus leaves it. A setting the
// file leaves out shows its default, and changing it adds it to the file; one
// the file sets has Reset to default, which takes it out. A check shows what
// it reports, its level, and, for the three that take one, its limit. The tab
// checks only what its controls can know (a number is a whole number); every
// rule of the content model is the server's, whose problems show under the
// setting.
//
// Above them, a search over every setting's name, key, and whole
// description, and a filter, Set in ascribe.toml, for what the file sets;
// beside them, the sections, each a button that scrolls to its heading. The
// search and the filter hide what doesn't match, in the webview alone: they
// change nothing in the file, and outlast a redraw and the tab being hidden,
// as its scroll and its section do (`getState`). A part of the project with
// structure, a content type or a build, is a card whose link opens the file
// at its table, and phrases, features, and terms are the Content model
// view's. When `ascribe.toml` is gone, the tab says so, and the next click
// closes it.
//
// The text comes from the registry and from `ascribe.toml`, and is set as
// text, never as HTML.

import type {
  CheckView,
  FromSettings,
  PartView,
  SettingsAnswer,
  SettingsReport,
  SettingView,
} from "./settingsProtocol.js";
import { seenViolations, startView, type ViewChannel } from "./view.js";

type Channel = ViewChannel<SettingsReport, FromSettings>;

const main = document.querySelector<HTMLElement>('[data-role="view"]');
if (!main) throw new Error("Project Settings' shell has no view");

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

/** A button that reads as a link: the webview has no `<a>`, and opens nothing itself. */
function linkButton(text: string, onClick: () => void): HTMLButtonElement {
  const button = element("button", "link", text);
  button.type = "button";
  button.addEventListener("click", onClick);
  return button;
}

// The tab's layout, made once: the search and the filter stay as they are
// while each answer redraws the sections and the settings.
const toolbar = element("div", "toolbar");
const search = element("input", "search");
search.type = "search";
search.placeholder = "Search settings";
search.setAttribute("aria-label", "Search settings");
search.spellcheck = false;
const setOnly = element("input");
setOnly.type = "checkbox";
setOnly.className = "set-only";
const setOnlyLabel = element("label", "filter");
setOnlyLabel.append(setOnly, " Set in ascribe.toml");
toolbar.append(search, setOnlyLabel);
const intro = element("p", "intro");
const nav = element("nav", "sections");
nav.setAttribute("aria-label", "Sections");
const content = element("div", "content");
const layout = element("div", "layout");
layout.append(nav, content);
main.append(intro, toolbar, layout);
const view: HTMLElement = content;

/** The value a control shows, as text: what the report says it drew. */
function controlText(value: SettingView["value"]): string {
  return value === null ? "" : String(value);
}

/** A setting's control, which posts the value chosen. */
function control(setting: SettingView, channel: Channel, labelId: string): HTMLElement {
  const post = (value: string | number | boolean): void =>
    channel.post({ type: "set", seq: channel.seq, id: setting.id, value });
  const disabled = !setting.editable;
  let made: HTMLInputElement | HTMLSelectElement;
  if (setting.kind === "choice" || setting.kind === "level") {
    const select = element("select");
    const choices = [...setting.choices];
    const current = controlText(setting.value);
    // A value the list doesn't have, or none: shown as it is, never as
    // another.
    if (!choices.includes(current)) choices.unshift(current);
    for (const choice of choices) {
      const option = element("option", undefined, choice);
      option.value = choice;
      select.append(option);
    }
    select.value = current;
    select.addEventListener("change", () => post(select.value));
    made = select;
  } else if (setting.kind === "switch") {
    const box = element("input");
    box.type = "checkbox";
    box.checked = setting.value === true;
    box.addEventListener("change", () => post(box.checked));
    made = box;
  } else {
    const input = element("input");
    input.type = "text";
    if (setting.kind === "number") input.inputMode = "numeric";
    if (setting.kind === "size") input.placeholder = "500 KB";
    // The file's value, which a redraw compares to keep what's being typed.
    input.defaultValue = controlText(setting.value);
    input.value = input.defaultValue;
    input.spellcheck = false;
    // Enter and the change that follows it send a value once.
    let sent = input.defaultValue;
    const send = (): void => {
      if (input.value === sent) return;
      if (setting.kind === "number") {
        const number = Number(input.value.trim());
        if (input.value.trim() === "" || !Number.isSafeInteger(number)) {
          input.setCustomValidity("A whole number");
          input.reportValidity();
          return;
        }
        input.setCustomValidity("");
        post(number);
      } else if (setting.kind === "size") {
        const size = input.value.trim();
        if (size === "") {
          input.setCustomValidity("A number of bytes, or a size such as 500 KB");
          input.reportValidity();
          return;
        }
        input.setCustomValidity("");
        post(/^\d+$/.test(size) && Number.isSafeInteger(Number(size)) ? Number(size) : size);
      } else {
        post(input.value);
      }
      sent = input.value;
    };
    input.addEventListener("change", send);
    input.addEventListener("keydown", (event) => {
      if (event.key === "Enter") send();
    });
    made = input;
  }
  made.className = "control";
  made.dataset["id"] = setting.id;
  made.disabled = disabled;
  made.setAttribute("aria-labelledby", labelId);
  return made;
}

/** A setting's default, its place, and Reset to default, under its control. */
function meta(setting: SettingView, channel: Channel): HTMLElement {
  const meta = element("p", "meta");
  if (setting.defaultText)
    meta.append(element("span", "default", `Default: ${setting.defaultText}`));
  const where = setting.set
    ? linkButton(setting.where, () =>
        channel.post({ type: "openLine", seq: channel.seq, id: setting.id }),
      )
    : element("span", "where", setting.where);
  where.classList.add("where");
  meta.append(where);
  if (setting.resettable) {
    const reset = linkButton("Reset to default", () =>
      channel.post({ type: "reset", seq: channel.seq, id: setting.id }),
    );
    reset.classList.add("reset");
    reset.dataset["id"] = setting.id;
    reset.setAttribute("aria-label", `Reset ${setting.name} to default`);
    meta.append(reset);
  }
  return meta;
}

function problems(setting: SettingView): HTMLElement[] {
  if (setting.problems.length === 0) return [];
  const list = element("ul", "problems");
  list.setAttribute("aria-label", `Problems with ${setting.name}`);
  for (const problem of setting.problems) {
    list.append(element("li", problem.severity, problem.message));
  }
  return [list];
}

function description(pieces: SettingView["description"]): HTMLElement {
  const made = element("p", "description");
  for (const piece of pieces) {
    made.append(piece.code ? element("code", undefined, piece.text) : piece.text);
  }
  return made;
}

let labels = 0;

function drawSetting(setting: SettingView, channel: Channel): HTMLElement {
  const section = element("section", setting.set ? "setting set" : "setting");
  section.dataset["id"] = setting.id;
  const labelId = `setting-${labels++}`;
  const head = element("div", "head");
  const name = element("h3", "name", setting.name);
  name.id = labelId;
  head.append(name, element("code", "key", setting.key));
  section.append(
    head,
    description(setting.description),
    control(setting, channel, labelId),
    meta(setting, channel),
    ...problems(setting),
  );
  return section;
}

/** A check: what it reports, its level, and its limit if it takes one. */
function drawCheck(check: CheckView, channel: Channel): HTMLElement {
  const { level, limit } = check;
  const section = element("section", check.named ? "setting check set" : "setting check");
  section.dataset["id"] = level.id;
  const labelId = `setting-${labels++}`;
  const head = element("div", "head");
  const name = element("h3", "name", level.name);
  name.id = labelId;
  head.append(
    name,
    element("code", "key", level.key),
    element("span", "area", `${level.section} · ${check.code}`),
  );
  section.append(
    head,
    description(level.description),
    control(level, channel, labelId),
    meta(level, channel),
    ...problems(level),
  );
  if (limit) {
    const own = element("div", limit.set ? "limit set" : "limit");
    const limitId = `setting-${labels++}`;
    const label = element("span", "limit-name", "Limit");
    label.id = limitId;
    own.append(
      label,
      // Named by the check's name and "Limit", so it says whose limit it is.
      control(limit, channel, `${labelId} ${limitId}`),
      element("code", "key", limit.key),
      meta(limit, channel),
      ...problems(limit),
    );
    section.append(own);
  }
  return section;
}

/** The control with the focus, and what's typed in it, to keep across a redraw. */
function focused(): { id: string; typed: string | undefined } | undefined {
  const active = document.activeElement;
  if (!(active instanceof HTMLElement) || !active.dataset["id"]) return undefined;
  const typed =
    active instanceof HTMLInputElement && active.type === "text" ? active.value : undefined;
  return { id: active.dataset["id"], typed };
}

/** A part of the project with structure: a card whose link opens the file at its table. */
function drawPart(part: PartView, channel: Channel): HTMLElement {
  const section = element("section", "setting card set");
  section.dataset["id"] = part.id;
  const head = element("div", "head");
  head.append(element("h3", "name", part.name), element("code", "key", part.key));
  const meta = element("p", "meta");
  const edit = linkButton("Edit in ascribe.toml", () =>
    channel.post({ type: "openLine", seq: channel.seq, id: part.id }),
  );
  edit.classList.add("edit");
  edit.dataset["id"] = part.id;
  edit.setAttribute("aria-label", `Edit ${part.key} in ascribe.toml`);
  meta.append(element("span", "where", part.where), edit);
  section.append(head, element("p", "description", part.summary), meta);
  return section;
}

/** Phrases, features, and terms: the Content model view's, where they're added and renamed. */
function drawContentModel(channel: Channel): HTMLElement {
  const section = element("section", "setting card");
  section.dataset["id"] = "content-model";
  const head = element("div", "head");
  head.append(element("h3", "name", "Phrases, features, and terms"));
  const meta = element("p", "meta");
  meta.append(linkButton("Open Content model", () => channel.post({ type: "contentModel" })));
  section.append(
    head,
    element(
      "p",
      "description",
      "Phrases, features, and glossary terms are added, renamed, and found in the Content model view, with every page that uses them.",
    ),
    meta,
  );
  return section;
}

/** Something the search and the filter show or hide: a setting, a check, or a card. */
interface Item {
  element: HTMLElement;
  /** What the search looks in, in lowercase: its name, its key, and its whole description. */
  text: string;
  /** Whether the file sets it, which the filter keeps. */
  set: boolean;
}

/** A group of items under a heading, with its button in the list of sections. */
interface Group {
  title: string;
  element: HTMLElement;
  button: HTMLButtonElement;
  items: Item[];
}

let groups: Group[] = [];

const words = (setting: SettingView, more: string[] = []): string =>
  [setting.name, setting.key, ...setting.description.map((p) => p.text), ...more]
    .join(" ")
    .toLowerCase();

/** Whether `text` has every word of the search, in any of its sentences. */
function matches(text: string, query: string): boolean {
  return query
    .toLowerCase()
    .split(/\s+/)
    .filter((word) => word !== "")
    .every((word) => text.includes(word));
}

const none = element("p", "message none");

/** Hides what the search and the filter leave out, and a section with nothing left. */
function filter(): void {
  const query = search.value;
  let shown = 0;
  for (const group of groups) {
    let left = 0;
    for (const item of group.items) {
      const show = matches(item.text, query) && (!setOnly.checked || item.set);
      item.element.hidden = !show;
      if (show) left++;
    }
    group.element.hidden = left === 0;
    group.button.hidden = left === 0;
    shown += left;
  }
  none.hidden = groups.length === 0 || shown > 0;
  none.textContent = setOnly.checked
    ? "Nothing ascribe.toml sets matches the search."
    : "No setting matches the search.";
}

let scrolled = false;
let gone = false;

/** Starts a group: its heading, and its button in the list of sections. */
function group(title: string, channel: Channel): Group {
  const made = element("div", "group");
  made.append(element("h2", "section", title));
  const button = element("button", "section-link", title);
  button.type = "button";
  button.addEventListener("click", () => {
    made.scrollIntoView({ block: "start" });
    channel.setState({ ...channel.state(), section: title });
  });
  nav.append(button);
  view.append(made);
  const drawn: Group = { title, element: made, button, items: [] };
  groups.push(drawn);
  return drawn;
}

function draw(answer: SettingsAnswer, channel: Channel): SettingsReport {
  const keep = focused();
  const before = new Map(
    [...view.querySelectorAll<HTMLInputElement>("input.control[type=text]")].map((input) => [
      input.dataset["id"] ?? "",
      input.defaultValue,
    ]),
  );
  view.replaceChildren();
  nav.replaceChildren();
  groups = [];
  gone = answer.gone;
  intro.replaceChildren(
    "These settings are shared with everyone on the project, and written to ascribe.toml. ",
    linkButton("Your editor's settings", () => channel.post({ type: "editorSettings" })),
    " are VS Code's.",
  );
  labels = 0;
  const drawing = answer.message === null;
  toolbar.hidden = !drawing;
  nav.hidden = !drawing;
  if (!drawing) {
    view.append(element("p", "message", answer.message ?? ""));
  } else {
    let current: Group | undefined;
    for (const setting of answer.settings) {
      if (setting.section !== current?.title) current = group(setting.section, channel);
      const made = drawSetting(setting, channel);
      current.element.append(made);
      current.items.push({ element: made, text: words(setting), set: setting.set });
    }
    let parts: Group | undefined;
    for (const part of answer.parts) {
      if (part.section !== parts?.title) parts = group(part.section, channel);
      const made = drawPart(part, channel);
      parts.element.append(made);
      parts.items.push({
        element: made,
        text: [part.name, part.key, part.summary].join(" ").toLowerCase(),
        set: true,
      });
    }
    const model = group("Phrases, features, and terms", channel);
    const card = drawContentModel(channel);
    model.element.append(card);
    model.items.push({ element: card, text: card.textContent?.toLowerCase() ?? "", set: false });
    if (answer.checks.length > 0) {
      const checks = group("Checks", channel);
      checks.element.append(
        element(
          "p",
          "intro",
          "How loudly each check about your content speaks, set in [checks]. The ones ascribe.toml names are first.",
        ),
      );
      for (const check of answer.checks) {
        const made = drawCheck(check, channel);
        checks.element.append(made);
        checks.items.push({
          element: made,
          text: words(check.level, [check.slug, check.code, check.level.section]),
          set: check.named,
        });
      }
    }
    view.append(none);
    filter();
  }
  if (keep) {
    const again = view.querySelector<HTMLElement>(`.control[data-id="${CSS.escape(keep.id)}"]`);
    if (again instanceof HTMLInputElement && keep.typed !== undefined) {
      // What's being typed stays, unless the file's value changed under it.
      if (before.get(keep.id) === again.value) again.value = keep.typed;
    }
    again?.focus();
  }
  if (!scrolled && drawing) {
    scrolled = true;
    const scroll = channel.state()["scroll"];
    if (typeof scroll === "number") window.scrollTo(0, scroll);
  }
  return {
    message: answer.message,
    settings: [
      ...answer.settings,
      ...answer.checks.flatMap((c) => (c.limit ? [c.level, c.limit] : [c.level])),
    ].map((setting) => {
      const shown = view.querySelector<HTMLInputElement | HTMLSelectElement>(
        `.control[data-id="${CSS.escape(setting.id)}"]`,
      );
      const value =
        shown instanceof HTMLInputElement && shown.type === "checkbox"
          ? String(shown.checked)
          : (shown?.value ?? "");
      return {
        id: setting.id,
        value,
        where: setting.where,
        problems: setting.problems.map((p) => p.message),
        editable: shown ? !shown.disabled : false,
        resettable: view.querySelector(`.reset[data-id="${CSS.escape(setting.id)}"]`) !== null,
      };
    }),
    parts: answer.parts.map((part) => ({ id: part.id, where: part.where })),
    violations: seenViolations(),
  };
}

const channel = startView<SettingsAnswer, SettingsReport, FromSettings>(draw);
// The search, the filter, and the scroll, kept for when the tab is shown again.
const kept = channel.state();
if (typeof kept["search"] === "string") search.value = kept["search"];
setOnly.checked = kept["setOnly"] === true;
const keepFilter = (): void => {
  channel.setState({ ...channel.state(), search: search.value, setOnly: setOnly.checked });
  filter();
};
search.addEventListener("input", keepFilter);
setOnly.addEventListener("change", keepFilter);
window.addEventListener(
  "scroll",
  () => channel.setState({ ...channel.state(), scroll: window.scrollY }),
  { passive: true },
);
// A tab whose `ascribe.toml` is gone closes on the next click.
document.addEventListener("click", () => {
  if (gone) channel.post({ type: "close" });
});

// DIAG #764: the webview's focus, which the extension prints.
const diag764 = (line: string): void =>
  (channel.post as (m: unknown) => void)({
    type: "diag764",
    line: `${line} active=${document.activeElement?.tagName ?? "-"} hasFocus=${document.hasFocus()}`,
  });
window.addEventListener("focus", () => diag764("window focus"));
window.addEventListener("blur", () => diag764("window blur"));
