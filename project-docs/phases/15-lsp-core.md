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

_To be filled in by the implementing agent._
