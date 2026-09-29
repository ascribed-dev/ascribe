# Rename: Tessera becomes Ascribe

A brief for the agent doing the rename. It stands alone: read it in full before starting.

## Decision

The repository owner decided on 2026-09-29 that the product's public name is **Ascribe**. "Tessera" becomes the internal codename only.

- **npm org:** `@ascribed`. It's claimed; `@ascribe` wasn't available.
- **GitHub org:** `ascribed-dev` (https://github.com/ascribed-dev), claimed; `ascribed` wasn't available. The repository stays at `KyleBlankRollins/tessera` until the owner moves it into the org, and GitHub redirects the old URLs after a move. So this rename doesn't change repository URLs.
- **VS Code Marketplace publisher:** not claimed yet (phase 27).
- **Diagnostic codes** change prefix: `TSR` becomes `ASC`, with the same numbers (`TSR041` becomes `ASC041`).

Rename everything a user writes, types, or sees. Keep internal names as they are.

## Rules

- Work on the branch `rename/ascribe`, from `main`. Never push to `main`, publish, or create releases.
- This task is authorized to edit `SPEC.md`: rename only, with no other changes to the language.
- No other phase runs while this is open. Keep the PR a pure rename: no behavior changes beyond names, and no refactoring.
- End commit messages with `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>` and the PR description with `🤖 Generated with [Claude Code](https://claude.com/claude-code)`.
- CI is manual-only (`workflow_dispatch`); keep it so. Run every check locally (see "Verification").
- If a commit, push, or PR step is blocked, stop and say exactly what's unfinished.

## What changes

| Surface | From | To |
|---|---|---|
| Product and spec name, in prose | Tessera | Ascribe |
| Command (binary) | `tessera` | `ascribe`: `[[bin]] name` in `crates/tessera-cli/Cargo.toml`, `CARGO_BIN_EXE_tessera` in tests, clap's command name, help and error text |
| Config file | `tessera.toml` | `ascribe.toml`: the constant the loader and the language server look for, discovery, messages, docs, and every fixture file (`git mv`; 14 tracked files are named `tessera.toml`) |
| Default output directory | `.tessera/build` | `.ascribe/build`: the model default, docs, `.gitignore` entries, and paths in examples and tests |
| Site-output folder | `_tessera/` (`_tessera/schema.ts`, `_tessera/files/`) | `_ascribe/`: the emitter constants, the asset contract, the Astro integration, and examples |
| Custom elements | `<tessera-note>`, `<tessera-tabs>`, `<tessera-tab>`, `<tessera-steps>`, `<tessera-availability>`, `<tessera-availability-target>`, `<tessera-group>` | `<ascribe-…>`: the emitter, `packages/elements` (registration and CSS), `packages/elements/CONTRACT.md`, SPEC §9, snapshots, and tests |
| Attribute marker | `<tessera-attributes>` | `<ascribe-attributes>`: the emitter, `render_site_html`, the Astro plugin (`packages/astro/src/attributes.ts`), `project-docs/contracts/site-render.md`, and every `tests/render/*/input.md` and `expected.html` |
| CSS custom properties | `--tessera-*` | `--ascribe-*` |
| Tab-choice storage key | `tessera-tabs:<sync>` | `ascribe-tabs:<sync>` |
| Reserved widget prefix | Names starting with `tessera-` are reserved (content-model.md §15, Q9) | `ascribe-`: the model rule, its message, and its tests |
| Diagnostic codes | `TSR001`–`TSR125` | `ASC001`–`ASC125`: `tests/conformance/diagnostics.toml`, `crates/tessera-core/src/diagnostics.rs`, any prefix constant or parser, every expectation and snapshot, and the docs |
| npm packages | `@tessera/cli`, `@tessera/astro`, `@tessera/elements`, `@tessera/zod-check`, `@tessera/example-astro-site` | `@ascribed/cli`, `@ascribed/astro`, and so on. Update every import and `--filter` (package.json scripts, `.github/workflows/js.yml`, READMEs), then run `pnpm install` to regenerate `pnpm-lock.yaml`. All packages stay `private`. |
| VS Code extension | `tessera-vscode`, display name "Tessera", publisher `tessera` | `ascribe-vscode`, display name "Ascribe". Leave `publisher` as it is: phase 27 claims one. |
| VS Code settings and commands | `tessera.path`, `tessera.trace.server`, `tessera.formatOnSave`, `tessera.maxCrashes`, `tessera.restartServer`, `tessera.showOutput`, `tessera.minServerVersion`, the `Tessera` category and output channel | `ascribe.*` and "Ascribe" |
| Extension activation | `workspaceContains:**/tessera.toml`, and the `**/tessera.toml` document selector | `**/ascribe.toml` |
| Semantic tokens | Types `tesseraDirective`, `tesseraWidget`, and the rest, and scopes ending `.tessera` | `ascribeDirective` and so on, with scopes ending `.ascribe`. The language server's legend (`crates/tessera-lsp`, and the legend table in its README) and `packages/vscode/package.json` must match, because a unit test compares them. |
| TextMate grammars | Scope names `tessera.injection` and `tessera.injection.nested`, and scopes ending `.tessera` | `ascribe.…`, including the grammar tests' expectations |
| Language server | Its server name, and `tessera lsp` | "Ascribe", `ascribe lsp` |
| Astro integration | Integration name `@tessera/astro`, virtual module `virtual:tessera/site` | `@ascribed/astro`, `virtual:ascribe/site`; also the example's `content.config.ts` schema import path |
| Environment variables | `TESSERA_BIN`, `TESSERA_CHROMIUM`, `TESSERA_ENGINES`, `TESSERA_SUITE`, `TESSERA_WORKSPACE`, `TESSERA_BLESS`, `TESSERA_FILES`, `TESSERA_LINE`, `TESSERA_INCREMENTAL_CASES`, `TESSERA_LSP_SEEDS` | `ASCRIBE_*`, including the workflows and READMEs |
| Living docs | `README.md`, `SPEC.md`, `project-docs/PLAN.md`, `content-model.md`, `contracts/`, `phases/` (including handoff notes, because later phases read them for interfaces), crate and package READMEs | Every user-facing name above. Phase 27's "Names" task should record what's claimed: the npm org `@ascribed` and the GitHub org `ascribed-dev`. The Marketplace publisher is still open. |

## What stays

- **Crate names and Rust identifiers:** `tessera-core`, `tessera_resolve::…`, `comrak-tessera`, and the others, plus the `crates/tessera-*` directory names. They're internal and never published.
- **The repository name and URLs:** `KyleBlankRollins/tessera`.
- **`project-docs/questions.md`:** it's the historical record. Don't rewrite its entries. Add one paragraph near the top saying that the project was renamed to Ascribe on 2026-09-29, that entries before that date use the old names, and give the short mapping (command, config file, elements, marker, npm scope, `TSR` to `ASC`).
- **`examples/comparison/README.md`:** the owner asked that it not be touched.
- **`packages/vscode/test/fixtures/markdown.tmLanguage.json`** and its license: VS Code's grammar, unchanged.
- **Git history and commit messages.**

## Suggested order

Rename one surface at a time, with a commit and a targeted test run after each, so a failure points at one change:

1. The diagnostic codes (`TSR` to `ASC`).
2. The command and the config file, including the fixture file renames.
3. The output directories: `.ascribe/build` and `_ascribe/`.
4. The elements, the marker, CSS, and the storage key, including the render fixtures, snapshots, and contracts.
5. The npm packages and the Astro integration.
6. The VS Code extension: settings, commands, tokens, grammars, and activation.
7. Environment variables and workflows.
8. Prose in the docs, then the note in `questions.md`.

Watch for these traps:
- **`insta` snapshots.** Update them by rerunning with `INSTA_UPDATE=always` or `cargo insta accept`, and read the diff: it should contain renames only.
- **Tests that read docs:**
  - `tests/conformance/tests/render_fixtures.rs` checks the site-render contract's §6 table against `tests/render/fixtures.toml`;
  - `site_quill.rs` checks elements against `CONTRACT.md`;
  - the VS Code manifest test and the token-legend test read `crates/tessera-lsp/README.md`.
- **Case variants:** `Tessera`, `tessera`, `TESSERA`, and camelCase (`tesseraDirective`).
- **Blind replacement breaks things.** Replacing every `tessera` would rename crates, `tessera_*` paths, and repository URLs. Replace by pattern, per the table.

## Verification

Before opening the PR, run all of these, and say in the PR how each ran:

- **Rust:** `cargo build`, `cargo test --workspace` (including the conformance suite: 364 cases, 0 skipped), `cargo fmt --all --check`, and `cargo clippy --workspace --all-targets -- -D warnings`.
- **JavaScript:** `pnpm install`, `pnpm -r lint`, `pnpm -r typecheck`, `pnpm -r test` (use `ASCRIBE_ENGINES=chromium` if Firefox and WebKit can't run), and `pnpm format:check`.
- **VS Code integration tests:** `ASCRIBE_BIN=$PWD/target/debug/ascribe xvfb-run -a pnpm --filter ascribe-vscode test:integration`. Expect `activation` 2, `stub` 8, and `quill` 5.
- **The Astro end-to-end test:** `pnpm --filter @ascribed/elements build`, then `pnpm --filter @ascribed/astro build`, `pnpm --filter @ascribed/astro test`, and `pnpm --filter @ascribed/example-astro-site test:e2e`.
- **A leftover check.** `git grep -niE "tessera"` must return only the kept names:
  - crate and module names, and `crates/tessera-*` paths;
  - `comrak-tessera`;
  - repository URLs;
  - `project-docs/questions.md`;
  - `examples/comparison/README.md`;
  - this brief.

  `git grep -nE "TSR[0-9]{3}"` must return only `questions.md`. List any other leftover in the PR, with the reason it stays.

Open the PR against `main`, titled "Rename Tessera to Ascribe". In the description, give the mapping table, what stayed and why, the check results, and any surface you weren't sure about.
