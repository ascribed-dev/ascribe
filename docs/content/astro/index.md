---
type: how-to
title: Publish a site with Astro
description: "Choose how to publish an Ascribe project, and set up an Astro site that publishes it, with `@ascribed/astro`."
permalink: astro-guide
---

`@ascribed/astro` publishes an Ascribe project as an [Astro](https://astro.build) site. It runs `ascribe build --emit site` before Astro loads content, gives you a content collection of the built pages with a schema generated from `ascribe.toml`, applies heading ids and image attributes in Astro's own Markdown pipeline (so Astro's table of contents and image processing still work), loads the element library, and serves the files pages link to. In `astro dev`, it rebuilds as you edit.

It supports Astro 7.3.5 and later 7.x releases.

This guide sets a site up. [The `@ascribed/astro` reference](reference.md) has its options, its components, and everything it does in `astro build` and `astro dev`.

## Ways to publish
@id: ways-to-publish

Ascribe checks and builds a project's pages; a site generator publishes them. There are three ways to put them on the web, and one of them isn't for a published site:

| Way | What it is | For a published site |
|---|---|---|
| **Astro** | `@ascribed/astro`, which this guide sets up | Yes: everything Ascribe builds, published and tested together |
| **Your own generator** | Another generator that reads the site output, as the contracts say | Yes: what the contracts give, and the rest is yours to write |
| **`ascribe preview`** | Ascribe's own HTML, on your computer | No |

**Astro** is the way Ascribe builds for and tests. The integration renders the pages, and publishes, from the same build, each page's head, the sitemap and `robots.txt`, the outputs for agents, permalinks, every line of a versioned build, and a host's files. [Set up a site](#set-up-a-site) below, then [deploy it](deploy.md).

**Your own generator** reads what `ascribe build --emit site` writes, which the contracts fix: each changes only by a decision recorded with the change, so a generator can rely on it as `@ascribed/astro` does. The pages are CommonMark with Ascribe's elements, which the [site-render contract](../contracts/site-render.md) says how to render, with [`@ascribed/elements`](elements.md) for their script and styles; the [output-layout contract](../contracts/output-layout.md) says where each file is, the page records among them; and the [navigation contract](../contracts/navigation.md) gives the sidebar. A page's address is the one the `astro` profile gives it, the only [consumer profile](../content-model/output.md#consumer) there is, so the generator's routes follow it. What the integration does beyond rendering is the generator's to do, from the same files: [the head, sitemap, and `robots.txt`](../publishing/site.md), [the permalinks' pages](reference.md#permalinks), and the outputs for agents, copied from the plain output ([`[consumer]`](../content-model/output.md#consumer)).

**`ascribe preview`** renders the pages as HTML with a sidebar and search, to read them as a site without Astro, and isn't for a published site: it serves only the computer it runs on, renders a page with errors as far as it goes, and writes none of what a site tells search engines and agents ([`ascribe preview`](../checks/check.md#cli-preview)). [`ascribe versions export`](../versions/archive.md) renders the same HTML into a folder that needs nothing else, as an archive of a release line that's leaving the live versions, and every page in it asks not to be listed.

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

`astro build` and `astro dev` run the `ascribe` of the `@ascribed/cli` that `@ascribed/astro` depends on, which is always of the same version as `@ascribed/astro` (or the binary its [`binary` option](reference.md#options) names). The editor, CI, and `npx ascribe` run the project's own, the one [Getting started](../getting-started.md) installs. Keep the two packages on the same version, and the package manager installs one `@ascribed/cli` that both run; with two versions, the site and the check can disagree about a page. Upgrade them together:

@variant {pm=npm}:
```sh
npm install @ascribed/astro@latest
npm install --save-dev @ascribed/cli@latest
```
@variant {pm=pnpm}:
```sh
pnpm add @ascribed/astro@latest
pnpm add --save-dev @ascribed/cli@latest
```
@end

### 2. Write `ascribe.toml`

Put `ascribe.toml` in the Astro project's root, beside `astro.config.mjs`. Its `[consumer]` settings are the site's routing, because Ascribe writes every link: the integration gives Astro its `site`, `base`, and `trailingSlash` from them, so `astro.config.mjs` can leave all three out ([a home page in front of the docs](more-than-docs.md#a-home-page-in-front-of-the-docs) says how `base` can differ from `base-path`):

@snippet: code:examples/astro-site/ascribe.toml#site

The source pages go in `content/`. If your Ascribe project lives elsewhere, such as in a folder beside the site's, set the integration's `project` option to its directory (step 3). The [`ascribe.toml` reference](../content-model/index.md) has everything else it can declare.

#### From Getting started
@id: from-getting-started

With the `ascribe.toml` from [Getting started](../getting-started.md), keep your file, and your pages where they are: its content root stays `docs`, so where this guide says `content/`, read `docs/`. Its `[consumer]` already has `site`, and the defaults are an `astro` profile, the docs at the site's root, and addresses that end in `/`. Add a line only for what you want otherwise, such as the two in the example above:

```toml
[consumer]
site = "https://docs.example.com"
base-path = "/docs/"        # add: the docs under /docs/, not at the site's root
trailing-slash = "never"    # add: addresses without a final /
```

Set `base-path` before the pages' first commit if you can. Once a commit has them under another one, `ascribe check` reports that the change moved their addresses, as [`address-lost`](../checks/diagnostics/project-files.md#ASC255): see [before the first deploy](../publishing/redirects.md#before-the-first-deploy) for what to do when nothing was published.

### 3. Add the integration

@snippet {title=astro.config.mjs}: code:examples/astro-site/astro.config.mjs

`build` names the [build](../glossary.md#build) in `ascribe.toml` whose output the site shows. A project without `[builds]` has one, `site`.

### 4. Write the site's files

A site needs a content collection of the built pages, a route for them, and a layout that loads the elements; this one has a sidebar and a stylesheet too. One command writes the four files, in the Astro project's root:

@variant {pm=npm}:
```sh
npx @ascribed/astro init
```
@variant {pm=pnpm}:
```sh
pnpm exec ascribe-astro init
```
@end

It writes `src/content.config.ts`, `src/pages/[...slug].astro`, `src/layouts/Docs.astro`, and `src/styles/docs.css`, and from then on they're yours to change; it never writes over a file, and if any of them is there, it writes nothing. With the Ascribe project elsewhere, say where with `--project ../docs`, and with several builds in `ascribe.toml`, say which the site shows with `--build`. The sections below say what each file does, as `init` writes them for the `ascribe.toml` above.

#### The collection

@snippet {title=src/content.config.ts}: code:examples/astro-site/src/content.config.ts

The schema's path is `<output-dir>/<build>/site/_ascribe/schema.ts`, in the directory holding `ascribe.toml`. Importing it by path keeps its exact types, so `entry.data` is typed from your content types. If you change `[project] output-dir`, the build, or the integration's `project`, change this line too.

If the Ascribe project is outside the Astro root (`project: "../docs"`), import the copy of the schema the integration writes inside it instead, `.astro/integrations/_ascribed_astro/schema.ts`, which is `../.astro/integrations/_ascribed_astro/schema.ts` from `src/content.config.ts`, as `init --project` does. The schema imports `astro/zod`, which TypeScript can only find from inside the Astro project, so type-checking (`astro check`, `tsc`) fails on the original there; Astro's own build works with either. The integration updates the copy after each build.

#### The route

A page's URL is the base path plus its entry id, and the root page, `content/index.md` (entry id `index`), is at the base path itself. That's what Ascribe's links point to:

@snippet {title=src/pages/[...slug].astro}: code:examples/astro-site/src/pages/[...slug].astro

Entry ids are Astro's own: `Guides/My Setup.md` is `guides/my-setup`. The route is at the root of `src/pages/` because the base path is Astro's `base`; with the docs under it, move the route to their folder ([a home page in front of the docs](more-than-docs.md#a-home-page-in-front-of-the-docs)).

#### The layout

A layout loads the element library, and shows the sidebar, the page's availability, and its table of contents:

@snippet {title=src/layouts/Docs.astro}: code:packages/astro/starter/src/layouts/Docs.astro

`<Elements />` loads the element library, and `<Head entry={entry} />` writes the page's head from its record. The layout is yours: replace any part of it with your own components; the table of contents is `headings` from `render`. [The components](reference.md#components) lists what `@ascribed/astro` gives a layout, and what each entry's data has for it.

#### The stylesheet

`src/styles/docs.css`, which the layout imports, gives it a readable column in the system's font, the sidebar beside the page on a wide screen and behind a **Menu** on a phone, the table of contents beside the page where there's room and above it where there isn't, and both color schemes, as the reader's system is. Its colors and spaces are the element library's `--ascribe-*` properties, so setting one restyles the layout and the elements together ([Element library reference](elements.md)); the page's background and text are the browser's own light or dark.

It's a starting point, not a theme: change it as you like. To style the site from scratch, delete the file and its `import` line in the layout; the layout's markup reads in order without it.

#### The sidebar

The layout draws the sidebar from the project's [navigation](../navigation/index.md), with [`sidebar()`](reference.md#sidebar), and the links at the page's foot with [`neighbors()`](reference.md#neighbors), so the site keeps no list of its own. It lists a nested section's pages in their top section's group, each `<li>` with a `data-depth` for your stylesheet to indent:

![The starter's sidebar, with an indent for each depth: Start, with Loom documentation; Guides, with Set up Loom, Troubleshoot Loom, Work in teams, and, indented under Work in teams, the nested section's page Share a loom, the page being read, in bold; and Reference, with loom.yaml options.](../images/starter-sidebar-light.png){scheme=light}
![The starter's sidebar, with an indent for each depth: Start, with Loom documentation; Guides, with Set up Loom, Troubleshoot Loom, Work in teams, and, indented under Work in teams, the nested section's page Share a loom, the page being read, in bold; and Reference, with loom.yaml options.](../images/starter-sidebar-dark.png){scheme=dark}

To change the order or a label, write `ascribe.nav`, or drag the pages in the editor: [Put your pages in order](../navigation/order.md).

### 5. Build

```sh
npx astro dev     # rebuilds as you edit
npx astro build
```

`astro dev` prints the address it serves, such as `http://localhost:4321/`. Open it with your `base-path` after it, `http://localhost:4321/docs/` for the `ascribe.toml` above, and the site is there. `astro build` writes it to `dist/`, for your host: [Deploy the site](deploy.md) publishes it from CI.

## See also

- [Deploy the site](deploy.md): building the site in CI and publishing it on GitHub Pages or Netlify, and what each host needs.
- [Put the docs in a site with more than the docs](more-than-docs.md): a home page of your own at `/`, with the docs at `/docs/`.
- [Publish for AI agents](agents.md): `llms.txt` and each page's Markdown, with the site.
- [Serve the Markdown versions](serve-markdown.md): what your host does for agents.
- [The `@ascribed/astro` reference](reference.md): the options, the components, and everything the integration does in `astro build` and `astro dev`, with [versions](reference.md#versions) and [permalinks](reference.md#permalinks).
- [Element library reference](elements.md): styling the elements.
