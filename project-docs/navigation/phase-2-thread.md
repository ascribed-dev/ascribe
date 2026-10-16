# Phase 2: The thread

Part of [Navigation](README.md). Needs phase 1.

## Goal

An Astro site's sidebar comes from the project: the default order with no file, and `ascribe.nav`'s with one.

In `examples/astro-site`, delete `src/nav.ts` and change the layout to `sidebar(await getCollection("docs"), { entry })`. Build: the sidebar is Start (`index.md`), Guides (My Setup), Reference (Options), the default order. Add `ascribe.nav`:

```toml
version = 1

[[sections]]
folder = ""

[[sections]]
folder = "Reference/"

[[sections]]
folder = "Guides/"
```

Build again: Start, Reference, Guides. The site has no list of its own.

## Context

- README decisions 4, 5, 6, 8, 9, 10, 11, 12, and 13, and phase 1's 1, 2, 7, and 10.
- `crates/ascribe-model/src/permalinks.rs` and `loader.rs`: how `ascribe.permalinks` is found, read through `FileSystem`, and loaded beside the model.
- `crates/ascribe-resolve`: the source index's pages and their `type`, the build's published pages, and `pages.rs`, which writes the page records.
- `crates/ascribe-emit/src/site/mod.rs`: what the site emitter writes under `_ascribe/`, and the manifest's `generated` entries.
- `packages/astro/src/content.ts` and `src/nav.ts`; `examples/astro-site/src/layouts/Docs.astro` and `src/nav.ts`; its end-to-end tests in `examples/astro-site/test/`.
- `site/src/nav.ts`'s header comment: the same default order, written by hand.

## Design

### The file and the model

`ascribe-model` loads `ascribe.nav` beside `ascribe.toml` when it's there, by decision 9's grammar: `version`, and `[[sections]]` with `folder`, `pages`, and `label`. A file that can't be read as that is `nav-invalid`, and the project is checked as if it had none. The other errors are phase 3's: until then, a `pages` entry or a `folder` that names nothing is passed over, and the reference says so.

### The navigation

`ascribe-resolve` makes the project's navigation once, then each build's share:

1. **The sections:** the content root and every folder under it with a page, at any depth, leaving out fragments and `_` names. Each section's own page: its `index.md`, or the page beside the folder with its name.
2. **The default order:** sections in path order, the root first; in each, its own page, then its pages by type in `ascribe.toml`'s order, then by title. A nested section whose own page is beside its folder sits where that page sits among them; one with only an `index.md` comes after them (phase 1, decision 7).
3. **The file's order over it:** listed sections in the file's order, then the rest in path order; in a section, its listed pages, then the rest by title. Listing a nested section's page beside its folder places the section there.
4. **A build's share:** the pages it publishes and offers (not `unlisted`), and the sections left with one.

Labels: a section's `label`, else its own page's title, else the folder's name with its first letter capitalized; the root's is "Start" (decision 5).

### The output

The site emitter writes `_ascribe/nav.json` for each build (phase 1, decision 1), `generated` in the manifest, with each page's `source`, `label` both ways, and `type`. A build with no pages gets an empty root.

### `sidebar()`

`sidebar(entries, options)` reads the build's `nav.json`, as `pageRecord` reads `pages.json`, and joins each `source` to its entry. A source with no entry is an error naming it, as a list's page with no entry is today. A site's own list is `sidebar(entries, { nav })`, which overrides the project's, checked as a list is today (phase 1, decision 2); `examples/astro-site`'s and Ascribe's site's calls move to it. `SidebarGroup` gains `sections`, and an entry's label is its formatted form where the output has one.

## Tasks

1. SPEC §7: a section for `ascribe.nav`, by decision 9, and §9.4: the navigation in the site output. A decision file for the format, if phase 1 didn't record it whole.
2. The loader, `nav-invalid` in the registry and the diagnostics' file, and a conformance case.
3. The navigation in `ascribe-resolve`, with tests: the default order, the file's order, a nested section with its page beside its folder placed by default and by `pages`, one with only an `index.md`, a section's own page both ways, an unlisted page, and a build that leaves a section empty.
4. `_ascribe/nav.json` in the site output, with tests, and the output layout contract.
5. `sidebar()` from the navigation, its `nav` option, with unit tests, and `examples/astro-site` moved, with its end-to-end test checking the order with and without the file. Ascribe's site's call moves its list into the option, with the sidebar unchanged.
6. The full matrix from the branch, for "Guides/My Setup.md" on Windows and macOS.
7. The docs: `docs/content/navigation/index.md`, the reference for `ascribe.nav` (what this phase reads, and that names that match nothing are passed over); `docs/content/contracts/navigation.md` and `output-layout.md`; `command/project-files.md`; the Astro reference for `sidebar()` without a list, and with one in `nav`; the content model reference's `listing` table, where `unlisted` now leaves a page out of the sidebar too, and the amended decision (phase 1, decision 10); `changes/language/navigation-file.md` and `changes/astro/sidebar-from-project.md`, which says how a site that passes a list changes its call; and the surface map.

## Out of scope

Everything in the README's "Left out" table.

## Acceptance criteria

- The goal's steps, in `examples/astro-site`'s end-to-end test.
- `node scripts/compare/outputs.ts --base main` shows each site output gaining `_ascribe/nav.json` and its manifest entry, accepted file by file, and nothing else.
- `ascribe check` on every project in the repository reports nothing new.

## Verify

```sh
cargo test --workspace --locked
pnpm --filter @ascribed/astro test
pnpm --filter @ascribed/example-astro-site test
node scripts/compare/outputs.ts --base main
```

## Commits

1. "Read a project's order from ascribe.nav"
2. "Write the navigation beside the page records"
3. "Draw a site's sidebar from the project"
