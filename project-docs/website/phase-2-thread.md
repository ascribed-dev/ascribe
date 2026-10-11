# Phase 2: The thread

Part of [The website](README.md). Requires phase 1. `@ascribed/astro`, the docs' project, a new project, and `site/`. This is the plan's steel thread: the thinnest path through every layer, shipped and kept.

The design follows the README's decisions 11 to 20: one integration that takes a list of named projects, and the marketing project in `site/home/`.

## Goal

The site's root is a landing page written as an Ascribe page, and the docs are under `/docs/`:

```sh
$ curl -s https://ascribed-dev.com/ | grep -c "<hero-banner"
1
$ curl -sI https://ascribed-dev.com/docs/getting-started/ | head -1
HTTP/2 200
$ ascribe check --help | tail -1
Documentation: https://ascribed-dev.com/go/cli-check/
```

Following the last address still opens the section about `ascribe check`, now on a page under `/docs/`.

One landing page, one widget, one link into the docs. Everything it takes is built to the standard of the rest, and nothing in it is thrown away.

## Context

- [The thread](README.md#the-thread): the layers, and the seam this phase exists to test.
- Phase 1's decisions, in the [README](README.md#decisions).
- `packages/astro/src/project.ts`: `consumerMismatches`, which requires Astro's `base` to equal the base path, and `homeRoute`.
- `packages/astro/src/index.ts`: the hooks. What's single today: `siteRoot`, `agentsRoot`, the `virtual:ascribe/site` module, the route for permalinks, `rebuild`, `devProject`, the dev file, and the review server.
- `packages/astro/src/content.ts`: the loader, which reads the one root from the virtual module.
- `packages/astro/src/files.ts`: published files and agents' files, served and copied at Astro's `base`, not at the project's base path.
- `packages/astro/src/permalinks.ts`: `permalinkPattern`, and the list each project's build writes, with an empty table when it has none.
- `packages/astro/src/dev.ts` and `src/review/`: the watcher, `.ascribe/dev.json`, and review, each of which knows one project.
- `examples/astro-site/test/e2e/`: the end-to-end suite the Astro job runs against a real build.
- `site/`: `astro.config.mjs`, `src/content.config.ts`, `src/pages/[...slug].astro`, `src/layouts/`, `src/components/Search.astro`, `netlify.toml`, `netlify/edge-functions/markdown.ts`, `public/_redirects`, and `test/`.
- `docs/ascribe.toml`: `[consumer]`, and the comments that say which files repeat each setting.
- `scripts/docs-site/facts.test.ts`, and the tests that hold the site's address in the READMEs and the extension's manifest.
- `.github/workflows/site.yml`, `site-npm.yml`, and `ci.yml`: the path patterns, and how a preview sets its address. `.github/workflows/report.yml`: the nightly check of the published site.
- SPEC §6 and `docs/content/reference/content-model.md` §14: declaring a widget.
- `design/README.md`: adding a token and regenerating the stylesheets.

## Design

### A base path under Astro's `base`

The integration accepts a `[consumer] base-path` that is Astro's `base` or a path under it. A base path outside `base` is still refused, with a message that names both.

What follows from it, each to check against the code:

- **Published files and agents' files** are served and copied at the project's base path, where the site output's links point. Today both use Astro's `base`, which was the same path.
- **A page's route** in the site is the project's base path, less Astro's `base`, plus its entry id. The guide's example route shows it.
- **`homeRoute`** stays the site's root. A project's own root is its base path.

### Several projects

```js
// site/astro.config.mjs
integrations: [
  ascribe({
    projects: [
      { name: "docs", project: "../docs", build: "site" },
      { name: "home", project: "home", build: "site" },
    ],
    review: "docs",
  }),
],
```

```ts
// site/src/content.config.ts
export const collections = {
  docs: defineCollection(ascribeCollection({ schema: docsSchema, name: "docs" })),
  home: defineCollection(ascribeCollection({ schema: homeSchema, name: "home" })),
};
```

`ascribe({ build, project })` means what it does today, and `ascribeCollection({ schema })` with one project needs no name.

**Each entry has a name,** and the name is what everything else says: a collection, `review`, a codegen folder, an error message. A path can't be the key. Two entries may share a project and differ by build, which is what the [versions proposal](../proposals/versions.md) needs for a collection per release line (`{ name: "v3-4", project: "../docs", build: "3.4" }`), and `ascribeCollection({ project: "../docs" })` couldn't say which. A name is unique in the list, and has the characters a collection's name may have. This is a published option from the day it ships, so the shape isn't one the next plan has to change.

Rules for what the projects share:

- **Routing.** Each project's `trailing-slash` and `site` must agree with Astro's, as now. No project's base path may be another's, and each is `base` or under it. One project at the root with another beneath it is the case this plan needs, and it's allowed: a page of the first whose route falls under the second's base path is an error that names both.
- **Permalinks.** One route serves every project's list, and the projects must publish them at one path. A name two projects both declare is an error from the integration, since no build of either can see the other.
- **Development.** A change rebuilds the project it's in. Each project's folder gets its `.ascribe/dev.json`, with its own address.
- **Review** follows one project in a session: the one the open page belongs to. If that can't be done without redesigning the review server, review covers one project, the entry `review` names (`review: "docs"`), and the first entry when `review` is `true`. The site names the docs, which is the project that gets reviewed. The guide says so, and the pull request reports it.
- **Schemas.** Each entry's generated schema is copied to a folder named for it under the integration's codegen folder.

**This is the seam.** If any of these can't be made to work in both `astro build` and `astro dev`, stop and report before going on. [The thread](README.md#the-thread) says what each failure would change.

### The docs move

- `docs/ascribe.toml`: `base-path = "/docs/"`. `permalink-path` stays `/go/`.
- Every copy of a docs page's address moves with it, and the test that holds each copy is changed first so it fails until the copy does: the READMEs of the repository, the crates, the packages, the examples, and the plugin; about ten addresses in the crates' READMEs and source comments; the extension's manifest and walkthrough; `site/README.md`. Where a page has a permalink, the copy becomes the permalink's address instead, which doesn't move again (the surface audit's finding 43).
- `site/netlify/edge-functions/markdown.ts` probably doesn't change: `markdownPath` takes any page's address and adds `.md`, and a page with no Markdown file, such as a marketing page, goes on as it is. `site/test/negotiation.test.ts` gains a docs page under `/docs/` and a marketing page that falls through. Confirm by reading both before touching either.
- `site/public/_redirects` gains nothing (README decision 8).
- `llms.txt` is at `/docs/llms.txt`, and the site's root has none (README decision 19).
- **The nightly Report workflow** (`.github/workflows/report.yml`) runs `ascribe report links agents --config docs --site https://ascribed-dev.com --exit-code`, and opens an issue when it fails. Its `--site` becomes the docs' address, `https://ascribed-dev.com/docs/`, in this pull request. Before merging, run `ascribe report agents` against the pull request's preview at `/docs/`: if the checker only looks at a host's root for `llms.txt`, stop and report, since the job would go red the night after the merge and the choice is then between a root `llms.txt` and a checker that takes a path.
- A pull request's preview is built with its own address in both projects.

### The marketing project

`site/home/`, with its own `ascribe.toml`:

- `[consumer]`: the docs' `site` and `trailing-slash`, `base-path = "/"`, `agents = false`, and the docs' `permalink-path`.
- **The docs' release tables,** which phase 3's marks need (README decision 13): the release dimension, the `next` feature, `[versions] current`, and the `version` phrase. And `[sources.code]`, for the examples phase 3 reads from the repository. Each is a second home for a fact in `docs/ascribe.toml`, so `scripts/docs-site/facts.test.ts` gains a test that the two files agree on every one, written before the copies. The site's address is in a third file now, and the same test holds it (the surface audit's finding 32).
- One page type, for a marketing page, with a title and a description.
- One widget, a hero, which wraps a heading and a paragraph and takes a status.
- One page, `index.md`: what Ascribe is in a sentence, that it's early and open source, the two install commands, and a link to getting started.

**A page has one `h1`.** The hero wraps the page's heading, so the marketing frame prints no title of its own. The frontmatter's title is for the `<title>` and the social card.

The link into the docs is the permalink's address, `/go/getting-started/`. Read how Ascribe treats a link that starts with `/` in a page before relying on it: if `check` reports it, or the site output rewrites it, stop and report.

**Links into the docs are checked twice, by what's there already.** `site/test/links.test.ts` follows every `href` in the built site, and a permalink's address is a built file, so a name that doesn't exist fails it today. What it can't see is a name that was retired, which still gets a page. So the site's tests also run `ascribe permalinks verify` with the names the marketing pages link to, against the docs' `ascribe.permalinks`: it exits 1 for a retired name and gives the reason. That's the site using its own tool, as the proposal had it. No third test is written.

### The site

- **A frame for marketing pages:** the header, a wide measure, and no sidebar or table of contents. The header links to Docs and GitHub. Later phases add their pages to it.
- **A route for each collection:** the marketing pages at their ids, the docs under `docs/`.
- **The hero is drawn with a stylesheet.** The site output writes it as a custom element, and an element nothing defines still takes styles, so it needs no script.
- **A type size for the hero,** added to `design/tokens.toml` and generated.
- **Search indexes the docs only,** and its box stays on docs pages.
- **`src/nav.ts`** lists the docs' pages as before. `test/nav.test.ts` is about the docs' collection.

## What this phase doesn't handle, said where a user will look

- `docs/content/guides/astro.md` says, while it's true, what review does with several projects.
- The landing page says nothing it can't show yet. It has no demonstration and no list of claims until phase 3.

## Tasks

1. Accept a base path under `base`: the check, the files, and the agents' files, with unit tests, and end-to-end tests in `examples/astro-site` for a home page at `/` with docs under a path.
2. Several projects: the option, the loader, the permalinks route, the watcher, the dev file, the schemas, and review, each with unit tests. End-to-end: two projects in one built site and under `astro dev`, a permalink from each, two entries for one project with two builds, and the four refusals (a clashing base path, a page under another project's path, a permalink declared twice, an entry's name used twice).
3. `docs/content/guides/astro.md`: a home page in front of the docs, and several projects in one site. `packages/astro/README.md`. `CHANGELOG.md`. The decision in `project-docs/decisions.md`.
4. Move the docs: the setting, then each test and the copies it holds.
5. `site/home/`: the model with the docs' release tables and sources, the facts test that holds them equal, the widget, and the page. The hero's token.
6. The site: the frame, the routes, the hero's styles, search, and the site's tests: the negotiation test's new cases, and `ascribe permalinks verify` on the marketing pages' names.
7. Workflows and Netlify: path patterns, the preview's address, `ignore`, and the Report workflow's `--site`, with `scripts/docs-site/facts.test.ts` agreeing.

Tasks 1 to 3 are a feature of the integration and can be their own pull request, merged first. The rest then waits a night for the canary, or is opened against the checkout's build as a preview is.

## Out of scope

The demonstration, the claims, and marks for what's unreleased (phase 3). The changelog (phase 4). Why Ascribe (phase 5). Screenshots and social cards for each page (phase 6). `robots.txt` and a sitemap. Redirects from the docs' old addresses.

## Acceptance criteria

- In `examples/astro-site`, built and served, a home page at `/` and a docs page under a path both open, a published file downloads, and a permalink lands. The end-to-end tests do exactly this, in a build and under `astro dev`.
- The pull request's preview shows the landing page at `/`, and its link to getting started opens `/docs/getting-started/`.
- `ascribe check --help` prints the address it printed before, and so does every other command's help and every diagnostic's link. Nothing the binary compiles changed to make that so: the changes under `crates/` are to READMEs and comments, which hold page addresses and move with the docs.
- A project that uses the integration as before, with one project and a base path equal to `base`, builds the same site. `node scripts/compare/outputs.ts --base main` shows no change outside what the docs' base path moves.
- `git grep "ascribed-dev.com/"` finds no address of a docs page that isn't under `/docs/` or `/go/`.
- `ascribe report agents`, run against the preview's `/docs/`, passes what it passed against the root before.
- The night after this merges, the first two commands in the goal give what's shown there on the live site, and the Report workflow's run is no redder than the night before. The pull request says when each was checked.

## Verify

```sh
pnpm --filter @ascribed/astro test
pnpm --filter @ascribed/example-astro-site test:e2e
cargo test --workspace --locked
node scripts/compare/outputs.ts --base main
cd site && npm ci && npm run build:checkout && npm run typecheck && npm test
```

## Commits

1. "Accept a base path under Astro's base"
2. "Build one Astro site from several projects"
3. "Move the docs to /docs/"
4. "Give the site a landing page written in Ascribe"
