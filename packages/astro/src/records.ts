// The page records: `_ascribe/pages.json` in a build's site output
// (output-layout contract §1.3), what each page tells every reader that isn't
// a person. The integration writes from them a page's head (`Head.astro`),
// the site's one sitemap, and its one `robots.txt`, from every project's
// records together, at the host's root.
import { readFileSync, statSync } from "node:fs";
import path from "node:path";
import { normalizeBase } from "./project.js";

/** The records' format version this reads. */
const VERSION = 1;

/** How a page is treated: listed, listed and not to be quoted, or not listed. */
export type Listing = "listed" | "unquoted" | "unlisted";

/**
 * Where a page's listing comes from: `default` when nothing sets it, `page`
 * for the page's own, and for a rule of `[listing]` its level (`site`,
 * `folder`, or `type`), its folder or type, the rule as written, and its line
 * in `ascribe.toml`.
 */
export interface ListingFrom {
  level: "default" | "page" | "site" | "folder" | "type";
  folder?: string;
  type?: string;
  rule?: string;
  line?: number;
}

/** One page's record, with the route it's keyed by. */
export interface PageRecord {
  /** The page's route, from the host's root. */
  route: string;
  /** The page's path from the content root, with `/`. */
  source: string;
  /** The page's title, as plain text. */
  title?: string;
  /** The page's description, as plain text on one line. */
  description?: string;
  /** The page's own address, absolute, when the project sets `[consumer] site`. */
  canonical?: string;
  listing: Listing;
  /** Where `listing` comes from: the page, a rule of `[listing]`, or neither. */
  listing_from: ListingFrom;
  /** The address of the page's Markdown version, from the host's root, when the build publishes one. */
  markdown?: string;
  /**
   * The page's place in the site, from the top down to the page itself: the
   * page at each of its route's folders that has one, then the page. Absent
   * in records an earlier Ascribe wrote.
   */
  breadcrumbs?: Step[];
}

/** One step of a page's breadcrumbs: a page's route, from the host's root, and its title. */
export interface Step {
  route: string;
  title: string;
}

/** `_ascribe/pages.json`. */
export interface PageRecords {
  version: number;
  site?: string;
  pages: Record<string, Omit<PageRecord, "route">>;
  robots: { lines: string[] };
}

function recordsFile(siteRoot: string): string {
  return path.join(siteRoot, "_ascribe", "pages.json");
}

/** The records in a build's site output, or `undefined` when there aren't any. */
export function readRecords(siteRoot: string): PageRecords | undefined {
  const file = recordsFile(siteRoot);
  let text: string;
  try {
    text = readFileSync(file, "utf8");
  } catch {
    return undefined;
  }
  const records = JSON.parse(text) as PageRecords;
  if (records.version !== VERSION) {
    throw new Error(
      `@ascribed/astro: ${file} is version ${String(records.version)} of the page records, and this version of the integration reads version ${VERSION}. Update @ascribed/astro and @ascribed/cli together.`,
    );
  }
  return records;
}

/** A build's records by source path, as last read, and when its file was written. */
const bySource = new Map<string, { written: number; pages: Map<string, PageRecord> }>();

/**
 * The record of the page whose source path is `source`, in the build whose
 * site output is at `siteRoot`, or `undefined` when it has none. Read again
 * when the file changes, as it does with each build in `astro dev`.
 */
export function recordFor(siteRoot: string, source: string): PageRecord | undefined {
  let written: number;
  try {
    written = statSync(recordsFile(siteRoot)).mtimeMs;
  } catch {
    return undefined;
  }
  let known = bySource.get(siteRoot);
  if (known === undefined || known.written !== written) {
    const pages = new Map<string, PageRecord>();
    for (const record of pageList(readRecords(siteRoot))) {
      if (!pages.has(record.source)) pages.set(record.source, record);
    }
    known = { written, pages };
    bySource.set(siteRoot, known);
  }
  return known.pages.get(source);
}

/** The records' pages, each with its route, in the file's order. */
export function pageList(records: PageRecords | undefined): PageRecord[] {
  return Object.entries(records?.pages ?? {}).map(([route, page]) => ({ route, ...page }));
}

/**
 * A route as the sitemap compares it: with a leading `/` and no trailing one,
 * except at the root, so `/docs/` and `/docs` are one page.
 */
export function routeKey(route: string): string {
  const withSlash = route.startsWith("/") ? route : `/${route}`;
  return withSlash.length > 1 && withSlash.endsWith("/") ? withSlash.slice(0, -1) : withSlash;
}

/** A route ending the way Astro's `trailingSlash` says, as a built page's address. */
export function routeAs(route: string, trailingSlash: "always" | "never" | "ignore"): string {
  if (trailingSlash === "never") return routeKey(route);
  if (trailingSlash === "always" && !route.endsWith("/")) return `${route}/`;
  return route;
}

/** What the sitemap is drawn from. */
export interface SitemapSources {
  /** Every project's records. */
  projects: readonly PageRecord[][];
  /**
   * Whether a project's page has a route in the site: was built, or, in
   * `astro dev`, matches one of the site's routes.
   */
  routed: (route: string) => boolean;
  /** The site's own pages, each by its route from the host's root. */
  sitePages: readonly string[];
  /** Routes from the host's root to leave out: forwarding pages, the not-found page, and the site's list. */
  excluded: readonly string[];
  /** Astro's `site`. */
  site: string;
}

/**
 * The sitemap's addresses, absolute, sorted, each once: every listed page of
 * every project that has a route in the site, and every page of the site's
 * own, less the excluded routes. A project's page that isn't listed is left
 * out, and so is a page of the site's own at the same route as one.
 */
export function sitemapAddresses(sources: SitemapSources): string[] {
  const excluded = new Set(sources.excluded.map(routeKey));
  const records = new Set<string>();
  const routes = new Map<string, string>();
  for (const pages of sources.projects) {
    for (const page of pages) {
      const key = routeKey(page.route);
      records.add(key);
      if (page.listing === "unlisted" || excluded.has(key) || !sources.routed(page.route)) continue;
      if (!routes.has(key)) routes.set(key, page.route);
    }
  }
  for (const route of sources.sitePages) {
    const key = routeKey(route);
    if (records.has(key) || excluded.has(key) || routes.has(key)) continue;
    routes.set(key, route);
  }
  return [
    ...new Set([...routes.values()].map((route) => new URL(route, sources.site).href)),
  ].sort();
}

function escapeXml(text: string): string {
  return text
    .replaceAll("&", "&amp;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;")
    .replaceAll('"', "&quot;")
    .replaceAll("'", "&apos;");
}

/** A sitemap of `addresses`, with no dates. */
export function sitemapXml(addresses: readonly string[]): string {
  const urls = addresses.map((address) => `  <url><loc>${escapeXml(address)}</loc></url>\n`);
  return (
    '<?xml version="1.0" encoding="UTF-8"?>\n' +
    '<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">\n' +
    urls.join("") +
    "</urlset>\n"
  );
}

const USER_AGENT = /^\s*user-agent\s*:/i;

/**
 * A site's `robots.txt` from each project's lines: a `User-agent: *` group
 * with every project's lines before its first `User-agent` line, which are
 * for every crawler, then the groups each project wrote whole, then the
 * sitemap's address. `undefined` when it would say nothing.
 */
export function robotsText(
  projects: readonly (readonly string[])[],
  sitemap: string | undefined,
): string | undefined {
  const everyone: string[] = [];
  const groups: string[][] = [];
  for (const lines of projects) {
    const first = lines.findIndex((line) => USER_AGENT.test(line));
    everyone.push(...(first === -1 ? lines : lines.slice(0, first)));
    if (first !== -1) groups.push(lines.slice(first) as string[]);
  }
  const blocks: string[][] = [];
  if (everyone.length > 0) blocks.push(["User-agent: *", ...everyone]);
  blocks.push(...groups);
  if (sitemap !== undefined) blocks.push([`Sitemap: ${sitemap}`]);
  if (blocks.length === 0) return undefined;
  return `${blocks.map((block) => block.join("\n")).join("\n\n")}\n`;
}

/** Where the sitemap is, from the host's root: at Astro's `base`. */
export function sitemapRoute(base: string): string {
  return `${normalizeBase(base)}sitemap.xml`;
}
