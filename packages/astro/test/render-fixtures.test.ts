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

// Pages written with and without source anchors (contract §7.5): each anchor
// fixture's, and examples/quill's in the corpus.
const pairs: { name: string; anchored: string; unanchored: string }[] = [
  ...fixtures
    .filter((name) => existsSync(`${root}${name}/unanchored.md`))
    .map((name) => ({
      name,
      anchored: readFileSync(`${root}${name}/input.md`, "utf8"),
      unanchored: readFileSync(`${root}${name}/unanchored.md`, "utf8"),
    })),
  ...readdirSync(`${root}corpus/quill`).flatMap((build) =>
    readdirSync(`${root}corpus/quill/${build}`)
      .filter((file) => file.endsWith(".md") && !file.endsWith(".unanchored.md"))
      .map((file) => ({
        name: `quill/${build}/${file}`,
        anchored: readFileSync(`${root}corpus/quill/${build}/${file}`, "utf8"),
        unanchored: readFileSync(
          `${root}corpus/quill/${build}/${file.replace(/\.md$/, ".unanchored.md")}`,
          "utf8",
        ),
      })),
  ),
];

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

  for (const [processor, render] of [
    ["unified", withUnified],
    ["satteri", withSatteri],
  ] as const) {
    describe(`${processor}: with anchors, the same page`, () => {
      it("finds the pages", () => {
        expect(pairs.length).toBeGreaterThanOrEqual(13);
      });
      for (const { name, anchored, unanchored } of pairs) {
        it(name, async () => {
          const withAnchors = await render(anchored);
          const without = await render(unanchored);
          expect(withAnchors).not.toContain("ascribe-anchor");
          expect(withAnchors).toContain("data-ascribe-source");
          expect(firstDifference(without, withAnchors, { dropAnchors: true }), withAnchors).toBe(
            undefined,
          );
        });
      }
    });
  }

  // Without the plugin, an anchored page is the same page plus invisible
  // comments, and the attributes on the elements Ascribe writes.
  it("degrades quietly without the plugin", () => {
    for (const { name, anchored, unanchored } of pairs) {
      const { html } = markdownToHtml(anchored, { features: { gfm: true } });
      const withoutComments = html.replace(/<!--ascribe-anchor [^>]*-->\n?/g, "");
      const plain = markdownToHtml(unanchored, { features: { gfm: true } }).html;
      expect(firstDifference(plain, withoutComments, { dropAnchors: true }), name).toBeUndefined();
    }
  });

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
