# Design tokens

`tokens.toml` is the one place a color, font stack, space, or radius of Ascribe's own stylesheets is written. Each stylesheet's variables are generated from it, between marker comments; nothing else in a stylesheet names a color.

## The file

- `[palette]`: every color value, named by hue and step (`blue.460`). The step is 1000 × (1 − the color's OKLCH lightness), rounded, so a higher step is darker; a tie takes the next step.
- `[color]`: what each color is for (`elements.note`, `review.added`, `site.text`), with a `light` and a `dark` value, each a palette name. Stylesheets take these, never a palette name.
- `[font]`, `[space]`, `[radius]`: values the stylesheets use as they are.
- `[emit."<stylesheet>".<block>]`: what a stylesheet's generated block declares. Each key is a property, and its value is a token (`"color.review.muted"`) or a template with tokens in braces (`"var(--vscode-charts-blue, {color.elements.note})"`).

## The stylesheets

| Stylesheet | Its variables |
|---|---|
| `packages/elements/css/style.css` | `--ascribe-*`, public |
| `packages/review/src/marks/marks.css` | `--ascribe-review-*`, public |
| `packages/review/src/overlay/style.ts` | The overlay's own, from `--ascribe-review-*` |
| `packages/review/src/report/report.css` | `--r-*`, and the element library's in dark |
| `packages/astro/src/toolbar/style.ts` | The toolbar panel's own |
| `site/src/styles/site.css` | The docs site's own |
| `packages/vscode/src/webview/preview.css` | `--ascribe-*` from the editor's theme, with fallbacks |

`crates/ascribe-diff/src/html/report.css` is built from the first, second, and fourth by `pnpm --filter @ascribed/review embed`.

## Generated blocks

A block starts and ends with a marker comment:

```css
:root {
  /* Generated from design/tokens.toml by scripts/design/tokens.ts: root, light. */
  --ascribe-note-color: #0969da;
  /* End of generated root. */
}
```

The marker names the `[emit]` block and how to write colors: `light` or `dark` writes that value, and `light-dark` writes `light-dark(<light>, <dark>)`. A `dark` or `light-dark` block declares only the properties that take a color. Everything outside the markers is left as it is, so a stylesheet keeps its own arrangement of light and dark.

## Changing a value

Edit `tokens.toml`, then rewrite the blocks and the report's embedded stylesheet, and read the diff:

```sh
ASCRIBE_BLESS=1 pnpm exec vitest run scripts/design/tokens.test.ts
pnpm --filter @ascribed/review embed
```

`scripts/design/tokens.test.ts` fails when a block is stale, when a stylesheet writes a color outside its blocks (a hex value, a color function, or a named color; `transparent`, `currentColor`, and system colors such as `Canvas` are allowed), and when a palette entry or a color token is unused.

## The mark

`mark.svg` is the mark in one color, from `currentColor`; its asterisk carries `class="second"`, the part a second color goes on. `mark-color.svg` is the same paths in the chosen palette's light text and accent colors, for a light background; a test holds it to both. `wordmark.svg` is "ascribe". `tagline.svg` is the line on the social card. All four are outlines from Inter (SIL Open Font License), so no font file ships with them.

## The assets

Every image of the mark that something needs is generated from `mark.svg`, `wordmark.svg`, `tagline.svg`, and the chosen palette's colors in `candidates/chosen.toml`, by one script. The full-color mark for a dark background is `mark.svg` with its `class="second"` asterisk in the dark accent and the rest in the dark text color; it isn't a file of its own.

| Asset | For | Form |
|---|---|---|
| `site/public/favicon.svg` | The docs site's tab | The mark in the text color, with a `prefers-color-scheme` rule inside for a dark tab |
| `site/public/favicon.ico` | Browsers that want an `.ico` | 32 pixels, the full-color mark on a white tile, so it shows on a light or a dark tab |
| `site/public/apple-touch-icon.png` | A phone's home screen | 180 pixels, the full-color mark on white |
| `site/public/social-card.png` | Links to the docs site and the repository | 1200 × 630: the mark, the wordmark, and the tagline on white |
| `packages/vscode/media/icon.png` | The extension's Marketplace listing and Extensions view | 256 pixels, the dark full-color mark on a rounded tile in the dark surface color, so it reads in a light or a dark editor |
| `packages/vscode/media/activity.svg` | The extension's activity bar icon | 24 pixels, `currentColor` only, which VS Code tints. The mark takes the middle 20, as tall as the codicons beside it |
| `design/out/header-light.svg`, `header-dark.svg` | The header of `README.md` and `packages/vscode/README.md`, in GitHub's `<picture>` | The wordmark, 48 pixels tall, in the light and the dark text color |
| `design/out/header-vscode.png` | The header of `packages/vscode/README.md`, the extension's Marketplace page, which takes no SVG and can't follow the reader's scheme | 192 pixels tall, for a 96 pixel header on a high-density screen: the wordmark in the dark text color on a rounded plate in the dark surface color, the icon's tile, so it reads on a light or a dark page |
| `design/out/avatar.png` | The GitHub organization and the Marketplace publisher | 512 pixels, the full-color mark on white, inside the circle an avatar is cropped to |

The colors come from `candidates/chosen.toml` because the chosen palette isn't in `tokens.toml` yet. When `tokens.toml` holds it, the script reads the colors from there instead, so the two can't drift.

The SVGs are written in a fixed form (paths only, two decimals, no comments or metadata), so they're the same bytes on every platform. The PNGs come from `sharp`, whose output can differ slightly between platforms.

After changing a source, rewrite them all and look at them:

```sh
node scripts/design/assets.ts
```

`design/assets.json` records each asset, its size, and the hash of each source it's made from. `scripts/design/assets.test.ts` fails when a source has changed since the script last ran, naming the command, and when an asset is missing, the wrong size, or, for an SVG, different from what the script writes now. It doesn't compare a PNG's pixels.

## Candidates and the specimen

`candidates/chosen.toml` is the chosen palette and type scale, and `candidates/baseline.toml` today's values, to compare with. `candidates/pairs.toml` lists every pairing of colors the stylesheets use, checked at 4.5:1 for text and 3:1 for graphics in light and dark. `specimen.html` shows them all, with the mark, generated by `node scripts/design/specimen.ts`; `scripts/design/specimen.test.ts` fails when it's stale (`ASCRIBE_BLESS=1` rewrites it) and when the chosen palette misses a pairing.
