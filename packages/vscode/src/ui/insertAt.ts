// Content model's Insert at the Cursor: which action a phrase or a widget
// runs, with the entry chosen, and the project's last Markdown editor it
// writes into. VS Code's active editor is none while a webview or a view has
// focus, so the extension keeps the last one itself (decision 17). No VS Code
// in this module.

import { ACTIONS, type Action } from "../actions/registry.js";
import type { Answers } from "../actions/steps.js";
import { comparable } from "../projects.js";
import type { ContextResult, InventoryEntry, LspRange } from "../shapes.js";

/** Where the cursor was in a Markdown editor of a project. */
export interface LastEditor {
  /** The document's URI. */
  uri: string;
  /** Its selection, as it was last. */
  selection: LspRange;
}

/** The last Markdown editor of each project, by project folder. */
export class LastEditors {
  private readonly editors = new Map<string, LastEditor>();

  /** Notes an editor of the project in `folder` as its last, with its selection now. */
  note(folder: string, editor: LastEditor): void {
    this.editors.set(comparable(folder), editor);
  }

  /** The project's last Markdown editor, if one has been active. */
  get(folder: string): LastEditor | undefined {
    return this.editors.get(comparable(folder));
  }

  /** Forgets a document's editor, once it's closed, in whichever project it was. */
  forget(uri: string): void {
    for (const [folder, editor] of this.editors) {
      if (editor.uri === uri) this.editors.delete(folder);
    }
  }
}

/** The action Insert at the Cursor runs for an entry, with the entry chosen. */
export function insertAction(
  entry: InventoryEntry,
): { action: Action; preset: Answers } | undefined {
  const id =
    entry.kind === "phrase" ? "insertPhrase" : entry.kind === "widget" ? "insertWidget" : undefined;
  const action = ACTIONS.find((a) => a.id === id);
  if (!action) return undefined;
  return { action, preset: entry.kind === "phrase" ? { key: entry.key } : { name: entry.key } };
}

/** What Insert at the Cursor does: run the action there, or show its hint and change nothing. */
export type InsertPlan =
  { kind: "run"; action: Action; preset: Answers } | { kind: "hint"; hint: string };

/**
 * What Insert at the Cursor does for `entry`, given whether the project has
 * a last Markdown editor and the context at its cursor: the action runs
 * there only where it applies, as it would from the actions bar.
 */
export function insertPlan(
  entry: InventoryEntry,
  editor: LastEditor | undefined,
  context: ContextResult | undefined,
): InsertPlan | undefined {
  const insert = insertAction(entry);
  if (!insert) return undefined;
  if (!editor || !context || !insert.action.applies(context)) {
    return { kind: "hint", hint: insert.action.hint };
  }
  return { kind: "run", ...insert };
}
