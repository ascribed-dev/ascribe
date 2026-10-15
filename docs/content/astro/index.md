---
type: how-to
title: Astro
description: Publishing a site with @ascribed/astro.
permalink: astro-guide
---

`@ascribed/astro` publishes an Ascribe project as an [Astro](https://astro.build) site. It runs `ascribe build --emit site` before Astro loads content, gives you a content collection of the built pages with a schema generated from `ascribe.toml`, applies heading ids and image attributes in Astro's own Markdown pipeline (so Astro's table of contents and image processing still work), loads the element library, and serves the files pages link to. In `astro dev`, it rebuilds as you edit.

It supports Astro 7.3.5 and later 7.x releases.

This guide sets a site up. [The `@ascribed/astro` reference](reference.md) has its options, everything it does in `astro build` and `astro dev`, what it writes for search engines and crawlers, review in the site preview, and styling.

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

Put `ascribe.toml` in the Astro project's root, beside `astro.config.mjs`. Its `[consumer]` settings are the site's routing, because Ascribe writes every link: from the next release, the integration gives Astro its `site`, `base`, and `trailingSlash` from them, so `astro.config.mjs` can leave all three out ([a home page in front of the docs](index.md#a-home-page-in-front-of-the-docs) says how `base` can differ from `base-path`):

@snippet: code:examples/astro-site/ascribe.toml#site

The source pages go in `content/`. If your Ascribe project lives elsewhere, such as in a folder beside the site's, set the integration's `project` option to its directory (step 3). The [`ascribe.toml` reference](../content-model/index.md) has everything else it can declare.

### 3. Add the integration

@snippet {title=astro.config.mjs}: code:examples/astro-site/astro.config.mjs

`build` names the [build](../content-model/output.md#16-buildsname) in `ascribe.toml` whose output the site shows. A project without `[builds]` has one, `site`.

### 4. Write the site's files

A site needs a content collection of the built pages, a route for them, and a layout that loads the elements; this one has a sidebar too. One command writes the four files, in the Astro project's root:

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
It writes `src/content.config.ts`, `src/pages/[...slug].astro`, `src/layouts/Docs.astro`, and `src/nav.ts`, and from then on they're yours to change; it never writes over a file, and if any of the four is there, it writes nothing. It builds the project first, to list its pages in the sidebar. With the Ascribe project elsewhere, say where with `--project ../docs`, and with several builds in `ascribe.toml`, say which the site shows with `--build`. The sections below say what each file does, as `init` writes them for the `ascribe.toml` above.

#### The collection

@snippet {title=src/content.config.ts}: code:examples/astro-site/src/content.config.ts

The schema's path is `<output-dir>/<build>/site/_ascribe/schema.ts`, in the directory holding `ascribe.toml`. Importing it by path keeps its exact types, so `entry.data` is typed from your content types. If you change `[project] output-dir`, the build, or the integration's `project`, change this line too.

@available: ascribe 0.2.0
If the Ascribe project is outside the Astro root (`project: "../docs"`), import the copy of the schema the integration writes inside it instead, `.astro/integrations/_ascribed_astro/schema.ts`, which is `../.astro/integrations/_ascribed_astro/schema.ts` from `src/content.config.ts`, as `init --project` does. The schema imports `astro/zod`, which TypeScript can only find from inside the Astro project, so type-checking (`astro check`, `tsc`) fails on the original there; Astro's own build works with either. The integration updates the copy after each build.

#### The route

A page's URL is the base path plus its entry id, and the root page, `content/index.md` (entry id `index`), is at the base path itself. That's what Ascribe's links point to:

@snippet {title=src/pages/[...slug].astro}: code:examples/astro-site/src/pages/[...slug].astro

Entry ids are Astro's own: `Guides/My Setup.md` is `guides/my-setup`. The route is at the root of `src/pages/` because the base path is Astro's `base`; with the docs under it, move the route to their folder ([a home page in front of the docs](index.md#a-home-page-in-front-of-the-docs)).

#### The layout

A layout loads the element library, and shows the sidebar, the page's availability, and its table of contents:

@available: ascribe 0.2.0
@snippet {title=src/layouts/Docs.astro}: code:examples/astro-site/src/layouts/Docs.astro

`<Elements />` loads the element library's stylesheet and the small script `<ascribe-tabs>` needs. The layout is yours: style it, and replace any part of it with your own components; the table of contents is `headings` from `render`.

@available: ascribe 0.3.0
`<Head entry={entry} />`, from `@ascribed/astro/Head.astro`, writes everything the page's head says about the page, from its record: its `<title>`, description, canonical link, robots rule, link to its Markdown version, link preview, and breadcrumbs ([the head](reference.md#the-head)).

@available: ascribe 0.2.0
The example's content model sets `inline = "code"` on `title`, so a title can show code, as "`` `loom.yaml` options ``". `entry.data.title` is then plain text, for `<title>` and search, and `entry.data.formatted.title` is the title as HTML, with each code span a `<code>`, for the heading and navigation (`set:html`). Without the setting, there's no `formatted`, and the layout shows `title`. See [code in a field](../content-model/index.md#53-code-in-a-field).

@available: ascribe 0.3.0
`entry.data.source` is the page's path from the content root, with `/`, as `guides/install.md`. An "Edit this page" link is your repository's edit address, the content root's folder in it, and `source`, wherever Ascribe's build folder is.

A page's `available` frontmatter reaches the layout as `entry.data.available`: a list of targets, each with the text to show.

@available: ascribe 0.2.0
`<Availability available={entry.data.available} />`, from `@ascribed/astro/Availability.astro`, renders it as a badge, as the layout above does: the same markup the site output gives a section's `@available`, which the element library styles. It renders nothing for a page with no availability.

@available: next
`<Versions entry={entry} />`, from `@ascribed/astro/Versions.astro`, lists the other lines of a [versioned build](#versions) that have the page. It renders nothing for a build that isn't versioned.

#### The sidebar
@available: ascribe 0.3.0

Ascribe's content model doesn't order or group pages yet, so the site lists them, by their source files under the content root, in groups:

@snippet {title=src/nav.ts}: code:examples/astro-site/src/nav.ts

`init` writes the pages there are, the root's under **Start** and a group for each folder; reorder and relabel them as you like. The layout's `sidebar(nav, await getCollection("docs"), { entry })`, from `@ascribed/astro/content`, gives each page its entry and its address, the one Ascribe's links use, and fails the build when the list leaves out a page, names a file that isn't one, or names a page twice. So when you add a page, add it to the list too. With several projects, it lists the project the entries came from; `sidebar(nav, entries, { name: "docs" })` names it, as `ascribeCollection` does. `entry` is the page the sidebar is on: with a [versioned build](#versions) the sidebar shows that page's line, and the list is checked against every line's pages.

### 5. Build

```sh
npx astro dev     # rebuilds as you edit
npx astro build
```

`astro dev` prints the address it serves, such as `http://localhost:4321/`. Open it with your `base-path` after it, `http://localhost:4321/docs/` for the `ascribe.toml` above, and the site is there. `astro build` writes it to `dist/`, for your host.

## A home page in front of the docs
@id {permalink=true}: a-home-page-in-front-of-the-docs
@available: ascribe 0.3.0

`[consumer] base-path` can be a path under Astro's `base`, which leaves the rest of the site to your own pages: a home page at `/` with the docs at `/docs/`, in one Astro site. Set `base-path` to the docs' path, leave Astro's `base` at the site's root, its default, and list the project in [`projects`](index.md#several-projects-in-one-site), even alone. With `build`, the integration gives Astro's `base` the docs' base path, which would put your home page under it; with `projects`, `base` stays the site's:

```js
// astro.config.mjs: `site` and `trailingSlash` come from [consumer].
export default defineConfig({
  integrations: [ascribe({ projects: [{ name: "docs", build: "site" }] })],
});
```

```toml
# ascribe.toml
[consumer]
profile = "astro"
site = "https://docs.example.com"
base-path = "/docs/"
trailing-slash = "never"
```

The collection names its entry, `ascribeCollection({ schema, name: "docs" })`, and imports its schema from `.astro/integrations/_ascribed_astro/docs/schema.ts`. A page's route is then `base-path` less Astro's `base`, plus its entry id, so the docs' route goes in `src/pages/docs/[...slug].astro`, with the same `getStaticPaths` as [the route](index.md#the-route). Your home page is `src/pages/index.astro`, or a second Ascribe project ([below](index.md#several-projects-in-one-site)).

Everything Ascribe publishes for the docs is at their base path: the files pages link to (`/docs/_ascribe/files/`), `llms.txt` and each page's Markdown, and, by default, the permalinks (`/docs/go/`). [`permalink-path`](../content-model/output.md#consumer) can put the permalinks anywhere under Astro's `base`, such as `/go/`. A base path outside Astro's `base` still fails the build, since an Astro site serves nothing outside it.

## Several projects in one site
@id {permalink=true}: several-projects-in-one-site
@available: ascribe 0.3.0

One integration can build several Ascribe projects into one site, each under its own base path, such as a home page written in Ascribe at `/` and the docs at `/docs/`. List them in `projects`, each entry with a name:

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

The name is what everything else says: the collection, `review`, the folder of the entry's schema, and an error message. It has letters, digits, `-`, and `_`, starts with a letter, and is the entry's own. Each collection names its entry, and imports the copy of its schema from a folder named for it:

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

Astro's `site` and `trailingSlash`, when `astro.config.mjs` leaves them out, come from the first entry whose project sets them; every project must agree with them anyway, and the build fails, naming the entry, when one doesn't. Astro's `base` is the site's, the root every base path is under: the integration never sets it from a project.

Each collection gets its own route, under its project's base path: here `src/pages/[...slug].astro` for `home` and `src/pages/docs/[...slug].astro` for `docs`.

Two entries can name one project with two builds, such as a collection for each release line. They share the project's `ascribe.toml`, so they share its base path: route one of them at it, and use the other's collection as you like. Where both publish a file or a permalink, the first entry's wins.

What the projects share, the integration checks, and it fails the build, naming the entries, when:

- **Two projects have one base path.** Each needs its own. One project at `/` with another beneath it at `/docs/` is fine.
- **A page of one project is under another's base path,** such as a page at `/docs/extra/` in the project at `/`.
- **Their permalinks are published at two paths.** One route serves every project's permalinks, so set `permalink-path` to one path in each project that declares any. A project with none can leave it.
- **Two projects declare one permalink.** Neither project's build can see the other's, so the integration reports it; rename or retire it in one of them.
- **Two entries have one name,** or a name has other characters.

In `astro dev`, a change rebuilds the project it's in, and each project's folder gets its own `.ascribe/dev.json`, with its own base path. Review covers one project: the entry `review` names, or the first entry when it's `true`. Each removed permalink's page links to the root of the project that retired it, and [`MissingPermalink`](../permalinks/index.md#a-permalink-that-isnt-here) offers the root of the first project with permalinks.

## Publish for AI agents
@available: ascribe 0.3.0

An AI agent reading your docs does better with Markdown than with HTML, and with an index to choose pages from. Turn on the outputs for agents in `ascribe.toml`'s `[consumer]`, beside `site`, and each build publishes them with the site, as the [Web Documentation Delivery Spec](https://agentdocsspec.com/spec/web/) lays them out:

@snippet: code:examples/astro-site/ascribe.toml#agents

- **`llms.txt`** at the base path: every page, by folder, as its title, a link to its Markdown, and its description (the field its type marks `role = "description"`). Past 50,000 characters it's split into one `llms.txt` per folder, which the root file links to.
- **Each page's Markdown** at its URL with `.md`: `/docs/guides/setup` is also `/docs/guides/setup.md`.
- **A pointer** at the top of each page to both, which the elements' stylesheet hides from sight but not from an agent reading the page.

Nothing is written at the host's root for a project under a path. With docs at `/docs/`, an agent reaches `/docs/llms.txt` through each page's pointer, and the spec's checker looks there as well as at `/llms.txt`. A site that wants a file at the root, such as one whose docs are somewhere else, puts its own in Astro's `public/`.

`site` is required, because everything an agent reads links by absolute URL. The integration builds the plain output too, copies it into the built site, and serves it in `astro dev`. The [`ascribe.toml` reference](../content-model/output.md#consumer) says how `llms.txt` is put together.

`ascribe check` then adds two pieces of advice: a page whose description is too long for its line in `llms.txt`, and an `llms.txt` that's still too long when split. Write each page's `description` as one sentence saying what the page is for; it's what an agent picks a page by.

### What your host does

The files are static, so any host serves them. A few things the spec asks are the server's, not the build's:

- **The `.md` files' type.** Serve them as `text/markdown; charset=utf-8`, and `llms.txt` as `text/plain; charset=utf-8`, as `astro dev` does. Check what your host sends (`curl -I` a page's `.md`), and set the header in its configuration if it's something else.
- **Content negotiation.** An agent may ask for a page with `Accept: text/markdown`; answering it with the page's `.md` needs a rule on the server, such as an edge function or middleware that rewrites to `<path>.md` when that header asks for Markdown. Without it, agents still find the `.md` URLs through `llms.txt` and the pointer. On Netlify, an edge function does it: this site's is [`markdown.ts`]({repo}/blob/main/site/netlify/edge-functions/markdown.ts). Whatever answers that way passes on the `.md` file's [response headers](reference.md#response-headers), `X-Robots-Tag` and `Link`: the answer is at the page's own address, where no rule for the `.md` path applies, and on Netlify a function's response gets none of `_headers` at all.
- **Status codes and caching.** A missing page should answer 404, not 200 with a page saying so, and the `.md` files and `llms.txt` should be cached no longer than the pages.
- **Bot protection.** A rule that challenges unfamiliar clients blocks agents too; let them read `llms.txt` and the `.md` files.

## Versions
@available: next

With a [versioned build](../versions/index.md), one site serves every live line: the current line's pages at the base path, and each older line's under its number, `/docs/3.4/`, from one collection, with nothing to change in `astro.config.mjs` or `src/content.config.ts`. [Publish with Astro](../versions/index.md#publish-with-astro) says what the collection, the layout, and `astro dev` do with them. For now, two things are the current line's alone:

- **A permalink always forwards to the current line.**
- **An older line has no Markdown version and no `llms.txt` at its addresses.** Its pages don't point to them, and the sitemap leaves its pages out.

## Permalinks
@available: ascribe 0.3.0

The integration gives each [permalink](../permalinks/index.md) an address, in `astro build` and in `astro dev`; [With Astro](../permalinks/index.md#with-astro) says what it serves and where. [Permalinks](../permalinks/index.md) covers the rest: linking to them from your product, checking those links in its CI, retiring one, and [`MissingPermalink`](../permalinks/index.md#a-permalink-that-isnt-here) for your 404 page.
