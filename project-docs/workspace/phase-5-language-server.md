# Phase 5: The language server

Part of [Load a project once](README.md). Needs phase 4.

## Goal

The language server keeps a `Workspace`, with its open buffers laid over it, and applies each edit and each watched file's change through `Workspace::apply`. The hook's check server hands its one `Workspace` to `ascribe_lsp::Watched`, so it keeps the project once. ARCHITECTURE.md's "Loading a project, and reading files" describes one path.

## Context

- README decisions 3 and 6, and phase 4's `apply`.
- The language server's `Loaded` and `load_project` (`crates/ascribe-lsp/src/core.rs`): its `IncrementalProject` over `BufferFs`, the side files it keeps as buffers in `LayerFs`, the baseline it keeps (`crates/ascribe-lsp/src/baseline.rs`), and how a watched file's change becomes a `Change` (`classify`).
- `Watched` (`crates/ascribe-lsp/src/headless.rs`), which the hook's check server keeps beside the MCP cache.
- `crates/ascribe-cli/tests/all/lsp_parity.rs`, which holds the language server and `ascribe check` to the same diagnostics.

## Design

- **The buffers.** A `Workspace` loads over a `FileSystem` with the open buffers laid over the disk, as `BufferFs` and `LayerFs` do for the sources and the side files today; one layer does both. An edit to an open file is a change applied to the `Workspace`; closing a file whose buffer differed from the disk applies the disk's text.
- **`Loaded`** keeps a `Workspace` in place of its `IncrementalProject`, model, model text, and `LayerFs`. What's the language server's own stays: the dirty files, the resolved pages for the editor's build, the content checks' results, the baseline matches, the published diagnostics. A change to a side file reaches it through the `Workspace`'s `Affected`, and `Loaded::side_file` and `nav_file` go, with the `Workspace` finding the entry and its `Stale` saying what follows.
- **Loading again** happens when `apply` says so, as phase 4 defines, which is when it does today.
- **`Watched`** takes a `Workspace` it's given. The hook's check server makes one, hands it to `Watched` for the quick check, and checks the whole project on the same one.

## Tasks

1. The buffer layer over a `FileSystem`, for sources and side files, with tests.
2. `Loaded` over a `Workspace`; the watched files' changes and the editor's edits through `apply`.
3. `Watched` over a given `Workspace`, and the hook's check server keeping one.
4. ARCHITECTURE.md: "Loading a project, and reading files" describes one path: a `Workspace` loaded through a `FileSystem`, which the commands load once, and the MCP server, the hook's check server, and the language server keep and update. The Surfaces table's language server row.

## Out of scope

Anything about what the language server publishes or when.

## Acceptance criteria

- `crates/ascribe-lsp/src/core.rs` calls neither `IncrementalProject::load` nor `apply`; the language server goes through the `Workspace`.
- The hook's check server keeps one `Workspace` per project.
- The language server's tests, `lsp_parity.rs`, and the hook's tests pass unchanged.
- The perf benchmark's language server timings, typing included, are no slower.

## Verify

```sh
cargo test -p ascribe-lsp
cargo test -p ascribe-cli --test all lsp_parity
cargo test -p ascribe-cli --test all hook
cargo test --workspace --locked
cargo bench -p ascribe-corpora --bench perf
```

## Commits

1. "Keep the language server's project as a Workspace, with the open buffers over it"
2. "Keep one Workspace in the hook's check server"
