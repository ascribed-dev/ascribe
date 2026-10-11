// Sites the end-to-end tests build from copies of the example: a home page in
// front of the docs, and a second project beside them.
import { mkdir, rename, rm, writeFile } from "node:fs/promises";
import path from "node:path";
import { copySite, edit } from "./harness.js";

/** A plain Astro home page at the site's root, linking to the docs. */
const HOME_PAGE = `---
---

<!doctype html>
<html lang="en">
  <head><meta charset="utf-8" /><title>Loom</title></head>
  <body><main><h1>Loom weaves</h1><a href="/docs">Read the docs</a></main></body>
</html>
`;

/** The integration with one project in the list form, whose `base` stays the site's. */
export const ONE_ENTRY = 'ascribe({ projects: [{ name: "docs", build: "site" }] })';

/**
 * A copy of the site with Astro's `base` at `/` and the docs still at
 * `/docs/`, their route under `src/pages/docs/`, and a home page at `/`. The
 * docs are an entry of `projects`: with `build`, Astro's `base` would come
 * from their base path, and the home page would move under it.
 */
export async function homeInFront(name: string): Promise<string> {
  const root = await copySite(name);
  await edit(path.join(root, "astro.config.mjs"), (text) =>
    text.replace('ascribe({ build: "site" })', ONE_ENTRY),
  );
  await edit(path.join(root, "src", "content.config.ts"), (text) =>
    text
      .replace(
        '"../.ascribe/build/site/site/_ascribe/schema.ts"',
        '"../.astro/integrations/_ascribed_astro/docs/schema.ts"',
      )
      .replace("ascribeCollection({ schema })", 'ascribeCollection({ schema, name: "docs" })'),
  );
  const pages = path.join(root, "src", "pages");
  await mkdir(path.join(pages, "docs"));
  await rename(path.join(pages, "[...slug].astro"), path.join(pages, "docs", "[...slug].astro"));
  await edit(path.join(pages, "docs", "[...slug].astro"), (text) =>
    text.replace('"../layouts/Docs.astro"', '"../../layouts/Docs.astro"'),
  );
  await writeFile(path.join(pages, "index.astro"), HOME_PAGE);
  return root;
}

/** Writes `files`, by path under `root`. */
export async function write(root: string, files: Record<string, string>): Promise<void> {
  for (const [relative, text] of Object.entries(files)) {
    await mkdir(path.dirname(path.join(root, relative)), { recursive: true });
    await writeFile(path.join(root, relative), text);
  }
}

/** The second project, `home/`: the site's root, with a permalink of its own. */
const HOME_PROJECT = {
  "home/ascribe.toml": `spec = "0.1"

[project]
content-root = "content"

[consumer]
profile = "astro"
site = "https://docs.example.com"
base-path = "/"
trailing-slash = "never"
permalink-path = "/go/"

[types.page]
default = true

[types.page.frontmatter]
title = "string"
`,
  "home/content/index.md": `---
title: Loom
---

## Loom weaves
@id {permalink=true}: welcome

Loom weaves your strands together.
`,
  "home/content/why.md": `---
title: Why Loom
---

Because strands belong together.
`,
};

/** The route of the home collection's pages, at the site's root. */
const HOME_ROUTE = `---
import { getCollection, render } from "astro:content";

export async function getStaticPaths() {
  const entries = await getCollection("home");
  return entries.map((entry) => ({
    params: { slug: entry.id === "index" ? undefined : entry.id },
    props: { entry },
  }));
}

const { entry } = Astro.props;
const { Content } = await render(entry);
---

<!doctype html>
<html lang="en">
  <head><meta charset="utf-8" /><title>{entry.data.title}</title></head>
  <body><main><h1>{entry.data.title}</h1><Content /></main></body>
</html>
`;

/** The collections of two named entries, `docs` and `second`. */
export function contentConfig(second: string): string {
  return `import { defineCollection } from "astro:content";
import { ascribeCollection } from "@ascribed/astro/content";
import { schema as docs } from "../.astro/integrations/_ascribed_astro/docs/schema.ts";
import { schema as second } from "../.astro/integrations/_ascribed_astro/${second}/schema.ts";

export const collections = {
  docs: defineCollection(ascribeCollection({ schema: docs, name: "docs" })),
  ${second}: defineCollection(ascribeCollection({ schema: second, name: "${second}" })),
};
`;
}

/**
 * A copy with two projects: the docs at `/docs/` and `home/` at `/`, each
 * with its permalinks at `/go/`.
 */
export async function twoProjects(name: string): Promise<string> {
  const root = await homeInFront(name);
  await rm(path.join(root, "src", "pages", "index.astro"));
  await write(root, {
    ...HOME_PROJECT,
    "src/pages/[...slug].astro": HOME_ROUTE,
    "src/content.config.ts": contentConfig("home"),
  });
  await edit(path.join(root, "ascribe.toml"), (text) =>
    text.replace('trailing-slash = "never"', 'trailing-slash = "never"\npermalink-path = "/go/"'),
  );
  await edit(path.join(root, "astro.config.mjs"), (text) =>
    text.replace(
      ONE_ENTRY,
      `ascribe({
      projects: [
        { name: "docs", build: "site" },
        { name: "home", project: "home", build: "site" },
      ],
      review: "docs",
    })`,
    ),
  );
  return root;
}
