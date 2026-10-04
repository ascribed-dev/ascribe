// The preview's blocks and the lines of the previewed file they stand for,
// read from the source anchors on the page's HTML (site-render contract §7).
// The webview scrolls by them; they're here, apart from the DOM, so the
// mapping can be tested by itself.

/** Lines of the previewed file, counted from 0, first and last included. */
export interface Lines {
  first: number;
  last: number;
}

/**
 * The lines of the previewed file a block stands for, from its
 * `data-ascribe-source` and `data-ascribe-via`. A block written in the page is
 * its own lines; a block from a fragment is the line of the `@include` in the
 * page that brought it in (its outermost include), since the page is the file
 * in the editor. `undefined` when the anchor is malformed or names another
 * file.
 */
export function linesInPage(
  source: string,
  via: string | null | undefined,
  pagePath: string,
): Lines | undefined {
  const includes = (via ?? "").split(" ").filter((s) => s !== "");
  const outermost = includes[0];
  if (outermost !== undefined) {
    const site = split(outermost);
    if (site === undefined || site.path !== pagePath) return undefined;
    const line = Number(site.rest);
    if (!Number.isInteger(line) || line < 1) return undefined;
    return { first: line - 1, last: line - 1 };
  }
  const block = split(source);
  if (block === undefined || block.path !== pagePath) return undefined;
  const match = /^(\d+)-(\d+)$/.exec(block.rest);
  if (match === null) return undefined;
  const first = Number(match[1]);
  const last = Number(match[2]);
  if (first < 1 || last < first) return undefined;
  return { first: first - 1, last: last - 1 };
}

/** `<path>:<rest>`, split at the last `:`, with the path decoded. */
function split(anchor: string): { path: string; rest: string } | undefined {
  const at = anchor.lastIndexOf(":");
  if (at <= 0) return undefined;
  try {
    return { path: decodeURIComponent(anchor.slice(0, at)), rest: anchor.slice(at + 1) };
  } catch {
    return undefined;
  }
}

/**
 * The block to show for a line of the previewed file: among the blocks whose
 * lines hold it, the innermost, which is the one that starts last and, of
 * those, ends first; when none holds it, the nearest one before it, which is
 * the one that ends last and, of those, starts last; and when none starts
 * before it, the first. Of blocks with the same lines (an item and its one
 * paragraph, or everything one `@include` brought in), the first in the page.
 * Blocks with no lines are skipped. The index is into `blocks`; `undefined`
 * when there are none to choose.
 */
export function blockAt(blocks: readonly (Lines | undefined)[], line: number): number | undefined {
  let holding: number | undefined;
  let before: number | undefined;
  let first: number | undefined;
  blocks.forEach((lines, index) => {
    if (lines === undefined) return;
    first ??= index;
    if (lines.first <= line && line <= lines.last) {
      const best = holding === undefined ? undefined : blocks[holding];
      if (
        best === undefined ||
        lines.first > best.first ||
        (lines.first === best.first && lines.last < best.last)
      ) {
        holding = index;
      }
    } else if (lines.last < line) {
      const best = before === undefined ? undefined : blocks[before];
      if (
        best === undefined ||
        lines.last > best.last ||
        (lines.last === best.last && lines.first > best.first)
      ) {
        before = index;
      }
    }
  });
  return holding ?? before ?? first;
}
