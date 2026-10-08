# Phase 6: The extension

Part of [Visual design](README.md). Requires phases 1 and 3. `packages/vscode` only.

## Goal

The extension carries the mark where VS Code lets it, and takes everything else from the user's theme (README decision 6):

- a Marketplace icon and banner;
- the activity bar icon, ready for the editor UI plan's view container;
- named theme colors for the few things no built-in color fits;
- the icon decision (README decision 10) carried out.

## Context

- The README's "What VS Code allows".
- `packages/vscode/package.json`: no `icon`, no `galleryBanner`, no `contributes.colors`, no `contributes.icons`. Commands use codicons (`$(open-preview)`, `$(git-compare)`).
- `packages/vscode/media/`: phase 3 put the Marketplace icon and the activity bar icon here.
- `packages/vscode/src/webview/preview.css`: maps `--ascribe-*` to `--vscode-*` (chart colors for note accents, `color-mix` for tints). `packages/vscode/src/preview/`: the review status bar item and the source-line comments.
- The [editor UI plan](../editor-ui/README.md): its phase 6 adds the view container, its phase 7 the views, its phase 8 the build lens's decorations. See the README's section on it for who owns what.
- `packages/vscode/test/unit/docs.test.ts`: generates the settings and commands tables from `package.json`.
- VS Code's theme color reference and its codicon list, for what already exists before adding anything.

## Design

### The listing

- `"icon"`: the 256 pixel PNG. `"galleryBanner"`: a `color` from the palette and the matching `theme` (`dark` or `light`), chosen so the icon and the title both read on it.
- `packages/vscode/README.md` is the Marketplace page. Its images must be absolute `https` URLs, and SVG isn't allowed there: use PNG for anything it shows.

### Named colors

Add a `contributes.colors` entry only where a built-in theme color would mislead or doesn't exist. Start from this list and cut what the code doesn't need:

| Id | For | Default |
|---|---|---|
| `ascribe.review.addedForeground`, `changedForeground`, `removedForeground`, `movedForeground` | Review's marks in the source editor, if it draws any | The matching `gitDecoration.*` or `charts.*` color |

The build lens needs none: it dims by opacity.

Each entry has `light`, `dark`, `highContrast`, and `highContrastLight` defaults, and each default is a reference to an existing theme color where one fits, so a user's theme carries through. A hex default is the last resort. An id is public once released: users put it in their settings. Name them carefully, and record them in the extension's README.

If nothing needs a named color yet, add none and say so. The editor UI plan's phases add theirs by these rules.

### Icons

- **Codicons** for everything they cover: projects, pages, links, references, builds, problems.
- **Ascribe glyphs,** only for concepts with no fitting codicon. The candidates: a variant, a phrase, a fragment, an availability state. For each, first look for a codicon that reads right (`$(symbol-enum)`, `$(symbol-string)`, `$(file-symlink-file)`, `$(tag)`). Add a glyph only where none does.
- If any are added, they ship as one icon font built from one-color SVGs in `design/icons/` by phase 3's script, declared in `contributes.icons` (`ascribe-variant`, `ascribe-phrase`), and used as `$(ascribe-variant)`. They're drawn on the codicon grid (16 pixels, 1 pixel strokes) so they sit beside codicons without looking foreign.
- If none are added, record that in the pull request and leave `design/icons/` out.

### The preview

Keep the mapping to `--vscode-*`. Check it against the new element defaults from phase 4 in a light, a dark, and both high-contrast themes, and fix only what reads badly. The preview should look like the user's editor, not like the docs site.

## Tasks

1. The listing: `icon`, `galleryBanner`, and the README's images. `vsce package`, then check the package's contents and size.
2. The activity bar icon in `media/`, with its file name recorded in the README's section on the editor UI plan if that plan hasn't added the view container yet.
3. Named colors, if any, with a unit test that every `contributes.colors` entry has all four defaults and a description.
4. The icon decision: the table of concept, codicon tried, and what was chosen, in the pull request. The font and `contributes.icons` if any glyph is added, with a unit test that every `$(ascribe-…)` used in `package.json` or the source is declared.
5. The preview, checked in four themes, with screenshots.
6. `docs/content/guides/editor.md`: the named colors, if any, and how to override them. `CHANGELOG.md`.

## Out of scope

The view container, views, and decorations themselves (the editor UI plan); walkthrough images (its phase 9); a color theme or icon theme for VS Code.

## Acceptance criteria

- The packaged extension shows its icon in the Extensions view and on the Marketplace page preview (`vsce package`, then install the `.vsix`).
- The activity bar icon reads in a light, a dark, and a high-contrast theme, at 100% and 200% zoom.
- Nothing in the native UI uses a hex color. A test fails on one in `packages/vscode/src` outside the webview's generated block.
- The extension package grew by no more than 100 KB. Say the number.

## Verify

```sh
pnpm --filter ascribe-vscode typecheck
pnpm --filter ascribe-vscode test
pnpm lint && pnpm format:check
```

Then build the `.vsix` the way `RELEASING.md` and the release workflow do (the package has no `package` script of its own), install it locally, and look. Don't publish it.

## Commits

1. "Give the extension its Marketplace icon and banner"
2. "Add the activity bar icon"
3. "Declare the extension's theme colors and icons" (if any)
