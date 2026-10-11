# Phase 2: The inventory, and what can be done where

Part of [The surface audit](README.md). Requires phase 1. A script, its test, and the file it writes; then one table written by hand from it.

## Goal

- **A map of the whole surface,** generated from what the code already declares, and kept current by a test. It's what phase 5 lays the questions against, and it stays after the audit: the next plan reads it to see what it's adding to.
- **The capability table,** `capabilities.md`: every thing a user wants to do, in a user's words, against every surface, with how it's done there or that it isn't. It's the audit's answer to question 3, and the first thing the maintainer asked for: the commands mapped to the use cases they serve. Built from the inventory, by hand, since it takes judgment.

## Context

- The proposal's table of surfaces: the fifteen rows are what the map has to cover.
- Where each surface is declared, for a script to read:
  - the commands: `crates/ascribe-cli/src/cli.rs`, and the option fragments `crates/ascribe-cli/src/docs.rs` already generates into `docs/content/_generated/`;
  - the diagnostics: `tests/conformance/diagnostics.toml`;
  - the directives: `ascribe_core::builtin_schemas`;
  - the content model's sections: `docs/content/reference/content-model.md`, or the loader in `crates/ascribe-model`;
  - the extension: `packages/vscode/package.json` (commands, settings, views, keybindings, menus, walkthrough, language model tools), and the action registry;
  - the server's own requests: the `METHOD` constants in `crates/ascribe-lsp/src/`;
  - the MCP server: `crates/ascribe-cli/src/mcp/tools.rs`, and its resources and prompts;
  - the plugin: `plugins/ascribe/`, itself generated;
  - the integration's options: `packages/astro/src/index.ts`;
  - the elements and their CSS properties: `packages/elements/`;
  - the JSON shapes: `crates/ascribe-cli/src/shapes.rs` and `schemas/`.
- `ARCHITECTURE.md`'s table of generated files and the tests that keep them (`ASCRIBE_BLESS=1`): the pattern to follow, and where this one is listed.
- Phase 1's two hand-built tables, which this replaces.

## Design

- **One file, in sections by surface,** each a table: the thing's name as a user types or sees it, what it's for in a few words (taken from its own description, not written here), who it's for, and where it's declared.
- **Generated, with a test** that fails when it's stale, like the other generated files. A new command, setting, tool, or diagnostic changes it, so a pull request that adds to the surface shows that in its diff.
- **Nothing typed that can be read.** Where a surface isn't declared anywhere a script can read (what `ascribe agents sync` writes, the files Ascribe puts in a project, the status bar item), the inventory lists it from a short hand-kept table in the script, and says so. Each of those is a small finding in itself: a surface with no single home.
- **Where it lives** follows the repository's habit for generated maps. Read `ARCHITECTURE.md` first; if nothing fits, beside `ARCHITECTURE.md`'s own tables is the natural place, linked from it.

### The capability table

`capabilities.md`: every thing a user can do, down the side; every surface, across. Each cell says how it's done there, or that it isn't.

The rows come from the use cases, not from the surface: the proposal's eight journeys, the tasks the guides under `docs/content/guides/` teach, and what the agents plan's pass asked agents to do, each in a user's words ("find where a phrase is used"), never the surface's ("refs"). The columns are the inventory's surfaces, grouped as the proposal's table groups them. An empty cell is marked as deliberate, with where that's said, or as a gap. A row with four full cells is looked at too: four ways to do one thing is a cost someone pays in learning them.

It's written from the generated inventory, so every cell can be checked against a row of it, and it isn't generated itself. Phase 3's sessions may show a cell wrong (an agent couldn't do what the table says it can); phase 5 settles those.

## Tasks

1. The script and its test, a surface at a time, starting with the ones that are already machine-readable.
2. The hand-kept table for what isn't, with a finding for each.
3. Replace phase 1's two tables with links into the inventory.
4. `ARCHITECTURE.md`: the file, and its line in the table of generated files.
5. The capability table, from the inventory and the use cases.

## Out of scope

Judging anything in the inventory. Deciding what to do about a gap or a duplicate the capability table shows: that's a finding, and phase 5's.

## Acceptance criteria

- Every row of the proposal's table has a section.
- Adding a command, a setting, or a diagnostic and not regenerating fails a test that says what to run.
- No section's content is typed where the code declares it.
- Every cell of the capability table is filled, or marked as a gap or as deliberate, and every row is in a user's words.

## Verify

```sh
cargo test --workspace --locked
pnpm format:check && pnpm lint && pnpm typecheck && pnpm test
```

## Commits

1. "Generate a map of Ascribe's surface"
2. "List the surfaces nothing declares"
3. "Lay out what can be done where"
