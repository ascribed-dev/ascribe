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
