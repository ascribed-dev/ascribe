# Phase 9: The walkthrough and a docs review

Part of [Editor UI](README.md). Requires phases 1 to 8. Extension and docs.

## Goal

- **A getting-started walkthrough** in VS Code (the panel shown after install, and under **Help → Welcome**).
- **The editor docs reviewed as a whole**, now that eight phases have each added to them.

## Context

- VS Code's `contributes.walkthroughs`: steps with a title, a Markdown description (which can link to commands with `command:` links), media (an image or Markdown), and `completionEvents` (`onCommand:`, `onView:`, `onContext:`, and `onLink:`, for example `onCommand:ascribe.actions`, `onView:ascribe.projects`).
- `docs/content/getting-started.md` and `docs/content/guides/editor.md`.
- `packages/vscode/README.md`: the Marketplace page.
- `packages/vscode/test/unit/docs.test.ts`: generates the commands and settings tables (`docs/content/_generated/editor-commands.md` and `editor-settings.md`) from `package.json` and its own `commands` descriptions, and fails when a palette command has no description.

## Design

### Walkthrough steps

1. **Open a project.** What an `ascribe.toml` is, with a link to `docs/content/getting-started.md`. Done on `onContext:ascribe.active`, which the extension sets when the workspace has a project.
2. **Preview a page.** Runs **Open Page Preview to the Side**. Done on `onCommand:ascribe.openPreview`.
3. **Use the actions bar.** The key, and what it offers; a button runs it. Done on `onCommand:ascribe.actions`.
4. **See your projects.** Opens the Ascribe sidebar. Done on `onView:ascribe.projects`.
5. **See a build.** Turns on the build lens. Done on `onCommand:` with the lens command's id.
6. **Check in CI.** `npx ascribe check` and a link to the command reference on the docs site. Done on `onLink:` with that link's URL, so the step completes when the writer follows it.

Media for each step is a small image or a short Markdown snippet in `packages/vscode/media/walkthrough/`. Keep images light: they ship in every platform's extension package.

### Docs review

Read `docs/content/guides/editor.md` and `packages/vscode/README.md` start to finish, as a writer new to Ascribe would, and fix:

- the order, so it follows what a writer does: preview, actions, the sidebar, the build lens, then settings and commands;
- duplication between sections added by different phases;
- the commands and settings tables, so they're complete and each row reads well. They're generated: change a command's title or a setting's description in `package.json`, or a command's description in `docs.test.ts`, then bless. Don't edit the files in `docs/content/_generated/`;
- the CHANGELOG's unreleased section, so it reads as one release's notes and not nine phases' worth.

## Tasks

1. The walkthrough in `package.json`, with its media, and a unit test that every command, view, and context key it references exists, and that every step has a completion event.
2. The docs review, as above.
3. Run the extension by hand on `examples/monorepo` (**Run and Debug → Extension: several projects**) and go through the walkthrough. Say in the PR what you checked, and anything that felt wrong to a first-time user.

## Out of scope

Releasing (a separate release, following `RELEASING.md`).

## Acceptance criteria

- The walkthrough appears after install, each step's button works, and each completes on its event.
- `docs/content/guides/editor.md` covers every feature from this plan once, in a writer's order.

## Verify

```sh
pnpm --filter ascribe-vscode typecheck
pnpm --filter ascribe-vscode test
pnpm lint && pnpm format:check
```

## Commits

1. "Add a getting-started walkthrough"
2. "Review the editor docs as a whole"
