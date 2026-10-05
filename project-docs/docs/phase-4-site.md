# Phase 4: The site

Part of [Docs](README.md). Requires phases 1 and 3. Can run at the same time as phase 2. An Astro site.

## Goal

`site/` builds the user docs into a site a reader can find their way around: a layout, navigation, search, and the Ascribe elements. It's set up exactly as a user's site would be: its own folder, its own lockfile, Ascribe installed from npm.

## Context

- `docs/content/guides/astro.md` and `docs/content/getting-started.md`: the guides a user follows. **Follow them to make this site**, from an empty folder, and time it (the README's [measures](README.md#the-measures)).
- `examples/astro-site`: the example site, which uses workspace links; `site/` must not.
- `packages/astro/src/index.ts`: the integration's options. `project` names the directory holding `ascribe.toml`, relative to the Astro root. `packages/astro/src/binary.ts`: how the binary is found (`ASCRIBE_BIN`, then `@ascribed/cli`).
- `pnpm-workspace.yaml`: what's in the workspace.
- Brainstorm, "Navigation and site structure": Ascribe doesn't define a sidebar or page order; the site generator does.
- [Decisions 1, 4, and 6](README.md#decisions).

## Design

### Where it lives, and what it installs

`site/`, **outside the pnpm workspace**, with its own `package.json` and lockfile, using npm. It depends on `@ascribed/astro`, `@ascribed/cli`, and `@ascribed/elements` at `next`, from the registry, and on nothing in `packages/` by path. Its integration is `ascribe({ project: "../docs", build: "site" })`.

So a plain `npm ci && npm run build` in `site/`, with no Rust and no workspace install, builds the site, and it's what a user's host runs.

**Following the canary.** A lockfile pins the Ascribe packages to the canary that was current when it was written, so `npm ci` alone would never pick up a newer one. A user wants that: they pin a release and move on purpose. Our site is meant to follow `main`, so its production build takes one more step after `npm ci`: it installs the three Ascribe packages at `next` again, without saving. Everything else stays locked. This is the one place our build differs from a user's, and `site/`'s README says so. Check the exact npm commands against npm's current behavior for dist-tags and lockfiles, and that the three packages always resolve to the same canary; if they can't be made to, stop and report.

`docs/ascribe.toml` gains `[consumer]`: `profile = "astro"`, `site` (the Netlify address, which the owner supplies in phase 5; a placeholder until then), `base-path = "/"`, and `trailing-slash`.

### Building with a pull request's Ascribe

For local work and previews (decision 4), a script in `site/` builds with what's in the checkout: `ASCRIBE_BIN` for the binary, and the workspace's built packages installed over the registry's without saving. It's the only place the site touches `packages/`, and its header comment says it isn't how production builds.

### The site

Plain Astro, written here: no documentation theme. It's small, and writing it is the test of what `@ascribed/astro` gives a site author.

- **Layout:** a header (name, search, a link to the repository), a sidebar, the page, and a table of contents from the page's headings. Light and dark, following the system. Readable at phone width.
- **Navigation:** a hand-written list in `site/src/nav.ts`, grouping pages (Start, Guides, Reference). A test fails when a published page isn't in it, or when it names a page that doesn't exist. This is the workaround for navigation not being in the content model; write down what the content model would have needed.
- **Every Ascribe element in use:** variants as switchers (a reader's choice remembered across pages), availability badges (which is how unreleased features are marked), notes, steps, glossary links.
- **Search:** Pagefind, run on the built site, so it needs no service.
- **Each page:** its title, description, and an "Edit this page" link to the source file on GitHub.
- **A 404 page.**

### Checks

- An end-to-end test: the site builds; a page with a variant shows its switcher; an availability badge shows on a page marked unreleased; a link between pages resolves; search finds a known word.
- A check of the built site's internal links and anchors.
- In CI, on pull requests that touch `docs/`, `site/`, or the packages: the build with the pull request's Ascribe.

### What dogfooding finds

Every place the guides were wrong or silent, and everything `@ascribed/astro` or the elements made hard: issues, listed in the pull request. Fix the guides here.

## Tasks

1. `site/` made by following the guides, with the time and every stumble recorded.
2. The layout, navigation and its test, table of contents, and theme.
3. Search, the edit link, and the 404 page.
4. The script for building with a checkout's Ascribe, the end-to-end test, and the CI job.
5. The guides corrected; `CONTRIBUTING.md`: how to run the site locally.

## Out of scope

Deploying (phase 5); a starter template or a setup command for users (Later, informed by this phase's time); versions; analytics.

## Acceptance criteria

- In a copy of `docs/` and `site/` alone, with no Rust and nothing from the workspace, `npm ci && npm run build` builds every page.
- Every page is reachable from the navigation, and the test holds that.
- Variants, availability, notes, steps, and glossary links render as the element library intends, in light and dark, at 1280 px and 390 px.
- Search works on the built site with no network.
- The time to a working site, following the guides, is in `measures.md`.

## Verify

```sh
cd site && npm ci && npm run build && npm test
pnpm format:check && pnpm lint
```

## Commits

1. "Build the docs with Astro, from npm"
2. "Lay out the docs site"
3. "Search the docs site"
4. "Build the docs site with a checkout's Ascribe"
