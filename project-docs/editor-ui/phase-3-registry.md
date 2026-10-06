# Phase 3: The action registry

Part of [Editor UI](README.md). Requires phases 1 and 2. Extension only (`packages/vscode`).

## Goal

Every page action is defined once and reachable from the Command Palette, an **Ascribe** submenu in the editor's context menu, and, where it's a natural fix or refactor, the lightbulb. Actions that need input collect it in short quick-input steps. The actions bar (phase 4) will be one more consumer of this registry.

## Context

- `packages/vscode/src/extension.ts`: how commands are registered today, and the `AscribeApi` tests use.
- `packages/vscode/src/registry.ts`: `ProjectRegistry.serverFor(uri)` and `ProjectServer.request(method, params)`, which send a request to the project that owns a file.
- `packages/vscode/package.json`: `contributes.commands`, `contributes.menus` (`commandPalette` with `when` clauses on `ascribe.active`), and the `ascribe.active` context key.
- Phase 1's `ascribe/context` and `ascribe/targets`, and phase 2's `ascribe/edit` (documented in `crates/ascribe-lsp/README.md`).
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
  applies(context: Context): boolean;   // from ascribe/context
  lightbulb?: "refactor" | "quickfix";  // offered by Cmd+. too
  ask?(context: Context, targets: Targets): Promise<Args | undefined>;  // the wizard; undefined = cancelled
  operation: string;          // the ascribe/edit action
}
```

One action per row of phase 2's table, with titles and descriptions written for writers: plain words first, the syntax in the description.

### What the registry feeds

- **Palette:** `contributes.commands` has one command per action (`Ascribe: Wrap in a note`), shown when `ascribe.active && editorLangId == markdown`. Run from the palette, a command asks the server for the context at the cursor first; if the action doesn't apply there, it says why ("Put the cursor in a note to change its type") instead of failing silently.
- **Context menu:** an **Ascribe** submenu (`contributes.submenus` and `menus["editor/context"]`) listing the actions. VS Code's `when` clauses can't evaluate `applies`, so set context keys from the latest `ascribe/context` for the cursor (debounced on selection change), such as `ascribe.at.note`, `ascribe.at.heading`, `ascribe.selection.prose`, and use them in `when`.
- **Lightbulb:** a `CodeActionProvider` registered by the extension for Markdown in Ascribe projects. It returns the actions with a `lightbulb` kind that apply, each as a code action that runs the action's command. Kinds: `refactor.rewrite.ascribe` and `quickfix.ascribe`. The server's own quick fixes for diagnostics stay as they are.
- **Wizards:** `src/actions/ask.ts`: shared steps built on `createQuickPick` and `createInputBox`: pick one, pick several, enter text (with validation), with a step counter ("2 of 3") and Back. Choices come from `ascribe/targets` (pages and headings for links, note types with labels, dimensions and their value labels, widgets and their attributes, fragments, images).

### Running an action

1. Get the context for the active editor's selection (`ascribe/context`).
2. If the action has `ask`, run the wizard; stop if it's cancelled.
3. Send `ascribe/edit` with the document's version.
4. Apply the returned `WorkspaceEdit` with `vscode.workspace.applyEdit` (one undo step). If the result has a placeholder range, select it.
5. On an error result, show its message; on a stale version, get the context again and retry once.

## Tasks

1. `src/actions/registry.ts`, `src/actions/ask.ts`, and `src/actions/run.ts` as above; register every action's command in `extension.ts`.
2. `package.json`: a command per action, the submenu, `editor/context` entries with context-key `when` clauses, and `commandPalette` entries.
3. Context keys from the cursor's context, updated on selection change (debounced, cancelled when the selection moves again).
4. The lightbulb provider.
5. Unit tests (`test/unit/actions.test.ts`):
   - every registered action has a command in `package.json`, a `commandPalette` entry, and a context-menu entry (the rule in the README's decision 2);
   - `applies` for each action against sample contexts;
   - the wizard step builders.
6. Integration tests (a new `test/integration/suite/actions.it.ts`, real server, a copy of `examples/quill`): run several actions through their commands with scripted wizard answers and check the resulting text, including one inside a list item and one that the server rejects as stale.
7. `docs/editor.md`: a section on the actions, listing each with where it applies; a line in `CHANGELOG.md`.

## Out of scope

The actions bar and its key (phase 4); content-model actions (phase 5).

## Acceptance criteria

- Every action works from the palette, the context menu, and (for those marked) the lightbulb, in a single undo step.
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
