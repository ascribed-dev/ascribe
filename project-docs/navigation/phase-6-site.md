# Phase 6: Ascribe's own site

Part of [Navigation](README.md). Needs phases 4 and 5.

## Goal

Ascribe's site and every new site have no list of their own, and the docs have a guide.

`site/src/nav.ts` and `site/test/nav.test.ts` are gone. `docs/ascribe.nav` orders the docs' sections as the site's list did, and the published sidebar is the same as before, page for page. `npx @ascribed/astro init` writes a starter whose layout calls `sidebar()` with the collection, and no `src/nav.ts`. The docs have a how-to, "Put your pages in order".

## Context

- `site/src/nav.ts`: its groups, their labels, the order of pages in each, and "Contracts keep an order of their own". `site/test/nav.test.ts`, and the layout that calls `sidebar(nav, …)`.
- `docs/ascribe.toml`'s types, in the order the default sorts by.
- `packages/astro/src/init.ts`: `src/nav.ts` in the starter, `NAV_LIST`, and `navList`. The starter's layout. Its tests.
- `docs/content/astro/index.md`: setting up a site, and its sidebar.

## Design

### Ascribe's site

`docs/ascribe.nav` lists the sections in the site's order, with a label where the folder's own page's title isn't the group's label. A section's `pages` is written only where the site's order isn't the default: where a page's parts follow it, and in `contracts/`. The site's layout calls `sidebar()` with the collection. `checks/`'s `pages` names `diagnostics.md` where the site has it, which places its section there (phase 1, decision 7). The site's end-to-end test asserts the published sidebar against a copy of today's list, saved as a fixture before `nav.ts` is removed, so the comparison outlives the pull request.

`nav.test.ts`'s three failures are `ascribe check`'s now: a missing page is placed by default, a name that isn't a page is `nav-page-unknown`, and a page named twice is `nav-duplicate`.

### The starter

`init` stops writing `src/nav.ts`, and the starter's layout calls `sidebar()` with the collection and `neighbors()` for previous and next. A site that ran an older `init` keeps its `src/nav.ts` by passing it as `sidebar(entries, { nav })`, which phase 2's changelog entry described.

### The guide

A how-to in `docs/content/navigation/`: a section is a folder; the order you get with no file; writing `ascribe.nav`, or dragging in the editor; a short label; leaving a page out; and moving a site's own list over, by hand, for a site that has one.

## Tasks

1. The fixture of today's sidebar, then `docs/ascribe.nav`, the site's layout, and the removal of `nav.ts` and `nav.test.ts`, with the end-to-end test asserting against the fixture.
2. The starter, its layout, and `init`'s tests.
3. The guide, with `type: how-to`.
4. The docs: the Astro guide's sidebar section (no list; `sidebar()` and `neighbors()`), the guide linked from the navigation reference, `changes/astro/starter-without-nav.md`, and the surface map.

## Out of scope

- A command that converts a site's `src/nav.ts` (README, "Later").
- Regrouping Ascribe's docs further. Their folders are decision #452's.

## Acceptance criteria

- The site's sidebar matches the saved fixture, entry by entry, in its end-to-end test.
- `ascribe check --deny-warnings` passes on `docs` and `site/home`.
- A site made by `npx @ascribed/astro init` builds with a sidebar and has no `src/nav.ts`.

## Verify

```sh
cargo run -p ascribe-cli -- check --deny-warnings --config docs
pnpm --filter @ascribed/astro test
pnpm --filter ascribe-docs-site test
```

## Commits

1. "Order Ascribe's docs with ascribe.nav"
2. "Start a site without a list of its own"
3. "Add a guide to putting pages in order"
