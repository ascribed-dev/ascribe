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
import type { SettingEntry, SettingsResult, SettingValue } from "../shapes.js";
import type {
  FromSettings,
  SettingsAnswer,
  SettingsReport,
  SettingView,
} from "../webview/settingsProtocol.js";
import type { FromView } from "../webview/viewProtocol.js";

/**
 * The settings the tab shows, by id, in order. The server answers with every
 * simple setting; the tab shows these two until it shows them all.
 */
export const SHOWN: readonly string[] = ["consumer.trailing-slash", "consumer.base-path"];

/** A setting's id: `table.key`. */
export function settingId(entry: Pick<SettingEntry, "table" | "key">): string {
  return entry.table ? `${entry.table}.${entry.key}` : entry.key;
}

/** A value as the tab writes it: a string in quotes, as TOML has it, and the rest as they are. */
function written(value: SettingValue): string {
  return typeof value === "string" ? JSON.stringify(value) : String(value);
}

function view(entry: SettingEntry): SettingView {
  const set = entry.set !== null;
  return {
    id: settingId(entry),
    name: entry.name,
    key: entry.table ? `[${entry.table}] ${entry.key}` : entry.key,
    kind: entry.kind,
    choices: entry.choices,
    value: entry.value ?? (set ? null : entry.default),
    defaultText: entry.default !== null ? written(entry.default) : (entry.defaultText ?? ""),
    where: entry.set ? `ascribe.toml:${entry.set.line}` : "Not set in ascribe.toml",
    set,
    editable: set && entry.value !== null,
    description: entry.description,
    problems: entry.problems,
  };
}

/** What the tab draws for the server's answer. */
export function settingsAnswer(result: SettingsResult): SettingsAnswer {
  if (result.unreadable !== null) {
    return {
      message: `${result.unreadable} Fix it in the file to see its settings.`,
      settings: [],
    };
  }
  const settings = SHOWN.flatMap((id) => {
    const entry = result.settings.find((s) => settingId(s) === id);
    return entry ? [view(entry)] : [];
  });
  return { message: null, settings };
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
    }),
  ),
  violations: listOf(text),
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
    case "openLine":
      return shape<{ seq: number; id: string }>({ seq: whole, id: text })(message)
        ? (message as FromSettings)
        : undefined;
    default:
      return undefined;
  }
}
