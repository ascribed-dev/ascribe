// How a page's changes are counted in words, for the preview's header (in the
// webview) and the list of changed pages (in the extension).

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
