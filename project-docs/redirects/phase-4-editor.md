# Phase 4: The editor

Part of [Redirects](README.md). Needs phase 2.

## Goal

The editor shows lost addresses between checks, and This page lists the old addresses that lead to a page.

A writer deletes a page in the Explorer. Within seconds, with no check run, the Project view's Health says "1 address lost", and the deleted page's lost addresses are in the Problems panel, as Check Every Build would put them. On `security/keys.md`, This page's About has "Also answers at: /docs/guides/keys".

## Context

- README decisions 4, 8, and 9; the proposal's "Where authors see it".
- The spike: the base's addresses are under 4 MB for 18,000, and change only when the base commit or Ascribe's version does.
- `crates/ascribe-lsp/src/review.rs`, which reads a base through `ascribe-diff` in the server today.
- `crates/ascribe-lsp/src/page.rs` and `page_rows.rs` for This page's rows; `packages/vscode/src/ui/projectHealth.ts` for the Project view's Health; `packages/vscode/src/ui/checkEveryBuild.ts` for how its problems are shown.

## Design

### The kept base

When `ascribe.addresses` exists, the server lists the base's addresses in the background, keyed by the base commit and Ascribe's version, and keeps them; a new base commit lists them again. With them, the server runs the comparison after each change, as Check Every Build would, and publishes `address-lost` with Check Every Build's problems, not as a page's diagnostics as you type (decision 8): the problems are the project's, and Check Every Build replaces them when it runs.

### The views

- **This page's About:** "Also answers at", each address an entry leads to the page from, from `redirects.json`'s `entry`, in the server's `ascribe/page` answer.
- **The Project view's Health:** addresses lost, with the count, opening the Problems panel's.
- **`init` offered:** a project without `ascribe.addresses`, after the editor moves a page, is offered **Keep the addresses you've published**, once, which runs `ascribe addresses init`.

### The editor's moves

Phase 1 records the Explorer's and Pages' moves through `refactor::will_rename`. This phase tests what's the editor's own: a move's entry is in the same undoable edit, an undo before saving takes it back with the move, and an undo after saving that leaves the entry is caught by `addresses-entry-live`.

## Tasks

1. The kept base and the comparison in the server, with tests: the base listed once per commit, a change compared without a check, and Check Every Build replacing the problems.
2. This page's row and the Project view's count, in the server's answers and the extension's views, with their tests.
3. `init` offered, with the extension's test.
4. The editor's moves and undo, in the extension's integration tests.
5. The docs: `docs/content/editor/sidebar.md`'s rows, and the redirects page's editor section; `changes/editor/redirects.md`.

## Out of scope

Review's report of the redirects a branch adds (the later proposal).

## Acceptance criteria

- The goal's walk-through, in the extension's integration test.
- The server lists the base's addresses once per base commit, in its test.

## Verify

```sh
cargo test -p ascribe-lsp addresses
pnpm --filter ascribe-vscode test
```

## Commits

1. "Keep the base's addresses in the language server, and report a lost one between checks"
2. "Show a page's old addresses in This page, and lost addresses in Health"
