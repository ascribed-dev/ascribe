---
type: reference
title: "`@ascribed/astro` reference"
description: "What the Astro integration takes, what it does to a site, and what it writes for search engines, crawlers, and review."
permalink: astro-reference
---

Everything `@ascribed/astro` takes and does, once a site is set up: its options, what it adds to Astro's build and dev server, the components a layout uses, several projects, versions, and permalinks in one site, the head, sitemap, `robots.txt`, and response headers it writes, review in the site preview, and styling. [Astro](index.md) sets a site up.

## Options

| Option | Meaning |
|---|---|
| `build` | The build whose site output is the collection: a build name in `ascribe.toml`. Required, unless `projects` is given. |
| `project` | The directory holding `ascribe.toml`, relative to the Astro root. By default, the root. |
| `projects` {available="ascribe 0.3.0"} | [Several projects](reference.md#several-projects): a list of entries, each `{ name, build, project }`, in place of `build` and `project`. |
| `binary` | The `ascribe` binary to run, relative to the Astro root. By default, the `ASCRIBE_BIN` environment variable, then the binary `@ascribed/cli` installed. |
| `anchors` {available="ascribe 0.2.0"} | Mark each block of the page with the source file and lines it came from (`data-ascribe-source`; see the [site-render contract](../contracts/site-render.md#7-source-anchors)), for review: `"dev"` in `astro dev` only, `true` in `astro build` too. Use `"dev"` unless the build is for reviewers: `true` puts source file paths, fragments' included, in the published pages. By default, `false`, though `review` turns them on in `astro dev`. |
| `review` {available="ascribe 0.2.0"} | [Review in the site preview](reference.md#review-in-the-site-preview): the **Ascribe review** app in `astro dev`'s toolbar. `false` leaves it out. By default, `true`. `astro build` never has it. With `projects`, it reviews one entry: the one an entry's name names (`review: "docs"`), or the first for `true`. |
| `codeTitles` {available="ascribe 0.2.0"} | Show a code block's title above it ([code block titles](reference.md#code-block-titles)). `false` leaves titles out, for a site that shows them itself. By default, `true`. |
| `sitemap` {available="ascribe 0.3.0"} | `{ exclude: [...] }`: routes of the site's own pages, from the host's root, to leave out of the [sitemap](reference.md#search-engines-and-crawlers), such as `["/drafts"]`. |

## What the integration does

- **Fails the Astro build** when `ascribe build` reports an error (the compiler's report is the error), and when `ascribe.toml`'s `[consumer]` `site`, `base-path`, or `trailing-slash` disagrees with Astro's `site`, `base`, or `trailingSlash`. In ascribe 0.2.0, Astro's `trailingSlash: "ignore"` agrees with either value; from the next release, the integration replaces it, as below. `base-path` agrees with Astro's `base` when it's that path or, from the next release, a path under it ([A home page in front of the docs](more-than-docs.md#a-home-page-in-front-of-the-docs)).
- @available: ascribe 0.3.0
  **Gives Astro its `site`, `base`, and `trailingSlash`** from `[consumer]` when `astro.config.mjs` leaves them out, so the site's address is written once: `site`, `base-path`, and `trailing-slash`. A setting `astro.config.mjs` does give must still agree. With [`projects`](reference.md#several-projects), `base` stays the site's. Astro can't tell an integration a setting left out from one set to its default, so `base: "/"` and `trailingSlash: "ignore"` are replaced too.
- **Adds its Markdown plugin** to Astro's Markdown processor, to apply heading ids, image attributes, glossary terms' `data-ascribe-term`, and source anchors: to the default Sätteri processor's `hastPlugins`, or to a `unified()` processor's `rehypePlugins`. Both plugins are exported, as `@ascribed/astro/satteri` and `@ascribed/astro/rehype`, for a processor you configure yourself. Their types come from `satteri` and `@types/hast`, optional peer dependencies: a Sätteri processor already has the first, and a `unified()` one the second.
- **Serves the files pages link to** (other than pages and images) at `<base-path>_ascribe/files/`, in `astro dev` and in the built site.
- @available: ascribe 0.3.0
  **Publishes the outputs for agents** with `[consumer] agents = true`: `llms.txt` and each page's Markdown, in `astro dev` and in the built site ([Publish for AI agents](agents.md)).
- @available: ascribe 0.3.0
  **Gives each permalink an address** that forwards to its page or heading, in `astro dev` and in the built site ([permalinks](reference.md#permalinks)).
- @available: ascribe 0.3.0
  **Writes the site's sitemap and `robots.txt`** from every project's page records, and gives a layout each page's head ([search engines and crawlers](reference.md#search-engines-and-crawlers)), in `astro dev` and in the built site.
- @available: ascribe 0.3.0
  **Sends each Markdown version's response headers** in `astro dev`, and with `[consumer] headers = "netlify"` writes them into the built site's `_headers`, or fails the build when the site's own `public/_headers` lacks them ([response headers](reference.md#response-headers)).
- @available: ascribe 0.3.2
  **Serves every line of a versioned build** from one collection, builds every line in `astro build` and `astro dev`, and serves each line's linked files at its base path ([versions](reference.md#versions)). Each entry's data has `line`, the line it's from, and `current`; the generated schema doesn't type them. A permalink forwards to the current line, and each line's outputs for agents are published at its base path, with its headers.
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
  It writes where it's running (`url` and `build`) to `.ascribe/dev.json` in the project, for the editor's [Open Site Preview](../editor/preview.md#site-preview), and removes the file when it stops. The `url` is the project's base path on the dev server.

## Components
@id: components

A layout imports these from `@ascribed/astro`, each given the page's collection entry or a part of it. [The layout](index.md#the-layout) that `init` writes uses every one but `MissingPermalink`, which goes on the site's 404 page.

| Component | What it renders |
|---|---|
| `Elements.astro` {available="ascribe 0.2.0"} | `<Elements />`: the element library's stylesheet, and the small script `<ascribe-tabs>` needs. |
| `Head.astro` {available="ascribe 0.3.0"} | `<Head entry={entry} />`: everything the page's head says about the page, from its record: its `<title>`, description, canonical link, robots rule, link to its Markdown version, link preview, and breadcrumbs ([the head](reference.md#the-head)). |
| `Availability.astro` {available="ascribe 0.2.0"} | `<Availability available={entry.data.available} />`: the page's `available` frontmatter as a badge, the same markup the site output gives a section's `@available`, which the element library styles. Nothing for a page with no availability. |
| `Versions.astro` {available="ascribe 0.3.2"} | `<Versions entry={entry} />`: the lines of a [versioned build](reference.md#versions), those that have the page, or the page it replaces or the page that replaces it, as links, the others unlinked, and the archived lines that have an address. Nothing for a build that isn't versioned. |
| `VersionBanner.astro` {available="ascribe 0.3.2"} | `<VersionBanner entry={entry} />`: on an older line's page, which version it documents, with a link to the current line's. Nothing on the current line. |
| `MissingPermalink.astro` {available="ascribe 0.3.0"} | `<MissingPermalink />`, on the site's 404 page: [a permalink that isn't here](reference.md#a-permalink-that-isnt-here). |

An entry's data, besides its frontmatter:

- @available: ascribe 0.2.0
  `formatted`, when the content model sets `inline = "code"` on a field: the field as HTML, with each code span a `<code>`, for the heading and navigation (`set:html`). The example's model sets it on `title`, so a title can show code, as "`` `loom.yaml` options ``": `entry.data.title` is plain text, for `<title>` and search, and `entry.data.formatted.title` the HTML. See [code in a field](../content-model/index.md#53-code-in-a-field).
- @available: ascribe 0.3.0
  `source`, the page's path from the content root, with `/`, as `guides/install.md`. An "Edit this page" link is your repository's edit address, the content root's folder in it, and `source`, wherever Ascribe's build folder is.
- `available`, the page's `available` frontmatter: a list of targets, each with the text to show.

## Several projects
@id: several-projects
@available: ascribe 0.3.0

With `projects`, one integration builds several Ascribe projects into one site, each under its own base path. [Put the docs in a site with more than the docs](more-than-docs.md) sets one up.

- **An entry's name** is what everything else says: the collection, `review`, the folder of the entry's schema, `.astro/integrations/_ascribed_astro/<name>/schema.ts`, and an error message. It has letters, digits, `-`, and `_`, starts with a letter, and is the entry's own. Each collection names its entry, `ascribeCollection({ schema, name })`.
- **Astro's `site` and `trailingSlash`,** when `astro.config.mjs` leaves them out, come from the first entry whose project sets them; every project must agree with them anyway, and the build fails, naming the entry, when one doesn't. Astro's `base` is the site's, the root every base path is under: the integration never sets it from a project.
- **Two entries can name one project** with two builds, such as a collection for each release line. They share the project's `ascribe.toml`, so they share its base path: route one of them at it, and use the other's collection as you like. Where both publish a file or a permalink, the first entry's wins.
- **In `astro dev`,** a change rebuilds the project it's in, and each project's folder gets its own `.ascribe/dev.json`, with its own base path. Review covers one project: the entry `review` names, or the first entry when it's `true`. Each removed permalink's page links to the root of the project that retired it, and `MissingPermalink` offers the root of the first project with permalinks.

What the projects share, the integration checks, and it fails the build, naming the entries, when:

- **Two projects have one base path.** Each needs its own. One project at `/` with another beneath it at `/docs/` is fine.
- **A page of one project is under another's base path,** such as a page at `/docs/extra/` in the project at `/`.
- **Their permalinks are published at two paths.** One route serves every project's permalinks, so set `permalink-path` to one path in each project that declares any. A project with none can leave it.
- **Two projects declare one permalink.** Neither project's build can see the other's, so the integration reports it; rename or retire it in one of them.
- **Two entries have one name,** or a name has other characters.

## Versions
@id: versions
@available: ascribe 0.3.2

With a [versioned build](../versions/index.md), one site serves every live line: the current line's pages at the base path, and each older line's under its number, `/docs/3.4/`, from one collection, with nothing to change in `astro.config.mjs` or `src/content.config.ts`. [Publish with Astro](../versions/index.md#publish-with-astro) says what the collection, the layout, and `astro dev` do with them. Each line's `llms.txt` and Markdown are published at its base path, an older line's pages name the current line's as canonical, and the sitemap lists each page once. A permalink always forwards to the current line.

## Permalinks
@id: permalinks
@available: ascribe 0.3.0

Each build lists its [permalinks](../permalinks/index.md) in the site output, `_ascribe/permalinks.json` ([output-layout contract §1.2](../contracts/output-layout.md#12-the-permalinks-list)), and the integration publishes a page at each address, in `astro build` and in `astro dev`, where a permalink you add or remove is served on the next rebuild, with no restart. The page:

- forwards at once with a `<meta http-equiv="refresh">`, without a script, keeping a heading's anchor;
- with a script, first takes the reader's choices from the address's parameters;
- links to the same place, for a browser that doesn't follow;
- is marked `noindex`, with a canonical link to where it leads, so search engines list the page and not the address.

So it works on any static host. The addresses end the way the site's routes do (`trailing-slash`), and aren't pages of the collection, `llms.txt`, or the Markdown for agents. A retired permalink's page says what was there was removed from the docs, gives the reason, and links to the docs' home page, unless it was retired `--to` another page, where it forwards.

The parameters are named for the content model's [dimensions](../content-model/content.md#dimensions), as a product [links to a permalink](../permalinks/index.md#linking-from-a-product):

- **A parameter that names a dimension, with one of its values,** sets the reader's choice for that dimension, as if they'd picked the tab, and the choice holds on the pages they read next.
- **Anything else is ignored:** a name that isn't a dimension, or a value the dimension doesn't have. A link with only unknown parameters works as one with none.
- **The choice needs a script and the browser's storage.** With scripts off, the address forwards to the section with the tabs as they were; with storage blocked, the choice doesn't reach the next page.
- **A version isn't a parameter yet.** `self-managed=3.4` is ignored like any unknown one.

[`permalink-path`](../content-model/output.md#consumer) moves the addresses, as a path from the site's root, such as `"/go/"` when the docs move under `/docs/` but the addresses your product prints shouldn't. It must be under Astro's `base`, since that's all an Astro site serves; the integration fails the build when it isn't.

Another generator reads the same list and publishes a page like these at `path` and each name, forwarding to `route`, and `#` and `fragment` for a heading. For the parameters, its script calls `@ascribed/elements`' `choose` for each one that names a dimension in the list's `dimensions` with one of its values, then forwards. A host's redirect rules can do the same as the page without its script, but a redirect drops nothing only if the host keeps the anchor.

### A permalink that isn't here
@id: a-permalink-that-isnt-here

A product released before a section was renamed, or after one was added, may link to a name these docs don't have. The host answers with the site's 404 page, as for any missing address. Put `MissingPermalink` on it:

```astro
---
// src/pages/404.astro
import MissingPermalink from "@ascribed/astro/MissingPermalink.astro";
---

<h1>Page not found</h1>
<MissingPermalink />
```

On an address under the permalinks' path, it says the link came from a product that expected a section of the docs there, names it, and offers the docs' home page and search. Anywhere else, and without a script, it shows nothing. `home` sets where it sends the reader, the docs' base path by default; children replace its last paragraph, such as to point at your search:

```astro
<MissingPermalink>
  <p>Search the docs for what you need, or go back to the <a href="/docs">home page</a>.</p>
</MissingPermalink>
```

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

- every page of every project that isn't `unlisted`, when the site has a route that shows it: a project whose collection the site only quotes from adds nothing;
- every other page Astro built, such as a changelog page of the site's own, except a permalink's forwarding page, the not-found page, and the routes in the `sitemap` option's `exclude`.

`astro dev` knows only the routes of the site's own that have no parameters, so a page of the site's own that a route's `getStaticPaths` lists is in the built sitemap only.

@available: ascribe 0.3.2
Of a [versioned build](reference.md#versions), a page is listed only when its canonical address is its own, so an older line adds only the pages the current line has nothing in the place of, and each current page is listed once.

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

**On another host,** read the list after the build and give each path its headers in the host's own configuration: [Serve the Markdown versions](serve-markdown.md) says how for nginx and Cloudflare Pages.

## The sidebar
@id: sidebar
@available: ascribe 0.4.0

`sidebar(entries, options)`, from `@ascribed/astro/content`, gives a layout its sidebar: the project's [navigation](../navigation/index.md), from the build's `_ascribe/nav.json` ([navigation contract](../contracts/navigation.md)), each page joined to its collection entry. `entries` is the collection, `await getCollection("docs")`.

| Option | Meaning |
|---|---|
| `entry` | The page the sidebar is on. With a [versioned build](reference.md#versions), the sidebar shows that page's line; without it, the current line's. |
| `name` | Which project's collection, with [several projects](reference.md#several-projects), as `ascribeCollection` takes it. By default, the project the entries were loaded from. |
| `nav` | A site's own list of groups, `{ label, pages }`, each page a source file from the content root, which overrides the project's navigation. The build fails when the list leaves out a page of the collection, names a file that isn't one, or names a page twice. |

It returns the groups in order. The content root's pages are the first group, labeled **Start** unless `ascribe.nav` says otherwise, and each top section is a group of its own; a site's list gives its groups as they are.

| Field | Meaning |
|---|---|
| `label` | The group's label, as plain text. |
| `formatted.label` | The label as HTML, when it's a title with code (`inline = "code"`). |
| `folder` | The section's folder, from the content root. Absent in a site's own list. |
| `page` | The section's own page. |
| `pages` | Its pages in order, each `{ entry, href, label, formatted? }`: the collection entry, its address, the one Ascribe's links use, and its label. A section's `index.md` is first among them; an own page beside the folder, such as `checks/diagnostics.md` for `checks/diagnostics/`, is among the parent's pages instead, and the nested group sits right after it. A top section's own page beside its folder, such as `guides.md` for `guides/`, is first in that section's group, not in the root's. |
| `sections` | Its nested sections, as groups, in order. Empty in a site's own list. A layout that draws one level leaves them out. |

The build fails when the navigation names a page the collection has no entry for, which happens only when the site output is from another build than the collection; build again.

A site that passed its list first, `sidebar(nav, entries, options)`, passes it as an option: `sidebar(entries, { ...options, nav })`.

A page's label is its [`nav-label`](../navigation/index.md#a-page-s-label) when it has one, and its title otherwise.

### Previous and next
@id: neighbors
@available: ascribe 0.4.0

`neighbors(entry, entries, options)`, from `@ascribed/astro/content`, gives a page's previous and next: the pages before and after it in the navigation's [reading order](../navigation/index.md#sections-and-pages), depth first, so a nested section's pages follow its own page. `entry` is the page's collection entry and `entries` the collection; the options are `name` and `nav`, as `sidebar()` takes them. With a [versioned build](reference.md#versions), both are in the line `entry` is on.

```astro
---
import { getCollection } from "astro:content";
import { neighbors } from "@ascribed/astro/content";

const { previous, next } = neighbors(entry, await getCollection("docs"));
---
{previous && <a rel="prev" href={previous.href}>{previous.label}</a>}
{next && <a rel="next" href={next.href}>{next.label}</a>}
```

It returns `{ previous?, next? }`, each `{ entry, href, label, formatted? }`, as a page of a sidebar group is. The first page has no `previous`, and the last no `next`; a page the navigation leaves out, such as one that's `unlisted`, has neither. With a site's own list, `nav`, they're the pages before and after it in the list's groups, in order.

## Review in the site preview
@id {permalink=true}: review-in-the-site-preview
@available: ascribe 0.2.0

In `astro dev`, Astro's dev toolbar has an **Ascribe review** app. It shows a pull request's changes and review comments on the real page, in your site's layout: the same marks and threads as the editor's [page preview](../editor/review.md), from the same [`@ascribed/review`]({repo}/blob/main/packages/review/README.md) overlay. Nothing runs until you open the app: opening it starts review, and after **Stop Review**, the panel offers **Start Review**. [Review](../review/index.md) walks through reviewing a pull request with it; this section is the reference for the site's side.

The app's panel sits above the toolbar, in one row: the pull request and the base (**#12 against main**), your place in the changes ("3 of 10 on this page", click it for the breakdown), **Changes / As it will be / As it was**, next and previous change, **Comments**, **Refresh**, and **More review actions** (**⋯**), whose **Prompt agent** items copy a prompt for your agent about the page's changes or a fragment's pages ([In review](../agents/index.md#in-review)). Each thread's **Prompt agent** copies one about its comment. Past the last change it offers the next changed page, by its title. When `ascribe check` finds errors in the working tree, the panel says how many, since a page with an error may not show as it will once it's fixed ([`ascribe diff`](../review/diff.md#cli-diff) explains why). Close it to get the page back; it stays closed or open as you move between pages, and review stays on until **Stop Review** or until `astro dev` stops. With the panel closed, a dot on the toolbar button says you have comments you haven't submitted.

Starting review compares the checkout with the base of its branch's pull request, or, with no pull request, with the default branch (the first of `origin/HEAD`, `origin/main`, `origin/master`, `main`, and `master` that exists), from where the branch left it, as [`ascribe diff`](../review/diff.md#cli-diff) does. It compares again after each rebuild, so the marks follow your edits on save. A [versioned](../versions/index.md) project is compared in every line, and a page shows the changes of the line it's in: `/3.4/install/` marks what changed in 3.4. Comments, replies, and resolving work as in the page preview, and [map to the pull request](../review/index.md#how-comments-map-to-the-pull-request) the same way: new comments are unsent until **Submit review…** in the panel sends them. Each mark's label and each thread's **Open source** opens the file at the line in your editor, through Vite's open-in-editor: set `LAUNCH_EDITOR` (for example, `LAUNCH_EDITOR=code`) to choose which.

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
