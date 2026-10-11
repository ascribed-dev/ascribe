// The page records, `_ascribe/pages.json`, as the site serves them: each
// page's head from `Head.astro`, and the site's one sitemap and `robots.txt`,
// in a build and under `astro dev`. Each test works on a copy of the site
// under `.e2e-tmp/`, with a page that isn't listed beside one with nothing set.
import { existsSync } from "node:fs";
import { mkdir, readFile, rm, writeFile } from "node:fs/promises";
import path from "node:path";
import { afterAll, describe, expect, it } from "vitest";
import { buildSite, copySite, edit, serveDev, servePreview, siteDir } from "./harness.js";
import { homeInFront, ONE_ENTRY, twoProjects, write } from "./sites.js";

afterAll(async () => {
  await rm(path.join(siteDir, ".e2e-tmp"), { recursive: true, force: true });
});

/** `_ascribe/pages.json`, as much as these tests read. */
interface Records {
  pages: Record<
    string,
    { title?: string; description?: string; canonical?: string; listing: string; markdown?: string }
  >;
}

/** The page that isn't listed, and the page beside it with nothing set. */
const UNLISTED = "/docs/reference/options";
const LISTED = "/docs/guides/my-setup";

/** Marks the options page as not listed, and gives the docs a line for every crawler. */
async function withUnlisted(root: string): Promise<string> {
  await edit(path.join(root, "content", "Reference", "Options.md"), (text) =>
    text.replace("permalink: options\n", "permalink: options\nlisting: unlisted\n"),
  );
  await edit(
    path.join(root, "ascribe.toml"),
    (text) => `${text}\n[crawlers]\nlines = ["Disallow: /drafts/"]\n`,
  );
  return root;
}

function records(root: string, project = "."): Promise<Records> {
  return readFile(
    path.join(root, project, ".ascribe", "build", "site", "site", "_ascribe", "pages.json"),
    "utf8",
  ).then((text) => JSON.parse(text) as Records);
}

/** The text of `<origin><route>`, failing unless it answers 200. */
async function text(origin: string, route: string): Promise<string> {
  const response = await fetch(`${origin}${route}`);
  expect(response.status, route).toBe(200);
  return response.text();
}

function decode(html: string): string {
  return html
    .replaceAll("&lt;", "<")
    .replaceAll("&gt;", ">")
    .replaceAll("&quot;", '"')
    .replaceAll("&#39;", "'")
    .replaceAll("&amp;", "&");
}

/** What a page's head says about it. */
function head(html: string): {
  title?: string;
  description?: string;
  canonical?: string;
  robots?: string;
} {
  const match = (pattern: RegExp) => {
    const found = pattern.exec(html)?.[1];
    return found === undefined ? undefined : decode(found);
  };
  const result = {
    title: match(/<title>([^<]*)<\/title>/),
    description: match(/<meta name="description" content="([^"]*)"/),
    canonical: match(/<link rel="canonical" href="([^"]*)"/),
    robots: match(/<meta name="robots" content="([^"]*)"/),
  };
  return Object.fromEntries(Object.entries(result).filter(([, value]) => value !== undefined));
}

/** The sitemap's addresses. */
function locations(sitemap: string): string[] {
  return [...sitemap.matchAll(/<loc>([^<]*)<\/loc>/g)].map((found) => decode(found[1] ?? ""));
}

/**
 * Checks each page's head, the sitemap, and `llms.txt` against the records,
 * page by page: a page that isn't listed has `noindex` and is in neither; any
 * other is in both.
 */
async function checkAgainstRecords(
  origin: string,
  pages: Records["pages"],
  sitemap: string[],
  llms: string,
): Promise<void> {
  expect(Object.keys(pages).length).toBeGreaterThan(0);
  for (const [route, record] of Object.entries(pages)) {
    const unlisted = record.listing === "unlisted";
    expect(head(await text(origin, route)), route).toEqual({
      ...(record.title === undefined ? {} : { title: record.title }),
      ...(record.description === undefined ? {} : { description: record.description }),
      ...(record.canonical === undefined ? {} : { canonical: record.canonical }),
      ...(unlisted ? { robots: "noindex" } : {}),
    });
    expect(sitemap.includes(record.canonical ?? ""), `${route} in the sitemap`).toBe(!unlisted);
    expect(llms.includes(`${record.markdown ?? route})`), `${route} in llms.txt`).toBe(!unlisted);
  }
}

describe("a site at the host's root", () => {
  /** The docs at /docs/ with a home page of the site's own at /, and one it leaves out. */
  async function site(name: string): Promise<string> {
    const root = await withUnlisted(await homeInFront(name));
    await edit(path.join(root, "astro.config.mjs"), (text) =>
      text.replace(
        ONE_ENTRY,
        'ascribe({ projects: [{ name: "docs", build: "site" }], sitemap: { exclude: ["/hidden"] } })',
      ),
    );
    await write(root, {
      // A dot in a page's last segment doesn't make it a file.
      "src/pages/v1.2.astro": '<html lang="en"><head><title>1.2</title></head><body>1.2</body></html>\n',
      "src/pages/hidden.astro":
        '<html lang="en"><head><meta name="robots" content="noindex" /></head><body>Hidden</body></html>\n',
    });
    return root;
  }

  const ROBOTS =
    "User-agent: *\nDisallow: /docs/drafts/\n\nSitemap: https://docs.example.com/sitemap.xml\n";
  const SITEMAP = [
    "https://docs.example.com/",
    "https://docs.example.com/docs",
    `https://docs.example.com${LISTED}`,
    "https://docs.example.com/v1.2",
  ];

  it("writes each page's head, the sitemap, and robots.txt from the records in a build", async () => {
    const root = await site("records-build");
    await buildSite(root);
    expect(await readFile(path.join(root, "dist", "robots.txt"), "utf8")).toBe(ROBOTS);
    const server = await servePreview(root);
    try {
      const sitemap = locations(await text(server.origin, "/sitemap.xml"));
      // The docs' listed pages and the home page; not the page that isn't
      // listed, the forwarding pages, the not-found page, or the excluded page.
      expect(sitemap).toEqual(SITEMAP);
      expect(await text(server.origin, "/robots.txt")).toBe(ROBOTS);
      const llms = await text(server.origin, "/docs/llms.txt");
      await checkAgainstRecords(server.origin, (await records(root)).pages, sitemap, llms);
      expect(head(await text(server.origin, UNLISTED)).robots).toBe("noindex");
      expect(head(await text(server.origin, LISTED)).robots).toBeUndefined();
      // The home page is the site's own, with no record, and writes its own head.
      expect(head(await text(server.origin, "/"))).toEqual({ title: "Loom" });
    } finally {
      await server.stop();
    }
  });

  it("serves them in astro dev", async () => {
    const root = await site("records-dev");
    const server = await serveDev(root);
    try {
      const sitemap = locations(await text(server.origin, "/sitemap.xml"));
      expect(sitemap).toEqual(SITEMAP);
      expect(await text(server.origin, "/robots.txt")).toBe(ROBOTS);
      const llms = await text(server.origin, "/docs/llms.txt");
      await checkAgainstRecords(server.origin, (await records(root)).pages, sitemap, llms);
    } finally {
      await server.stop();
    }
  }, 90_000);
});

describe("two projects", () => {
  async function site(name: string): Promise<string> {
    const root = await withUnlisted(await twoProjects(name));
    await edit(
      path.join(root, "home", "ascribe.toml"),
      (text) =>
        `${text}\n[crawlers]\nlines = ["Disallow: /private/", "User-agent: ExampleBot", "Disallow: /"]\n`,
    );
    return root;
  }

  const ROBOTS =
    "User-agent: *\nDisallow: /docs/drafts/\nDisallow: /private/\n\n" +
    "User-agent: ExampleBot\nDisallow: /\n\n" +
    "Sitemap: https://docs.example.com/sitemap.xml\n";
  const SITEMAP = [
    "https://docs.example.com/",
    "https://docs.example.com/docs",
    `https://docs.example.com${LISTED}`,
    "https://docs.example.com/why",
  ];

  it("make one sitemap and one robots.txt in a build", async () => {
    const root = await site("records-two-build");
    await buildSite(root);
    expect(locations(await readFile(path.join(root, "dist", "sitemap.xml"), "utf8"))).toEqual(
      SITEMAP,
    );
    expect(await readFile(path.join(root, "dist", "robots.txt"), "utf8")).toBe(ROBOTS);
  });

  it("make them in astro dev", async () => {
    const root = await site("records-two-dev");
    const server = await serveDev(root);
    try {
      expect(locations(await text(server.origin, "/sitemap.xml"))).toEqual(SITEMAP);
      expect(await text(server.origin, "/robots.txt")).toBe(ROBOTS);
    } finally {
      await server.stop();
    }
  }, 90_000);
});

describe("what a site leaves out", () => {
  it("keeps a robots.txt and a sitemap of the site's own", async () => {
    const root = await withUnlisted(await homeInFront("records-own-robots"));
    const own = "User-agent: *\nAllow: /\n";
    const sitemap = "<urlset/>\n";
    await mkdir(path.join(root, "public"));
    await writeFile(path.join(root, "public", "robots.txt"), own);
    await buildSite(root);
    expect(await readFile(path.join(root, "dist", "robots.txt"), "utf8")).toBe(own);
    expect(existsSync(path.join(root, "dist", "sitemap.xml"))).toBe(true);
    await writeFile(path.join(root, "public", "sitemap.xml"), sitemap);
    await buildSite(root);
    expect(await readFile(path.join(root, "dist", "sitemap.xml"), "utf8")).toBe(sitemap);
  });

  it("writes no sitemap for a site with no address, and robots.txt without it", async () => {
    const root = await withUnlisted(await homeInFront("records-no-site"));
    // What agents read links by absolute address, so it needs `site` too.
    await edit(path.join(root, "ascribe.toml"), (text) =>
      text.replace('site = "https://docs.example.com"\n', "").replace("agents = true\n", ""),
    );
    await buildSite(root);
    expect(existsSync(path.join(root, "dist", "sitemap.xml"))).toBe(false);
    expect(await readFile(path.join(root, "dist", "robots.txt"), "utf8")).toBe(
      "User-agent: *\nDisallow: /docs/drafts/\n",
    );
    const html = await readFile(
      path.join(root, "dist", "docs", "reference", "options", "index.html"),
      "utf8",
    );
    expect(head(html)).toEqual({
      title: "loom.yaml options",
      description: "Every Loom option.",
      robots: "noindex",
    });
  });

  it("writes the sitemap under a base that isn't /, and no robots.txt", async () => {
    // The example as it is: Astro's `base` is the docs' base path.
    const root = await withUnlisted(await copySite("records-base"));
    await buildSite(root);
    expect(existsSync(path.join(root, "dist", "robots.txt"))).toBe(false);
    const server = await servePreview(root);
    try {
      expect(locations(await text(server.origin, "/docs/sitemap.xml"))).toEqual([
        "https://docs.example.com/docs",
        `https://docs.example.com${LISTED}`,
      ]);
      expect(head(await text(server.origin, UNLISTED)).robots).toBe("noindex");
    } finally {
      await server.stop();
    }
  });
});
