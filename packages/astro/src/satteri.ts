// The attribute marker as a Sätteri hast plugin, for Astro 7.3's default
// markdown processor (`@astrojs/markdown-satteri`).
//
// Astro runs that processor's hast plugins in this order: highlighting, the
// user's plugins, its image marker (which reads an `<img>`'s properties), and
// its heading ids (which keeps an id a heading already has). A user plugin
// therefore runs before both, and sees a marker as two `raw` nodes, as the
// unified processor's plugin does. The rules are `findEdits`'s.

import type { HastNode as SatteriNode, HastVisitorInstance } from "satteri";
import { findEdits, toProperty, type HastNode } from "./attributes.js";

/** Applies the site output's attribute markers (`ascribe-attributes`). */
export function satteriAscribeAttributes(): HastVisitorInstance & { name: string } {
  return {
    name: "ascribe-attributes",
    before(root, ctx) {
      for (const edit of findEdits(root as HastNode)) {
        const target = edit.target as unknown as SatteriNode;
        for (const attribute of edit.attributes) {
          const [name, value] = toProperty(attribute);
          ctx.setProperty(target, name, value);
        }
        for (const node of edit.remove) ctx.removeNode(node as unknown as SatteriNode);
        for (const { node, value } of edit.trim) {
          if (value === "") ctx.removeNode(node as unknown as SatteriNode);
          else ctx.replaceNode(node as unknown as SatteriNode, { type: "text", value });
        }
      }
    },
  };
}
