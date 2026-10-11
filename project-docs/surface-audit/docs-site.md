# The docs site

Part of [The surface audit](README.md), [phase 4](phase-4-docs-site.md). The docs site is looked at twice (README decision 7): as the first site built with Ascribe, and as the way people find the surface. Section 1 is the first; sections 2 and 3 are the second. What's certain is written as findings 28 to 43 in [findings.md](findings.md#the-docs-site).

Everything here is from `main` at `a138d68b`, the published site at <https://ascribed-dev.com>, and a binary built from that code (`ascribe 0.2.0`), on 10 October 2026. Phase 2's generated inventory wasn't done, so "the surface" is the fifteen rows of the [proposal's table](../proposals/surface-audit.md#what-the-surface-is), with [phase 1's tables](inventory-slice.md) and the sources they were built from.

## 1. What the site built for itself

`site/src`, `site/scripts`, and `site/test` hold 26 files and 1,611 lines. Each file is in one of three groups below. A file that does two jobs is in the group of its larger job, and the other job is said in its row.

| Group | Files | Lines |
|---|---|---|
| A site's own business | 7 | 1,035 |
| Something any Ascribe site would need | 13 | 544 |
| A workaround for something Ascribe has and doesn't quite fit | 1 | 32 |

The third group is small in `site/` because most of the workarounds aren't there. They're in `docs/ascribe.toml`, in a test at the repository's root, and in a workflow, and are listed under [the third group](#a-workaround-for-something-ascribe-has-and-doesnt-quite-fit).

### A site's own business

No docs tool should do these for a site. Nothing is recommended for them.

| File | Lines | What it does |
|---|---|---|
| `site/src/styles/site.css` | 503 | The site's styles. Its colors and type are generated from `design/tokens.toml`. Six of its lines are a workaround, listed in the third group: "Unreleased features are this site's one lifecycle state." |
| `site/src/layouts/Base.astro` | 104 | The shell: the head, the header, the menu button's script, and a slot. About twelve lines of the head (the canonical link, the Open Graph tags, the description) are what the [search and answers proposal](../proposals/search-and-answers.md) would write: "The docs site writes its own page head by hand in its layout (`site/src/layouts/Base.astro`)". |
| `site/src/assets/logo.svg` | 1 | The header's mark, generated. |
| `site/src/theme.ts` | 18 | Reads the page's background from `site.css` for `<meta name="theme-color">`. |
| `site/scripts/checkout.mjs` | 73 | Builds the site with the Ascribe in this checkout. Only a site in Ascribe's own repository needs it. |
| `site/test/e2e.test.ts` | 209 | Thirteen tests of the built site in Chromium. Eight are of what the site wrote (the head, the mark, the sidebar, search, the edit link, the two not-found cases, the phone layout). Five are of what Ascribe's packages render in a real page: "a variant group is a switcher, and the reader's choice holds on the next page", "a page marked with an availability shows its badge", "notes and steps render as elements", "a code example taken from a file shows its title", and a glossary link landing on its heading. |
| `site/test/built.ts` | 127 | The tests' helpers: serves `dist/`, lists the published pages from `docs/.ascribe/build/site/site`, and reads `[consumer] site` out of `docs/ascribe.toml` with a regular expression. |

### Something any Ascribe site would need

Each of these would be written again by the next site. The last column names the proposal in `project-docs/proposals/` that would remove it, or says none does.

| What | Files | Lines | What it does | What Ascribe would need | Proposal |
|---|---|---|---|---|---|
| The sidebar | `site/src/nav.ts`, `site/src/components/Sidebar.astro`, `site/test/nav.test.ts` | 55, 47, 24 | Four groups and nineteen pages, listed by source file. The test fails when "a published page is missing from this list, or when it names a page that doesn't exist". | What `site/README.md` lists under "Navigation": "an order and a group for each page", checking, "the result in the outputs", and "a label for a page in the navigation". | None. The [brainstorm](../brainstorm.md#navigation-and-site-structure) records it as a gap the site found. The [versions proposal](../proposals/versions.md) works around the same gap for its archive: "The content model has no navigation". Finding 28. |
| A route for each page | `site/src/pages/[...slug].astro`, `site/src/content.config.ts` | 20, 7 | The collection and the route, as the Astro guide's steps 4 and 5 give them. `examples/astro-site/src` has the same two files. | A starter that already has them. | None. Finding 29. |
| The page's frame | `site/src/layouts/Docs.astro` | 38 | The title, the availability badge (`@ascribed/astro/Availability.astro`), the description, the edit link, and the table of contents. | Nothing for the frame. The edit link is the next row. | None; the frame is the site's. |
| The edit link | `site/src/site.ts` | 19 | Finds a page's source file by taking apart the path the collection loaded it from: `/\.ascribe\/build\/[^/]+\/site\/(.+)$/`, which works because "the site output mirrors the content root's paths". | The page's source path in what the collection gives a layout. `_ascribe/permalinks.json` already has it for a permalink (`"source": "a.md"`). | None. Finding 31. |
| The table of contents | `site/src/components/Toc.astro` | 24 | Lists the `h2` and `h3` headings Astro's `render` returns. | Nothing: Astro gives the headings, and the Astro guide says so ("table of contents (`headings` from `render`)"). | None needed. Finding 29. |
| Search | `site/src/components/Search.astro`, and `pagefind --site dist` in `site/package.json` | 78 | A dialog that loads Pagefind's interface over an index of `dist/`. | Nothing planned. Two proposals leave it out on purpose: [search](../proposals/search.md) ("Search for readers, and anything a site serves. That's Pagefind's job, or the site's choice.") and [search and answers](../proposals/search-and-answers.md) ("What the site generator owns: page speed, social card images, the order of a page's HTML, site search."). | None, deliberately. Finding 29. |
| The not-found page | `site/src/pages/404.astro` | 22 | A page with one paragraph, and `<MissingPermalink>` from `@ascribed/astro` with its default text replaced by nothing. | Nothing more: the part about permalinks is already Ascribe's. | [Permalinks](../proposals/permalinks.md), built. Finding 29. |
| Links in the built pages | `site/test/links.test.ts` | 71 | "Every link and anchor inside the built site lands: on a file dist/ serves, and on an element with that id." `ascribe check` checks the same links in the source; this checks them again after the layout, the sidebar, and Astro have had their turn. | A check of a built site's own links. `ascribe report links` asks about external links, and `ascribe report agents` reads a published site. | None. Finding 29. |
| Redirects for pages that moved | `site/test/redirects.test.ts`, and `site/public/_redirects` | 41 (and 5) | The file has five lines of comment and no rule yet. The test checks that each rule "leads from an address that isn't a page to one that is". `site/README.md`: "when one moves, add a line to `public/_redirects`." | A record of the addresses a project has published, so a moved page is noticed and its redirect written. | [Search and answers](../proposals/search-and-answers.md) plans "Redirects, through `@ascribed/astro`", and carves out the part that would notice a moved page: "everything that needs to know what was published before". [Permalinks](../proposals/permalinks.md) covers links from outside by name, not a page's own address. Finding 29. |
| A page as Markdown, when asked | `site/netlify/edge-functions/markdown.ts` (outside the three folders), `site/test/negotiation.test.ts` | 84, 98 | Answers `Accept: text/markdown` with the page's `.md`. The Astro guide tells a Netlify user to copy it: "On Netlify, an edge function does it: this site's is `markdown.ts`". | Nothing planned: [search and answers](../proposals/search-and-answers.md) says "What the host owns: status codes, headers, content negotiation, bot protection, logs." | None, deliberately. Finding 29. |

### A workaround for something Ascribe has and doesn't quite fit

| What | Where | How much | What it does | What Ascribe would need | Proposal |
|---|---|---|---|---|---|
| "Unreleased", declared by hand | `docs/ascribe.toml`: `[dimensions.release]`, `[versions]`, `[lifecycle.unreleased]`, `[features.next]`; `site/src/styles/site.css` lines 52 to 57 | About 25 lines of the model with their comments, 6 of the stylesheet, and 106 `next` marks in `docs/content/` (86 written by hand, 20 in generated files) | A dimension with one value, a lifecycle state, and a feature, so `@available: next` can mean "in main, not yet released". After a release, `RELEASING.md` has a person "change each `next` mark under `docs/content/` to `ascribe <version>`". | A list of which versions exist. | [Versions](../proposals/versions.md), level 2: "Ascribe's docs declare a lifecycle state, `unreleased`, and a feature, `next`, that points at it. With a list, the state is unnecessary". Finding 30. |
| The latest release, written twice | `docs/ascribe.toml`: `[versions] current = "0.2.0"` and `[phrases] version = "0.2.0"` | 2 lines | `RELEASING.md`: "set `version` in `docs/ascribe.toml`'s `[phrases]` and `[versions] current` to the release". | A phrase that is the current version. | [Versions](../proposals/versions.md): "A built-in phrase for each versioned target, resolving to the line of the build it's in, gives that fact one home." Finding 30. |
| A build step that follows the canary | `site/scripts/follow-next.mjs` | 32 | After `npm ci`, installs the three Ascribe packages at the version the `next` tag names, without saving. `site/README.md`: "This is the one place the site's build differs from a user's." | Nothing: this is a site documenting `main`, which only Ascribe's own site does. | None. Not a finding; it's the cause of finding 7. |
| The site's address, in two files that must agree | `docs/ascribe.toml` `[consumer] site`; `site/astro.config.mjs` `site`; `.github/workflows/site.yml` | 3 lines, and a step in the workflow | The integration "fails the build if they disagree", so a pull request's preview rewrites both: `sed -i "s\|\"$site\"\|\"$address\"\|" docs/ascribe.toml site/astro.config.mjs`. | One home for the address. | None. Finding 32. |
| Facts from other files, kept as phrases by hand | `docs/ascribe.toml` `[phrases]`: `version`, `node`, `glibc`, `vscode`; `the_docs_phrases_match_their_sources` in `tests/conformance/tests/docs.rs`; `scripts/docs-site/facts.test.ts` | 4 of the 5 phrases; about 60 lines of one test, and about half of the other's 199 | The model's comments name each one's home: "`engines.node` in package.json", "`GLIBC` in .github/workflows/release-build.yml", "`engines.vscode` in packages/vscode/package.json". The first test fails with "the phrase `{key}` is … in docs/ascribe.toml, but … in {file}"; the second holds `node` and `glibc` to every other file that names them. | A phrase whose value is read from a file. | None for `node`, `glibc`, and `vscode`. [Versions](../proposals/versions.md) would give `version` its home. Finding 33. |
| The folders code examples come from, listed four times | `docs/ascribe.toml` `[sources.code] include`; `site/netlify.toml`; `.github/workflows/site-npm.yml`; `.github/workflows/ci.yml` | 3 globs, repeated; the other half of `scripts/docs-site/facts.test.ts` | The model's comment: "The same folders are listed in three other places, which need the same change when `include` changes". | A command that lists the files a project's pages read, for a workflow's `paths`. | None. Finding 33. |
| The schema, imported from Astro's folder | `site/src/content.config.ts` | 1 line | Imports `../.astro/integrations/_ascribed_astro/schema.ts`, a copy the integration makes "inside this folder, where TypeScript finds `astro/zod`", because the docs' project is outside the Astro root. | Nothing more. The integration already does the copying. | None needed. |
| Ordinary words that are also terms | `docs/ascribe.toml`: `match = "marked"` on `build`, `variant`, and `phrase` | 3 lines, each with the comment "Also an ordinary word: link the occurrences that mean the term." | Turns automatic linking off for three of the eight glossary terms. | Decided already: [decision 28](../decisions.md) says there's "no syntax to skip one occurrence of an automatic term; it waits for a real case". | None. The docs are a case: three of its eight terms. |
| A check turned off for whoever lacks Vale | `docs/ascribe.toml`: `[checks] prose-not-checked = "off"` | 1 line | "Vale isn't needed to work on the docs, so a machine without it isn't told so." | Nothing: the setting is the answer. | None needed. |

Three more things the site carries that aren't in the three folders, for completeness: `site/netlify.toml` (20 lines, the host's build), `site/astro.config.mjs` (17 lines, the integration and the code themes), and `site/README.md`, whose list of the files says `src/components/` holds "search, the sidebar, the table of contents, the page's availability badge". The folder has three files; the badge is `@ascribed/astro/Availability.astro` now.

### What the first customer asked for

Counting the rows above that are a gap in the product, and leaving out the site's own business and the two rows that need nothing:

- **Already planned, three:** unreleased work and the current version ([versions](../proposals/versions.md)), and the page's head ([search and answers](../proposals/search-and-answers.md)). Permalinks, the fourth thing the site needed, is built.
- **Half planned, one:** redirects for moved pages. A proposal would write them; none would notice that a page moved.
- **Left to the site on purpose, two:** search for readers, and answering a request for Markdown.
- **Asked for and planned nowhere, six:** navigation, a starter for a site's first files, a page's source path for an edit link, one home for the site's address, a phrase read from a file, and a check of a built site's own links.

Navigation is the largest by far: 126 lines of the site, a section of its README, a section of the brainstorm, and a way around it in another proposal.

### What the docs' own content model uses

`docs/ascribe.toml` is 179 lines. `ascribe model --config docs` gives two page types, two dimensions, five phrases, one feature, eight glossary terms, and one build. `ascribe report inventory --config docs` gives "19 pages. By type: reference 10, guide 9."

Used, counted in the 5,582 lines of the nineteen pages and the one hand-written fragment (generated files apart):

| Feature | How much |
|---|---|
| `@available` | 102 lines, on 13 files; and `available:` in the frontmatter of 6 pages |
| `@snippet` | 59, on 9 files, all from `[sources.code]`, this repository |
| `@include` | 55, on 6 files. All but two take a generated file; one hand-written fragment, `_fragments/field-type-grammar.md`, is shared by two pages |
| `@id` | 37 written by hand, 181 in generated files; 213 of the 218 are permalinks |
| `{repo}` | 327 uses. The other four phrases have five uses between them |
| `@variant` | 6 arms in 3 groups, on 2 pages, all `pm` |
| `@note` | 1 |
| `@steps` | 1 |
| An image | 1 |
| The glossary | 8 terms |
| `permalink:` in frontmatter | 6 pages |

Never used by the project's own docs:

- **`@details`.** No page has collapsible content.
- **`@intended`, and `[[intended]]`.** Nothing a check reports is acknowledged. The one `@intended` in the pages is an example in a code block.
- **Project widgets,** `[widgets.<name>]`.
- **A note type of the project's own,** `[notes.<type>]`.
- **A second build.** `[builds.site]` is the only one, with `variants = "switch"` and `availability = "badge"`. So no build of the docs selects a variant or filters by availability: the half of the language that leaves content out is never run on Ascribe's own pages, and no diagnostic that "names the builds it appears in" can differ between two.
- **A whole-page variant,** `variant:` in frontmatter.
- **A source in another repository.** `[sources.code]` is `path = ".."`, so the docs have no `ascribe.lock`, and `ascribe sources fetch` and `update` are never run on them.
- **`[fragments]`, `[images]`, `[checks.links]`, and `[permalinks]`** (no alias, nothing retired).
- **A field role other than `description`.** No owner and no review date, so the report's "Overdue reviews: none" is true of a project that has no reviews.
- **A dimension content varies along, beyond the package manager.** `[dimensions.release]` has one value, `ascribe`, and exists so an availability has a target.

One thing is declared and not used: `since = "string?"` in `[types.reference.frontmatter]`. No page has `since:`. Availability does that job now. Nothing reports it: the inventory's unused entries are "the fragments, phrases, features, glossary terms, and images nothing uses", and it answers "Entries nothing uses: none." Finding 34.

The two page types differ only by that field. `guide` and `reference` have the same `title` and `description`, so the type a page has changes nothing a reader or a check sees.
