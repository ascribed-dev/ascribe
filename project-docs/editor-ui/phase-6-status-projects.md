# Phase 6: The status bar item and the Projects view

Part of [Editor UI](README.md). Requires phase 3 (for the registry pattern), not phases 4 or 5. Extension only.

## Goal

- **A status bar item** showing the active file's project, the build you're looking at, and the project's server state, with a way to switch the build.
- **An Ascribe view container** in the activity bar, with a **Projects** view listing every project in the workspace.

## Context

- `packages/vscode/src/registry.ts`: `ProjectRegistry` (`projects`, `servers`, `serverFor`, `name`, `onDidChangeProjects`, `onDidStart`). `packages/vscode/src/client.ts`: `ProjectServer` (`state`, `binary`, `showOutput`, `restart`, `start`).
- `packages/vscode/src/extension.ts`: the existing commands (`ascribe.restartServer`, `ascribe.showOutput`, `ascribe.openPreview`, `ascribe.selectPreviewBuild`).
- `packages/vscode/src/preview/controller.ts`: the preview already keeps a chosen build per project folder (`builds`, `chooseBuild`, `pickBuild`), where choosing the editor build means "follow `[editor] build`". That choice becomes the shared one (README decision 10).
- [The agents plan](README.md#the-agents-plan): its phase 3 may start a server when a file changes on disk, which shapes how decision 7 is tested here.
- Phase 1's `ascribe/targets` (`builds`, with the editor build).
- VS Code: `window.createStatusBarItem`, `contributes.viewsContainers.activitybar`, `contributes.views`, `window.createTreeView` with a `TreeDataProvider`, `contributes.viewsWelcome`, `menus["view/item/context"]` for inline buttons.

## Design

### Status bar item

- Text: the project's name and the build you're looking at, with a codicon for the server's state, such as `$(book) docs · site`, `$(sync~spin) docs` while starting, `$(warning) docs` when it failed.
- Shown only while an Ascribe page or `ascribe.toml` is the active editor; hidden otherwise. A file in no project shows nothing.
- Tooltip: the project's folder, its `ascribe.toml`, the binary in use, and the state.
- Click: a small quick pick with **Switch build**, **Show output**, **Restart server**, and **Open preview**. The last three run the existing command for this project. **Switch build** lists the project's builds, marks the editor build, and sets the project's chosen build.
- The builds come from `ascribe/targets` when the server is running. Don't start a server to fill it (README decision 7). The page being active already started it.

### The build you're looking at

`src/ui/chosenBuild.ts` holds one chosen build per project for the session, with an event when it changes. Move the preview controller's per-folder choice into it, keeping its rule that choosing the editor build means following `[editor] build`. The preview's picker (`ascribe.selectPreviewBuild`), the status bar's **Switch build**, and phase 8's lens all read and set this one value, so switching in any of them changes the others. The preview behaves as it does today.

### Projects view

- Every project the registry knows, started or not, by name, with its folder as the description and its state as the icon.
- Children: the project's `ascribe.toml` (opens it) and, when running, its editor build and binary.
- Inline buttons: **Show output** and **Restart** (restart only for a running or failed server).
- It updates when projects appear or go and when a server's state changes. `ProjectServer` needs an `onDidChangeState` event for this; add it.
- A welcome view when there are no projects: a short explanation and a link to `docs/content/getting-started.md`.

## Tasks

1. `ProjectServer.onDidChangeState`, fired on every state change, with a unit test.
2. `src/ui/chosenBuild.ts`, with the preview controller reading and setting it; the preview's existing build tests pass unchanged.
3. `src/ui/statusBar.ts` and `src/ui/projectsView.ts`; the view container (with an icon in `media/`), the view, the welcome content, and the menus in `package.json`.
4. Unit tests for the item's text and visibility rules and the tree's items, from fake registries.
5. Integration tests on `test/fixtures/monorepo`:
   - the view lists all projects, and showing and refreshing it starts no server (assert on what the view caused, not on no server running: see the README's section on the agents plan);
   - opening a page starts its server and the view shows it running;
   - the status bar shows that project;
   - **Switch build** changes the build an open preview renders, and the preview's picker changes what the status bar names.
6. Descriptions for new commands in `docs.test.ts`'s `commands`, then bless the commands table. `docs/content/guides/editor.md` and `CHANGELOG.md`.

## Out of scope

Changing the editor build, which decides the diagnostics (it's `[editor] build` in `ascribe.toml`); the other views (phase 7); dimming by the chosen build (phase 8).

## Acceptance criteria

- The item always names the active file's project correctly, including nested projects, and its state.
- The view lists every project, never starts a server by itself, and its buttons act on the right project.
- A project has one chosen build: the status bar and the preview always name the same one.

## Verify

```sh
pnpm --filter ascribe-vscode typecheck
pnpm --filter ascribe-vscode test
cargo build -p ascribe-cli && ASCRIBE_BIN=$PWD/target/debug/ascribe pnpm --filter ascribe-vscode test:integration
pnpm lint && pnpm format:check
```

## Commits

1. "Show the active file's project in the status bar"
2. "Add an Ascribe sidebar with a Projects view"
