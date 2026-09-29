// The content collection over a build's site output. Import it from
// `content.config.ts`, which Vite loads, so it can read what the integration
// resolved (`virtual:tessera/site`) instead of finding the project itself.
//
//   import { defineCollection } from "astro:content";
//   import { tesseraCollection } from "@tessera/astro/content";
//   import { schema } from "../.tessera/build/site/site/_tessera/schema.ts";
//   export const collections = { docs: defineCollection(tesseraCollection({ schema })) };
import { pathToFileURL } from "node:url";
import path from "node:path";
import { glob } from "astro/loaders";
import { siteRoot } from "virtual:tessera/site";

/**
 * The collection's loader and schema: the `glob` loader on the site output's
 * pages, with the generated Zod schema (`_tessera/schema.ts`'s `schema`).
 *
 * Entry ids are Astro's own (`guides/my-setup` for `Guides/My Setup.md`, `index`
 * for the root page), which is what `AstroRouter` in `tessera-resolve`
 * computes, so a page's route is the base path plus its id, and the root page is
 * at the base path itself.
 */
export function tesseraCollection<S>(options: { schema: S }): {
  loader: ReturnType<typeof glob>;
  schema: S;
} {
  // `_tessera/` holds the schema and published files, never pages.
  const base = pathToFileURL(siteRoot + path.sep);
  return { loader: glob({ pattern: ["**/*.md", "!_tessera/**"], base }), schema: options.schema };
}
