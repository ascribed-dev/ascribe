# Editor UI

Writer-friendly UI for the VS Code extension: an actions bar that offers what applies where the cursor is, the actions behind it, a status bar item, sidebar views, a build lens, and a getting-started walkthrough. This plan comes from section 4 of the [brainstorm](../brainstorm.md#4-editor-ui-an-actions-bar-a-sidebar-and-more).

## Who it's for

Writers more than power users. Many Ascribe authors would rather not remember directive syntax (`@note {type=tip}`, `@variant {pm=npm}:`, `@available: cloud, self-hosted beta 2.4`). Today the editor checks what they write and completes some of it, but it doesn't help them *write* it. This work adds that.

## Decisions

These are settled. Don't reopen them in a phase; if one can't be met, stop and report.

1. **The actions bar is context-sensitive actions only.** One key opens a quick pick (`vscode.window.createQuickPick`) listing what applies to the cursor or selection, with fixes for problems at the cursor first. No prefix modes, no project-wide search, nothing beyond the editor (no running checks or builds, no opening the published site).
2. **Every action has another path.** Each action is defined once, in one registry. The registry feeds the Command Palette (one command per action), an **Ascribe** submenu in the editor's context menu, the lightbulb (`Cmd+.`) where the action is a natural fix or refactor, and the actions bar. A test fails if a registered action has no palette command.
3. **Writers first.** Each action has a plain-language title ("Wrap in a note") and a one-line description that names the syntax (`@note`). Actions that need input ask for it in short steps in the same quick input ("Which dimension?", "Which values?"), never by expecting syntax.
4. **The server works out context and makes edits; the extension owns the UI.** The language server already parses every page and knows the content model and canonical form, so it answers "what's at this position?" and "what's the edit for this action?". The extension owns the registry, menus, wizards, views, and decorations. Edits travel as LSP `WorkspaceEdit`s, so undo, the dirty marker, and git behave as for typing. An action undoes in one step per file: an action that changes one file is one undo, and one that changes several (a phrase declared in `ascribe.toml`, a rename) is one undo in each file it changed, which is how VS Code undoes any edit across files.
5. **Edits are canonical.** The text an action writes is in canonical form: formatting the page afterwards (`ascribe fmt`) changes nothing in the edited range.
6. **`ascribe.toml` keeps its comments.** Actions that change the content model edit it in place with a round-trip TOML editor, never by re-serializing it.
7. **Multi-project aware.** Everything goes to the server of the project that owns the file (`ProjectRegistry.serverFor`). Views that list projects show every project in the workspace, started or not, and never start a server just to fill a view.
8. **The sidebar is views, not forms.** Tree views (Projects, Used by, Pages, Content model), not a form editor for `ascribe.toml`.
9. **A rename is a rename.** Renaming a phrase key or a dimension value goes through `textDocument/rename`, so F2, other editors, and the registry's actions share one implementation. `ascribe/edit` has no rename operations.
10. **One build you're looking at.** Each project has one chosen build, for the session: the preview renders it, the build lens dims by it, and the status bar names it. It starts as the editor build (`[editor] build`), which still decides the diagnostics. The preview's picker, the status bar, and the lens all change the same choice.

## Phases

Do them in order. Each phase leaves the repository green and can be its own pull request.

| Phase | Result |
|---|---|
| [1: Context and targets](phase-1-context.md) | The server answers what's at a position (`ascribe/context`) and what an action can point at (`ascribe/targets`). |
| [2: Page edits](phase-2-edits.md) | The server makes each page action's edit (`ascribe/edit`), in canonical form. |
| [3: The action registry](phase-3-registry.md) | Every page action is a palette command, a context-menu item, and (where natural) a lightbulb action, with wizards for input. |
| [4: The actions bar](phase-4-actions-bar.md) | One key opens the context-sensitive menu. |
| [5: Content-model actions](phase-5-model-actions.md) | Make a phrase, add a glossary term, and promote a feature, editing `ascribe.toml` in place; rename a phrase key or a dimension value. |
| [6: Status bar and Projects view](phase-6-status-projects.md) | The active file's project and the build you're looking at in the status bar, with a way to switch it; an Ascribe sidebar with the Projects view. |
| [7: Used by, Pages, Content model](phase-7-views.md) | The rest of the sidebar, with the server requests behind it. |
| [8: The build lens](phase-8-build-lens.md) | The editor dims what a chosen build leaves out. |
| [9: Walkthrough and release notes](phase-9-walkthrough.md) | A getting-started walkthrough, and the docs reviewed as a whole. |

## Rules for every phase

- Branch before committing; never commit to `main`.
- Read the current code before the phase file's pointers: line numbers drift. If the phase file and the code disagree, or a decision above can't be met, stop and report instead of choosing silently.
- Match the surrounding code's style, comment density, and naming. Libraries don't panic on user input; `unwrap` and `expect` are linted.
- **Server work:** a new request gets its own module like `crates/ascribe-lsp/src/preview.rs` (a `METHOD` constant, serde parameter and result types), a handler in `server.rs`, scenario tests in `crates/ascribe-lsp/tests/` (the in-process harness in `tests/support/mod.rs`), and a section in `crates/ascribe-lsp/README.md` documenting the request for other clients. Requests answer from the current snapshot, so they include unsaved edits.
- **A request's result type is generated, not retyped.** Derive `JsonSchema` on it behind the crate's `json-schema` feature, as `PreviewResult` does, add it to `SHAPES` in `crates/ascribe-cli/src/shapes.rs`, and run `ASCRIBE_BLESS=1 cargo test -p ascribe-cli shapes`. That writes its JSON Schema in `schemas/`, its TypeScript type in `packages/vscode/src/shapes.ts`, and its docs fragment. The extension imports the type from `shapes.ts` and never declares its own copy.
- **Project files are read through `ascribe_resolve::FileSystem`.** A direct `std::fs` read of one fails `crates/ascribe-resolve/tests/file_reads.rs`.
- **Extension work:** unit tests with vitest (`packages/vscode/test/unit/`) for anything that doesn't need VS Code; integration tests (`packages/vscode/test/integration/suite/`) for what does, against the real server. `test/fixtures/monorepo` is the multi-project fixture; `examples/monorepo` is for trying things by hand (**Run and Debug → Extension: several projects**).
- **User-visible changes** update `docs/content/guides/editor.md` and add a line to the unreleased section of `CHANGELOG.md` in the same phase. The pages under `docs/content/` are the docs; the files directly in `docs/` are "Moved" stubs.
- **The commands and settings tables are generated.** `docs/content/_generated/editor-commands.md` and `editor-settings.md` come from `packages/vscode/package.json` and the `commands` descriptions in `packages/vscode/test/unit/docs.test.ts`. A new command or setting gets its description there, then `ASCRIBE_BLESS=1 pnpm --filter ascribe-vscode test` rewrites the tables. Read the diff; never edit the generated files.
- Tests must be correct on Windows: no hard-coded `/` in filesystem paths, `file:///C:/…` URIs with three slashes, and drive-letter case folded when comparing.
- No phase history in code or docs. Describe what the code does now.
- Before finishing a phase, all of these pass:

  ```sh
  cargo fmt --all --check
  cargo clippy --workspace --all-targets --locked -- -D warnings
  cargo test --workspace --locked
  pnpm format:check && pnpm lint && pnpm typecheck && pnpm test
  cargo build -p ascribe-cli && ASCRIBE_BIN=$PWD/target/debug/ascribe pnpm --filter ascribe-vscode test:integration
  ```

  (`corepack pnpm` where `pnpm` isn't on the path.)

## Assumptions

- **Versions.** The extension requires VS Code 1.138 and uses `vscode-languageclient` 10.1 (`packages/vscode/package.json`). Both have `workspace.applyEdit`'s `isRefactoring` and change annotations with `needsConfirmation`. The server uses `lsp-types` 0.97, which has no `SnippetTextEdit` (it's still a proposed LSP feature), so no edit in this plan is a snippet edit.
- **The extension has never applied a server's edit itself.** Today the language client applies them, and `ProjectServer.request` (`packages/vscode/src/client.ts`) returns raw JSON. Phase 3 adds the conversion.
- **The content model keeps no source spans.** "Go to declaration" finds an entry by searching `ascribe.toml`'s text (`find_entry` in `crates/ascribe-lsp/src/definition.rs`).

## The agents plan

The [agents plan](../agents/README.md) overlaps this one in three places. Whichever plan reaches one first builds it as described here, and the other reuses it.

- **Where things are used.** This plan's `textDocument/references` and `ascribe/inventory` (phase 7) and the agents plan's `ascribe refs` (its phase 2) are one search. It lives in a crate below the language server (`ascribe-resolve`, unless reading the code shows `ascribe-check` fits better), moved or written there in its own commit with no behavior change. The server and the command both call it, and a test checks that they agree on the same target.
- **What a project has.** `ascribe/targets` (phase 1) and the agents plan's `ascribe model` and `ascribe outline` list the same pages, headings, and model entries. They share the functions that build those lists, in the same crate, and differ only in what they print.
- **When servers start.** The agents plan's phase 3 starts a project's server when one of its files changes on disk. That doesn't break decision 7, which is about views. Phases 6 and 7 test decision 7 as "showing or refreshing a view starts no server", not as "no server runs until a file is opened", so the tests hold with either plan first.

## The visual design plan

The [visual design plan](../visual-design/README.md) makes the mark, the activity bar icon, and the rules for icons and colors in the extension (codicons and theme colors first). Phases 1 to 5 here need nothing from it. Phase 6 needs its activity bar icon, phase 7's views follow its icon rules, and phase 9's walkthrough uses its mark.

## Later, not in this plan

- Structural editing in the preview (brainstorm section 2) will reuse phase 2's edits and phase 3's registry: a click in the preview becomes an action.
- An MCP server or agent tools (brainstorm sections 9 and 10) can expose `ascribe/context`, `ascribe/targets`, and `ascribe/edit` to agents.
- A form editor for `ascribe.toml`.
- Removing a table row's availability, or changing a `@snippet`'s address or attributes, from an action. Phase 2 adds both; editing them afterwards is by hand.
