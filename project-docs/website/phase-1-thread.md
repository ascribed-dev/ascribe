# Phase 1: The new colors and type, everywhere

Part of [The website, and the brand on everything else](README.md). It needs nothing, and it's the plan's thread: the shared palette moved to Deep River and Shallows on every surface at once, with the story's headline on the landing page so a visitor sees the change.

## Goal

- **Every surface in the new colors.** That's the docs site, the element library, review, the HTML report, the archive, the Astro toolbar, and the VS Code preview's fallbacks, in light (Shallows) and dark (Deep River).
- **Headings in the system's serif** on the site and in the report's frame.
- **The landing page leads with the story:** "Docs as code, with the tools docs need." and the sentence that says what Ascribe is.

## Context

- [The identity](../brand/identity.md) and decision [#801](../decisions/801-pixel-mark.md): the mark's colors, already `color.brand.*`.
- Board 11, exported to `project-docs/brand/boards/`:
  - `board.png` has the swatches with their roles.
  - `docs-light.png` and `docs-dark.png` show a docs page in each scheme: the header and its rule, a serif heading, muted body text, and a tip.
  - Match them where the tokens allow.
- [design/README.md](../../design/README.md): the palette, the semantic tokens, the `[emit]` blocks, and which stylesheet takes what.
- `design/tokens.toml`: the `deep`, `river`, and `moss` steps added for the mark; the shared tokens still point at `gray`, `indigo`, `green`, `amber`, `red`, `magenta`, and `yellow`.
- `design/candidates/pairs.toml`: the pairings the contrast test checks. `design/candidates/chosen.toml`: the candidate the palette came from, named "Ink", whose shared tokens are generated (decision 114).
- Decisions 44 to 48 in `project-docs/decisions.md`, under "The look".
- The landing page: `site/home/content/index.md`, its hero widget in `site/home/ascribe.toml`, and its styles in `site/src/styles/site.css`.
- `scripts/docs-site/screenshots.ts`: every picture of the site, review, the report, and the editor, retaken by name.

## Design

### The palette

The neutrals come from `deep`, which replaces `gray` for every shared token: background, surface, text, muted, border, and code background. It needs more steps than the mark used. Each new value is named by its step, 1000 × (1 − OKLCH L), as the file's header says.

| Role | Light | Dark |
|---|---|---|
| Background | `#f2f7f9` | `#0b1824` |
| Surface | `#ffffff` | `#112233` |
| Text | `#0b1824` | `#e3f1f7` |
| Muted | `#47606f` | `#8aa6b8` |
| Accent, link, steps, the active tab | river `#0b6f8e` | river `#56d0f0` |
| Focus | amber `#b4530f` | amber `#f0a35e` |

These are the identity's. The rest are derived from the same hues and checked:

- **Border, code background, accent surface, accent text:** steps of `deep` and `river`.
- **Notes:**
  - A note is river.
  - A tip is moss.
  - An important note is magenta.
  - A warning is yellow, kept apart from focus's amber.
  - A caution is red.

  Each has a tinted background. The red, magenta, and yellow ramps are retuned toward Deep River's cool cast, and kept as far apart as the current ones are for red-green color blindness.
- **States:** `state` and its background from `deep`; each lifecycle state takes its note's color, as it does now.
- **Review:**
  - Changed is river, so review's change marks are the mark's.
  - Added is moss.
  - Removed is red.
  - Moved is magenta.

  "Reviewed" has no token yet; it's the mark's motion, in phase 4.
- **The flash and the scrim** are retuned to the new ground.

Every new pairing goes in `pairs.toml`: 4.5:1 for text on its background, and 3:1 for a bar, mark, or icon. Steps of `gray` and `indigo` that no token takes any more are removed. `chosen.toml` is renamed from "Ink" to "Deep River and Shallows", and its summary and comment say what it is now.

If a pairing can't pass without losing the hue, the step moves, not the rule. If river and moss can't be told apart in the simulation the specimen shows, stop and report: that changes the identity, not this phase.

### The type

A token `font.heading`: `ui-serif, Charter, "Bitstream Charter", Cambria, Georgia, serif`. The site's `h1` to `h3`, the hero, and the report frame's headings take it. Weights stay the type scale's. A serif with only regular and bold draws 650 and 750 as bold, which is acceptable. Body, UI, and code fonts don't change. The element library still takes the site's type.

Look at it on macOS (New York), Windows (Cambria), and Linux (whatever `serif` resolves to). The site's e2e test runs in Chromium on Linux in CI, which is the fallback a reader is likeliest to get. If the Linux fallback reads badly, put a better-installed serif ahead of `serif` in the stack, and write down why.

### The headline

In `site/home/content/index.md`:

- The hero's heading is the story's headline, and its sentence is the story's sentence: "Ascribe is a toolchain for documentation written as code: Markdown with the structure docs need, checked as you write, and built into a website, plain Markdown, and JSON."
- The `description` is the same sentence.
- The `intended` entry's reason says the hero holds the headline.

Nothing else on the page changes in this phase. `node scripts/design/assets.ts` then redraws `site/public/cards/index.png` from the new heading.

### The record

- **A decision,** `project-docs/decisions/<issue>-pages-palette.md`, in the section "The look". It says that the shared colors are Deep River and Shallows, what each hue is for, that headings are serif, and why. It changes decisions 44 to 48's particulars only where they name the old palette or fonts.
- **A changelog entry** in the area `changes/README.md` gives the element library. It's a **Behavior change**: the `--ascribe-*` and `--ascribe-review-*` defaults moved, with the previous values to paste (decision 44).

## Tasks

1. Add the `deep`, `river`, `moss`, `amber`, `red`, `magenta`, and `yellow` steps the roles need. Point every shared `color.*` token at them, add their pairings to `pairs.toml`, and remove the steps nothing takes.
2. Rebless the generated blocks and `chosen.toml`, rebuild the report's embedded stylesheet, rewrite the specimen and the mark's images, and read each diff.
3. Add `font.heading`, and have the site and the report frame take it for headings.
4. Change the landing page's hero and description.
5. Retake every screenshot `scripts/docs-site/screenshots.ts` names.
6. Build the site with this checkout. Look at every kind of page in both schemes, at desktop and phone width, and check that the header's wordmark is sharp at 18 px.
7. Write the decision and the changelog entry.

## Out of scope

- Any change to the landing page's layout or sections, or to Why Ascribe and Credits (phase 2).
- The split bar on notes, and the mark's motion (phase 4).
- Any new `--ascribe-*` property. Only defaults change.
- The markup the site output writes: `tests/render/` doesn't change.

## Acceptance criteria

- `pnpm exec vitest run scripts/design` passes, with a pairing in `pairs.toml` for every new text or meaningful graphic color.
- No shared token points at `gray` or `indigo`, and the specimen shows no indigo.
- `node scripts/compare/outputs.ts --base main` differs only in the HTML report's files, which are accepted by name, with the label `outputs changed`.
- The built site, the report, and the Astro toolbar look right in both schemes. Screenshots of each, light and dark, are in the pull request.
- ascribed-dev.com's landing page reads "Docs as code, with the tools docs need." after the merge's deploy.
- The maintainer has looked at the specimen and approved the palette.

## Verify

```sh
ASCRIBE_BLESS=1 pnpm exec vitest run scripts/design/tokens.test.ts
pnpm --filter @ascribed/review embed
node scripts/design/assets.ts
node scripts/design/specimen.ts
pnpm exec vitest run scripts/design
node scripts/docs-site/screenshots.ts
node scripts/compare/outputs.ts --base main
cargo build -p ascribe-cli && pnpm install
cd site && npm ci && npm run build:checkout && npm test && npm run preview
```

Then the full set in `AGENTS.md`.

## Commits

1. "Derive the shared colors from Deep River and Shallows"
2. "Set headings in the system's serif"
3. "Lead the landing page with the story's headline"
4. "Retake the screenshots in the new colors"
5. "Record the pages' palette"
