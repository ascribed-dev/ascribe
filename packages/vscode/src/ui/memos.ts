// The command a lens on several memos carries (`ascribe.showMemos`, which
// the language server names in crates/ascribe-lsp/src/memos.rs): it lists
// the memos to pick one from, and opens the one picked at its text. A lens
// on one memo opens it through the server's own `ascribe.openFile`.

import * as vscode from "vscode";
import type { LspRange } from "../shapes.js";

/** A memo as the lens lists it. */
interface ListedMemo {
  /** The memo file's `file:` URI. */
  uri: string;
  /** Its path from the project root. */
  path: string;
  /** The first line of its text that isn't blank. */
  firstLine: string;
  /** Whether its quote is no longer found. */
  outdated: boolean;
  /** Whether it's kept on this machine only. */
  personal: boolean;
  /** Where its text starts. */
  body: LspRange;
}

/** How a memo is shown in the list: its first line, then its file and what it is. */
function memoItem(memo: ListedMemo): { label: string; description: string } {
  const notes = [memo.personal ? "personal" : "", memo.outdated ? "outdated" : ""].filter(
    (note) => note !== "",
  );
  return {
    label: memo.firstLine || memo.path,
    description: notes.length > 0 ? `${memo.path} · ${notes.join(", ")}` : memo.path,
  };
}

/** Lists the memos, and opens the one picked at its text. */
export async function showMemos(memos: unknown): Promise<void> {
  if (!Array.isArray(memos) || memos.length === 0) return;
  const listed = memos as ListedMemo[];
  const picked =
    listed.length === 1
      ? listed[0]
      : (
          await vscode.window.showQuickPick(
            listed.map((memo) => ({ ...memoItem(memo), memo })),
            { title: "Memos", placeHolder: "Open a memo" },
          )
        )?.memo;
  if (!picked) return;
  const { start } = picked.body;
  const at = new vscode.Position(start.line, start.character);
  await vscode.window.showTextDocument(vscode.Uri.parse(picked.uri), {
    selection: new vscode.Range(at, at),
  });
}
