---
type: how-to
title: Astro
description: Set up an Astro site that publishes an Ascribe project, with @ascribed/astro.
permalink: astro-guide
---

`@ascribed/astro` publishes an Ascribe project as an [Astro](https://astro.build) site. It runs `ascribe build --emit site` before Astro loads content, gives you a content collection of the built pages with a schema generated from `ascribe.toml`, applies heading ids and image attributes in Astro's own Markdown pipeline (so Astro's table of contents and image processing still work), loads the element library, and serves the files pages link to. In `astro dev`, it rebuilds as you edit.

It supports Astro 7.3.5 and later 7.x releases.

This guide sets a site up. [The `@ascribed/astro` reference](reference.md) has its options, its components, and everything it does in `astro build` and `astro dev`.

## Set up a site

These steps add Ascribe to an Astro project. [`examples/astro-site`]({repo}/tree/main/examples/astro-site) is a complete site built this way.

### 1. Install

@variant {pm=npm}:
```sh
npm install @ascribed/astro
```
@variant {pm=pnpm}:
```sh
pnpm add @ascribed/astro
```
@end

It brings `@ascribed/cli`, the `ascribe` command for your platform, and `@ascribed/elements`. Astro's image processing also needs `sharp` (`npm install sharp`) if your project doesn't have it.

### 2. Write `ascribe.toml`

Put `ascribe.toml` in the Astro project's root, beside `astro.config.mjs`. Its `[consumer]` settings are the site's routing, because Ascribe writes every link: the integration gives Astro its `site`, `base`, and `trailingSlash` from them, so `astro.config.mjs` can leave all three out ([a home page in front of the docs](more-than-docs.md#a-home-page-in-front-of-the-docs) says how `base` can differ from `base-path`):

@snippet: code:examples/astro-site/ascribe.toml#site

The source pages go in `content/`. If your Ascribe project lives elsewhere, such as in a folder beside the site's, set the integration's `project` option to its directory (step 3). The [`ascribe.toml` reference](../content-model/index.md) has everything else it can declare.

### 3. Add the integration

@snippet {title=astro.config.mjs}: code:examples/astro-site/astro.config.mjs

`build` names the [build](../glossary.md#build) in `ascribe.toml` whose output the site shows. A project without `[builds]` has one, `site`.

### 4. Write the site's files

A site needs a content collection of the built pages, a route for them, and a layout that loads the elements; this one has a sidebar too. One command writes the three files, in the Astro project's root:

@available: ascribe 0.3.0
@variant {pm=npm}:
```sh
npx @ascribed/astro init
```
@variant {pm=pnpm}:
```sh
pnpm exec ascribe-astro init
```
@end

@available: ascribe 0.3.0
It writes `src/content.config.ts`, `src/pages/[...slug].astro`, and `src/layouts/Docs.astro`, and from then on they're yours to change; it never writes over a file, and if any of the three is there, it writes nothing. With the Ascribe project elsewhere, say where with `--project ../docs`, and with several builds in `ascribe.toml`, say which the site shows with `--build`. The sections below say what each file does, as `init` writes them for the `ascribe.toml` above.

#### The collection

@snippet {title=src/content.config.ts}: code:examples/astro-site/src/content.config.ts

The schema's path is `<output-dir>/<build>/site/_ascribe/schema.ts`, in the directory holding `ascribe.toml`. Importing it by path keeps its exact types, so `entry.data` is typed from your content types. If you change `[project] output-dir`, the build, or the integration's `project`, change this line too.

@available: ascribe 0.2.0
If the Ascribe project is outside the Astro root (`project: "../docs"`), import the copy of the schema the integration writes inside it instead, `.astro/integrations/_ascribed_astro/schema.ts`, which is `../.astro/integrations/_ascribed_astro/schema.ts` from `src/content.config.ts`, as `init --project` does. The schema imports `astro/zod`, which TypeScript can only find from inside the Astro project, so type-checking (`astro check`, `tsc`) fails on the original there; Astro's own build works with either. The integration updates the copy after each build.

#### The route

A page's URL is the base path plus its entry id, and the root page, `content/index.md` (entry id `index`), is at the base path itself. That's what Ascribe's links point to:

@snippet {title=src/pages/[...slug].astro}: code:examples/astro-site/src/pages/[...slug].astro

Entry ids are Astro's own: `Guides/My Setup.md` is `guides/my-setup`. The route is at the root of `src/pages/` because the base path is Astro's `base`; with the docs under it, move the route to their folder ([a home page in front of the docs](more-than-docs.md#a-home-page-in-front-of-the-docs)).

#### The layout

A layout loads the element library, and shows the sidebar, the page's availability, and its table of contents:

@available: ascribe 0.2.0
@snippet {title=src/layouts/Docs.astro}: code:packages/astro/starter/src/layouts/Docs.astro

`<Elements />` loads the element library, and `<Head entry={entry} />` writes the page's head from its record. The layout is yours: style it, and replace any part of it with your own components; the table of contents is `headings` from `render`. [The components](reference.md#components) lists what `@ascribed/astro` gives a layout, and what each entry's data has for it.

#### The sidebar
@available: ascribe 0.3.0

@available: ascribe 0.4.0
The layout draws the sidebar from the project's [navigation](../navigation/index.md), with [`sidebar()`](reference.md#sidebar), and the links at the page's foot with [`neighbors()`](reference.md#neighbors), so the site keeps no list of its own. It lists a nested section's pages in their top section's group, each `<li>` with a `data-depth` for your stylesheet to indent. To change the order or a label, write `ascribe.nav`, or drag the pages in the editor: [Put your pages in order](../navigation/order.md).

### 5. Build

```sh
npx astro dev     # rebuilds as you edit
npx astro build
```

`astro dev` prints the address it serves, such as `http://localhost:4321/`. Open it with your `base-path` after it, `http://localhost:4321/docs/` for the `ascribe.toml` above, and the site is there. `astro build` writes it to `dist/`, for your host.

## See also

- [Put the docs in a site with more than the docs](more-than-docs.md): a home page of your own at `/`, with the docs at `/docs/`.
- [Publish for AI agents](agents.md): `llms.txt` and each page's Markdown, with the site.
- [Serve the Markdown versions](serve-markdown.md): what your host does for agents.
- [The `@ascribed/astro` reference](reference.md): the options, the components, and everything the integration does in `astro build` and `astro dev`, with [versions](reference.md#versions) and [permalinks](reference.md#permalinks).
- [Element library reference](elements.md): styling the elements.
