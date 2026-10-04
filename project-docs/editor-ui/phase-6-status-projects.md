# Phase 6: The status bar item and the Projects view

Part of [Editor UI](README.md). Requires phase 3 (for the registry pattern), not phases 4 or 5. Extension only.

## Goal

- **A status bar item** showing the active file's project and its editor build, and the project's server state.
- **An Ascribe view container** in the activity bar, with a **Projects** view listing every project in the workspace.

## Context

- `packages/vscode/src/registry.ts`: `ProjectRegistry` (`projects`, `servers`, `serverFor`, `name`, `onDidChangeProjects`, `onDidStart`) and `ProjectServer` (`state`, `binary`, `showOutput`, `restart`, `start`).
- `packages/vscode/src/extension.ts`: the existing commands (`ascribe.restartServer`, `ascribe.showOutput`, `ascribe.openPreview`).
- Phase 1's `ascribe/targets` (`builds`, with the editor build).
- VS Code: `window.createStatusBarItem`, `contributes.viewsContainers.activitybar`, `contributes.views`, `window.createTreeView` with a `TreeDataProvider`, `contributes.viewsWelcome`, `menus["view/item/context"]` for inline buttons.

## Design

### Status bar item

- Text: the project's name and editor build, with a codicon for the server's state, such as `$(book) docs · site`, `$(sync~spin) docs` while starting, `$(warning) docs` when it failed.
- Shown only while an Ascribe page or `ascribe.toml` is the active editor; hidden otherwise. A file in no project shows nothing.
- Tooltip: the project's folder, its `ascribe.toml`, the binary in use, and the state.
- Click: a small quick pick with **Show output**, **Restart server**, and **Open preview**, each running the existing command for this project.
- The build comes from `ascribe/targets` when the server is running. Don't start a server to fill it (README decision 7). The page being active already started it.

### Projects view

- Every project the registry knows, started or not, by name, with its folder as the description and its state as the icon.
- Children: the project's `ascribe.toml` (opens it) and, when running, its editor build and binary.
- Inline buttons: **Show output** and **Restart** (restart only for a running or failed server).
- It updates when projects appear or go and when a server's state changes. `ProjectServer` needs an `onDidChangeState` event for this; add it.
- A welcome view when there are no projects: a short explanation and a link to `docs/getting-started.md`.

## Tasks

1. `ProjectServer.onDidChangeState`, fired on every state change, with a unit test.
2. `src/ui/statusBar.ts` and `src/ui/projectsView.ts`; the view container (with an icon in `media/`), the view, the welcome content, and the menus in `package.json`.
3. Unit tests for the item's text and visibility rules and the tree's items, from fake registries.
4. Integration tests on `test/fixtures/monorepo`:
   - the view lists all projects with none running;
   - opening a page starts its server and the view shows it running;
   - the status bar shows that project.
5. `docs/editor.md` and `CHANGELOG.md`.

## Out of scope

Choosing a different editor build (it's `[editor] build` in `ascribe.toml`); the other views (phase 7).

## Acceptance criteria

- The item always names the active file's project correctly, including nested projects, and its state.
- The view lists every project, never starts a server by itself, and its buttons act on the right project.

## Verify

```sh
pnpm --filter ascribe-vscode typecheck
pnpm --filter ascribe-vscode test
cargo build -p tessera-cli && ASCRIBE_BIN=$PWD/target/debug/ascribe pnpm --filter ascribe-vscode test:integration
pnpm lint && pnpm format:check
```

## Commits

1. "Show the active file's project in the status bar"
2. "Add an Ascribe sidebar with a Projects view"
