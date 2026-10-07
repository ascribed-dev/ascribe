# Phase 3: The action registry

Part of [Editor UI](README.md). Requires phases 1 and 2. Extension only (`packages/vscode`).

## Goal

Every page action is defined once and reachable from the Command Palette, an **Ascribe** submenu in the editor's context menu, and, where it's a natural fix or refactor, the lightbulb. Actions that need input collect it in short quick-input steps. This phase is also the first time the extension applies an edit the server made. The actions bar (phase 4) will be one more consumer of this registry.

## Context

- `packages/vscode/src/extension.ts`: how commands are registered today, and the `AscribeApi` tests use.
- `packages/vscode/src/registry.ts`: `ProjectRegistry.serverFor(uri)`, which finds the server of the project that owns a file.
- `packages/vscode/src/client.ts`: `ProjectServer`, which owns the project's `LanguageClient`. Its `request(method, params)` returns the raw JSON; nothing converts a server's `WorkspaceEdit` today, because the language client applies the ones standard requests return.
- `packages/vscode/package.json`: `contributes.commands`, `contributes.menus` (`commandPalette` with `when` clauses on `ascribe.active`), and the context keys the extension sets (`ascribe.active`, `ascribe.previewOpen`, `ascribe.reviewOn`, `ascribe.devServer`). `ascribe.active` is true for the whole workspace once it has a project.
- Phase 1's `ascribe/context` and `ascribe/targets`, and phase 2's `ascribe/edit` (documented in `crates/ascribe-lsp/README.md`). Their result types are in `packages/vscode/src/shapes.ts`, generated; import `ContextResult`, `TargetsResult`, and the edit result from there.
- `packages/vscode/test/unit/docs.test.ts`: the commands table in the docs is generated from `package.json` and this file's `commands` descriptions (the README's rule).
- `packages/vscode/src/preview/controller.ts`: an example of quick picks (`selectPreviewBuild`).

## Design

### The registry

`src/actions/registry.ts`: a list of actions, each:

```ts
interface Action {
  id: string;                 // "wrapNote"; the command is `ascribe.action.wrapNote`
  title: string;              // "Wrap in a note"
  description: string;        // "Put the selection in an @note callout"
  group: "fix" | "write" | "structure" | "link" | "media";
  applies(context: ContextResult): boolean;   // from ascribe/context
  lightbulb?: "refactor" | "quickfix";  // offered by Cmd+. too
  ask?(context: ContextResult, targets: TargetsResult): Promise<Args | undefined>;  // the wizard; undefined = cancelled
  // What it does: an ascribe/edit operation, or the extension's own function.
  does: { operation: string } | { run(context: ContextResult, args: Args | undefined): Promise<void> };
}
```

One action per row of phase 2's table, with titles and descriptions written for writers: plain words first, the syntax in the description. `insertLink` picks a page or heading and `insertPhrase` picks from the project's phrases, showing each key with its value, so neither asks the writer to remember a path or a key. `insertSnippet` picks a source, a file, and a region.

One action sends no edit and uses `run`: **Copy a link to this section** (`copyLinkToSection`), on a heading. It copies the heading's destination from the content root (`/guides/install.md#install-cli`) to the clipboard, a form that works pasted into any page of the project. On a heading with no id of its own it copies the generated one. Phase 5's renames use `run` too.

### What the registry feeds

- **Palette:** `contributes.commands` has one command per action (`Ascribe: Wrap in a note`), shown when `ascribe.inProject && editorLangId == markdown`. `ascribe.inProject` is a new context key, true when the active editor's file belongs to a project (`serverFor` finds one), so a Markdown file outside every project lists no actions even when the workspace has a project. Run from the palette, a command asks the server for the context at the cursor first; if the action doesn't apply there, it says why ("Put the cursor in a note to change its type") instead of failing silently.
- **Context menu:** an **Ascribe** submenu (`contributes.submenus` and `menus["editor/context"]`) listing the actions. VS Code's `when` clauses can't evaluate `applies`, so set context keys from the latest `ascribe/context` for the cursor (debounced on selection change), such as `ascribe.at.note`, `ascribe.at.heading`, `ascribe.selection.prose`, and use them in `when`.
- **One cached context.** `src/actions/context.ts` holds the latest `ascribe/context` answer, keyed by document, version, and selection. The context keys and the lightbulb both read it, and only it sends the request.
- **Lightbulb:** a `CodeActionProvider` registered by the extension for Markdown in Ascribe projects. It returns the actions with a `lightbulb` kind that apply, each as a code action that runs the action's command. Kinds: `refactor.rewrite.ascribe` and `quickfix.ascribe`. VS Code calls a provider on every cursor move, so the provider never sends its own `ascribe/context`: it answers from the cache when the cache matches the document's version and the requested range, and with nothing when it doesn't yet (VS Code asks again). The server's own quick fixes for diagnostics stay as they are.
- **Wizards:** `src/actions/ask.ts`: shared steps built on `createQuickPick` and `createInputBox`: pick one, pick several, enter text (with validation), with a step counter ("2 of 3") and Back. Choices come from `ascribe/targets` (pages and headings for links, phrases, note types with labels, dimensions and their value labels, widgets and their attributes, fragments, images, snippets).

### Running an action

1. Get the context for the active editor's selection (`ascribe/context`).
2. If the action has `ask`, run the wizard; stop if it's cancelled.
3. An action with `run` runs it, and that's all. Otherwise, send `ascribe/edit` with the document's version through `ProjectServer.requestEdit`.
4. Apply the returned edit with `vscode.workspace.applyEdit`: one undo step in each file it changes (README decision 4). If the result has a `select` range, select it.
5. On an error result, show its message; on a stale version, get the context again and retry once.

### Converting the server's edit

Add `ProjectServer.requestEdit(method, params)` to `client.ts`. It sends the request and converts the result's `edit` with the language client's `protocol2CodeConverter.asWorkspaceEdit`, and its `select` with `asRange`, so the runner and every later phase get a `vscode.WorkspaceEdit` and never convert by hand. The converter stays private to `ProjectServer`.

## Tasks

1. `ProjectServer.requestEdit`, with a unit test of the conversion.
2. `src/actions/registry.ts`, `src/actions/ask.ts`, `src/actions/context.ts`, and `src/actions/run.ts` as above; register every action's command in `extension.ts`.
3. `package.json`: a command per action, the submenu, `editor/context` entries with context-key `when` clauses, and `commandPalette` entries.
4. `ascribe.inProject`, set from the active editor, and the context keys from the cursor's context, updated on selection change (debounced, cancelled when the selection moves again).
5. The lightbulb provider, answering from the cached context.
6. Unit tests (`test/unit/actions.test.ts`):
   - every registered action has a command in `package.json`, a `commandPalette` entry, and a context-menu entry (the rule in the README's decision 2);
   - `applies` for each action against sample contexts;
   - the wizard step builders;
   - the lightbulb provider sends no request: it answers from a matching cache and returns nothing for a stale one.
7. Integration tests (a new `test/integration/suite/actions.it.ts`, real server, a copy of `examples/quill`): run several actions through their commands with scripted wizard answers and check the resulting text, including one inside a list item, one whose placeholder ends up selected, one that the server rejects as stale, and **Copy a link to this section**. In `test/fixtures/monorepo`, a Markdown file outside every project has `ascribe.inProject` false.
8. A description for every new command in `docs.test.ts`'s `commands`, then bless the commands table. `docs/content/guides/editor.md`: a section on the actions, listing each with where it applies; a line in `CHANGELOG.md`.

## Out of scope

The actions bar and its key (phase 4); content-model actions (phase 5).

## Acceptance criteria

- Every action works from the palette, the context menu, and (for those marked) the lightbulb. Each of this phase's actions changes one file, and undoes in one step.
- Moving the cursor sends at most one `ascribe/context` per debounced selection change, whatever the lightbulb asks.
- The registry test fails if an action lacks any of its palette, menu, or command entries.
- Actions route to the owning project's server in a multi-project workspace.

## Verify

```sh
pnpm --filter ascribe-vscode typecheck
pnpm --filter ascribe-vscode test
cargo build -p ascribe-cli && ASCRIBE_BIN=$PWD/target/debug/ascribe pnpm --filter ascribe-vscode test:integration
pnpm lint && pnpm format:check
```

## Commits

1. "Define editor actions once, with wizards for their input"
2. "Offer the actions in the palette, the context menu, and the lightbulb"
