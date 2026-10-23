// Each permalink's help at an address of its own, for a product to fetch its
// line's: one entry of a build's `_ascribe/help.json` (output-layout contract
// §1.7), as JSON. The current line's entries are beside its permalinks'
// addresses, at `<path><name>.json`; an older line's are at `go/<name>.json`
// under the line's own base path, `/docs/3.4/go/connect.json`, since the
// permalinks' addresses are the current line's. The integration adds a route
// (`help-route.ts`) for each of those folders, in `astro build` and `astro dev`.
import { readFileSync } from "node:fs";
import path from "node:path";
import { permalinkPattern, readPermalinks } from "./permalinks.js";
import { normalizeBase } from "./project.js";
import { olderLines } from "./versions.js";

/** The file's format version this reads. */
const VERSION = 1;

/** A permalink's section, as `_ascribe/help.json` has it. */
export interface HelpEntry {
  title: string;
  summary?: string;
  text: string;
  address?: string;
  source: string;
}

/** `_ascribe/help.json`. */
export interface HelpFile {
  version: number;
  /** The build's name, as `ascribe.toml` declares it. */
  build: string;
  /** For a line of a versioned build, the line. */
  line?: string;
  help: Record<string, HelpEntry>;
}

/** The file in a build's site output, or `undefined` when there isn't one. */
export function readHelp(siteRoot: string): HelpFile | undefined {
  const file = path.join(siteRoot, "_ascribe", "help.json");
  let text: string;
  try {
    text = readFileSync(file, "utf8");
  } catch {
    return undefined;
  }
  const help = JSON.parse(text) as HelpFile;
  if (help.version !== VERSION) {
    throw new Error(
      `@ascribed/astro: ${file} is version ${String(help.version)} of the help file, and this version of the integration reads version ${VERSION}. Update @ascribed/astro and @ascribed/cli together.`,
    );
  }
  return help;
}

/** A folder of entries' addresses, from the host's root, and the file they're served from. */
interface HelpFolder {
  folder: string;
  help: HelpFile;
}

/**
 * Each line's file, with the folder its entries are served under: the
 * current line's at the permalinks' path, each older line's at `go/` under
 * its base path. A site output with no file, or no permalinks list, has none.
 */
function helpFolders(siteRoots: readonly string[]): HelpFolder[] {
  return siteRoots.flatMap((siteRoot) => {
    const list = readPermalinks(siteRoot);
    const help = readHelp(siteRoot);
    const current =
      list === undefined || help === undefined ? [] : [{ folder: normalizeBase(list.path), help }];
    const older = olderLines(siteRoot).flatMap((line) => {
      const lineHelp = readHelp(line.siteRoot);
      return lineHelp === undefined
        ? []
        : [{ folder: `${normalizeBase(line.base)}go/`, help: lineHelp }];
    });
    return [...current, ...older];
  });
}

/**
 * What each entry's address answers, by the address from the host's root:
 * the file it's in with that entry alone, its `version`, `build`, and
 * `line` as the file has them, so a product reads a fetched entry and its
 * bundled copy alike. An address two files give keeps the first's.
 */
export function helpAddresses(siteRoots: readonly string[]): Map<string, HelpFile> {
  const addresses = new Map<string, HelpFile>();
  for (const { folder, help } of helpFolders(siteRoots)) {
    for (const [name, entry] of Object.entries(help.help)) {
      const address = `${folder}${name}.json`;
      if (addresses.has(address)) continue;
      addresses.set(address, {
        version: help.version,
        build: help.build,
        ...(help.line === undefined ? {} : { line: help.line }),
        help: { [name]: entry },
      });
    }
  }
  return addresses;
}

/** The route pattern for a folder's entries, as Astro takes it: under `base`, which Astro adds. */
function helpPattern(folder: string, base: string): string {
  return `${permalinkPattern(folder, base)}.json`;
}

/**
 * The route patterns the entries need, one for each folder that has any:
 * the current line's, then each older line's. Throws when a folder isn't
 * under `base`, since an Astro site serves nothing outside it.
 */
export function helpPatterns(siteRoots: readonly string[], base: string): string[] {
  const patterns = helpFolders(siteRoots)
    .filter(({ help }) => Object.keys(help.help).length > 0)
    .map(({ folder }) => helpPattern(folder, base));
  return [...new Set(patterns)];
}

/**
 * `getStaticPaths` for the route at `routePattern`, one of `helpPatterns`:
 * one answer for each entry whose address it matches, the entry as JSON.
 */
export function helpPaths(
  siteRoots: readonly string[],
  base: string,
  routePattern: string,
): { params: { id: string }; props: { body: string } }[] {
  return [...helpAddresses(siteRoots)].flatMap(([address, help]) => {
    const name = path.posix.basename(address, ".json");
    const folder = address.slice(0, address.length - `${name}.json`.length);
    return helpPattern(folder, base) === routePattern
      ? [{ params: { id: name }, props: { body: `${JSON.stringify(help, null, 2)}\n` } }]
      : [];
  });
}
