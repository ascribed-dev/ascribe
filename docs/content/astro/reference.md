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
| `projects` | [Several projects](reference.md#several-projects): a list of entries, each `{ name, build, project }`, in place of `build` and `project`. |
| `binary` | The `ascribe` binary to run, relative to the Astro root. By default, the `ASCRIBE_BIN` environment variable, then the binary `@ascribed/cli` installed. |
| `anchors` | Mark each block of the page with the source file and lines it came from (`data-ascribe-source`; see the [site-render contract](../contracts/site-render.md#7-source-anchors)), for review: `"dev"` in `astro dev` only, `true` in `astro build` too. Use `"dev"` unless the build is for reviewers: `true` puts source file paths, fragments' included, in the published pages. By default, `false`, though `review` turns them on in `astro dev`. |
| `review` | [Review in the site preview](../review/site-preview.md): the **Ascribe review** app in `astro dev`'s toolbar. `false` leaves it out. By default, `true`. `astro build` never has it. With `projects`, it reviews one entry: the one an entry's name names (`review: "docs"`), or the first for `true`. |
| `codeTitles` | Show a code block's title above it, and what a [tested example](../examples/tested.md) was tested with ([code block titles](reference.md#code-block-titles)). `false` leaves both out, for a site that shows them itself. By default, `true`. |
| `sitemap` | `{ exclude: [...] }`: routes of the site's own pages, from the host's root, to leave out of the [sitemap](../publishing/site.md#the-sitemap), such as `["/drafts"]`. |

## What the integration does

- **Fails the Astro build** when `ascribe build` reports an error (the compiler's report is the error), and when `ascribe.toml`'s `[consumer]` `site`, `base-path`, or `trailing-slash` disagrees with Astro's `site`, `base`, or `trailingSlash`. In ascribe 0.2.0, Astro's `trailingSlash: "ignore"` agrees with either value; since ascribe 0.3.0, the integration replaces it, as below. `base-path` agrees with Astro's `base` when it's that path or, since ascribe 0.3.0, a path under it ([A home page in front of the docs](more-than-docs.md#a-home-page-in-front-of-the-docs)).
- **Gives Astro its `site`, `base`, and `trailingSlash`** from `[consumer]` when `astro.config.mjs` leaves them out, so the site's address is written once: `site`, `base-path`, and `trailing-slash`. A setting `astro.config.mjs` does give must still agree. With [`projects`](reference.md#several-projects), `base` stays the site's. Astro can't tell an integration a setting left out from one set to its default, so `base: "/"` and `trailingSlash: "ignore"` are replaced too.
- **Adds its Markdown plugin** to Astro's Markdown processor, to apply heading ids, image attributes, glossary terms' `data-ascribe-term`, and source anchors: to the default Sätteri processor's `hastPlugins`, or to a `unified()` processor's `rehypePlugins`. Both plugins are exported, as `@ascribed/astro/satteri` and `@ascribed/astro/rehype`, for a processor you configure yourself. Their types come from `satteri` and `@types/hast`, optional peer dependencies: a Sätteri processor already has the first, and a `unified()` one the second.
- **Serves the files pages link to** (other than pages and images) at `<base-path>_ascribe/files/`, in `astro dev` and in the built site.
- **Publishes the outputs for agents**, on by default with a `[consumer] site`: `llms.txt` and each page's Markdown, in `astro dev` and in the built site ([Publish for AI agents](agents.md)).
- **Gives each permalink an address** that forwards to its page or heading, in `astro dev` and in the built site ([permalinks](reference.md#permalinks)), and serves each line's [help entry](reference.md#help-entries) for it as JSON.
- **Writes the site's sitemap and `robots.txt`** from every project's page records, and gives a layout each page's head ([the head, sitemap, and `robots.txt`](../publishing/site.md)), in `astro dev` and in the built site.
- **Sends each Markdown version's response headers** in `astro dev`, and with `[consumer] headers = "netlify"` writes them into the built site's `_headers`, or fails the build when the site's own `public/_headers` lacks them ([response headers](../publishing/site.md#response-headers)).
- **Serves every line of a versioned build** from one collection, builds every line in `astro build` and `astro dev`, and serves each line's linked files at its base path ([versions](reference.md#versions)). Each entry's data has `line`, the line it's from, and `current`; the generated schema doesn't type them. A permalink forwards to the current line, and each line's outputs for agents are published at its base path, with its headers.
- **Shows code block titles** and tested lines, with a Shiki transformer it adds to Astro's code highlighting ([code block titles](reference.md#code-block-titles)).
- **Copies the generated schema** into `.astro/integrations/_ascribed_astro/schema.ts` after each build, for a project outside the Astro root ([the collection](index.md#the-collection)).

### In `astro dev`

- Saving a page, a fragment, an image or other file under the content root, an asset elsewhere in the project that a page uses, or `ascribe.toml` rebuilds. Changes are batched, and builds run one at a time.
- After a successful rebuild, Astro reloads the collection and the page. A versioned build rebuilds every line.
- After a failed one, the diagnostics are in Astro's terminal, and pages answer with HTTP 503 until a save builds cleanly again, so you never see a stale page.
- A change to `[project] output-dir` needs a restart of `astro dev`, and says so.
- It writes where it's running (`url` and `build`) to `.ascribe/dev.json` in the project, for the editor's [Open Site Preview](../editor/preview.md#site-preview), and removes the file when it stops. The `url` is the project's base path on the dev server.

## Components
@id: components

A layout imports these from `@ascribed/astro`, each given the page's collection entry or a part of it. [The layout](index.md#the-layout) that `init` writes uses every one but `MissingPermalink`, which goes on the site's 404 page.

| Component | What it renders |
|---|---|
| `Elements.astro` | `<Elements />`: the element library's stylesheet, and the small script `<ascribe-tabs>` needs. |
| `Head.astro` | `<Head entry={entry} />`: everything the page's head says about the page, from its record: its `<title>`, description, canonical link, robots rule, link to its Markdown version, link preview, and breadcrumbs ([the head](../publishing/site.md#the-head)). |
| `Availability.astro` | `<Availability available={entry.data.available} />`: the page's `available` frontmatter as a badge, the same markup the site output gives a section's `@available`, which the element library styles. Nothing for a page with no availability. |
| `Versions.astro` | `<Versions entry={entry} />`: the lines of a [versioned build](reference.md#versions), those that have the page, or the page it replaces or the page that replaces it, as links, the others unlinked, and the archived lines that have an address. Nothing for a build that isn't versioned. |
| `VersionBanner.astro` | `<VersionBanner entry={entry} />`: on an older line's page, which version it documents, with a link to the current line's. Nothing on the current line. |
| `MissingPermalink.astro` | `<MissingPermalink />`, on the site's 404 page: [a permalink that isn't here](reference.md#a-permalink-that-isnt-here). |

An entry's data, besides its frontmatter:

- `formatted`, when the content model sets `inline = "code"` on a field: the field as HTML, with each code span a `<code>`, for the heading and navigation (`set:html`). The example's model sets it on `title`, so a title can show code, as "`` `loom.yaml` options ``": `entry.data.title` is plain text, for `<title>` and search, and `entry.data.formatted.title` the HTML. See [code in a field](../content-model/index.md#53-code-in-a-field).
- `source`, the page's path from the content root, with `/`, as `guides/install.md`. An "Edit this page" link is your repository's edit address, the content root's folder in it, and `source`, wherever Ascribe's build folder is.
- `available`, the page's `available` frontmatter: a list of targets, each with the text to show.

## Several projects
@id: several-projects

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

With a [versioned build](../versions/index.md), one site serves every live line: the current line's pages at the base path, and each older line's under its number, `/docs/3.4/`, from one collection, with nothing to change in `astro.config.mjs` or `src/content.config.ts`. [The Astro components](../versions/lines.md#the-astro-components) says what the collection, the layout, and `astro dev` do with them. Each line's `llms.txt` and Markdown are published at its base path, an older line's pages name the current line's as canonical, and the sitemap lists each page once. A permalink always forwards to the current line.

## Permalinks
@id: permalinks

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

### Help entries
@id: help-entries
@available: ascribe planned

Each build writes each permalink's section to `_ascribe/help.json` ([output-layout contract §1.7](../contracts/output-layout.md#17-the-help-file)), for a product to [show in itself](../permalinks/in-your-product.md). The integration serves each entry at an address of its own, in `astro build` and in `astro dev`, so a product can fetch its line's:

- **The current line's** at the permalink's address with `.json` in place of its end: `/docs/go/connect.json`, beside `/docs/go/connect/`.
- **An older line's** at `go/` under the line's base path: `/docs/3.4/go/connect.json`. A permalink's own address always forwards to the current line, so the line is in the path.

Each answers with the file narrowed to that one entry: `version`, `build`, and `line` as the line's file has them, and `help` with the entry under its name, so a product reads a fetched entry and its bundled file the same way. A line that has left the window has no address, and an alias has its own. `astro dev` answers with `Access-Control-Allow-Origin: *`, and with `[consumer] headers = "netlify"` the build's `_headers` gives each address the same, since the build's [headers list](../publishing/site.md#response-headers) has a rule for each.

Another generator serves the same: for each name in a line's `help.json`, the file with that entry alone, at `<path><name>.json` for the current line, where `path` is `_ascribe/permalinks.json`'s, and at `<base>go/<name>.json` for an older line, where `base` is the line's in `_ascribe/versions.json`.

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

From every project's page records, `_ascribe/pages.json`, the integration writes what a search engine reads: each page's head, through `Head.astro`; the site's `sitemap.xml` and `robots.txt`; and each Markdown version's response headers, which `astro dev` sends and `headers = "netlify"` writes into `_headers`. [The head, sitemap, `robots.txt`, and headers](../publishing/site.md) says what each holds, and [Publishing](../publishing/index.md) what a project tells search engines and agents.

## The sidebar
@id: sidebar

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

## Styling

The elements render into the page (no shadow DOM), so your site's styles apply to them, and they're themed with CSS custom properties such as `--ascribe-tip-color` and `--ascribe-tab-active-color`. The [element library reference](elements.md) lists them, and shows how to style your own note types and lifecycle states. Their names don't change, but their default colors are Ascribe's and may change in a release, as the changelog notes, so set the ones your site needs fixed.

### Code block titles

A code block's title, such as a [`@snippet`](../writing/directives.md#snippet)'s `title`, is in its info string (`` ```mjs title="astro.config.mjs" ``). So is a tested example's `tested`, what it was tested with. Astro's highlighting ignores both, so the integration adds a Shiki transformer that puts such a block in a figure with the title as its caption, and the tested line under it in a `<div class="ascribe-code-tested">`, whose "Tested with" lead-in is the [element library's stylesheet's](elements.md#generated-text):

```html
<figure class="code-title">
  <figcaption>astro.config.mjs</figcaption>
  <pre class="astro-code">…</pre>
</figure>
```

The figure has your browser's default margins, and the caption no style of its own; style `figure.code-title` and its `figcaption` to suit your site. Titles need Astro's default highlighting, Shiki; with `syntaxHighlight: "prism"` or `false`, they aren't shown.

## Other Markdown processors

The integration supports Astro's default processor (Sätteri) and `unified()`. Any other processor is an error at startup, since heading ids and image attributes would be lost.
