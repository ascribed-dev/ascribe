# Phase 4: Sized for a product

Part of [Embedded help](README.md). Needs phase 3.

## Goal

`ascribe check` reports a summary too long for a tooltip or missing, and the editor shows what a product gets for a permalink.

In a page with a permalinked heading whose first paragraph is 420 characters, `ascribe check` gives advice `help-summary` at the heading's `@id` line: the summary is 420 characters, a product shows under 300, and the fix is to mark a shorter paragraph. A permalinked section that opens with a list and marks nothing gets the same check's other message: it has no summary. In VS Code, the hover on `@id {permalink=true}: connect`, and This page's Permalink row, show the summary a product gets, or that there's none.

## Context

- Phase 1's and phase 3's measurements: how many of the docs' summaries are over 300 characters, and how many are missing, once the diagnostics are marked.
- `description-too-long` (`tests/conformance/diagnostics.toml`): how a length is measured and reported, and the limit's key in `[checks]`. `crates/ascribe-check/src/page/`.
- `crates/ascribe-lsp/src/hover.rs`; the language server's `ascribe/page` rows (`crates/ascribe-lsp/src/page_rows.rs`) and This page's Permalink row in `packages/vscode/src/ui/sidebar.ts`.

## Design

- **`help-summary`,** advice, configurable in `[checks]`, with a `limit` of 300 characters, measured as `description-too-long` measures: one message for a summary at or over the limit, one for a permalink with none. It runs for every permalink a build publishes, the entries `help.json` has. A permalink in a fragment is reported once, at its `@id` line in the fragment's own file, as `check` reports a fragment's problems, so a reader isn't sent to a page that includes it.
- **The editor:** the language server's hover on a permalink's `@id` line, and its page rows for the Permalink row, give the entry's summary, from the same function `help.rs` uses, so the editor and the file can't disagree. The extension shows it in the row's tooltip.

## Tasks

1. The check, in `diagnostics.toml` and `diagnostics.rs` in the same order, with conformance cases: over the limit, under it, a marked summary under it in a section whose first paragraph is over, none, a fragment's, and the limit set.
2. The docs' own advice from it, fixed or marked: each over-limit summary marked shorter, or acknowledged where the section is right as it is, as the measurements say.
3. The summary function shared between `help.rs` and the language server.
4. The hover and the page rows, with language server tests; the Permalink row's tooltip, with the extension's unit and webview tests.
5. The docs: the check's reference (generated), the embedded help page on keeping summaries short, `docs/content/editor/sidebar.md`'s Permalink row and the hover; `changes/language/help-summary.md` and `changes/editor/permalink-summary.md`.

## Out of scope

Fetching, arms, and agents (phases 5 to 7).

## Acceptance criteria

- `ascribe check --config docs` reports no `help-summary`.
- The goal's hover and row, in tests.

## Verify

```sh
cargo test -p ascribe-conformance help_summary
cargo test -p ascribe-lsp permalink
cargo run -p ascribe-cli -- check --config docs
```

## Commits

1. "Report a permalink's summary too long for a product, or missing"
2. "Show a permalink's summary in the editor"
