# Phase 4: The docs site

Part of [The surface audit](README.md). Requires phase 1. Reading and writing; no code.

## Goal

Two things, from the one site built with Ascribe so far:

- **What it had to build for itself.** Everything in `site/` and `docs/` that exists because Ascribe didn't provide it is evidence about the product, from the only customer there is.
- **How well the docs lead to the surface.** For each part of the surface, where a person would read about it, and whether they'd find it.

## Context

- `site/`: the Astro site. `site/README.md` describes how it's put together and why.
- `docs/`: the docs' project. `docs/ascribe.toml` is a real content model with real compromises in its comments.
- `site/src/nav.ts` and `site/test/nav.test.ts`: the sidebar, listed by hand, and the test that holds it to the pages.
- `site/src/components/`, `site/src/layouts/`, `site/scripts/`, and the other tests in `site/test/`.
- The brainstorm's section on navigation, which already records one gap the site found.
- The docs as published: <https://ascribed-dev.com>, and `docs/content/` behind it.
- Phase 2's inventory, if it's done, as the list of what the docs should lead to. Phase 1's tables otherwise.

## Design

### As a customer

Go through `site/` and `docs/` and list everything that's there to make up for something. For each: what it does, how much of it there is, and what Ascribe would need for it not to exist. Sorted three ways:

- **A site's own business,** which no docs tool should do for it: its layout, its styles.
- **Something any Ascribe site would need,** and each will build again: the sidebar's list and its test, search, the table of contents, the redirects file, the not-found page.
- **A workaround for something Ascribe has and doesn't quite fit:** a lifecycle state declared to mean "unreleased", a phrase for the version kept by hand with a test, a build step that follows the canary.

The second and third are findings. Several are already proposals (navigation, versions, permalinks); say which, so the report shows how much of what's planned the first customer had already asked for, and what it asked for that nothing plans.

Also from this side: what the docs' own content model uses and doesn't. A feature of the language the project's own docs never needed is worth a line.

### As the way in

For each section of the inventory, where the docs cover it:

- which page, and whether it's a guide, a reference, or both;
- how someone gets there: from the home page, from the sidebar, from search, from a command's `--help`, from a diagnostic, from the editor;
- whether the words on the page are the words on the surface.

Then the reverse: pages or sections that describe nothing in the inventory.

And the structure itself: what a guide is and what a reference is here, whether each page is one or the other, and where a reader looking for a task lands on a specification.

The maintainer's first journey (phase 3) is the test of the getting-started page. This phase covers the rest of the docs by reading.

## Tasks

1. The list of what the site built for itself, sorted, each with what would replace it.
2. The table of surface against docs, both ways.
3. Notes on the structure.
4. Each as a section of this folder's notes, with anything already certain written as a finding in phase 1's form.

## Out of scope

Restructuring the docs; the landing page and the rest of the [website proposal](../website.md); how the site looks.

## Acceptance criteria

- Every file under `site/src`, `site/scripts`, and `site/test` is accounted for in one of the three groups.
- Every section of the inventory has a line saying where the docs cover it, or that they don't.
- Each workaround names the proposal that would remove it, or says none does.

## Verify

```sh
pnpm test
```

## Commits

1. "List what the docs site built for itself"
2. "Map the surface to the docs that cover it"
