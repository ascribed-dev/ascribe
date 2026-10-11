import { mkdirSync, mkdtempSync, utimesSync, writeFileSync } from "node:fs";
import type { IncomingMessage, ServerResponse } from "node:http";
import { tmpdir } from "node:os";
import path from "node:path";
import { describe, expect, it } from "vitest";
import { forwardingRoutes } from "../src/permalinks.js";
import {
  pageList,
  readRecords,
  recordFor,
  robotsText,
  routeAs,
  sitemapAddresses,
  sitemapXml,
  type PageRecord,
} from "../src/records.js";
import {
  devRouted,
  devSitePages,
  rootFiles,
  rootFilesMiddleware,
  type AstroRoute,
  type RootFileSources,
} from "../src/root-files.js";

function siteRoot(records: unknown): string {
  const root = mkdtempSync(path.join(tmpdir(), "ascribe-records-"));
  if (records !== undefined) write(root, records);
  return root;
}

function write(root: string, records: unknown): void {
  mkdirSync(path.join(root, "_ascribe"), { recursive: true });
  writeFileSync(path.join(root, "_ascribe", "pages.json"), JSON.stringify(records));
}

const RECORDS = {
  version: 1,
  site: "https://docs.example.com",
  pages: {
    "/docs/": {
      source: "index.md",
      title: "Loom",
      canonical: "https://docs.example.com/docs/",
      listing: "listed",
    },
    "/docs/drafts/": {
      source: "drafts.md",
      title: "Drafts",
      canonical: "https://docs.example.com/docs/drafts/",
      listing: "unlisted",
    },
  },
  robots: { lines: ["Disallow: /docs/private/"] },
};

function page(route: string, listing: PageRecord["listing"] = "listed"): PageRecord {
  return { route, source: `${route.slice(1)}.md`, listing };
}

describe("readRecords and recordFor", () => {
  it("reads each page with its route, and finds a page by its source", () => {
    const root = siteRoot(RECORDS);
    expect(pageList(readRecords(root)).map((record) => record.route)).toEqual([
      "/docs/",
      "/docs/drafts/",
    ]);
    expect(recordFor(root, "drafts.md")).toEqual({
      route: "/docs/drafts/",
      source: "drafts.md",
      title: "Drafts",
      canonical: "https://docs.example.com/docs/drafts/",
      listing: "unlisted",
    });
    expect(recordFor(root, "missing.md")).toBeUndefined();
  });

  it("has nothing for a build without the file", () => {
    const root = siteRoot(undefined);
    expect(readRecords(root)).toBeUndefined();
    expect(recordFor(root, "index.md")).toBeUndefined();
  });

  it("reads the file again when a build rewrites it", () => {
    const root = siteRoot(RECORDS);
    expect(recordFor(root, "index.md")?.title).toBe("Loom");
    write(root, {
      ...RECORDS,
      pages: { "/docs/": { source: "index.md", title: "Loom, again", listing: "listed" } },
    });
    const later = new Date(Date.now() + 5_000);
    utimesSync(path.join(root, "_ascribe", "pages.json"), later, later);
    expect(recordFor(root, "index.md")?.title).toBe("Loom, again");
  });

  it("refuses a version it doesn't know", () => {
    const root = siteRoot({ ...RECORDS, version: 2 });
    expect(() => readRecords(root)).toThrow(/version 2 of the page records/);
  });
});

describe("sitemapAddresses", () => {
  const base = {
    routed: () => true,
    sitePages: [] as string[],
    excluded: [] as string[],
    site: "https://docs.example.com",
  };

  it("lists the listed pages of every project, and leaves out one that isn't", () => {
    expect(
      sitemapAddresses({
        ...base,
        projects: [
          [page("/docs/"), page("/docs/drafts/", "unlisted"), page("/docs/quiet/", "unquoted")],
          [page("/")],
        ],
      }),
    ).toEqual([
      "https://docs.example.com/",
      "https://docs.example.com/docs/",
      "https://docs.example.com/docs/quiet/",
    ]);
  });

  it("leaves out a project's pages that have no route in the site", () => {
    expect(
      sitemapAddresses({
        ...base,
        projects: [[page("/docs/")], [page("/demo/")]],
        routed: (route) => route.startsWith("/docs/"),
      }),
    ).toEqual(["https://docs.example.com/docs/"]);
  });

  it("adds the site's own pages, less the excluded routes and the projects' pages", () => {
    expect(
      sitemapAddresses({
        ...base,
        projects: [[page("/docs/"), page("/docs/drafts/", "unlisted")]],
        // Astro built the unlisted page too, and the forwarding page, and the not-found page.
        sitePages: ["/docs/", "/docs/drafts", "/changelog/", "/go/a/", "/404", "/private/"],
        excluded: ["/go/a", "/404", "/private"],
      }),
    ).toEqual(["https://docs.example.com/changelog/", "https://docs.example.com/docs/"]);
  });

  it("leaves out a project's page the site excludes", () => {
    expect(
      sitemapAddresses({ ...base, projects: [[page("/docs/")]], excluded: ["/docs/"] }),
    ).toEqual([]);
  });
});

describe("sitemapXml", () => {
  it("lists each address, escaped, with no dates", () => {
    expect(sitemapXml(["https://docs.example.com/a?b=1&c=2"])).toBe(
      '<?xml version="1.0" encoding="UTF-8"?>\n' +
        '<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">\n' +
        "  <url><loc>https://docs.example.com/a?b=1&amp;c=2</loc></url>\n" +
        "</urlset>\n",
    );
  });
});

describe("robotsText", () => {
  it("opens a group for every crawler, then each project's whole groups, then the sitemap", () => {
    expect(
      robotsText(
        [
          ["Disallow: /docs/drafts/", "User-agent: ExampleBot", "Disallow: /"],
          ["Disallow: /private/"],
        ],
        "https://docs.example.com/sitemap.xml",
      ),
    ).toBe(
      "User-agent: *\nDisallow: /docs/drafts/\nDisallow: /private/\n\n" +
        "User-agent: ExampleBot\nDisallow: /\n\n" +
        "Sitemap: https://docs.example.com/sitemap.xml\n",
    );
  });

  it("is the sitemap's address alone with no lines, and nothing with neither", () => {
    expect(robotsText([[], []], "https://docs.example.com/sitemap.xml")).toBe(
      "Sitemap: https://docs.example.com/sitemap.xml\n",
    );
    expect(robotsText([[], []], undefined)).toBeUndefined();
  });

  it("opens no group for every crawler when every line is in a named one", () => {
    expect(robotsText([["user-agent: ExampleBot", "Disallow: /"]], undefined)).toBe(
      "user-agent: ExampleBot\nDisallow: /\n",
    );
  });
});

describe("rootFiles", () => {
  const sources: RootFileSources = {
    projects: [[page("/docs/")]],
    routed: () => true,
    sitePages: [],
    excluded: [],
    lines: [["Disallow: /docs/drafts/"]],
    site: "https://docs.example.com",
    base: "/",
    ownRobots: false,
    ownSitemap: false,
  };

  it("writes both at the host's root", () => {
    const files = rootFiles(sources);
    expect(files.sitemap).toContain("<loc>https://docs.example.com/docs/</loc>");
    expect(files.robots).toBe(
      "User-agent: *\nDisallow: /docs/drafts/\n\nSitemap: https://docs.example.com/sitemap.xml\n",
    );
    expect(files.notes).toEqual([]);
  });

  it("writes no sitemap without a site, and says why", () => {
    const files = rootFiles({ ...sources, site: undefined });
    expect(files.sitemap).toBeUndefined();
    expect(files.robots).toBe("User-agent: *\nDisallow: /docs/drafts/\n");
    expect(files.notes).toEqual([
      { level: "warn", text: expect.stringContaining("no sitemap.xml") as string },
    ]);
  });

  it("writes no robots.txt when the site has its own, and gives its lines", () => {
    const files = rootFiles({ ...sources, ownRobots: true });
    expect(files.robots).toBeUndefined();
    expect(files.sitemap).toBeDefined();
    expect(files.notes[0]?.text).toContain("public/robots.txt is the site's own");
    expect(files.notes[0]?.text).toContain("Disallow: /docs/drafts/");
  });

  it("writes no sitemap when the site has its own, and robots.txt points to it", () => {
    const files = rootFiles({ ...sources, ownSitemap: true });
    expect(files.sitemap).toBeUndefined();
    expect(files.robots).toContain("Sitemap: https://docs.example.com/sitemap.xml\n");
    expect(files.notes).toEqual([
      { level: "info", text: "no sitemap.xml: public/sitemap.xml is the site's own." },
    ]);
    expect(rootFiles({ ...sources, ownSitemap: true, site: undefined }).notes).toHaveLength(1);
  });

  it("writes the sitemap under a base that isn't /, no robots.txt, and the host's lines", () => {
    const files = rootFiles({ ...sources, base: "/docs" });
    expect(files.robots).toBeUndefined();
    expect(files.sitemap).toBeDefined();
    expect(files.notes[0]?.text).toContain('Astro\'s `base` is "/docs/"');
    expect(files.notes[0]?.text).toContain("Sitemap: https://docs.example.com/docs/sitemap.xml");
  });

  it("says nothing about robots.txt when there'd be nothing in it", () => {
    const files = rootFiles({ ...sources, site: undefined, lines: [[]], ownRobots: true });
    expect(files.robots).toBeUndefined();
    expect(files.notes.map((note) => note.level)).toEqual(["warn"]);
  });
});

describe("the dev server's routes", () => {
  const routes: AstroRoute[] = [
    route("/", /^\/$/, []),
    route("/changelog", /^\/changelog\/?$/, []),
    route("/docs/[...slug]", /^\/docs(?:\/(.*?))?\/?$/, ["...slug"]),
    route("/404", /^\/404\/?$/, []),
    { ...route("/go/[id]", /^\/go\/([^/]+?)\/?$/, ["id"]), origin: "external" },
  ];

  function route(pattern: string, patternRegex: RegExp, params: string[]): AstroRoute {
    return { pattern, patternRegex, params, type: "page", origin: "project" };
  }

  it("names the site's own pages with no parameters", () => {
    expect(devSitePages(routes, { base: "/", trailingSlash: "always" })).toEqual([
      "/",
      "/changelog/",
    ]);
    expect(devSitePages(routes, { base: "/site", trailingSlash: "never" })).toEqual([
      "/site",
      "/site/changelog",
    ]);
  });

  it("knows a project's page by a route that matches it", () => {
    const routed = devRouted(routes, { base: "/", trailingSlash: "always" });
    expect(routed("/docs/guides/install/")).toBe(true);
    expect(routed("/docs")).toBe(true);
    expect(routed("/demo/page/")).toBe(false);
    // Under a base, a route is matched without it.
    const under = devRouted([route("/[...slug]", /^(?:\/(.*?))?\/?$/, ["...slug"])], {
      base: "/docs",
      trailingSlash: "never",
    });
    expect(under("/docs")).toBe(true);
    expect(under("/docs/guides/install")).toBe(true);
    expect(under("/other/page")).toBe(false);
  });
});

describe("rootFilesMiddleware", () => {
  async function get(
    url: string,
    base = "/",
  ): Promise<{ next: boolean; type?: string | undefined; body?: string }> {
    const middleware = rootFilesMiddleware(base, () => ({
      sitemap: "<urlset/>",
      robots: "Sitemap: x\n",
      notes: [],
    }));
    return new Promise((resolve) => {
      const headers: Record<string, string> = {};
      const res = {
        setHeader: (name: string, value: string) => (headers[name] = String(value)),
        end: (body: string) => resolve({ next: false, type: headers["Content-Type"], body }),
      } as unknown as ServerResponse;
      middleware({ method: "GET", url, originalUrl: url } as unknown as IncomingMessage, res, () =>
        resolve({ next: true }),
      );
    });
  }

  it("serves the sitemap at the base and robots.txt at the root", async () => {
    expect(await get("/sitemap.xml")).toEqual({
      next: false,
      type: "application/xml; charset=utf-8",
      body: "<urlset/>",
    });
    expect(await get("/robots.txt?x=1")).toMatchObject({ body: "Sitemap: x\n" });
    expect(await get("/docs/sitemap.xml", "/docs")).toMatchObject({ body: "<urlset/>" });
    expect(await get("/sitemap.xml", "/docs")).toEqual({ next: true });
    expect(await get("/other.xml")).toEqual({ next: true });
  });
});

describe("forwardingRoutes", () => {
  it("is each name's address, at the path of the list the route serves", () => {
    expect(
      forwardingRoutes([
        undefined,
        { version: 1, path: "/docs/go/", permalinks: {} },
        {
          version: 1,
          path: "/go",
          permalinks: { a: { route: "/a/", source: "a.md" } },
          removed: { b: { reason: "Gone." } },
        },
      ]),
    ).toEqual(["/go/a", "/go/b"]);
    expect(forwardingRoutes([undefined])).toEqual([]);
  });
});

describe("routeAs", () => {
  it("ends a route as Astro's trailingSlash says", () => {
    expect(routeAs("/a/", "never")).toBe("/a");
    expect(routeAs("/", "never")).toBe("/");
    expect(routeAs("/a", "always")).toBe("/a/");
    expect(routeAs("/a", "ignore")).toBe("/a");
  });
});
