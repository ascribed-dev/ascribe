# Phase 5: The editor

Part of [Navigation](README.md). Needs phase 3.

## Goal

The Pages view is the navigation, a page is put in order by dragging it, and a rename keeps `ascribe.nav` right.

In VS Code, in `examples/astro-site` with no `ascribe.nav`, Pages shows Start, Guides, Reference, each with its pages, and Fragments by folder. Drag Reference above Guides: `ascribe.nav` is created with the two sections, unsaved, and undo removes it. Drag Options below a new page in Reference: the section's `pages` is written as it stood, then the one moved. Drag a page onto Guides: a dialog says the page's address changes, and on **Move** the file moves and its links and `ascribe.nav` follow. A page added with no place is shown at the end of its section, marked, with **Place Here** in its menu. Open a page, and Pages reveals it.

## Context

- README decisions 10, 15, and 13, and phase 1's decisions 1 and 6.
- `packages/vscode/src/ui/sidebar.ts` and `sidebarViews.ts`: the Pages nodes, by content type, Fragments, and Orphans; `SidebarApi`.
- `crates/ascribe-lsp/src/inventory.rs`: what Pages is drawn from, which gains the navigation.
- `crates/ascribe-lsp/src/edit.rs` and `edit/`: the `ascribe/edit` operations the actions bar uses, which keep a file's comments and layout; and the refactor preview.
- `crates/ascribe-lsp/src/rename.rs` and `crates/ascribe-cli/src/commands/rename.rs`: what a page's rename or move rewrites, in pages and in `ascribe.toml`.
- VS Code's `TreeDragAndDropController` and `TreeView.reveal`.
- The [editor's views plan](../editor-ui/phase-5-pages-and-model.md), phase 5: Pages' menus, colors, dimming, and filters.

## Design

### The view

`ascribe/inventory` gains the navigation, the project's, with each page's builds and whether it's listed in the file. Pages draws it: sections, in order, each with its own page first, its pages, and its nested sections; then Fragments, by folder. A page not listed in a section that lists pages is marked "Not placed", where the default puts it. A section's label is shown, and its folder beside it.

### The drags

Each drop is an `ascribe/edit` operation on `ascribe.nav`, applied as a workspace edit, so it's unsaved, undoable, and in git as typing is:

- **A section, among its siblings:** writes the sections' order. With no file, the first drop creates it with the sections listed and nothing else.
- **A page, within its section:** writes the section's `pages` as the view shows it, the first time, then moves the one.
- **A page, onto another section:** asks first ("Moving `My Setup.md` to Reference changes its address."), then makes the edits a file rename makes, `ascribe.nav`'s among them (below).
- **A section's own page** can't be dragged: it's first by decision 4.

**Place Here** on a marked page appends it to its section's `pages`, as the safe fix does. **Rename Label…** on a section or a page edits `label` or `nav-label`.

### Following the open file

When the active editor changes to a page of the project, Pages reveals it, expanding its sections, without taking focus.

### Rename

`ascribe rename` and the editor's rename of a page or a folder rewrite `ascribe.nav`'s names of it: a page's entry in `pages`, and a folder's `folder` and its nested sections'. A move to another folder removes the page from the old section's `pages`, and the new section places it by its default, or reports it unplaced.

## Tasks

1. The navigation in `ascribe/inventory`, its type in `shapes.rs`, and the view drawn from it, with unit tests.
2. The `ascribe/edit` operations for the drops, Place Here, and Rename Label…, with tests of the server that each keeps the file's comments. The first drop creates `ascribe.nav` with a `CreateFile` in the workspace edit, the part of a `WorkspaceEdit` the extension uses least, so an integration test checks that undo removes the file and leaves nothing behind.
3. The drag and drop controller, and the move's dialog.
4. Revealing the open page.
5. `ascribe.nav` in `ascribe rename` and the editor's rename, with tests for a page, a folder, and a move between sections.
6. Integration tests for the goal's steps.
7. The docs: the editor page's Pages (the navigation, the drags, Place Here, Rename Label…); the navigation reference ("the editor is the easy way to change it"); `ascribe rename`'s reference; `changes/editor/pages-navigation.md`; and the surface map.

## Out of scope

- Dragging between projects.
- What the editor's views plan adds to Pages, which applies to these sections whichever lands first.

## Acceptance criteria

- The goal's steps, in integration tests.
- After any sequence of drags, `ascribe check` reports nothing in `ascribe.nav`, and the sidebar `sidebar()` draws has the view's order.
- `ascribe rename` of a listed page leaves `ascribe check` with nothing to report.

## Verify

```sh
cargo test --workspace --locked
pnpm --filter ascribe-vscode test
```

The integration tests run in CI.

## Commits

1. "Show the navigation in the Pages view"
2. "Put pages and sections in order by dragging them"
3. "Keep ascribe.nav right when a page moves"
