---
type: how-to
title: Put the docs in a site with more than the docs
description: Serve the docs under a path, such as /docs/, in an Astro site whose home page and other pages are your own or a second Ascribe project.
---

A site often has more than its docs: a home page at `/`, with the docs at `/docs/`. This guide puts an Ascribe project's docs under a path of one Astro site, and leaves the rest of the site to your own pages, or to a second Ascribe project.

## Before you start

- A site [set up with `@ascribed/astro`](index.md), whose docs are at Astro's `base`.

## A home page in front of the docs
@id {permalink=true}: a-home-page-in-front-of-the-docs
@available: ascribe 0.3.0

### 1. Give the docs their path

Set `[consumer] base-path` to the docs' path:

```toml
# ascribe.toml
[consumer]
profile = "astro"
site = "https://docs.example.com"
base-path = "/docs/"
trailing-slash = "never"
```

### 2. List the project in `projects`

List the project in [`projects`](reference.md#several-projects), even alone, and leave Astro's `base` out, so it stays at the site's root. With `build`, the integration gives Astro's `base` the docs' base path, which would put your home page under it; with `projects`, `base` stays the site's:

```js
// astro.config.mjs: `site` and `trailingSlash` come from [consumer].
export default defineConfig({
  integrations: [ascribe({ projects: [{ name: "docs", build: "site" }] })],
});
```

### 3. Name the collection's entry

The collection names its entry, and imports its schema from the folder named for it:

```ts
// src/content.config.ts
import { defineCollection } from "astro:content";
import { ascribeCollection } from "@ascribed/astro/content";
import { schema } from "../.astro/integrations/_ascribed_astro/docs/schema.ts";

export const collections = {
  docs: defineCollection(ascribeCollection({ schema, name: "docs" })),
};
```

### 4. Move the route under the docs' path

A page's route is `base-path` less Astro's `base`, plus its entry id, so move the route to `src/pages/docs/[...slug].astro`, with the same `getStaticPaths` as [the route](index.md#the-route).

### 5. Write the home page

Your home page is `src/pages/index.astro`, an Astro page like any other, or a second Ascribe project ([below](more-than-docs.md#several-projects-in-one-site)).

### 6. Build

Run `npx astro dev`, and open the address it prints: your home page is at `/`, and the docs at `/docs/`.

Everything Ascribe publishes for the docs is at their base path: the files pages link to (`/docs/_ascribe/files/`), `llms.txt` and each page's Markdown, and, by default, the permalinks (`/docs/go/`). [`permalink-path`](../content-model/output.md#consumer) can put the permalinks anywhere under Astro's `base`, such as `/go/`. A base path outside Astro's `base` fails the build, since an Astro site serves nothing outside it.

## Several projects in one site
@id {permalink=true}: several-projects-in-one-site
@available: ascribe 0.3.0

One integration can build several Ascribe projects into one site, each under its own base path, such as a home page written in Ascribe at `/`, beside the docs at `/docs/`.

@steps
1. Add the second project to `projects`, each entry with a name, and say which project review covers:

   ```js
   // astro.config.mjs
   integrations: [
     ascribe({
       projects: [
         { name: "docs", project: "../docs", build: "site" },
         { name: "home", project: "home", build: "site" },
       ],
       review: "docs",
     }),
   ],
   ```

2. Give each project a collection, named for its entry, with the copy of its schema from the folder named for it:

   ```ts
   // src/content.config.ts
   import { defineCollection } from "astro:content";
   import { ascribeCollection } from "@ascribed/astro/content";
   import { schema as docsSchema } from "../.astro/integrations/_ascribed_astro/docs/schema.ts";
   import { schema as homeSchema } from "../.astro/integrations/_ascribed_astro/home/schema.ts";

   export const collections = {
     docs: defineCollection(ascribeCollection({ schema: docsSchema, name: "docs" })),
     home: defineCollection(ascribeCollection({ schema: homeSchema, name: "home" })),
   };
   ```

3. Give each collection a route under its project's base path: `src/pages/[...slug].astro` for `home`, and `src/pages/docs/[...slug].astro` for `docs`.
4. Build. The integration fails the build, naming the entries, when the projects disagree about what they share, such as two with one base path or one permalink: [several projects](reference.md#several-projects) lists each, and what fixes it.

## See also

- [Several projects](reference.md#several-projects), in the `@ascribed/astro` reference: an entry's name, and what the projects must agree on.
- [`[consumer]`](../content-model/output.md#consumer): `base-path`, `permalink-path`, and the site's other routing.
