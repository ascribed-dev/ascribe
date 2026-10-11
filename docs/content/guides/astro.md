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

Put `ascribe.toml` in the Astro project's root, beside `astro.config.mjs`. Its `[consumer]` settings must agree with Astro's `site`, `base`, and `trailingSlash`, because Ascribe writes every link ([a home page in front of the docs](#a-home-page-in-front-of-the-docs) says how `base-path` can differ from `base`):

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

@available: ascribe 0.2.0
The example's content model sets `inline = "code"` on `title`, so a title can show code, as "`` `loom.yaml` options ``". `entry.data.title` is then plain text, for `<title>` and search, and `entry.data.formatted.title` is the title as HTML, with each code span a `<code>`, for the heading and navigation (`set:html`). Without the setting, there's no `formatted`, and the layout shows `title`. See [code in a field](../reference/content-model.md#53-code-in-a-field).

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

## What the integration does

- **Fails the Astro build** when `ascribe build` reports an error (the compiler's report is the error), and when `ascribe.toml`'s `[consumer]` `site`, `base-path`, or `trailing-slash` disagrees with Astro's `site`, `base`, or `trailingSlash`. Astro's `trailingSlash: "ignore"` agrees with either value. `base-path` agrees with Astro's `base` when it's that path or, from the next release, a path under it ([A home page in front of the docs](#a-home-page-in-front-of-the-docs)).
- **Adds its Markdown plugin** to Astro's Markdown processor, to apply heading ids, image attributes, glossary terms' `data-ascribe-term`, and source anchors: to the default Sätteri processor's `hastPlugins`, or to a `unified()` processor's `rehypePlugins`. Both plugins are exported, as `@ascribed/astro/satteri` and `@ascribed/astro/rehype`, for a processor you configure yourself. Their types come from `satteri` and `@types/hast`, optional peer dependencies: a Sätteri processor already has the first, and a `unified()` one the second.
- **Serves the files pages link to** (other than pages and images) at `<base-path>_ascribe/files/`, in `astro dev` and in the built site.
- @available: next
  **Publishes the outputs for agents** with `[consumer] agents = true`: `llms.txt` and each page's Markdown, in `astro dev` and in the built site ([Publish for AI agents](#publish-for-ai-agents)).
- @available: next
  **Gives each permalink an address** that forwards to its page or heading, in `astro dev` and in the built site ([Permalinks](#permalinks)).
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

`[consumer] base-path` can be a path under Astro's `base`, which leaves the rest of the site to your own pages: a home page at `/` with the docs at `/docs/`, in one Astro site. Set Astro's `base` to the site's root and `base-path` to the docs' path:

```js
// astro.config.mjs
export default defineConfig({
  site: "https://docs.example.com",
  base: "/",
  trailingSlash: "never",
  integrations: [ascribe({ build: "site" })],
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

A page's route is then `base-path` less Astro's `base`, plus its entry id, so the docs' route goes in `src/pages/docs/[...slug].astro`, with the same `getStaticPaths` as [the route](#the-route). Your home page is `src/pages/index.astro`, or a second Ascribe project ([below](#several-projects-in-one-site)).

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

`site` is required, because everything an agent reads links by absolute URL. The integration builds the plain output too, copies it into the built site, and serves it in `astro dev`. The [`ascribe.toml` reference](../reference/content-model.md#15-consumer) says how `llms.txt` is put together.

`ascribe check` then adds two pieces of advice: a page whose description is too long for its line in `llms.txt`, and an `llms.txt` that's still too long when split. Write each page's `description` as one sentence saying what the page is for; it's what an agent picks a page by.

### What your host does

The files are static, so any host serves them. A few things the spec asks are the server's, not the build's:

- **The `.md` files' type.** Serve them as `text/markdown; charset=utf-8`, and `llms.txt` as `text/plain; charset=utf-8`, as `astro dev` does. Check what your host sends (`curl -I` a page's `.md`), and set the header in its configuration if it's something else.
- **Content negotiation.** An agent may ask for a page with `Accept: text/markdown`; answering it with the page's `.md` needs a rule on the server, such as an edge function or middleware that rewrites to `<path>.md` when that header asks for Markdown. Without it, agents still find the `.md` URLs through `llms.txt` and the pointer. On Netlify, an edge function does it: this site's is [`markdown.ts`]({repo}/blob/main/site/netlify/edge-functions/markdown.ts).
- **Status codes and caching.** A missing page should answer 404, not 200 with a page saying so, and the `.md` files and `llms.txt` should be cached no longer than the pages.
- **Bot protection.** A rule that challenges unfamiliar clients blocks agents too; let them read `llms.txt` and the `.md` files.

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

The elements render into the page (no shadow DOM), so your site's styles apply to them, and they're themed with CSS custom properties such as `--ascribe-tip-color` and `--ascribe-tab-active-color`. The [element library's README]({repo}/blob/main/packages/elements/README.md#theming) lists them, and shows how to style your own note types and lifecycle states. Their names don't change, but their default colors are Ascribe's and may change in a release, as the changelog notes, so set the ones your site needs fixed.

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
