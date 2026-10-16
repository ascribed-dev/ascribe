# Phase 5: Pages and Content model

Part of [The editor's views](README.md). Needs phase 4.

## Goal

Pages and the content model's entries have menus and filters, pages show their problems and the build's reach, and a phrase or a widget goes in from the list.

In VS Code, in Ascribe's docs, a page with an error is red in Pages, as in the Explorer. Switch to a build that leaves pages out, and they're dimmed. Right-click a page: **Open Preview to the Side**, **Copy Link**, **Rename or Move…**, **Find All References**, **Reveal in Explorer**; right-click a section: **New Page Here**. Turn on **Orphans Only**, and Pages lists the pages and fragments nothing links to or includes. In Content model, right-click a phrase: **Find All References**, **Rename…**, **Insert at the Cursor**, which puts `{key}` where the cursor was in the page; **Unused Only** leaves the entries nothing uses. Page types are listed with the rest.

## Context

- README decisions 5 and 17.
- `packages/vscode/src/ui/sidebar.ts` and `sidebarViews.ts`: the Pages nodes (by content type, Fragments, Orphans) and the Content model nodes, and `SidebarApi`.
- Phase 4's `builds` on each page of `ascribe/inventory`, and the build the status bar shows (`src/ui/chosenBuild.ts`, `projectBuilds.ts`).
- `src/actions/registry.ts`: **Insert a phrase** and **Insert a widget**, their operations, and their hints. **Copy a link to this section**, for the address Copy Link copies. **New Page** (`ascribe.newPage`), and its folder step.
- The server's rename (`crates/ascribe-lsp/src/rename.rs`, and `willRenameFiles`), and Find All References on each kind of entry.
- `ascribe model`'s page types, which `ascribe/inventory` doesn't list yet; the versions, if `ascribe model` reports them when this phase starts.
- The docs: the editor page's Pages and Content model, and its actions bar.

## Design

### Pages

- **Problem colors:** each page and fragment item has its file as `resourceUri`, so VS Code's problem decorations apply, as in the Explorer.
- **Dimming:** a page whose `builds` doesn't have the build the status bar shows is drawn with the disabled color and says "Not in cloud" in its description, as the build lens dims text. It follows the build as it changes.
- **The menus,** at most one inline button (**Open Preview to the Side**):

  | On | Item | Does |
  |---|---|---|
  | A page | Open Preview to the Side | `ascribe.openPreview` on it |
  | A page | Copy Link | Its source path from the content root, `/keys.md`, which is what a link in a page uses, as **Copy a link to this section** writes `/guides/install.md#install-cli`. The published address is This page's Address row |
  | A page or fragment | Rename or Move… | Asks for the new path, then makes the edits renaming the file in the Explorer makes |
  | A page or fragment | Find All References | The references peek at the page's start |
  | A page or fragment | Reveal in Explorer | `revealInExplorer` |
  | A group of pages | New Page Here | `ascribe.newPage` with the folder chosen, when the group is a folder |

- **Orphans Only,** a toggle in the title bar backed by a context key, replaces the Orphans group, and covers fragments that nothing includes too.

Pages keeps its grouping, by content type today; the [navigation plan](../navigation/phase-5-editor.md) changes it, and these actions apply to whatever groups it has. New Page Here is offered on a group that's a folder only.

### Content model

- **The menus:** Find All References on every entry; **Rename…** on a phrase and a dimension value, which runs the server's rename and shows the refactor preview; **Insert at the Cursor** on a phrase and a widget.
- **Insert at the Cursor** runs **Insert a phrase**'s or **Insert a widget**'s operation, with the entry chosen, into the project's last Markdown editor at its cursor. The extension keeps that editor as Markdown editors of the project become active, per project. When there's none, or its cursor isn't where the action applies, it shows the action's hint and changes nothing. The editor is brought to the front after.
- **Unused Only,** a toggle in the title bar.
- **Page types,** a group of their own, each with how many pages it has, opening its declaration; and **versions**, if `ascribe model` reports them, opening `[versions]`.

## Tasks

1. `resourceUri` on page and fragment items, and the dimming, with unit tests.
2. Pages' menus, New Page Here, and Orphans Only.
3. Content model's menus, Rename, and Unused Only.
4. The last Markdown editor of each project, and Insert at the Cursor, with unit tests for no editor, a cursor where a phrase can't go, and one where it can.
5. Page types, and versions if they're there, in `ascribe/inventory` and the view.
6. Integration tests on `examples/quill`.
7. The docs: the editor page's Pages and Content model, with each menu, the toggles, and the dimming; the actions bar section saying Insert at the Cursor runs its actions from Content model; `changes/editor/views-actions.md`; and the surface map.

## Out of scope

- Pages' structure (the navigation plan).
- Dragging an entry from Content model into a page (decision 5).
- Inserting a glossary term, a feature, a dimension, or a note type.

## Acceptance criteria

- The goal's steps, in integration tests.
- Insert at the Cursor writes what **Insert a phrase** writes for the same phrase at the same place.
- Rename… from Content model makes the same edits as `F2` on the phrase in a page.
- A page's color in Pages matches the Explorer's for the same file.

## Verify

```sh
cargo test -p ascribe-lsp --locked
pnpm --filter ascribe-vscode test
```

The integration tests run in CI.

## Commits

1. "Color, dim, and filter the pages in Pages"
2. "Act on a page or an entry from the sidebar"
3. "Insert a phrase or a widget from the content model"
