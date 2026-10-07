# Phase 4: The actions bar

Part of [Editor UI](README.md). Requires phase 3. Extension only.

## Goal

One key opens a quick pick listing only the actions that apply to the cursor or selection, fixes first, for writers.

## Context

- Phase 3's registry (`src/actions/registry.ts`), runner (`src/actions/run.ts`), and context keys.
- `vscode.window.createQuickPick`: items with `label`, `description`, `detail`, `iconPath`; `kind: QuickPickItemKind.Separator` for groups; `matchOnDescription`; `busy`.
- `vscode.commands.executeCommand("vscode.executeCodeActionProvider", uri, range, "quickfix")`: every provider's quick fixes at the cursor. That's the server's fixes for problems, and also phase 3's own lightbulb actions of kind `quickfix.ascribe`.

## Design

- **Command:** `ascribe.actions` ("Ascribe: Actions for the cursor"), in the palette and the context menu, and bound to a key (below).
- **Opening:**
  1. Show the quick pick at once with `busy`, so the key feels instant.
  2. Get the context from phase 3's cache (`src/actions/context.ts`), which asks the server if it has no answer for this version and selection, and ask VS Code for the quick fixes at the cursor, in parallel.
  3. Fill the list. If the context request fails (the server isn't running), say so in one item that opens the project's output.
- **Items, in this order, with separators:**
  1. **Fix** (only when there are problems at the cursor): each quick fix, titled as the server titles it, with the diagnostic's message as the detail line. Drop the code actions whose kind is `quickfix.ascribe`: they're registry actions, and each is already listed once, in its registry group.
  2. The registry's groups that have applicable actions: **Write** (notes, details, steps), **Structure** (variant groups, arms, ids, availability, the page's variant and availability), **Link** (links, phrases, includes, copying a link to a section), **Media** (images, snippets, widgets).
- **Each item:** the action's title as the label, its description, and a `detail` line previewing what it writes when that's short and known before the wizard (`@note {type=tip}`, `@steps`, `@id: install-cli`).
- **Running:** choosing an item runs the action through phase 3's runner; its wizard continues in the same quick input, so the bar never closes and reopens between steps.
- **Empty:** if nothing applies, one disabled item says what to try ("Select some text, or put the cursor on a heading, note, link, or image").
- **Nothing else:** no search modes, no project-wide actions, nothing outside the editor (README decision 1).

### The key

Pick a default that:

- is unbound in VS Code's default keymaps on macOS, Windows, and Linux, checked against the current defaults (**Preferences: Open Default Keyboard Shortcuts (JSON)**) and listed in the PR;
- is easy to reach and to remember, such as `Cmd+Alt+A` / `Ctrl+Alt+A`, or a chord such as `Cmd+K A`;
- is bound only when `editorTextFocus && ascribe.inProject && editorLangId == markdown`.

`Cmd+.` is the lightbulb and stays VS Code's. Document the key, and how to change it, in `docs/content/guides/editor.md`.

## Tasks

1. `src/actions/bar.ts` with the behavior above; register `ascribe.actions` and its keybinding.
2. Unit tests: item ordering, groups, separators, the empty state, and that no action is listed twice when a quick fix of kind `quickfix.ascribe` comes back, from sample contexts and quick fixes.
3. Integration tests (extend `actions.it.ts`):
   - the bar lists exactly the applicable actions for a cursor on a note, on a heading, on a selection of prose, and on a blank line;
   - a problem at the cursor puts its fix first;
   - choosing an action with a wizard completes it in the same quick input;
   - in `test/fixtures/monorepo`, the bar in a nested project's page offers that project's note types and dimensions, not the parent's.
4. A description for `ascribe.actions` in `docs.test.ts`'s `commands`, then bless the commands table. `docs/content/guides/editor.md`: the bar, its key, and how it relates to the palette, the context menu, and `Cmd+.`; a line in `CHANGELOG.md`.

## Out of scope

Content-model actions (phase 5), which join the bar when they exist through the same registry.

## Acceptance criteria

- The key opens the bar in an Ascribe page, and does nothing elsewhere, including in a Markdown file outside every project.
- The list is right for each context in the tests, fixes first, and no action appears twice.
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
