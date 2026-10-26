# Phase 2: Loading moves down

Part of [Load a project once](README.md). Needs phase 1. This is part 1 of #1001.

## Goal

A new crate, `ascribe-workspace`, finds and loads a project. `ascribe-templates` and `ascribe-query` don't depend on `ascribe-check` to load one, and nothing a project loads changes.

## Context

- README "What exists today", loading's home, and README decision 2.
- #1001's revised "Done when", which leaves this item to the plan: `ascribe-templates` and `ascribe-query` don't depend on `ascribe-check` to load a project.
- #1007's registry, wherever it put the side files' parsers.

## Design

- **The crate:** `crates/ascribe-workspace`, depending on core, syntax, model, and resolve, with a README saying what it owns. It takes from `crates/ascribe-check/src/project.rs` what finds and reads a project, unchanged: `find_config`, `find_below`, `locate`, `load_model` and `load_model_bounded`, `read_sources`, the loader over any `FileSystem` from phase 1, `ModelFile`, `SourceFile`, `ReadFailure`, `LoadError`, and `LocateError`, with their `Outside FileSystem:` comments.
- **What the loader returns** is the parts `ascribe_check::Project` is built from: the model and its text, the sources and their listing, and the side files through #1007's registry. A side file whose parser #1007 left in `ascribe-check` moves down with the registry (`ascribe.baseline`'s, likely), or the phase stops and reports if it can't without the checks.
- **`ascribe_check::Project`** is built from those parts, with the methods it has today. Nothing re-exports the moved items from `ascribe-check`: each caller names `ascribe_workspace`.
- **Error codes stay.** `LoadError` and `LocateError` keep their `Coded` codes, and `crates/ascribe-cli/src/exit.rs`'s list doesn't change.

## Tasks

1. The crate, its `Cargo.toml` from `[workspace.dependencies]`, its README, and the moved code with its tests.
2. `ascribe-check` builds its `Project` from the loader's parts.
3. The callers: `ascribe-templates`' `init.rs` and `start.rs`, `ascribe-query`, `ascribe-diff`, `ascribe-sources`, `ascribe-lsp`, and `ascribe-cli`'s `context.rs`, `answer.rs`, and the MCP cache.
4. `ascribe-templates` drops `ascribe-check` from its `Cargo.toml` if nothing else needs it; `cargo machete` says whether anything does.
5. ARCHITECTURE.md: the crate table, the dependency list, and "Loading a project, and reading files".

## Out of scope

Changing what loading does or how often it runs; the `Workspace` type (phase 3).

## Acceptance criteria

- `ascribe-templates` and `ascribe-query` load a project through `ascribe-workspace`, and neither calls `ascribe_check::Project::load`, `load_model`, `locate`, or `find_config`.
- `ascribe-workspace` doesn't depend on `ascribe-check` or `ascribe-emit`, and writes no files.
- `node scripts/compare/outputs.ts --base main` shows nothing, and phase 1's test passes.

## Verify

```sh
cargo test -p ascribe-workspace
cargo machete
cargo deny check
cargo test --workspace --locked
node scripts/compare/outputs.ts --base main
```

## Commits

1. "Move finding and loading a project into ascribe-workspace"
