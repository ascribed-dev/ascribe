# Editor UI

Writer-friendly UI for the VS Code extension: an actions bar that offers what applies where the cursor is, the actions behind it, a status bar item, sidebar views, a build lens, and a getting-started walkthrough. This plan comes from section 4 of the [brainstorm](../brainstorm.md#4-editor-ui-an-actions-bar-a-sidebar-and-more).

## Who it's for

Writers more than power users. Many Ascribe authors would rather not remember directive syntax (`@note {type=tip}`, `@variant {pm=npm}:`, `@available: cloud, self-hosted beta 2.4`). Today the editor checks what they write and completes some of it, but it doesn't help them *write* it. This work adds that.

## Decisions

These are settled. Don't reopen them in a phase; if one can't be met, stop and report.

1. **The actions bar is context-sensitive actions only.** One key opens a quick pick (`vscode.window.createQuickPick`) listing what applies to the cursor or selection, with fixes for problems at the cursor first. No prefix modes, no project-wide search, nothing beyond the editor (no running checks or builds, no opening the published site).
2. **Every action has another path.** Each action is defined once, in one registry. The registry feeds the Command Palette (one command per action), an **Ascribe** submenu in the editor's context menu, the lightbulb (`Cmd+.`) where the action is a natural fix or refactor, and the actions bar. A test fails if a registered action has no palette command.
3. **Writers first.** Each action has a plain-language title ("Wrap in a note") and a one-line description that names the syntax (`@note`). Actions that need input ask for it in short steps in the same quick input ("Which dimension?", "Which values?"), never by expecting syntax.
4. **The server works out context and makes edits; the extension owns the UI.** The language server already parses every page and knows the content model and canonical form, so it answers "what's at this position?" and "what's the edit for this action?". The extension owns the registry, menus, wizards, views, and decorations. Edits travel as LSP `WorkspaceEdit`s, so undo, the dirty marker, and git behave as for typing, and an action undoes in one step.
5. **Edits are canonical.** The text an action writes is in canonical form: formatting the page afterwards (`ascribe fmt`) changes nothing in the edited range.
6. **`ascribe.toml` keeps its comments.** Actions that change the content model edit it in place with a round-trip TOML editor, never by re-serializing it.
7. **Multi-project aware.** Everything goes to the server of the project that owns the file (`ProjectRegistry.serverFor`). Views that list projects show every project in the workspace, started or not, and never start a server just to fill a view.
8. **The sidebar is views, not forms.** Tree views (Projects, Used by, Pages, Content model), not a form editor for `ascribe.toml`.

## Phases

Do them in order. Each phase leaves the repository green and can be its own pull request.

| Phase | Result |
|---|---|
| [1: Context and targets](phase-1-context.md) | The server answers what's at a position (`ascribe/context`) and what an action can point at (`ascribe/targets`). |
| [2: Page edits](phase-2-edits.md) | The server makes each page action's edit (`ascribe/edit`), in canonical form. |
| [3: The action registry](phase-3-registry.md) | Every page action is a palette command, a context-menu item, and (where natural) a lightbulb action, with wizards for input. |
| [4: The actions bar](phase-4-actions-bar.md) | One key opens the context-sensitive menu. |
| [5: Content-model actions](phase-5-model-actions.md) | Make a phrase, add a glossary term, promote a feature, rename a phrase key or dimension value, editing `ascribe.toml` in place. |
| [6: Status bar and Projects view](phase-6-status-projects.md) | The active file's project and build in the status bar; an Ascribe sidebar with the Projects view. |
| [7: Used by, Pages, Content model](phase-7-views.md) | The rest of the sidebar, with the server requests behind it. |
| [8: The build lens](phase-8-build-lens.md) | The editor dims what a chosen build leaves out. |
| [9: Walkthrough and release notes](phase-9-walkthrough.md) | A getting-started walkthrough, and the docs reviewed as a whole. |

## Rules for every phase

- Branch before committing; never commit to `main`.
- Read the current code before the phase file's pointers: line numbers drift. If the phase file and the code disagree, or a decision above can't be met, stop and report instead of choosing silently.
- Match the surrounding code's style, comment density, and naming. Libraries don't panic on user input; `unwrap` and `expect` are linted.
- **Server work:** a new request gets its own module like `crates/tessera-lsp/src/preview.rs` (a `METHOD` constant, serde parameter and result types), a handler in `server.rs`, scenario tests in `crates/tessera-lsp/tests/` (the in-process harness in `tests/support/mod.rs`), and a section in `crates/tessera-lsp/README.md` documenting the request for other clients. Requests answer from the current snapshot, so they include unsaved edits.
- **Extension work:** unit tests with vitest (`packages/vscode/test/unit/`) for anything that doesn't need VS Code; integration tests (`packages/vscode/test/integration/suite/`) for what does, against the real server. `test/fixtures/monorepo` is the multi-project fixture; `examples/monorepo` is for trying things by hand (**Run and Debug → Extension: several projects**).
- **User-visible changes** update `docs/editor.md` and add a line to the unreleased section of `CHANGELOG.md` in the same phase.
- Tests must be correct on Windows: no hard-coded `/` in filesystem paths, `file:///C:/…` URIs with three slashes, and drive-letter case folded when comparing.
- No phase history in code or docs. Describe what the code does now.
- Before finishing a phase, all of these pass:

  ```sh
  cargo fmt --all --check
  cargo clippy --workspace --all-targets --locked -- -D warnings
  cargo test --workspace --locked
  pnpm format:check && pnpm lint && pnpm typecheck && pnpm test
  cargo build -p tessera-cli && ASCRIBE_BIN=$PWD/target/debug/ascribe pnpm --filter ascribe-vscode test:integration
  ```

  (`corepack pnpm` where `pnpm` isn't on the path.)

## Later, not in this plan

- Structural editing in the preview (brainstorm section 2) will reuse phase 2's edits and phase 3's registry: a click in the preview becomes an action.
- An MCP server or agent tools (brainstorm sections 9 and 10) can expose `ascribe/context`, `ascribe/targets`, and `ascribe/edit` to agents.
- A form editor for `ascribe.toml`.
