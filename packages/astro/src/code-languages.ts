// A code block's language as a class on its <code>, `language-sh`, as
// CommonMark renders it before Shiki highlights it. Shiki's output keeps the
// language only in the <pre>'s `data-language`, which HTML-to-text and
// HTML-to-Markdown converters don't read, so their fences came out with no
// language (decision #1038, site-render contract §5.2). The integration adds
// this Shiki transformer, so it's the same in `astro dev` and the build.
import type { AstroUserConfig } from "astro";

type ShikiTransformer = NonNullable<
  NonNullable<NonNullable<AstroUserConfig["markdown"]>["shikiConfig"]>["transformers"]
>[number];

/**
 * The language Shiki highlighted the block as, the `data-language` Astro
 * gives its <pre>. Astro highlights a block with no language, and one in a
 * language Shiki doesn't know, as `plaintext`, which has no class, as a block
 * with no language has none before highlighting.
 */
export const codeLanguages: ShikiTransformer = {
  name: "@ascribed/astro:code-languages",
  code(node) {
    const lang = this.options.lang;
    if (typeof lang !== "string" || lang === "" || lang === "plaintext") return;
    this.addClassToHast(node, `language-${lang}`);
  },
};
