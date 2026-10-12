# Phase 7: Ascribe's own docs

Part of [Page templates](README.md). Requires phases 3 and 4. The docs, and the site's navigation.

## Goal

The docs use the types. They have two today, `guide` and `reference`. Taking on the six is the first test of the templates on pages that already exist, and it settles where the docs' guides end and their references begin.

## Context

- README decision 12, and the proposal: "Ascribe's own docs".
- `docs/ascribe.toml`: the two types, and their frontmatter.
- The surface audit's notes on the docs' structure, in [docs-site.md](../surface-audit/docs-site.md), section 3, and its findings about the docs: 38 (no guide to content that varies), 39 (permalinks inside the Astro guide), 40 and 54 (the home page and getting started), 41 (the reference and its contract in the same words). Several have issues. Read what's landed.
- `site/src/nav.ts` and `site/test/nav.test.ts`: the sidebar, by hand, with its groups.
- Each of the six guides, as the set has them, and Ascribe's notes on each.
- Every permalink in `ascribe.permalinks`: a page that moves keeps its permalink, and the binary's links follow.

## Design

### Sort before moving

One table, in the pull request, before any file changes: every page of the docs, the type it is today, the type it should be, and what would have to change for it to fit that type's template. Some pages are one type already. Some are two pages in one, usually a how-to with its reference attached. A few fit nothing, and that's a finding about the set, not about the page.

The maintainer reads the table before the pages move. It's a restructuring of the docs, and the table is where to disagree with it cheaply.

The table is [docs-types.md](docs-types.md).

### Take the set as a user would

`ascribe templates eject` in `docs/`, for the types the table needs, and no others.

It will be refused at once, and that's the first thing this phase shows. The docs already declare a type named `reference`, and `eject` refuses a type that's declared and writes nothing (README decision 16). So the order is:

1. Take the docs' own `reference` and `guide` types out of `docs/ascribe.toml`, noting what each has that the set's won't: `title` drawn as code (`inline = "code"`), and each field's `role`.
2. `eject` the types the table needs.
3. Put back what the docs need on the new types, by hand.

That's three steps and a broken project between the first and the last, for the only kind of project that will ever take the set late: one that has pages. Write down what a command would have to do to make it one step, such as taking a template for a type that exists and leaving its fields alone, and open the issue. It's the finding this phase is most likely to produce.

### Fit each page

A page takes its type, and its template's shape where that helps a reader. This isn't a rewrite to match a template heading for heading: their guides say what a kind of page is for, and a page that already does that is left alone. A page that's two kinds is split, and the links follow, checked.

Each page that moves keeps its permalink, and any that doesn't have one and is linked from outside gets one first.

### The sidebar

The types give the sidebar's groups a meaning they don't quite have now. Regroup by type where the table shows that's how the pages fall, in `nav.ts`, by hand as now: navigation isn't in the content model, and this phase doesn't put it there.

### What it showed

A short section in the page on templates, or in the brainstorm where it's a gap: which templates fit real pages, which of Ascribe's notes were wrong or missing, what the check reported on pages that predate their template (nothing, if decision 5 is right: a page that never had a placeholder has none left), and what the command needs for a project that isn't new.

## Tasks

1. The table, read by the maintainer.
2. The types, taken with the command, and what that showed.
3. The pages, a type at a time, one commit each: reference first, since it changes least.
4. The sidebar, and the site's tests.
5. What it showed, written down, with an issue for each thing to change in the set or the command.

## Out of scope

Navigation in the content model. Rewriting a page that already does its job. New pages the audit asked for, which have their own issues.

## Acceptance criteria

- Every page of the docs has one of the types, and `ascribe check docs` is clean, with nothing acknowledged to get there.
- No permalink changed its name, and `ascribe permalinks verify` passes for every name the binary uses.
- The site's link test passes, and the sidebar lists every page.
- The pull request has the table, and the section on what it showed quotes at least one thing that didn't fit.

## Verify

```sh
cargo run -p ascribe-cli -- check docs
cargo test --workspace --locked
cd site && npm run build:checkout && npm run typecheck && npm test
```

## Commits

One for each type's pages, then:

1. "Group the docs' sidebar by what each page is"
2. "Say what the docs taking the page types showed"
