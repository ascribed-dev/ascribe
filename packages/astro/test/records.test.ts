import { mkdirSync, mkdtempSync, utimesSync, writeFileSync } from "node:fs";
import type { IncomingMessage, ServerResponse } from "node:http";
import { tmpdir } from "node:os";
import path from "node:path";
import { describe, expect, it } from "vitest";
import { forwardingRoutes } from "../src/permalinks.js";
import {
  pageList,
  purposeConflicts,
  readRecords,
  recordFor,
  robotsText,
  routeAs,
  sitemapAddresses,
  sitemapXml,
  type PageRecord,
  type RobotsGroup,
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
      listing_from: { level: "default" },
    },
    "/docs/drafts/": {
      source: "drafts.md",
      title: "Drafts",
      canonical: "https://docs.example.com/docs/drafts/",
      listing: "unlisted",
      listing_from: { level: "folder", folder: "drafts/", rule: '"drafts/" = "unlisted"', line: 3 },
    },
  },
  robots: { lines: ["Disallow: /docs/private/"] },
};

function page(route: string, listing: PageRecord["listing"] = "listed"): PageRecord {
  return { route, source: `${route.slice(1)}.md`, listing, listing_from: { level: "default" } };
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
      listing_from: { level: "folder", folder: "drafts/", rule: '"drafts/" = "unlisted"', line: 3 },
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
      pages: {
        "/docs/": {
          source: "index.md",
          title: "Loom, again",
          listing: "listed",
          listing_from: { level: "default" },
        },
      },
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

  it("leaves out an older line's page whose canonical address is the current line's", () => {
    const at = (route: string, canonical?: string): PageRecord => ({
      ...page(route),
      ...(canonical === undefined ? {} : { canonical: `https://docs.example.com${canonical}` }),
    });
    expect(
      sitemapAddresses({
        ...base,
        projects: [
          [at("/docs/install", "/docs/install")],
          [at("/docs/3.4/install", "/docs/install"), at("/docs/3.4/old", "/docs/3.4/old")],
          [at("/docs/3.4/bare")],
        ],
      }),
    ).toEqual([
      "https://docs.example.com/docs/3.4/bare",
      "https://docs.example.com/docs/3.4/old",
      "https://docs.example.com/docs/install",
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

/**
 * Whether `agent` may fetch `route` under `robots`, read as RFC 9309 says a
 * crawler reads it: the groups that name the agent, without case, combined,
 * or the `*` groups when none does; the longest matching rule wins, and an
 * `Allow` wins a tie. Paths here are prefixes, without patterns.
 */
function mayFetch(robots: string, agent: string, route: string): boolean {
  const lines = robots.split("\n").map((line) => line.split("#")[0]?.trim() ?? "");
  const groups: { agents: string[]; rules: [boolean, string][] }[] = [];
  let open = false;
  for (const line of lines) {
    const [field = "", ...rest] = line.split(":");
    const value = rest.join(":").trim();
    const name = field.trim().toLowerCase();
    if (name === "user-agent") {
      if (!open) groups.push({ agents: [], rules: [] });
      groups.at(-1)?.agents.push(value.toLowerCase());
      open = true;
    } else {
      open = false;
      if ((name === "allow" || name === "disallow") && value !== "")
        groups.at(-1)?.rules.push([name === "allow", value]);
    }
  }
  const own = groups.filter((group) => group.agents.includes(agent.toLowerCase()));
  const applying = own.length > 0 ? own : groups.filter((group) => group.agents.includes("*"));
  let best: [boolean, string] | undefined;
  for (const rule of applying.flatMap((group) => group.rules)) {
    if (!route.startsWith(rule[1])) continue;
    if (
      best === undefined ||
      rule[1].length > best[1].length ||
      (rule[1].length === best[1].length && rule[0])
    )
      best = rule;
  }
  return best?.[0] ?? true;
}

const TRAINING: RobotsGroup = {
  purpose: "training",
  user_agents: ["GPTBot", "ClaudeBot"],
  lines: ["Disallow: /"],
};
const SEARCH: RobotsGroup = {
  purpose: "search",
  user_agents: ["OAI-SearchBot"],
  lines: ["Disallow: /"],
};
const ON_REQUEST: RobotsGroup = {
  purpose: "on-request",
  user_agents: ["ChatGPT-User"],
  lines: ["Disallow: /"],
};
const SITEMAP = "https://docs.example.com/sitemap.xml";

describe("robotsText", () => {
  it("opens a group for every crawler, then each project's whole groups, then the sitemap", () => {
    expect(
      robotsText(
        [
          { lines: ["Disallow: /docs/drafts/", "User-agent: ExampleBot", "Disallow: /"] },
          { lines: ["Disallow: /private/"] },
        ],
        SITEMAP,
      ),
    ).toBe(
      "User-agent: *\nDisallow: /docs/drafts/\nDisallow: /private/\n\n" +
        "User-agent: ExampleBot\nDisallow: /\n\n" +
        `Sitemap: ${SITEMAP}\n`,
    );
  });

  it("is the sitemap's address alone with no lines, and nothing with neither", () => {
    const none = { lines: [], purposes: {}, groups: [] };
    expect(robotsText([none, none], SITEMAP)).toBe(`Sitemap: ${SITEMAP}\n`);
    expect(robotsText([none, none], undefined)).toBeUndefined();
  });

  it("opens no group for every crawler when every line is in a named one", () => {
    expect(robotsText([{ lines: ["user-agent: ExampleBot", "Disallow: /"] }], undefined)).toBe(
      "user-agent: ExampleBot\nDisallow: /\n",
    );
  });

  it("writes a disallowed purpose's group first, naming each of its tokens, and nothing for one allowed", () => {
    const robots = robotsText(
      [{ lines: [], purposes: { training: "disallow", search: "allow" }, groups: [TRAINING] }],
      SITEMAP,
    );
    expect(robots).toBe(
      `User-agent: GPTBot\nUser-agent: ClaudeBot\nDisallow: /\n\nSitemap: ${SITEMAP}\n`,
    );
  });

  it("writes each purpose once for the site, before the lines for every crawler", () => {
    const robots = robotsText(
      [
        { lines: ["Disallow: /docs/drafts/"], groups: [TRAINING, SEARCH, ON_REQUEST] },
        { lines: ["User-agent: ExampleBot", "Disallow: /"], groups: [TRAINING] },
      ],
      undefined,
    );
    expect(robots).toBe(
      "User-agent: GPTBot\nUser-agent: ClaudeBot\nDisallow: /\n\n" +
        "User-agent: OAI-SearchBot\nDisallow: /\n\n" +
        "User-agent: ChatGPT-User\nDisallow: /\n\n" +
        "User-agent: *\nDisallow: /docs/drafts/\n\n" +
        "User-agent: ExampleBot\nDisallow: /\n",
    );
  });

  it("reads as a crawler of each purpose would", () => {
    // Training is disallowed; search and on-request are allowed, and so stay
    // under the lines for every crawler, the drafts among them.
    const robots =
      robotsText(
        [
          {
            lines: ["Disallow: /docs/drafts/", "User-agent: ExampleBot", "Disallow: /"],
            purposes: { training: "disallow", search: "allow", "on-request": "allow" },
            groups: [TRAINING],
          },
        ],
        SITEMAP,
      ) ?? "";
    expect(mayFetch(robots, "GPTBot", "/docs/")).toBe(false);
    expect(mayFetch(robots, "claudebot", "/docs/guide/")).toBe(false);
    expect(mayFetch(robots, "OAI-SearchBot", "/docs/")).toBe(true);
    expect(mayFetch(robots, "OAI-SearchBot", "/docs/drafts/")).toBe(false);
    expect(mayFetch(robots, "ChatGPT-User", "/docs/")).toBe(true);
    expect(mayFetch(robots, "ChatGPT-User", "/docs/drafts/a/")).toBe(false);
    expect(mayFetch(robots, "Googlebot", "/docs/drafts/")).toBe(false);
    expect(mayFetch(robots, "ExampleBot", "/docs/")).toBe(false);
  });
});

describe("purposeConflicts", () => {
  const project = (name: string, purposes: Record<string, "allow" | "disallow">) => ({
    name,
    robots: { lines: [], purposes },
  });

  it("names both projects when one allows a purpose and another disallows it", () => {
    expect(
      purposeConflicts([
        project("docs", { training: "disallow", search: "allow" }),
        project("site", { training: "allow", search: "allow" }),
      ]),
    ).toEqual([
      '"docs" has [crawlers] training = "disallow" and "site" has training = "allow". The site has one robots.txt, so its projects must agree: make them the same, or remove one.',
    ]);
  });

  it("lets a project that says nothing agree with either", () => {
    expect(
      purposeConflicts([project("docs", { training: "disallow" }), project("site", {})]),
    ).toEqual([]);
  });
});

describe("rootFiles", () => {
  const sources: RootFileSources = {
    projects: [[page("/docs/")]],
    routed: () => true,
    sitePages: [],
    excluded: [],
    robots: [{ name: "docs", robots: { lines: ["Disallow: /docs/drafts/"] } }],
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
    expect(files.problems).toEqual([]);
  });

  it("writes no robots.txt when two projects disagree on a purpose, and says why", () => {
    const files = rootFiles({
      ...sources,
      robots: [
        { name: "docs", robots: { lines: [], purposes: { search: "disallow" }, groups: [SEARCH] } },
        { name: "site", robots: { lines: [], purposes: { search: "allow" } } },
      ],
    });
    expect(files.robots).toBeUndefined();
    expect(files.problems).toHaveLength(1);
    expect(files.problems[0]).toContain('"docs" has [crawlers] search = "disallow"');
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
    const files = rootFiles({
      ...sources,
      site: undefined,
      robots: [{ name: "docs", robots: { lines: [] } }],
      ownRobots: true,
    });
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
      problems: [],
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
