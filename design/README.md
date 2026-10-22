# Design

Ascribe's mark and design tokens, and the images made from them. Everything Ascribe shows takes its colors, type, and mark from here: the docs site, the element library, review, the HTML report, an archive, the Astro toolbar, and the VS Code extension. The decisions behind them are under [The look](../project-docs/decisions.md#the-look) in `project-docs/decisions.md`.

## The files

| File | What it is |
|---|---|
| `tokens.toml` | Every color of Ascribe's stylesheets, the only place one is written, and their font stacks, type scale, spaces, and radii. Some sizes and spaces are still written in the stylesheets by hand; a new one goes here |
| `mark.toml` | The mark, its 16 px cut, and the wordmark's letters, as pixel grids, and the social card's line: the source of every image of the mark |
| `mark.svg`, `wordmark.svg` | The mark and the wordmark in one color, from `currentColor`, generated from `mark.toml`. The change marks carry `class="second"`, the part a second color goes on |
| `mark-color.svg` | The mark in its light colors, for a light background, generated |
| `fonts/Charter-Bold.ttf` | The font the social cards' text is drawn in, for generation only, with its license (Bitstream's, for the Charter fonts given to the X Consortium) in `fonts/LICENSE.txt` |
| `ascribe.af` | The brand's exploration boards (Affinity), where the mark came from |
| `assets.json` | The generated images of the mark, their sizes, and the hashes of their sources |
| `out/` | The generated images nothing builds from: the README headers and the avatar |
| `candidates/chosen.toml` | The palette and type scale the specimen shows. Its shared colors, and the palette steps they take, are generated from `tokens.toml`; the type scale and the rest of each hue's ramp are written in it |
| `candidates/pairs.toml` | Every pairing of colors the stylesheets use, with the contrast each must reach |
| `specimen.html` | The mark, the palette, and the type scale in light and dark, with a docs page in each and every pairing's contrast. Open it in a browser |

The mark and the wordmark are pixels, drawn from `mark.toml`, so no font ships with them. Ascribe uses no web font anywhere; the stylesheets use the system's. The one font file here, Charter Bold from [Practical Typography's copy](https://practicaltypography.com/charter.html) of Bitstream's Charter, is read by the assets script, which outlines the social cards' text with it, so it's drawn the same on every machine; nothing ships it to a reader (decision 48). Charter is the brand's reading face: headings set in `ui-serif` look like it on Apple's systems.

## Changing a color

To change the accent, or any color:

1. Point the token at another palette entry in `tokens.toml`, or add the entry under `[palette]`. A palette entry's name is its hue and its step: 1000 × (1 − the color's OKLCH lightness), rounded, so a higher step is darker; a tie takes the next step. The specimen shows each hue's whole ramp under these names: a step under `[ramp]` in `candidates/chosen.toml` is one no color takes yet, and a color that comes to take it moves it from there into `tokens.toml`'s palette.
2. Rewrite everything generated from it, then read the diff:

   ```sh
   ASCRIBE_BLESS=1 pnpm exec vitest run scripts/design/tokens.test.ts
   pnpm --filter @ascribed/review embed
   node scripts/design/assets.ts
   node scripts/design/specimen.ts
   ```

   The first rewrites the stylesheets' generated blocks and the shared colors in `candidates/chosen.toml` (the top-level `[color]` tokens such as `color.accent` or `color.note`, and the palette steps they take); the second the HTML report's embedded stylesheet, the third the mark's images (only a `color.brand` color can change them), and the fourth the specimen.
3. Run the tests: `pnpm exec vitest run scripts/design`. They fail on a pairing below its contrast in light or dark (4.5:1 for text, 3:1 for a border, bar, or mark that carries meaning), on a stale block or image, and on a color written in a stylesheet outside its generated blocks.
4. Look at it, in light and dark: the specimen, then each surface the token reaches. The tests check contrast, not taste.
5. The report's embedded stylesheet is compiled into the binary, so the change is an output change: `node scripts/compare/outputs.ts --base main` lists the report's files, which the pull request accepts by name (see [CONTRIBUTING.md](../CONTRIBUTING.md)).
6. A change to a default of an `--ascribe-*` or `--ascribe-review-*` property is a **Behavior change** in its changelog entry, a file in `changes/`, with the previous values for a site to paste (decision 44).

The full list is the checklist for [a change to how something looks](../project-docs/checklists.md#a-change-to-how-something-looks).

## Adding a token

Reuse a token when the new use means the same thing: a new panel is `color.surface`, a new link is `color.accent`, whatever it sits in. Add one when the meaning is new, or when the color must be able to change without the others changing with it. Don't add one for a single stylesheet's tweak of an existing color: pick a palette entry for an existing token instead, or ask whether the tweak is needed.

- Name it for what it's for (`color.flash`, "a block that was just scrolled to"), never for how it looks, with a comment above it saying where it's used. A color only one surface uses goes under that surface's name (`color.preview.error`, `color.toolbar.shadow`).
- A color that's text, or a graphic that carries meaning, gets its pairings in `candidates/pairs.toml`, so its contrast is checked.
- A new shared color is also added to `COLORS` in `scripts/design/specimen.ts`, which puts it in `candidates/chosen.toml` the next time the tokens test is blessed; a property of the element library's or review's that takes it, to `EMIT` there too, so the specimen shows it.
- The stylesheet takes it through its `[emit]` block; see below. A new `--ascribe-*` or `--ascribe-review-*` property is a contract addition, made by a recorded decision (`packages/elements/CONTRACT.md`).

### How `tokens.toml` is laid out

- `[palette]`: every color value, by hue and step (`river.494`). Only `[color]` points here; no stylesheet does.
- `[color]`: what each color is for, with a `light` and a `dark` value, each a palette name. The element library, review, the HTML report, the Astro toolbar, and the docs site share the top-level ones. The VS Code preview's fallbacks and a few colors of one surface's own are under that surface's name.
- `[font]`, `[space]`, `[radius]`: values the stylesheets use as they are.
- `[type]`: the type scale, each step a weight, a size, and a line height (`650 1.375rem/1.3`), for the `font` shorthand with a family after it: `font: var(--type-h2) var(--font)`. The docs site and the HTML report's frame set their text with it; the element library takes the site's type.
- `[emit."<stylesheet>".<block>]`: what a stylesheet's generated block declares. Each key is a property, and its value is a token (`"color.muted"`) or a template with tokens in braces (`"var(--vscode-charts-blue, {color.note})"`).

### The stylesheets

| Stylesheet | Its variables |
|---|---|
| `packages/elements/css/style.css` | `--ascribe-*`, public |
| `packages/review/src/marks/marks.css` | `--ascribe-review-*`, public |
| `packages/review/src/overlay/style.ts` | The overlay's own, from `--ascribe-review-*` |
| `packages/review/src/report/report.css` | `--r-*`, and the element library's in dark |
| `crates/ascribe-emit/src/archive/archive.css` | `--a-*`, an archive's own; an archive's `style.css` is the element library's, then this |
| `packages/astro/src/toolbar/style.ts` | The toolbar panel's own |
| `site/src/styles/site.css` | The docs site's own |
| `packages/vscode/src/webview/preview.css` | `--ascribe-*` from the editor's theme, with fallbacks |

`crates/ascribe-diff/src/html/report.css` is built from the first, second, and fourth by `pnpm --filter @ascribed/review embed`.

The `--ascribe-*` and `--ascribe-review-*` properties are public: sites theme the elements and review with them. Their names don't change, and a test holds them; their default values are these tokens and may change in a release (decision 44).

A generated block starts and ends with a marker comment:

```css
:root {
  /* Generated from design/tokens.toml by scripts/design/tokens.ts: root, light. */
  --ascribe-note-color: #0b6f8e;
  /* End of generated root. */
}
```

The marker names the `[emit]` block and how to write colors: `light` or `dark` writes that value, and `light-dark` writes `light-dark(<light>, <dark>)`. A `dark` or `light-dark` block declares only the properties that take a color. Everything outside the markers is left as it is, so a stylesheet keeps its own arrangement of light and dark, and names no color of its own: `transparent`, `currentColor`, and system colors such as `Canvas` are the only ones allowed outside a block.

## Using the mark

The mark is a heavy single-storey a with the change marks of a gutter beside it, drawn on a grid of square cells: a writer's change bars, and the bar an editor shows beside a changed line. The marks split around the a's middle row, so they read as two changed lines and never as an l. The wordmark is the mark grown into the word: the same a, then the letters of "ascribe" in the same hand. The letters exist for the wordmark only; set anything else in a typeface.

- **Take the file made for the place.** The table below lists them. For a new place, add an asset to the script rather than exporting one by hand, and use `mark.svg` or `wordmark.svg` with `currentColor` when the mark sits in text or UI that already has a color.
- **Two colors at most, from `color.brand` in `tokens.toml`:** one color (the ink, or whatever `currentColor` is), or the ink with the gutter's river blue on the change marks. Deep River is the mark's home: its ground, `color.brand.ground`, dark, is behind every tile and card. Never another color, a gradient, or an outline.
- **Whole pixels only.** Draw it a whole number of pixels a cell: 2 at 24 pixels (20 of art), 3 at 32, 12 at 180, and so on, so every edge is sharp. At 16 pixels, use the 16 px cut, `small` in `mark.toml`, 2 pixels a cell; the docs site's header draws the wordmark 2 pixels a cell, 18 pixels tall. Never stretch it, rotate it, smooth it, or redraw it.
- **Leave it room.** Keep clear at least two cells on every side, and put nothing busier than a flat color behind it: not a photo, a pattern, or text. The icons' tiles are drawn tighter, since the tile is the room.
- **In motion,** the bar is one bar in three states: after the a, it's a cursor, while the page is written; it steps into the margin and splits when the page changes; and the marks turn to `color.brand.reviewed`, moss, once the change is reviewed. No asset moves yet.

## The assets

Every image of the mark that something needs is generated from `mark.toml` and the mark's colors, `color.brand` in `tokens.toml`, by one script, and the social cards from those, the font, and each page's title. Every one is on Deep River's ground but the README header for a light page, which is in Shallows' colors, and the one-color files.

| Asset | For | Form |
|---|---|---|
| `design/mark.svg`, `design/wordmark.svg` | Wherever the mark sits in text or UI that has a color, and the specimen | One unit a cell, `currentColor` only, the change marks keeping `class="second"` |
| `design/mark-color.svg` | A light page that wants the mark in full color | The mark in `color.brand.ink` and `color.brand.gutter`, light |
| `site/public/favicon.svg` | The docs site's tab | The 16 px cut, 2 units a cell, in the ink and gutter colors, with a `prefers-color-scheme` rule inside for a dark tab |
| `site/public/favicon.ico` | Browsers that want an `.ico` | 32 pixels: the mark 3 pixels a cell on a rounded Deep River tile, so it shows on a light or a dark tab |
| `site/public/apple-touch-icon.png` | A phone's home screen | 180 pixels: the mark 12 pixels a cell on Deep River |
| `site/public/social-card.png` | Links to the docs site and the repository | 1200 × 630: the wordmark 16 pixels a cell, and the line from `mark.toml` under it in Charter, on Deep River |
| `site/public/cards/<page>.png` | Links to a page of the site outside the docs: one for each page of `site/home/content/`, by its entry id (`index.png` for the landing page), and `changelog.png` | 1200 × 630: the wordmark at the top, and the page's level-1 heading below in Charter, on Deep River. A page's heading is a source of its card, by itself (`<file>#heading` in `assets.json`), so the rest of the page can change without the card. A new page of `site/home/` gets a card when the script runs |
| `site/src/assets/logo.svg` | The docs site's header, inline | The wordmark, `currentColor` only, its change marks keeping `class="second"` for the page to color with `color.brand.gutter` |
| `packages/vscode/media/icon.png` | The extension's Marketplace listing and Extensions view | 256 pixels: the mark 16 pixels a cell on a rounded Deep River tile, so it reads in a light or a dark editor |
| `packages/vscode/media/activity.svg` | The extension's activity bar icon | 24 pixels, `currentColor` only, which VS Code tints: the mark 2 pixels a cell, 20 wide, as tall as the codicons beside it |
| `design/out/header-light.svg`, `header-dark.svg` | The header of `README.md`, in GitHub's `<picture>` | The wordmark 5 pixels a cell, 45 tall, in the light and the dark colors |
| `design/out/header-vscode.png` | The header of `packages/vscode/README.md`, the extension's Marketplace page, which takes no SVG and can't follow the reader's scheme | 192 pixels tall, for a 96 pixel header on a high-density screen: the wordmark 10 pixels a cell on a rounded Deep River plate, the icon's tile, so it reads on a light or a dark page |
| `design/out/avatar.png` | The GitHub organization and the Marketplace publisher | 512 pixels: the mark 28 pixels a cell on Deep River, inside the circle an avatar is cropped to |

The SVGs are written in a fixed form (paths only, two decimals, no comments or metadata), so they're the same bytes on every platform. The cards' text is outlined from the font by `opentype.js`, at a version pinned in the root `package.json`, so its drawing is the same everywhere too. The PNGs come from `sharp`, whose output can differ slightly between platforms.

After changing a source, rewrite them all and look at them:

```sh
node scripts/design/assets.ts
```

`scripts/design/assets.test.ts` fails when a source has changed since the script last ran, naming the command, and when an asset is missing, the wrong size, or, for an SVG, different from what the script writes now. It doesn't compare a PNG's pixels. It also checks the mark's colors against each other, as `candidates/pairs.toml` checks the stylesheets': 4.5:1 for the ink and the muted line on the ground, and 3:1 for the change marks. A new page outside the docs, or a changed heading on one, is a changed source too. The avatar is uploaded by hand where [project-docs/outside.md](../project-docs/outside.md) lists it.

## In VS Code

The user's theme wins. The extension's own colors appear only in its Marketplace icon and banner, and in the preview as fallbacks for theme colors the editor doesn't give.

- **Icons:** a [codicon](https://microsoft.github.io/vscode-codicons/dist/codicon.html) for every concept one covers, the same one for a concept everywhere. An icon of Ascribe's own is only for a concept no codicon reads right for, drawn in one color on the codicon grid and shipped in one icon font.
- **Colors:** a built-in theme color (`new vscode.ThemeColor("…")`), never a hex value. A color of the extension's own is declared in `contributes.colors`, with defaults for the light, dark, and both high-contrast themes, only when no built-in color fits.

How to add either, and the codicon for each concept, are in [packages/vscode/DEVELOPMENT.md](../packages/vscode/DEVELOPMENT.md#icons-and-colors). A test fails on a color written in the extension's code.

## The specimen

`specimen.html` is generated by `node scripts/design/specimen.ts` from `candidates/` and the stylesheets, and `scripts/design/specimen.test.ts` fails when it's stale (`ASCRIBE_BLESS=1` rewrites it) and when the palette misses a pairing. A candidate palette (another `candidates/*.toml`) or mark (`candidates/mark-*.svg`) added for a future redesign is shown beside the chosen one with no new code.

## What isn't here

Components: no shared header, buttons, or form controls across the site, review, and the report, and no CSS framework. Each surface keeps its own stylesheet and takes only its values from here. Nor a theme picker for the docs site, which follows the system, or a VS Code color theme.
