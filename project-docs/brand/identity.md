# Ascribe's identity

The mark, its colors, and its type, as they came out of the exploration boards in `design/ascribe.af`. [story.md](story.md) says what Ascribe is and promises; this says how it looks. [design/README.md](../../design/README.md) says how to use what's here.

## What's decided

These came from Kyle on 21 October 2026, choosing among the boards.

1. **The mark is Margin × Cursor.** A heavy single-storey a, drawn on a grid of square cells, with the change marks of a gutter beside it: the writer's change bar, and the bar an editor shows beside a changed line. Docs as code, with the tools docs need, in one picture.
2. **The change marks are split,** around the a's middle row, so they read as two changed lines and not as an l.
3. **The wordmark is the mark grown into the word:** the same a, then "scribe" in the same heavy pixel letters. The letters exist for the wordmark and its variants only. There's no pixel alphabet, and nothing else is set in pixels.
4. **The colors are Bryo Studio's Deep River,** for the mark's home and every dark surface, with **Shallows,** a light companion made from the same hues, where the mark sits on a light page. River blue is the change marks; moss is a change that was reviewed, in motion; amber is focus.
5. **Charter is the reading face** for headings and the social cards. Pages use `ui-serif`, which is New York on Apple's systems; images are outlined from Charter Bold, pinned in `design/fonts/` for generation only, so decision 48 holds: no font ships to a reader.
6. **The marks are drawn from pixel grids by a script,** `design/mark.toml` through `scripts/design/assets.ts`, as Bryo Studio's are. `mark.svg` and `wordmark.svg` are generated now, not sources.

## The mark in motion

The bar is one bar in three states:

1. **Writing:** a cursor after the a, blinking.
2. **Changed:** it steps left past the a, a cell at a time, and lands in the margin as the split change marks.
3. **Reviewed:** the marks turn moss.

No asset animates it yet.

## What's done

- `design/mark.toml`: the mark (10 × 7), its 16 px cut (8 × 7), and the wordmark's letters.
- `color.brand` in `design/tokens.toml`: ink, muted, gutter, reviewed, ground, and surface, light and dark. The assets' test checks their contrast.
- Every image `scripts/design/assets.ts` makes, redrawn: the favicons and touch icon, the social cards, the docs site's header logo, the Marketplace and activity bar icons, the README headers, and the avatar. Each is drawn a whole number of pixels a cell.
- The docs site's header draws the wordmark 2 pixels a cell, with the change marks in `color.brand.gutter`.
- The Marketplace banner is Deep River's ground.
- Decision [#801](../decisions/801-pixel-mark.md) records the mark, its colors, and the font, which change decisions 48 and 49's particulars.

## Next

1. **The pages' colors.** The site, the element library, review, and the HTML report still take the indigo palette. Moving them to Deep River and Shallows changes every stylesheet's defaults: an output change, and a **Behavior change** for the `--ascribe-*` properties (decision 44). It's a plan of its own, and it starts from the website revamp.
2. **The avatar,** uploaded by hand where `project-docs/outside.md` lists it.
3. **Motion:** the three states as an animated SVG for the site, holding still for anyone who prefers reduced motion.
4. **The publisher.** Story item 2 moves the maker to Bryo Studio; the avatar and banner are ready for it.
