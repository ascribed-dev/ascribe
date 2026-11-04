---
type: concept
title: What to measure
description: Where a team sees how search engines and AI agents treat its docs, since Ascribe sees no traffic, and which of the advice about them has evidence behind it.
---

Ascribe builds your docs and checks them before they're deployed. It runs no server, sees no traffic, and calls no model, so it can't tell you how many people found a page through a search engine, or whether an assistant answered a question about your product correctly. This page says where a team looks for those, and how much each piece of the advice you'll read about search and AI is worth.

## What Ascribe tells you before a deploy

Everything Ascribe knows about a page is in its record: its address, title, description, and listing, and where the listing comes from. The head of each page, the sitemap, `robots.txt`, and `llms.txt` are all written from the records, so they can't disagree with each other.

- **`ascribe check`** reports what would go wrong for a reader that isn't a person. Each is advice you can raise or turn off in [`[checks]`](../content-model/checks.md#22-checks):
  - Two pages with the same title or description.
  - A title its page's own heading contradicts.
  - A page kept out of search whose address `robots.txt` won't let a crawler fetch.
  - A page too long for an agent to read in one fetch.
- **`ascribe report pages`** lists every page's record, and `--not-listed` the ones kept out of search, with why. It's what a team would otherwise crawl its own site to see. See [the project report](../checks/report.md#pages).
- **`ascribe report delivery --site <URL>`** runs the delivery spec's checker on the published site, for what only a running server can answer: [Check what the site delivers to agents](#check-what-the-site-delivers-to-agents).

None of these is a score. Each problem is listed at its file, with what to do about it; a number would count different things from one tool to the next, and invite tuning to the number.

## Check what the site delivers to agents

`ascribe report delivery --site <URL>` runs the [delivery spec](https://agentdocsspec.com/spec/web/)'s checker, `afdocs`, on a built site, the published one or a preview of it, and lists each check as passing, warning, failing, or skipped. Install it with `npm install -g afdocs@0.22.2`.

A check that fails or warns is reported by who changes what it checks, and none of it asks you to edit a page:

- **The hosting** (`delivery-hosting`): status codes, caching, the type a `.md` file is served as, content negotiation, bot protection. The problem names the setting; [Serve the Markdown versions](../astro/serve-markdown.md) says how on the hosts it covers. `content-start-position` is here too, since it's the site's page template: a template that writes the navigation before the content warns, even with the content in `<main>`. Put the navigation after the content in the HTML, and place it with CSS. A check the checker added after the version Ascribe knows is counted here until Ascribe lists it.
- **Ascribe** (`delivery-output`): `llms.txt`, the Markdown pages, and the pointer on each page, which Ascribe writes with the outputs for agents, on by default with a `[consumer] site`. Check first that the site publishes what the build wrote; when it does, it's a bug in Ascribe, and the problem says where to report it. When a check names addresses it didn't reach, as `llms-txt-links-resolve` does, the problem lists each with the status it answered or why it couldn't be fetched. When every one only failed to fetch, the checker got no answer, which may be the network's doing, so the problem says to run the report again instead.
- **The pages**, such as a page too long for an agent to read in one fetch: `ascribe check` reports these on the page itself, so the report names the check and doesn't repeat it. A page over the checker's `page-size-html` limit that `ascribe check` doesn't report as too long is over it because of what the site's page template adds around the content.

To check the site every day, run this section in the [scheduled job](../checks/report.md#on-a-schedule).

## Where search engines report

The search engines report on your site themselves, once you've shown them you own it: Google in [Search Console](https://search.google.com/search-console), Bing in [Webmaster Tools](https://www.bing.com/webmasters). Look at:

- **Which pages are indexed, and why the others aren't.** A page excluded by `noindex` should be one `ascribe report pages --not-listed` lists; one that's listed and still excluded is worth reading about in the report's reason.
- **The queries that show each page, and how often it's chosen.** A page shown often and rarely chosen usually has a title or description that doesn't say what's on it.
- **Their AI features,** where an engine counts them in its reports. What each engine counts, and how it labels it, changes; read its own documentation for the current answer rather than a summary of it.

## Where agents show up

Your host's request logs are the only place agents reading your docs show up. Look for:

- **Requests for the Markdown versions,** the `.md` address of each page, which a site publishes with the outputs for agents, on by default with a `[consumer] site`. A browser never asks for them, so they're the cleanest sign of an agent reading, and the address says which page it read.
- **Requests for `llms.txt`,** which an agent reads to choose a page.
- **The user agents** of the crawlers you've made a choice about in [`[crawlers]`](../content-model/output.md#18-crawlers): the tokens of each purpose are listed there, with where each operator documents its own. An assistant fetching a page because a person asked often sends no token of its own, so these counts are low, never high.

## Whether the answers are right

No report tells you whether an assistant answers questions about your product correctly. The test is your own: keep a set of the questions your readers ask, ask them again the same way after a change to the docs, and compare the answers with what the docs say. Write the questions down, in your repository, so the set is the same each time, and ask them before the change too, since the models change on their own. Ascribe calls no model, so it can't run this for you.

## What has evidence, and what doesn't

Most of what's written about being found by AI has no evidence behind it, and some of it is contradicted. This is what the research behind Ascribe's choices found, with the source of each finding; it's the part of this page most likely to be read as a promise, so it says how strong each finding is.

**Documented by the search engines themselves:** an accurate title and description for each page, different from every other page's ([title links](https://developers.google.com/search/docs/appearance/title-link), [snippets](https://developers.google.com/search/docs/appearance/snippet)); a [canonical link](https://developers.google.com/search/docs/crawling-indexing/consolidate-duplicate-urls); a [sitemap](https://developers.google.com/search/docs/crawling-indexing/sitemaps/overview). Ascribe writes each of these from the page records, and checks the first two.

**Measured, once each:**

- **A pointer to `llms.txt` on each Markdown page** cut an agent's fetches of addresses that don't exist from 2.23 a task, reading the HTML pages, to 0.11, in [Mintlify's benchmark](https://mintlify.com/blog/llms-txt-agent-benchmark) of 2,400 runs on sites it hosts. Markdown pages without the pointer got 1.42, and the answers were as accurate in every case. Every Markdown page Ascribe publishes has one.
- **A compact index of the docs in a product's `AGENTS.md`,** about 8 KB, took [Vercel's evaluation](https://vercel.com/blog/agents-md-outperforms-skills-in-our-agent-evals) of agents using new Next.js APIs from 53% to 100%. The same docs as a skill reached 79% at best. [`ascribe agents index`](product-index.md) writes one.
- **Size.** An agent's fetch tool cuts a page off at its limit and doesn't say so: about 100,000 characters for Claude Code, and 5,000 by default for the MCP fetch server, in [the delivery spec's comparison of agents](https://agentdocsspec.com/platforms/). `ascribe check` reports a page over the spec's limit, 50,000 characters.

**Not supported, or contradicted:**

- **`llms.txt` gets a site found by AI search.** Google says its search doesn't use the file, in [its guide to generative AI features](https://developers.google.com/search/docs/fundamentals/ai-optimization-guide), and in [Ahrefs' study](https://ahrefs.com/blog/llmstxt-study/) of about 137,000 domains, 97% of the files got no requests in May 2026. It's worth publishing for the agents that are already reading your docs, not for being found.
- **Structured data raises how often AI cites a page.** [A matched test by Ahrefs](https://ahrefs.com/blog/schema-ai-citations/), of 1,885 pages that added JSON-LD, found no significant gain in citations on any of the AI products it measured. Ascribe writes breadcrumbs and the site's name, which search engines document, and nothing more.
- **Titles and descriptions have length limits, and a page needs exactly one top heading.** Google documents no limit for [titles](https://developers.google.com/search/docs/appearance/title-link) or [descriptions](https://developers.google.com/search/docs/appearance/snippet), and its [SEO starter guide](https://developers.google.com/search/docs/fundamentals/seo-starter-guide) lists the number and order of headings among the things not to focus on.
- **"No AI" markers control how AI uses a page.** None of them is a standard: the [IETF's AI preferences](https://datatracker.ietf.org/wg/aipref/documents/) are still drafts, and no major operator's crawler documentation says it honors one. `robots.txt` is a request, and the fetchers that act for a person, [Google's](https://developers.google.com/search/docs/crawling-indexing/google-user-triggered-fetchers), [OpenAI's](https://developers.openai.com/api/docs/bots), and [Perplexity's](https://docs.perplexity.ai/guides/bots), document that they may ignore it.

**Where the evidence is thin:** no study looks at documentation or developer questions alone; the two strongest results for agents are each one vendor's study; and no project has published a before-and-after number for any change to versioned docs in search. Treat a claim that one change will move your traffic, Ascribe's included, as untested until your own numbers say otherwise.

## Related resources

- [Act on the project report](../checks/report.md): the pages a build lists or keeps out, and the rest of what `ascribe report` says.
- [`[crawlers]`](../content-model/output.md#18-crawlers): what each kind of AI crawler may do.
- [Index your docs for your product's agents](product-index.md): giving your product's own agents the docs.
