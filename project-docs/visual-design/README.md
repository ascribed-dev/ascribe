# Visual design

A mark, a set of design tokens, and the assets made from them, used by everything Ascribe shows: the docs site, the element library, review, the HTML report, and the VS Code extension. Not a design system: there's no component library and no new CSS framework.

## Why now

- Ascribe has no logo, no favicon, and no Marketplace icon. The extension's only image is `packages/vscode/media/preview.svg`.
- Colors are written by hand in five stylesheets, about 68 distinct hex values, most of them GitHub's. The docs site, the element library, review, and the report each name their own variables for the same things (`--accent`, `--ascribe-note-color`, `--ascribe-review-focus`, `--r-accent`).
- The [editor UI plan](../editor-ui/README.md) needs an activity bar icon in its phase 6 and walkthrough images in its phase 9.

## What exists today

| Surface | Stylesheet | Its variables | Who reads them |
|---|---|---|---|
| The element library | `packages/elements/css/style.css` | `--ascribe-*`, light and dark through `light-dark()` | Every site built with Ascribe. A contract: `packages/elements/CONTRACT.md` and the list in its README |
| Review's marks and overlay | `packages/review/src/marks/marks.css`, and the styles under `overlay/` | `--ascribe-review-*` | Sites in `astro dev`, the preview, the report. Documented as themable |
| The HTML report's frame | `packages/review/src/report/report.css` | `--r-*` | The report only |
| The report's embedded stylesheet | `crates/ascribe-diff/src/html/report.css` | All of the above | Generated from the three above by `pnpm --filter @ascribed/review embed`, and compiled into the binary |
| The docs site | `site/src/styles/site.css` | `--text`, `--muted`, `--accent`, and so on | The docs site only |
| The preview webview | `packages/vscode/src/webview/preview.css` | Sets `--ascribe-*` from `--vscode-*` theme colors | The extension only |

## What VS Code allows

The plan is shaped by this, so it's written down once.

| Surface | An extension controls | It doesn't control |
|---|---|---|
| Native UI: tree views, quick picks, the status bar, menus, hovers | Text, one icon per item, a theme color on some icons, badges, Markdown in tooltips | Layout, fonts, spacing, backgrounds, any CSS |
| The text editor | Decorations: opacity, color, background, border, gutter icons, text before or after a range | Anything that moves or resizes text |
| Webviews: the preview, review | All of it; it's our HTML and CSS | Nothing, but the convention is to take colors from `--vscode-*` so the page follows the user's theme |

- **Colors are theme colors.** Native UI and decorations take a named theme color, not a hex value. An extension can declare its own (`contributes.colors`), each with defaults for light, dark, and both high-contrast themes, which users and themes can override.
- **Icons are one color almost everywhere.** The activity bar icon is a single-color SVG that VS Code tints. Menus, tree items, and the status bar use VS Code's own icons (codicons), or glyphs from an icon font the extension ships (`contributes.icons`).
- **Full color is possible in three places:** the Marketplace icon (a PNG), walkthrough media, and webviews.

## Decisions

These are settled. Don't reopen them in a phase; if one can't be met, stop and report.

1. **Tokens and assets, not a design system.** The plan delivers a mark, tokens, generated assets, and a page of guidance. It adds no components and restructures no stylesheet beyond its variables.
2. **One home for tokens.** `design/tokens.toml` is the only place a color, font stack, space, or radius is written. The `:root` blocks of every stylesheet are generated from it, and a test fails on a color literal anywhere else in a stylesheet.
3. **Two tiers.** The source has a private palette (`blue.600`) and semantic tokens that point into it (`accent`, `note.color`, `review.added`). Stylesheets use semantic tokens only; a palette name never reaches CSS.
4. **Public names don't move.** `--ascribe-*` and `--ascribe-review-*` are what sites theme. Their names stay as they are. Their default values change only in phase 4, as a contract change recorded with it. The docs site's and the report's own names (`--accent`, `--r-*`) are private and may be renamed.
5. **Light and dark, checked.** Every semantic color has a light and a dark value. A test computes contrast: 4.5:1 for text on its background, 3:1 for borders, marks, and icons that carry meaning. Nothing is told by color alone, as the elements and review already hold.
6. **In VS Code, the user's theme wins.** Native UI uses codicons and theme colors. The preview keeps taking its colors from `--vscode-*`. Ascribe's own colors appear in the Marketplace icon and the walkthrough, and nowhere else in the editor.
7. **System fonts.** No web font, anywhere. The report must make no network requests, the elements inherit the site's font, and a font file costs every reader a download for little gain.
8. **The mark is chosen by the maintainer.** Phase 2 produces candidates and stops. Nothing downstream starts on a mark that hasn't been chosen. Whatever is chosen must read in one color at 16 pixels, because the activity bar and the favicon need that.
9. **Assets are generated.** Every raster and every size comes from the source SVGs in `design/` by one script. The generated files are committed, since the extension package and the site need them at build time, beside a manifest of the source's hashes; a test fails when a source changed and the script wasn't run.
10. **Codicons first.** The extension uses VS Code's icons for everything they cover. An Ascribe glyph is added only for a concept on phase 6's list, and all of them ship as one icon font.
11. **The mark is the footnote, in Inter; the palette and type scale are Ink.** Chosen by the maintainer in phase 2 ([#169](https://github.com/ascribed-dev/ascribe/pull/169)): a lowercase a with a raised asterisk, outlined from Inter Bold, with the wordmark in Inter Semibold (`design/mark.svg`, `design/mark-color.svg`, `design/wordmark.svg`); an indigo accent over cool neutrals, with a compact type scale (`design/candidates/chosen.toml`). A full-color mark for a dark background is generated from `mark.svg`, its `class="second"` asterisk in the dark accent and the rest in the dark text color, never drawn by hand.

## Phases

| Phase | Result |
|---|---|
| [1: The token source](phase-1-tokens.md) | `design/tokens.toml` holds today's values, and every stylesheet's variables are generated from it. Nothing looks different. |
| [2: The mark and the palette](phase-2-direction.md) | Candidates for the mark, the palette, and the type scale, on a specimen page. Stops for the maintainer's choice. |
| [3: Assets](phase-3-assets.md) | The favicon set, the Marketplace icon, the activity bar icon, the social card, and the README header, generated from the chosen mark. |
| [4: Elements, review, and the report](phase-4-outputs.md) | The chosen palette as the default values of `--ascribe-*` and `--ascribe-review-*`, and in the report's frame. A contract change. |
| [5: The docs site](phase-5-site.md) | The site restyled from the tokens, with the mark, a favicon, and a social card. |
| [6: The extension](phase-6-extension.md) | The Marketplace icon and banner, the activity bar icon, named theme colors, and the icon decision carried out. |
| [7: Guidance, and closing the plan](phase-7-guidance.md) | A short page on using the mark and the tokens, and the decisions that last moved to `decisions.md`. |

Phase 1 needs nothing and can start now. Phase 2 needs nothing either, and is the slow one: it waits on a person. Phases 3 to 6 need phase 2's choice. Phases 4, 5, and 6 need phase 1, and can run in any order after phase 3.

## The editor UI plan

- Its phases 1 to 5 need nothing from this plan.
- Its phase 6 (the activity bar icon) needs this plan's phase 3. If it gets there first, it stops and reports rather than drawing a placeholder.
- Its phase 7 (the views' icons) follows decision 10 and this plan's phase 6 list.
- Its phase 9 (walkthrough images) uses the mark and screenshots of the finished UI. This plan's phase 6 doesn't make them.
- Whichever plan adds `contributes.viewsContainers` first owns that entry; this plan's phase 6 only supplies and names the icon file.

## Rules for every phase

- Branch before committing; never commit to `main`.
- Read the current code before the phase file's pointers. If the phase file and the code disagree, or a decision above can't be met, stop and report instead of choosing silently.
- **A stylesheet change is an output change.** `crates/ascribe-diff/src/html/report.css` is compiled into the binary, so `ascribe diff --format html` writes different bytes when any of its three sources changes. Run `pnpm --filter @ascribed/review embed`, then `node scripts/compare/outputs.ts --base main`, and name the report files it lists with `--accept` (in CI, the label `outputs changed` and the block in the description; see CONTRIBUTING.md). Nothing else may differ.
- **Generated files aren't edited by hand.** The generated blocks, the embedded report stylesheet, and the assets are rewritten by their script or by a test run with `ASCRIBE_BLESS=1`. Read the diff.
- **Contracts change only by a recorded decision.** Phase 4 is the one phase that changes one, and says how.
- **Look at it.** A phase that changes how something looks is checked by eye, in light and dark, and the pull request says what was looked at, with screenshots. The tests check contrast and structure, not taste.
- **User-visible changes** update the page under `docs/content/` and the unreleased section of `CHANGELOG.md` in the same phase.
- No phase history in code or docs. Describe what the code does now.
- Before finishing a phase, all of these pass:

  ```sh
  cargo fmt --all --check
  cargo clippy --workspace --all-targets --locked -- -D warnings
  cargo test --workspace --locked
  pnpm format:check && pnpm lint && pnpm typecheck && pnpm test
  ```

## Later, not in this plan

- Components: a shared header, buttons, or form controls across the site, review, and the report.
- A theme picker on the docs site. It follows the system.
- Illustrations, a marketing page, or motion.
- Ascribe-themed syntax highlighting or a VS Code color theme.
- Tokens for the sites people build with Ascribe beyond the `--ascribe-*` names that exist.
