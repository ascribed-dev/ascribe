# Phase 1: The token source

Part of [Visual design](README.md). Needs no other phase. TypeScript and CSS; no Rust beyond the regenerated report stylesheet.

## Goal

Every color, font stack, space, and radius in Ascribe's stylesheets is written once, in `design/tokens.toml`, with today's values. The stylesheets' variables are generated from it. Nothing looks different: every custom property computes to the value it has now.

This is the refactor that makes every later phase a change to one file.

## Context

- The stylesheets, in the README's table. Read each one's `:root` blocks first: `packages/elements/css/style.css` declares light values, then repeats them inside `@supports (color: light-dark(#000, #fff))` with `light-dark()`, and has `data-ascribe-scheme` rules; `site/src/styles/site.css` uses `@media (prefers-color-scheme: dark)`; the report and the marks have their own arrangements. The generator reproduces each file's arrangement. It doesn't unify them.
- `packages/elements/README.md`: the table of `--ascribe-*` properties sites may set. `packages/elements/CONTRACT.md`: the names are the contract.
- `packages/review/embed.ts` (`pnpm --filter @ascribed/review embed`): builds `crates/ascribe-diff/src/html/report.css` from the elements', marks', and report's stylesheets.
- `crates/ascribe-core/src/names.rs` and the `names.ts` generated from it in each package: the repository's pattern for one home with generated copies and a test that fails on a literal elsewhere. `packages/vscode/test/unit/docs.test.ts`: the pattern for a generated block rewritten with `ASCRIBE_BLESS=1`.
- `scripts/`: where repository scripts and their tests live (`scripts/compare/outputs.test.ts`). `smol-toml` is already a dependency of two packages.
- `packages/vscode/src/webview/preview.css`: sets `--ascribe-*` from `--vscode-*`, with the element library's light values as fallbacks.

## Design

### The source

`design/tokens.toml`, in two tiers (README decision 3):

```toml
[palette.blue]
600 = "#0969da"
400 = "#4493f8"

[color.accent]
light = "blue.600"
dark = "blue.400"

[color.note]
light = "blue.600"
dark = "blue.400"

[font]
ui = 'system-ui, -apple-system, "Segoe UI", Roboto, sans-serif'
mono = "ui-monospace, SFMono-Regular, Menlo, Consolas, monospace"

[space]
base = "1rem"
small = "0.5rem"
```

A third section maps each stylesheet's variables to semantic tokens, so the public names stay and the private ones can differ:

```toml
[emit."packages/elements/css/style.css"]
"--ascribe-note-color" = "color.note"
"--ascribe-muted-color" = "color.muted"
```

Choose the palette's names and steps from what's there. Where two stylesheets use slightly different values for the same idea (`--muted: #59636e` on the site, `--ascribe-muted-color: #57606a` in the elements, `--r-muted: #5b6475` in the report), keep each value in this phase, as its own palette entry, and list the near-duplicates in the pull request. Merging them changes how things look, and is phase 4's and phase 5's job.

### The generator

`scripts/design/tokens.ts` reads the source and rewrites the variable declarations of each stylesheet between two marker comments, which say what generates them. Everything outside the markers is left alone. Run with `ASCRIBE_BLESS=1` it writes; otherwise its test fails when a file is stale.

Non-color properties that aren't shared (`--header-height`, `--ascribe-steps-marker-size`) stay in the stylesheet, outside the markers. A token is something more than one place uses, or a color.

### The checks

- **Stale:** each generated block matches the source.
- **No literal colors:** no hex, `rgb()`, `hsl()`, or named color outside the generated blocks of the six stylesheets, apart from `transparent`, `currentColor`, and `inherit`. `preview.css` keeps its `--vscode-*` references; its hex fallbacks are generated.
- **Nothing changed:** a test resolves every custom property of every stylesheet, in light and in dark, before and after, and compares. Write the "before" as a fixture from `main` in the first commit, compare in the second, and delete the fixture in the last: it's proof for this pull request, not a test to keep.

## Tasks

1. An inventory in the pull request description: every distinct value, where it's used, and the near-duplicates.
2. `design/tokens.toml` with today's values, and `design/README.md` saying what the file is and how to regenerate.
3. `scripts/design/tokens.ts` and its test, with the three checks.
4. The marker comments and generated blocks in each stylesheet. `pnpm --filter @ascribed/review embed` to regenerate the report's stylesheet.
5. `node scripts/compare/outputs.ts --base main`: only the HTML report's files differ, and only inside their stylesheet. Accept them by name.
6. `ARCHITECTURE.md`: a row in the one-home table (tokens: `design/tokens.toml`), a row in the `ASCRIBE_BLESS` table, and `design/` in the map.

## Out of scope

Changing any value; merging near-duplicates; renaming a public variable; the mark and assets.

## Acceptance criteria

- Every custom property computes to what it did on `main`, in light and in dark.
- A hex value added to a stylesheet outside a generated block fails a test that names the file and line.
- Changing one palette value and blessing changes every stylesheet that uses it, and nothing else.
- `packages/elements/README.md`'s list of properties is still complete; if it can be generated from the `emit` section, generate it.

## Verify

```sh
pnpm format:check && pnpm lint && pnpm typecheck && pnpm test
pnpm --filter @ascribed/review embed && git diff --stat crates/ascribe-diff/src/html/report.css
cargo test --workspace --locked
node scripts/compare/outputs.ts --base main --accept '<the report files it lists>'
```

## Commits

1. "Record every stylesheet's computed variables"
2. "Write colors, fonts, and spacing once: design/tokens.toml"
3. "Generate each stylesheet's variables from the tokens"
