// Runs every fixture in tests/render/ through both markdown processors Astro
// 7.3 can use, each with the plugin where Astro puts it (tests/render/README.md):
// the unified pipeline with `rehypeAscribeAttributes`, and Sätteri with
// `satteriAscribeAttributes`. Not a full Astro build, whose image
// optimization would replace every `src`.
import { existsSync, readdirSync, readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { markdownToHtml } from "satteri";
import rehypeRaw from "rehype-raw";
import rehypeStringify from "rehype-stringify";
import remarkGfm from "remark-gfm";
import remarkParse from "remark-parse";
import remarkRehype from "remark-rehype";
import remarkSmartypants from "remark-smartypants";
import { unified } from "unified";
import { describe, expect, it } from "vitest";
import rehypeAscribeAttributes from "../src/rehype.js";
import { satteriAscribeAttributes } from "../src/satteri.js";
import { firstDifference } from "./html.js";

const root = fileURLToPath(new URL("../../../tests/render/", import.meta.url));
const fixtures = readdirSync(root, { withFileTypes: true })
  .filter((entry) => entry.isDirectory() && existsSync(`${root}${entry.name}/input.md`))
  .map((entry) => entry.name)
  .sort();

// Astro's `unified()` processor, up to the user's rehype plugins and `rehype-raw`.
async function withUnified(markdown: string): Promise<string> {
  const file = await unified()
    .use(remarkParse)
    .use(remarkGfm)
    .use(remarkSmartypants)
    .use(remarkRehype, { allowDangerousHtml: true, passThrough: [] })
    .use(rehypeAscribeAttributes)
    .use(rehypeRaw)
    .use(rehypeStringify, { allowDangerousHtml: true })
    .process(markdown);
  return String(file);
}

// Astro's default Sätteri processor: GFM and smart punctuation on.
async function withSatteri(markdown: string): Promise<string> {
  const { html } = await markdownToHtml(markdown, {
    hastPlugins: [satteriAscribeAttributes()],
    features: { gfm: true, smartPunctuation: true },
  });
  return html;
}

describe("tests/render fixtures", () => {
  it("finds the fixtures", () => {
    expect(fixtures.length).toBeGreaterThanOrEqual(12);
  });

  for (const [processor, render] of [
    ["unified", withUnified],
    ["satteri", withSatteri],
  ] as const) {
    describe(processor, () => {
      for (const name of fixtures) {
        it(name, async () => {
          const input = readFileSync(`${root}${name}/input.md`, "utf8");
          const expected = readFileSync(`${root}${name}/expected.html`, "utf8");
          const actual = await render(input);
          expect(firstDifference(expected, actual), actual).toBeUndefined();
        });
      }
    });
  }

  // Without the plugin the fixtures must fail: the comparison can tell.
  it("fails when the plugin isn't applied", async () => {
    const input = readFileSync(`${root}heading-id/input.md`, "utf8");
    const expected = readFileSync(`${root}heading-id/expected.html`, "utf8");
    const { html } = markdownToHtml(input, { features: { gfm: true } });
    expect(firstDifference(expected, html)).toContain("expected attributes");
    const image = readFileSync(`${root}image-inline/input.md`, "utf8");
    const imageExpected = readFileSync(`${root}image-inline/expected.html`, "utf8");
    expect(firstDifference(imageExpected, markdownToHtml(image).html)).toBeDefined();
  });
});
