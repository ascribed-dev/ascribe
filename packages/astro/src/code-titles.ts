// A code block's caption, shown above it: its title, and what a tested
// example was tested with. A `@snippet`'s title is in the code block's info
// string, as `title="…"`, and its tested line as `tested="…"` (SPEC §4.8), and
// Astro's own code highlighting ignores both (issue #106), so the integration
// adds this Shiki transformer, which puts such a block in a
// <figure class="code-title"> with a <figcaption>: the title, then the tested
// line in a <div class="ascribe-code-tested"> of its own, whose "Tested with"
// lead-in is the element library's stylesheet's (site-render contract §5.1).
import type { AstroUserConfig } from "astro";
import type { Element, ElementContent } from "hast";
import { CLASS_CODE_TESTED } from "./names.js";

type ShikiTransformer = NonNullable<
  NonNullable<NonNullable<AstroUserConfig["markdown"]>["shikiConfig"]>["transformers"]
>[number];

/**
 * The value of the first `key=value` pair in a code block's meta, the info
 * string past its language, when it isn't empty. The meta has its Markdown
 * escapes taken off, and is read as SPEC §3.3.1 reads an info string, as
 * `ascribe_core::info_string` does: words parted by spaces and tabs, a pair's
 * value a token or a quoted string in which `\"` and `\\` escape, and a pair
 * followed by a space, a tab, or the end. Any other word isn't a pair.
 */
function metaValue(meta: string, key: string): string | undefined {
  let i = 0;
  while (i < meta.length) {
    if (meta[i] === " " || meta[i] === "\t") {
      i += 1;
      continue;
    }
    const pair = /^([a-z][a-z0-9-]*)=/.exec(meta.slice(i));
    let end = wordEnd(meta, i);
    let value: string | undefined;
    if (pair !== null) {
      let j = i + pair[0].length;
      if (meta[j] === '"') {
        let text = "";
        j += 1;
        while (j < meta.length && meta[j] !== '"') {
          if (meta[j] === "\\" && (meta[j + 1] === '"' || meta[j + 1] === "\\")) j += 1;
          text += meta[j];
          j += 1;
        }
        if (j < meta.length && (j + 1 === meta.length || isSpace(meta[j + 1]))) {
          value = text;
          end = j + 1;
        }
      } else if (j < end && !meta.slice(j, end).includes('"')) {
        value = meta.slice(j, end);
      }
    }
    if (value !== undefined && pair?.[1] === key) return value || undefined;
    i = end;
  }
  return undefined;
}

function isSpace(c: string | undefined): boolean {
  return c === " " || c === "\t";
}

function wordEnd(text: string, from: number): number {
  let i = from;
  while (i < text.length && !isSpace(text[i])) i += 1;
  return i;
}

/** The `title="…"` in a code block's meta, if it has one. */
export function codeTitle(meta: string): string | undefined {
  return metaValue(meta, "title");
}

/** The `tested="…"` in a code block's meta, if it has one. */
export function codeTested(meta: string): string | undefined {
  return metaValue(meta, "tested");
}

/**
 * The block, in a figure whose caption holds its title and its tested line;
 * `undefined` when it has neither.
 */
export function captioned(meta: string, block: ElementContent): Element | undefined {
  const title = codeTitle(meta);
  const tested = codeTested(meta);
  if (title === undefined && tested === undefined) return undefined;
  const caption: ElementContent[] = [];
  if (title !== undefined) caption.push({ type: "text", value: title });
  if (tested !== undefined) {
    caption.push({
      type: "element",
      tagName: "div",
      properties: { className: [CLASS_CODE_TESTED] },
      children: [{ type: "text", value: tested }],
    });
  }
  return {
    type: "element",
    tagName: "figure",
    properties: { className: ["code-title"] },
    children: [
      { type: "element", tagName: "figcaption", properties: {}, children: caption },
      block,
    ],
  };
}

export const codeTitles: ShikiTransformer = {
  name: "@ascribed/astro:code-titles",
  root(root) {
    const meta = (this.options.meta as { __raw?: string } | undefined)?.__raw ?? "";
    const pre = root.children.find((node) => node.type === "element");
    // A site's own transformer may have put it in a figure already.
    if (pre === undefined || pre.tagName === "figure") return;
    const figure = captioned(meta, pre);
    if (figure !== undefined) root.children = [figure];
  },
};
