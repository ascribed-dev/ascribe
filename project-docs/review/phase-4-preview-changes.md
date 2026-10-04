# Phase 4: Changes in the page preview

Part of [Review](README.md). Requires phases 1 to 3. Runs by itself, though phase 5 may still be in progress. Server and extension.

## Goal

In VS Code, turn on review for a project and the page preview marks what changed against the base, live as you edit. A list of changed pages lets the reviewer walk through them.

## Context

- `crates/tessera-lsp/src/preview.rs` (`ascribe/preview`: params, result, `document_version`) and `crates/tessera-lsp/src/core.rs` (the snapshot, and how it updates on edits).
- `tessera-diff` from phase 2: `GitFs` and the comparison.
- `packages/vscode/src/preview/` (`controller.ts`, `routing.ts`, `protocol.ts`, `html.ts`) and `packages/vscode/src/webview/preview.ts`.
- `packages/review/src/marks/` from phase 3.
- `packages/vscode/src/registry.ts`: `ProjectRegistry.serverFor`.
- [`mockup.html`](mockup.html): the reference for this phase's UI. Set **State** to "Not signed in to GitHub" to see the page preview with changes and no comments, which is what this phase delivers. Build from it: the header (base, change count with its breakdown, the show modes, next and previous with the position, and the offer of the next changed page after the last change); the **Changed Pages** quick pick and its title action; the "changed only through" link to the cause; the status bar's review item; and "Before review starts" for **Start Review** and its base prompt.

## Design

### Server

- **`ascribe/review/setBase`** (params: `{ base: string | null }`). The server resolves the revision (as `ascribe diff` does, including the merge base), loads the base project through `GitFs`, and keeps it beside the live snapshot. `null` drops it. The answer says what the base resolved to, or why it couldn't (not a repository, unknown revision, `git` missing). The base is loaded once and not watched; `setBase` again reloads it.
- **`ascribe/review/changes`** (params: `{ build? }`): the changed pages for the build, as phase 2's `pages` without `changes`, computed against the current snapshot, so it includes unsaved edits.
- **`ascribe/preview`** gains `review: true`. The page's result then includes its `changes` (phase 2's shape), and the HTML for its removed blocks, rendered from the base.
- The language server gains a dependency on `tessera-diff`, and so on `git` being on the path, only when a base is set. Without `git`, everything else works and `setBase` explains.
- Memory: a base roughly doubles a server's project data. Drop it when review is turned off, and record the measured cost on the synthetic 3,000-page project in the pull request.

### Extension

- **Ascribe: Start Review** asks for the base, offering the default branch's merge base first, then a revision the user types. It applies to the active file's project (never starting a server just for this: if the project's server isn't running, the command says to open a page first). **Ascribe: Stop Review** turns it off. The state is per project and lasts for the session.
- The preview shows a header while review is on: the base, the page's change count, the show toggle from phase 3 (**changes / as it will be / as it was**), and next and previous change with the position ("3 of 10 on this page"). Past the last change it offers the next changed page instead of wrapping. These are drawn in the preview because they act on the rendered page, and the site preview needs the same controls where there is no editor around it.
- Actions that don't act on the rendered page are native editor title actions on the preview panel (`menus["editor/title"]`, scoped to the preview with a `when` clause), as VS Code's guidelines prefer: **Changed Pages** here, and **Refresh** when phase 6 adds comments.
- The webview applies the marks with phase 3's script. Clicking a mark's source goes to the line in the editor.
- A status bar item shows review's state for the active file's project: "Review: off", or the base ("Review: #128 ← main" once phase 6 knows the pull request, the base's name before that). Clicking it opens **Start Review** when off and **Changed Pages** when on. It shows only while an Ascribe page or the preview is active.
- **Ascribe: Changed Pages** is a quick pick of the changed pages for the preview's build, each showing its counts and cause; choosing one opens the page and its preview. See the README for adding the same list to the Editor UI's Pages view if it exists.
- A page that changed only through something it uses says so in the header, with a link to the cause.

## Tasks

1. The three server changes, with scenario tests using a temporary repository: set a base, edit a buffer, and see the page's changes update; a fragment edit changing an including page; a base that fails.
2. The commands, the header, and the marks in the extension, with unit tests and an integration test on a copy of `examples/quill` made into a repository with one commit and one working-tree change.
3. Document the requests in `crates/tessera-lsp/README.md`, the commands in `docs/editor.md` and `docs/review.md`, and `CHANGELOG.md`.

## Out of scope

Comments (phases 5 and 6); choosing the base from a pull request (phase 6 sets it from the pull request).

## Acceptance criteria

- With review on, the preview's marks match `ascribe diff` for the same base, and update as the buffer changes.
- Turning review off removes the marks and frees the base.
- A project with no git repository gets a clear message and an otherwise working preview.

## Verify

```sh
cargo test --workspace --locked
pnpm --filter ascribe-vscode test
cargo build -p tessera-cli && ASCRIBE_BIN=$PWD/target/debug/ascribe pnpm --filter ascribe-vscode test:integration
pnpm lint && pnpm format:check && cargo fmt --all --check && cargo clippy --workspace --all-targets --locked -- -D warnings
```

## Commits

1. "Keep a review base in the language server"
2. "Mark changes in the page preview"
3. "List the pages a change touches"
