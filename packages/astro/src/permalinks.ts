// Permalinks' addresses: a page at `<path><id>` for each permalink the build
// publishes, which sends the reader on to its heading, and for each retired
// one with nowhere to lead, which says why it was removed. Ascribe writes the list,
// `_ascribe/permalinks.json` in the site output (output-layout contract §1.2);
// the integration adds the route (`Permalink.astro`) and builds its pages
// from it, in `astro build` and `astro dev`. With several projects, one route
// serves every project's list.
import { readFileSync } from "node:fs";
import path from "node:path";
import { normalizeBase } from "./project.js";

/** The list's format version this reads. */
const VERSION = 1;

/** `_ascribe/permalinks.json`. */
export interface PermalinkList {
  version: number;
  /** Where the addresses are published, from the site's root, ending in `/`. */
  path: string;
  /** The content model's dimensions, each with its values: the parameters an address takes. */
  dimensions?: Record<string, string[]>;
  /**
   * Each permalink's destination. An alias names the permalink it leads to,
   * and a retired permalink that leads somewhere its reason.
   */
  permalinks: Record<
    string,
    { route: string; fragment?: string; source: string; alias?: string; retired?: string }
  >;
  /** The retired permalinks with nowhere to lead in this build, each with why. */
  removed?: Record<string, { reason: string }>;
}

/**
 * What an address's page needs: where it sends the reader, and the
 * parameters it takes; or, for a retired permalink with nowhere to lead,
 * why it was removed.
 */
export type Forwarding =
  { to: string; dimensions: Record<string, string[]> } | { reason: string; home?: string };

/** A build's site output whose list the route serves, and the route of its project's root. */
export interface PermalinkSource {
  siteRoot: string;
  home: string;
}

/** The list's text in a build's site output, or `undefined` when there isn't one. */
export function permalinksText(siteRoot: string): string | undefined {
  try {
    return readFileSync(listFile(siteRoot), "utf8");
  } catch {
    return undefined;
  }
}

function listFile(siteRoot: string): string {
  return path.join(siteRoot, "_ascribe", "permalinks.json");
}

/** The list in a build's site output, or `undefined` when there isn't one. */
export function readPermalinks(siteRoot: string): PermalinkList | undefined {
  const file = listFile(siteRoot);
  const text = permalinksText(siteRoot);
  if (text === undefined) return undefined;
  const list = JSON.parse(text) as PermalinkList;
  if (list.version !== VERSION) {
    throw new Error(
      `@ascribed/astro: ${file} is version ${String(list.version)} of the permalinks list, and this version of the integration reads version ${VERSION}. Update @ascribed/astro and @ascribed/cli together.`,
    );
  }
  return list;
}

/**
 * The route pattern for the addresses, as Astro takes it: under `base`, which
 * Astro adds. Throws when the list's path isn't under `base`, since an Astro
 * site serves nothing outside it.
 */
export function permalinkPattern(listPath: string, base: string): string {
  const root = normalizeBase(base);
  const at = normalizeBase(listPath);
  if (!at.startsWith(root)) {
    throw new Error(
      `@ascribed/astro: permalinks are published at "${at}", which isn't under Astro's \`base\`, "${base}", and an Astro site serves nothing outside it. Set [consumer] permalink-path in ascribe.toml to a path under "${root}".`,
    );
  }
  return `/${at.slice(root.length)}[id]`;
}

/** Where a permalink leads: its page's route, and the heading's anchor. */
function destination(entry: { route: string; fragment?: string }): string {
  return entry.fragment === undefined ? entry.route : `${entry.route}#${entry.fragment}`;
}

/**
 * `getStaticPaths` for the route: one page for each permalink in the site
 * outputs' lists, and one for each retired permalink they list as removed,
 * whose page links to its project's root. A name in more than one list gets
 * the first's page; the integration refuses one that two projects declare.
 */
export function forwardingPaths(
  sources: string | readonly PermalinkSource[],
): { params: { id: string }; props: Forwarding }[] {
  const all = typeof sources === "string" ? [{ siteRoot: sources, home: undefined }] : sources;
  const pages = new Map<string, Forwarding>();
  for (const { siteRoot, home } of all) {
    const list = readPermalinks(siteRoot);
    if (list === undefined) continue;
    const dimensions = list.dimensions ?? {};
    for (const [id, entry] of Object.entries(list.permalinks)) {
      if (!pages.has(id)) pages.set(id, { to: destination(entry), dimensions });
    }
    for (const [id, entry] of Object.entries(list.removed ?? {})) {
      if (!pages.has(id))
        pages.set(
          id,
          home === undefined ? { reason: entry.reason } : { reason: entry.reason, home },
        );
    }
  }
  const forwarding = [...pages].filter(([, props]) => "to" in props);
  const removed = [...pages].filter(([, props]) => !("to" in props));
  return [...forwarding, ...removed].map(([id, props]) => ({ params: { id }, props }));
}

/** Whether a list names anything: a permalink, or a retired one it lists as removed. */
export function hasPermalinks(list: PermalinkList): boolean {
  return Object.keys(list.permalinks).length + Object.keys(list.removed ?? {}).length > 0;
}

/**
 * The list a site's not-found page explains addresses from, and the route of
 * its project's root: the first that has a permalink, or else the first list.
 */
export function servedList(
  sources: readonly PermalinkSource[],
): { path: string; home: string } | undefined {
  let first: { path: string; home: string } | undefined;
  for (const { siteRoot, home } of sources) {
    const list = readPermalinks(siteRoot);
    if (list === undefined) continue;
    first ??= { path: list.path, home };
    if (hasPermalinks(list)) return { path: list.path, home };
  }
  return first;
}

/**
 * The routes of the forwarding pages, from the host's root: one for each name
 * in `lists`, at the path of the list the route serves, as `hasPermalinks`
 * picks it. They aren't pages of a build, and no sitemap lists them.
 */
export function forwardingRoutes(lists: readonly (PermalinkList | undefined)[]): string[] {
  const known = lists.filter((list) => list !== undefined);
  const served = known.find(hasPermalinks) ?? known[0];
  if (served === undefined) return [];
  const at = normalizeBase(served.path);
  return known.flatMap((list) =>
    [...Object.keys(list.permalinks), ...Object.keys(list.removed ?? {})].map((id) => `${at}${id}`),
  );
}
