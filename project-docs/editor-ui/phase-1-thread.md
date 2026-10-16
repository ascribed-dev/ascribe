# Phase 1: The thread

Part of [The editor's views](README.md). Needs nothing.

## Goal

**This page** replaces Used by: the page's title, type, and addresses, and what links to the heading at the cursor, from one request.

In VS Code, in `examples/quill`, open `docs/keys.md` and put the cursor under "Rotate keys". The sidebar's second view is **This page**. Its **About** section says "API keys", its type, its address, and its Markdown version's address. Its **Connections** section says "Linked from 1, to Rotate keys" over `install-agent.md`, and "Included by" is empty. Move the cursor to "Create a key", and Connections follows.

## Context

- README decisions 3, 4, 10, and 11.
- `packages/vscode/src/ui/sidebarViews.ts`: `askUsedBy`, which asks `ascribe/context` then `textDocument/references`, its debouncing, and the view's messages ("Open a page to see what links to it.", "Waiting for the project's language server."). `src/ui/sidebar.ts` for the nodes, and `SidebarApi`.
- `crates/ascribe-lsp/src/context.rs`, `references.rs`, and `inventory.rs`, for the answers This page puts together; `server.rs` for how a custom request is registered; the snapshot the server answers from.
- `crates/ascribe-cli/src/shapes.rs`, which takes `ContextResult` and `InventoryResult` to the extension's `src/shapes.ts`.
- `packages/vscode/package.json`: the view `ascribe.usedBy`.
- `packages/vscode/test/unit/sidebar.test.ts`, and `test/integration/suite/quill.it.ts`.
- The docs: `docs/content/editor/index.md` ("The Ascribe sidebar", and "Used by, Pages, and Content model"), `packages/vscode/media/walkthrough/sidebar.md`, and `packages/vscode/README.md`.

## Design

### The request

```text
ascribe/page
  params: { textDocument, position }
  result: { about: { title, type, address, markdownAddress }, heading?: { id, text },
            linkedFrom: [place], includedBy: [place] }
```

`linkedFrom` is the links to the heading at `position` when there's one, and to the page otherwise, as Used by's are now, through a page that includes it among them. `includedBy` is what includes the page, or the heading's section. A place is a file, a range, and the line's text, as Used by shows.

**What doesn't depend on the cursor** (`about`, and the links to each of the page's headings) is kept for the page and made again when the snapshot changes the page or anything that links to it, so a cursor move only picks the heading's share. Where that bookkeeping is more than the request is worth, the server answers the cursor-free part from the snapshot each time and keeps nothing; what decides it is the measurement below.

The request answers from the snapshot, so unsaved edits are in it. A file that isn't a page or a fragment gets an empty answer, and the view says "Open a page to see what's known about it."

### The view

`ascribe.usedBy` becomes `ascribe.thisPage`, named **This page**, in the same place. Two sections, **About** and **Connections**, each collapsible. About's rows are a label and a description ("Address", "/keys/"). Connections' items open the link or include, as Used by's do, grouped by file then line. Its title says which heading narrows it.

The view asks once for each cursor move, after the same debounce Used by has, and only while it's visible.

### Measuring

On Ascribe's own docs, a cursor move answered by `ascribe/page` is timed against Used by's two requests for the same move, over the same 50 moves on a long page and a short one, and the numbers go in the pull request. If `ascribe/page` is slower, stop and report: decision 11 says what changes.

## Tasks

1. `ascribe/page` in `ascribe-lsp`, with tests of the server for a page, a heading, a fragment, a file that isn't a page, and an unsaved edit.
2. Its result type in `shapes.rs`, and the generated types blessed.
3. The view, its test API (`thisPage` in `AscribeApi.ui`, replacing Used by's part of `sidebar`), its unit tests, and `quill.it.ts` moved onto it.
4. The measurement.
5. The docs: the sidebar section of the editor's page, with **This page** and its two sections; the walkthrough's sidebar step; the README's sidebar line; `changes/editor/this-page.md`; and the surface map.

## Out of scope

- The rest of About, Health, and what the page includes (phase 4), Review (phase 2), and Memos (the memos plan).
- Any other view.

## Acceptance criteria

- The goal's steps, in `quill.it.ts`.
- After an unsaved edit to `install-agent.md` that adds a second link to "Rotate keys", This page on `keys.md` counts two links, before the edit is saved.
- A cursor move asks one request, not two.
- The measurement shows `ascribe/page` no slower than Used by's requests.
- No view, command, or setting is named Used by, in the manifest, the docs, the walkthrough, or the README.

## Verify

```sh
cargo test -p ascribe-lsp --locked
ASCRIBE_BLESS=1 cargo test -p ascribe-cli shapes && git diff --stat
pnpm --filter ascribe-vscode test
```

The integration tests run in CI.

## Commits

1. "Answer what's known about a page and the heading at the cursor"
2. "Show This page in place of Used by"
