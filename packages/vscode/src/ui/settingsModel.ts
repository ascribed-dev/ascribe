// Project Settings' model, apart from VS Code so the unit tests can play it
// out: the answer the tab draws, made from the server's `ascribe/settings`;
// the rule that keeps the tab in step with `ascribe.toml`; and the messages
// its webview may send.
//
// The rule (decision #658): the tab draws only an answer read from the text
// the editor has. The server says which version of `ascribe.toml`'s open
// document it read, or that it read the disk; an answer for any other text
// is dropped and asked for again. A change the tab makes names the version it
// was shown from, and the server refuses it for any other; the tab applies
// the server's edit only if the document is still at that version. So a value typed
// in the file, an edit from the tab, and Undo all reach the tab the same way:
// the document changes, and the tab asks.

import {
  boolean,
  isObject,
  listOf,
  nullable,
  shape,
  text,
  whole,
  type Check,
} from "../preview/messages.js";
import type {
  CheckEntry,
  PartEntry,
  SettingEntry,
  SettingPlace,
  SettingsResult,
  SettingValue,
} from "../shapes.js";
import type {
  CheckView,
  FromSettings,
  PartView,
  SettingsAnswer,
  SettingsReport,
  SettingView,
} from "../webview/settingsProtocol.js";
import type { FromView } from "../webview/viewProtocol.js";

/** A setting's id: `table.key`. */
function settingId(entry: Pick<SettingEntry, "table" | "key">): string {
  return entry.table ? `${entry.table}.${entry.key}` : entry.key;
}

/** A value as the tab writes it: a string in quotes, as TOML has it, and the rest as they are. */
function written(value: SettingValue): string {
  return typeof value === "string" ? JSON.stringify(value) : String(value);
}

function where(set: SettingPlace | null): string {
  return set ? `ascribe.toml:${set.line}` : "Not set in ascribe.toml";
}

function view(entry: SettingEntry): SettingView {
  const set = entry.set !== null;
  return {
    id: settingId(entry),
    section: entry.table ? `[${entry.table}]` : "The top level",
    name: entry.name,
    key: entry.table ? `[${entry.table}] ${entry.key}` : entry.key,
    kind: entry.kind,
    choices: entry.choices,
    value: entry.value ?? (set ? null : entry.default),
    defaultText: entry.default !== null ? written(entry.default) : (entry.defaultText ?? ""),
    where: where(entry.set),
    set,
    // A value the file leaves out is added; one it writes as something the
    // tab can't show, such as a list, is the file's.
    editable: !set || entry.value !== null,
    resettable: set,
    description: entry.description,
    problems: entry.problems,
  };
}

/** A check's level and limit, as settings of the table `checks.<check>`. */
function checkView(check: CheckEntry): CheckView {
  const set = check.set !== null;
  const level: SettingView = {
    id: `checks.${check.slug}.level`,
    section: check.area,
    name: check.name,
    key: `[checks] ${check.slug}`,
    kind: "level",
    choices: check.choices,
    value: check.level ?? (set ? null : check.default),
    defaultText: check.default,
    where: where(check.set),
    set,
    editable: !set || check.level !== null,
    resettable: set,
    description: check.when,
    problems: check.problems,
  };
  const limit = check.limit;
  return {
    slug: check.slug,
    code: check.code,
    named: check.named,
    level,
    limit: limit && {
      id: `checks.${check.slug}.limit`,
      section: check.area,
      name: `${check.name} limit`,
      key: `[checks.${check.slug}] limit`,
      kind: limit.kind,
      choices: [],
      value:
        limit.value ??
        (limit.set ? null : limit.kind === "size" ? limit.defaultText : limit.default),
      defaultText: limit.defaultText,
      where: where(limit.set),
      set: limit.set !== null,
      editable: limit.set === null || limit.value !== null,
      resettable: limit.set !== null,
      description: [],
      problems: limit.problems,
    },
  };
}

/** The heading of each table whose parts are cards, by the table. */
const PART_SECTIONS: Readonly<Record<string, string>> = {
  types: "Content types",
  dimensions: "Dimensions",
  releases: "Releases",
  builds: "Builds",
  sources: "Sources",
};

/** A part's id: `part:<table>.<name>`, apart from every setting's. */
function partId(part: Pick<PartEntry, "table" | "name">): string {
  return `part:${part.table}.${part.name}`;
}

function partView(part: PartEntry): PartView {
  return {
    id: partId(part),
    section: PART_SECTIONS[part.table] ?? `[${part.table}]`,
    name: part.name,
    key: `[${part.table}.${part.name}]`,
    summary: part.keys.length > 0 ? `Sets ${part.keys.join(", ")}` : "Sets nothing yet",
    where: where(part.set),
  };
}

/** What the tab draws in place of the settings: a message, and whether it's because the file is gone. */
export function settingsMessage(message: string, gone = false): SettingsAnswer {
  return { message, gone, settings: [], checks: [], parts: [] };
}

/** What the tab draws for the server's answer. */
export function settingsAnswer(result: SettingsResult): SettingsAnswer {
  if (result.unreadable !== null) {
    return settingsMessage(`${result.unreadable} Fix it in the file to see its settings.`);
  }
  return {
    message: null,
    gone: false,
    settings: result.settings.map(view),
    checks: result.checks.map(checkView),
    parts: result.parts.map(partView),
  };
}

/** What the tab says once its `ascribe.toml` is deleted or renamed. */
export const GONE =
  "ascribe.toml was deleted or renamed, so this project is gone. Click anywhere to close this tab.";

/** Every setting an answer draws a control for: the settings, then each check's level and limit. */
export function allSettings(answer: SettingsAnswer): SettingView[] {
  return [
    ...answer.settings,
    ...answer.checks.flatMap((c) => (c.limit ? [c.level, c.limit] : [c.level])),
  ];
}

/**
 * Where the server finds the setting `id` of its answer: the table and key
 * `setSetting` names, and the place the file sets it, if it does. A part's
 * id names its table and its name, which only a link opens: `setSetting`
 * changes no part, and the tab sends none.
 */
export function settingTarget(
  result: SettingsResult,
  id: string,
): { table: string; key: string; set: SettingPlace | null } | undefined {
  const part = result.parts.find((p) => partId(p) === id);
  if (part) return { table: part.table, key: part.name, set: part.set };
  const entry = result.settings.find((s) => settingId(s) === id);
  if (entry) return { table: entry.table, key: entry.key, set: entry.set };
  for (const check of result.checks) {
    const table = `checks.${check.slug}`;
    if (id === `${table}.level`) return { table, key: "level", set: check.set };
    if (id === `${table}.limit` && check.limit)
      return { table, key: "limit", set: check.limit.set };
  }
  return undefined;
}

/**
 * Whether an answer read from `answered` (a version of the open document, or
 * `null` for the disk) is for the text the editor has: `open` is the open
 * document's version, or `undefined` when it isn't open.
 */
export function inStep(answered: number | null, open: number | undefined): boolean {
  return open === undefined ? answered === null : answered === open;
}

/** How a change from the tab ended. */
export type SetOutcome =
  /** The edit was applied, or VS Code refused it (`false`). */
  | { type: "applied"; ok: boolean }
  /** The server refused it, in its words. */
  | { type: "refused"; error: string }
  /** The file changed while the server planned it, so it wasn't applied. */
  | { type: "stale" };

/**
 * Asks for a change shown from `shown`, and applies it only when the open
 * document is still the text it was planned for: checked again when the
 * answer comes back, in the same tick as `apply`, since typing in the file
 * while the server planned it would leave the edit's ranges on other text.
 * VS Code records the document's version when `apply` is called, so nothing
 * comes between the check and the edit.
 */
export async function setInStep<Edit>(
  shown: number | null,
  open: () => number | undefined,
  request: () => Promise<{ edit: Edit } | { error: string }>,
  apply: (edit: Edit) => PromiseLike<boolean>,
): Promise<SetOutcome> {
  const answer = await request();
  if ("error" in answer) return { type: "refused", error: answer.error };
  if (!inStep(shown, open())) return { type: "stale" };
  return { type: "applied", ok: await apply(answer.edit) };
}

/** The value a `set` carries, when it's one the setting takes; `undefined` otherwise. */
export function checkedValue(setting: SettingView, value: unknown): SettingValue | undefined {
  switch (setting.kind) {
    case "switch":
      return typeof value === "boolean" ? value : undefined;
    case "number":
      return Number.isSafeInteger(value) ? (value as number) : undefined;
    case "choice":
    case "level":
      return typeof value === "string" && setting.choices.includes(value) ? value : undefined;
    case "text":
      return typeof value === "string" && !/[\r\n]/.test(value) ? value : undefined;
    case "size":
      return Number.isSafeInteger(value) ||
        (typeof value === "string" && value.trim() !== "" && !/[\r\n]/.test(value))
        ? (value as number | string)
        : undefined;
    default:
      return undefined;
  }
}

const settingsReport: Check<SettingsReport> = shape<SettingsReport>({
  message: nullable(text),
  settings: listOf(
    shape<SettingsReport["settings"][number]>({
      id: text,
      value: text,
      where: text,
      problems: listOf(text),
      editable: boolean,
      resettable: boolean,
    }),
  ),
  violations: listOf(text),
  parts: listOf(shape<SettingsReport["parts"][number]>({ id: text, where: text })),
});

const value = (v: unknown): v is string | number | boolean =>
  text(v) || typeof v === "boolean" || Number.isFinite(v);

/** A message from the webview, when it's one the tab takes; `undefined` otherwise. */
export function checkSettingsMessage(
  message: unknown,
): FromView<SettingsReport> | FromSettings | undefined {
  if (!isObject(message)) return undefined;
  switch (message["type"]) {
    case "ready":
    case "focusEditor":
    case "editorSettings":
    case "contentModel":
    case "close":
      return message as FromView<SettingsReport> | FromSettings;
    case "drawn":
      return shape<{ seq: number; report: SettingsReport }>({ seq: whole, report: settingsReport })(
        message,
      )
        ? (message as FromView<SettingsReport>)
        : undefined;
    case "set":
      return shape<{ seq: number; id: string; value: string | number | boolean }>({
        seq: whole,
        id: text,
        value,
      })(message)
        ? (message as FromSettings)
        : undefined;
    case "reset":
    case "openLine":
      return shape<{ seq: number; id: string }>({ seq: whole, id: text })(message)
        ? (message as FromSettings)
        : undefined;
    default:
      return undefined;
  }
}
