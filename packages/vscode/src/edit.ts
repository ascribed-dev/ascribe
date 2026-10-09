// A server's edit (`ascribe/edit`'s `EditResult`, and the requests that answer
// the same way) turned into the editor's types. `ProjectServer.requestEdit`
// converts with its language client's converter; this module checks the
// answer's shape and routes its parts through that converter, so it can be
// tested without VS Code.

import type { LspPosition, LspRange, LspWorkspaceEdit } from "./shapes.js";

/** A server's edit, converted: the edit and the range to select, or why there is none. */
export type ServerEdit<Edit, Range> = { edit: Edit; select: Range | undefined } | { error: string };

/** What converts the protocol's types: the language client's `protocol2CodeConverter`. */
export interface EditConverter<Edit, Range> {
  asWorkspaceEdit(edit: LspWorkspaceEdit): Promise<Edit>;
  asRange(range: LspRange): Range;
}

/**
 * Converts a server's answer. An `error` answer passes through; an edit is
 * converted, with its `select` (a `null` one is `undefined`). Anything else
 * is a broken server, and throws.
 */
export async function convertEdit<Edit, Range>(
  value: unknown,
  converter: EditConverter<Edit, Range>,
): Promise<ServerEdit<Edit, Range>> {
  if (!isRecord(value)) throw new Error("the language server returned an invalid edit");
  if (typeof value.error === "string") return { error: value.error };
  const { edit, select } = value;
  if (!isWorkspaceEdit(edit) || !(select === null || select === undefined || isRange(select))) {
    throw new Error("the language server returned an invalid edit");
  }
  return {
    edit: await converter.asWorkspaceEdit(edit),
    select: select ? converter.asRange(select) : undefined,
  };
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

function isPosition(value: unknown): value is LspPosition {
  return (
    isRecord(value) &&
    Number.isInteger(value.line) &&
    (value.line as number) >= 0 &&
    Number.isInteger(value.character) &&
    (value.character as number) >= 0
  );
}

function isRange(value: unknown): value is LspRange {
  return isRecord(value) && isPosition(value.start) && isPosition(value.end);
}

function isWorkspaceEdit(value: unknown): value is LspWorkspaceEdit {
  if (!isRecord(value) || !isRecord(value.changes)) return false;
  return Object.values(value.changes).every(
    (edits) =>
      Array.isArray(edits) &&
      edits.every((e) => isRecord(e) && isRange(e.range) && typeof e.newText === "string"),
  );
}
