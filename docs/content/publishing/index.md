---
type: concept
title: Publishing
description: "What a project tells search engines and AI crawlers about its pages, and what it delivers to agents: listing, robots.txt, the response headers, llms.txt, and the Markdown versions."
---

A site has readers that aren't people: search engines, the crawlers that gather pages for AI models and AI search, and agents that fetch a page when someone asks about it. A project says what each is told in its content model, and each build writes it down, so every output of a site says the same thing about a page.

| What | Set in | Written to |
|---|---|---|
| Whether a page is listed | a page's [`listing`](../content-model/index.md#listing-a-page), and [`[listing]`](../content-model/checks.md#22-listing) for a folder, a type, or the site | each page's head, the sitemap, `llms.txt`, and the Markdown versions' headers |
| What each kind of crawler may fetch | [`[crawlers]`](../content-model/checks.md#21-crawlers) | `robots.txt` |
| What agents read | [`[consumer] agents`](../content-model/output.md#consumer) | `llms.txt`, and a Markdown version of each page |
| What the host sends with a Markdown version | [`[consumer] headers`](../content-model/output.md#consumer) | `_ascribe/headers.json`, and a host's own file, such as Netlify's `_headers` |

Each build writes every page's record, with its title, description, canonical address, and listing, and the project's crawler rules, to `_ascribe/pages.json` in the site output ([output-layout contract](../contracts/output-layout.md#13-the-page-records)). A site writes its head, sitemap, and `robots.txt` from the records: `@ascribed/astro` does, as [the head, sitemap, `robots.txt`, and headers](site.md) says, and any other generator can read the same file. `llms.txt` and the Markdown versions are written by the build itself.

A canonical address needs [`[consumer] site`](../content-model/output.md#consumer): without it, a site gets no canonical links and no sitemap, which needs absolute addresses.

## How a page is listed

A page's listing is one of three values, from open to closed:

| Value | What it means |
|---|---|
| `listed` | The default. Search engines may list the page, and agents are offered it in `llms.txt`. |
| `unquoted` | Listed, and asks not to be quoted (`nosnippet`): a search result, or an AI answer inside a search product, shows no text from it. It's in `llms.txt` and the sitemap. It asks nothing of a crawler gathering training data. |
| `unlisted` | Asks not to be listed (`noindex`), and isn't offered: it's out of `llms.txt`, the sitemap, and the [navigation](../navigation/index.md#what-a-build-shows), so no sidebar drawn from it shows the page. It's still built, at its address, with its Markdown version. |

A page sets its own in its frontmatter. A rule in `[listing]` sets it for the whole site, a folder, or a content type, so a folder can be kept out of search without a change to any of its pages. A rule only restricts: a page's listing is the strictest of its own and the rules it's under, and a page that sets a looser one is an error.

## What crawlers are told

`robots.txt` tells crawlers what they may fetch. It's a request: a crawler may ignore it, and some bots document that they may. A project writes its own lines in `[crawlers] lines`, and says what each kind of AI crawler may do by its purpose, `training`, `search`, and `on-request`, without naming the bots: Ascribe knows which bots each purpose names ([crawlers by purpose](../content-model/checks.md#crawlers-by-purpose)). A project that says nothing gets no rules about AI crawlers.

What each choice costs:

- **Disallowing `training`** asks those operators not to train models on the site. It doesn't take the site out of any search or answer, and it stops nothing already gathered. Google's and Apple's are tokens, not crawlers: Googlebot and Applebot still fetch every page for search, and the token says only what may be done with it.
- **Disallowing `search`** takes the site out of those AI search products' answers, which is the point of being crawled by them: a site opted out of OpenAI's search crawler isn't shown in ChatGPT's search answers, for one. It doesn't touch Google's or Bing's search, whose AI answers are drawn from their ordinary index, which only a page's listing keeps a page out of. `ascribe check` says what it costs (`crawlers-disallowed`).
- **Disallowing `on-request`** asks assistants not to fetch a page when a person asks about it, so they answer from what they already have, or from another site. Several operators say their fetcher may not follow it, since a person asked (see [crawlers by purpose](../content-model/checks.md#crawlers-by-purpose)), and [Google's own fetchers](https://developers.google.com/crawling/docs/crawlers-fetchers/google-user-triggered-fetchers) for its agents generally ignore `robots.txt` and have no token, so they aren't listed. `ascribe check` says what it costs (`crawlers-disallowed`).

**Keep a page out of search with its listing, not a `Disallow` line.** A crawler that follows `robots.txt` and is told not to fetch a page never reads that page's head, so it can't see that the page asks not to be listed, and a search engine can list the address alone. `ascribe check` says so when a page that isn't listed is at an address a search crawler may not fetch (`listing-disallowed`).

A host's or a CDN's own settings for bots can block a crawler before it reads `robots.txt`, whatever the file says, and can allow it in spite of it. Those are the host's, not Ascribe's: check them when a crawler you allowed can't reach the site.

## What agents get

With a `[consumer] site`, a build writes `llms.txt`, an index of the pages it offers agents, and a Markdown version of each page, beside the site at the page's address with `.md`; each page of the site points to both. `agents = false` turns them off. [The outputs for agents](../content-model/output.md#consumer) says what each holds, and [Publish for AI agents](../astro/agents.md) publishes them with Astro.

## The Markdown versions' headers

A Markdown version has no head, so what its page's head says to a search engine has to be a response header, and only the host sends those. Each build lists, for each Markdown version's path, `X-Robots-Tag: noindex` for a page that's `unlisted`, `nosnippet` for one that's `unquoted`, and `Link: <page>; rel="canonical"` on every one, so a search engine lists the page and not its Markdown copy, while an agent can still find either.

The list is `_ascribe/headers.json` ([output-layout contract](../contracts/output-layout.md#14-the-headers)). With `[consumer] headers = "netlify"`, `@ascribed/astro` writes it into the built site's `_headers` ([response headers](site.md#response-headers)); on another host, a deploy step gives the host the list ([Serve the Markdown versions](../astro/serve-markdown.md)). Without either, the Markdown version of a page that isn't listed can still be indexed, since only a response header keeps a file that isn't HTML out of a search index.
