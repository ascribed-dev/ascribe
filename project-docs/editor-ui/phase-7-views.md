# Phase 7: Used by, Pages, and Content model views

Part of [Editor UI](README.md). Requires phase 6. Server and extension.

## Goal

The rest of the sidebar, for the active file's project:

- **Used by:** what links to the active page (or a heading in it) and what includes the active fragment.
- **Pages:** the project's pages by type, its fragments with what includes them, and orphaned pages.
- **Content model:** phrases, features, glossary terms, dimensions, note types, widgets, and builds, each with how many pages use it, and a jump to where it's declared.

## Context

- `crates/ascribe-resolve`: the project's resolved links and includes (the snapshot already knows every link's target and every include's fragment, for diagnostics and rename).
- `crates/ascribe-lsp/src/refactor.rs`: rename finds every link to a page or id; the same search answers "used by".
- `crates/ascribe-lsp/src/complete.rs` and phase 1's `src/targets.rs`: lists of pages, fragments, and model entries.
- `crates/ascribe-model`: whether model entries keep their source spans in `ascribe.toml` (for "go to declaration"); `crates/ascribe-lsp/src/definition.rs` already jumps from a phrase or feature key to its entry, so the spans exist somewhere.
- Phase 6's view container and `src/ui/projectsView.ts`.

## Design

### Server

- **`textDocument/references`** (standard LSP, so other editors get it): for a page (cursor in its frontmatter or title), a heading or `@id`, a fragment, a phrase, a feature key, or a glossary term, every place it's used: links, includes, `{key}`, availability specs, glossary occurrences. Add the `references_provider` capability.
- **`ascribe/inventory`** (custom): one answer for the views, from the snapshot, cheap enough to ask on every save:

  ```jsonc
  {
    "pages": [{ "path": "…", "title": "…", "type": "guide", "incoming": 3 }],
    "fragments": [{ "path": "…", "includedBy": ["…"] }],
    "orphans": ["…"],           // pages no page links to and no navigation lists
    "model": [{ "kind": "phrase", "key": "product", "uses": 14, "declaration": { "uri": "…", "range": … } }]
  }
  ```

  An "orphan" is a page with no incoming links, other than the project's index pages. Say in the docs that, without a navigation file, this is a hint, not an error.

### Views

- **Used by** follows the active editor. For a page it lists incoming links grouped by page, then includes. For a fragment it lists the pages that include it. With the cursor on a heading, it narrows to links to that heading. Each item opens the location. It uses `textDocument/references`.
- **Pages** groups by type, then fragments, then orphans. It shows the title as the label and the path as the description, and each item opens the page.
- **Content model** has a node per kind with a count, then each entry with its use count. Clicking an entry goes to its declaration in `ascribe.toml`. Entries with no uses are marked, since those are candidates to remove.
- **Refresh:** Pages and Content model refresh on save and when the active project changes, debounced. They show the active file's project only (README decision 7: a page being active means its server is running).

## Tasks

1. `textDocument/references` and `ascribe/inventory` in the server, with scenario tests (including references across includes, and counts after an unsaved edit), documented in `crates/ascribe-lsp/README.md`.
2. The three views and their refresh rules in the extension, with unit tests for tree building and integration tests on a copy of `examples/quill` and on `test/fixtures/monorepo` (a nested project's views list only its own pages and model).
3. `docs/editor.md` and `CHANGELOG.md`.

## Out of scope

Editing from the views beyond opening locations; navigation files.

## Acceptance criteria

- Used by is right for pages, headings, and fragments, including links made through includes.
- Counts match a fresh `ascribe check`'s view of the project, and update after edits.
- Other LSP clients get `textDocument/references`.

## Verify

```sh
cargo test --workspace --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
pnpm --filter ascribe-vscode test
cargo build -p ascribe-cli && ASCRIBE_BIN=$PWD/target/debug/ascribe pnpm --filter ascribe-vscode test:integration
pnpm lint && pnpm format:check && cargo fmt --all --check
```

## Commits

1. "Find references to pages, headings, fragments, and model entries"
2. "Answer the project's inventory: ascribe/inventory"
3. "Add Used by, Pages, and Content model views"
