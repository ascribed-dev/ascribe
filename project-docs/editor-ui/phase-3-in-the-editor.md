# Phase 3: In the editor

Part of [The editor's views](README.md). Needs nothing; follows phase 2 by risk.

## Goal

A code file a page shows says which pages show it, and the safe fixes for a page are one action, or one save.

In VS Code, in `examples/tested`, open `code/examples/greet.ts`. Above the region `greet` is a lens, "Shown in 1 page"; choosing it lists `index.md`, and choosing that opens the page at its `@snippet`. In a page with a link to a route and an acknowledgement that covers nothing, **Source Action… → Apply the safe fixes in this page** fixes both and leaves the rest; with `"editor.codeActionsOnSave": { "source.fixAll.ascribe": "explicit" }`, saving does the same.

## Context

- README decisions 14 and 15.
- `crates/ascribe-lsp/src/links.rs` and `server.rs`, for the `@include` lens the server has, and how it registers its capabilities; `examples.rs` for what the server knows about a source's files and regions.
- `ascribe_resolve::Project::uses` and `ascribe refs`, for the pages a snippet's file or region is shown in.
- `packages/vscode/src/client.ts`: the document selector, which has the pages and Ascribe's TOML files, and no code files. `src/scope.ts`: the middleware that keeps a server from every file that belongs to another project or to none, `provideCodeLenses` among them, so a code file in `crates/`, in no project, reaches no server today.
- `crates/ascribe-lsp/src/code_action.rs`, and the `safe` applicability each fix carries; `ascribe check --fix` in `crates/ascribe-cli`, which applies them and settles two that edit the same text.
- The docs: `docs/content/examples/tested.md` and `@snippet` in `docs/content/writing/directives.md`, for the lens; the editor page's quick fixes, and `docs/content/checks/check.md`'s "Applying the safe fixes", for the action.

## Design

### "Shown in N pages"

After it starts, the server registers `textDocument/codeLens` dynamically, with a document selector of the project's sources' folders (`[sources.<name>]` paths and their `include`), and again when `ascribe.toml` changes them. The client then asks for lenses in those files, without sending their text: the server answers from the file on disk, read through `FileSystem`.

- **A region a `@snippet` shows** gets a lens on its start marker's line. **A whole file a `@snippet` shows** gets one on its first line.
- **"Shown in 3 pages"**, counting pages, a fragment counting as each page that includes it. Choosing it shows the places as Find All References lists them, in the references peek, and choosing one opens the page at its `@snippet`.
- **Placed from the saved file.** Lines added above a region leave the lens where the region was until the file's saved, and the server's file watching re-answers then. The editor's page says so.
- **A file in no source** gets nothing, and a source the server can't read gets nothing, with the reason in the server's log.
- **A file in several projects' sources** gets a lens from each (decision 14), each naming its project when there's more than one. The scope middleware lets a code-lens request for a code file through to every project whose registration covers it, whichever project the file's folder is in or none; for pages and Ascribe's files it keeps deciding as it does now.

### `source.fixAll.ascribe`

A code action of kind `source.fixAll.ascribe`, offered for a page or a fragment with at least one safe fix: every safe fix of the file's problems, in one edit, settled as `--fix` settles two that touch the same text, the first kept and the other left for the next run. Titled **Apply the safe fixes in this page**. The actions bar lists it first among its fixes when the page has two or more.

On save, only through `editor.codeActionsOnSave`. Nothing sets it, and the editor's page shows the setting.

## Tasks

1. The registration for the sources' folders, re-registered when the model changes, with a test of the server that a code file in a source gets a lens and one outside gets none; and the scope middleware's rule for code files, with unit tests for a file in no project, in another project's folder, and in two projects' sources.
2. The lens and its command, and the references peek it opens, with tests of the server for a region, a whole file, a fragment's snippet, and a file in two pages.
3. The fix-all action, with tests of the server: the same fixes as `check --fix` on the same file, and two that overlap.
4. The action in the actions bar.
5. An integration test for each, in `examples/tested` and `examples/quill`.
6. The docs: the editor page (a section for code files a page shows, with "placed from the saved file", and fix-all with the setting, both marked `@available: next`); `examples/tested.md` and the `@snippet` reference linking it; "Applying the safe fixes" naming the editor's action; "Other editors" saying both come from the server; `changes/language-server/snippet-lens.md` and `changes/language-server/fix-all.md`; and the surface map.

## Out of scope

- Keeping the lens in place while a code file is edited (decision 14).
- Lenses in code files for anything but snippets.
- Unsafe fixes in the fix-all action.

## Acceptance criteria

- The goal's steps, in integration tests.
- `ascribe check --fix` and the fix-all action make the same edits to the same file, in a test of the server.
- A code file outside every source asks the server nothing.
- A code file in two projects' sources has two lenses, each naming its project, in an integration test with a fixture of two projects whose sources share a folder.

## Verify

```sh
cargo test -p ascribe-lsp --locked
pnpm --filter ascribe-vscode test
```

The integration tests run in CI.

## Commits

1. "Say which pages show a code file, in the code file"
2. "Apply a page's safe fixes in one action"
