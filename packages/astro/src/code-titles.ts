// A code block's title, shown above it. A `@snippet`'s title is in the code
// block's info string, as `title="…"` (SPEC §4.8), and Astro's own code
// highlighting ignores it (issue #106), so the integration adds this Shiki
// transformer, which puts a titled block in a <figure class="code-title"> with
// the title as its <figcaption>.
import type { AstroUserConfig } from "astro";

type ShikiTransformer = NonNullable<
  NonNullable<NonNullable<AstroUserConfig["markdown"]>["shikiConfig"]>["transformers"]
>[number];

/** The `title="…"` in a code block's meta, if it has one. */
export function codeTitle(meta: string): string | undefined {
  return /(?:^|\s)title="([^"]*)"/.exec(meta)?.[1] || undefined;
}

export const codeTitles: ShikiTransformer = {
  name: "@ascribed/astro:code-titles",
  root(root) {
    const meta = (this.options.meta as { __raw?: string } | undefined)?.__raw ?? "";
    const title = codeTitle(meta);
    const pre = root.children.find((node) => node.type === "element");
    // A site's own transformer may have put it in a figure already.
    if (title === undefined || pre === undefined || pre.tagName === "figure") return;
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
