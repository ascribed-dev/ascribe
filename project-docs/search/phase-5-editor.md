# Phase 5: In the editor

Part of [Search](README.md). Needs phase 3; follows phase 4 by value.

## Goal

**Search Pages as Readers See Them** finds words as you type, in unsaved text, and opens the line.

In VS Code, in `examples/quill`, run **Ascribe: Search Pages as Readers See Them** and type `sign in to quill cloud` in quotes. One result: the reader's text with the match marked, and under it `install-agent.md:62 · Connect to Quill · through phrase cloud`. Choose it: `docs/install-agent.md` opens at line 62. Type `quill cloud` in quotes: the result opens `ascribe.toml` at the phrase `cloud`. Edit a page without saving, and search again: the unsaved words are found. Switch the status bar's build to `cloud`: the search covers that build; the quick pick's button searches every build.

## Context

- README decisions 5, 9 to 11, 15, 16, and 20. The proposal's "In the editor".
- "A new editor command, action, or setting" in `project-docs/checklists.md`.
- `crates/ascribe-lsp/src/targets.rs` and `inventory.rs`: a custom request answered from the current snapshot, its `METHOD`, and its types through `crates/ascribe-cli/src/shapes.rs` to `packages/vscode/src/shapes.ts`.
- `crates/ascribe-lsp/src/compute.rs`: `ResolvedCache`, the editor's build's resolved pages, kept as the project changes; `build_view.rs`, how the server answers for a build other than the editor's.
- `packages/vscode/src/ui/chosenBuild.ts` (`ChosenBuilds`): the build the preview renders and the status bar names. `src/ui/switchProject.ts` and `src/actions/barPick.ts`: the extension's quick picks.
- `tests/corpora/baselines/perf.json`'s `lsp/` metrics and their 50 ms targets.

## Design

### The request

`ascribe/search`, with `{ textDocument, query, build?, limit? }`: `textDocument` is any file of the project, which says which project; `build` is one build, or absent for every build. Its result is `{ places, total, truncated }`, each place `{ uri, range, in, text, match, through, arm, pages, pagesTotal }`, found by `ascribe_resolve::reading::find`, the function the command's entry calls, so the server and the command can't disagree. `range` is the match's start to the end of its line in the source, in the protocol's UTF-16 positions; `match` is in characters of `text`, as the JSON's is.

The server answers from its current snapshot, so unsaved text is searched. It keeps each build's reading of each page with the snapshot, made when a search first asks and updated as pages change, as `ResolvedCache` is, so a query as you type is the scan alone.

### The command

**Ascribe: Search Pages as Readers See Them** (`ascribe.search`), in the Command Palette while a project is open. It opens a quick pick that asks the server as you type, debounced, and shows:

- **each place's label:** the reader's text, cut around the match, with the match highlighted;
- **its description:** the file and line, the heading, and `through …`, as the command's second line;
- **its detail,** for a place many pages show: `on 30 pages`.

Choosing one opens the source at the place's range; a place in `ascribe.toml` opens the phrase's line. The quick pick searches the build `ChosenBuilds` has for the project, named in its placeholder, and its button, **Search Every Build**, searches every build. Nothing matched shows the command's last line, from phase 3, as the quick pick's only item.

The actions bar is unchanged: it holds what applies at the cursor, and this doesn't.

### A budget

`lsp/search-3000` joins the corpora's metrics: a query on the synthetic project with its reading held, target 50 ms, as the other `lsp/` requests have.

## Tasks

1. `ascribe/search` in the server, its types in `shapes.rs`, the extension's `src/shapes.ts` regenerated, and tests of the server: unsaved text, a build other than the editor's, every build, and a reading updated after an edit.
2. The readings kept with the snapshot.
3. The command and its quick pick, with unit tests of what each item shows and an integration test of the goal's steps.
4. The metric and its target.
5. The docs: a "Search" section in `docs/content/editor/writing.md`, `@available: next`, what it searches and how it differs from VS Code's own search; the generated tables of commands (`ASCRIBE_BLESS=1 pnpm --filter ascribe-vscode exec vitest run test/unit/docs.test.ts`); the extension's README where it lists what it does; `changes/editor/search.md`; and the surface map.

## Out of scope

- A search view in the sidebar (decision 16).
- Replace.

## Acceptance criteria

- The goal's steps, in an integration test.
- `lsp/search-3000` within its target on the runner.
- `node scripts/compare/outputs.ts --base main` shows every project byte for byte the same.

## Verify

```sh
cargo test -p ascribe-lsp search
ASCRIBE_BLESS=1 cargo test -p ascribe-cli shapes   # then read the diff
pnpm --filter ascribe-vscode test
node scripts/compare/outputs.ts --base main
```

The integration tests run in CI.

## Commits

1. "Answer ascribe/search from the editor's snapshot"
2. "Search pages as readers see them from the Command Palette"
