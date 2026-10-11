// The site's sitemap and `robots.txt`, from every project's page records
// together: one of each for the site, at the host's root, which is above the
// base path of every project that isn't mounted there. `astro build` writes
// them into its output, and `astro dev` serves them.
//
// A site whose Astro `base` isn't `/` has no host root: its sitemap goes
// under the base, and no `robots.txt` is written, since one under a path does
// nothing. The build says what the host's own needs instead.
import type { IncomingMessage, ServerResponse } from "node:http";
import { normalizeBase } from "./project.js";
import {
  robotsText,
  routeAs,
  routeKey,
  sitemapAddresses,
  sitemapRoute,
  sitemapXml,
  type SitemapSources,
} from "./records.js";

/** What the files are drawn from. */
export interface RootFileSources extends Omit<SitemapSources, "site"> {
  /** Each project's `robots.txt` lines, from the host's root, each project once. */
  lines: readonly (readonly string[])[];
  /** Astro's `site`, when it's set. */
  site: string | undefined;
  /** Astro's `base`. */
  base: string;
  /** Whether the site has its own `robots.txt`, in `public/`. */
  ownRobots: boolean;
}

/** A thing the build says about the files, and how loudly. */
interface RootFileNote {
  level: "info" | "warn";
  text: string;
}

/** The files, each `undefined` when none is written, and what to say about them. */
export interface RootFiles {
  sitemap: string | undefined;
  robots: string | undefined;
  notes: RootFileNote[];
}

/** The sitemap and `robots.txt` for a site, and what the build says about them. */
export function rootFiles(sources: RootFileSources): RootFiles {
  const notes: RootFileNote[] = [];
  const base = normalizeBase(sources.base);
  let sitemap: string | undefined;
  let sitemapAddress: string | undefined;
  if (sources.site === undefined) {
    notes.push({
      level: "warn",
      text: "no sitemap.xml: a sitemap lists pages by their absolute addresses, and the site has no address. Set [consumer] site in ascribe.toml, or Astro's `site`.",
    });
  } else {
    sitemap = sitemapXml(sitemapAddresses({ ...sources, site: sources.site }));
    sitemapAddress = new URL(sitemapRoute(base), sources.site).href;
  }
  let robots = robotsText(sources.lines, sitemapAddress);
  if (robots !== undefined && base !== "/") {
    notes.push({
      level: "info",
      text: `no robots.txt: Astro's \`base\` is "${base}", and a crawler reads robots.txt only at the host's root. Put these lines in the host's own:\n${robots}`,
    });
    robots = undefined;
  } else if (robots !== undefined && sources.ownRobots) {
    notes.push({
      level: "info",
      text: `no robots.txt: public/robots.txt is the site's own. Ascribe's lines for it:\n${robots}`,
    });
    robots = undefined;
  }
  return { sitemap, robots, notes };
}

type Middleware = (req: IncomingMessage, res: ServerResponse, next: () => void) => void;

/**
 * A dev-server middleware serving the sitemap at `<base>sitemap.xml` and
 * `robots.txt` at `/`, each as `files` has it when it's asked for, so each
 * rebuild's records are served.
 */
export function rootFilesMiddleware(base: string, files: () => RootFiles): Middleware {
  const sitemapPath = sitemapRoute(base);
  return (req, res, next) => {
    if (req.method !== "GET" && req.method !== "HEAD") return next();
    // Vite strips the base from `req.url`; connect keeps the original in `originalUrl`.
    const pathname =
      ((req as { originalUrl?: string }).originalUrl ?? req.url ?? "").split(/[?#]/, 1)[0] ?? "";
    if (pathname !== sitemapPath && pathname !== "/robots.txt") return next();
    const current = files();
    const [text, type] =
      pathname === sitemapPath
        ? [current.sitemap, "application/xml; charset=utf-8"]
        : [current.robots, "text/plain; charset=utf-8"];
    if (text === undefined) return next();
    res.setHeader("Content-Type", type);
    res.setHeader("Content-Length", Buffer.byteLength(text));
    res.end(req.method === "HEAD" ? undefined : text);
  };
}

/** A route as `astro:routes:resolved` gives it, as much as these read. */
export interface AstroRoute {
  pattern: string;
  patternRegex: RegExp;
  params: string[];
  type: string;
  origin: string;
}

/** Astro's routing, as much as these read. */
interface Routing {
  base: string;
  trailingSlash: "always" | "never" | "ignore";
}

/**
 * The site's own pages `astro dev` can name: each route of the site's own
 * with no parameters, other than the not-found and error pages, by its route
 * from the host's root. A page a route's `getStaticPaths` lists is known only
 * to a build.
 */
export function devSitePages(routes: readonly AstroRoute[], astro: Routing): string[] {
  const root = normalizeBase(astro.base);
  return routes
    .filter(
      (route) =>
        route.type === "page" &&
        route.origin === "project" &&
        route.params.length === 0 &&
        route.pattern !== "/404" &&
        route.pattern !== "/500",
    )
    .map((route) => routeAs(`${root}${route.pattern.replace(/^\//, "")}`, astro.trailingSlash));
}

/**
 * Whether a project's page, by its route from the host's root, has a route
 * in the site in `astro dev`: whether one of the site's own routes matches
 * it. A project no route shows, such as one whose collection a page only
 * quotes from, has no pages in the sitemap.
 */
export function devRouted(
  routes: readonly AstroRoute[],
  astro: Routing,
): (route: string) => boolean {
  const root = normalizeBase(astro.base);
  const pages = routes.filter((route) => route.type === "page" && route.origin === "project");
  return (route) => {
    if (!`${route}/`.startsWith(root)) return false;
    const under = `/${route.slice(root.length)}`;
    const forms = new Set([under, routeKey(under), `${routeKey(under)}/`]);
    return pages.some((page) => [...forms].some((form) => page.patternRegex.test(form)));
  };
}
