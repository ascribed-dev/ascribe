# Phase 6: No Projects view

Part of [The editor's views](README.md). Needs nothing; comes after phases 1 to 5 so the tests move once.

## Goal

The Projects view is gone; a workspace with several projects switches with **Switch Project**.

In VS Code, open Ascribe's own repository. The sidebar has This page, Pages, and Content model, and no Projects. Pages' title bar has a **Switch Project** button beside "docs". It lists every project, the current first and marked, each with its folder and "running, docs build", "not checked yet", or "failed", and buttons for its output and a restart. Choose `examples/quill`, and the views show it, its server starting. Open a page of `docs/`, and they follow it back. In a workspace with one project, there's no button and no command in the palette. In a workspace with none, Pages says how to start one.

## Context

- README decisions 1, 2, and 10.
- `packages/vscode/src/ui/projectsView.ts`, its commands (`ascribe.projects.refresh`, `.showOutput`, `.restart`), and `describe.ts`'s project nodes and `tildeFolder`.
- `packages/vscode/src/registry.ts`: the projects found, `MAX_PROJECTS`, `ascribe.projects.exclude`, and the cap's message.
- `src/ui/statusBar.ts`: the project menu, and its `answerNext` for tests.
- `src/ui/sidebarViews.ts`: `follow`, and the views' descriptions naming the project.
- `packages/vscode/package.json`: the view `ascribe.projects`, its two welcome entries, its menus, and Pages' `when: ascribe.active`.
- `AscribeApi.ui.projects`, and the tests through it: `test/integration/suite/monorepo.it.ts` (run on Windows too) and `test/unit/sidebar.test.ts`.
- The docs: the editor page's sidebar section ("The Projects view") and status bar; `docs/content/editor/settings.md` ("Workspaces with several projects", the cap, and "Commands act on the project that owns the active file"); the walkthrough's `sidebar.md` and `project.md`; and the extension's README.

## Design

- **`ascribe.multipleProjects`,** a context key the registry sets as its list changes: found, left out, or deleted. True at two or more.
- **Ascribe: Switch Project** (`ascribe.switchProject`): a quick pick of the projects, as the goal describes, with **Show Output** and **Restart** as item buttons (Restart on a running or failed server only, as the view had it). Choosing one sets the sidebar's project and starts its server if it isn't running. It's in the Command Palette, as a title button on Pages and Content model, and in the status bar item's menu, each `when: ascribe.multipleProjects`.
- **The jump:** the sidebar's project is the chosen one until a file of another project becomes active, and then it follows, as `follow` does now.
- **A server that fails to start:** the views show the project's name with "The language server failed to start" and **Show Output** and **Restart** as their message's links, not "Waiting for the project's language server", and Switch Project's item for it says "failed". The sidebar stays on that project until another is chosen or a file of another becomes active.
- **The view goes:** `ascribe.projects`, its commands, its menus, and `ProjectsViewApi`. Its welcome entries move to Pages, which loses `when: ascribe.active`. The Refresh button's job, asking the running servers for their builds again, is a button in Switch Project's title.
- **The cap's message** names **Switch Project** and `ascribe.projects.exclude`.
- **The test API** gains `switchProject`, answered as the status bar's menu is, and `monorepo.it.ts` and `sidebar.test.ts` move onto it.

The binary, which only the Projects view and the status bar's tooltip showed, is in the tooltip and in Switch Project's items' detail.

## Tasks

1. The context key, with unit tests for 0, 1, 2, and a project left out; and the views' message for a chosen project whose server failed.
2. Switch Project, its buttons, the jump, and its test API.
3. The view's removal, the welcome entries on Pages, and the cap's message.
4. `monorepo.it.ts` and `sidebar.test.ts` moved, and the full matrix run from the branch (Actions → CI → Run workflow), since the multi-project tests are about paths.
5. The docs: the editor page's sidebar section without Projects, and Switch Project where several projects are described; `editor/settings.md`'s several projects and the cap; the walkthrough's sidebar and project steps; the README's sidebar line; `changes/editor/switch-project.md`; and the surface map, which loses three commands and gains one.

## Out of scope

- Pinning the sidebar to a project (decision 2).
- A view of the projects anywhere else, such as the Explorer.

## Acceptance criteria

- The goal's steps, in `monorepo.it.ts`, on Linux and Windows.
- With one project, `ascribe.switchProject` isn't in the Command Palette, the menus, or the title bars.
- With no project, the Ascribe container shows Pages' welcome message, and in Restricted Mode the trust message.
- No command, view, or test API is named for Projects.

## Verify

```sh
pnpm --filter ascribe-vscode test
```

The integration tests run in CI, on the full matrix for this phase.

## Commits

1. "Switch between projects from a quick pick, when there's more than one"
2. "Remove the Projects view"
