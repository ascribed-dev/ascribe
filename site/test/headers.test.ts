// The built site's `_headers`, which `[consumer] headers = "netlify"` in
// ../docs/ascribe.toml has @ascribed/astro write: each docs page's Markdown
// version has its rule, and no other path has one. No docs page is marked
// unlisted, so a fixture, a `_headers` and the `pages.json` it was built for,
// shows the rules hold for such a page too.
import { readFile } from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { describe, expect, test } from "vitest";
import { distDir, outputDir, requireBuild } from "./built.ts";
import { headerProblems, parseHeaders, readPages } from "./headers.ts";

const fixtureDir = fileURLToPath(new URL("fixtures/headers", import.meta.url));

describe("a _headers for pages with one that isn't listed (fixture)", () => {
  async function fixture(): Promise<string> {
    return readFile(path.join(fixtureDir, "_headers"), "utf8");
  }

  test("gives the page that isn't listed noindex, and every page its canonical link", async () => {
    const headers = parseHeaders(await fixture());
    expect(headers.get("/docs/hidden.md")).toEqual({
      link: '<https://example.com/docs/hidden/>; rel="canonical"',
      "x-robots-tag": "noindex",
    });
    expect(headers.get("/docs/guide.md")).toEqual({
      link: '<https://example.com/docs/guide/>; rel="canonical"',
    });
  });

  test("passes", async () => {
    expect(headerProblems(await fixture(), await readPages(fixtureDir))).toEqual([]);
  });

  test("fails for a rule or a header left out, a header that's wrong or extra, and a path that isn't a Markdown version", async () => {
    const wrong = [
      "/docs/guide.md",
      '  Link: <https://example.com/docs/other/>; rel="canonical"',
      "  X-Robots-Tag: nosnippet",
      "/docs/hidden.md",
      '  Link: <https://example.com/docs/hidden/>; rel="canonical"',
      "/docs/quoted.md",
      '  Link: <https://example.com/docs/quoted/>; rel="canonical"',
      "  X-Robots-Tag: noindex",
      "/docs/guide/",
      "  X-Robots-Tag: noindex",
      "",
    ].join("\n");
    expect(headerProblems(wrong, await readPages(fixtureDir))).toEqual([
      "/docs/index.md: no rule",
      '/docs/guide.md: link is <https://example.com/docs/other/>; rel="canonical", not <https://example.com/docs/guide/>; rel="canonical"',
      "/docs/guide.md: x-robots-tag, which its page doesn't give",
      "/docs/hidden.md: no x-robots-tag",
      "/docs/quoted.md: x-robots-tag is noindex, not nosnippet",
      "/docs/guide/: a rule for a path that isn't a Markdown version",
    ]);
  });
});

describe("the built site", () => {
  test("has a _headers rule for each docs page's Markdown version, and for nothing else", async () => {
    requireBuild();
    const text = await readFile(path.join(distDir, "_headers"), "utf8");
    expect(headerProblems(text, await readPages(outputDir))).toEqual([]);
  });

  test("has the rules the docs' _ascribe/headers.json lists, exactly", async () => {
    requireBuild();
    const text = await readFile(path.join(distDir, "_headers"), "utf8");
    const list = JSON.parse(
      await readFile(path.join(outputDir, "_ascribe", "headers.json"), "utf8"),
    ) as { headers: { path: string; headers: Record<string, string> }[] };
    const lower = (headers: Record<string, string>) =>
      Object.fromEntries(
        Object.entries(headers).map(([name, value]) => [name.toLowerCase(), value]),
      );
    expect(parseHeaders(text)).toEqual(
      new Map(list.headers.map((rule) => [rule.path, lower(rule.headers)])),
    );
  });
});
