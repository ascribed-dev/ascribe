# Load a project once

A project is loaded once, as one `Workspace`, and every surface works on it: a command loads one per run and reads each source file once; the MCP server and the hook's check server keep one and update it in place; the language server keeps one with its open buffers over it. Loading a project moves out of `ascribe-check` with it. The decision is [decision #994](../decisions/994-one-workspace.md); this is the plan for building it. It covers [issue #994](https://github.com/ascribed-dev/ascribe/issues/994) and part 1 of [issue #1001](https://github.com/ascribed-dev/ascribe/issues/1001), and each phase is filed as an issue of its own.

## Who it's for

- **Anyone running a command on a large project,** which reads and indexes every source twice today.
- **An agent,** whose MCP server loads the whole project again after any file changes, and whose hook's check server keeps two copies of it.
- **Whoever adds to Ascribe next:** a feature that reads a project today chooses between three loaded forms, and a side file is added to each.

## What exists today

Checked against `main` on 26 October 2026, after #1007 (#1090):

- **Three types.** `ascribe_check::Project` (`crates/ascribe-check/src/project.rs`) holds the model and its text, the source texts, the side files, each read through the registry and asked for with `side_file::<F>()`, and what it reads on demand: code files, images, templates, memo files. `ascribe_resolve::Project` (`crates/ascribe-resolve/src/project.rs`) is the source index. `ascribe_resolve::IncrementalProject` (`crates/ascribe-resolve/src/incremental/mod.rs`) keeps an index current through `Change`s, and the language server's `Loaded` (`crates/ascribe-lsp/src/core.rs`) wraps it with the model, the side files it watches, and its caches.
- **Two reads.** `Project::load` reads every source through `DiskFs`. `Project::index` loads an `ascribe_resolve::Project` through the same `FileSystem`, which reads every source again; `build`, `diff`, `drift`, `sources`, `outline`, `link`, `refs`, `resolve`, `search`, `permalinks`, and `versions` call it. The page checks build a third over the held texts (`held_index`, through `Held` in `crates/ascribe-check/src/page/bridge.rs`), kept once per `Project` in `link_index`, unless given one (`with_link_index`, `PageIndex`). `Held` lists no nested projects and no unreadable files, so its index isn't quite `Project::index`'s.
- **Three ways to keep one warm.** The language server applies each edit and each watched file's change to its `IncrementalProject`. The MCP server's `Cache` (`crates/ascribe-cli/src/mcp/cache.rs`) lists the project's files' sizes and times before each call and loads it all again when the listing differs; `answer::Loaded` keeps the index and the checks' results beside it. The hook's check server (`crates/ascribe-cli/src/agents/checker.rs`) keeps both: an `ascribe_lsp::Watched` for the files an edit wrote, and the MCP server's cache for the whole project.
- **The side files.** The `SideFile` trait, its `Entry`, its `Stale`, and four entries (`Lock`, `Permalinks`, `Nav`, `Addresses`) are `ascribe-resolve`'s (`crates/ascribe-resolve/src/side_file.rs`). The list, `SIDE_FILES`, is `ascribe-check`'s (`crates/ascribe-check/src/side_files.rs`), since the fifth entry, `Baseline`, is the checks': its matching uses a finding's identity and the checks' diagnostics. A change to a side file reaches the source index as `Change::SideFiles`.
- **Loading's home.** `locate`, `find_config`, `find_below`, `load_model`, `read_sources`, `ModelFile`, `LoadError`, and `LocateError` are `ascribe-check`'s, so `ascribe-templates` (`init.rs`, `start.rs`) and `ascribe-query` depend on the checker to load a project.
- **Ids.** `ascribe_check::Project` numbers sources in path order from 1, as a fresh `ascribe_resolve::Project` does, and accepts the ids of an incremental snapshot, which are stable per path and have gaps. Nothing has to be renumbered to put one under the other.
- **The other `Workspace`.** `ascribe_sources::Workspace` (`crates/ascribe-sources/src/lib.rs`) is a project as `ascribe sources` sees it: its folder, sources, lock, and the paths snippets name.

## Decisions

These are settled. Don't reopen them in a phase; if one can't be met, stop and report.

1. **The side files come in as an argument.** #1007's trait and `Entry` are how the `Workspace` holds the side files, and the loader takes the list as `&'static [Entry]`: `ascribe-check` passes `SIDE_FILES`, so `ascribe-workspace` needs nothing of the baseline, and nothing moves out of the checks. A `Workspace` keeps each side file as an `Entry` reads it, and `ascribe_check::Project`'s `side_file::<F>()` reads from there.
2. **The crate is `ascribe-workspace`,** depending on core, syntax, model, and resolve. `ascribe-check` depends on it, and so do `ascribe-templates`, `ascribe-query`, `ascribe-diff`, `ascribe-sources`, `ascribe-lsp`, and `ascribe-cli` where they load a project. It writes no files.
3. **A `Workspace` owns an `IncrementalProject`,** and its state is a snapshot that never changes: the index snapshot, the model, and the side files, behind an `Arc`, so a check or a build reads one consistent version while the next is made. Changes go in as `ascribe_resolve::Change`s, extended where a side file needs it, and come out as `Affected`. Its snapshots are `Send + Sync`; the MCP server's single thread and the language server's worker take the same type.
4. **A command's `Workspace` is never updated,** only loaded, so it doesn't pay for updates it won't make. `IncrementalProject::load` does more per file than `ascribe_resolve::Project::load`: it hashes each source into the parse cache, computes its structure signature, and records its dependencies and include revisions. A `Workspace` loaded for a command skips that bookkeeping and builds it on its first `apply`, if one ever comes; the MCP server and the hook's check server pay for it on their first update. Loading then costs what loading an `ascribe_resolve::Project` costs today, and a benchmark that gets slower, in time or memory, stops the phase.
5. **What's read on demand is the `Workspace`'s,** with what a change invalidates: code files, images, memo files. The templates the content types name are the checks' and stay in `ascribe-check`, read through the `Workspace`'s `FileSystem`.
6. **`ascribe_check::Project` is a view,** not a copy: a snapshot of a `Workspace` and the checks' own settings, the day and whether personal memos count. It doesn't load, index, or hold texts of its own, and `Project::index`, `held_index`, and `with_link_index` go.
7. **The MCP cache's listing stays,** to find what changed, since the server has no watcher, but a difference becomes `Change`s for the files that differ, and only those are read. A change `IncrementalProject` refuses (`ApplyError::NestedProjectChanged`) loads the project again, as the language server does.
8. **`ascribe_sources::Workspace` is renamed** in phase 3, to a name that says it's the sources' view of a project, and takes the `Workspace`'s index rather than an `ascribe_resolve::Project` it's handed.
9. **No output changes in any phase.** `node scripts/compare/outputs.ts --base main` is empty, and each phase's pull request is labeled `optimization`.

## Phases

| Phase | Result |
|---|---|
| [1: Read once](phase-1-read-once.md) | Each command reads each source file once and indexes once, and a test counts the reads. |
| [2: Loading moves down](phase-2-loading.md) | `ascribe-workspace` finds and loads a project; `ascribe-templates` and `ascribe-query` don't depend on `ascribe-check` to load one. |
| [3: The `Workspace`](phase-3-workspace.md) | Every command loads one `Workspace` and hands it to every library call; `ascribe_check::Project` is a view over it. |
| [4: Kept warm, one way](phase-4-kept-warm.md) | A `Workspace` updates in place; the MCP server, and the hook's check server's whole-project check, apply what changed on disk instead of loading again. |
| [5: The language server](phase-5-language-server.md) | The language server keeps a `Workspace` with its open buffers over it, the hook's check server keeps one `Workspace` for both its checks, and ARCHITECTURE.md describes one path. |

The order is by what each needs. Phase 1 needs nothing new and gives the test that holds every later phase to one read. Phase 2 moves code without changing it, so phase 3's change to what a project is lands in its new home. Phase 4 needs phase 3, and adds the updates the language server then goes through in phase 5. No two phases run at once, since each builds on the one before.

## Rules for every phase

- Branch before committing; never commit to `main`.
- Read the current code before the phase file's pointers. If the phase file and the code disagree, or a decision above can't be met, stop and report instead of choosing silently.
- **No output changes** (decision 9): `node scripts/compare/outputs.ts --base main` shows none, and the pull request is labeled `optimization`.
- **Not slower.** Run the perf benchmark (`cargo bench -p ascribe-corpora --bench perf`) on `main` and on the branch, and put both in the pull request. A phase that's slower on the synthetic project or the Elastic sample stops and reports.
- **Read project files through `FileSystem`.** A phase that moves a read moves its `Outside FileSystem:` comment with it, and `crates/ascribe-resolve/tests/all/file_reads.rs` passes.
- **Libraries don't print, and don't panic on input.** A file that can't be read is an issue or an error with a code, as today.
- **The tests that exist pass unchanged,** above all `crates/ascribe-resolve/tests/all/incremental_differential.rs` and `crates/ascribe-cli/tests/all/lsp_parity.rs`. A test that moves with its code says so in the commit.
- **ARCHITECTURE.md changes with the code:** the crate table and dependency list when a crate's dependencies change, and "Loading a project, and reading files" and "Each command's entry" when what they say changes.
- No changelog entry: nothing a user sees changes. A phase that makes something a user would notice faster says so in `changes/command/` as one line ([changes/README.md](../../changes/README.md)).
- No phase history in code or docs. Describe what the code does now.
- Before finishing a phase, `pnpm check:all` passes.

## Later, not in this plan

- A `Workspace` that several surfaces share across processes, such as the MCP server reading the language server's.
- A file watcher for the MCP server in place of its listing.
- Parts 2 and 3 of #1001, the `ascribe-report` crate and the page-size check without `ascribe-emit`, which #1001 keeps.
