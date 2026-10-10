# Phase 2: The inventory

Part of [The surface audit](README.md). Requires phase 1. A script, its test, and the file it writes.

## Goal

A map of the whole surface, generated from what the code already declares, and kept current by a test. It's what phase 5 lays the questions against, and it stays after the audit: the next plan reads it to see what it's adding to.

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

## Tasks

1. The script and its test, a surface at a time, starting with the ones that are already machine-readable.
2. The hand-kept table for what isn't, with a finding for each.
3. Replace phase 1's two tables with links into the inventory.
4. `ARCHITECTURE.md`: the file, and its line in the table of generated files.

## Out of scope

Judging anything in it; the capability table, which needs judgment and is phase 5's.

## Acceptance criteria

- Every row of the proposal's table has a section.
- Adding a command, a setting, or a diagnostic and not regenerating fails a test that says what to run.
- No section's content is typed where the code declares it.

## Verify

```sh
cargo test --workspace --locked
pnpm format:check && pnpm lint && pnpm typecheck && pnpm test
```

## Commits

1. "Generate a map of Ascribe's surface"
2. "List the surfaces nothing declares"
