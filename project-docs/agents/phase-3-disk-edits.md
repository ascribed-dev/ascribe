# Phase 3: Diagnostics for files changed on disk

Part of [Agents](README.md). Needs no other phase, and can run at the same time as phases 1 and 2. TypeScript only: `packages/vscode`.

## Goal

An agent that edits a project's files on disk, without opening them in an editor, still gets that project's problems in VS Code. Today, with `ascribe.startServers: "onDemand"`, the project's server hasn't started, the Problems panel is empty, and the agent concludes its page is clean.

This is for agents inside VS Code (Copilot, Cursor, Claude Code's panel), which read the Problems panel. Claude Code in a terminal gets diagnostics another way, from the plugin's language server (phase 9).

## Context

- `packages/vscode/src/registry.ts` (`ProjectRegistry`): finding projects, starting servers on demand, and the watcher for `ascribe.toml`. `projects.ts` holds the project type and path helpers.
- `crates/tessera-lsp/README.md`, "Diagnostics": the server publishes file-level diagnostics and the page-level ones of the editor's build only. An agent reading the Problems panel sees less than `ascribe check` reports.
- `packages/vscode/package.json`: `ascribe.startServers` and its description.
- `packages/vscode/test/integration/suite/monorepo.it.ts`: the suite that checks what starts when.
- Issue [#56](https://github.com/ascribed-dev/ascribe/issues/56): VS Code's file watcher can miss a new directory on Linux. Read it before adding a watcher; the same weakness applies here.
- The editor UI plan's decision 7 and the review plan's decision 8: nothing starts a server just to fill a view. This phase adds one narrow reason to start one.

## Design

- **When a source file of a project changes on disk** (created, changed, or deleted; a `.md` file under the project's content root, or a file its model names), and that project's server isn't running, start it. Once started, it behaves as today.
- **Only for changes made while the window is open.** Opening a workspace starts nothing new. A `git checkout` that touches 400 files starts each touched project's server once, not 400 times: starts are deduplicated, and a burst is debounced (choose an interval and test it).
- **`ascribe.startServers` gains nothing.** `onDemand` now means "when one of its files is opened or changes". Update its description. `all` is unchanged.
- **One watcher per workspace folder,** on a glob, filtered to projects in the registry. Don't add a watcher per project.
- **What the agent is told.** The Problems panel covers the editor's build. `docs/editor.md` and the setting's description say so, and say that `ascribe check` covers every build ([decision 12](README.md#decisions)).
- **Files outside any project,** and files in `node_modules`, `.git`, and a project's output directory, start nothing.

## Tasks

1. The watcher and the start, with unit tests for the filtering and the debounce.
2. Integration tests in the monorepo suite: writing a file of an idle project (with `fs`, not through the editor) starts its server and its problems appear; writing into the output directory doesn't; a burst starts the server once.
3. `docs/editor.md`, the setting's description, `CHANGELOG.md`.

## Out of scope

Fixing #56 (say in the pull request whether this phase makes it better, worse, or neither); a setting to turn this off, unless the integration suite shows it's needed.

## Acceptance criteria

- With the default settings and no Ascribe file open, a file written on disk with a broken link shows the problem in the Problems panel within a few seconds.
- Opening a workspace still starts no server until a file is opened or changed.

## Verify

```sh
pnpm --filter ascribe-vscode test
pnpm typecheck && pnpm lint && pnpm format:check
cargo build -p tessera-cli && ASCRIBE_BIN=$PWD/target/debug/ascribe pnpm --filter ascribe-vscode test:integration
```

## Commits

1. "Start a project's server when one of its files changes on disk"
