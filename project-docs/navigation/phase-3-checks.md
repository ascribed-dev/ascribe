# Phase 3: The file, checked

Part of [Navigation](README.md). Needs phase 2.

## Goal

A mistake in `ascribe.nav` is an error where it's written, an unplaced page is advice with a safe fix, and an orphan says what reaches it.

In `examples/astro-site`, list `pages = ["Options.md", "Optoins.md"]` under `Reference/`: `ascribe check` reports `nav-page-unknown` at `"Optoins.md"`, with a fix to `Options.md`. List `pages = ["Options.md"]` and add `Reference/Defaults.md`: `nav-page-unplaced` is advice on the new page, and `ascribe check --fix` appends it to the list. In VS Code, the same problems are underlined in `ascribe.nav` as it's typed, `pages` completes with the folder's pages, and the file's language is TOML, colored where a TOML extension is installed.

## Context

- README decisions 1, 2, 7, and 14, and phase 1's decisions 4 and 9.
- `crates/ascribe-check/src/permalinks.rs`: problems in a file beside `ascribe.toml`, and their places in it. `src/fix.rs` for safe fixes and `--fix`.
- `crates/ascribe-check/src/across.rs` and `page/mod.rs`: `page-orphan`, and its message and fix text in `tests/conformance/diagnostics.toml`.
- `crates/ascribe-lsp/src/model_file.rs` and `complete.rs`: diagnostics and completion in `ascribe.toml` and `ascribe.permalinks`; `packages/vscode/src/client.ts`'s document selector; `packages/vscode/package.json`'s `languages` (none for TOML files today).
- The agents' hook (`ascribe agents hook`), which applies safe fixes after an agent's edit.

## Design

### The checks

Each of phase 1's diagnostics, with its place: in `ascribe.nav` at the value for the file's own problems, and on the page for `nav-page-unplaced` and `section-page-both`. The fixes are edits to `ascribe.nav` that keep its comments and layout (`toml_edit`, as `ascribe.toml`'s edits are made). `nav-page-unplaced`'s fix appends in title order among the unplaced, so applying every one at once gives the order the default already shows (decision 2).

### `page-orphan`

As phase 1's decision 9 settles it. With its recommendation, the check reports what it does today, a page nothing links to or includes, and its message says what reaches the page instead of hedging: only the sidebar, for a page in the build's navigation, or nothing but its address, for one that's `unlisted`. The editor's Pages view stops saying a menu may reach it.

### In the editor

- `ascribe.nav` is sent to the server, as `ascribe.permalinks` is, and its problems are shown as it's typed.
- Completion: `folder` with the project's sections, `pages` with the folder's pages not yet listed, and the keys.
- The extension contributes `filenames` for `ascribe.nav`, `ascribe.permalinks`, and `ascribe.lock` to the `toml` language id (decision 1). VS Code has no TOML grammar of its own, so this colors them only with a TOML extension installed; the editor's page says so.

## Tasks

1. The diagnostics, in the registry and the diagnostics' file in order, each with a conformance case.
2. The fixes, with tests that `--fix` leaves the order a reader sees unchanged for `nav-page-unplaced` and `nav-section-unplaced`.
3. `page-orphan` by phase 1's decision 9, its messages, and its cases.
4. The language server: diagnostics, completion, and the client's selector; the `filenames` entry.
5. The docs: the navigation reference, each problem and what fixes it; the diagnostics reference (generated) and `page-orphan`'s entry; the editor page's files it checks as you type; `changes/language/navigation-checks.md`, `changes/editor/navigation-file.md`; and the surface map.

## Out of scope

- The Pages view (phase 5).
- Raising `nav-page-unplaced` to an error by default. A project does it as with any advice.

## Acceptance criteria

- The goal's steps, in conformance cases and an integration test.
- `ascribe check --fix` on a project with unplaced pages gives a navigation whose order is what it was before the fix.
- The agents' hook settles `nav-page-unplaced` after an agent adds a page.
- `page-orphan`'s message on a page in the navigation names the sidebar, and on an unlisted page says nothing reaches it, in conformance cases.
- With a TOML extension installed, `ascribe.nav` is colored; without one, its language is still TOML and its problems are shown.

## Verify

```sh
cargo test --workspace --locked
pnpm --filter ascribe-vscode test
```

## Commits

1. "Check ascribe.nav, and place an unplaced page with a safe fix"
2. "Report an orphan only when nothing reaches it"
3. "Check ascribe.nav as it's typed"
