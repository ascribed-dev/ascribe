// The redirects a build writes for the host: with `[consumer] host =
// "netlify"`, the compiler writes Netlify's `_redirects` in each build's site
// output, at `_ascribe/host/_redirects` (output-layout contract §1.9), from
// the moves `ascribe.addresses` records. `astro build` merges every
// project's into the site's `_redirects`, at the root of what's deployed.
//
// A site's own `public/_redirects` is kept, after Ascribe's rules: Netlify
// uses the first rule that matches, so a page's recorded move wins over a
// pattern of the site's.
import { readFileSync } from "node:fs";
import path from "node:path";
import { normalizeBase } from "./project.js";
import { routeKey } from "./records.js";

/** One redirect: an old address, from the host's root, to where it leads now. */
export interface Redirect {
  from: string;
  to: string;
  status: number;
}

/**
 * The redirects in the host's `_redirects` the compiler wrote in a build's
 * site output, in its order: none when it has none, as without `[consumer]
 * host`.
 */
export function readHostRedirects(siteRoot: string): Redirect[] {
  let text: string;
  try {
    text = readFileSync(path.join(siteRoot, "_ascribe", "host", "_redirects"), "utf8");
  } catch {
    return [];
  }
  const out: Redirect[] = [];
  for (const line of text.split(/\r?\n/)) {
    const trimmed = line.trim();
    if (trimmed === "" || trimmed.startsWith("#")) continue;
    const [from, to, status] = trimmed.split(/\s+/);
    if (from === undefined || to === undefined) continue;
    out.push({ from, to, status: status === undefined ? 301 : Number.parseInt(status, 10) });
  }
  return out;
}

/**
 * Every project's redirects as one list, in the order given: an address two
 * projects both redirect keeps the first's, as the first entry's files win.
 */
export function mergeRedirects(projects: readonly (readonly Redirect[])[]): Redirect[] {
  const seen = new Set<string>();
  const out: Redirect[] = [];
  for (const list of projects) {
    for (const redirect of list) {
      if (seen.has(redirect.from)) continue;
      seen.add(redirect.from);
      out.push(redirect);
    }
  }
  return out;
}

/** The redirects in Netlify's `_redirects` format: each on a line, `from`, `to`, and the status. */
export function netlifyRedirects(redirects: readonly Redirect[]): string {
  return redirects.map((r) => `${r.from} ${r.to} ${String(r.status)}\n`).join("");
}

/** What the build does about the host's redirects. */
export type RedirectsOutcome =
  | { kind: "none" }
  | { kind: "write"; text: string }
  | { kind: "note"; text: string }
  | { kind: "fail"; text: string };

/** What the build does about `_redirects`, given every project's redirects and the site. */
export function redirectsOutcome(sources: {
  /** The redirects of every project that sets `[consumer] host = "netlify"`, merged. */
  redirects: readonly Redirect[];
  /** Astro's `base`. */
  base: string;
  /** Whether a route is a page the site publishes, which a redirect mustn't hide. */
  published: (route: string) => boolean;
  /** The site's own `public/_redirects`, when it has one. */
  own: string | undefined;
}): RedirectsOutcome {
  if (sources.redirects.length === 0) return { kind: "none" };
  const text = netlifyRedirects(sources.redirects);
  const base = normalizeBase(sources.base);
  if (base !== "/") {
    return {
      kind: "note",
      text: `no _redirects: Astro's \`base\` is "${base}", and Netlify reads _redirects only at the root of what's deployed. Put these lines in the host's own:\n${text}`,
    };
  }
  const taken = sources.redirects.filter((r) => sources.published(routeKey(r.from)));
  if (taken.length > 0) {
    return {
      kind: "fail",
      text: `a page the site publishes is at an address ascribe.addresses forwards somewhere else, so its redirect would hide the page or never be used. Remove the entry, or move the page:\n${netlifyRedirects(taken)}`,
    };
  }
  if (sources.own === undefined) return { kind: "write", text };
  const own = sources.own.endsWith("\n") || sources.own === "" ? sources.own : `${sources.own}\n`;
  return {
    kind: "write",
    text: `${text}\n# The site's own, from public/_redirects: after Ascribe's, since the first rule that matches is used.\n${own}`,
  };
}

/** The folder a retired address's page is in, under its build's base path. */
const GONE_FOLDER = "/_ascribe/gone/";

/** The format version of `_ascribe/redirects.json` this reads. */
const REDIRECTS_VERSION = 1;

/** A retired address, as a build's `_ascribe/redirects.json` lists it (output-layout contract §1.8). */
interface Gone {
  /** The address, from the host's root. */
  path: string;
  /** Why it was retired, as its authors wrote it. */
  reason: string;
  /** The `ascribe.addresses` key it comes from. */
  entry: string;
  /** Where its page is, from the host's root: the base path, `_ascribe/gone/`, and the address from there. */
  page: string;
}

/** What a retired address's page says: why, and where the docs' home is. */
export interface GonePage {
  reason: string;
  home: string;
}

/** The retired addresses a build's site output lists: none when it has no `_ascribe/redirects.json`. */
function readGone(siteRoot: string): Gone[] {
  const file = path.join(siteRoot, "_ascribe", "redirects.json");
  let text: string;
  try {
    text = readFileSync(file, "utf8");
  } catch {
    return [];
  }
  const list = JSON.parse(text) as { version: number; gone?: Gone[] };
  if (list.version !== REDIRECTS_VERSION) {
    throw new Error(
      `@ascribed/astro: ${file} is version ${String(list.version)} of the redirects list, and this version of the integration reads version ${String(REDIRECTS_VERSION)}. Update @ascribed/astro and @ascribed/cli together.`,
    );
  }
  return list.gone ?? [];
}

/**
 * Each retired address's page the site outputs list, by its route from the
 * host's root, with its folder (its build's base path and `_ascribe/gone/`),
 * its address from there, and what it says. A page two projects both list
 * keeps the first's.
 */
function gonePages(
  sources: readonly { siteRoot: string; home: string }[],
): Map<string, { folder: string; address: string; props: GonePage }> {
  const pages = new Map<string, { folder: string; address: string; props: GonePage }>();
  for (const { siteRoot, home } of sources) {
    for (const gone of readGone(siteRoot)) {
      const at = gone.page.indexOf(GONE_FOLDER);
      if (at < 0) continue;
      const folder = gone.page.slice(0, at + GONE_FOLDER.length);
      const address = gone.page.slice(folder.length).replace(/\/+$/, "");
      if (address === "") continue;
      const route = `${folder}${address}`;
      if (!pages.has(route))
        pages.set(route, { folder, address, props: { reason: gone.reason, home } });
    }
  }
  return pages;
}

/**
 * The route pattern for the pages in `folder`, as Astro takes it: under
 * `base`, which Astro adds. Throws when the folder isn't under `base`, since
 * an Astro site serves nothing outside it.
 */
function gonePattern(folder: string, base: string): string {
  const root = normalizeBase(base);
  if (!folder.startsWith(root)) {
    throw new Error(
      `@ascribed/astro: retired addresses' pages are published at "${folder}", which isn't under Astro's \`base\`, "${base}", and an Astro site serves nothing outside it. Set [consumer] base-path in ascribe.toml to a path under "${root}".`,
    );
  }
  return `/${folder.slice(root.length)}[...address]`;
}

/**
 * The route patterns the retired addresses' pages need, one for each
 * folder that has any, under `base`.
 */
export function gonePatterns(siteRoots: readonly string[], base: string): string[] {
  const pages = gonePages(siteRoots.map((siteRoot) => ({ siteRoot, home: "/" })));
  return [...new Set([...pages.values()].map(({ folder }) => gonePattern(folder, base)))];
}

/**
 * `getStaticPaths` for the route at `routePattern`, one of `gonePatterns`:
 * one page for each retired address's page in its folder, with the reason
 * and the route of its project's root.
 */
export function gonePaths(
  sources: readonly { siteRoot: string; home: string }[],
  base: string,
  routePattern: string,
): { params: { address: string }; props: GonePage }[] {
  return [...gonePages(sources).values()]
    .filter(({ folder }) => gonePattern(folder, base) === routePattern)
    .map(({ address, props }) => ({ params: { address }, props }));
}

/** The routes of the retired addresses' pages the site outputs at `siteRoots` list, from the host's root. */
export function goneRoutes(siteRoots: readonly string[]): string[] {
  return [...gonePages(siteRoots.map((siteRoot) => ({ siteRoot, home: "/" }))).keys()];
}
