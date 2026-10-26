# Phase 3: The `Workspace`

Part of [Load a project once](README.md). Needs phase 2.

## Goal

Every command loads one `Workspace` per run and hands it to every library call. `ascribe_check::Project` is what the checks see of a `Workspace`'s snapshot, and loads, indexes, and holds nothing of its own.

## Context

- README decisions 3 to 6 and 8.
- `IncrementalProject::load` and `Snapshot` (`crates/ascribe-resolve/src/incremental/mod.rs`): a snapshot is an `ascribe_resolve::Project` tagged with versions, and equals a fresh `Project::load_with_ids` of the same files.
- How the language server builds an `ascribe_check::Project` from a snapshot today (`crates/ascribe-lsp/src/compute.rs`, with `Project::from_parts_with_fs`), which is close to the view this phase makes.
- `ascribe_sources::Workspace` and its callers in `crates/ascribe-cli/src/commands/sources.rs`.

## Design

- **The type.** `ascribe_workspace::Workspace` owns the `IncrementalProject`, the `FileSystem` it reads through, the model and its text, and the side files through #1007's registry. `Workspace::load` and `Workspace::load_with(fs)` replace phase 2's loader. `Workspace::snapshot` returns a `WorkspaceSnapshot`: the index snapshot, the model, the side files, and the on-demand files (code files, images, memo files), behind an `Arc`, `Send + Sync`. This phase has no `apply`; phase 4 adds it.
- **The view.** `ascribe_check::Project` holds a `WorkspaceSnapshot`, the day, and whether personal memos count. Its methods read from the snapshot: `sources` and `source_at` from the index snapshot's texts, `lock`, `permalinks`, `nav`, `addresses`, and `baseline` from the side files, `link_index` as the snapshot's index. `Project::index`, `held_index`, `with_link_index`, `from_parts`, and `from_parts_with_fs` go; `Held` and most of `page/bridge.rs` go with them, since the snapshot's ids are the project's.
- **Text laid over** (`Project::with_source`, for `check --stdin` and the language server's `ascribe/agentPrompt`) makes a snapshot with that one file's text replaced and its own index, as today.
- **A project built in memory** (tests, `ascribe_query::explain`'s examples, `release` and `retire_line`'s before and after) loads a `Workspace` over `MemoryFs`, through a constructor that takes a model and `(path, text)` pairs, as `Project::from_sources` does now.
- **The commands.** `context::load_project` and `answer::load` return a `Workspace`; `answer::Loaded` holds one, and its `index` and `page_index` read the snapshot's. Each command's entry takes the `Workspace`, or the view made from it, and nothing takes an `ascribe_resolve::Project` it built itself. `ascribe_diff::GitFs` loads the base revision as a `Workspace` too.
- **`ascribe_sources::Workspace`** is renamed (README decision 8) and takes the `Workspace`'s index.
- **Memory.** An `IncrementalProject` keeps more than an `ascribe_resolve::Project`: parses and reverse edges. The perf benchmark records each command's peak memory on the synthetic project (`tests/corpora/benches/memory/`); put `main`'s and the branch's in the pull request, and a large rise stops the phase.

## Tasks

1. `Workspace` and `WorkspaceSnapshot`, with tests that a loaded snapshot equals `ascribe_resolve::Project::load` of the same files, and that each source is read once.
2. The view: `ascribe_check::Project` over a snapshot, and the in-memory constructor. The checks' tests build projects with it.
3. The commands, `answer::Loaded`, the MCP tools' `answer` functions, and `ascribe-diff`'s base revision.
4. The language server's `compute.rs` builds the view from its own snapshot through the same constructor, without changing how it keeps its project (phase 5).
5. `ascribe_sources::Workspace`'s rename.
6. ARCHITECTURE.md: "From a source file to an output" (step 3), "Loading a project, and reading files", and "Each command's entry".

## Out of scope

Updating a `Workspace` (phase 4); the language server's own project (phase 5).

## Acceptance criteria

- Phase 1's counting test passes over `Workspace::load_with`, unchanged.
- No crate but `ascribe-workspace` calls `ascribe_resolve::Project::load` or `IncrementalProject::load`, except the language server until phase 5 and `ascribe-resolve`'s own tests.
- `crates/ascribe-cli/tests/all/lsp_parity.rs` passes.
- `node scripts/compare/outputs.ts --base main` shows nothing, and the perf benchmark is no slower.

## Verify

```sh
cargo test -p ascribe-workspace
cargo test --workspace --locked
node scripts/compare/outputs.ts --base main
cargo bench -p ascribe-corpora --bench perf
```

## Commits

1. "Add the Workspace: a project loaded once"
2. "Check a Workspace's snapshot instead of a project of the checks' own"
3. "Load one Workspace in every command"
4. "Rename the sources commands' view of a project"
