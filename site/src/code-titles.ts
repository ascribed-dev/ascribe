// A code block's title, shown above it. Ascribe writes a `@snippet`'s title
// into the code block's info string, as `title="…"`, and Astro's own code
// highlighting doesn't show it (issue #106 in this repository), so this
// Shiki transformer puts the block in a <figure> with the title as its caption.
import type { ShikiTransformer } from "shiki";

export const codeTitles: ShikiTransformer = {
  name: "code-titles",
  root(root) {
    const meta = (this.options.meta as { __raw?: string } | undefined)?.__raw ?? "";
    const title = /(?:^|\s)title="([^"]*)"/.exec(meta)?.[1];
    const pre = root.children.find((node) => node.type === "element");
    if (!title || !pre) {
      return;
    }
    root.children = [
      {
        type: "element",
        tagName: "figure",
        properties: { className: ["code-title"] },
        children: [
          {
            type: "element",
            tagName: "figcaption",
            properties: {},
            children: [{ type: "text", value: title }],
          },
          pre,
        ],
      },
    ];
  },
};
