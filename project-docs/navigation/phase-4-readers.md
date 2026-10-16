# Phase 4: Every reader

Part of [Navigation](README.md). Needs phase 3.

## Goal

`llms.txt`, the archive, the JSON output, `ascribe agents index`, and `ascribe model` follow the navigation, with each page's short label and its previous and next.

In `examples/quill`, with `[consumer] agents = true`, `llms.txt`'s first section is "Start", and its pages come in the navigation's order. `ascribe model --format json` has a `navigation` tree. A page with `nav-label: Keys` is "Keys" in the sidebar and its neighbors' links, and keeps its title in `llms.txt`. In `examples/astro-site`, a page's layout calls `neighbors()` and shows its previous and next. `ascribe versions archive` writes an archive whose sidebar and contents page follow the line's `ascribe.nav`.

## Context

- README decisions 3, 5, 11, and 12, and phase 1's 1, 2, 3, 5, 6, and 7.
- `crates/ascribe-resolve/src/llms.rs`: `TOP_SECTION`, the sections, the order inside one, and the split into `<folder>/llms.txt`.
- `crates/ascribe-query/src/agents_index.rs`: `ascribe agents index`'s sections.
- `crates/ascribe-emit/src/archive/mod.rs`: `Nav`, the archive's reading order, its sidebar, its contents page, and its previous and next.
- `crates/ascribe-emit/src/json.rs`: the JSON output's root.
- `crates/ascribe-query/src/model.rs`: `ascribe model`'s JSON.
- SPEC §2.1's reserved keys; `crates/ascribe-check/src/checks/frontmatter.rs` and `model-field-reserved`.
- Versioned builds: each line's root, and how `sidebar()` picks the line (`packages/astro/src/content.ts`).

## Design

- **`llms.txt`:** its sections are the navigation's top sections, in order, the root's "Start" first; nested sections' pages fold into their parent in reading order (phase 1, decision 7). The split keeps its folders. With no `ascribe.nav`, the default order, per phase 1's decision 3.
- **`ascribe agents index`** has the same sections, from the same code.
- **The archive:** its sidebar and contents page are the line's navigation, and previous and next follow reading order. The "way around" by path order goes.
- **The JSON output:** `_ascribe/nav.json`, as the site output has it.
- **`ascribe model`:** the `navigation` key, with each section's `listed`.
- **`nav-label`:** the reserved key, and `nav-label-long`. A page's `label` in `nav.json` is its `nav-label` when it has one, both ways, with its code spans in `formatted.label`, as a title's are.
- **`neighbors()`** in `@ascribed/astro`, from `nav.json` in reading order, in the line the entry is on.
- **Versioned lines:** each line's navigation is from that line's sources, so a page added in 3.5 isn't in 3.4's sidebar, and a section 3.4 orders differently keeps its order there.

## Tasks

1. `llms.txt` and `ascribe agents index` from the navigation, with "Start", and their tests.
2. The archive from the navigation.
3. `nav.json` in the JSON output, and `ascribe model`'s `navigation`, with its schema regenerated.
4. `nav-label`: SPEC §2.1, the reserved-key checks, `nav-label-long` with its case, and its use in `nav.json`.
5. `neighbors()`, with unit tests, and `examples/astro-site`'s layout using it.
6. A versioned project's lines, each with its own order, in a test.
7. `node scripts/compare/outputs.ts --base main`, with every `llms.txt` and archive difference accepted file by file under phase 1's decision 3, and the output of tested examples observed again.
8. The docs: the navigation reference (`nav-label`, and who reads the order); `llms.txt`'s section of the content model's reference and the agents page; the archive's page in `docs/content/versions/` and its contract; `ascribe model`'s reference and its generated JSON page; the Astro reference for `neighbors()`; `changes/agents/llms-navigation.md`, which says that `ascribe agents sync` rewrites the index in a project's `AGENTS.md` with the new order the next time it runs, so a product repository's diff isn't a surprise; `changes/command/model-navigation.md`; `changes/astro/neighbors.md`; and the surface map.

## Out of scope

- A sidebar with nesting drawn by Ascribe's own renderer. The preview draws one page.
- Ascribe's own site (phase 6).

## Acceptance criteria

- The goal's steps, in tests.
- Each `llms.txt` difference against `main` is the "Start" heading or a page's place in its section, and nothing else.
- The archive of a line and that line's site sidebar list the same pages in the same order.

## Verify

```sh
cargo test --workspace --locked
pnpm --filter @ascribed/astro test
pnpm --filter @ascribed/example-astro-site test
node scripts/compare/outputs.ts --base main
```

## Commits

1. "Give llms.txt and the agents' index the navigation's sections"
2. "Order an archive by its line's navigation"
3. "Report the navigation in ascribe model and the JSON output"
4. "Shorten a page's name in a menu with nav-label"
5. "Link a page to its previous and next"
