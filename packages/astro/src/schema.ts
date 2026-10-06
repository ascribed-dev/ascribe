// The collection's generated schema, copied inside the Astro root.
//
// `ascribe build` writes `_ascribe/schema.ts` in the site output, which can be
// outside the Astro project (`project: "../docs"`). Its `import { z } from
// "astro/zod"` then resolves from there for TypeScript, where no
// `node_modules/astro` is, so type-checking the site fails (issue #93); Vite
// resolves it either way. A copy in the integration's folder of `.astro/`
// resolves `astro/zod` from the Astro root, and keeps the schema's exact
// inferred types, which a virtual module would hide.
import { existsSync, readFileSync, writeFileSync } from "node:fs";
import path from "node:path";

/** The generated schema inside a site output root. */
export function schemaPath(siteRoot: string): string {
  return path.join(siteRoot, "_ascribe", "schema.ts");
}

/**
 * Copies the site output's schema to `<dir>/schema.ts`, unless the copy is already the same,
 * so an unchanged schema doesn't wake the dev server's watcher. Returns whether it wrote.
 */
export function copySchema(siteRoot: string, dir: string): boolean {
  const from = schemaPath(siteRoot);
  if (!existsSync(from)) return false;
  const text = readFileSync(from, "utf8");
  const to = path.join(dir, "schema.ts");
  if (existsSync(to) && readFileSync(to, "utf8") === text) return false;
  writeFileSync(to, text);
  return true;
}
