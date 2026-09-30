// Comparing two renderings of a page's content: the tree of elements, their
// attributes, and their text, in one form for both.
//
// Both sides are parsed by one browser engine (scripts stripped, so the
// element library hasn't run), and each element is reduced to `{ tag, attrs,
// children }`. What is deliberately not compared, and why:
//
// - whitespace between and around nodes: comrak and Astro's processors
//   break lines differently; the words and their order are compared;
// - the `height`, `loading`, `decoding`, and `data-image-component` of an image: Astro's image
//   service adds them; the `alt`, `title`, and `width` an author writes, and
//   the marker's other attributes, are compared;
// - the inside of a `<pre>`: Astro highlights code with Shiki (`<span>`s
//   with inline colors); the language and the text are compared;
// - typographic punctuation: Astro's `smartypants` turns quotes and dashes
//   into curly ones, which never happens inside a marker; both sides are
//   compared with straight ones.
import type { Page } from "playwright-core";

// What the comparison leaves out.
export interface Tree {
  tag: string;
  attrs: Record<string, string>;
  children: (Tree | string)[];
  /** For a `<pre>`: the language and the text, which are what is compared. */
  language?: string;
  text?: string;
}

/** Reduces the element matching `selector` to a tree, in the page. */
export async function treeOf(page: Page, selector: string): Promise<Tree | undefined> {
  return page.evaluate((selector) => {
    const IMAGE_ADDED_BY_ASTRO = new Set(["height", "loading", "decoding", "data-image-component"]);
    const straight = (text: string): string =>
      text
        .replace(/[‘’]/g, "'")
        .replace(/[“”]/g, '"')
        .replace(/[–—]/g, "-")
        .replace(/…/g, "...")
        .replace(/\s+/g, " ");
    const reduce = (element: Element): unknown => {
      const tag = element.tagName.toLowerCase();
      if (tag === "pre") {
        const code = element.querySelector("code");
        const language =
          element.getAttribute("data-language") ??
          /(?:^|\s)language-([^\s]+)/.exec(code?.getAttribute("class") ?? "")?.[1] ??
          "";
        return {
          tag,
          attrs: {},
          children: [],
          language,
          text: (element.textContent ?? "").replace(/\n$/, ""),
        };
      }
      const attrs: Record<string, string> = {};
      for (const name of element.getAttributeNames().sort()) {
        if (tag === "img" && IMAGE_ADDED_BY_ASTRO.has(name)) continue;
        attrs[name] = element.getAttribute(name) ?? "";
      }
      const children: unknown[] = [];
      for (const child of element.childNodes) {
        if (child.nodeType === Node.ELEMENT_NODE) {
          children.push(reduce(child as Element));
        } else if (child.nodeType === Node.TEXT_NODE) {
          const text = straight(child.textContent ?? "");
          const last = children[children.length - 1];
          if (typeof last === "string") children[children.length - 1] = straight(last + text);
          else children.push(text);
        }
      }
      return {
        tag,
        attrs,
        children: children
          .map((c) => (typeof c === "string" ? c.trim() : c))
          .filter((c) => c !== ""),
      };
    };
    const root = document.querySelector(selector);
    return root ? (reduce(root) as never) : undefined;
  }, selector);
}

/** Calls `visit` on every element of a tree, and returns the tree. */
export function walk(tree: Tree, visit: (element: Tree) => void): Tree {
  visit(tree);
  for (const child of tree.children) if (typeof child !== "string") walk(child, visit);
  return tree;
}
