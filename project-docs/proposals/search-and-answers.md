# Proposal: being found by search engines and by AI

A proposal, not a plan. It says what the feature is and what it would look like, so the idea can be judged before anything is designed in detail. It follows the research in [SEO and AEO for technical docs](../reports/SEO%20and%20AEO%20for%20technical%20docs.md).

## The goal

A site built with Ascribe is found by search engines, read correctly by AI assistants and coding agents, and tells a consistent story to both. The team that runs it can see and control how each page is treated, from the source, before anything is deployed.

"SEO" is the search-engine half of this, and "AEO" (answer engine optimization) the AI half. The research found they share most of their mechanics, and that for documentation the valuable part is the same in both: say true things about each page, once, everywhere.

## What the research found

**Most of what's sold as AEO has no evidence behind it, and some is contradicted:**

| Claim | Finding |
|---|---|
| `llms.txt` gets a site discovered by AI search | Google says Search doesn't use it. In one study of 137,000 domains, 97% of the files got no requests in a month |
| Structured data raises AI citations | No significant gain in a matched test |
| A docs MCP server improves agents' answers | The one independent test showed no benefit: the agent rarely called it |
| Titles and descriptions have length limits; a page needs exactly one top heading | Google documents no limits, and lists heading count and order among things not to focus on |
| FAQ and how-to markup earn rich results | Google ended both, the first in May 2026 |
| Content signals and "no AI" markers control AI use | No major operator has committed to honoring any of them |

**What does have support:**

- **A pointer to the index on each Markdown page.** The one controlled benchmark found it cut agents' fetches of dead addresses from 2.23 to 0.11 a task. Ascribe ships this already.
- **A compact index of the docs placed where the agent already is,** in a product's `AGENTS.md`. One first-party evaluation went from 53% to 100% with one of 8 KB.
- **Accurate, unique titles and descriptions; canonical links; a sitemap with true dates.** These are documented by the search engines themselves.
- **Size.** Agents' fetch tools cut a page off around 100,000 characters and don't ask for the rest.

**Where teams actually get hurt is disagreement between outputs.** Hosted platforms' changelogs record the same class of bug again and again, each fixed after it shipped: hidden pages in the sitemap, raw Markdown files indexed by Google, canonical links to pages that don't exist, `noindex` on a page the canonical points at. One project took about 18 months to recover from the last of those.

Every existing checker needs a deployed site. Nothing reports these before deploy. Ascribe knows every page's title, description, route, headings, version, variants, and links at build time, so it can.

## What exists

- **Ascribe emits** `llms.txt`, a Markdown version of each page, and the pointer, and checks page size and missing descriptions. `ascribe report` runs a link checker and the agent delivery spec's checker against a published site.
- **Nothing else about search is Ascribe's today.** The docs site writes its own page head by hand in its layout (`site/src/layouts/Base.astro`): the canonical link, the Open Graph tags, the social card. It has no sitemap and no `robots.txt`. The first Ascribe site built these for itself or went without, and so will every other.
- **Proposed, and related:** [versions](versions.md) (canonical links across versions, and whether old versions are indexed), [permalinks](permalinks.md) (stable addresses, and a committed list), redirects (in the [brainstorm](../brainstorm.md)), and the [website](website.md).

## The principle: one set of facts, every output from it

For each page of each build, Ascribe works out one record: its address, title, description, canonical address, version and status, a hash of its content, and how it's to be treated by crawlers. Every output that speaks about the page is written from that record:

| Output | From the record |
|---|---|
| The page's head | Title, description, canonical link, Open Graph tags, robots rule, a link to the Markdown version |
| The sitemap | Address and last-modified date, for pages that are indexed |
| `llms.txt` | Title, description, address of the Markdown version, for pages agents should read |
| The Markdown version | Its first lines: what version and status it's for |
| Structured data | The page's place in the site, as breadcrumbs |

They can't disagree, because none of them is written separately. A page marked as not indexed is out of the sitemap, carries the rule in its head, and isn't offered to agents, by construction.

The records are also a file in the build's output, as the permalinks list is. `@ascribed/astro` turns them into a head, a sitemap, and the rest. A site built with another generator gets the file.

## What a build would write

- **A complete head for each page,** through a component the integration supplies, so a layout includes one thing instead of writing fifteen tags.
- **A sitemap.** Addresses and a last-modified date only: search engines ignore the other fields. The date comes from a hash of the page's content, since they also ignore dates that turn out not to be true, and "every page changed today" is the commonest way to lose that trust. That needs a committed record of each address's hash and the date it last changed.
- **Breadcrumbs and the site's name, as structured data.** The two kinds a docs page still gets anything for. Described as what they are: they change how a result is displayed, and nothing else.
- **Version and status in the text of the Markdown version.** "Self-managed 3.4. Not the current version." Fetch tools throw away a page's metadata, so what an agent must know has to be in the words. One `llms.txt` for each version.
- **Every variant, written out and labeled,** in the Markdown version. Content that exists only in a tab that isn't showing is lost when a page is converted for an agent.
- **An index split by section** when it would pass the size agents read.
- **A compact index for a product's `AGENTS.md`,** with the Markdown pages as a folder that can ship inside a package. The evidence is one vendor's evaluation, and it's the strongest there is.

## Managing how crawlers treat pages

A team needs to say, per page and per site, what crawlers may do. The research found the working controls are few, and that most of the mistakes come from combining them.

**The controls that exist:**

| Control | Where it's written | What it does |
|---|---|---|
| `robots.txt` | One file, at the host's root | Asks crawlers not to fetch addresses. Names crawlers by token |
| `noindex` | A tag in a page's head, or a response header | Tells a search engine not to list a page it has fetched |
| Snippet controls (`nosnippet`, `max-snippet`, `data-nosnippet`) | A page's head, or an element | Limits what a result, or an AI answer in search, may quote |
| A canonical link | A page's head | Says which of several similar pages is the one to list |

There's no standard beyond these that anyone honors. Files and tags for "no AI" exist as proposals and drafts, and no major operator has committed to them.

**What Ascribe would add:**

- **Every address is visible unless something says otherwise.** A page that states nothing is listed by search engines and offered to agents.
- **A page says how it's treated, in the source.** One setting with a few values (listed and offered to agents; not listed; not quoted), set for a version, for a page type or a folder in `ascribe.toml`, or on a page. The record carries it, and every output follows.
- **A narrower setting can restrict and never loosen.** The levels run from the site, to a version, to a type or folder, to the page. Each may be stricter than the one above it and none may be looser. So "this version is not indexed" is a guarantee: no page inside it can opt back in, and a page that tries is an error that says which rule it's under.
- **Crawlers by purpose, not by name.** Operators now run three kinds of bot: one that gathers training data, one that builds a search index, and one that fetches a page because a person asked. A project states a choice for each purpose, and Ascribe writes the rules for the tokens it knows (`GPTBot`, `OAI-SearchBot`, `ClaudeBot`, `Claude-SearchBot`, `PerplexityBot`, `Google-Extended`, and the rest). The list of tokens is data shipped with Ascribe and updated with it, and a project can add lines of its own.
- **Said honestly.** `robots.txt` is a request. OpenAI, Perplexity, and Google document that their fetch-on-request bots may ignore it; Anthropic says its bots honor it. Blocking a search bot removes a site from that product's answers. A host's or a CDN's own bot settings can override the file. The reference says each of these where the setting is described.
- **Nothing by default.** A project that states no choice gets no rules about AI crawlers. Ascribe doesn't take a position for its users.

**The mistakes it would catch,** each documented in the research:

- A page that's `noindex` and also blocked in `robots.txt`, so the crawler never sees the rule.
- `noindex` on a page that another page's canonical link points at.
- A canonical link to a page that doesn't exist, or doesn't exist in that version.
- A page in the sitemap, or in `llms.txt`, that's marked as not listed.
- A `robots.txt` that would be published under a path, where it does nothing: only the one at a host's root counts.

**What stays the host's.** A response header is the only way to keep a Markdown file out of a search index, and Ascribe serves nothing. The same goes for the `robots.txt` at the root of a site whose docs are mounted under a path: Ascribe writes the rules, and the site puts them where they count.

### Reaching the host

Headers and redirects are each host's own format, and Ascribe shouldn't learn them all. Three layers, from most portable to least:

| Layer | Works on | For |
|---|---|---|
| **A neutral list** in the build's output: every redirect and every header rule, as data | Anything, with a few lines of glue | A team on a host Ascribe doesn't know |
| **The site generator.** Astro has its own setting for redirects, which its adapters turn into each host's format | Whatever Astro's adapters cover | Redirects, through `@ascribed/astro`, with no host named in Ascribe |
| **One host format, written directly:** the `_headers` and `_redirects` files | Netlify, which Ascribe's own site uses, and hosts that read the same files | Headers, which the generator has no setting for |

And one fallback that needs no host at all: a page that sends the reader on, as a permalink's address already is. It's the only thing that works on a host with no redirect rules of its own.

So Ascribe names one format, not four, and the neutral list is what keeps that from being a limit. More formats are added when someone needs one.

## Checks before deploy

Under the [content checks](../content-checks/README.md) contract: advisory, able to be raised or turned off, and acknowledged with `@intended` where a finding may be right as it is.

- **Titles and descriptions:** two pages with the same one; a title that's only the site's name; a title that disagrees with the page's heading.
- **Addresses:** a page no page links to; a link that goes through a redirect; a chain or loop of redirects; a redirect to nothing.
- **What went away:** an address or a heading's id that was published and now isn't, with no redirect. This needs the committed record of addresses above.
- **Versions:** the checks in the list above, across versions, and a link that leaves a frozen version.
- **For agents:** the parts of the delivery spec's checks that don't need a server, so they're found in the editor and not after a deploy.

## For the team

- **A table of every page's record** in `ascribe report`: address, title, description, canonical, version, how it's treated. The thing a team otherwise crawls its own site to get.
- **Findings by file, and no score.** The platforms' scores each count different things and can't be compared, and a number invites tuning to it.
- **What to measure, as guidance.** Ascribe has no server and sees no traffic. The guide says where to look: the search engines' own reports on AI features, and requests for the Markdown versions in a host's logs, which are the cleanest sign of agents reading. For whether answers are right, the test is a team's own set of questions asked again after a change. Ascribe calls no model, so it can describe that and not run it.

## What this leaves out

- **A docs server for agents,** `llms-full.txt`, and blocks of content shown only to agents or only to people. The first two have no measured benefit. The third lets the two readers see different docs, which is the disagreement this proposal exists to prevent.
- **Structured data beyond breadcrumbs and the site's name.**
- **Rules for content style:** question-shaped headings, answers first, and the like. The research found advice and no measured effect.
- **What the site generator owns:** page speed, social card images, the order of a page's HTML, site search.
- **What the host owns:** status codes, headers, content negotiation, bot protection, logs.
- **Submitting to search engines.** Ascribe can write the list of what changed. Sending it is a deploy step's.

## Where the evidence is thin

- No study looks at documentation or developer questions alone.
- The two strongest results for agents are each one vendor's study.
- No project has published a before-and-after number for any fix to versioned docs in search.
- Version labels in the text are argued from how fetch tools and models behave, not measured.

## Settled

- **The record of addresses is its own file, apart from the permalinks list.** Permalinks are a few addresses someone promised to keep. The record is every address a build publishes, kept so Ascribe can tell when one goes away and when a page last changed. Being in it says nothing about visibility: every address is visible unless set otherwise.
- **A version's rule is a guarantee.** A narrower setting restricts and never loosens.
- **One host format to start,** with a neutral list for the rest, and redirects through the site generator.

## One plan, with one piece carved out

Asked of the three pieces (the outputs, the crawler controls, the checks): do they share a layer and a seam, and is there one use case that runs through all of them?

They do, and there is. All three are made from the page's record, and all three pass through the same join: Ascribe's record becoming what a site actually serves. The use case that crosses everything:

> A page marked as not listed is out of the sitemap, carries the rule in its head, isn't offered in `llms.txt`, and `ascribe check` says so if `robots.txt` would also hide it from the crawler that has to read the rule.

That's one thread. It needs the setting in the content model, the record in the resolver, three outputs, the integration, and a check, and it's the proposal's whole claim in miniature: the outputs can't disagree. Split into three plans, each would build its part of the record alone, and the agreement between them, which is the point, would be the last thing joined.

So it's one plan. Its thread is that page. Later phases widen it: the rest of the head and the structured data, crawlers by purpose and the rules for them, the levels a setting can come from, reaching the host, the remaining checks, the table in the report, and the index for a product's `AGENTS.md`.

**What's carved out** is everything that needs to know what was published before: an address or a heading's id that went away, the redirect it should have left, and a sitemap's dates. That's a different seam, Ascribe reading its own state from an earlier run, and it's bound up with redirects, which aren't designed. It belongs with that work. The plan here ships a sitemap of addresses, and gains dates when the record of addresses exists.

**What waits on another plan:** a version's rule, and the checks across versions, need [versions](versions.md). The plan leaves a place for them and doesn't build them first.

**What comes before the thread:** one decision. The setting's name and values, and the record's shape, are contracts from the first phase on.
