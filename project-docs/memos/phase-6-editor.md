# Phase 6: Memo files in the editor

Part of [Memos](README.md). Needs phase 5.

## Goal

A memo is a click to read and a command to write, a rename keeps it attached, and a personal memo stays on one machine.

In VS Code, a heading with memos has a lens, "2 memos", that opens them. With the cursor in a paragraph, **Ascribe: Add Memo** makes a file in `memos/` with `about` and `quote` filled in from the cursor, and opens it at the body. **Ascribe: Add Personal Memo** does the same in `.ascribe/memos/`. Renaming the heading's id with `F2` rewrites the memo's `about`, and so does `ascribe rename`.

## Context

- README decisions 12, 15 to 17, 24, and 25.
- `crates/ascribe-lsp/src/`: `links.rs` and `server.rs` for the lenses the server has, `code_action.rs`, `complete.rs`, `rename.rs`, `references.rs`, and `fsx.rs` for the files it watches.
- `packages/vscode/src/`: the commands, `ui/buildLens.ts`, and the sidebar's Used by view in `ui/sidebarViews.ts`.
- `crates/ascribe-cli/src/commands/rename.rs`, and how a rename rewrites a glossary term's `link` and a retired permalink's `to` in `ascribe.toml`.
- The memo store from phase 5.

## Design

### Personal memos

The second implementation of the memo store reads `.ascribe/memos/`, in the same format. The editor, `ascribe outline`, and `ascribe refs` read both, and mark a personal memo as one. `ascribe check` reads the committed folder only, so what it reports doesn't depend on whose machine it runs on; the editor reports a personal memo's problems as it reports a team memo's.

### In the editor

- **A lens** on a heading, or on a block, with memos about it: "1 memo" or "3 memos", one outdated when one is. It opens the memo, or a pick list of them.
- **Ascribe: Add Memo** and **Ascribe: Add Personal Memo**, in the Command Palette and the editor's context menu. `about` is the section the cursor is in, by its id, or the page when there's no heading; `quote` is the selection, or the first line of the block at the cursor. The file is named from the quote's first words, and never over a file that's there.
- **In a memo file:** completion of `about`'s targets as `ascribe refs` takes them, a link from `about` to its target, and the checks' diagnostics as the file changes. A memo file is watched as a page is.
- **Used by** lists the memos about the target, beside its links and includes.

### Rename

Renaming a page, moving it, or changing a heading's id, from the command or the editor, rewrites each memo's `about` that names it, keeping its `#id`, in both folders. The memo's body and `quote` are left alone.

## Tasks

1. The personal store, and `ascribe outline` and `ascribe refs` reading both, with `personal` in their JSON.
2. The memo files watched by the language server, with their diagnostics, completion, and link.
3. The lens, and its test.
4. The two commands, and their tests in the extension.
5. Used by.
6. Rename, from `ascribe rename` and the editor, with tests for a page moved, an id changed, and a memo in each folder.
7. The editor's page, the memo file reference, `ascribe rename`'s reference, `changes/`, and the surface map.

## Out of scope

- Showing memos beside blocks in the preview (README, "Carved out").
- A memo store other than the two folders.

## Acceptance criteria

- In the extension's tests, a heading with a memo has a lens that opens it, and **Ascribe: Add Memo** in a paragraph writes a memo that `ascribe outline` lists as placed on that paragraph's section.
- A personal memo is listed by `ascribe outline` with `personal`, and isn't read by `ascribe check`.
- `ascribe rename install-agent.md#install-agent install` in a copy of `examples/quill` rewrites the memo's `about`, and `ascribe check` reports nothing after it.
- `F2` on the heading's `@id` in the editor does the same.

## Verify

```sh
cargo test --workspace --locked
pnpm --filter ascribe-vscode test
```

Then, in a copy of `examples/quill`, `ascribe rename install-agent.md#install-agent install` and `ascribe check`.

## Commits

1. "Keep personal memos in .ascribe/memos/"
2. "Show a section's memos in the editor, and add one from the cursor"
3. "Rewrite a memo's about when its target is renamed"
