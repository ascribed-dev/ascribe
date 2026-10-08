# Phase 4: Elements, review, and the report

Part of [Visual design](README.md). Requires phases 1 and 2. Tokens, CSS, and a contract change.

## Goal

The chosen palette and type scale become the default look of what Ascribe puts on other people's sites and in its report: the element library, review's marks and overlay, and the HTML report's frame. The three stop disagreeing about what "muted" or "accent" is.

## Context

- `design/tokens.toml` and its generator (phase 1); `design/candidates/chosen.toml` (phase 2).
- `packages/elements/CONTRACT.md` and the README's property table. The names are the contract. The default values aren't promised in so many words, but every site that didn't override them changes how it looks when they change.
- `packages/review/src/marks/marks.css`: `--ascribe-review-*`, documented as themable.
- `packages/review/src/report/report.css`: `--r-*`, private.
- `packages/vscode/src/webview/preview.css`: the preview's fallbacks are the element library's values.
- `packages/elements/test/` and `packages/review/test/`: what's tested in a browser with `playwright-core`.
- `AGENTS.md`, "Contracts don't move", and `project-docs/decisions.md`: how a contract change is recorded.

## Design

- **Move the chosen palette into `design/tokens.toml`** and bless. Merge the near-duplicates phase 1 listed: one muted, one border, one accent, one focus color across the elements, review, and the report.
- **The contract change.** Record a decision in `project-docs/decisions.md`: the names of `--ascribe-*` and `--ascribe-review-*` are stable; their default values are the design tokens' and may change in a release, noted in the changelog under **Behavior change**. Say the same in `packages/elements/README.md` and `CONTRACT.md`, so a site that needs a fixed look knows to set the properties.
- **No new public names** unless the palette needs one the old set can't express. If it does, stop and report: a new property is a contract addition with its own line in the decision.
- **The type scale** applies to the report's frame and review's overlay, which have their own text. The elements keep `--ascribe-font-family: inherit` and sizes in `em`: they take the site's type, not Ascribe's.
- **The report's frame** uses the same semantic tokens as the docs site will. Rename `--r-*` only if it makes the stylesheet clearer; it's private.
- **High contrast and forced colors.** Check the elements and the marks under `forced-colors: active`. What they already do must still hold.

## Tasks

1. The palette in the token source; blessed stylesheets; the report's stylesheet regenerated with `pnpm --filter @ascribed/review embed`.
2. The contrast test from phase 2, now run on the real tokens and on every pairing the three stylesheets use.
3. The decision in `decisions.md`, and the wording in the elements' README and `CONTRACT.md`.
4. By eye, in light and dark: every note type, steps, tabs, each availability state, glossary links, each kind of review mark with and without a thread, and the report on `examples/quill` (`ascribe diff --format html`). Before-and-after screenshots in the pull request.
5. `node scripts/compare/outputs.ts --base main`: only the HTML report's files differ. Accept them by name.
6. `CHANGELOG.md`, under **Behavior change**: the defaults changed, and how to keep the old ones (a block of the previous values to paste). `docs/content/guides/astro.md` or wherever theming is described: update any value it quotes.

## Out of scope

The docs site's own styles (phase 5); anything in the VS Code extension beyond the preview's generated fallbacks (phase 6); new elements or layout changes.

## Acceptance criteria

- One value each for muted text, borders, accent, and focus across the three stylesheets, from one token.
- Every pairing passes decision 5's contrast, in light and dark, by test.
- No public property was renamed or removed. A test compares the set of `--ascribe-*` names with the list from `main`.
- The changelog gives a site everything it needs to keep its old look.

## Verify

```sh
pnpm format:check && pnpm lint && pnpm typecheck && pnpm test
pnpm --filter @ascribed/review embed
cargo test --workspace --locked
node scripts/compare/outputs.ts --base main --accept '<the report files it lists>'
```

## Commits

1. "Record that the elements' default values follow the design tokens"
2. "Use the chosen palette in the elements, review, and the report"
