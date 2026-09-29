# Phase 15: Language server core

**Track:** Editor · **Start after:** 10, 13 · **Finish after:** 14 · **Parallel with:** 20 · **Unblocks:** 16, 17 (to finish), 25, 26

## Goal

Ship `tessera lsp`: a language server that keeps a Tessera project in memory, follows every change to it, and publishes the same diagnostics `tessera check` reports, as the author types.

## Read first

- [SPEC.md](../../SPEC.md): §8 and §10.
- [PLAN.md](../PLAN.md): Editor integration, VS Code extension.
- Handoff notes from phases 10, 13, and 14.

## Deliverables

- `crates/tessera-lsp`: the server.
- `crates/tessera-cli/src/lsp.rs`: the `tessera lsp` subcommand, speaking LSP over stdio.
- A diagnostic parity test (task 7).

## Tasks

1. **Library choice.** Evaluate `tower-lsp-server` and `lsp-server` (with `lsp-types`). Choose one, and record the reasons in the crate README.
2. **Initialization.** Find the project's `tessera.toml` from the workspace folders. Negotiate position encoding: use UTF-16 unless the client offers UTF-8, and convert with `tessera-core`'s `LineIndex`. Advertise only the capabilities this phase implements.
3. **Changes from every source.** Feed all of them into phase 13's `apply` API:
   - Open documents: incremental text sync. An open document's contents take precedence over the file on disk.
   - Files not open in the editor: register for file watching (`workspace/didChangeWatchedFiles`) on the content root, assets, and `tessera.toml`, so edits made outside the editor (a `git checkout`, another tool) are picked up, including creations, deletions, and renames.
   - `tessera.toml`: a model change reparses every file, open or not (phase 13's rules). Report the model's own problems as diagnostics on `tessera.toml`.
4. **Diagnostics.** Compute diagnostics with phase 14's `check_project` for the editor's default build (the content model's `[editor]` setting), and publish them for every affected file, including files that aren't open when an open file's change affects them. Clear diagnostics for deleted files.
5. **Stale results.** Work is asynchronous and can be overtaken by newer edits. Tag each computation with the snapshot versions it used (phase 13), cancel outdated work where possible, and never publish diagnostics for a document version older than the one the editor currently has.
6. **Semantic tokens.** Tokens for directive names (built-in versus project widget), attribute keys and values, the trailing colon, end lines, title lines, declared phrases versus undeclared candidates, and availability specs. Title lines get their own token type so accidental titles stand out (SPEC §10).
7. **Parity test.** For every build of `examples/quill` and a set of fixture projects with known problems, the diagnostics the server publishes equal `tessera check --build <name> --format json` output: same codes, files, and spans. Run it in CI on every platform.
8. **Robustness.** A panic while handling one request must not take down the server. Log to stderr, never stdout.

## Acceptance criteria

- [ ] The parity test passes for every build.
- [ ] Scripted tests show: editing an open fragment updates an including page's diagnostics; changing an *unopened* fragment on disk does the same; declaring a new widget in `tessera.toml` changes the diagnostics of an unopened file that uses it; deleting a linked file produces broken-link diagnostics; a slow computation overtaken by a newer edit never publishes its results.
- [ ] Diagnostics for a keystroke in a typical page publish within 50 ms on a developer laptop; a benchmark records the number.
- [ ] Positions are correct for lines containing multi-byte characters.

## Out of scope

- Completion, hover, navigation, CodeLens, and inlay hints (phase 16).
- Code actions, rename, and formatting (phase 24).

## Notes

- The server computes nothing itself; everything comes from the same crates as `tessera check`. If the editor and CLI ever disagree, that's a bug in how the server calls them.

## Handoff notes

### What was built

- **`crates/tessera-lsp`** (`lsp-server` + `lsp-types`; the reasons are in its README):
  - `server.rs`: the message loop (`serve(Connection, Options)`, `run_stdio()`), request and notification dispatch (each in `catch_unwind`), and the diagnostics worker thread.
  - `core.rs`: the state under one lock: open documents, the `IncrementalProject`, what is queued, what was published. Document sync, file watching, model changes, reload, publishing.
  - `compute.rs`: one round of diagnostics from a `Snapshot`: `check_file` for each file in `Affected::recheck`, then the editor build's page-level checks; conversion to LSP diagnostics.
  - `tokens.rs`: semantic tokens and the legend. `position.rs`: the negotiated encoding over `LineIndex`. `docs.rs`: incremental text sync. `fsx.rs`: the two file systems below. `uri.rs`: `file:` URIs.
- **`tessera lsp`** (`crates/tessera-cli/src/commands/lsp.rs`, plus `pub mod lsp;` in `commands/mod.rs` and one variant and one arm in `cli.rs`). No options. Exit code 0 after `shutdown` then `exit`, 1 for `exit` without `shutdown` (the protocol's), 2 when it can't run.
- **Tests.** `tessera-lsp/tests/scenarios.rs` (22: the acceptance scenarios, models, files, directories, panics, positions), `tokens.rs` (5), `differential.rs` (the server equals a fresh `check_project` after random sequences of edits, disk changes, asset changes, and model changes; `TESSERA_LSP_SEEDS=150` ran clean, and two deliberate breakages were caught), unit tests for URIs, positions, and text sync, and `tessera-cli/tests/lsp_parity.rs` (the real `tessera lsp` process against `tessera check --build <name> --format json`, for every build of `examples/quill` and of `tests/fixtures/lsp/problems`, which draws 30 different diagnostic rows, including a file that isn't UTF-8 and lines with astral-plane characters, in UTF-16 and UTF-8). `benches/keystroke.rs` is the benchmark.

### Interfaces later phases use

- **Semantic token legend** (types and modifiers): the README; append only. Phase 17 maps them to scopes; the README suggests one for each.
- **A diagnostic's `data`**: `{ "slug", "builds", "unpublished" }` (phase 24 finds a diagnostic's fix by `slug` and recomputes it from the snapshot; fixes aren't in `data`).
- **Adding a request** (phases 16, 24, 25): add a method name to `handle_request` in `server.rs`, a handler that takes `shared.lock()`, clones what it needs (`Core::tokens_target` is the pattern: a `Snapshot`, a content path, the model), and computes without the lock; advertise the capability in `serve`. `Core` is the place for state (`docs`, `loaded`, `encoding`).
- **`Options`**: `before_publish`, `before_request`, and `idle` are for tests (hold a round, panic a handler, wait for quiet).
- **Positions**: `Encoding::position`, `range`, `offset`, `offset_lenient` over a `LineIndex`; use them for every conversion.

### Decisions

- **One lock, one worker.** Every change (`apply`, a document version) and the check that a result is still current happen under `Shared::core`, so they can't interleave. The worker takes the dirty files and a `Snapshot`, computes without the lock, and publishes under it: a file whose `is_file_current` is false, or whose open document's version isn't the one the round started with, is dropped and queued again. A round abandons itself between stages when `Snapshot::is_current()` turns false; its files go back in the queue, and the next round takes their union. Keystrokes that arrive during a round coalesce into the next.
- **Both levels are incremental** (Q137). `check_file` runs for the files of a round (`Affected::recheck`, widened as below). The page-level diagnostics come from `PageChecker::with_index` over the snapshot's own index and `check_resolved` for the pages that can have a diagnostic located in a file of the round: the file itself when it's a page, and the pages that include it (`Project::including_pages`). Their resolved forms come from a `ResolvedCache` kept in the project's state; each update's `Affected` is queued and applied to it (`cache.apply`) at the start of the next round, by the worker, which is the only user of the cache. The other pages' diagnostics are simply not recomputed: they're what was last published. Numbers (3,000 pages): about 3 ms for a page keystroke, 9 ms for a fragment with 30 includers; the README has the table.
- **A round covers what its files include, now and at the last round.** A page that starts or stops including a fragment changes the page-level diagnostics located in the fragment (an `include-cycle` is located where the cycle closes; a fragment no page includes has none, Q104), but `Affected::recheck` lists the page and not the fragment. The differential test found it (a cycle in a fragment stayed after the only including page was edited, closed, or deleted). The server keeps what each file included at the last round (`compute::with_included`) and adds those files, and the targets of a deleted page. Phase 13's `recheck` could list them instead, which would make this unnecessary.
- **Two file systems, mirroring phase 13's overlay.** `BufferFs` is the incremental project's base: the disk, the open buffers over the sources at load, and **each probe answered once and remembered**. `IncrementalProject::apply` asks its base whether an asset was there *before* an update, and by the time the watcher reports a file created or deleted the disk shows the new state, so a live disk makes `AssetCreated` and `AssetDeleted` no-ops (found by the "deleting a linked file" scenario). `LayerFs` is what `Project::from_parts_with_fs` gives the file-level checks: the disk with the reported changes layered over it, answering as the overlay does.
- **Model changes.** A model that loads is `Change::Model`; `ApplyError::LayoutChanged` reloads the project from scratch (open buffers over the disk) and clears the diagnostics of files the new project doesn't have. One that doesn't load: Q131. Model warnings are published on `tessera.toml` synchronously, since they're cheap.
- **Open documents win.** A watcher event for an open source is ignored; closing a buffer re-reads the disk (a file that isn't there is deleted).
- **Directories.** An event for a directory that's gone deletes every source and every referenced asset the project knows under it; one for a directory that appeared is walked.
- **`positionEncoding`** UTF-16 unless the client offers UTF-8. No `--build` or setting: the build is the model's `[editor] build`.
- **Nothing extra advertised**: text sync (incremental, open/close), semantic tokens (full and range), the position encoding. The watcher is registered dynamically (`**/*`, filtered by the server: Q135).
- **New questions**: Q131 to Q137 (a model that doesn't load, several projects, a source that becomes unreadable, unpublished content, which files are followed, diagnostics of closed files, incremental page checks), each implemented as proposed and marked `SPEC-QUESTION`.

### Changes outside the crate

- `tessera-check/src/project.rs`: `Project::from_parts_with_fs(root, content_root, model, model_text, sources, Arc<dyn FileSystem + Send + Sync>)`, and `file_system()` returns it. `from_parts` is unchanged (the disk). `tests/file_system.rs` (3); `tests/parity.rs` passes unchanged.
- `tessera-resolve/src/incremental`: `IncrementalProject::load` takes `impl FileSystem + Send + 'static` (was without `Send`), and the overlay boxes `dyn FileSystem + Send`, so an `IncrementalProject` is `Send` and the server can compute on a worker. Every caller in the repository already passed a `Send` file system.
- `tessera-check/src/page/`: `PageChecker::with_index` and `PageChecker::check_resolved` (additive; every other entry point is unchanged), the index held as owned or borrowed in `bridge.rs`. `tests/page_index.rs` checks that `check_resolved` over every page equals `check`.
- `tessera-cli`: `commands/lsp.rs`, `pub mod lsp;`, the `Lsp` variant and arm, the `tessera-lsp` dependency.
- `project-docs/questions.md`: Q131 to Q137.

### Acceptance criteria

See the pull request for the status and evidence of each.

### Left open

- **`Affected::recheck` and fragments** (above): the fragments a page reaches aren't listed when only the page's includes change; the server covers it, and phase 13 could.
- **Model changes re-check every page** (`Affected::model` above `Warnings` clears the cache and rechecks every file), which is the first-load cost again (825 ms at 3,000 pages).
- **A source that becomes unreadable while running** (Q133) is treated as deleted; phase 13's `Change` has no way to say "unreadable".
- **CI on Windows and macOS.** The parity test and the server tests use no platform-specific behavior (paths are `file:` URIs converted by `uri.rs`, which handles `file:///c:/…` but has only been run on Linux); they were run locally on Linux only, because CI is manual-only.
- **No progress reporting** (`$/progress`) while the first load computes; a 3,000-page project takes about a second.
- **Multi-root and workspace-folder changes** (Q132) aren't handled.
- **Symbolic links.** Paths are compared lexically, so a workspace opened through a symlink whose real path the watcher reports differently can miss events.
