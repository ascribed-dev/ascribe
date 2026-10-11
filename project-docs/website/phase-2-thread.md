# Phase 2: The thread

Part of [The website](README.md). Requires phase 1. `@ascribed/astro`, the docs' project, a new project, and `site/`. This is the plan's steel thread: the thinnest path through every layer, shipped and kept.

The design below assumes phase 1 took its recommendations: one integration that takes a list of projects, and the marketing project in `site/home/`. If it didn't, this file was rewritten there.

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
- `.github/workflows/site.yml`, `site-npm.yml`, and `ci.yml`: the path patterns, and how a preview sets its address.
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
      { project: "home", build: "site" },
      { project: "../docs", build: "site" },
    ],
  }),
],
```

```ts
// site/src/content.config.ts
export const collections = {
  home: defineCollection(ascribeCollection({ schema: homeSchema, project: "home" })),
  docs: defineCollection(ascribeCollection({ schema: docsSchema, project: "../docs" })),
};
```

`ascribe({ build, project })` means what it does today, and `ascribeCollection({ schema })` with one project needs no name.

Rules for what the projects share:

- **Routing.** Each project's `trailing-slash` and `site` must agree with Astro's, as now. No project's base path may be another's, and each is `base` or under it. One project at the root with another beneath it is the case this plan needs, and it's allowed: a page of the first whose route falls under the second's base path is an error that names both.
- **Permalinks.** One route serves every project's list, and the projects must publish them at one path. A name two projects both declare is an error from the integration, since no build of either can see the other.
- **Development.** A change rebuilds the project it's in. Each project's folder gets its `.ascribe/dev.json`, with its own address.
- **Review** follows one project in a session: the one the open page belongs to. If that can't be done without redesigning the review server, review covers the first project listed, the guide says so, and the pull request reports it.
- **Schemas.** Each project's generated schema is copied to its own folder under the integration's codegen folder.

**This is the seam.** If any of these can't be made to work in both `astro build` and `astro dev`, stop and report before going on. [The thread](README.md#the-thread) says what each failure would change.

### The docs move

- `docs/ascribe.toml`: `base-path = "/docs/"`. `permalink-path` stays `/go/`.
- Every copy of a docs address moves with it, and the test that holds each copy is changed first so it fails until the copy does: the READMEs of the repository, the crates, the packages, the examples, and the plugin; the extension's manifest and walkthrough; `site/README.md`.
- `site/netlify/edge-functions/markdown.ts` answers for pages under `/docs/`, and `site/test/negotiation.test.ts` follows.
- `site/public/_redirects` gains nothing (README decision 8).
- `llms.txt` is at `/docs/llms.txt`. Check what `ascribe report agents` says about a site whose root has none, and record it in the pull request; don't work around it here.
- A pull request's preview is built with its own address in both projects.

### The marketing project

`site/home/`, with its own `ascribe.toml`:

- `[consumer]`: the docs' `site` and `trailing-slash`, `base-path = "/"`, `agents = false`, and the docs' `permalink-path`.
- One page type, for a marketing page, with a title and a description.
- One widget, a hero, which wraps a heading and a paragraph and takes a status.
- One page, `index.md`: what Ascribe is in a sentence, that it's early and open source, the two install commands, and a link to getting started.

The link into the docs is the permalink's address, `/go/getting-started/`. Read how Ascribe treats a link that starts with `/` in a page before relying on it: if `check` reports it, or the site output rewrites it, stop and report.

A test in `site/test/` checks every link from a marketing page into the docs: each `/go/<id>/` it uses is in the docs' `_ascribe/permalinks.json`.

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
2. Several projects: the option, the loader, the permalinks route, the watcher, the dev file, the schemas, and review, each with unit tests. End-to-end: two projects in one built site and under `astro dev`, a permalink from each, and the three refusals (a clashing base path, a page under another project's path, a name declared twice).
3. `docs/content/guides/astro.md`: a home page in front of the docs, and several projects in one site. `packages/astro/README.md`. `CHANGELOG.md`. The decision in `project-docs/decisions.md`.
4. Move the docs: the setting, then each test and the copies it holds.
5. `site/home/`: the model, the widget, and the page. The hero's token.
6. The site: the frame, the routes, the hero's styles, search, the edge function, and the site's tests, including the new one for links into the docs.
7. Workflows and Netlify: path patterns, the preview's address, and `ignore`, with `scripts/docs-site/facts.test.ts` agreeing.

Tasks 1 to 3 are a feature of the integration and can be their own pull request, merged first. The rest then waits a night for the canary, or is opened against the checkout's build as a preview is.

## Out of scope

The demonstration, the claims, and marks for what's unreleased (phase 3). The changelog (phase 4). Why Ascribe (phase 5). Screenshots and social cards for each page (phase 6). `robots.txt` and a sitemap. Redirects from the docs' old addresses.

## Acceptance criteria

- In `examples/astro-site`, built and served, a home page at `/` and a docs page under a path both open, a published file downloads, and a permalink lands. The end-to-end tests do exactly this, in a build and under `astro dev`.
- The pull request's preview shows the landing page at `/`, and its link to getting started opens `/docs/getting-started/`.
- `ascribe check --help` prints the address it printed before, and no file under `crates/` changed to make that so.
- A project that uses the integration as before, with one project and a base path equal to `base`, builds the same site. `node scripts/compare/outputs.ts --base main` shows no change outside what the docs' base path moves.
- `git grep "ascribed-dev.com/"` finds no address of a docs page that isn't under `/docs/` or `/go/`.
- The night after this merges, the first two commands in the goal give what's shown there on the live site. The pull request says when it was checked.

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
