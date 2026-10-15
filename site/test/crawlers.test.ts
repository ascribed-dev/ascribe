// What a search engine and an agent find: the site's one sitemap lists every
// built page meant to be found, the docs', home/'s, and the changelog, and
// nothing else, and a page that says `noindex` is in neither the sitemap nor
// `llms.txt`. No docs page is marked unlisted, so a fixture, a small built
// site with one, shows the rules hold for such a page too.
import { readFile } from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { describe, expect, test } from "vitest";
import { unindexed } from "../src/site.ts";
import { forwardingPages, distDir, requireBuild, siteAddress } from "./built.ts";
import { crawl, crawlerProblems, type Crawled } from "./crawled.ts";

const fixtureDir = fileURLToPath(new URL("fixtures/crawled", import.meta.url));

describe("a built site with a page that isn't listed (fixture)", () => {
  const site = "https://example.com";
  const skipped = ["/go/start/"];

  test("is read as a crawler finds it", async () => {
    const crawled = await crawl(fixtureDir);
    expect(crawled.pages.filter((page) => page.noindex).map((page) => page.route)).toEqual([
      "/docs/hidden/",
    ]);
    expect(crawled.pages.map((page) => page.route)).not.toContain("/404.html");
    expect(crawled.sitemap).toHaveLength(4);
    expect(crawled.llms).toEqual([
      "https://example.com/docs/index.md",
      "https://example.com/docs/guide.md",
    ]);
  });

  test("passes", async () => {
    expect(crawlerProblems(await crawl(fixtureDir), { site, skipped })).toEqual([]);
  });

  test("fails for a page that isn't listed in the sitemap or llms.txt, and for a page left out", async () => {
    const crawled = await crawl(fixtureDir);
    const wrong: Crawled = {
      ...crawled,
      sitemap: [
        ...(crawled.sitemap ?? []).filter((address) => !address.endsWith("/changelog/")),
        `${site}/docs/hidden/`,
        `${site}/docs/gone/`,
      ],
      llms: [...crawled.llms, `${site}/docs/hidden.md`],
    };
    expect(crawlerProblems(wrong, { site, skipped })).toEqual([
      "sitemap.xml: https://example.com/docs/hidden/ says noindex",
      "sitemap.xml: https://example.com/docs/gone/ isn't a built page",
      "/changelog/: a built page that isn't in sitemap.xml",
      "llms.txt: /docs/hidden.md is of a page that says noindex",
    ]);
  });
});

describe("the built site", () => {
  test("has a sitemap of every page meant to be found, and no page that says noindex in it or in llms.txt", async () => {
    requireBuild();
    const site = await siteAddress();
    const skipped = [...(await forwardingPages()), ...unindexed];
    expect(crawlerProblems(await crawl(distDir), { site, skipped })).toEqual([]);
  });

  test("lists the docs' pages, home/'s, and the changelog, and not the demo's page", async () => {
    requireBuild();
    const site = await siteAddress();
    const sitemap = (await crawl(distDir)).sitemap ?? [];
    for (const route of ["/", "/why/", "/changelog/", "/docs/", "/docs/getting-started/"]) {
      expect(sitemap).toContain(`${site}${route}`);
    }
    expect(sitemap.filter((address) => new URL(address).pathname.startsWith("/demo/"))).toEqual([]);
  });

  test("has a robots.txt at its root that names the sitemap", async () => {
    requireBuild();
    const { robots } = await crawl(distDir);
    expect(robots?.trimEnd().split("\n").at(-1)).toBe(
      `Sitemap: ${await siteAddress()}/sitemap.xml`,
    );
  });

  test("gives a page with no record, the changelog, its own canonical link", async () => {
    requireBuild();
    const html = await readFile(path.join(distDir, "changelog", "index.html"), "utf8");
    expect(html).toContain(`<link rel="canonical" href="${await siteAddress()}/changelog/">`);
  });
});
