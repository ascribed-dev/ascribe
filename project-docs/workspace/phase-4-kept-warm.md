# Phase 4: Kept warm, one way

Part of [Load a project once](README.md). Needs phase 3.

## Goal

A `Workspace` takes changes and updates in place. The MCP server keeps one per project and applies what changed on disk since the last call, reading only the files that differ, instead of loading the project again. The hook's check server checks the whole project on the same `Workspace`.

## Context

- README decisions 3, 5, and 7.
- `IncrementalProject::apply`, `Change`, `Affected`, `ApplyError`, and its contract: after any sequence of changes, the snapshot equals a fresh load of the same files (`crates/ascribe-resolve/tests/all/incremental_differential.rs`).
- #1007's registry: what a change to each side file invalidates.
- The MCP cache (`crates/ascribe-cli/src/mcp/cache.rs`): its listing, which files it lists, and how it watches the code files snippets read.
- The hook's check server (`crates/ascribe-cli/src/agents/checker.rs`), which checks the whole project through the MCP cache.
- How the language server turns a watched file's change into `Change`s and side-file updates (`crates/ascribe-lsp/src/core.rs`: `classify`, `is_addresses`, `nav_file`, `in_memos`, `is_baseline`), which this phase's mapping is the same as.

## Design

- **`Workspace::apply`** takes `ascribe_resolve::Change`s and side-file changes, by path from the project root, and returns `Affected`, or an error saying the project must be loaded again: `ApplyError::NestedProjectChanged`, and a model change that moves the content root or the output directory. A side file goes through #1007's registry. A memo file, an image, or a code file a snippet reads drops what the snapshot read of it. A change that changes nothing makes no new snapshot.
- **The differential test** extends to the `Workspace`: random sequences of changes to sources, side files, memo files, code files, and `ascribe.toml`, and after each step the snapshot equals a fresh `Workspace::load` of the same files.
- **From a listing to changes.** A function in `ascribe-workspace` compares two listings of a project's files and gives the `Change`s between them: a path in one and not the other is created or deleted, a stamp that differs is edited, and each is read once. The listing is the MCP cache's today, moved down so the hook's check server and the MCP server share it. It reads no file it doesn't have to.
- **The MCP cache** keeps a `Workspace` and its listing per project. Before each call it lists again, and applies the difference; a refusal loads the project again. `answer::Loaded`'s kept results (the checks for each set of builds, the readings) belong to a snapshot's version, and a new version drops them.
- **The hook's check server's whole-project check** takes the MCP cache's `Workspace`, as now, and so updates instead of loading again.

## Tasks

1. `Workspace::apply`, with the side files, the on-demand files, and the model; unit tests for each kind.
2. The differential test over the `Workspace`.
3. The listing moved into `ascribe-workspace`, and the changes between two listings, with tests: a file created, deleted, edited, renamed (a deletion and a creation), a side file appearing, a code file changing outside the project's folder.
4. The MCP cache over it, and `answer::Loaded`'s kept results by version. The MCP tests (`crates/ascribe-cli/tests/all/mcp.rs`) pass, with a new one: a call after an edit to one page reads that page and no other, through phase 1's counting `FileSystem`.
5. The hook's check server, and its tests (`crates/ascribe-cli/tests/all/hook.rs`).
6. ARCHITECTURE.md's Surfaces table, MCP server row.

## Out of scope

The hook's quick check of the files an edit wrote, which goes through the language server's `Watched` until phase 5; a file watcher for the MCP server.

## Acceptance criteria

- The differential test passes over the `Workspace`.
- After an edit to one page, the MCP server's next call reads that page and no other source.
- `node scripts/compare/outputs.ts --base main` shows nothing, and the perf benchmark is no slower.

## Verify

```sh
cargo test -p ascribe-workspace
cargo test -p ascribe-cli --test all mcp
cargo test -p ascribe-cli --test all hook
cargo test --workspace --locked
cargo bench -p ascribe-corpora --bench perf
```

## Commits

1. "Update a Workspace in place from the changes to its files"
2. "Keep the MCP server's projects up to date instead of loading them again"
