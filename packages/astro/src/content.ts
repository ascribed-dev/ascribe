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
import { pathToFileURL } from "node:url";
import path from "node:path";
import { glob } from "astro/loaders";
import { collections, siteRoot } from "virtual:ascribe/site";

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
