// Shows the actions bar: a quick pick of what `bar.ts` lists for the active
// editor's cursor or selection. It shows at once, busy, and fills when the
// context and the quick fixes there are in. A chosen action runs through the
// runner, and its wizard's first step takes the bar's place in the same quick
// input, so the bar doesn't close and reopen in between.

import * as vscode from "vscode";
import type { ContextResult } from "../shapes.js";
import { entries, shown, type Entry, type Fix } from "./bar.js";
import type { Action } from "./registry.js";
import type { Prompter } from "./steps.js";

/** What an opening of the bar did, for tests. */
export interface BarRecord {
  /** The rows it listed: a separator as `-- Fix`, any other row as its label. */
  rows: string[];
  /** The label of the row chosen, if any. */
  chosen: string | undefined;
  /** Whether a wizard's first step took the bar's place while the bar was open. */
  handedOver: boolean;
}

/** What the bar needs from the extension. */
export interface BarHost {
  /** The context at the editor's selection; rejects when the project's server can't answer. */
  context(editor: vscode.TextEditor): Promise<ContextResult>;
  /** Runs an action, its wizard shown by the prompter `wrap` makes of the usual one. */
  run(action: Action, wrap: (prompter: Prompter) => Prompter): Promise<void>;
  /** Opens the output of the project that owns a file. */
  showOutput(uri: vscode.Uri): void;
}

/** A code action at the cursor, with what the bar shows of it. */
interface CodeFix extends Fix {
  action: vscode.CodeAction | vscode.Command;
  severity: vscode.DiagnosticSeverity | undefined;
}

interface Row extends vscode.QuickPickItem {
  entry?: Entry<CodeFix> | { kind: "failed" };
}

/**
 * How many of the code actions VS Code resolves before returning them, for
 * a provider that fills in a fix's edit only when asked: more than a cursor
 * ever has.
 */
const RESOLVE = 50;

const SEVERITY_ICON: Record<vscode.DiagnosticSeverity, string> = {
  [vscode.DiagnosticSeverity.Error]: "error",
  [vscode.DiagnosticSeverity.Warning]: "warning",
  [vscode.DiagnosticSeverity.Information]: "info",
  [vscode.DiagnosticSeverity.Hint]: "info",
};

const isCommand = (action: vscode.CodeAction | vscode.Command): action is vscode.Command =>
  typeof action.command === "string";

/** The quick fixes every provider offers for a range of a document. */
async function quickFixes(uri: vscode.Uri, range: vscode.Range): Promise<CodeFix[]> {
  let found: (vscode.CodeAction | vscode.Command)[] | undefined;
  try {
    found = await vscode.commands.executeCommand<(vscode.CodeAction | vscode.Command)[]>(
      "vscode.executeCodeActionProvider",
      uri,
      range,
      vscode.CodeActionKind.QuickFix.value,
      RESOLVE,
    );
  } catch {
    return [];
  }
  return (found ?? []).map((action) => {
    if (isCommand(action)) {
      return {
        action,
        title: action.title,
        kind: undefined,
        problem: undefined,
        disabled: false,
        severity: undefined,
      };
    }
    const diagnostic = action.diagnostics?.[0];
    return {
      action,
      title: action.title,
      kind: action.kind?.value,
      problem: diagnostic?.message,
      disabled: action.disabled !== undefined,
      severity: diagnostic?.severity,
    };
  });
}

/** Applies a code action as the lightbulb would: its edit, then its command. */
async function applyFix(action: vscode.CodeAction | vscode.Command): Promise<void> {
  const command = isCommand(action) ? action : action.command;
  if (!isCommand(action) && action.edit && !(await vscode.workspace.applyEdit(action.edit))) {
    return;
  }
  if (command) await vscode.commands.executeCommand(command.command, ...(command.arguments ?? []));
}

function row(entry: Entry<CodeFix>): Row {
  if (entry.kind === "separator") {
    return { label: entry.label, kind: vscode.QuickPickItemKind.Separator };
  }
  const item: Row = { ...shown(entry), entry };
  if (entry.kind === "fix") {
    const severity = entry.fix.severity;
    if (severity !== undefined) item.iconPath = new vscode.ThemeIcon(SEVERITY_ICON[severity]);
  }
  if (entry.kind === "empty") item.iconPath = new vscode.ThemeIcon("info");
  return item;
}

const FAILED: Row = {
  label: "The project's language server isn't answering",
  detail: "Open its output to see why",
  iconPath: new vscode.ThemeIcon("warning"),
  entry: { kind: "failed" },
};

export class ActionsBar {
  /** The bar that's open, if one is. */
  private open: vscode.QuickPick<Row> | undefined;
  readonly records: BarRecord[] = [];

  constructor(private readonly host: BarHost) {}

  /** Opens the bar for the editor's cursor or selection. */
  async show(editor: vscode.TextEditor): Promise<void> {
    const document = editor.document;
    const range = editor.selection;
    const record: BarRecord = { rows: [], chosen: undefined, handedOver: false };
    this.records.push(record);

    const pick = vscode.window.createQuickPick<Row>();
    pick.placeholder = "What would you like to do here?";
    pick.matchOnDescription = true;
    pick.busy = true;
    // Hidden by the writer, by another quick input, or by the bar itself.
    let closed = false;
    pick.onDidHide(() => {
      closed = true;
      if (this.open === pick) this.open = undefined;
      pick.dispose();
    });

    /** The wizard's first step, shown in the bar's place: nothing if the writer closed the bar. */
    const wrap = (inner: Prompter): Prompter => ({
      show: (step, position) => {
        if (record.handedOver) return inner.show(step, position);
        if (closed) return Promise.resolve(undefined);
        record.handedOver = true;
        const shownStep = inner.show(step, position);
        pick.dispose();
        return shownStep;
      },
    });

    pick.onDidAccept(() => {
      const chosen = pick.activeItems[0];
      const entry = chosen?.entry;
      if (!chosen || !entry || entry.kind === "empty" || record.chosen !== undefined) return;
      record.chosen = chosen.label;
      if (entry.kind === "failed") {
        pick.hide();
        this.host.showOutput(document.uri);
      } else if (entry.kind === "fix") {
        pick.hide();
        void applyFix(entry.fix.action);
      } else if (entry.kind === "action") {
        // Busy until the wizard takes the bar's place, or the action is done.
        pick.busy = true;
        pick.enabled = false;
        void this.host.run(entry.action, wrap).finally(() => pick.dispose());
      }
    });

    this.open = pick;
    pick.show();

    const [context, fixes] = await Promise.all([
      this.host.context(editor).then(
        (answer) => answer,
        () => undefined,
      ),
      quickFixes(document.uri, range),
    ]);
    if (closed) return;
    const rows = context ? entries(context, fixes).map(row) : [FAILED];
    record.rows = rows.map((r) =>
      r.kind === vscode.QuickPickItemKind.Separator ? `-- ${r.label}` : r.label,
    );
    pick.items = rows;
    pick.busy = false;
  }

  /** For tests: makes the open bar's row with this label the active one, as the arrow keys would. */
  select(label: string): boolean {
    const found = this.open?.items.find(
      (item) => item.label === label && item.kind !== vscode.QuickPickItemKind.Separator,
    );
    if (!this.open || !found) return false;
    this.open.activeItems = [found];
    return true;
  }
}
