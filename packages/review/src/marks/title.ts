// A page's title as a heading or a list entry shows it.
import type { FormattedPiece } from "../shapes.js";

export type { FormattedPiece, PieceKind } from "../shapes.js";

/**
 * A page's title as nodes for a heading or a list entry: its formatted form,
 * each code span a `<code>`, when its field sets `inline = "code"`; else
 * `plain`. Tooltips and search keep the plain title.
 */
export function titleNodes(
  formatted: readonly FormattedPiece[] | null | undefined,
  plain: string,
  doc: Document = document,
): (Node | string)[] {
  if (!formatted || formatted.length === 0) return [plain];
  return formatted.map((piece) => {
    if (piece.type === "text") return piece.value;
    const code = doc.createElement("code");
    code.textContent = piece.value;
    return code;
  });
}
