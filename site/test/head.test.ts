// Each page's head, as built. A page of a project, a docs page or one of
// home/'s, has what its head says about it from its record, through
// `Head.astro`; the changelog and the 404 page have no record and write their
// own. Each kind of page keeps the tags it had when the layout wrote them by
// hand, test/fixtures/head.json, and adds only the link to its Markdown
// version and its structured data. A tag's value is held to the record.
import { readFile } from "node:fs/promises";
import path from "node:path";
import { describe, expect, test } from "vitest";
import { distDir, outputDir, requireBuild, siteAddress, siteDir } from "./built.ts";
import { key, tagsOf, type Tag } from "./heads.ts";

/** A page's record in `_ascribe/pages.json`, as much as this reads. */
interface PageRecord {
  title?: string;
  description?: string;
  canonical?: string;
  markdown?: string;
  breadcrumbs: { route: string; title: string }[];
}

/** The tags of the page at `route` in dist/. */
async function builtTags(route: string): Promise<Tag[]> {
  const file = route.endsWith(".html")
    ? path.join(distDir, route)
    : path.join(distDir, ...route.split("/"), "index.html");
  return tagsOf(await readFile(file, "utf8"));
}

/** Each project's records, by route: the docs' and home/'s. */
async function records(): Promise<Record<string, PageRecord>> {
  const read = async (root: string) =>
    (
      JSON.parse(await readFile(path.join(root, "_ascribe", "pages.json"), "utf8")) as {
        pages: Record<string, PageRecord>;
      }
    ).pages;
  const home = path.join(siteDir, "home", ".ascribe", "build", "site", "site");
  return { ...(await read(outputDir)), ...(await read(home)) };
}

/** What the head adds to the saved copy: the Markdown version's link, and structured data. */
const ADDED = new Set([
  '<link rel="alternate" type="text/markdown">',
  '<script type="application/ld+json"> BreadcrumbList',
  '<script type="application/ld+json"> WebSite',
]);

describe("each page's head", () => {
  test("has the tags it had when the layout wrote them, and adds only what's new", async () => {
    requireBuild();
    const saved = JSON.parse(
      await readFile(path.join(siteDir, "test", "fixtures", "head.json"), "utf8"),
    ) as Record<string, string[]>;
    for (const [route, tags] of Object.entries(saved)) {
      const built = (await builtTags(route)).map(key).filter((tag) => !ADDED.has(tag));
      expect(built.toSorted(), route).toEqual(tags.toSorted());
    }
  });

  test("says what its record says, tag for tag", async () => {
    requireBuild();
    const site = await siteAddress();
    const pages = Object.entries(await records());
    expect(pages.length).toBeGreaterThan(20);
    for (const [route, record] of pages) {
      const tags = await builtTags(route);
      const one = (name: string) => {
        const found = tags.filter((tag) => key(tag) === name);
        expect(found, `${route}: ${name}`).toHaveLength(1);
        return found[0] as Tag;
      };
      const title = route === "/" ? "Ascribe" : `${record.title} · Ascribe`;
      expect(one("<title>").text, route).toBe(title);
      expect(one('<meta property="og:title">').attributes["content"], route).toBe(title);
      expect(one('<meta name="description">').attributes["content"], route).toBe(
        record.description,
      );
      expect(one('<meta property="og:description">').attributes["content"], route).toBe(
        record.description,
      );
      expect(one('<link rel="canonical">').attributes["href"], route).toBe(record.canonical);
      expect(one('<meta property="og:url">').attributes["content"], route).toBe(record.canonical);
      expect(one('<meta property="og:site_name">').attributes["content"], route).toBe("Ascribe");
      expect(new URL(one('<meta property="og:image">').attributes["content"] ?? "").origin).toBe(
        site,
      );
      const markdown = tags.filter(
        (tag) => key(tag) === '<link rel="alternate" type="text/markdown">',
      );
      expect(
        markdown.map((tag) => tag.attributes["href"]),
        route,
      ).toEqual(record.markdown === undefined ? [] : [record.markdown]);
      const trail = tags
        .filter((tag) => key(tag).endsWith(" BreadcrumbList"))
        .map(
          (tag) =>
            (JSON.parse(tag.text ?? "") as { itemListElement: { name: string; item: string }[] })
              .itemListElement,
        );
      expect(trail, route).toEqual(
        record.breadcrumbs.length < 2
          ? []
          : [
              record.breadcrumbs.map((step, at) => ({
                "@type": "ListItem",
                position: at + 1,
                name: step.title,
                item: `${site}${step.route}`,
              })),
            ],
      );
    }
  });

  test("names the site as structured data on the home page alone", async () => {
    requireBuild();
    const named = async (route: string) =>
      (await builtTags(route))
        .filter((tag) => key(tag).endsWith(" WebSite"))
        .map((tag) => JSON.parse(tag.text ?? "") as unknown);
    expect(await named("/")).toEqual([
      {
        "@context": "https://schema.org",
        "@type": "WebSite",
        name: "Ascribe",
        url: `${await siteAddress()}/`,
      },
    ]);
    for (const route of ["/why/", "/docs/", "/docs/getting-started/", "/changelog/"]) {
      expect(await named(route), route).toEqual([]);
    }
  });
});
