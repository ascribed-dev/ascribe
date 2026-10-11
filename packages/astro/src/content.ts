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
// A layout's sidebar, from the site's list of pages (`src/nav.ts`, which
// `npx @ascribed/astro init` writes), checked against the collection:
//
//   const groups = sidebar(nav, await getCollection("docs"));
import { pathToFileURL } from "node:url";
import path from "node:path";
import { glob } from "astro/loaders";
import { astroRoot, collections, routes, siteRoot } from "virtual:ascribe/site";
import {
  pageRoute,
  sidebarGroups,
  type NavEntry,
  type NavGroup,
  type SidebarGroup,
} from "./nav.js";

export type { NavGroup, SidebarGroup, SidebarPage } from "./nav.js";

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
 */
export function ascribeCollection<S>(options: { schema: S; name?: string }): {
  loader: ReturnType<typeof glob>;
  schema: S;
} {
  // `_ascribe/` holds the schema and published files, never pages.
  const base = pathToFileURL(rootFor(options.name) + path.sep);
  return { loader: glob({ pattern: ["**/*.md", "!_ascribe/**"], base }), schema: options.schema };
}

/**
 * A layout's sidebar: `nav`'s groups, each page with its collection entry and
 * its address, which is the one Ascribe's links use. `entries` is the
 * collection (`await getCollection("docs")`), and `name` says which project's,
 * as for `ascribeCollection`; with several projects and no `name`, it's the one
 * the entries were loaded from. Throws, naming each, when `nav` leaves out a page
 * of the collection, names a file that isn't one, or names a page twice, so the
 * build fails until the list is complete.
 */
export function sidebar<E extends NavEntry>(
  nav: readonly NavGroup[],
  entries: readonly E[],
  options: { name?: string } = {},
): SidebarGroup<E>[] {
  const root = sidebarRoot(entries, options.name);
  const at = routes.find((route) => route.siteRoot === root);
  const basePath = at?.basePath ?? "/";
  const trailingSlash = at?.trailingSlash ?? "always";
  return sidebarGroups(
    nav,
    entries,
    (id) => pageRoute(id, basePath, trailingSlash),
    (entry) => sourceFile(root, entry),
  );
}

/**
 * The site output root of the collection a sidebar lists: the entry `name`
 * names, or, with several and no name, the one the entries were loaded from.
 */
function sidebarRoot(entries: readonly NavEntry[], name: string | undefined): string {
  const file = entries[0]?.filePath;
  if (name === undefined && Object.keys(collections).length > 1 && file !== undefined) {
    const from = Object.values(collections).find((root) => inside(root, file));
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
