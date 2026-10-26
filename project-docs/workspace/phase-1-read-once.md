# Phase 1: Read once

Part of [Load a project once](README.md). Needs #1007.

## Goal

Each command reads each source file once per run and indexes the project once. A test counts the reads through a `FileSystem` that counts, and fails on a second.

## Context

- README "What exists today", the two reads: `Project::load` reads every source, and `Project::index` loads an `ascribe_resolve::Project` that reads them again through `Project::file_system`.
- The page checks already index the held texts once per project (`link_index`, built by `held_index` through `Held`), so a command that checks and builds makes two indexes today, `Project::index`'s and the page checks', from two reads.
- `Held` lists no nested projects and no unreadable files. An index over it can differ from `Project::index`'s: a link into a nested project's folder, and `Project::unreadable`.

## Design

- **Keep the listing.** Loading keeps the `Sources` it listed (`FileSystem::sources`: the paths, the unreadable entries, the nested projects' folders, and the project's own folder), and `Held` gives it back as it was, with the held texts for `read`. An index over `Held` is then what `Project::index` gives today, from the texts already read.
- **One index.** `Project::index` returns the project's `link_index`: an `Arc<ascribe_resolve::Project>` built once, which the page checks and the command share. `held_index` and `Project::index` are the same thing, so one goes. A project with text laid over it (`Project::with_source`) builds its own, as it does now.
- **Loading through any `FileSystem`.** `Project::load` reads through `DiskFs`; a loader that takes the `FileSystem` (the one `ascribe_diff::GitFs` and the test need) is how both read, so the counting test loads as a command does.
- **The callers** that took an owned `ascribe_resolve::Project` from `Project::index` take the `Arc`. `answer::Loaded::index` keeps nothing of its own and hands out the project's.

## Tasks

1. A `FileSystem` that counts `sources` and each `read` by path, wrapping another, in the test support `crates/ascribe-cli/tests/` shares ([writing-tests](../../.claude/skills/writing-tests/SKILL.md) says where).
2. The test: `examples/quill` loaded through it, then each command's library entry as its command calls it, `diagnose`, `write_outputs` into memory, `diff_project` and `drift_project` against a fixture's revision, `outline`, `link`, `refs`, `resolve`, `search`, `permalinks::list`, and `versions::list`; each source read once, and the listing taken once. It fails first, on `main`.
3. Keep the listing, give it back through `Held`, and the loader over any `FileSystem`.
4. `Project::index` as the shared index; the callers, and `answer::Loaded`.
5. ARCHITECTURE.md's "Loading a project, and reading files" no longer says the commands index twice.

## Out of scope

The `Workspace` type, the crate, the MCP server and the hook's check server, the language server.

## Acceptance criteria

- The counting test passes, and failed on `main`.
- `node scripts/compare/outputs.ts --base main` shows nothing.
- The perf benchmark is no slower on the synthetic project and the Elastic sample.

## Verify

```sh
cargo test -p ascribe-cli --test all read_once
cargo test --workspace --locked
node scripts/compare/outputs.ts --base main
cargo bench -p ascribe-corpora --bench perf
```

## Commits

1. "Count a command's reads of a project's files"
2. "Read each source once, and index once, in every command"
