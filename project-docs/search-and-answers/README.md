# Search and answers

A site built with Ascribe says one true thing about each page to every reader that isn't a person: a search engine, an AI assistant, a coding agent. The team that runs it sees and controls how each page is treated, from the source, before anything is deployed. The idea, and the research behind it, are in the [proposal](https://github.com/ascribed-dev/ascribe/blob/097cc3d496fa2f3b08a69958ccacc50c76849126/project-docs/proposals/search-and-answers.md). This is the plan for building it.

## Who it's for

- **A team that publishes docs with Ascribe,** which today writes its own page head, has no sitemap, and can't say "don't list this page" anywhere.
- **Someone who has to keep a page out of search,** and needs that to hold in every output at once.
- **Ascribe's own site,** whose root has had no `robots.txt`, no sitemap, and no `llms.txt` since the docs moved under `/docs/`.

## What exists today

- **A page's facts are known at build time** and written in three places that don't share a source: the site output's frontmatter (title, description), `llms.txt` in the plain output (`ascribe_resolve::llms`), and whatever a site's layout writes in its head.
- **The docs site writes its head by hand,** in `site/src/layouts/Base.astro`: the title, the description, the canonical link (`new URL(Astro.url.pathname, site)`), the Open Graph tags, the social card. It has no sitemap and no `robots.txt`.
- **`[consumer] agents = true`** publishes `llms.txt` and each page's Markdown, with a pointer to the index on each. `ascribe check` has two advice checks about them (`description-too-long`, `llms-section-large`).
- **A build already writes one list for a site to draw from:** `_ascribe/permalinks.json`, in the site output, which `@ascribed/astro` turns into a page for each permalink (`packages/astro/src/permalinks.ts`). It's the pattern for a list of pages.
- **One Astro site can hold several projects,** each under its own base path. Ascribe's own has three: the docs, the site's own pages, and a demonstration with no route.
- **Checks about quality have a contract:** each is advice a project can raise or turn off in `[checks]`, with a kind of next step, and can be acknowledged with `@intended` or `intended` in frontmatter.
- **Four frontmatter keys are reserved** for pages: `available`, `variant`, `permalink`, and `intended` (`crates/ascribe-model/src/sections.rs`).
- **Nothing reads what an earlier run published,** except `ascribe.permalinks`, which is committed.

## Decisions

These are settled. Don't reopen them in a phase; if one can't be met, stop and report.

1. **One record for each page of each build, and every output written from it.** The head, the sitemap, `llms.txt`, and the Markdown version's first lines can't disagree, because none is written separately.
2. **The records are a file in the build's output.** `@ascribed/astro` turns them into a head, a sitemap, and `robots.txt`. A site built with another generator gets the file.
3. **Every address is visible unless something says otherwise.** A page that states nothing is listed by search engines and offered to agents.
4. **A page says how it's treated, in the source,** with one setting of a few values.
5. **A narrower setting can restrict and never loosen.** The levels run from the site, to a version, to a type or a folder, to the page. A page that tries to loosen is an error that names the rule it's under.
6. **Crawlers are chosen by purpose, not by name:** gathering training data, building a search index, fetching because a person asked. The tokens are data shipped with Ascribe.
7. **Nothing by default.** A project that states no choice gets no rules about AI crawlers. Ascribe takes no position for its users.
8. **Said honestly.** `robots.txt` is a request, some bots document that they may ignore it, and blocking a search bot removes a site from that product's answers. The reference says each where the setting is described.
9. **Ascribe serves nothing.** A response header is the host's. Ascribe writes the rules as data, and in one host's format: `_headers`.
10. **Checks are advice,** under the content checks contract: able to be raised, turned off, or acknowledged. A setting that contradicts itself is an error, since the project isn't valid.
11. **No score.** Problems are listed by file.
12. **Left out, on the evidence:** a docs server for agents, `llms-full.txt`, content shown only to agents or only to people, structured data beyond breadcrumbs and the site's name, and rules for content style.
13. **No work is done for released versions.** Ascribe is before 1.0.

The rest were made in [phase 1](phase-1-decisions.md), which keeps the options each was chosen from. Each fixes a name or a shape phase 2 ships.

14. **The setting is `listing`, with three values in one order: `listed`, `unquoted`, `unlisted`.** Absent means `listed`. `unquoted` is listed, in the sitemap and `llms.txt`, and asks not to be quoted (`nosnippet`). `unlisted` asks not to be listed (`noindex`) and is out of the sitemap and `llms.txt`. The name says what happens in a reader's terms, where `robots` would name one output of several. The values are an order from open to closed, which is what "restrict and never loosen" needs; the crawler's own words (`noindex`, `nosnippet`) are a set with no order, and a boolean with a second key could contradict itself.
15. **A page that isn't listed is still built.** It has its address and its Markdown version. It isn't offered: nothing Ascribe writes points a crawler or an agent at it. So one setting governs search engines and agents together, which is the proposal's claim: the outputs can't disagree. What a kind of crawler may fetch is a separate choice, in `[crawlers]`.
16. **A project's words to crawlers are in a table of their own, `[crawlers]`.** Phase 2 gives it `lines`, a project's own `robots.txt` lines, and phase 4 adds its choices by purpose beside them. Not under `[consumer]`, which is how the site output fits one site generator: what a project says to crawlers is about the content, whoever builds the site.
17. **The records are `_ascribe/pages.json`, in the site output, shaped as the permalinks list is.** At the top: `version`; `site`, the project's address, absent when `[consumer] site` isn't set; `pages`; and `robots`, an object holding what the project says to crawlers, with `lines` in phase 2. Each is contract from phase 2 on, and later phases add keys and change none. `pages` is keyed by route, and an entry doesn't repeat its route. Phase 2's entries have `source`, `title`, `description`, `canonical`, `listing`, and `markdown`. `title` and `description` are plain text: the site output's `title`, not `formatted.title`, since `<title>`, a link preview, and a sitemap can't take markup.
18. **`source` is the page's path from the content root, with `/`,** and it's the name [#274](https://github.com/ascribed-dev/ascribe/issues/274) uses for the same thing on a collection entry. One name in both places, whichever lands first.
19. **`canonical` is the page's own address, absolute.** It's absent, with `site`, when `[consumer] site` isn't set, and a site without it gets no canonical links and no sitemap. It becomes another page's address only with versions.
20. **One sitemap and one `robots.txt` for a site, written by the integration from every project's records,** at the host's root. A `robots.txt` under a path does nothing, which is one of the mistakes the plan sets out to catch. The sitemap holds every listed page of every project that has routes, and every other page the site built, since a site has pages of its own that no project knows. `robots.txt` holds each project's lines, then the sitemap's address. Two projects that say contradictory things about one purpose are an error that names both.
21. **A layout includes a component, `@ascribed/astro/Head.astro`, given the page's entry.** Phase 2: the title, the description, the canonical link, and the robots rule. Phase 6 adds to it. A function that returned the record would leave every site writing its own tags, and able to leave one out.
22. **The title is the page's own, and a site says how its titles read with one property, `title`:** a function from the page's title and its record to the whole title, such as `(page) => `${page} · Ascribe``. The record it's given has the route it's keyed by beside its fields, so a site can title its home page with its name alone, as Ascribe's does. Decided now because it's a property of a published component from phase 2 on.
23. **A fragment can't use `listing`, and a type can't declare a field of that name,** as with `available` and `permalink`.
24. **With `agents = false` a build publishes no `llms.txt`, and `listing` still governs the head and the sitemap.**
25. **The sitemap has addresses and no dates,** until the record of what was published exists.
26. **Phase 2's check is advice, configurable, with the next step `fix`.**

Four more, from a review of the plan against the code and against how `robots.txt` is read (RFC 9309), before phase 2 started. None changes a decision above; each says what one left unsaid.

27. **A path in `lines` is from the project's own root, and the build writes it from the host's.** A project writes `Disallow: /drafts/`, and with a base path of `/docs/` the file says `Disallow: /docs/drafts/`. The base path keeps its one home, and a project's lines reach only its own pages, which is the right scope when a site holds several. Only `Allow` and `Disallow` values are changed; any other line is written as it is.
28. **Lines before any `User-agent` line are for every crawler.** Ascribe opens the `User-agent: *` group they belong to. A project that names a crawler writes its own group, whole, in `lines`.
29. **A crawler obeys only the most specific group that names it,** and the `*` group doesn't apply to it at all. So anything Ascribe writes for a named crawler has to carry everything that crawler should obey. Phase 4's design follows from this, and a token in two groups is an error.
30. **A site whose Astro `base` isn't `/` has no host root to write to.** The integration writes the sitemap under the base, writes no `robots.txt`, and says so in the build, with the lines the host's own `robots.txt` needs. It doesn't write a file where it does nothing.

## The thread

**The use case:** a page marked as not listed is out of the sitemap, carries the rule in its head, isn't offered in `llms.txt`, and `ascribe check` says so if the project's own `robots.txt` lines would hide it from the crawler that has to read the rule.

**The layers it crosses:**

| Layer | Where | What the thread needs from it |
|---|---|---|
| The language | `SPEC.md` §2.1, `ascribe-model` | One reserved frontmatter key |
| The content model | `ascribe-model`, `ascribe.toml` | A table for what the project says to crawlers, holding its own `robots.txt` lines |
| Resolve | `ascribe-resolve` | Each build's page records |
| Check | `ascribe-check`, the registry | One advice check: a rule no crawler can read |
| Emit | `ascribe-emit` | The records, in the site output; `llms.txt` without what isn't listed |
| The consumer | `@ascribed/astro` | A head from a record; one sitemap and one `robots.txt` for the site, at the host's root |
| The first site | `site/` | The head it writes by hand gives way, and the root gets a sitemap and a `robots.txt` |
| The editor | `ascribe-lsp`, the extension | Nothing in the thread. Phase 3 adds completion, hover, and a quick fix for the key |

**The seam it tests** is Ascribe's record becoming what a site serves, in three places the integration has never reached: a page's `<head>`, which the site's layout owns; and two files at the host's root, which is above the base path of every project that isn't mounted there, and shared by all of them. If the head can't be given to a layout as one thing to include, each site goes on writing its own and the outputs can disagree again, which is the plan's whole claim. If one site's projects can't share a sitemap and a `robots.txt`, the files become the site's to assemble from the records, and decision 2 gets narrower.

**Left out, and where it comes back:**

| Left out | Phase |
|---|---|
| The setting on a folder, a type, or the whole site; the third value, not quoted | 3 |
| Crawlers by purpose, and the rules written for them | 4 |
| Keeping the Markdown versions out of a search index, which takes a header | 5 |
| The rest of the head, breadcrumbs, and the site's name as structured data | 6 |
| What agents read: variants written out, and an index for a product's `AGENTS.md` | 7 |
| The other checks, and the table of every page's record | 8 |

Until phase 5, the Markdown version of a page that isn't listed can still be indexed, since only a header prevents it. The reference says so while that's true.

## Phases

| Phase | Result |
|---|---|
| [1: Decisions](phase-1-decisions.md) | Done. The setting's name and values, the record's shape, and where a project's words to crawlers live are decisions 14 to 26. |
| [2: The thread](phase-2-thread.md) | A page can be marked as not listed, and its head, the sitemap, and `llms.txt` all follow. An Astro site gets a head, a sitemap, and a `robots.txt` from Ascribe. |
| [3: Levels](phase-3-levels.md) | The setting can be made for a folder, a type, or the site, and nothing narrower can loosen it. A page can be listed and not quoted. |
| [4: Crawlers by purpose](phase-4-crawlers.md) | A project says what training, search, and on-request bots may do, and `robots.txt` has the rules for each bot Ascribe knows. |
| [5: Reaching the host](phase-5-host.md) | A page that isn't listed has a Markdown version that isn't either, on a host that reads `_headers`, and as data on any other. |
| [6: The whole head](phase-6-head.md) | A layout includes one component for everything a page's head says about it, with breadcrumbs and the site's name. |
| [7: For agents](phase-7-agents.md) | The Markdown version has every variant, labeled, and a product can ship a compact index of its docs. |
| [8: Checks, and the table](phase-8-checks.md) | `ascribe check` reports what would disagree before a deploy, and `ascribe report` lists every page's record. |

Phase 1 came first because phase 2 ships a frontmatter key, a table in `ascribe.toml`, and a file in the output. After the thread the order is by risk: phases 3 and 4 settle what the setting can say, phase 5 is the one new seam (the host), and phases 6 to 8 widen what's already joined.

Each phase's result is something a person can run or open. None builds a layer for a later phase to use.

## Carved out, and waiting

- **What was published before.** An address or a heading's id that went away, the redirect it should have left, and a sitemap's last-modified dates all need Ascribe to read its own state from an earlier run. That's a different seam, bound up with redirects, which aren't designed. This plan ships a sitemap of addresses, without dates.
- **Versions.** A version's rule, version and status in the Markdown version's first lines, an `llms.txt` for each version, and the checks across versions wait for the [versions proposal](../proposals/versions.md). Decision 5's levels leave the place for a version, between the site and a type.
- **Navigation.** Breadcrumbs say a page's place in the site, and the content model has no navigation (the surface audit's finding 28). Phase 6 draws them from a page's folders, and says that's what they are.

## The other plans

- **The surface audit.** Its conventions apply: a choice about the content lives in `ascribe.toml`, a problem is a "problem", and a new section of `ascribe report` is named as the others are. Two of its issues touch this work: [#273](https://github.com/ascribed-dev/ascribe/issues/273), the integration supplying Astro's `site` from `[consumer]`, and [#274](https://github.com/ascribed-dev/ascribe/issues/274), the page's source path on a collection entry, which the record also carries.
- **The website.** It left the root without `robots.txt`, a sitemap, or `llms.txt` for this plan to fill. Phase 2 fills the first two. Whether the root gets an `llms.txt` is phase 7's. The weekly Report workflow already runs the delivery spec's checker against `/docs/`, where the docs' `llms.txt` is, and not against the root; phase 2 leaves it there, and phase 7's decision says whether it should look at the root too.
- **Permalinks.** A permalink's forwarding page is `noindex` already, by the integration. It isn't a page of a build, has no record, and is never in the sitemap.

## Rules for every phase

- Branch before committing; never commit to `main`.
- Read the current code before the phase file's pointers. If the phase file and the code disagree, or a decision above can't be met, stop and report instead of choosing silently.
- **The language and the contracts change only by a recorded decision.** Every phase from 2 on changes at least one. Go through the matching list in `project-docs/checklists.md` first, and add the decision to `project-docs/decisions.md` in the same pull request.
- **A contract's shape doesn't change after it ships.** A later phase may add to the record, the table, or a component's properties. It may not change what an earlier phase wrote. If a phase finds it must, stop and report.
- **A new diagnostic** is added to `tests/conformance/diagnostics.toml` and `crates/ascribe-core/src/diagnostics.rs`, in the same order, with a conformance case. A check about quality is advice, configurable, with its kind of next step.
- **Every output is tested against the record,** not against another output: a test reads the record and each thing written from it, and fails on a difference.
- **Two renderers must agree.** A change to the site output's markup changes the fixtures in `tests/render/`.
- **Project files are read through `ascribe_resolve::FileSystem`.** Libraries don't print, and don't panic on input.
- **The map of the surface** is regenerated when a phase adds to it (`node scripts/surface/inventory.ts`), and its diff is read against the conventions.
- **User-visible changes** update the page under `docs/content/` and the unreleased section of `CHANGELOG.md` in the same phase.
- **Ascribe's site is the first user.** A phase that adds something the site can use, uses it there.
- No phase history in code or docs. Describe what the code does now.
- Before finishing a phase, all of these pass:

  ```sh
  cargo fmt --all --check
  cargo clippy --workspace --all-targets --locked -- -D warnings
  cargo test --workspace --locked
  pnpm format:check && pnpm lint && pnpm typecheck && pnpm test
  ```

  And, in `site/`, after `npm run build:checkout`: `npm run typecheck && npm test`.

## Later, not in this plan

- Redirects, and the record of what was published.
- A sitemap's dates.
- A sitemap index. One file holds 50,000 addresses, and a site with more needs several and an index of them.
- More host formats than `_headers`.
- Sending a list of what changed to a search engine. A deploy step's.
- Measuring anything. Ascribe sees no traffic; the guide says where a team looks.
