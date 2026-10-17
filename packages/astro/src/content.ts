// The content collection over a build's site output. Import it from
// `content.config.ts`, which Vite loads, so it can read what the integration
// resolved (`virtual:ascribe/site`) instead of finding the project itself.
//
//   import { defineCollection } from "astro:content";
//   import { ascribeCollection } from "@ascribed/astro/content";
//   import { schema } from "../.ascribe/build/site/site/_ascribe/schema.ts";
//   export const collections = { docs: defineCollection(ascribeCollection({ schema })) };
//
// With several projects, each collection names its entry:
//
//   import { schema as docs } from "../.astro/integrations/_ascribed_astro/docs/schema.ts";
//   export const collections = {
//     docs: defineCollection(ascribeCollection({ schema: docs, name: "docs" })),
//   };
//
// A layout's sidebar, from the project's navigation (`ascribe.nav`'s order,
// or the default), or from a site's own list of pages as `nav`:
//
//   const groups = sidebar(await getCollection("docs"), { entry });
import { readFileSync, statSync } from "node:fs";
import { pathToFileURL } from "node:url";
import path from "node:path";
import { glob, type LoaderContext } from "astro/loaders";
import { astroRoot, collections, routes, siteRoot } from "virtual:ascribe/site";
import { recordFor, type PageRecord } from "./records.js";
import {
  lineNotice,
  linesOf,
  listedLines,
  pageLines,
  type LineNotice,
  type ListedLine,
  type PageLine,
} from "./versions.js";
import {
  navigationGroups,
  pageRoute,
  sidebarGroups,
  type NavEntry,
  type NavGroup,
  type Navigation,
  type SidebarGroup,
} from "./nav.js";

export type {
  Formatted,
  NavGroup,
  NavPage,
  NavSection,
  Navigation,
  SidebarGroup,
  SidebarPage,
} from "./nav.js";
export type { Listing, ListingFrom, PageRecord, Step } from "./records.js";
export type { LineNotice } from "./versions.js";
export { headTags, scriptJson, type Card, type HeadOptions, type HeadTag } from "./head.js";

/**
 * The collection's loader and schema: the `glob` loader on the site output's
 * pages, with the generated Zod schema (`_ascribe/schema.ts`'s `schema`).
 * With several projects (`ascribe({ projects })`), `name` says which entry's
 * site output; it can be left out when there's one entry.
 *
 * Entry ids are Astro's own (`guides/my-setup` for `Guides/My Setup.md`, `index`
 * for the root page), which is what `AstroRouter` in `ascribe-resolve`
 * computes, so a page's route is the base path plus its id, and the root page is
 * at the base path itself.
 *
 * A versioned build's collection has every live line's pages: the current
 * line's with the ids above, and an older line's with the line in front
 * (`3.4/guides/my-setup`, and `3.4` for its root page), so a page's route is
 * still the base path plus its id. Each line's pages are `glob`'s over that
 * line's site output, and each entry's data has `line`, the line it's from,
 * and `current`, whether that's the current line. The schema doesn't type
 * them: `Versions.astro` and `sidebar` read them.
 */
export function ascribeCollection<S>(options: { schema: S; name?: string }): {
  loader: ReturnType<typeof glob>;
  schema: S;
} {
  const root = rootFor(options.name);
  // Whether the build is versioned is known once it's built, so it's read when the collection loads.
  const loader: ReturnType<typeof glob> = {
    name: "@ascribed/astro",
    load: async (context) => {
      const lines = linesOf(root);
      if (lines === undefined) return pagesOf(root).load(context);
      const older = lines.filter((line) => line.status !== "current").map((line) => line.line);
      for (const line of lines) {
        const current = line.status === "current";
        await pagesOf(line.siteRoot).load({
          ...context,
          store: lineStore(context.store, current ? undefined : line.line, older),
          parseData: async (props) => {
            const data: Record<string, unknown> = await context.parseData(props);
            for (const key of ["line", "current"]) {
              if (key in data)
                throw new Error(
                  `@ascribed/astro: ${props.filePath ?? props.id} has a frontmatter field \`${key}\`, and a versioned build's entries have \`${key}\` of their own. Rename the field.`,
                );
            }
            return { ...data, line: line.line, current } as unknown as typeof props.data;
          },
        });
      }
    },
  };
  return { loader, schema: options.schema };
}

/** `glob` over one site output's pages. `_ascribe/` holds the schema and published files, never pages. */
function pagesOf(siteRoot: string): ReturnType<typeof glob> {
  return glob({ pattern: ["**/*.md", "!_ascribe/**"], base: pathToFileURL(siteRoot + path.sep) });
}

/**
 * The collection's store as one line's `glob` sees it: its own entries, by
 * the ids `glob` gives them, which the store keeps with the line in front.
 * `line` is `undefined` for the current line, whose ids are `glob`'s, and
 * whose entries are every one not under an older line, so an entry of a
 * line that's no longer live is removed with the current line's stale ones.
 */
function lineStore(
  store: LoaderContext["store"],
  line: string | undefined,
  older: readonly string[],
): LoaderContext["store"] {
  const under = (id: string, prefix: string) => id === prefix || id.startsWith(`${prefix}/`);
  const outer = (id: string) => (line === undefined ? id : id === "index" ? line : `${line}/${id}`);
  const inner = (id: string) =>
    line === undefined ? id : id === line ? "index" : id.slice(line.length + 1);
  const mine = (id: string) =>
    line === undefined ? !older.some((prefix) => under(id, prefix)) : under(id, line);
  const scoped: Partial<LoaderContext["store"]> = {
    get: (id) => store.get(outer(id)),
    set: (entry) => store.set({ ...entry, id: outer(entry.id) }),
    delete: (id) => store.delete(outer(id)),
    has: (id) => store.has(outer(id)),
    keys: () => store.keys().filter(mine).map(inner),
    values: () => store.entries().flatMap(([id, entry]) => (mine(id) ? [entry] : [])),
    entries: () =>
      store.entries().flatMap(([id, entry]) => (mine(id) ? [[inner(id), entry] as const] : [])),
    clear: () => {
      for (const id of store.keys().filter(mine)) store.delete(id);
    },
  };
  // What `glob` uses beyond these (`addModuleImport`, `addAssetImports`) is by file, not by id.
  return new Proxy(store, {
    get: (target, key, receiver) =>
      key in scoped
        ? scoped[key as keyof typeof scoped]
        : (Reflect.get(target, key, receiver) as unknown),
  });
}

/**
 * A layout's sidebar: the project's navigation, from the build's
 * `_ascribe/nav.json`, each page with its collection entry, its address,
 * which is the one Ascribe's links use, and its label. The root's pages are
 * the first group, "Start", and each top section is a group of its own, with
 * its nested sections in `sections`. `entries` is the collection (`await
 * getCollection("docs")`), and `name` says which project's, as for
 * `ascribeCollection`; with several projects and no `name`, it's the one the
 * entries were loaded from. Throws when the navigation names a page the
 * collection has no entry for.
 *
 * A site's own list, `nav`, overrides the project's: its groups, in its
 * order, checked against the collection. It throws, naming each, when the
 * list leaves out a page of the collection, names a file that isn't one, or
 * names a page twice, so the build fails until the list is complete.
 *
 * With a versioned build, `entry`, the page the sidebar is on, says which
 * line's pages it shows: that line's navigation, or each page of the list
 * that line has, so an older line's sidebar stays in that line. A list is
 * checked against every line's pages. Without `entry`, it shows the current
 * line's.
 */
export function sidebar<E extends NavEntry>(
  entries: readonly E[],
  options: { name?: string; entry?: { data?: unknown }; nav?: readonly NavGroup[] } = {},
): SidebarGroup<E>[] {
  if (!Array.isArray(entries) || (entries.length > 0 && !("id" in (entries[0] as object)))) {
    throw new Error(
      "@ascribed/astro: sidebar() takes the collection first, and a site's own list as an option: `sidebar(entries, { nav })`.",
    );
  }
  const root = sidebarRoot(entries, options.name);
  const at = routes.find((route) => route.siteRoot === root);
  const basePath = at?.basePath ?? "/";
  const trailingSlash = at?.trailingSlash ?? "always";
  const route = (id: string) => pageRoute(id, basePath, trailingSlash);
  const lines = linesOf(root);
  const current = lines?.find((line) => line.status === "current")?.line;
  const shown = lines === undefined ? undefined : (lineOf(options.entry) ?? current);
  const lineRoot = (entry: NavEntry) =>
    lines?.find((line) => line.line === lineOf(entry))?.siteRoot ?? root;
  // An older line's ids have the line in front.
  const id = (entry: NavEntry) => {
    const line = lineOf(entry);
    if (line === undefined || line === current) return entry.id;
    return entry.id === line ? "index" : entry.id.slice(line.length + 1);
  };
  const title = (entry: E) => recordFor(lineRoot(entry), sourceFile(lineRoot(entry), entry))?.title;
  if (options.nav !== undefined) {
    if (lines === undefined)
      return sidebarGroups(
        options.nav,
        entries,
        route,
        (e) => sourceFile(root, e),
        undefined,
        title,
      );
    return sidebarGroups(
      options.nav,
      entries,
      route,
      (entry) => sourceFile(lineRoot(entry), entry),
      { id, shown: (entry) => lineOf(entry) === shown },
      title,
    );
  }
  const shownRoot = lines?.find((line) => line.line === shown)?.siteRoot ?? root;
  const navigation = readNavigation(shownRoot);
  const own = entries.filter((entry) => lines === undefined || lineOf(entry) === shown);
  return navigationGroups(
    navigation,
    own,
    (entry) => route(entry.id),
    (entry) => sourceFile(lineRoot(entry), entry),
  );
}

/** The navigation's format version this reads. */
const NAV_VERSION = 1;

/** A build's navigation by its site output root, as last read, and when its file was written. */
const navigations = new Map<string, { written: number; navigation: Navigation }>();

/**
 * The navigation in a build's site output, `_ascribe/nav.json`. Read again
 * when the file changes, as it does with each build in `astro dev`.
 */
function readNavigation(siteRoot: string): Navigation {
  const file = path.join(siteRoot, "_ascribe", "nav.json");
  let written: number;
  try {
    written = statSync(file).mtimeMs;
  } catch {
    throw new Error(
      `@ascribed/astro: ${file} isn't there. \`ascribe build\` writes it beside the page records; update @ascribed/cli and @ascribed/astro together, or pass the site's own list as \`sidebar(entries, { nav })\`.`,
    );
  }
  const known = navigations.get(siteRoot);
  if (known !== undefined && known.written === written) return known.navigation;
  const navigation = JSON.parse(readFileSync(file, "utf8")) as Navigation;
  if (navigation.version !== NAV_VERSION) {
    throw new Error(
      `@ascribed/astro: ${file} is version ${String(navigation.version)} of the navigation, and this version of the integration reads version ${NAV_VERSION}. Update @ascribed/astro and @ascribed/cli together.`,
    );
  }
  navigations.set(siteRoot, { written, navigation });
  return navigation;
}

/** The line an entry of a versioned build's collection is from, or `undefined`. */
function lineOf(entry: object | undefined): string | undefined {
  const data = (entry as { data?: unknown } | undefined)?.data;
  const line = (data as { line?: unknown } | undefined)?.line;
  return typeof line === "string" ? line : undefined;
}

/**
 * A collection entry's page record, from its build's `_ascribe/pages.json`:
 * its route, title, description, canonical address, and listing, as every
 * output Ascribe writes has them. `undefined` for an entry that isn't from an
 * Ascribe collection, or whose page has no record. An entry of an older line
 * has that line's record.
 */
export function pageRecord(entry: {
  filePath?: string | undefined;
  data?: unknown;
}): PageRecord | undefined {
  const found = entryRoot(entry);
  if (found === undefined) return undefined;
  return recordFor(found.lineRoot, found.source);
}

/** A line that publishes a page: its route there, and whether it's the line the page is from. */
export interface PageVersion extends PageLine {
  /** Whether this is the line of the entry asked about. */
  own: boolean;
}

/**
 * The lines of a versioned build that publish a collection entry's page,
 * newest first, each with the page's route in it and whether it's the
 * entry's own line, from `<output-dir>/_ascribe/versions.json`. A line that
 * doesn't have the page isn't listed. Empty for an entry of a build that
 * isn't versioned. `Versions.astro` writes them as a list of links.
 */
export function pageVersions(entry: {
  filePath?: string | undefined;
  data?: unknown;
}): PageVersion[] {
  const found = entryRoot(entry);
  if (found === undefined) return [];
  const own = lineOf(entry);
  return pageLines(found.root, found.source).map((line) => ({
    ...line,
    own: own === undefined ? line.current : line.line === own,
  }));
}

/** A line of a versioned build, as a page's list of versions shows it. */
export interface LineVersion extends ListedLine {
  /** Whether this is the line of the entry asked about. */
  own: boolean;
}

/**
 * Every line of a versioned build, for the list of versions on a collection
 * entry's page, newest first: each live line, with the page's route in it,
 * or the page in its place, and no route when it has neither; then each
 * archived line that has an address, with that address. Empty for an entry
 * of a build that isn't versioned. `Versions.astro` writes them.
 */
export function lineVersions(entry: {
  filePath?: string | undefined;
  data?: unknown;
}): LineVersion[] {
  const found = entryRoot(entry);
  if (found === undefined) return [];
  const own = lineOf(entry);
  return listedLines(found.root, found.source).map((line) => ({
    ...line,
    own: own === undefined ? line.status === "current" : line.line === own,
  }));
}

/**
 * What a collection entry's page of an older line says about the line: its
 * label and number, the last day it's supported, and the page's route in the
 * current line, or the page that takes its place there, or else the current
 * line's home. `undefined` on the current line, and for an entry of a build
 * that isn't versioned. `VersionBanner.astro` writes it.
 */
export function versionNotice(entry: {
  filePath?: string | undefined;
  data?: unknown;
}): LineNotice | undefined {
  const found = entryRoot(entry);
  const line = lineOf(entry);
  if (found === undefined || line === undefined) return undefined;
  const at = routes.find((route) => route.siteRoot === found.root);
  const home = pageRoute("index", at?.basePath ?? "/", at?.trailingSlash ?? "always");
  return lineNotice(found.root, found.source, line, home);
}

/**
 * The site output root of the entry's build (the current line's, for a
 * versioned build), the root of the line it's from, and its page's source
 * path, or `undefined` for an entry from none of the integration's
 * collections.
 */
function entryRoot(entry: {
  filePath?: string | undefined;
  data?: unknown;
}): { root: string; lineRoot: string; source: string } | undefined {
  if (entry.filePath === undefined) return undefined;
  const file = entry.filePath;
  for (const root of declaredRoots()) {
    const lineRoot = [root, ...(linesOf(root) ?? []).map((line) => line.siteRoot)].find(
      (candidate) => inside(candidate, file),
    );
    if (lineRoot === undefined) continue;
    const data = entry.data as { source?: unknown } | undefined;
    const source =
      typeof data?.source === "string"
        ? data.source
        : path.relative(lineRoot, path.resolve(astroRoot, file)).split(path.sep).join("/");
    return { root, lineRoot, source };
  }
  return undefined;
}

/** The site output roots of the integration's entries. */
function declaredRoots(): string[] {
  return [...Object.values(collections), ...(siteRoot === "" ? [] : [siteRoot])];
}

/**
 * The site output root of the collection a sidebar lists: the entry `name`
 * names, or, with several and no name, the one the entries were loaded from.
 */
function sidebarRoot(entries: readonly NavEntry[], name: string | undefined): string {
  const first = entries[0];
  if (name === undefined && Object.keys(collections).length > 1 && first !== undefined) {
    const from = entryRoot(first)?.root;
    if (from !== undefined) return from;
  }
  return rootFor(name);
}

/** Whether `file`, from the Astro root as an entry's `filePath` is, is under `root`. */
function inside(root: string, file: string): boolean {
  const relative = path.relative(root, path.resolve(astroRoot, file));
  return !relative.startsWith("..") && !path.isAbsolute(relative);
}

/** An entry's source file, by its path in the site output, which mirrors the content root. */
function sourceFile(root: string, entry: NavEntry): string {
  if (entry.filePath === undefined) return `the page ${entry.id}`;
  return inside(root, entry.filePath)
    ? path.relative(root, path.resolve(astroRoot, entry.filePath)).split(path.sep).join("/")
    : entry.filePath;
}

/** The site output root of the entry `name` names. */
function rootFor(name: string | undefined): string {
  const names = Object.keys(collections);
  if (names.length === 0) {
    if (name !== undefined) {
      throw new Error(
        `@ascribed/astro: ascribeCollection was given the name "${name}", but ascribe() was given one project, which has no name. Leave out \`name\`, or list the project in \`projects\`.`,
      );
    }
    return siteRoot;
  }
  if (name === undefined) {
    if (names.length === 1) return collections[names[0] as string] as string;
    throw new Error(
      `@ascribed/astro: ascribeCollection needs a \`name\` when ascribe() has several projects: one of ${names.map((n) => `"${n}"`).join(", ")}.`,
    );
  }
  const root = collections[name];
  if (root === undefined) {
    throw new Error(
      `@ascribed/astro: ascribeCollection was given the name "${name}", and ascribe()'s projects are ${names.map((n) => `"${n}"`).join(", ")}.`,
    );
  }
  return root;
}
