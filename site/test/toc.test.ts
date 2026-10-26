// The docs page's table of contents: which headings it lists, and each as
// HTML, with its code spans, from the rendered page. It needs no build.
import { expect, test } from "vitest";
import { tocHeadings } from "../src/toc.ts";

test("the contents list sections and subsections, each heading's code spans as <code>", () => {
  const rendered = [
    '<h1 id="getting-started">Getting started</h1>',
    '<h2 id="create-ascribetoml">Create <code>ascribe.toml</code></h2>',
    '<h3 id="3-project">3. <code>[project]</code></h3>',
    '<h4 id="deeper">Deeper</h4>',
    '<h2 id="plain">Plain &amp; simple</h2>',
  ].join("\n");
  const headings = [
    { depth: 1, slug: "getting-started", text: "Getting started" },
    { depth: 2, slug: "create-ascribetoml", text: "Create ascribe.toml" },
    { depth: 3, slug: "3-project", text: "3. [project]" },
    { depth: 4, slug: "deeper", text: "Deeper" },
    { depth: 2, slug: "plain", text: "Plain & simple" },
  ];
  expect(tocHeadings(headings, rendered).map(({ slug, html }) => [slug, html])).toEqual([
    ["create-ascribetoml", "Create <code>ascribe.toml</code>"],
    ["3-project", "3. <code>[project]</code>"],
    ["plain", "Plain &amp; simple"],
  ]);
});

test("a heading's links and ids are left out, so nothing nests in the contents' link", () => {
  const rendered =
    '<h2 class="x" id="see"><a href="#see" aria-hidden="true"></a>See <a href="/docs/"><code id="c">ascribe</code></a></h2>';
  expect(tocHeadings([{ depth: 2, slug: "see", text: "See ascribe" }], rendered)[0]?.html).toBe(
    "See <code>ascribe</code>",
  );
});

test("a heading the page doesn't have is its text, escaped", () => {
  expect(tocHeadings([{ depth: 2, slug: "gone", text: "a <b> & c" }])[0]?.html).toBe(
    "a &lt;b&gt; &amp; c",
  );
});
