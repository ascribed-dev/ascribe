# Phase 4: The actions bar

Part of [Editor UI](README.md). Requires phase 3. Extension only.

## Goal

One key opens a quick pick listing only the actions that apply to the cursor or selection, fixes first, for writers.

## Context

- Phase 3's registry (`src/actions/registry.ts`), runner (`src/actions/run.ts`), and context keys.
- `vscode.window.createQuickPick`: items with `label`, `description`, `detail`, `iconPath`; `kind: QuickPickItemKind.Separator` for groups; `matchOnDescription`; `busy`.
- `vscode.commands.executeCommand("vscode.executeCodeActionProvider", uri, range, "quickfix")`: the server's quick fixes for the problems at the cursor.

## Design

- **Command:** `ascribe.actions` ("Ascribe: Actions for the cursor"), in the palette and the context menu, and bound to a key (below).
- **Opening:**
  1. Show the quick pick at once with `busy`, so the key feels instant.
  2. Ask the server for the context (`ascribe/context`) and VS Code for the quick fixes at the cursor, in parallel.
  3. Fill the list. If the context request fails (the server isn't running), say so in one item that opens the project's output.
- **Items, in this order, with separators:**
  1. **Fix** (only when there are problems at the cursor): each quick fix, titled as the server titles it, with the diagnostic's message as the detail line.
  2. The registry's groups that have applicable actions: **Write** (notes, details, steps), **Structure** (variant groups, arms, ids, availability), **Link** (links, includes), **Media** (images, widgets).
- **Each item:** the action's title as the label, its description, and a `detail` line previewing what it writes when that's short and known before the wizard (`@note {type=tip}`, `@steps`, `@id: install-cli`).
- **Running:** choosing an item runs the action through phase 3's runner; its wizard continues in the same quick input, so the bar never closes and reopens between steps.
- **Empty:** if nothing applies, one disabled item says what to try ("Select some text, or put the cursor on a heading, note, link, or image").
- **Nothing else:** no search modes, no project-wide actions, nothing outside the editor (README decision 1).

### The key

Pick a default that:

- is unbound in VS Code's default keymaps on macOS, Windows, and Linux, checked against the current defaults (**Preferences: Open Default Keyboard Shortcuts (JSON)**) and listed in the PR;
- is easy to reach and to remember, such as `Cmd+Alt+A` / `Ctrl+Alt+A`, or a chord such as `Cmd+K A`;
- is bound only when `editorTextFocus && ascribe.active && editorLangId == markdown`.

`Cmd+.` is the lightbulb and stays VS Code's. Document the key, and how to change it, in `docs/editor.md`.

## Tasks

1. `src/actions/bar.ts` with the behavior above; register `ascribe.actions` and its keybinding.
2. Unit tests: item ordering, groups, separators, and the empty state, from sample contexts and quick fixes.
3. Integration tests (extend `actions.it.ts`):
   - the bar lists exactly the applicable actions for a cursor on a note, on a heading, on a selection of prose, and on a blank line;
   - a problem at the cursor puts its fix first;
   - choosing an action with a wizard completes it in the same quick input;
   - in `test/fixtures/monorepo`, the bar in a nested project's page offers that project's note types and dimensions, not the parent's.
4. `docs/editor.md`: the bar, its key, and how it relates to the palette, the context menu, and `Cmd+.`; a line in `CHANGELOG.md`.

## Out of scope

Content-model actions (phase 5), which join the bar when they exist through the same registry.

## Acceptance criteria

- The key opens the bar in an Ascribe page, and does nothing elsewhere.
- The list is right for each context in the tests, fixes first.
- Every item in the bar is also a palette command (enforced by phase 3's test).

## Verify

```sh
pnpm --filter ascribe-vscode typecheck
pnpm --filter ascribe-vscode test
cargo build -p ascribe-cli && ASCRIBE_BIN=$PWD/target/debug/ascribe pnpm --filter ascribe-vscode test:integration
pnpm lint && pnpm format:check
```

## Commits

"Open the actions for the cursor with one key"
