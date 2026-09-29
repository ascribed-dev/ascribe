// The attribute marker as a rehype plugin, for Astro's `unified()` markdown
// processor (`@astrojs/markdown-remark`).
//
// Astro builds that processor as: remark-parse, remark-gfm, remark-smartypants,
// the user's remark plugins, remark-collect-images, remark-rehype (raw HTML
// allowed), syntax highlighting, the user's rehype plugins, `rehypeImages`,
// `rehypeHeadingIds`, and finally `rehype-raw`. A user rehype plugin therefore
// runs where markers are still `raw` nodes and before both of Astro's passes:
// `rehypeImages` copies an `<img>`'s properties into the image it optimizes,
// and `rehypeHeadingIds` keeps an id a heading already has and records it for
// the table of contents. A rehype plugin, not a remark one, so that both of
// Astro's processors can share `findEdits` (Q151).

import type { Root } from "hast";
import { findEdits, toProperty, type HastNode } from "./attributes.js";

/** Applies the site-render contract's attribute markers (`tessera-attributes`). */
export default function rehypeTesseraAttributes(): (tree: Root) => void {
  return (tree) => {
    const edits = findEdits(tree as HastNode);
    if (edits.length === 0) return;
    const removed = new Set<HastNode>();
    for (const edit of edits) {
      const properties = (edit.target.properties ??= {});
      for (const attribute of edit.attributes) {
        const [name, value] = toProperty(attribute);
        properties[name] = value;
      }
      for (const node of edit.remove) removed.add(node);
      for (const { node, value } of edit.trim) {
        if (value === "") removed.add(node);
        else node.value = value;
      }
    }
    dropNodes(tree as HastNode, removed);
  };
}

function dropNodes(node: HastNode, removed: Set<HastNode>): void {
  if (node.children === undefined) return;
  node.children = node.children.filter((child) => !removed.has(child));
  for (const child of node.children) dropNodes(child, removed);
}
