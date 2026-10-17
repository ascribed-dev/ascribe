// A site's sidebar: the project's navigation, `_ascribe/nav.json` in a
// build's site output (the navigation contract), joined to the collection;
// or a site's own list of pages, which the starter writes as the site's
// `src/nav.ts` (`npx @ascribed/astro init`), checked against the collection.
import { slug } from "github-slugger";

/** A group of the sidebar: a label, and its pages' source files, in order. */
export interface NavGroup {
  label: string;
  /** Source files, relative to the content root, as `Guides/My Setup.md`. */
  pages: string[];
}

/** A label in the site output's formatted form: HTML, with code spans as `<code>`. */
export interface Formatted {
  label: string;
}

/** A page of the sidebar: its collection entry, its address, and its label. */
export interface SidebarPage<E> {
  entry: E;
  href: string;
  /** Its label as plain text: its `nav-label`, else its title. */
  label: string;
  /** Its label as HTML, when it's read with inline markup, as the title is. */
  formatted?: Formatted;
}

/**
 * A group of the sidebar: a section of the navigation, or a group of a
 * site's own list. `pages` starts with the section's own page when that's
 * its `index.md`; an own page beside the folder (`checks/diagnostics.md`
 * for `checks/diagnostics/`) is among the parent's pages instead, and the
 * nested group sits right after it.
 */
export interface SidebarGroup<E> {
  label: string;
  /** Its label as HTML, when it's its own page's title, read with inline markup. */
  formatted?: Formatted;
  /** The folder, from the content root, ending in `/`; absent for a group of a site's own list. */
  folder?: string;
  /** Its own page, when the build offers one. */
  page?: SidebarPage<E>;
  pages: SidebarPage<E>[];
  /** Its nested sections, in order; empty for a group of a site's own list. */
  sections: SidebarGroup<E>[];
}

/** A page in `_ascribe/nav.json`. */
export interface NavPage {
  /** Its path from the content root, as its record's `source`. */
  source: string;
  label: string;
  formatted?: Formatted;
  /** Its content type. */
  type?: string;
}

/** A section in `_ascribe/nav.json`. */
export interface NavSection {
  /** From the content root, ending in `/`; `""` for the root. */
  folder: string;
  label: string;
  formatted?: Formatted;
  page?: NavPage;
  pages: NavPage[];
  sections: NavSection[];
}

/** `_ascribe/nav.json`. */
export interface Navigation {
  version: number;
  root: NavSection;
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
 * The sidebar's groups from the project's navigation: the root's own page
 * and pages under its label, "Start" unless `ascribe.nav` gives another,
 * when it has any, then each top section, each page with its entry and
 * address. `source` gives an entry's page's source path, which the
 * navigation names it by. Throws, naming each, when the navigation names a
 * page the collection has no entry for.
 */
export function navigationGroups<E extends NavEntry>(
  navigation: Navigation,
  entries: readonly E[],
  route: (entry: E) => string,
  source: (entry: E) => string,
): SidebarGroup<E>[] {
  const bySource = new Map(entries.map((entry) => [source(entry), entry]));
  const missing: string[] = [];
  const page = (nav: NavPage): SidebarPage<E>[] => {
    const entry = bySource.get(nav.source);
    if (entry === undefined) {
      missing.push(nav.source);
      return [];
    }
    return [
      {
        entry,
        href: route(entry),
        label: nav.label,
        ...(nav.formatted === undefined ? {} : { formatted: nav.formatted }),
      },
    ];
  };
  const group = (section: NavSection, parent: NavSection | undefined): SidebarGroup<E> => {
    const own = section.page === undefined ? [] : page(section.page);
    // An own page beside the folder is one of the parent's pages.
    const beside =
      section.page !== undefined &&
      parent?.pages.some((other) => other.source === section.page?.source) === true;
    return {
      label: section.label,
      ...(section.formatted === undefined ? {} : { formatted: section.formatted }),
      folder: section.folder,
      ...(own[0] === undefined ? {} : { page: own[0] }),
      pages: [...(beside ? [] : own), ...section.pages.flatMap(page)],
      sections: section.sections.map((nested) => group(nested, section)),
    };
  };
  const { root } = navigation;
  // A top section's own page beside its folder is in that section's group.
  const top = {
    ...group({ ...root, pages: root.pages.filter((nav) => !beside(root, nav)) }, undefined),
    sections: [],
  };
  const groups = [
    ...(top.pages.length === 0 ? [] : [top]),
    ...root.sections.map((section) => group(section, undefined)),
  ];
  if (missing.length > 0) {
    throw new Error(
      `@ascribed/astro: the project's navigation (_ascribe/nav.json) names pages the collection has no entry for:\n${[...new Set(missing)].map((file) => `- ${file}`).join("\n")}\nBuild the project again (\`ascribe build\`), so the navigation and the pages are from one build.`,
    );
  }
  return groups;
}

/**
 * The navigation's pages in reading order, depth first: a section's own
 * page, then its pages, each nested section whose own page is beside its
 * folder right after that page, with its pages and nested sections, and the
 * other nested sections after them, each from its own page. The content
 * root reads as the sidebar groups it: its own page and its pages that
 * aren't beside a folder, then each top section from its own page. Previous
 * and next follow it, as an archive and `llms.txt` do.
 */
export function readingOrder(navigation: Navigation): NavPage[] {
  const out: NavPage[] = [];
  const read = (section: NavSection, own: boolean) => {
    if (own && section.page !== undefined) out.push(section.page);
    const done = section.sections.map(() => false);
    for (const page of section.pages) {
      out.push(page);
      section.sections.forEach((nested, at) => {
        if (!done[at] && nested.page?.source === page.source) {
          done[at] = true;
          read(nested, false);
        }
      });
    }
    section.sections.forEach((nested, at) => {
      if (!done[at]) read(nested, true);
    });
  };
  const { root } = navigation;
  if (root.page !== undefined) out.push(root.page);
  out.push(...root.pages.filter((nav) => !beside(root, nav)));
  for (const section of root.sections) read(section, true);
  return out;
}

/** Whether `page` is the own page of one of `section`'s nested sections. */
function beside(section: NavSection, page: NavPage): boolean {
  return section.sections.some((nested) => nested.page?.source === page.source);
}

/** A page's previous and next, each with its entry, its address, and its label. */
export interface Neighbors<E> {
  previous?: SidebarPage<E>;
  next?: SidebarPage<E>;
}

/**
 * The pages before and after `at` in `pages`, the order a reader goes in.
 * Neither, for a page that isn't in it.
 */
export function neighborsIn<E>(
  pages: readonly SidebarPage<E>[],
  at: (page: SidebarPage<E>) => boolean,
): Neighbors<E> {
  const i = pages.findIndex(at);
  if (i === -1) return {};
  const previous = pages[i - 1];
  const next = pages[i + 1];
  return {
    ...(previous === undefined ? {} : { previous }),
    ...(next === undefined ? {} : { next }),
  };
}

/**
 * The navigation's pages in reading order, each with its entry, its
 * address, and its label. `source` gives an entry's page's source path.
 * A page the collection has no entry for is passed over.
 */
export function readingPages<E extends NavEntry>(
  navigation: Navigation,
  entries: readonly E[],
  route: (entry: E) => string,
  source: (entry: E) => string,
): SidebarPage<E>[] {
  const bySource = new Map(entries.map((entry) => [source(entry), entry]));
  return readingOrder(navigation).flatMap((nav) => {
    const entry = bySource.get(nav.source);
    if (entry === undefined) return [];
    return [
      {
        entry,
        href: route(entry),
        label: nav.label,
        ...(nav.formatted === undefined ? {} : { formatted: nav.formatted }),
      },
    ];
  });
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
 * The sidebar's groups from a site's own list, each page with its entry, its
 * address, and its label, which `label` gives, or else its entry id. Throws,
 * naming each, when the list leaves out a page of the collection, names a
 * file that isn't one, or names a page twice, so the list stays complete.
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
  label?: (entry: E) => string | undefined,
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
      return entry === undefined
        ? []
        : [{ entry, href: route(entry.id), label: label?.(entry) ?? entry.id }];
    }),
    sections: [],
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
