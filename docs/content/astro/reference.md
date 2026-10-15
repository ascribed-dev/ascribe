---
type: reference
title: "`@ascribed/astro` reference"
description: "What the Astro integration takes, what it does to a site, and what it writes for search engines, crawlers, and review."
permalink: astro-reference
---

Everything `@ascribed/astro` takes and does, once a site is set up: its options, what it adds to Astro's build and dev server, the head, sitemap, `robots.txt`, and response headers it writes, review in the site preview, and styling. [Astro](index.md) sets a site up, puts the docs behind a home page or several projects in one site, and publishes them for AI agents.

## Options

| Option | Meaning |
|---|---|
| `build` | The build whose site output is the collection: a build name in `ascribe.toml`. Required, unless `projects` is given. |
| `project` | The directory holding `ascribe.toml`, relative to the Astro root. By default, the root. |
| `projects` {available="ascribe 0.3.0"} | [Several projects in one site](index.md#several-projects-in-one-site): a list of entries, each `{ name, build, project }`, in place of `build` and `project`. |
| `binary` | The `ascribe` binary to run, relative to the Astro root. By default, the `ASCRIBE_BIN` environment variable, then the binary `@ascribed/cli` installed. |
| `anchors` {available="ascribe 0.2.0"} | Mark each block of the page with the source file and lines it came from (`data-ascribe-source`; see the [site-render contract](../contracts/site-render.md#7-source-anchors)), for review: `"dev"` in `astro dev` only, `true` in `astro build` too. Use `"dev"` unless the build is for reviewers: `true` puts source file paths, fragments' included, in the published pages. By default, `false`, though `review` turns them on in `astro dev`. |
| `review` {available="ascribe 0.2.0"} | [Review in the site preview](reference.md#review-in-the-site-preview): the **Ascribe review** app in `astro dev`'s toolbar. `false` leaves it out. By default, `true`. `astro build` never has it. With `projects`, it reviews one entry: the one an entry's name names (`review: "docs"`), or the first for `true`. |
| `codeTitles` {available="ascribe 0.2.0"} | Show a code block's title above it ([code block titles](reference.md#code-block-titles)). `false` leaves titles out, for a site that shows them itself. By default, `true`. |
| `sitemap` {available="ascribe 0.3.0"} | `{ exclude: [...] }`: routes of the site's own pages, from the host's root, to leave out of the [sitemap](reference.md#search-engines-and-crawlers), such as `["/drafts"]`. |

## What the integration does

- **Fails the Astro build** when `ascribe build` reports an error (the compiler's report is the error), and when `ascribe.toml`'s `[consumer]` `site`, `base-path`, or `trailing-slash` disagrees with Astro's `site`, `base`, or `trailingSlash`. In ascribe 0.2.0, Astro's `trailingSlash: "ignore"` agrees with either value; from the next release, the integration replaces it, as below. `base-path` agrees with Astro's `base` when it's that path or, from the next release, a path under it ([A home page in front of the docs](index.md#a-home-page-in-front-of-the-docs)).
- @available: ascribe 0.3.0
  **Gives Astro its `site`, `base`, and `trailingSlash`** from `[consumer]` when `astro.config.mjs` leaves them out, so the site's address is written once: `site`, `base-path`, and `trailing-slash`. A setting `astro.config.mjs` does give must still agree. With [`projects`](index.md#several-projects-in-one-site), `base` stays the site's. Astro can't tell an integration a setting left out from one set to its default, so `base: "/"` and `trailingSlash: "ignore"` are replaced too.
- **Adds its Markdown plugin** to Astro's Markdown processor, to apply heading ids, image attributes, glossary terms' `data-ascribe-term`, and source anchors: to the default Sätteri processor's `hastPlugins`, or to a `unified()` processor's `rehypePlugins`. Both plugins are exported, as `@ascribed/astro/satteri` and `@ascribed/astro/rehype`, for a processor you configure yourself. Their types come from `satteri` and `@types/hast`, optional peer dependencies: a Sätteri processor already has the first, and a `unified()` one the second.
- **Serves the files pages link to** (other than pages and images) at `<base-path>_ascribe/files/`, in `astro dev` and in the built site.
- @available: ascribe 0.3.0
  **Publishes the outputs for agents** with `[consumer] agents = true`: `llms.txt` and each page's Markdown, in `astro dev` and in the built site ([Publish for AI agents](index.md#publish-for-ai-agents)).
- @available: ascribe 0.3.0
  **Gives each permalink an address** that forwards to its page or heading, in `astro dev` and in the built site ([With Astro](../permalinks/index.md#with-astro)).
- @available: ascribe 0.3.0
  **Writes the site's sitemap and `robots.txt`** from every project's page records, and gives a layout each page's head ([search engines and crawlers](reference.md#search-engines-and-crawlers)), in `astro dev` and in the built site.
- @available: ascribe 0.3.0
  **Sends each Markdown version's response headers** in `astro dev`, and with `[consumer] headers = "netlify"` writes them into the built site's `_headers`, or fails the build when the site's own `public/_headers` lacks them ([response headers](reference.md#response-headers)).
- @available: next
  **Serves every line of a versioned build** from one collection, builds every line in `astro build` and `astro dev`, and serves each line's linked files at its base path ([Versions](index.md#versions)). Each entry's data has `line`, the line it's from, and `current`; the generated schema doesn't type them. A permalink forwards to the current line, and each line's outputs for agents are published at its base path, with its headers.
- @available: ascribe 0.2.0
  **Shows code block titles**, with a Shiki transformer it adds to Astro's code highlighting ([code block titles](reference.md#code-block-titles)).
- @available: ascribe 0.2.0
  **Copies the generated schema** into `.astro/integrations/_ascribed_astro/schema.ts` after each build, for a project outside the Astro root ([the collection](index.md#the-collection)).

### In `astro dev`

- Saving a page, a fragment, an image or other file under the content root, an asset elsewhere in the project that a page uses, or `ascribe.toml` rebuilds. Changes are batched, and builds run one at a time.
- After a successful rebuild, Astro reloads the collection and the page. A versioned build rebuilds every line.
- After a failed one, the diagnostics are in Astro's terminal, and pages answer with HTTP 503 until a save builds cleanly again, so you never see a stale page.
- A change to `[project] output-dir` needs a restart of `astro dev`, and says so.
- @available: ascribe 0.2.0
  It writes where it's running (`url` and `build`) to `.ascribe/dev.json` in the project, for the editor's [Open Site Preview](../editor/index.md#site-preview), and removes the file when it stops. The `url` is the project's base path on the dev server.

## Search engines and crawlers
@available: ascribe 0.3.0

Each build writes a record of every page, `_ascribe/pages.json` ([output-layout contract](../contracts/output-layout.md#13-the-page-records)), with its title, description, canonical address, and [`listing`](../content-model/index.md#listing-a-page), whether the page set it or a rule of [`[listing]`](../content-model/checks.md#22-listing) did, for a folder, a type, or the whole site, and what the project says to crawlers in [`[crawlers]`](../content-model/checks.md#21-crawlers): what each kind of AI crawler may do, and its own `robots.txt` lines. The integration writes what a search engine reads from those records, so the head, the sitemap, `robots.txt`, and `llms.txt` can't disagree about a page.

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
- `<link rel="alternate" type="text/markdown">` to the page's Markdown version, when the build publishes one ([`[consumer] agents`](../content-model/output.md#consumer)) and the page isn't `unlisted`;
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

- every page of every project that isn't `unlisted`, and whose canonical address is its own, when the site has a route that shows it: a project whose collection the site only quotes from adds nothing, and an older line of a [versioned build](index.md#versions) adds only the pages the current line has nothing in the place of;
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

It writes a group for each purpose a project disallows, such as `training = "disallow"`, naming every crawler of that purpose Ascribe knows ([crawlers by purpose](../content-model/checks.md#crawlers-by-purpose)), each purpose once. Then it opens a `User-agent: *` group with each project's lines for every crawler, the lines before its first `User-agent` line, then adds the groups each project wrote whole, then the sitemap's address. Each project's paths have its base path before them. With no sitemap, there's no last line, and with nothing to say at all, no file: a project with no `[crawlers]` and an address gets the sitemap's line alone.

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

**On another host,** read the list after the build and give each path its headers in the host's own configuration. For nginx, a script that writes a `map` for each header, from a Markdown version's path to the header's value, for the `http` block to include:

```js
// After `astro build`: headers.conf, a map for each header Ascribe's rules give.
import { readFileSync, writeFileSync } from "node:fs";
const { headers } = JSON.parse(
  readFileSync(".ascribe/build/site/site/_ascribe/headers.json", "utf8"),
);
const names = [...new Set(headers.flatMap((rule) => Object.keys(rule.headers)))];
const maps = names.map((name) => {
  const variable = `$ascribe_${name.toLowerCase().replaceAll("-", "_")}`;
  const lines = headers
    .filter((rule) => name in rule.headers)
    .map((rule) => `  '${rule.path}' '${rule.headers[name]}';`);
  return `map $uri ${variable} {\n  default "";\n${lines.join("\n")}\n}\n`;
});
writeFileSync("headers.conf", maps.join(""));
```

The server then sends each header from its variable, and nginx leaves out a header whose value is empty, so every other file is answered as before:

```nginx
http {
  include headers.conf;

  server {
    add_header Link $ascribe_link;
    add_header X-Robots-Tag $ascribe_x_robots_tag;
  }
}
```

A `location` with `add_header` lines of its own, such as one that gives `.md` files their type, doesn't inherit the server's, so it repeats these two.

Cloudflare Pages reads a `_headers` file in the same form as Netlify's, but takes at most 100 rules, one for each Markdown version here; a larger site gives the rules to a Worker instead.

## Review in the site preview
@id {permalink=true}: review-in-the-site-preview
@available: ascribe 0.2.0

In `astro dev`, Astro's dev toolbar has an **Ascribe review** app. It shows a pull request's changes and review comments on the real page, in your site's layout: the same marks and threads as the editor's [page preview](../editor/index.md#review-in-the-preview), from the same [`@ascribed/review`]({repo}/blob/main/packages/review/README.md) overlay. Nothing runs until you open the app: opening it starts review, and after **Stop Review**, the panel offers **Start Review**. [Review](../review/index.md) walks through reviewing a pull request with it; this section is the reference for the site's side.

The app's panel sits above the toolbar, in one row: the pull request and the base (**#12 against main**), your place in the changes ("3 of 10 on this page", click it for the breakdown), **Changes / As it will be / As it was**, next and previous change, **Comments**, **Refresh**, and **More review actions** (**⋯**), whose **Prompt agent** items copy a prompt for your agent about the page's changes or a fragment's pages ([In review](../agents/index.md#in-review)). Each thread's **Prompt agent** copies one about its comment. Past the last change it offers the next changed page, by its title. When `ascribe check` finds errors in the working tree, the panel says how many, since a page with an error may not show as it will once it's fixed ([`ascribe diff`](../review/diff.md#cli-diff) explains why). Close it to get the page back; it stays closed or open as you move between pages, and review stays on until **Stop Review** or until `astro dev` stops. With the panel closed, a dot on the toolbar button says you have comments you haven't submitted.

Starting review compares the checkout with the base of its branch's pull request, or, with no pull request, with the default branch (the first of `origin/HEAD`, `origin/main`, `origin/master`, `main`, and `master` that exists), from where the branch left it, as [`ascribe diff`](../review/diff.md#cli-diff) does. It compares again after each rebuild, so the marks follow your edits on save. Comments, replies, and resolving work as in the page preview, and [map to the pull request](../review/index.md#how-comments-map-to-the-pull-request) the same way: new comments are unsent until **Submit review…** in the panel sends them. Each mark's label and each thread's **Open source** opens the file at the line in your editor, through Vite's open-in-editor: set `LAUNCH_EDITOR` (for example, `LAUNCH_EDITOR=code`) to choose which.

Comments need the [GitHub CLI](https://cli.github.com), signed in (`gh auth login`); the dev server runs it, and no token reaches the page. Without it, or without a pull request, the app shows the changes only and says why. The page and the dev server talk over Vite's own connection, which only pages from the dev server can open. If your Vite config loosens that (`server.cors: true`, `server.allowedHosts: true`, or `legacy.skipWebSocketTokenCheck`), any web page open in your browser could use it, so the app keeps comments off and says which setting to change. It keeps them off too when the dev server listens on the network (`astro dev --host`, or a `server.host` other than localhost), where anyone who can reach the site could comment as you; the changes still show.

The marks, the threads, and the panel are light or dark as the page is, not as your system is, so a light-only site stays readable when your system is in dark mode. The app reads the page's background (or, with none, its `color-scheme`) and sets `data-ascribe-scheme` on the root element; a site that sets `data-ascribe-scheme="light"` or `"dark"` there itself keeps its own.

Review is on for the whole dev server, not one tab: once you start it, every page of the site that loads, in any tab or browser, shows the marks and threads without a click, until **Stop Review**.

Two pages have nothing to place:

- **A route that isn't an Ascribe page** (a changelog page of your own, an index) says so, and lists the pages the change touches, each a link.
- **A page whose layout drops the anchors**: the marks are placed by the `data-ascribe-source` attributes on each block, so a layout or component that rebuilds the content without them leaves nothing to mark. The panel says so, and lists the page's changes and threads instead, each with **Open source**.

Comparing again after a save runs `ascribe diff` once, which takes about as long as an `ascribe build` of the project: under 50 ms on `examples/astro-site`. `astro dev` logs how long the first comparison took when review starts.

## Styling

The elements render into the page (no shadow DOM), so your site's styles apply to them, and they're themed with CSS custom properties such as `--ascribe-tip-color` and `--ascribe-tab-active-color`. The [element library reference](elements.md) lists them, and shows how to style your own note types and lifecycle states. Their names don't change, but their default colors are Ascribe's and may change in a release, as the changelog notes, so set the ones your site needs fixed.

### Code block titles
@available: ascribe 0.2.0

A code block's title, such as a [`@snippet`](../writing/directives.md#snippet)'s `title`, is in its info string (`` ```mjs title="astro.config.mjs" ``). Astro's highlighting ignores it, so the integration adds a Shiki transformer that puts a titled block in a figure with the title as its caption:

```html
<figure class="code-title">
  <figcaption>astro.config.mjs</figcaption>
  <pre class="astro-code">…</pre>
</figure>
```

The figure has your browser's default margins, and the caption no style of its own; style `figure.code-title` and its `figcaption` to suit your site. Titles need Astro's default highlighting, Shiki; with `syntaxHighlight: "prism"` or `false`, they aren't shown.

## Other Markdown processors

The integration supports Astro's default processor (Sätteri) and `unified()`. Any other processor is an error at startup, since heading ids and image attributes would be lost.
