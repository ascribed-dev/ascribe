# Phase 7: Guidance, and closing the plan

Part of [Visual design](README.md). Requires phases 1 to 6. Docs only.

## Goal

- **One short page** that tells a contributor how to use the mark and the tokens, so the next stylesheet change doesn't undo this work.
- **The plan closed:** the decisions that outlast it moved to `project-docs/decisions.md`.

## Context

- `design/README.md`: started in phase 1, added to in phase 3.
- `design/specimen.html` and its generator: now shows the chosen design.
- `project-docs/decisions.md`: how decisions are numbered and where a finished plan's go. `project-docs/checklists.md`: what a kind of change must touch.
- `CONTRIBUTING.md` and `AGENTS.md`: the rules a contributor reads first.
- `project-docs/outside.md`: the accounts that show the avatar.

## Design

### `design/README.md`

Keep it to what someone needs in order to make a change correctly. One screen each:

- **The files:** the token source, the mark's sources, the generated assets and where each goes, the specimen.
- **Changing a color or adding a token:** edit `tokens.toml`, bless, look at the specimen, run the contrast test. When to add a semantic token and when to reuse one.
- **Using the mark:** which file for which place, the clear space around it, the smallest size, and the few things not to do (recolor it outside the palette, stretch it, set it on a busy background). Three or four, not a brand manual.
- **In VS Code:** codicons first, theme colors first, and how a named color or a glyph is added.
- **What isn't here:** components. Say so, and point at "Later" in this plan.

### Closing

- Move to `decisions.md`, under a new heading: one home for tokens (2), public names don't move and default values follow the tokens (4, with phase 4's contract decision), light and dark with checked contrast (5), the theme wins in VS Code (6), system fonts (7), generated assets (9), codicons first (10).
- `project-docs/checklists.md`: a short checklist, "a change to how something looks": tokens, bless, embed, contrast, specimen, screenshots in light and dark, the outputs comparison for the report.
- `AGENTS.md`, "Rules that aren't obvious": one line. Colors are written in `design/tokens.toml` only, and a stylesheet change is an output change.
- `decisions.md`'s line on plans in progress no longer lists this one.
- Whether the plan's folder stays as history or is removed is the maintainer's call; ask in the pull request.

## Tasks

1. `design/README.md`, as above. Regenerate the specimen and check it shows only the chosen design.
2. The decisions, the checklist, and the `AGENTS.md` line.
3. Read every stylesheet's header comment and each package README's theming section once more: each should describe what's true now, and none should describe the old values.
4. A last look at every surface together, in light and dark: the docs site, a page of every element, a review in `astro dev`, the HTML report, the preview, and the extension's listing. List anything that still looks like it came from somewhere else, as issues, not as work in this phase.

## Out of scope

New design work; anything under "Later" in the README.

## Acceptance criteria

- A contributor can change the accent color by reading `design/README.md` alone, and every surface follows.
- Every decision that still binds is in `decisions.md`, numbered, with its reason.
- No doc or comment describes values, files, or names that no longer exist.

## Verify

```sh
pnpm format:check && pnpm lint && pnpm typecheck && pnpm test
```

`pnpm test` includes `scripts/project-docs/checklists.test.ts` and `scripts/repo-docs/paths.test.ts`, which check the checklists and the paths these docs name.

## Commits

1. "Say how to use the mark and the tokens"
2. "Move the visual design decisions to the decisions list"
