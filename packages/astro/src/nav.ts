// A site's sidebar: its list of pages, which the starter writes as the site's
// `src/nav.ts` (`npx @ascribed/astro init`), checked against the collection.
// Ascribe's content model has no page order or grouping yet, so the site keeps
// the list, and the check keeps it complete: `ascribe.toml` giving a site its
// navigation is the surface audit's finding 28, which has no design yet.
import { slug } from "github-slugger";

/** A group of the sidebar: a label, and its pages' source files, in order. */
export interface NavGroup {
  label: string;
  /** Source files, relative to the content root, as `Guides/My Setup.md`. */
  pages: string[];
}

/** A page of the sidebar: its collection entry, and its address. */
export interface SidebarPage<E> {
  entry: E;
  href: string;
}

/** A group of the sidebar, each of its pages with its entry. */
export interface SidebarGroup<E> {
  label: string;
  pages: SidebarPage<E>[];
}

/** What the check needs of a collection entry: its id, and the file it was loaded from. */
export interface NavEntry {
  id: string;
  filePath?: string | undefined;
}

/**
 * A page's entry id, as Astro's `glob` loader computes it from the page's
 * path in the site output: each segment slugged, without the extension, and
 * without a final `/index`.
 */
export function entryId(pagePath: string): string {
  return pagePath
    .replace(/\.md$/, "")
    .split("/")
    .map((segment) => slug(segment))
    .join("/")
    .replace(/\/index$/, "");
}

/**
 * A page's route, as Ascribe's router writes every link to it: the base path
 * plus its entry id, and the root page (`index`) at the base path itself.
 */
export function pageRoute(id: string, basePath: string, trailingSlash: "always" | "never"): string {
  if (id === "index")
    return trailingSlash === "never" && basePath !== "/" ? basePath.slice(0, -1) : basePath;
  return `${basePath}${id}${trailingSlash === "always" ? "/" : ""}`;
}

/**
 * The ways the list and the collection disagree, as sentences: a page the
 * list leaves out, a file it names that isn't a page, and a page it names twice.
 * `source` names an entry for a person, by its source file where it can.
 */
export function navProblems(
  nav: readonly NavGroup[],
  entries: readonly NavEntry[],
  source: (entry: NavEntry) => string = (entry) => entry.id,
): string[] {
  const listed = nav.flatMap((group) => group.pages);
  const ids = new Set(entries.map((entry) => entry.id));
  const listedIds = new Set(listed.map(entryId));
  const problems: string[] = [];
  for (const entry of entries) {
    if (!listedIds.has(entry.id))
      problems.push(`${source(entry)} isn't in the sidebar's list. Add it to a group.`);
  }
  for (const [i, file] of listed.entries()) {
    if (!ids.has(entryId(file)))
      problems.push(`the sidebar's list names ${file}, which isn't a page of the site.`);
    else if (listed.findIndex((other) => entryId(other) === entryId(file)) !== i)
      problems.push(`the sidebar's list names ${file} twice.`);
  }
  return problems;
}

/**
 * The sidebar's groups, each page with its entry and address. Throws, naming
 * each, when the list leaves out a page of the collection, names a file that
 * isn't one, or names a page twice, so the list stays complete.
 *
 * A versioned build's collection has every line's pages, and the list is
 * checked against them all: `id` gives an entry's id within its line, and
 * the groups have the pages `shown` keeps, one line's, each listed page that
 * line has.
 */
export function sidebarGroups<E extends NavEntry>(
  nav: readonly NavGroup[],
  entries: readonly E[],
  route: (id: string) => string,
  source?: (entry: NavEntry) => string,
  lines: { id: (entry: E) => string; shown: (entry: E) => boolean } = {
    id: (entry) => entry.id,
    shown: () => true,
  },
): SidebarGroup<E>[] {
  // A page in several lines is one page of the list, and one problem.
  const checked = new Map<NavEntry, E>(
    entries.map((entry) => [{ id: lines.id(entry), filePath: entry.filePath }, entry]),
  );
  const problems = navProblems(
    nav,
    [...checked.keys()],
    source === undefined ? undefined : (entry) => source(checked.get(entry) ?? entry),
  ).filter((problem, i, all) => all.indexOf(problem) === i);
  if (problems.length > 0) {
    throw new Error(
      `@ascribed/astro: the sidebar's list (src/nav.ts) doesn't match the site's pages:\n${problems.map((problem) => `- ${problem}`).join("\n")}`,
    );
  }
  const byId = new Map(
    entries.filter((entry) => lines.shown(entry)).map((entry) => [lines.id(entry), entry]),
  );
  return nav.map((group) => ({
    label: group.label,
    pages: group.pages.flatMap((file) => {
      const entry = byId.get(entryId(file));
      return entry === undefined ? [] : [{ entry, href: route(entry.id) }];
    }),
  }));
}

/**
 * The starter's list for a project's pages, by their paths in the site
 * output: the pages at the content root in a group named "Start", the root
 * page first, then a group for each folder, named as the folder is, in order
 * of their names.
 */
export function navFor(pages: readonly string[]): NavGroup[] {
  const groups = new Map<string, string[]>();
  const sorted = pages.toSorted((a, b) => {
    const rootA = a === "index.md";
    const rootB = b === "index.md";
    if (rootA !== rootB) return rootA ? -1 : 1;
    return a < b ? -1 : a > b ? 1 : 0;
  });
  for (const page of sorted) {
    const slash = page.indexOf("/");
    const label = slash === -1 ? "Start" : page.slice(0, slash);
    groups.set(label, [...(groups.get(label) ?? []), page]);
  }
  const start = groups.get("Start");
  groups.delete("Start");
  return [
    ...(start === undefined ? [] : [{ label: "Start", pages: start }]),
    ...[...groups].map(([label, files]) => ({ label, pages: files })),
  ];
}
