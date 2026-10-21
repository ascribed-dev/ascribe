// Project Settings' script, run in its webview panel. It draws the settings
// the extension sends (`../ui/settingsModel.ts` makes them from the server's
// `ascribe/settings`), and says which value the reader chose; the extension
// asks the server for the edit and applies it, and the next answer shows the
// file as it then is. The webview changes nothing itself: a control shows the
// value of the answer it drew until the next answer comes.
//
// Each setting is a section with its name as a heading, so a screen reader
// moves between them by heading, and its control is labelled by the name. A
// choice is a `<select>`, a switch a checkbox, and a number or a text an
// `<input>` that sends its value when it's changed: on Enter, or when the
// focus leaves it. The tab checks only what its controls can know (a number
// is a whole number); every rule of the content model is the server's, whose
// problems show under the setting.
//
// The text comes from the registry and from `ascribe.toml`, and is set as
// text, never as HTML.

import type {
  FromSettings,
  SettingsAnswer,
  SettingsReport,
  SettingView,
} from "./settingsProtocol.js";
import { seenViolations, startView, type ViewChannel } from "./view.js";

type Channel = ViewChannel<SettingsReport, FromSettings>;

const main = document.querySelector<HTMLElement>('[data-role="view"]');
if (!main) throw new Error("Project Settings' shell has no view");
const view: HTMLElement = main;

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
    if (current && !choices.includes(current)) choices.unshift(current);
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

function drawSetting(setting: SettingView, index: number, channel: Channel): HTMLElement {
  const section = element("section", setting.set ? "setting set" : "setting");
  section.dataset["id"] = setting.id;
  const labelId = `setting-${index}`;
  const head = element("div", "head");
  const name = element("h2", "name", setting.name);
  name.id = labelId;
  head.append(name, element("code", "key", setting.key));
  section.append(head);
  const description = element("p", "description");
  for (const piece of setting.description) {
    description.append(piece.code ? element("code", undefined, piece.text) : piece.text);
  }
  section.append(description, control(setting, channel, labelId));
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
  section.append(meta);
  if (setting.problems.length > 0) {
    const problems = element("ul", "problems");
    problems.setAttribute("aria-label", `Problems with ${setting.name}`);
    for (const problem of setting.problems) {
      problems.append(element("li", problem.severity, problem.message));
    }
    section.append(problems);
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

let scrolled = false;

function draw(answer: SettingsAnswer, channel: Channel): SettingsReport {
  const keep = focused();
  const before = new Map(
    [...view.querySelectorAll<HTMLInputElement>("input.control[type=text]")].map((input) => [
      input.dataset["id"] ?? "",
      input.defaultValue,
    ]),
  );
  view.replaceChildren();
  const intro = element(
    "p",
    "intro",
    "These settings are shared with everyone on the project, and written to ascribe.toml. ",
  );
  intro.append(
    linkButton("Your editor's settings", () => channel.post({ type: "editorSettings" })),
    " are VS Code's.",
  );
  view.append(intro);
  if (answer.message !== null) {
    view.append(element("p", "message", answer.message));
  } else {
    answer.settings.forEach((setting, index) => view.append(drawSetting(setting, index, channel)));
  }
  if (keep) {
    const again = view.querySelector<HTMLElement>(`.control[data-id="${CSS.escape(keep.id)}"]`);
    if (again instanceof HTMLInputElement && keep.typed !== undefined) {
      // What's being typed stays, unless the file's value changed under it.
      if (before.get(keep.id) === again.value) again.value = keep.typed;
    }
    again?.focus();
  }
  if (!scrolled) {
    scrolled = true;
    const scroll = channel.state()["scroll"];
    if (typeof scroll === "number") window.scrollTo(0, scroll);
  }
  return {
    message: answer.message,
    settings: answer.settings.map((setting) => {
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
      };
    }),
    violations: seenViolations(),
  };
}

const channel = startView<SettingsAnswer, SettingsReport, FromSettings>(draw);
window.addEventListener(
  "scroll",
  () => channel.setState({ ...channel.state(), scroll: window.scrollY }),
  { passive: true },
);
