# Phase 7: Used by, Pages, and Content model views

Part of [Editor UI](README.md). Requires phase 6. Server and extension.

## Goal

The rest of the sidebar, for the active file's project:

- **Used by:** what links to the active page (or a heading in it) and what includes the active fragment.
- **Pages:** the project's pages by type, its fragments with what includes them, and orphaned pages.
- **Content model:** phrases, features, glossary terms, dimensions, note types, widgets, and builds, each with how many pages use it, and a jump to where it's declared.

## Context

- `crates/ascribe-resolve`: the project's resolved links and includes (the snapshot already knows every link's target and every include's fragment, for diagnostics and rename).
- `crates/ascribe-lsp/src/refactor.rs`: rename finds every link to a page or id, and every use of a phrase (extended in phase 5); the same search answers "used by".
- `crates/ascribe-lsp/src/complete.rs` and phase 1's `src/targets.rs`: lists of pages, fragments, and model entries.
- `crates/ascribe-lsp/src/definition.rs`: the model keeps no source spans, so `find_entry` finds an entry by searching `ascribe.toml`'s text. It covers phrases and features, and dimension values since phase 1. The inventory's declarations reuse it, extended to glossary terms, dimensions, note types, widgets, and builds. Don't add spans to `ascribe-model`.
- [The agents plan](README.md#the-agents-plan): the search behind references and the inventory is shared with `ascribe refs`, and lives in a crate below the language server. If the agents plan's phase 2 has landed, call what it built; if not, build it there in this phase.
- `crates/ascribe-lsp/benches/keystroke.rs` and `completion.rs`: the existing benchmarks, each a plain `main` over the shared synthetic generator (`ascribe-synthetic`).
- Phase 6's view container and `src/ui/projectsView.ts`.

## Design

### Server

- **`textDocument/references`** (standard LSP, so other editors get it): for a page (cursor in its frontmatter or title), a heading or `@id`, a fragment, a phrase, a feature key, or a glossary term, every place it's used: links, includes, `{key}`, availability specs, glossary occurrences. Add the `references_provider` capability.
- **`ascribe/inventory`** (custom): one answer for the views, from the snapshot. The views ask on every save, so it has a budget: a median under 50 ms on the 3,000-page synthetic project, measured by a new `benches/inventory.rs` beside the existing ones. If it can't be met by counting from the snapshot's index, stop and report instead of caching counts a second way.

  ```jsonc
  {
    "pages": [{ "path": "…", "title": "…", "type": "guide", "incoming": 3 }],
    "fragments": [{ "path": "…", "includedBy": ["…"] }],
    "orphans": ["…"],           // pages no page links to and no navigation lists
    "model": [{ "kind": "phrase", "key": "product", "uses": 14, "declaration": { "uri": "…", "range": … } }]
  }
  ```

  `ascribe/inventory` and `ascribe refs` share one implementation: the count of a target's uses is the length of the list `textDocument/references` returns for it.

  An "orphan" is a page with no incoming links, other than the project's index pages. Say in the docs that, without a navigation file, this is a hint, not an error.

### Views

- **Used by** follows the active editor. For a page it lists incoming links grouped by page, then includes. For a fragment it lists the pages that include it. With the cursor on a heading, it narrows to links to that heading. Each item opens the location. It uses `textDocument/references`.
- **Pages** groups by type, then fragments, then orphans. It shows the title as the label and the path as the description, and each item opens the page.
- **Content model** has a node per kind with a count, then each entry with its use count. Clicking an entry goes to its declaration in `ascribe.toml`. Entries with no uses are marked, since those are candidates to remove.
- **Refresh:** Pages and Content model refresh on save and when the active project changes, debounced. They show the active file's project only (README decision 7: a page being active means its server is running). No view starts a server to fill itself.

## Tasks

1. The shared search, in the crate below the server, in its own commit with no behavior change to rename (skip if the agents plan built it).
2. `textDocument/references` and `ascribe/inventory` in the server, with scenario tests (including references across includes, and counts after an unsaved edit), documented in `crates/ascribe-lsp/README.md`. Derive `JsonSchema` on the inventory's result, add it to `SHAPES`, and bless.
3. `benches/inventory.rs`, with its numbers in the pull request.
4. The three views and their refresh rules in the extension, with unit tests for tree building and integration tests on a copy of `examples/quill` and on `test/fixtures/monorepo` (a nested project's views list only its own pages and model; showing a view starts no server).
5. `docs/content/guides/editor.md` (its "Navigation" section gains Find All References) and `CHANGELOG.md`.

## Out of scope

Editing from the views beyond opening locations; navigation files.

## Acceptance criteria

- Used by is right for pages, headings, and fragments, including links made through includes.
- Counts are right and update after edits. `ascribe check` reports no use counts, so the test compares each count with one made independently from the resolved snapshot: for every example project, the number of links, includes, and `{key}`s the resolver recorded for each target. Once the agents plan's `ascribe refs` exists, a test also checks that it and **Find All References** list the same places.
- The inventory meets its budget.
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

1. "Move the search for uses below the language server" (if this phase builds it)
2. "Find references to pages, headings, fragments, and model entries"
3. "Answer the project's inventory: ascribe/inventory"
4. "Add Used by, Pages, and Content model views"
