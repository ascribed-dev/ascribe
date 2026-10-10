# Phase 3: Pages, and every link Ascribe makes

Part of [Permalinks](README.md). Requires phase 2. Rust, the generated docs, and the docs.

## Goal

- **A page can have a permalink,** not only a heading.
- **Every address the binary builds is an id.** After this, nothing in the binary knows where a docs page is, and the docs can move, which is what the [website](../website.md) waits for.

It widens the thread two ways: a second kind of thing to name, and from one of Ascribe's links to all of them. The second is also the first real load: about 150 permalinks, most of them on headings that reach their page through an include.

## Context

- Phase 2's attribute, resolver, list, and route.
- `crates/ascribe-model/src/sections.rs`: the reserved frontmatter keys, and `model-field-reserved` for a type that declares one. `crates/ascribe-emit/src/site/frontmatter.rs` and `crates/ascribe-emit/src/zod/`: how `available` reaches a site's layout and the generated schema.
- The binary's addresses, all built on `ascribe_core::docs_site!`:
  - `crates/ascribe-cli/src/cli.rs`: `docs_page!` on every command;
  - `crates/ascribe-check/src/registry.rs`: `REFERENCE` and `Entry::docs`, which `check`'s JSON, `explain`, and the language server's `codeDescription` all use;
  - `crates/ascribe-query/src/rules.rs` and `crates/ascribe-cli/src/agents/plugin.rs`.
- `tests/conformance/tests/docs.rs`: generates the diagnostics reference's fragments, headings included. `docs/content/_generated/diagnostics-retired.md`: codes that are no longer reported and still have entries.
- `docs/content/contracts/json-reports.md`: the `docs` field of a diagnostic in `check`'s JSON.

## Design

### A page's permalink

```yaml
---
title: Command reference
permalink: cli-reference
---
```

- `permalink` is a reserved frontmatter key for pages, as `available` is. A type that declares a field with that name is a model error, and a fragment can't use it.
- Its value follows the rule for a name from phase 1. It shares one set of names with headings: a page and a heading with the same permalink in one build is `permalink-duplicate`.
- In the list, a page's entry has a `route` and no `fragment`.
- It reaches a site's layout as `available` does, so a site can show a page's permanent address if it wants to.

### Ascribe's names

Every id the binary uses, by one convention, written down in the docs' contributing notes so the next one follows it:

| For | Name | On |
|---|---|---|
| A command | `cli-<command>`, such as `cli-sources-update` | Its heading in the command reference |
| The command reference | `cli-reference` | The page |
| A diagnostic | Its code, `ASC036` | Its heading in the diagnostics reference |
| The diagnostics and directive references, the agents guide | `diagnostics`, `directives`, `agents-guide` | Each page |

The diagnostics' headings are generated, so the generator writes the `@id` line under each. Their anchors change from `asc036-link-target-missing` to `ASC036`, and links to them inside the docs follow. Retired codes keep their entries and get permalinks too; phase 5 is where a name is retired.

### The binary

- One macro builds an address from the site, the path, and a name. `docs_page!` goes.
- `Entry::docs` returns `…/go/ASC036`. The `docs` field in `check`'s JSON keeps its meaning (where to read about this diagnostic) and gets a shorter value.
- One test lists every name the binary can produce (each command's, each registry code, and the few written by hand) and checks each against the list `ascribe build` writes for `docs/`. A new command or diagnostic without a permalink in the docs fails it.

## Tasks

1. The frontmatter key: the model, its validation, the generated schema, the site output's frontmatter, with conformance cases (a page; a fragment using it; a type declaring it; a page and a heading sharing a name).
2. The generator writes each diagnostic's `@id`. Bless the fragments, and fix the docs' links to the old anchors.
3. Mark every command's heading and the four pages.
4. The macro, `Entry::docs`, the rules, and the plugin's homepage. Bless what's generated from them (the plugin, the `--help` fragments in `docs/content/_generated/`).
5. The test over every name.
6. Measure `ascribe check` on `docs/` and on the 3,000-page corpus before and after, since collecting permalinks runs for every build. Say the numbers in the pull request.
7. `docs/content/reference/content-model.md` (the reserved key), `directives.md`, the JSON contract's note on `docs`, `CONTRIBUTING.md` (the naming convention), `CHANGELOG.md`.

## Out of scope

Moving the docs to `/docs/` (the website's work, which this unblocks); the committed list; retiring.

## Acceptance criteria

- `git grep docs_site` finds the macro's definition, the new address macro, and the test that holds it to `docs/ascribe.toml`, and no place that adds a page's path to it.
- Every command's `--help`, `ascribe explain ASC036`, and a diagnostic's link in the editor each lead to the right section on a built site.
- Changing `base-path` in a copy of `docs/` to `/docs/` and building changes no address the binary produces. A test does this.
- `ascribe check` on the corpus is no slower than the benchmark's tolerance.

## Verify

```sh
cargo test --workspace --locked
ASCRIBE_BLESS=1 cargo test -p ascribe-conformance --test docs && git diff --stat docs/content/_generated
target/debug/ascribe check --deny-warnings --config docs
pnpm --filter @ascribed/example-astro-site test:e2e
```

## Commits

1. "Let a page have a permalink"
2. "Give every command and diagnostic a permalink in the docs"
3. "Link to the docs by permalink from the binary"
