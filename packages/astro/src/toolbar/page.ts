// What the toolbar app reads from the page the dev server rendered: its
// anchored blocks, the element that holds them, the page's own file, and
// whether the anchors arrived. Apart from the app so it can be tested in a
// DOM by itself.
import { findBlock, parseSource } from "@ascribed/review/marks";
import type { Change } from "../review/protocol.js";

/** An anchor, as the marks and the overlay take it. */
export interface Anchor {
  source: string;
  via: string[];
}

/** The page's anchored blocks, in order, leaving out review's own elements. */
export function anchoredBlocks(doc: Document): HTMLElement[] {
  return Array.from(doc.querySelectorAll<HTMLElement>("[data-ascribe-source]")).filter(
    (element) => element.closest("[data-ascribe-ui]") === null,
  );
}

/**
 * The element holding the page's content: the innermost one holding every
 * anchored block, and not a block itself. `undefined` with none.
 */
export function contentRoot(blocks: readonly HTMLElement[]): HTMLElement | undefined {
  const first = blocks[0];
  if (first === undefined) return undefined;
  let root: HTMLElement | null = first.parentElement;
  while (root && !blocks.every((block) => root?.contains(block))) root = root.parentElement;
  while (root?.hasAttribute("data-ascribe-source")) root = root.parentElement;
  if (!root || root === first.ownerDocument.documentElement) return first.ownerDocument.body;
  return root;
}

/**
 * The page's own file, as a content path: the file of the first block
 * written in the page itself (with no includes). `null` when no block is.
 */
export function pageFile(blocks: readonly HTMLElement[]): string | null {
  for (const block of blocks) {
    if ((block.getAttribute("data-ascribe-via") ?? "").trim() !== "") continue;
    const parsed = parseSource(block.getAttribute("data-ascribe-source") ?? "");
    if (parsed) return parsed.path;
  }
  return null;
}

/** An element's anchor. */
export function anchorOf(element: Element): Anchor {
  return {
    source: element.getAttribute("data-ascribe-source") ?? "",
    via: (element.getAttribute("data-ascribe-via") ?? "").split(" ").filter(Boolean),
  };
}

/**
 * Whether the page's anchors arrived: false when the page has changes but
 * no anchored blocks, or when fewer than half of its changed blocks can be
 * found. A layout or component that rebuilds the content's elements drops
 * their attributes, and then nothing can be placed.
 */
export function anchorsArrived(
  root: ParentNode,
  blocks: number,
  changes: readonly Change[],
): boolean {
  // A removed block is found by the block it came after, or the one it was in.
  const anchors = changes
    .map((change) => change.now ?? change.after ?? change.parent)
    .filter((anchor) => anchor !== undefined);
  if (anchors.length === 0) return true;
  if (blocks === 0) return false;
  const found = anchors.filter((anchor) => findBlock(root, anchor) !== null).length;
  return found * 2 >= anchors.length;
}

/** Where a block's source is, for opening it: `guides/install.md`, line 12. */
export function sourceLocation(source: string): { path: string; line: number } | undefined {
  const parsed = parseSource(source);
  return parsed ? { path: parsed.path, line: parsed.first } : undefined;
}
