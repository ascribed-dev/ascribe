// The page records, `_ascribe/pages.json`, as the site serves them: each
// page's head from `Head.astro`, the site's one sitemap and `robots.txt`, and
// the headers of each page's Markdown version (`_ascribe/headers.json`), in
// `_headers` and from `astro dev`. Each test works on a copy of the site under
// `.e2e-tmp/`, with rules of `[listing]` at every level: a folder whose page
// isn't listed, and one whose page isn't quoted.
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
    {
      title?: string;
      description?: string;
      canonical?: string;
      listing: string;
      listing_from: { level: string; rule?: string; line?: number };
      markdown?: string;
    }
  >;
}

/** The page that isn't listed, and the page that's listed and not quoted. */
const UNLISTED = "/docs/reference/options";
const UNQUOTED = "/docs/guides/my-setup";

/**
 * Lists the docs' pages by a rule at every level of `[listing]`, with no
 * change to a page: the reference isn't listed, and the guides aren't quoted.
 * Gives the docs a line for every crawler too, and their headers in Netlify's
 * format.
 */
async function withUnlisted(root: string): Promise<string> {
  await edit(
    path.join(root, "ascribe.toml"),
    (text) =>
      `${text.replace("agents = true\n", 'agents = true\nheaders = "netlify"\n')}\n[crawlers]\nlines = ["Disallow: /drafts/"]\n` +
      `\n[listing]\ndefault = "listed"\n` +
      `\n[listing.folders]\n"Reference/" = "unlisted"\n"Guides/" = "unquoted"\n` +
      `\n[listing.types]\npage = "listed"\n`,
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
 * other is in both, and one that isn't quoted has `nosnippet`.
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
      ...(record.listing === "unquoted" ? { robots: "nosnippet" } : {}),
    });
    expect(sitemap.includes(record.canonical ?? ""), `${route} in the sitemap`).toBe(!unlisted);
    expect(llms.includes(`${record.markdown ?? route})`), `${route} in llms.txt`).toBe(!unlisted);
  }
}

/** The rule a Markdown version's `X-Robots-Tag` gives, by its page's listing. */
const ROBOTS_RULE: Record<string, string | null> = {
  listed: null,
  unquoted: "nosnippet",
  unlisted: "noindex",
};

/**
 * Checks `_headers` against the records: each page's Markdown version has
 * its page as canonical, and the rule its head has when it isn't listed, and
 * no other path has a rule.
 */
function checkHeaders(headers: string, pages: Records["pages"]): void {
  const want = Object.values(pages)
    .filter((record) => record.markdown !== undefined)
    .sort((a, b) => ((a.markdown ?? "") < (b.markdown ?? "") ? -1 : 1))
    .map((record) => {
      const robots = ROBOTS_RULE[record.listing];
      return (
        `${record.markdown ?? ""}\n  Link: <${record.canonical ?? ""}>; rel="canonical"\n` +
        (robots === null || robots === undefined ? "" : `  X-Robots-Tag: ${robots}\n`)
      );
    });
  expect(want.length).toBeGreaterThan(0);
  expect(headers).toBe(want.join(""));
  expect(headers).toContain(`${UNLISTED}.md\n`);
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
    `https://docs.example.com${UNQUOTED}`,
    "https://docs.example.com/v1.2",
  ];

  it("writes each page's head, the sitemap, and robots.txt from the records in a build", async () => {
    const root = await site("records-build");
    await buildSite(root);
    expect(await readFile(path.join(root, "dist", "robots.txt"), "utf8")).toBe(ROBOTS);
    checkHeaders(
      await readFile(path.join(root, "dist", "_headers"), "utf8"),
      (await records(root)).pages,
    );
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
      expect(head(await text(server.origin, UNQUOTED)).robots).toBe("nosnippet");
      // Each from the rule it's under.
      const pages = (await records(root)).pages;
      expect(pages[UNLISTED]?.listing_from).toMatchObject({
        level: "folder",
        rule: '"Reference/" = "unlisted"',
      });
      expect(pages[UNQUOTED]?.listing_from).toMatchObject({ level: "folder" });
      // A type and the default as strict: the narrower, the type.
      expect(pages["/docs"]?.listing_from).toMatchObject({ level: "type", rule: 'page = "listed"' });
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
      // Each Markdown version answers with its page's rule, and its page as canonical.
      for (const [route, record] of Object.entries((await records(root)).pages)) {
        const response = await fetch(`${server.origin}${record.markdown ?? ""}`);
        expect(response.status, route).toBe(200);
        expect(response.headers.get("x-robots-tag"), route).toBe(ROBOTS_RULE[record.listing]);
        expect(response.headers.get("link"), route).toBe(`<${record.canonical ?? ""}>; rel="canonical"`);
      }
      const unlisted = await fetch(`${server.origin}${UNLISTED}.md`, { method: "HEAD" });
      expect(unlisted.headers.get("x-robots-tag")).toBe("noindex");
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
    `https://docs.example.com${UNQUOTED}`,
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

  it("fails the build when the site's own _headers lacks the lines, with the lines", async () => {
    const root = await withUnlisted(await homeInFront("records-own-headers"));
    await mkdir(path.join(root, "public"));
    const own = "/*\n  X-Frame-Options: DENY\n";
    await writeFile(path.join(root, "public", "_headers"), own);
    const failed = await buildSite(root).then(
      () => undefined,
      (error: unknown) => (error instanceof Error ? error.message : String(error)),
    );
    expect(failed).toContain("public/_headers is the site's own");
    expect(failed).toContain(`${UNLISTED}.md\n`);
    expect(failed).toContain("  X-Robots-Tag: noindex\n");
    // With the lines it asked for, the build passes, and leaves the file as it is.
    const lines = failed?.slice(failed.indexOf("Add these lines to it:\n") + 23) ?? "";
    await writeFile(path.join(root, "public", "_headers"), `${own}${lines}`);
    await buildSite(root);
    expect(await readFile(path.join(root, "dist", "_headers"), "utf8")).toBe(`${own}${lines}`);
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
    expect(existsSync(path.join(root, "dist", "_headers"))).toBe(false);
    const server = await servePreview(root);
    try {
      expect(locations(await text(server.origin, "/docs/sitemap.xml"))).toEqual([
        "https://docs.example.com/docs",
        `https://docs.example.com${UNQUOTED}`,
      ]);
      expect(head(await text(server.origin, UNLISTED)).robots).toBe("noindex");
    } finally {
      await server.stop();
    }
  });
});
