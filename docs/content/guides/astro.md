---
title: Astro
description: Publishing a site with @ascribed/astro.
permalink: astro-guide
---

`@ascribed/astro` publishes an Ascribe project as an [Astro](https://astro.build) site. It runs `ascribe build --emit site` before Astro loads content, gives you a content collection of the built pages with a schema generated from `ascribe.toml`, applies heading ids and image attributes in Astro's own Markdown pipeline (so Astro's table of contents and image processing still work), loads the element library, and serves the files pages link to. In `astro dev`, it rebuilds as you edit.

It supports Astro 7.3.5 and later 7.x releases.

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

Put `ascribe.toml` in the Astro project's root, beside `astro.config.mjs`. Its `[consumer]` settings are the site's routing, because Ascribe writes every link: from the next release, the integration gives Astro its `site`, `base`, and `trailingSlash` from them, so `astro.config.mjs` can leave all three out ([a home page in front of the docs](#a-home-page-in-front-of-the-docs) says how `base` can differ from `base-path`):

@snippet: code:examples/astro-site/ascribe.toml#site

The source pages go in `content/`. If your Ascribe project lives elsewhere, such as in a folder beside the site's, set the integration's `project` option to its directory (step 3). The [`ascribe.toml` reference](../reference/content-model.md) has everything else it can declare.

### 3. Add the integration

@snippet {title=astro.config.mjs}: code:examples/astro-site/astro.config.mjs

`build` names the [build](../reference/content-model.md#16-buildsname) in `ascribe.toml` whose output the site shows. A project without `[builds]` has one, `site`.

### 4. Write the site's files
@available: next

A site needs a content collection of the built pages, a route for them, and a layout that loads the elements; this one has a sidebar too. One command writes the four files, in the Astro project's root:

@variant {pm=npm}:
```sh
npx @ascribed/astro init
```
@variant {pm=pnpm}:
```sh
pnpm exec ascribe-astro init
```
@end

It writes `src/content.config.ts`, `src/pages/[...slug].astro`, `src/layouts/Docs.astro`, and `src/nav.ts`, and from then on they're yours to change; it never writes over a file, and if any of the four is there, it writes nothing. It builds the project first, to list its pages in the sidebar. With the Ascribe project elsewhere, say where with `--project ../docs`, and with several builds in `ascribe.toml`, say which the site shows with `--build`. The sections below say what each file does, as `init` writes them for the `ascribe.toml` above.

In a release without `init`, write the four files yourself, from the sections below.

#### The collection

@snippet {title=src/content.config.ts}: code:examples/astro-site/src/content.config.ts

The schema's path is `<output-dir>/<build>/site/_ascribe/schema.ts`, in the directory holding `ascribe.toml`. Importing it by path keeps its exact types, so `entry.data` is typed from your content types. If you change `[project] output-dir`, the build, or the integration's `project`, change this line too.

@available: ascribe 0.2.0
If the Ascribe project is outside the Astro root (`project: "../docs"`), import the copy of the schema the integration writes inside it instead, `.astro/integrations/_ascribed_astro/schema.ts`, which is `../.astro/integrations/_ascribed_astro/schema.ts` from `src/content.config.ts`, as `init --project` does. The schema imports `astro/zod`, which TypeScript can only find from inside the Astro project, so type-checking (`astro check`, `tsc`) fails on the original there; Astro's own build works with either. The integration updates the copy after each build.

#### The route

A page's URL is the base path plus its entry id, and the root page, `content/index.md` (entry id `index`), is at the base path itself. That's what Ascribe's links point to:

@snippet {title=src/pages/[...slug].astro}: code:examples/astro-site/src/pages/[...slug].astro

Entry ids are Astro's own: `Guides/My Setup.md` is `guides/my-setup`. The route is at the root of `src/pages/` because the base path is Astro's `base`; with the docs under it, move the route to their folder ([a home page in front of the docs](#a-home-page-in-front-of-the-docs)).

#### The layout

A layout loads the element library, and shows the sidebar, the page's availability, and its table of contents:

@available: ascribe 0.2.0
@snippet {title=src/layouts/Docs.astro}: code:examples/astro-site/src/layouts/Docs.astro

`<Elements />` loads the element library's stylesheet and the small script `<ascribe-tabs>` needs. The layout is yours: style it, and replace any part of it with your own components; the table of contents is `headings` from `render`.

@available: next
`<Head entry={entry} />`, from `@ascribed/astro/Head.astro`, writes everything the page's head says about the page, from its record: its `<title>`, description, canonical link, robots rule, link to its Markdown version, link preview, and breadcrumbs ([the head](#the-head)).

@available: ascribe 0.2.0
The example's content model sets `inline = "code"` on `title`, so a title can show code, as "`` `loom.yaml` options ``". `entry.data.title` is then plain text, for `<title>` and search, and `entry.data.formatted.title` is the title as HTML, with each code span a `<code>`, for the heading and navigation (`set:html`). Without the setting, there's no `formatted`, and the layout shows `title`. See [code in a field](../reference/content-model.md#53-code-in-a-field).

@available: next
`entry.data.source` is the page's path from the content root, with `/`, as `guides/install.md`. An "Edit this page" link is your repository's edit address, the content root's folder in it, and `source`, wherever Ascribe's build folder is.

A page's `available` frontmatter reaches the layout as `entry.data.available`: a list of targets, each with the text to show.

@available: ascribe 0.2.0
`<Availability available={entry.data.available} />`, from `@ascribed/astro/Availability.astro`, renders it as a badge, as the layout above does: the same markup the site output gives a section's `@available`, which the element library styles. It renders nothing for a page with no availability.

In a release without `Availability.astro`, copy [its markup]({repo}/blob/main/packages/astro/src/Availability.astro) into your layout in place of the component.

#### The sidebar
@available: next

Ascribe's content model doesn't order or group pages yet, so the site lists them, by their source files under the content root, in groups:

@snippet {title=src/nav.ts}: code:examples/astro-site/src/nav.ts

`init` writes the pages there are, the root's under **Start** and a group for each folder; reorder and relabel them as you like. The layout's `sidebar(nav, await getCollection("docs"))`, from `@ascribed/astro/content`, gives each page its entry and its address, the one Ascribe's links use, and fails the build when the list leaves out a page, names a file that isn't one, or names a page twice. So when you add a page, add it to the list too. With several projects, it lists the project the entries came from; `sidebar(nav, entries, { name: "docs" })` names it, as `ascribeCollection` does.

### 5. Build

```sh
npx astro dev     # rebuilds as you edit
npx astro build
```

`astro dev` prints the address it serves, such as `http://localhost:4321/`. Open it with your `base-path` after it, `http://localhost:4321/docs/` for the `ascribe.toml` above, and the site is there. `astro build` writes it to `dist/`, for your host.

## Options

| Option | Meaning |
|---|---|
| `build` | The build whose site output is the collection: a build name in `ascribe.toml`. Required, unless `projects` is given. |
| `project` | The directory holding `ascribe.toml`, relative to the Astro root. By default, the root. |
| `projects` {available=next} | [Several projects in one site](#several-projects-in-one-site): a list of entries, each `{ name, build, project }`, in place of `build` and `project`. |
| `binary` | The `ascribe` binary to run, relative to the Astro root. By default, the `ASCRIBE_BIN` environment variable, then the binary `@ascribed/cli` installed. |
| `anchors` {available="ascribe 0.2.0"} | Mark each block of the page with the source file and lines it came from (`data-ascribe-source`; see the [site-render contract](../contracts/site-render.md#7-source-anchors)), for review: `"dev"` in `astro dev` only, `true` in `astro build` too. Use `"dev"` unless the build is for reviewers: `true` puts source file paths, fragments' included, in the published pages. By default, `false`, though `review` turns them on in `astro dev`. |
| `review` {available="ascribe 0.2.0"} | [Review in the site preview](#review-in-the-site-preview): the **Ascribe review** app in `astro dev`'s toolbar. `false` leaves it out. By default, `true`. `astro build` never has it. With `projects`, it reviews one entry: the one an entry's name names (`review: "docs"`), or the first for `true`. |
| `codeTitles` {available="ascribe 0.2.0"} | Show a code block's title above it ([code block titles](#code-block-titles)). `false` leaves titles out, for a site that shows them itself. By default, `true`. |
| `sitemap` {available=next} | `{ exclude: [...] }`: routes of the site's own pages, from the host's root, to leave out of the [sitemap](#search-engines-and-crawlers), such as `["/drafts"]`. |

## What the integration does

- **Fails the Astro build** when `ascribe build` reports an error (the compiler's report is the error), and when `ascribe.toml`'s `[consumer]` `site`, `base-path`, or `trailing-slash` disagrees with Astro's `site`, `base`, or `trailingSlash`. In ascribe 0.2.0, Astro's `trailingSlash: "ignore"` agrees with either value; from the next release, the integration replaces it, as below. `base-path` agrees with Astro's `base` when it's that path or, from the next release, a path under it ([A home page in front of the docs](#a-home-page-in-front-of-the-docs)).
- @available: next
  **Gives Astro its `site`, `base`, and `trailingSlash`** from `[consumer]` when `astro.config.mjs` leaves them out, so the site's address is written once: `site`, `base-path`, and `trailing-slash`. A setting `astro.config.mjs` does give must still agree. With [`projects`](#several-projects-in-one-site), `base` stays the site's. Astro can't tell an integration a setting left out from one set to its default, so `base: "/"` and `trailingSlash: "ignore"` are replaced too.
- **Adds its Markdown plugin** to Astro's Markdown processor, to apply heading ids, image attributes, glossary terms' `data-ascribe-term`, and source anchors: to the default Sätteri processor's `hastPlugins`, or to a `unified()` processor's `rehypePlugins`. Both plugins are exported, as `@ascribed/astro/satteri` and `@ascribed/astro/rehype`, for a processor you configure yourself. Their types come from `satteri` and `@types/hast`, optional peer dependencies: a Sätteri processor already has the first, and a `unified()` one the second.
- **Serves the files pages link to** (other than pages and images) at `<base-path>_ascribe/files/`, in `astro dev` and in the built site.
- @available: next
  **Publishes the outputs for agents** with `[consumer] agents = true`: `llms.txt` and each page's Markdown, in `astro dev` and in the built site ([Publish for AI agents](#publish-for-ai-agents)).
- @available: next
  **Gives each permalink an address** that forwards to its page or heading, in `astro dev` and in the built site ([Permalinks](#permalinks)).
- @available: next
  **Writes the site's sitemap and `robots.txt`** from every project's page records, and gives a layout each page's head ([search engines and crawlers](#search-engines-and-crawlers)), in `astro dev` and in the built site.
- @available: next
  **Sends each Markdown version's response headers** in `astro dev`, and with `[consumer] headers = "netlify"` writes them into the built site's `_headers`, or fails the build when the site's own `public/_headers` lacks them ([response headers](#response-headers)).
- @available: ascribe 0.2.0
  **Shows code block titles**, with a Shiki transformer it adds to Astro's code highlighting ([code block titles](#code-block-titles)).
- @available: ascribe 0.2.0
  **Copies the generated schema** into `.astro/integrations/_ascribed_astro/schema.ts` after each build, for a project outside the Astro root ([the collection](#the-collection)).

### In `astro dev`

- Saving a page, a fragment, an image or other file under the content root, an asset elsewhere in the project that a page uses, or `ascribe.toml` rebuilds. Changes are batched, and builds run one at a time.
- After a successful rebuild, Astro reloads the collection and the page.
- After a failed one, the diagnostics are in Astro's terminal, and pages answer with HTTP 503 until a save builds cleanly again, so you never see a stale page.
- A change to `[project] output-dir` needs a restart of `astro dev`, and says so.
- @available: ascribe 0.2.0
  It writes where it's running (`url` and `build`) to `.ascribe/dev.json` in the project, for the editor's [Open Site Preview](editor.md#site-preview), and removes the file when it stops. The `url` is the project's base path on the dev server.

## A home page in front of the docs
@id {permalink=true}: a-home-page-in-front-of-the-docs
@available: next

`[consumer] base-path` can be a path under Astro's `base`, which leaves the rest of the site to your own pages: a home page at `/` with the docs at `/docs/`, in one Astro site. Set `base-path` to the docs' path, leave Astro's `base` at the site's root, its default, and list the project in [`projects`](#several-projects-in-one-site), even alone. With `build`, the integration gives Astro's `base` the docs' base path, which would put your home page under it; with `projects`, `base` stays the site's:

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

The collection names its entry, `ascribeCollection({ schema, name: "docs" })`, and imports its schema from `.astro/integrations/_ascribed_astro/docs/schema.ts`. A page's route is then `base-path` less Astro's `base`, plus its entry id, so the docs' route goes in `src/pages/docs/[...slug].astro`, with the same `getStaticPaths` as [the route](#the-route). Your home page is `src/pages/index.astro`, or a second Ascribe project ([below](#several-projects-in-one-site)).

Everything Ascribe publishes for the docs is at their base path: the files pages link to (`/docs/_ascribe/files/`), `llms.txt` and each page's Markdown, and, by default, the permalinks (`/docs/go/`). [`permalink-path`](../reference/content-model.md#15-consumer) can put the permalinks anywhere under Astro's `base`, such as `/go/`. A base path outside Astro's `base` still fails the build, since an Astro site serves nothing outside it.

## Several projects in one site
@id {permalink=true}: several-projects-in-one-site
@available: next

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

In `astro dev`, a change rebuilds the project it's in, and each project's folder gets its own `.ascribe/dev.json`, with its own base path. Review covers one project: the entry `review` names, or the first entry when it's `true`. Each removed permalink's page links to the root of the project that retired it, and [`MissingPermalink`](permalinks.md#a-permalink-that-isnt-here) offers the root of the first project with permalinks.

## Publish for AI agents
@available: next

An AI agent reading your docs does better with Markdown than with HTML, and with an index to choose pages from. Turn on the outputs for agents in `ascribe.toml`'s `[consumer]`, beside `site`, and each build publishes them with the site, as the [Web Documentation Delivery Spec](https://agentdocsspec.com/spec/web/) lays them out:

@snippet: code:examples/astro-site/ascribe.toml#agents

- **`llms.txt`** at the base path: every page, by folder, as its title, a link to its Markdown, and its description (the field its type marks `role = "description"`). Past 50,000 characters it's split into one `llms.txt` per folder, which the root file links to.
- **Each page's Markdown** at its URL with `.md`: `/docs/guides/setup` is also `/docs/guides/setup.md`.
- **A pointer** at the top of each page to both, which the elements' stylesheet hides from sight but not from an agent reading the page.

Nothing is written at the host's root for a project under a path. With docs at `/docs/`, an agent reaches `/docs/llms.txt` through each page's pointer, and the spec's checker looks there as well as at `/llms.txt`. A site that wants a file at the root, such as one whose docs are somewhere else, puts its own in Astro's `public/`.

`site` is required, because everything an agent reads links by absolute URL. The integration builds the plain output too, copies it into the built site, and serves it in `astro dev`. The [`ascribe.toml` reference](../reference/content-model.md#15-consumer) says how `llms.txt` is put together.

`ascribe check` then adds two pieces of advice: a page whose description is too long for its line in `llms.txt`, and an `llms.txt` that's still too long when split. Write each page's `description` as one sentence saying what the page is for; it's what an agent picks a page by.

### What your host does

The files are static, so any host serves them. A few things the spec asks are the server's, not the build's:

- **The `.md` files' type.** Serve them as `text/markdown; charset=utf-8`, and `llms.txt` as `text/plain; charset=utf-8`, as `astro dev` does. Check what your host sends (`curl -I` a page's `.md`), and set the header in its configuration if it's something else.
- **Content negotiation.** An agent may ask for a page with `Accept: text/markdown`; answering it with the page's `.md` needs a rule on the server, such as an edge function or middleware that rewrites to `<path>.md` when that header asks for Markdown. Without it, agents still find the `.md` URLs through `llms.txt` and the pointer. On Netlify, an edge function does it: this site's is [`markdown.ts`]({repo}/blob/main/site/netlify/edge-functions/markdown.ts). Whatever answers that way passes on the `.md` file's [response headers](#response-headers), `X-Robots-Tag` and `Link`: the answer is at the page's own address, where no rule for the `.md` path applies, and on Netlify a function's response gets none of `_headers` at all.
- **Status codes and caching.** A missing page should answer 404, not 200 with a page saying so, and the `.md` files and `llms.txt` should be cached no longer than the pages.
- **Bot protection.** A rule that challenges unfamiliar clients blocks agents too; let them read `llms.txt` and the `.md` files.

## Search engines and crawlers
@available: next

Each build writes a record of every page, `_ascribe/pages.json` ([output-layout contract](../contracts/output-layout.md#13-the-page-records)), with its title, description, canonical address, and [`listing`](../reference/content-model.md#listing-a-page), whether the page set it or a rule of [`[listing]`](../reference/content-model.md#22-listing) did, for a folder, a type, or the whole site, and what the project says to crawlers in [`[crawlers]`](../reference/content-model.md#21-crawlers): what each kind of AI crawler may do, and its own `robots.txt` lines. The integration writes what a search engine reads from those records, so the head, the sitemap, `robots.txt`, and `llms.txt` can't disagree about a page.

### The head

A layout includes `Head.astro` in `<head>`, given the page's collection entry, and writes none of what it writes:

```astro
---
import Head from "@ascribed/astro/Head.astro";
import Elements from "@ascribed/astro/Elements.astro";
const { entry } = Astro.props;
---
<head>
  <meta charset="utf-8" />
  <meta name="viewport" content="width=device-width, initial-scale=1" />
  <Head
    entry={entry}
    title={(title, record) => (record.route === "/" ? "Loom" : `${title} · Loom`)}
    name="Loom"
    card={{ src: "/social-card.png", alt: "Loom's docs", width: 1200, height: 630 }}
  />
  <Elements />
</head>
```

From the page's record, it writes:

- `<title>`, and `<meta property="og:title">` with the same text;
- `<meta name="description">` and `<meta property="og:description">`;
- `<link rel="canonical">` and `<meta property="og:url">`, when `[consumer] site` is set, and `<meta property="og:type" content="website">`;
- `<meta name="robots" content="noindex">` for a page that's `unlisted`, and `<meta name="robots" content="nosnippet">` for one that's `unquoted`, which asks a search engine to show no text from it in a result, or in an AI answer inside its search product; it asks nothing of a crawler gathering training data;
- `<link rel="alternate" type="text/markdown">` to the page's Markdown version, when the build publishes one ([`[consumer] agents`](#publish-for-ai-agents)) and the page isn't `unlisted`;
- the page's breadcrumbs, as [`BreadcrumbList`](https://developers.google.com/search/docs/appearance/structured-data/breadcrumb) structured data, when there are two steps or more.

From the site, by property, since the record doesn't hold them:

- `title`, a function from the page's title and its record to the whole title. The record has the page's `route`, so a site can title its home page with its name alone. Without `title`, the title is the page's own, and a page whose title is missing gets no `<title>`.
- `name`, the site's name, for `<meta property="og:site_name">`, and, on the page at the host's root (`/`), the site's name as [`WebSite`](https://developers.google.com/search/docs/appearance/site-names) structured data, which a search engine shows as the name in a result. Google reads it only on the home page.
- `card`, the page's social card: `src`, its image, from the host's root or absolute; `alt`, what it shows; and `width` and `height`, in pixels. It's the `og:image` tags a link preview shows. Without it, there are no image tags.

The layout keeps the character set, the viewport, icons, theme colors, styles, the elements, and tags for one network that Open Graph doesn't cover, such as `<meta name="twitter:card">`. A tag `Head.astro` writes that the layout writes too is in the head twice; `astro dev` warns once about each such tag, naming it and the first page it's on. It's a warning and not an error, since a site may have a reason.

**Breadcrumbs.** A page's place in the site, from its route's folders: the page at each folder of its route that has one, named by its title, down to the page itself. `/docs/guides/install/` has the page at `/docs/`, the page at `/docs/guides/` if there is one, and itself. A folder with no page isn't a step, since there's nothing to link to. The steps are the record's `breadcrumbs`, so a layout can draw visible breadcrumbs from the same list with `pageRecord(entry)`. They change how a search result is displayed, and nothing else. They aren't the site's navigation: the content model has none, so a page's folders are all that say where it is.

**A page with no record**, such as an Astro page of the site's own, gets nothing from `Head.astro`, and writes its own `<title>`, description, canonical link, and link preview. A page that isn't an Ascribe page and shouldn't be listed writes its own `<meta name="robots" content="noindex">`, and goes in the `sitemap` option's `exclude`, since the integration reads no HTML and can't see the rule. A site whose home page isn't an Ascribe page writes the site's name there itself:

```html
<script type="application/ld+json">
  {"@context": "https://schema.org", "@type": "WebSite",
   "name": "Loom", "url": "https://loom.example.com/"}
</script>
```

`pageRecord(entry)`, from `@ascribed/astro/content`, gives a layout the record itself, for anything else it writes from it.

### The sitemap

`astro build` writes one sitemap for the site, `sitemap.xml` at Astro's `base`, and `astro dev` serves it there. It lists by absolute address, with no dates:

- every page of every project that isn't `unlisted`, when the site has a route that shows it: a project whose collection the site only quotes from adds nothing;
- every other page Astro built, such as a changelog page of the site's own, except a permalink's forwarding page, the not-found page, and the routes in the `sitemap` option's `exclude`.

`astro dev` knows only the routes of the site's own that have no parameters, so a page of the site's own that a route's `getStaticPaths` lists is in the built sitemap only.

A sitemap needs absolute addresses, so a site with no `site` (`[consumer] site`, or Astro's) gets none, and the build warns. A site with its own `sitemap.xml` in `public/` keeps it, the integration writes none, and `robots.txt` points to the site's.

### `robots.txt`

At the host's root, the integration writes one `robots.txt` for the site, and `astro dev` serves it:

```text
User-agent: GPTBot
User-agent: ClaudeBot
User-agent: Google-Extended
Disallow: /

User-agent: *
Disallow: /docs/drafts/

User-agent: ExampleBot
Disallow: /

Sitemap: https://docs.example.com/sitemap.xml
```

It writes a group for each purpose a project disallows, such as `training = "disallow"`, naming every crawler of that purpose Ascribe knows ([crawlers by purpose](../reference/content-model.md#crawlers-by-purpose)), each purpose once. Then it opens a `User-agent: *` group with each project's lines for every crawler, the lines before its first `User-agent` line, then adds the groups each project wrote whole, then the sitemap's address. Each project's paths have its base path before them. With no sitemap, there's no last line, and with nothing to say at all, no file: a project with no `[crawlers]` and an address gets the sitemap's line alone.

- **Projects of one site must agree on each purpose,** since they share the file. One that allows a purpose and one that disallows it fail `astro build`, naming both, and `astro dev` serves no `robots.txt` until they agree. A project that leaves a purpose out agrees with either.
- **A site with its own `robots.txt`** in `public/` keeps it, and the integration writes none. The build says so, with the lines Ascribe would have written; copy them into your file.
- **A site whose Astro `base` isn't `/`** has no host root to write to, and a crawler reads `robots.txt` only there. The sitemap goes under the base, no `robots.txt` is written, and the build prints the lines the host's own `robots.txt` needs, the sitemap's address among them.

### Response headers

A page's Markdown version is a file with no head, so the rule its page's head gives a crawler has to be a response header, and only the host sends those. Each build lists what each Markdown version needs in `_ascribe/headers.json` ([output-layout contract](../contracts/output-layout.md#14-the-headers)):

- `X-Robots-Tag: noindex` for a page that's `unlisted`, and `nosnippet` for one that's `unquoted`, the same rule as its head;
- `Link: <https://docs.example.com/docs/guides/install/>; rel="canonical"` on every one, naming the page, so a search engine lists the page and not its Markdown copy, while an agent can still fetch either.

`astro dev` sends them with each `.md` file, so you can try them with no host:

```sh
$ curl -sI http://localhost:4321/docs/internal/runbook.md | grep -i x-robots-tag
x-robots-tag: noindex
```

**On Netlify,** set `headers = "netlify"` in `[consumer]`, and `astro build` writes the rules into `_headers` at the root of the built site, which Netlify reads:

```toml
[consumer]
site = "https://docs.example.com"
base-path = "/docs/"
agents = true
headers = "netlify"
```

```text
/docs/internal/runbook.md
  Link: <https://docs.example.com/docs/internal/runbook/>; rel="canonical"
  X-Robots-Tag: noindex
```

Each rule is for one exact path, never a pattern, so a rule for a page can't reach its neighbor. With several projects, the file holds the rules of each project that sets `headers`.

- **A site with its own `_headers`** in `public/` keeps it, and the integration never rewrites it. The build checks that it has Ascribe's lines, and fails when it doesn't, with the lines to add as its message. A line counts when it's under the same path, with the same value; the case of a header's name doesn't matter.
- **A site whose Astro `base` isn't `/`** has no root to write `_headers` to, so the build prints the lines instead.

**On another host,** read the list after the build and give each path its headers in the host's own configuration. For nginx, a script that writes a `location` block for each rule, for the server's configuration to include:

```js
// After `astro build`: headers.conf, with a block for each Markdown version.
import { readFileSync, writeFileSync } from "node:fs";
const { headers } = JSON.parse(
  readFileSync(".ascribe/build/site/site/_ascribe/headers.json", "utf8"),
);
const blocks = headers.map(({ path, headers }) => {
  const lines = Object.entries(headers).map(([name, value]) => `  add_header ${name} '${value}';`);
  return `location = ${path} {\n${lines.join("\n")}\n}\n`;
});
writeFileSync("headers.conf", blocks.join(""));
```

Cloudflare Pages reads a `_headers` file in the same form as Netlify's, but takes at most 100 rules, one for each Markdown version here; a larger site gives the rules to a Worker instead.

## Permalinks
@available: next

The integration gives each [permalink](permalinks.md) an address, in `astro build` and in `astro dev`; [With Astro](permalinks.md#with-astro) says what it serves and where. [Permalinks](permalinks.md) covers the rest: linking to them from your product, checking those links in its CI, retiring one, and [`MissingPermalink`](permalinks.md#a-permalink-that-isnt-here) for your 404 page.

## Review in the site preview
@id {permalink=true}: review-in-the-site-preview
@available: ascribe 0.2.0

In `astro dev`, Astro's dev toolbar has an **Ascribe review** app. It shows a pull request's changes and review comments on the real page, in your site's layout: the same marks and threads as the editor's [page preview](editor.md#review-in-the-preview), from the same [`@ascribed/review`]({repo}/blob/main/packages/review/README.md) overlay. Nothing runs until you open the app: opening it starts review, and after **Stop Review**, the panel offers **Start Review**. [Review](review.md) walks through reviewing a pull request with it; this section is the reference for the site's side.

The app's panel sits above the toolbar, in one row: the pull request and the base (**#12 against main**), your place in the changes ("3 of 10 on this page", click it for the breakdown), **Changes / As it will be / As it was**, next and previous change, **Comments**, **Refresh**, and **More review actions** (**⋯**), whose **Prompt agent** items copy a prompt for your agent about the page's changes or a fragment's pages ([In review](agents.md#in-review)). Each thread's **Prompt agent** copies one about its comment. Past the last change it offers the next changed page, by its title. When `ascribe check` finds errors in the working tree, the panel says how many, since a page with an error may not show as it will once it's fixed ([`ascribe diff`](../reference/cli.md#cli-diff) explains why). Close it to get the page back; it stays closed or open as you move between pages, and review stays on until **Stop Review** or until `astro dev` stops. With the panel closed, a dot on the toolbar button says you have comments you haven't submitted.

Starting review compares the checkout with the base of its branch's pull request, or, with no pull request, with the default branch (the first of `origin/HEAD`, `origin/main`, `origin/master`, `main`, and `master` that exists), from where the branch left it, as [`ascribe diff`](../reference/cli.md#cli-diff) does. It compares again after each rebuild, so the marks follow your edits on save. Comments, replies, and resolving work as in the page preview, and [map to the pull request](review.md#how-comments-map-to-the-pull-request) the same way: new comments are unsent until **Submit review…** in the panel sends them. Each mark's label and each thread's **Open source** opens the file at the line in your editor, through Vite's open-in-editor: set `LAUNCH_EDITOR` (for example, `LAUNCH_EDITOR=code`) to choose which.

Comments need the [GitHub CLI](https://cli.github.com), signed in (`gh auth login`); the dev server runs it, and no token reaches the page. Without it, or without a pull request, the app shows the changes only and says why. The page and the dev server talk over Vite's own connection, which only pages from the dev server can open. If your Vite config loosens that (`server.cors: true`, `server.allowedHosts: true`, or `legacy.skipWebSocketTokenCheck`), any web page open in your browser could use it, so the app keeps comments off and says which setting to change. It keeps them off too when the dev server listens on the network (`astro dev --host`, or a `server.host` other than localhost), where anyone who can reach the site could comment as you; the changes still show.

The marks, the threads, and the panel are light or dark as the page is, not as your system is, so a light-only site stays readable when your system is in dark mode. The app reads the page's background (or, with none, its `color-scheme`) and sets `data-ascribe-scheme` on the root element; a site that sets `data-ascribe-scheme="light"` or `"dark"` there itself keeps its own.

Review is on for the whole dev server, not one tab: once you start it, every page of the site that loads, in any tab or browser, shows the marks and threads without a click, until **Stop Review**.

Two pages have nothing to place:

- **A route that isn't an Ascribe page** (a changelog page of your own, an index) says so, and lists the pages the change touches, each a link.
- **A page whose layout drops the anchors**: the marks are placed by the `data-ascribe-source` attributes on each block, so a layout or component that rebuilds the content without them leaves nothing to mark. The panel says so, and lists the page's changes and threads instead, each with **Open source**.

Comparing again after a save runs `ascribe diff` once, which takes about as long as an `ascribe build` of the project: under 50 ms on `examples/astro-site`. `astro dev` logs how long the first comparison took when review starts.

## Styling

The elements render into the page (no shadow DOM), so your site's styles apply to them, and they're themed with CSS custom properties such as `--ascribe-tip-color` and `--ascribe-tab-active-color`. The [element library reference](../reference/elements.md) lists them, and shows how to style your own note types and lifecycle states. Their names don't change, but their default colors are Ascribe's and may change in a release, as the changelog notes, so set the ones your site needs fixed.

### Code block titles
@available: ascribe 0.2.0

A code block's title, such as a [`@snippet`](../reference/directives.md#snippet)'s `title`, is in its info string (`` ```mjs title="astro.config.mjs" ``). Astro's highlighting ignores it, so the integration adds a Shiki transformer that puts a titled block in a figure with the title as its caption:

```html
<figure class="code-title">
  <figcaption>astro.config.mjs</figcaption>
  <pre class="astro-code">…</pre>
</figure>
```

The figure has your browser's default margins, and the caption no style of its own; style `figure.code-title` and its `figcaption` to suit your site. Titles need Astro's default highlighting, Shiki; with `syntaxHighlight: "prism"` or `false`, they aren't shown.

## Other Markdown processors

The integration supports Astro's default processor (Sätteri) and `unified()`. Any other processor is an error at startup, since heading ids and image attributes would be lost.
