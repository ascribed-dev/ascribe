---
type: reference
title: "The head, sitemap, `robots.txt`, and headers"
description: "What @ascribed/astro writes from the page records for search engines and crawlers: each page's head, the sitemap, robots.txt, and the Markdown versions' response headers."
permalink: search-engines
---

Each build writes a record of every page, `_ascribe/pages.json` ([output-layout contract](../contracts/output-layout.md#13-the-page-records)), with its title, description, canonical address, and [`listing`](index.md#how-a-page-is-listed), whether the page set it or a rule of [`[listing]`](../content-model/checks.md#22-listing) did, and what the project says to crawlers in [`[crawlers]`](../content-model/checks.md#21-crawlers). `@ascribed/astro` writes what a search engine reads from those records, so the head, the sitemap, `robots.txt`, and `llms.txt` can't disagree about a page. Another generator reads the same records, and `_ascribe/headers.json`, to write its own. [Publishing](index.md) says what a project tells search engines and agents, and why.

## The head

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
- `<meta name="robots" content="noindex">` for a page that's `unlisted`, and `<meta name="robots" content="nosnippet">` for one that's `unquoted`;
- `<link rel="alternate" type="text/markdown">` to the page's Markdown version, when the build publishes one ([`[consumer] agents`](../content-model/output.md#consumer)) and the page isn't `unlisted`;
- the page's breadcrumbs, as [`BreadcrumbList`](https://developers.google.com/search/docs/appearance/structured-data/breadcrumb) structured data, when there are two steps or more.

From the site, by property, since the record doesn't hold them:

- `title`, a function from the page's title and its record to the whole title. The record has the page's `route`, so a site can title its home page with its name alone. Without `title`, the title is the page's own, and a page whose title is missing gets no `<title>`.
- `name`, the site's name, for `<meta property="og:site_name">`, and, on the page at the host's root (`/`), the site's name as [`WebSite`](https://developers.google.com/search/docs/appearance/site-names) structured data, which a search engine shows as the name in a result. Google reads it only on the home page.
- `card`, the page's social card: `src`, its image, from the host's root or absolute; `alt`, what it shows; and `width` and `height`, in pixels. It's the `og:image` tags a link preview shows. Without it, there are no image tags.

The layout keeps the character set, the viewport, icons, theme colors, styles, the elements, and tags for one network that Open Graph doesn't cover, such as `<meta name="twitter:card">`. A tag `Head.astro` writes that the layout writes too is in the head twice; `astro dev` warns once about each such tag, naming it and the first page it's on. It's a warning and not an error, since a site may have a reason.

**Breadcrumbs.** A page's place in the site, from its route's folders: the page at each folder of its route that has one, named by its title, down to the page itself. `/docs/guides/install/` has the page at `/docs/`, the page at `/docs/guides/` if there is one, and itself. A folder with no page isn't a step, since there's nothing to link to. The steps are the record's `breadcrumbs`, so a layout can draw visible breadcrumbs from the same list with `pageRecord(entry)`. They change how a search result is displayed, and nothing else. They aren't the site's navigation: they're drawn from the route alone, not from [`ascribe.nav`](../navigation/index.md), whose order and labels a layout reads from `_ascribe/nav.json` for its sidebar.

**A page with no record**, such as an Astro page of the site's own, gets nothing from `Head.astro`, and writes its own `<title>`, description, canonical link, and link preview. A page that isn't an Ascribe page and shouldn't be listed writes its own `<meta name="robots" content="noindex">`, and goes in the [`sitemap` option](../astro/reference.md#options)'s `exclude`, since the integration reads no HTML and can't see the rule. A site whose home page isn't an Ascribe page writes the site's name there itself:

```html
<script type="application/ld+json">
  {"@context": "https://schema.org", "@type": "WebSite",
   "name": "Loom", "url": "https://loom.example.com/"}
</script>
```

`pageRecord(entry)`, from `@ascribed/astro/content`, gives a layout the record itself, for anything else it writes from it.

## The sitemap

`astro build` writes one sitemap for the site, `sitemap.xml` at Astro's `base`, and `astro dev` serves it there. It lists by absolute address, with no dates:

- every page of every project that isn't `unlisted`, when the site has a route that shows it: a project whose collection the site only quotes from adds nothing;
- every other page Astro built, such as a changelog page of the site's own, except a permalink's forwarding page, the not-found page, and the routes in the `sitemap` option's `exclude`.

`astro dev` knows only the routes of the site's own that have no parameters, so a page of the site's own that a route's `getStaticPaths` lists is in the built sitemap only.

Of a [versioned build](../astro/reference.md#versions), a page is listed only when its canonical address is its own, so an older line adds only the pages the current line has nothing in the place of, and each current page is listed once.

A sitemap needs absolute addresses, so a site with no `site` (`[consumer] site`, or Astro's) gets none, and the build warns. A site with its own `sitemap.xml` in `public/` keeps it, the integration writes none, and `robots.txt` points to the site's.

## `robots.txt`

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

## Response headers

A page's Markdown version has no head, so the rule its page's head gives is a [response header](index.md#the-markdown-versions-headers), which the host sends. Each build lists what each Markdown version needs in `_ascribe/headers.json` ([output-layout contract](../contracts/output-layout.md#14-the-headers)):

- `X-Robots-Tag: noindex` for a page that's `unlisted`, and `nosnippet` for one that's `unquoted`, the same rule as its head;
- `Link: <https://docs.example.com/docs/guides/install/>; rel="canonical"` on every one, naming the page, so a search engine lists the page and not its Markdown copy, while an agent can still fetch either.

The list also gives each [help entry's address](../permalinks/in-your-product.md#fetching-the-entry), such as `/docs/go/connect.json`, `Access-Control-Allow-Origin: *`, so a product on another origin can fetch it.

`astro dev` sends them with each `.md` file, so you can try them with no host:

```sh
$ curl -sI http://localhost:4321/docs/internal/runbook.md | grep -i x-robots-tag
x-robots-tag: noindex
```

**On Netlify,** set `host = "netlify"` in `[consumer]`, and `astro build` writes the rules into `_headers` at the root of the built site, which Netlify reads ([the host's files](#the-hosts-files)):

```text
/docs/internal/runbook.md
  Link: <https://docs.example.com/docs/internal/runbook/>; rel="canonical"
  X-Robots-Tag: noindex
```

Each rule is for one exact path, never a pattern, so a rule for a page can't reach its neighbor.

**On another host,** read the list after the build and give each path its headers in the host's own configuration: [Serve the Markdown versions](../astro/serve-markdown.md) says how for nginx and Cloudflare Pages.

## The host's files

`[consumer] host` names the host the site is deployed to. With it, each build writes that host's own files in its site output, in `_ascribe/host/` ([output-layout contract](../contracts/output-layout.md#19-the-hosts-files)), and `@ascribed/astro` places them at the root of the built site, where the host reads them. `"netlify"` is the one host Ascribe writes for, and Cloudflare Pages reads the same files:

```toml
[consumer]
site = "https://docs.example.com"
base-path = "/docs/"
host = "netlify"
```

- **`_headers`:** the [response headers](#response-headers) each Markdown version and help entry needs.
- **`_redirects`:** a `301` from each address [`ascribe.addresses`](redirects.md) records a move from to where it leads now, so a reader with an old link lands on the page.

With several projects, each file holds the lines of each project that sets `host`, the first entry's first when two give one path.

- **A site with its own `_headers`** in `public/` keeps it, and the integration never rewrites it. The build checks that it has Ascribe's lines, and fails when it doesn't, with the lines to add as its message. A line counts when it's under the same path, with the same value; the case of a header's name doesn't matter.
- **A site with its own `_redirects`** in `public/` keeps it too: the built `_redirects` has Ascribe's lines first and the site's after them, since Netlify uses the first rule that matches, so a page's recorded move wins over a pattern of the site's.
- **A redirect from an address a page of the site is at** fails the build, naming the line, since it would hide the page or never be used.
- **A site whose Astro `base` isn't `/`** has no root to write the files to, so the build prints the lines instead, for the host's own configuration.

Without `host`, nothing is written for a host, and the redirects and headers are in `_ascribe/redirects.json` and `_ascribe/headers.json`, for a deploy step to read.
