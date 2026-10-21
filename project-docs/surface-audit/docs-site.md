# The docs site

Part of [The surface audit](README.md), [phase 4](phase-4-docs-site.md). The docs site is looked at twice (README decision 7): as the first site built with Ascribe, and as the way people find the surface. Section 1 is the first; sections 2 and 3 are the second. What's certain is written as findings 28 to 44 in [findings.md](findings.md#the-docs-site).

Everything here is from `main` at `a138d68b`, the published site at <https://ascribed-dev.com>, and a binary built from that code (`ascribe 0.2.0`), on 10 October 2026. Phase 2's generated inventory wasn't done, so "the surface" is the fifteen rows of the [proposal's table](https://github.com/ascribed-dev/ascribe/blob/460259f6529f9e7d5a8e06e600a9f28b43ae03f5/project-docs/proposals/surface-audit.md#what-the-surface-is), with [phase 1's tables](inventory-slice.md) and the sources they were built from.

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
| `site/src/layouts/Base.astro` | 104 | The shell: the head, the header, the menu button's script, and a slot. About twelve lines of the head (the canonical link, the Open Graph tags, the description) are what the [search and answers proposal](https://github.com/ascribed-dev/ascribe/blob/097cc3d496fa2f3b08a69958ccacc50c76849126/project-docs/proposals/search-and-answers.md) would write: "The docs site writes its own page head by hand in its layout (`site/src/layouts/Base.astro`)". |
| `site/src/assets/logo.svg` | 1 | The header's mark, generated. |
| `site/src/theme.ts` | 18 | Reads the page's background from `site.css` for `<meta name="theme-color">`. |
| `site/scripts/checkout.mjs` | 73 | Builds the site with the Ascribe in this checkout. Only a site in Ascribe's own repository needs it. |
| `site/test/e2e.test.ts` | 209 | Thirteen tests of the built site in Chromium. Eight are of what the site wrote (the head, the mark, the sidebar, search, the edit link, the two not-found cases, the phone layout). Five are of what Ascribe's packages render in a real page: "a variant group is a switcher, and the reader's choice holds on the next page", "a page marked with an availability shows its badge", "notes and steps render as elements", "a code example taken from a file shows its title", and a glossary link landing on its heading. |
| `site/test/built.ts` | 127 | The tests' helpers: serves `dist/`, lists the published pages from `docs/.ascribe/build/site/site`, and reads `[consumer] site` out of `docs/ascribe.toml` with a regular expression. |

### Something any Ascribe site would need

Each of these would be written again by the next site. The last column names the proposal in `project-docs/proposals/` that would remove it, or says none does.

| What | Files | Lines | What it does | What Ascribe would need | Proposal |
|---|---|---|---|---|---|
| The sidebar | `site/src/nav.ts`, `site/src/components/Sidebar.astro`, `site/test/nav.test.ts` | 55, 47, 24 | Four groups and nineteen pages, listed by source file. The test fails when "a published page is missing from this list, or when it names a page that doesn't exist". | What `site/README.md` lists under "Navigation": "an order and a group for each page", checking, "the result in the outputs", and "a label for a page in the navigation". | None. The [brainstorm](../brainstorm.md#navigation-and-site-structure) records it as a gap the site found. The [versions proposal](https://github.com/ascribed-dev/ascribe/blob/395fe80d/project-docs/proposals/versions.md) works around the same gap for its archive: "The content model has no navigation". Finding 28. |
| A route for each page | `site/src/pages/[...slug].astro`, `site/src/content.config.ts` | 20, 7 | The collection and the route, as the Astro guide's steps 4 and 5 give them. `examples/astro-site/src` has the same two files. | A starter that already has them. | None. Finding 29. |
| The page's frame | `site/src/layouts/Docs.astro` | 38 | The title, the availability badge (`@ascribed/astro/Availability.astro`), the description, the edit link, and the table of contents. | Nothing for the frame. The edit link is the next row. | None; the frame is the site's. |
| The edit link | `site/src/site.ts` | 19 | Finds a page's source file by taking apart the path the collection loaded it from: `/\.ascribe\/build\/[^/]+\/site\/(.+)$/`, which works because "the site output mirrors the content root's paths". | The page's source path in what the collection gives a layout. `_ascribe/permalinks.json` already has it for a permalink (`"source": "a.md"`). | None. Finding 31. |
| The table of contents | `site/src/components/Toc.astro` | 24 | Lists the `h2` and `h3` headings Astro's `render` returns. | Nothing: Astro gives the headings, and the Astro guide says so ("table of contents (`headings` from `render`)"). | None needed. Finding 29. |
| Search | `site/src/components/Search.astro`, and `pagefind --site dist` in `site/package.json` | 78 | A dialog that loads Pagefind's interface over an index of `dist/`. | Nothing planned. Two proposals leave it out on purpose: [search](https://github.com/ascribed-dev/ascribe/blob/62bf76a9/project-docs/proposals/search.md) ("Search for readers, and anything a site serves. That's Pagefind's job, or the site's choice.") and [search and answers](https://github.com/ascribed-dev/ascribe/blob/097cc3d496fa2f3b08a69958ccacc50c76849126/project-docs/proposals/search-and-answers.md) ("What the site generator owns: page speed, social card images, the order of a page's HTML, site search."). | None, deliberately. Finding 29. |
| The not-found page | `site/src/pages/404.astro` | 22 | A page with one paragraph, and `<MissingPermalink>` from `@ascribed/astro` with its default text replaced by nothing. | Nothing more: the part about permalinks is already Ascribe's. | [Permalinks](https://github.com/ascribed-dev/ascribe/blob/e80d764ef544f7fb617dec8084248a3e161a6d23/project-docs/proposals/permalinks.md), built. Finding 29. |
| Links in the built pages | `site/test/links.test.ts` | 71 | "Every link and anchor inside the built site lands: on a file dist/ serves, and on an element with that id." `ascribe check` checks the same links in the source; this checks them again after the layout, the sidebar, and Astro have had their turn. | A check of a built site's own links. `ascribe report links` asks about external links, and `ascribe report agents` reads a published site. | None. Finding 29. |
| Redirects for pages that moved | `site/test/redirects.test.ts`, and `site/public/_redirects` | 41 (and 5) | The file has five lines of comment and no rule yet. The test checks that each rule "leads from an address that isn't a page to one that is". `site/README.md`: "when one moves, add a line to `public/_redirects`." | A record of the addresses a project has published, so a moved page is noticed and its redirect written. | [Search and answers](https://github.com/ascribed-dev/ascribe/blob/097cc3d496fa2f3b08a69958ccacc50c76849126/project-docs/proposals/search-and-answers.md) plans "Redirects, through `@ascribed/astro`", and carves out the part that would notice a moved page: "everything that needs to know what was published before". [Permalinks](https://github.com/ascribed-dev/ascribe/blob/e80d764ef544f7fb617dec8084248a3e161a6d23/project-docs/proposals/permalinks.md) covers links from outside by name, not a page's own address. Finding 29. |
| A page as Markdown, when asked | `site/netlify/edge-functions/markdown.ts` (outside the three folders), `site/test/negotiation.test.ts` | 84, 98 | Answers `Accept: text/markdown` with the page's `.md`. The Astro guide tells a Netlify user to copy it: "On Netlify, an edge function does it: this site's is `markdown.ts`". | Nothing planned: [search and answers](https://github.com/ascribed-dev/ascribe/blob/097cc3d496fa2f3b08a69958ccacc50c76849126/project-docs/proposals/search-and-answers.md) says "What the host owns: status codes, headers, content negotiation, bot protection, logs." | None, deliberately. Finding 29. |

### A workaround for something Ascribe has and doesn't quite fit

| What | Where | How much | What it does | What Ascribe would need | Proposal |
|---|---|---|---|---|---|
| "Unreleased", declared by hand | `docs/ascribe.toml`: `[dimensions.release]`, `[versions]`, `[lifecycle.unreleased]`, `[features.next]`; `site/src/styles/site.css` lines 52 to 57 | About 25 lines of the model with their comments, 6 of the stylesheet, and 106 `next` marks in `docs/content/` (86 written by hand, 20 in generated files) | A dimension with one value, a lifecycle state, and a feature, so `@available: next` can mean "in main, not yet released". After a release, `RELEASING.md` has a person "change each `next` mark under `docs/content/` to `ascribe <version>`". | A list of which versions exist. | [Versions](https://github.com/ascribed-dev/ascribe/blob/395fe80d/project-docs/proposals/versions.md), level 2: "Ascribe's docs declare a lifecycle state, `unreleased`, and a feature, `next`, that points at it. With a list, the state is unnecessary". Finding 30. |
| The latest release, written twice | `docs/ascribe.toml`: `[versions] current = "0.2.0"` and `[phrases] version = "0.2.0"` | 2 lines | `RELEASING.md`: "set `version` in `docs/ascribe.toml`'s `[phrases]` and `[versions] current` to the release". | A phrase that is the current version. | [Versions](https://github.com/ascribed-dev/ascribe/blob/395fe80d/project-docs/proposals/versions.md): "A built-in phrase for each versioned target, resolving to the line of the build it's in, gives that fact one home." Finding 30. |
| A build step that follows the canary | `site/scripts/follow-next.mjs` | 32 | After `npm ci`, installs the three Ascribe packages at the version the `next` tag names, without saving. `site/README.md`: "This is the one place the site's build differs from a user's." | Nothing: this is a site documenting `main`, which only Ascribe's own site does. | None. Not a finding; it's the cause of finding 7. |
| The site's address, in two files that must agree | `docs/ascribe.toml` `[consumer] site`; `site/astro.config.mjs` `site`; `.github/workflows/site.yml` | 3 lines, and a step in the workflow | The integration "fails the build if they disagree", so a pull request's preview rewrites both: `sed -i "s\|\"$site\"\|\"$address\"\|" docs/ascribe.toml site/astro.config.mjs`. | One home for the address. | None. Finding 32. |
| Facts from other files, kept as phrases by hand | `docs/ascribe.toml` `[phrases]`: `version`, `node`, `glibc`, `vscode`; `the_docs_phrases_match_their_sources` in `tests/conformance/tests/docs.rs`; `scripts/docs-site/facts.test.ts` | 4 of the 5 phrases; about 60 lines of one test, and about half of the other's 199 | The model's comments name each one's home: "`engines.node` in package.json", "`GLIBC` in .github/workflows/release-build.yml", "`engines.vscode` in packages/vscode/package.json". The first test fails with "the phrase `{key}` is … in docs/ascribe.toml, but … in {file}"; the second holds `node` and `glibc` to every other file that names them. | A phrase whose value is read from a file. | None for `node`, `glibc`, and `vscode`. [Versions](https://github.com/ascribed-dev/ascribe/blob/395fe80d/project-docs/proposals/versions.md) would give `version` its home. Finding 33. |
| The folders code examples come from, listed four times | `docs/ascribe.toml` `[sources.code] include`; `site/netlify.toml`; `.github/workflows/site-npm.yml`; `.github/workflows/ci.yml` | 3 globs, repeated; the other half of `scripts/docs-site/facts.test.ts` | The model's comment: "The same folders are listed in three other places, which need the same change when `include` changes". | A command that lists the files a project's pages read, for a workflow's `paths`. | None. Finding 33. |
| The schema, imported from Astro's folder | `site/src/content.config.ts` | 1 line | Imports `../.astro/integrations/_ascribed_astro/schema.ts`, a copy the integration makes "inside this folder, where TypeScript finds `astro/zod`", because the docs' project is outside the Astro root. | Nothing more. The integration already does the copying. | None needed. |
| Ordinary words that are also terms | `docs/ascribe.toml`: `match = "marked"` on `build`, `variant`, and `phrase` | 3 lines, each with the comment "Also an ordinary word: link the occurrences that mean the term." | Turns automatic linking off for three of the eight glossary terms. | Decided already: [decision 28](../decisions.md) says there's "no syntax to skip one occurrence of an automatic term; it waits for a real case". | None. The docs are a case: three of its eight terms. |
| A check turned off for whoever lacks Vale | `docs/ascribe.toml`: `[checks] prose-not-checked = "off"` | 1 line | "Vale isn't needed to work on the docs, so a machine without it isn't told so." | Nothing: the setting is the answer. | None needed. |

Three more things the site carries that aren't in the three folders, for completeness: `site/netlify.toml` (20 lines, the host's build), `site/astro.config.mjs` (17 lines, the integration and the code themes), and `site/README.md`, whose list of the files says `src/components/` holds "search, the sidebar, the table of contents, the page's availability badge". The folder has three files; the badge is `@ascribed/astro/Availability.astro` now.

And in `docs/`, outside `content/`: `docs/ascribe.permalinks` (223 lines, written by `ascribe permalinks sync`), `docs/README.md`, and twelve files of three lines each at the docs' old paths, such as `docs/cli.md`: "# Moved", then "This page is now [content/reference/cli.md](content/reference/cli.md). Package READMEs published before the move link here, so this file stays until a release republishes them." They're redirects written as pages, for links that named a source file on GitHub. Finding 43.

### What the first customer asked for

Counting the rows above that are a gap in the product, and leaving out the site's own business and the two rows that need nothing:

- **Already planned, three:** unreleased work and the current version ([versions](https://github.com/ascribed-dev/ascribe/blob/395fe80d/project-docs/proposals/versions.md)), and the page's head ([search and answers](https://github.com/ascribed-dev/ascribe/blob/097cc3d496fa2f3b08a69958ccacc50c76849126/project-docs/proposals/search-and-answers.md)). Permalinks, the fourth thing the site needed, is built.
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

## 2. The surface against the docs, both ways

"Reference" and "guide" here are the sidebar's groups, not a judgment; section 3 is about how well the two words fit. "How someone gets there" was checked on the published site with `curl` for every address the product prints; search was only read (`site/src/components/Search.astro` and its test), not tried.

### From the surface to the docs

| Surface | Reference | Guide | How someone gets there | Are the page's words the surface's? |
|---|---|---|---|---|
| The command line: 18 commands, 27 with the subcommands of `agents`, `permalinks`, `sources`, and `vale` | `reference/cli.md`, "Command reference", a section for each, with its options included from 26 generated files (`_generated/cli-*-options.md`) | By command, in the next table | Every command's and subcommand's `--help` ends `Documentation: https://ascribed-dev.com/go/cli-<command>/`, and all 31 of those addresses answer 200. The home page, the sidebar, and `packages/cli/README.md` link to the page. | Yes: each heading is the command as typed (`` ## `ascribe check` ``), and a test holds the options to the binary. The order isn't the binary's: `--help` lists them by name, and the page goes `check`, `build`, `diff`, `drift`, `report`, `fmt`, `lsp`, `mcp`, `sources`, `permalinks`, `vale`, `explain`, `model`, `outline`, `link`, `refs`, `render`, `agents`. The home page and getting started describe the page by six and by four of its commands (finding 40). |
| The language: directives, attributes, phrases, frontmatter | `reference/directives.md`, "Directive reference"; `SPEC.md`, on GitHub | Getting started's "Write a page", five bullets. No other guide (finding 38). | The home page; the sidebar; the last line of the rules `ascribe agents sync` writes ("Every directive: https://ascribed-dev.com/go/directives/"); the skill. From a diagnostic, not: its entry links to `SPEC.md` (finding 37). | Yes: a heading for each directive, as written (`` ### `@variant` ``). |
| `ascribe.toml` | `reference/content-model.md`, "`ascribe.toml` reference", and `contracts/content-model.md`, "`ascribe.toml` format contract", which share much of their text (finding 41) | Getting started's "Create `ascribe.toml`" | The home page; the sidebar; the extension's walkthrough links `/go/create-ascribetoml/`. From a `model-` diagnostic, not: none of the 72 entries links to either page. | Yes: a numbered section for each table, as written (`` ## 16. `[builds.<name>]` ``). The address says `content-model`, and the title says `ascribe.toml`. |
| Diagnostics: 180, and one retired | `reference/diagnostics.md`, "Diagnostics", generated from the registry | None of its own. `ascribe explain` is the guide, and the agents guide names it. | `ascribe explain`'s last line, `docs` in `check`'s JSON, and the code's link in the Problems panel are all `https://ascribed-dev.com/go/ASC036/`, which takes two requests on this host (finding 35). `ascribe check`'s text prints the code and no address. A diagnostic's name isn't an address: `/go/link-target-missing/` answers 404. | Yes, by generation. The page is titled "Diagnostics" and described as "Every problem Ascribe reports", which is finding 9's two words on one page. |
| The VS Code extension | `guides/editor.md`, "Editing": its "Settings", "Commands", and "Actions" tables are generated from the manifest and the action registry | The same page, and `guides/review.md` | The manifest's `homepage` is `/go/editor-guide/`; the Projects view's empty state links `/go/getting-started/`, and the walkthrough `/go/create-ascribetoml/` and `/go/cli-check/`; getting started's "Install the editor"; the Marketplace README links the page as "Editing with Ascribe". | The tables, yes: 51 of the manifest's 63 commands are in them, and the other 12 are buttons the palette doesn't list, each named in the prose. The page's own title isn't: the sidebar says "Editing", the README "Editing with Ascribe", the permalink `editor-guide`, and the thing itself is "the **Ascribe** extension for VS Code". |
| The language server | `reference/cli.md`, `ascribe lsp` | `guides/editor.md`, "Other editors" | `ascribe lsp --help`; the editor guide. | Yes, for the command. Its nine requests of its own (`ascribe/preview`, `ascribe/context`, `ascribe/edit`, and the rest) aren't on the site; they're in `crates/ascribe-lsp/README.md`. |
| The MCP server | `reference/cli.md`, `ascribe mcp`: a table of the nine tools, its resources and prompts, written by hand | `guides/agents.md`, "The MCP server" | `ascribe mcp --help`; the agents guide. | Yes: the nine tool names match the server's. I found no test that compares the table with the server. |
| VS Code's agent tools | `guides/editor.md`, "Agents in VS Code", with a generated table | `guides/agents.md`, "VS Code" | The two guides. | Yes, by generation: `#ascribe_editor_problems`, `#ascribe_review_threads`, `#ascribe_review_changes`. |
| The plugin | None | `guides/agents.md`, "The plugin"; `plugins/ascribe/README.md` | The plugin's `homepage` is `/go/agents-guide/`. | Yes, and it repeats a difference in the surface faithfully: the commands are "`/ascribe:check`, `/ascribe:new-page`, and `/ascribe:review`", and the prompts they run are "`new-page`, `fix`, and `review`". |
| What `ascribe agents sync` writes | `reference/cli.md`, `ascribe agents sync` | `guides/agents.md`, "Agent instructions", "The skill", and the table under "Hooks" | Getting started's "Work with an AI agent"; `ascribe agents sync --help`. | Yes. |
| `@ascribed/astro` | `guides/astro.md`, "Options" and "What the integration does" | The same page, and getting started's "Publish it with Astro" | The home page; the sidebar; `packages/astro/README.md`. | Yes: the six options in the table are the six in `packages/astro/src/index.ts`. The table is written by hand. |
| `@ascribed/elements`, `@ascribed/review` | None on the site. `packages/elements/README.md`, `packages/elements/CONTRACT.md`, and `packages/review/README.md`, on GitHub | None | The Astro guide's "Styling" links the element library's README, and "Review in the site preview" the review package's. | Not applicable: there's no page (finding 42). |
| The outputs | `contracts/output-layout.md`, `contracts/site-render.md`, `contracts/assets.md`, `contracts/json-reports.md` with 18 generated schemas; `reference/cli.md`'s "Outputs", "JSON output", and "The HTML report" | `guides/review.md`, "The report in CI", for the HTML report | The home page's "For implementers"; the sidebar's "Contracts". | Yes. |
| Files Ascribe puts in a project | `contracts/permalinks-list.md` for `ascribe.permalinks`; `contracts/content-model.md` §19.1 for `ascribe.lock` | None of its own | Each from the guide of the feature that writes it: seven pages (finding 44). | Yes, where they're named. |
| The docs themselves | | | `--help`, the manifest, and the plugin link by permalink. The READMEs link by a page's address (finding 43). | |

Each command, and where a person would read how to use it rather than what it takes. The count is how many times the guide names the command.

| Command | A guide that covers it | Notes |
|---|---|---|
| `check` | Getting started, "Check it"; `guides/agents.md`, "The loop" (14) | Named in all seven guides |
| `build` | Getting started, "Build it"; `guides/astro.md` | |
| `diff` | `guides/review.md` (5) | The guide is titled "Review" |
| `drift` | `guides/drift.md` (7) | |
| `report` | `guides/report.md` (9) | |
| `fmt` | Getting started, "Format it"; `guides/editor.md`, "Formatting" | |
| `lsp` | `guides/editor.md`, "Other editors" | |
| `mcp` | `guides/agents.md`, "The MCP server" | |
| `sources` | `guides/drift.md`, "Docs kept apart from the code" (8) | The guide is titled "Drift" |
| `permalinks` | `guides/astro.md`, "Permalinks" (7) | The guide is titled "Astro" (finding 39) |
| `vale` | `guides/vale.md` | |
| `explain`, `model`, `outline`, `link`, `refs`, `render` | `guides/agents.md`, one row each of a table under "The loop" | The command reference says they're "for writers and for agents". The only guide that names them is for agents. |
| `agents` | `guides/agents.md` (18) | |

### From the docs to the surface

Pages and sections that describe nothing a user of Ascribe touches:

- **`docs/content/contracts/content-model.md`, §2 "Map of SPEC §7.2 to this file" and §24 "Decisions".** They're about how the file format was decided, for someone changing the loader. The page says so: "The loader in `ascribe-model` implements it."
- **`docs/content/contracts/site-render.md`, `output-layout.md`, and `assets.md`, most of each.** They describe what an emitter writes for someone writing another consumer. The home page puts them apart and says who they're for: "They matter if you're changing Ascribe's outputs or writing a consumer, and not if you're writing documentation with it." They take six of the sidebar's nineteen lines.
- **`docs/content/contracts/output-layout.md`, §5 "Why this shape".** A decision's reasons.
- **Twelve files in `docs/` outside `content/`,** each "# Moved" and one sentence, at the paths the docs had before they moved under `content/`. They aren't published. Their reason has passed (finding 43).

Nothing else was found. No page describes a command, a setting, or a directive that isn't there: the command reference's sections are the binary's eighteen commands, the editor's tables are generated from its manifest, and the diagnostics are generated from the registry.

Where a page describes more than is released, it says so. 106 marks say `next`, and six pages are `available: next` or `ascribe 0.2.0` as a whole. That's the surface of `main`, not of what `npm install` gives, and the site follows `main` on purpose.

## 3. Notes on the structure

**What a guide and a reference are, here.** The content model has two page types, `guide` ("the default") and `reference` (`files = ["reference/**", "contracts/**"]`), and they declare the same fields (finding 34), so the type records which folder a page is in and nothing more. The sidebar has four groups, "Start", "Guides", "Reference", and "Contracts", which `site/src/nav.ts` lists by hand. So a page is a guide or a reference by its folder, and nothing checks that it reads like one.

**Most pages are what their group says.** `guides/drift.md`, `guides/review.md`, `guides/report.md`, and `guides/vale.md` are tasks in order, with numbered steps. `reference/diagnostics.md` is a list. The pages that are both:

- **`guides/editor.md` is the editor's reference too.** It's 289 lines under "Guides", with "## Settings", "## Commands", and the actions table included from generated files, and "### Which `ascribe` it runs" as a numbered list of rules. There's no other page for the extension, so a person looking up a setting and a person installing the extension for the first time read the same page.
- **`guides/astro.md` is the integration's reference, and the permalinks guide.** "## Options" and "## What the integration does" are reference. "## Review in the site preview" says of itself: "[Review] walks through reviewing a pull request with it; this section is the reference for the site's side." And "## Permalinks", 100 of the page's 287 lines, is the guide to a feature of the language and a command group, for any site (finding 39).
- **`reference/cli.md` has tasks inside `ascribe check`.** Its subsections are "Checking some files", "Checking text before it's saved", "Checking the prose", "A quick check after each edit", and "Output for agents", before "Exit codes" and "Text output". The other commands' sections are what the command takes and writes.
- **`reference/content-model.md` is called a guide by the pages that link to it.** The home page: "The [reference] above is the guide for writing one." The contract: "For a guide to writing one, see the [`ascribe.toml` reference]". It's a numbered section for each table with every key, which is a reference, and there is no guide to writing a content model beyond getting started's one section, "Create `ascribe.toml`".

**Where a reader with a task lands on a specification.** Taking the eight tasks of the [proposal's journeys](https://github.com/ascribed-dev/ascribe/blob/460259f6529f9e7d5a8e06e600a9f28b43ae03f5/project-docs/proposals/surface-audit.md#2-journeys):

| Task | Where the docs take it |
|---|---|
| Start a project and publish a first page | `getting-started.md`, then `guides/astro.md`. A guide. Journey 1 in phase 3 tests it. |
| Add a section that differs by platform | `reference/directives.md#variant`, then `reference/content-model.md` §6 and §16. A reference, and a specification's numbered sections. Getting started has one bullet: "`@variant` marks alternatives by package manager. The site shows them as tabs, and a build can keep just one." (finding 38) |
| Fix a broken link | The diagnostic's entry, then `SPEC.md` on GitHub (finding 37). `ascribe link` is named in one guide, for agents. |
| Show an example from the code, and keep it current | `guides/drift.md`. A guide. |
| Rename a phrase everywhere | `guides/editor.md`, "Refactoring" and "Changing the content model". A guide, for the editor. For the command line, `reference/cli.md`'s `ascribe refs`. |
| Review a pull request's docs | `guides/review.md`. A guide. |
| Set up checks in CI | No one place. Getting started's "Check it in CI" has `ascribe check --deny-warnings` and `ascribe fmt --check`. The rest is one section in each of five guides: "Keeping them current in CI" (`agents sync --check`), "The report in CI" twice (`diff` in the review guide, `drift` in the drift guide), "On a schedule" (`report`), and "Checking a product's links" (`permalinks verify`). |
| Arrive in an unfamiliar project and make a correct edit | `guides/agents.md`. A guide. |

Five of the eight have a guide. The three that don't are the language's own subject (content that varies), the most common problem (a broken link), and CI, which is spread out rather than missing.

**The guides are about tools, and the language has none.** The sidebar's seven guides are "Astro", "Editing", "Review", "Drift", "Agents", "Prose, through Vale", and "The project report": a site generator, an editor, and five features around the language. Variants, availability, builds, phrases, the glossary, and fragments are reached only through the two references. A count of where the guides write `@variant`, `[builds.`, or `[dimensions.`: getting started 5, the Astro guide 2, the editor guide 2, and the other five guides none.

**A guide's title is the feature's name, not the command's.** Someone who knows a command and looks for it in the sidebar finds "Review" for `ascribe diff`, "Drift" for `ascribe sources`, "Astro" for `ascribe permalinks`, "Editing" for the extension. Each `--help` leads to the command reference, and the reference's section links on to the guide, so the path exists; the sidebar alone doesn't give it.

**Section numbers are in the addresses.** The two `ascribe.toml` pages number their sections, and a heading's id is its slug, so the docs link to `reference/content-model.md#16-buildsname` and `#6-dimensionsname`, and the glossary in `docs/ascribe.toml` does too. A table added anywhere but the end renumbers what follows and changes those ids. `[permalinks]` was added last, as §20. The same table is §16 in the reference and §17 in the contract, since the contract has one more section before it.

**Three lists of the pages, kept three ways.** The sidebar (`site/src/nav.ts`, by hand, with a test), `llms.txt` (written by Ascribe, by folder and file name), and the home page (`docs/content/index.md`, by hand, with no test). Findings 28 and 40.
