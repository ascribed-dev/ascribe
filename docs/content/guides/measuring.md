---
title: What to measure
description: Where a team sees how search engines and AI agents treat its docs, since Ascribe sees no traffic, and which of the advice about them has evidence behind it.
available: next
---

Ascribe builds your docs and checks them before they're deployed. It runs no server, sees no traffic, and calls no model, so it can't tell you how many people found a page through a search engine, or whether an assistant answered a question about your product correctly. This guide says where a team looks for those, and how much each piece of the advice you'll read about search and AI is worth.

## What Ascribe tells you before a deploy

Everything Ascribe knows about a page is in its record: its address, title, description, and listing, and where the listing comes from. The head of each page, the sitemap, `robots.txt`, and `llms.txt` are all written from the records, so they can't disagree with each other.

- **`ascribe check`** reports what would go wrong for a reader that isn't a person: two pages with the same title or description, a title its page's own heading contradicts, a page kept out of search whose address `robots.txt` won't let a crawler fetch, a page too long for an agent to read in one fetch. Each is advice you can raise or turn off in [`[checks]`](../reference/content-model.md#19-checks).
- **`ascribe report pages`** lists every page's record, and `--not-listed` the ones kept out of search, with why. It's what a team would otherwise crawl its own site to see. See [the project report](report.md#pages).
- **`ascribe report agents --site <URL>`** runs the delivery spec's checker on the published site, for what only a running server can answer. See [the site and agents](report.md#the-site-and-agents).

None of these is a score. Each problem is listed at its file, with what to do about it; a number would count different things from one tool to the next, and invite tuning to the number.

## Where search engines report

The search engines report on your site themselves, once you've shown them you own it: Google in [Search Console](https://search.google.com/search-console), Bing in [Webmaster Tools](https://www.bing.com/webmasters). Look at:

- **Which pages are indexed, and why the others aren't.** A page excluded by `noindex` should be one `ascribe report pages --not-listed` lists; one that's listed and still excluded is worth reading about in the report's reason.
- **The queries that show each page, and how often it's chosen.** A page shown often and rarely chosen usually has a title or description that doesn't say what's on it.
- **Their AI features,** where an engine counts them in its reports. What each engine counts, and how it labels it, changes; read its own documentation for the current answer rather than a summary of it.

## Where agents show up

Your host's request logs are the only place agents reading your docs show up. Look for:

- **Requests for the Markdown versions,** the `.md` address of each page, which a site publishes with `[consumer] agents = true`. A browser never asks for them, so they're the cleanest sign of an agent reading, and the address says which page it read.
- **Requests for `llms.txt`,** which an agent reads to choose a page.
- **The user agents** of the crawlers you've made a choice about in [`[crawlers]`](../reference/content-model.md#21-crawlers): the tokens of each purpose are listed there, with where each operator documents its own. An assistant fetching a page because a person asked often sends no token of its own, so these counts are low, never high.

## Whether the answers are right

No report tells you whether an assistant answers questions about your product correctly. The test is your own: keep a set of the questions your readers ask, ask them again the same way after a change to the docs, and compare the answers with what the docs say. Write the questions down, in your repository, so the set is the same each time, and ask them before the change too, since the models change on their own. Ascribe calls no model, so it can't run this for you.

## What has evidence, and what doesn't

Most of what's written about being found by AI has no evidence behind it, and some of it is contradicted. This is what the research behind Ascribe's choices found; it's the part of this guide most likely to be read as a promise, so it says how strong each finding is.

**Documented by the search engines themselves:** an accurate title and description for each page, different from every other page's; a canonical link; a sitemap. Ascribe writes each of these from the page records, and checks the first two.

**Measured, once each:**

- **A pointer to `llms.txt` on each Markdown page** cut an agent's fetches of addresses that don't exist from 2.23 to 0.11 a task, in one controlled benchmark. Every Markdown page Ascribe publishes has one.
- **A compact index of the docs in a product's `AGENTS.md`,** about 8 KB, took one vendor's evaluation from 53% to 100%. [`ascribe agents index`](agents.md#index-your-docs-for-your-products-agents) writes one.
- **Size.** Agents' fetch tools cut a page off at around 100,000 characters and don't ask for the rest. `ascribe check` reports a page over the delivery spec's limit, 50,000 characters.

**Not supported, or contradicted:**

- **`llms.txt` gets a site found by AI search.** Google says its search doesn't use the file, and in one study of 137,000 domains, 97% of the files got no requests in a month. It's worth publishing for the agents that are already reading your docs, not for being found.
- **Structured data raises how often AI cites a page.** A matched test found no significant gain. Ascribe writes breadcrumbs and the site's name, which search engines document, and nothing more.
- **Titles and descriptions have length limits, and a page needs exactly one top heading.** Google documents no limits, and lists the number and order of headings among the things not to focus on.
- **"No AI" markers control how AI uses a page.** No major operator has committed to honoring any of them. `robots.txt` is a request, and some crawlers document that they may ignore it.

**Where the evidence is thin:** no study looks at documentation or developer questions alone; the two strongest results for agents are each one vendor's study; and no project has published a before-and-after number for any change to versioned docs in search. Treat a claim that one change will move your traffic, Ascribe's included, as untested until your own numbers say otherwise.
