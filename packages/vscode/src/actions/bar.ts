// What the actions bar lists for the cursor or selection: the quick fixes
// for problems there, then the registry's actions that apply, by group. No
// VS Code in this module, so the list is unit-tested as is; the controller
// shows it in a quick pick and runs what's chosen.

import type { ContextResult } from "../shapes.js";
import { ACTIONS, plain, type Action } from "./registry.js";

/** A code action VS Code offers at the cursor, as the bar needs it. */
export interface Fix {
  title: string;
  /** Its kind, such as `quickfix` or `quickfix.ascribe`. */
  kind: string | undefined;
  /** The message of the problem it fixes, when it names one. */
  problem: string | undefined;
  /** Whether its provider marked it as not applicable. */
  disabled: boolean;
  /** Whether it edits the text, rather than only running a command. */
  edits: boolean;
}

/** One row of the bar. */
export type Entry<F extends Fix = Fix> =
  | { kind: "separator"; label: string }
  | { kind: "fix"; fix: F }
  | { kind: "action"; action: Action; preview: string | undefined }
  | { kind: "empty"; label: string };

/** The bar's groups, in its order, with their separators' labels. */
const GROUPS: [Action["group"], string][] = [
  ["fix", "Fix"],
  ["write", "Write"],
  ["structure", "Structure"],
  ["link", "Link"],
  ["media", "Media"],
  ["model", "Content model"],
];

/** What the bar says when nothing applies. */
export const NOTHING =
  "Nothing here: select some text, or put the cursor on a heading, note, link, or image";

/**
 * Whether the bar lists a quick fix: one that edits the text, so not one
 * that only runs a command, such as asking a chat to fix or explain the
 * problem. The lightbulb's quick fixes from the registry come back as
 * `quickfix.ascribe`: the bar lists those actions in their groups instead.
 */
const listed = (fix: Fix): boolean => fix.edits && !fix.disabled && fix.kind !== "quickfix.ascribe";

/**
 * The bar's rows: the fixes, then each group of the registry's actions that
 * apply, each group under a separator. A row saying what to try when there's
 * nothing.
 */
export function entries<F extends Fix>(
  context: ContextResult,
  fixes: F[],
  actions: Action[] = ACTIONS,
): Entry<F>[] {
  const out: Entry<F>[] = [];
  const applying = actions.filter((action) => action.applies(context));
  for (const [group, label] of GROUPS) {
    const rows: Entry<F>[] = [];
    if (group === "fix") {
      for (const fix of fixes) {
        if (listed(fix)) rows.push({ kind: "fix", fix });
      }
    }
    for (const action of applying) {
      if (action.group === group) {
        rows.push({ kind: "action", action, preview: action.preview?.(context) });
      }
    }
    if (rows.length > 0) out.push({ kind: "separator", label }, ...rows);
  }
  if (out.length === 0) out.push({ kind: "empty", label: NOTHING });
  return out;
}

/** A row's label, description, and detail line, as the quick pick shows them. */
export function shown(entry: Entry): { label: string; description?: string; detail?: string } {
  switch (entry.kind) {
    case "separator":
    case "empty":
      return { label: entry.label };
    case "fix":
      return {
        label: entry.fix.title,
        ...(entry.fix.problem === undefined ? {} : { detail: entry.fix.problem }),
      };
    case "action":
      return {
        label: entry.action.title,
        description: plain(entry.action.description),
        ...(entry.preview === undefined ? {} : { detail: entry.preview }),
      };
  }
}
