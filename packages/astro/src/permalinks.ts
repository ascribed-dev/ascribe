// Permalinks' addresses: a page at `<path><id>` for each permalink the build
// publishes, which sends the reader on to its heading. Ascribe writes the list,
// `_ascribe/permalinks.json` in the site output (output-layout contract §1.2);
// the integration adds the route (`Permalink.astro`) and builds its pages
// from it, in `astro build` and `astro dev`.
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
  permalinks: Record<string, { route: string; fragment?: string; source: string }>;
}

/** What a forwarding page needs: where it sends the reader. */
export interface Forwarding {
  to: string;
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

/** `getStaticPaths` for the route: one page for each permalink in the site output's list. */
export function forwardingPaths(siteRoot: string): { params: { id: string }; props: Forwarding }[] {
  const list = readPermalinks(siteRoot);
  if (list === undefined) return [];
  return Object.entries(list.permalinks).map(([id, entry]) => ({
    params: { id },
    props: { to: destination(entry) },
  }));
}
