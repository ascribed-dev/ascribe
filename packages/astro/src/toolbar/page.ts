// What the toolbar app reads from the page the dev server rendered: its
// anchored blocks, the element that holds them, the page's own file, and
// whether the anchors arrived. Apart from the app so it can be tested in a
// DOM by itself.
import { findBlock, parseSource } from "@ascribed/review/marks";
import type { Change } from "../review/protocol.js";
import { DATA_SOURCE, DATA_UI, DATA_VIA } from "../names.js";

/** An anchor, as the marks and the overlay take it. */
export interface Anchor {
  source: string;
  via: string[];
}

/** The page's anchored blocks, in order, leaving out review's own elements. */
export function anchoredBlocks(doc: Document): HTMLElement[] {
  return Array.from(doc.querySelectorAll<HTMLElement>(`[${DATA_SOURCE}]`)).filter(
    (element) => element.closest(`[${DATA_UI}]`) === null,
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
  while (root?.hasAttribute(DATA_SOURCE)) root = root.parentElement;
  if (!root || root === first.ownerDocument.documentElement) return first.ownerDocument.body;
  return root;
}

/**
 * The page's own file, as a content path: the file of the first block
 * written in the page itself (with no includes). `null` when no block is.
 */
export function pageFile(blocks: readonly HTMLElement[]): string | null {
  for (const block of blocks) {
    if ((block.getAttribute(DATA_VIA) ?? "").trim() !== "") continue;
    const parsed = parseSource(block.getAttribute(DATA_SOURCE) ?? "");
    if (parsed) return parsed.path;
  }
  return null;
}

/** An element's anchor. */
export function anchorOf(element: Element): Anchor {
  return {
    source: element.getAttribute(DATA_SOURCE) ?? "",
    via: (element.getAttribute(DATA_VIA) ?? "").split(" ").filter(Boolean),
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

/**
 * Whether the page looks light or dark, from its own colors: the first
 * background that isn't see-through, from `element` up, or else its text's
 * color, or else the canvas, which the root's `color-scheme` picks (`prefersDark` says which
 * one "light dark" picks). Review's colors follow it, not the reader's
 * system, since a site may be light only.
 */
export function pageScheme(element: Element, prefersDark: boolean): "light" | "dark" {
  const view = element.ownerDocument.defaultView;
  if (!view) return prefersDark ? "dark" : "light";
  for (let at: Element | null = element; at; at = at.parentElement) {
    const color = parseRgb(view.getComputedStyle(at).backgroundColor);
    if (color && color.alpha >= 0.5) return luminance(color) < 0.18 ? "dark" : "light";
  }
  // No background color: it may be an image or a gradient. Light text means a dark page.
  const text = parseRgb(view.getComputedStyle(element).color);
  if (text && text.alpha >= 0.5) return luminance(text) > 0.4 ? "dark" : "light";
  const scheme = view
    .getComputedStyle(element.ownerDocument.documentElement)
    .colorScheme.split(/\s+/);
  const dark = scheme.includes("dark");
  return dark && (!scheme.includes("light") || prefersDark) ? "dark" : "light";
}

/** `rgb(…)` or `rgba(…)`, as computed styles give colors; `undefined` for anything else. */
function parseRgb(color: string): { r: number; g: number; b: number; alpha: number } | undefined {
  const match = /^rgba?\(([^)]*)\)$/.exec(color.trim());
  if (!match?.[1]) return undefined;
  const parts = match[1].split(/[\s,/]+/).filter(Boolean);
  const [r, g, b] = parts.slice(0, 3).map(Number);
  if (r === undefined || g === undefined || b === undefined) return undefined;
  const last = parts[3];
  const alpha = last === undefined ? 1 : last.endsWith("%") ? parseFloat(last) / 100 : Number(last);
  if ([r, g, b, alpha].some((n) => Number.isNaN(n))) return undefined;
  return { r, g, b, alpha };
}

/** A color's relative luminance, from 0 (black) to 1 (white). */
function luminance({ r, g, b }: { r: number; g: number; b: number }): number {
  const linear = (c: number) => {
    const s = c / 255;
    return s <= 0.04045 ? s / 12.92 : ((s + 0.055) / 1.055) ** 2.4;
  };
  return 0.2126 * linear(r) + 0.7152 * linear(g) + 0.0722 * linear(b);
}
