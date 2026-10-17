// Runs an action from its command: the context at the selection, the
// wizard, then the action's own function or the server's edit, applied as
// one undo step. A rename that changes files other than the page is shown
// in VS Code's refactor preview before anything is written.

import * as vscode from "vscode";
import type { ProjectServer } from "../client.js";
import type { ProjectRegistry } from "../registry.js";
import type { TargetsResult } from "../shapes.js";
import { isCancellation, type ContextCache, type Where } from "./context.js";
import type { Action, Effects } from "./registry.js";
import { runWizard, type Args, type Prompter } from "./steps.js";

/** What running an action did, for tests. */
export interface RunRecord {
  action: string;
  /** How many `ascribe/edit` requests it sent: 2 when the first found the page changed. */
  edits: number;
  /** Whether it changed the page or ran its function. */
  done: boolean;
  /** What it told the writer, if anything. */
  messages: string[];
  /** For a rename: whether it asked first, in the refactor preview, since it changes other files. */
  previewed?: boolean;
  /** For a rename: the files it changes, relative to the workspace, sorted. */
  files?: string[];
  /** For an edit that makes a file: the file, relative to the workspace. */
  file?: string;
}

/** What a run needs from the extension. */
export interface Runner {
  projects: ProjectRegistry;
  cache: ContextCache;
  /** Shows a wizard's steps: the quick input, or a test's script. */
  prompter(): Prompter;
}

/** The document and selection of an editor, as the context cache keys them. */
export function whereOf(editor: vscode.TextEditor): Where {
  const { start, end } = editor.selection;
  return {
    uri: editor.document.uri.toString(),
    version: editor.document.version,
    range: {
      start: { line: start.line, character: start.character },
      end: { line: end.line, character: end.character },
    },
  };
}

export async function runAction(action: Action, runner: Runner): Promise<RunRecord> {
  const record: RunRecord = { action: action.id, edits: 0, done: false, messages: [] };
  const say = (message: string, warn = false): void => {
    record.messages.push(message);
    const text = `Ascribe: ${message}`;
    void (warn
      ? vscode.window.showWarningMessage(text)
      : vscode.window.showInformationMessage(text));
  };

  const editor = vscode.window.activeTextEditor;
  const document = editor?.document;
  const server = document && (await runner.projects.ensureStartedFor(document.uri));
  if (!editor || !document || document.languageId !== "markdown" || !server) {
    say(`Open a page of an Ascribe project to use "${action.title}".`);
    return record;
  }
  if (server.state !== "running") {
    say("The project's language server isn't running. Restart it, then try again.", true);
    return record;
  }

  try {
    let where = whereOf(editor);
    let context = await runner.cache.request(where);
    if (!action.applies(context)) {
      say(action.hint);
      return record;
    }
    const targets = action.needs?.length
      ? ((await server.request("ascribe/targets", {
          textDocument: { uri: where.uri },
          kinds: action.needs,
          range: where.range,
        })) as TargetsResult)
      : {};
    let args: Args | undefined;
    if (action.ask) {
      const asked = await runWizard(action.title, action.ask(context, targets), runner.prompter());
      if (!asked) return record;
      if ("error" in asked) {
        say(asked.error);
        return record;
      }
      args = asked.args;
    }

    if ("run" in action.does) {
      const effects: Effects = {
        copy: (text) => Promise.resolve(vscode.env.clipboard.writeText(text)),
        say: (message) => say(message),
        rename: (position, newName, uri) =>
          renameAt(
            server,
            { uri: uri ?? where.uri, position, newName, page: document.uri.toString() },
            record,
            say,
          ),
      };
      record.done = await action.does.run(context, targets, args, effects);
      return record;
    }

    for (let attempt = 0; ; attempt++) {
      const result = await server.requestEdit("ascribe/edit", {
        textDocument: { uri: where.uri },
        range: where.range,
        action: action.does.operation,
        args: args ?? {},
        version: where.version,
      });
      record.edits += 1;
      if ("error" in result) {
        // The page changed after the context was asked for (the server
        // refuses a stale version): ask again where the selection is now,
        // and try once more with the same answers.
        const now = vscode.window.activeTextEditor;
        if (attempt === 0 && document.version !== where.version && now?.document === document) {
          where = whereOf(now);
          context = await runner.cache.request(where);
          if (action.applies(context)) continue;
          say(action.hint);
          return record;
        }
        say(result.error, true);
        return record;
      }
      // As a refactoring, so `files.refactoring.autoSave` saves the other
      // files it changes, such as `ascribe.toml`.
      if (!(await vscode.workspace.applyEdit(result.edit, { isRefactoring: true }))) {
        say("The edit couldn't be applied: the page changed. Try again.", true);
        return record;
      }
      record.done = true;
      if (action.does.opens) {
        await openMade(result.edit, result.select, document.uri, record);
        return record;
      }
      const shown = vscode.window.activeTextEditor;
      if (result.select && shown?.document === document) {
        shown.selection = new vscode.Selection(result.select.start, result.select.end);
        shown.revealRange(result.select);
      }
      return record;
    }
  } catch (error) {
    if (!isCancellation(error)) server.reportFeatureError(`"${action.title}"`, error);
    return record;
  }
}

/**
 * Saves and shows the file an edit made, the first it changes other than the
 * page, with the edit's selection there: a memo, at its text.
 */
async function openMade(
  edit: vscode.WorkspaceEdit,
  select: vscode.Range | undefined,
  page: vscode.Uri,
  record: RunRecord,
): Promise<void> {
  const made = edit.entries().find(([uri]) => uri.toString() !== page.toString())?.[0];
  if (!made) return;
  const document = await vscode.workspace.openTextDocument(made);
  if (document.isDirty) await document.save();
  const editor = await vscode.window.showTextDocument(document);
  record.file = vscode.workspace.asRelativePath(made, false);
  if (select) {
    editor.selection = new vscode.Selection(select.start, select.end);
    editor.revealRange(select);
  }
}

/** What to rename: what's at `position` of `uri`, to `newName`. */
export interface RenameAt {
  uri: string;
  position: { line: number; character: number };
  newName: string;
  /** The page it's asked from, whose changes alone apply without the preview. */
  page?: string | undefined;
}

/**
 * Renames with the server's rename (`F2`'s), applied as a refactoring. When
 * it changes a file other than `page`, every change is shown in VS Code's
 * refactor preview first. Whether it was applied; what it previewed and
 * changed is written to `record`.
 */
export async function renameAt(
  server: ProjectServer,
  { uri, position, newName, page }: RenameAt,
  record: Pick<RunRecord, "previewed" | "files">,
  say: (message: string, warn?: boolean) => void,
): Promise<boolean> {
  const result = await server.requestEdit(
    "textDocument/rename",
    { textDocument: { uri }, position, newName },
    `Nothing renamed: there's nothing to rename there, or ${newName} breaks the content model's rules or is taken.`,
  );
  if ("error" in result) {
    say(result.error, true);
    return false;
  }
  // Other files change: show every change in the refactor preview first.
  const files = result.edit.entries().map(([file]) => file);
  const elsewhere = files.some((file) => file.toString() !== page);
  record.previewed = elsewhere;
  record.files = files.map((file) => vscode.workspace.asRelativePath(file, false)).sort();
  if (!elsewhere) return vscode.workspace.applyEdit(result.edit, { isRefactoring: true });
  const applied = await vscode.workspace.applyEdit(confirmEach(result.edit, newName), {
    isRefactoring: true,
  });
  if (!applied) say("Nothing renamed: check the changes to make in the preview, then Apply.");
  return applied;
}

/** An edit with every change marked as needing confirmation, so VS Code previews it. */
function confirmEach(edit: vscode.WorkspaceEdit, newName: string): vscode.WorkspaceEdit {
  const confirmed = new vscode.WorkspaceEdit();
  const metadata: vscode.WorkspaceEditEntryMetadata = {
    needsConfirmation: true,
    label: `Rename to ${newName}`,
  };
  for (const [uri, edits] of edit.entries()) {
    for (const change of edits) confirmed.replace(uri, change.range, change.newText, metadata);
  }
  return confirmed;
}
