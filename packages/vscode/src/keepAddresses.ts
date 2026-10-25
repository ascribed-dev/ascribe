// What **Keep the addresses you've published** offers, worked out from plain
// values so the unit tests can check it without VS Code: which of a move's
// files are pages, whose old addresses the offer records once the file is
// made. `ui/keepAddresses.ts` makes the offer.

import * as path from "node:path";

/** The offer's button, and what the docs call it. */
export const KEEP_ADDRESSES = "Keep the addresses you've published";

/** A file moved, from one absolute path to another. */
export interface Moved {
  from: string;
  to: string;
}

/**
 * The pages among `moved` that moved inside the content root, `contentDir`,
 * absolute, each from its old content path to its new one, with `/`. A
 * fragment, or a file in a hidden folder, has no address of its own, so it
 * isn't one, as the editor's move records none.
 */
export function pageMoves(moved: readonly Moved[], contentDir: string): Moved[] {
  const inContent = (file: string): string | undefined => {
    const relative = path.relative(contentDir, file);
    if (relative === "" || relative.startsWith("..") || path.isAbsolute(relative)) return undefined;
    const parts = relative.split(path.sep);
    if (parts.some((part) => part.startsWith("_") || part.startsWith("."))) return undefined;
    return path.extname(relative).toLowerCase() === ".md" ? parts.join("/") : undefined;
  };
  const out: Moved[] = [];
  for (const { from, to } of moved) {
    const old = inContent(from);
    const now = inContent(to);
    if (old !== undefined && now !== undefined && old !== now) out.push({ from: old, to: now });
  }
  return out;
}

/** What the offer asks, after a move of `pages` pages in the project `name`. */
export function keepQuestion(name: string, pages: number): string {
  const moved = pages === 1 ? "a page" : `${pages} pages`;
  return `Ascribe: you moved ${moved} of ${name}, so ${pages === 1 ? "its old address stops" : "their old addresses stop"} working. Keep the addresses you've published? ascribe addresses init starts ascribe.addresses, which records ${pages === 1 ? "this move" : "these moves"} and every move from now on, and the build redirects them.`;
}
