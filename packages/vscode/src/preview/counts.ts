// How a page's changes, and the project's errors, are counted in words, for
// the preview's header (in the webview) and the list of changed pages (in the
// extension).

import type { Counts } from "./protocol.js";

/** "5 changed · 3 added · 1 removed · 1 moved", leaving out kinds with none. */
export function countsText(counts: Counts): string {
  const parts: string[] = [];
  if (counts.changed > 0) parts.push(`${counts.changed} changed`);
  if (counts.added > 0) parts.push(`${counts.added} added`);
  if (counts.removed > 0) parts.push(`${counts.removed} removed`);
  if (counts.moved > 0) parts.push(`${counts.moved} moved`);
  return parts.join(" · ");
}

/**
 * What review says when the project has errors, or `undefined` with none: a
 * page with an error may render oddly, which shouldn't read as the change.
 */
export function errorsText(errors: number): string | undefined {
  if (errors <= 0) return undefined;
  const one = errors === 1;
  return `This project has ${one ? "1 error" : `${errors} errors`}, so a page may not show as it will once ${one ? "it's" : "they're"} fixed.`;
}
