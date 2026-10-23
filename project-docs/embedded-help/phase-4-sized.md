# Phase 4: Summaries checked and shown

Part of [Embedded help](README.md). Needs phase 3.

## Goal

`ascribe check` reports a permalink with no summary, and a summary too long for a tooltip; the editor shows a section's summary; and agents are told to write one.

In a page with `@id {permalink=true}: connect` and no `@summary` in its section, `ascribe check` gives advice `summary-missing` at the `@id` line: the permalink has no summary, so a product showing it has none, and the fix is a `@summary`. A section whose `@summary` is 420 characters gets advice `summary-long`: a product shows under 300. A project that ships embedded help sets `summary-missing = "warning"` in `[checks]`, and every permalink then needs its summary. In VS Code, the hover on a section's heading or its `@id` line shows its summary, and This page's Permalink row shows it in its tooltip.

## Context

- Phase 2's decision: the check is advice by default, so a project that only links to its permalinks isn't asked for summaries.
- Phase 3's measurements: how many of the docs' permalinks have a summary once the diagnostics do.
- `description-too-long` (`tests/conformance/diagnostics.toml`): how a length is measured and reported, and the limit's key in `[checks]`. `crates/ascribe-check/src/page/`.
- `crates/ascribe-lsp/src/hover.rs`; the language server's `ascribe/page` rows (`crates/ascribe-lsp/src/page_rows.rs`) and This page's Permalink row in `packages/vscode/src/ui/sidebar.ts`.
- The rules `ascribe agents sync` writes, the skill, and `ascribe check --format prompt`.

## Design

- **Two checks, so a project can raise one without the other,** each advice and configurable in `[checks]`:
  - **`summary-missing`:** a section with a permalink and no `@summary`. A page's own permalink is covered by its description, which `page-description-missing` already reports.
  - **`summary-long`:** any section's summary at or over a `limit`, 300 characters unless set, measured as `description-too-long` measures. Any section, since a summary is general.
  - A permalink or a summary in a fragment is reported once, in the fragment's own file, as `check` reports a fragment's problems, so a reader isn't sent to a page that includes it.
- **The editor:** the language server's hover on a section's heading and on its `@id` line shows the section's summary, and its page rows give it for the Permalink row, from the same function `help.rs` uses, so the editor and the file can't disagree. The extension shows it in the row's tooltip.
- **Agents:** the rules and the skill say to give a section with a permalink a `@summary`, and how to choose between the two bindings; `--format prompt` lists the missing ones with the section's text, so an agent can write each.

## Tasks

1. The two checks, in `diagnostics.toml` and `diagnostics.rs` in the same order, with conformance cases: a permalink with no summary, with one of each binding, a page's own permalink, a fragment's, a summary over the limit and under it, the limit set, and `summary-missing` raised to a warning.
2. The docs' own advice from them: every heading permalink outside the diagnostics given a `@summary` (59 sections at phase 1's count), choosing the following binding where a sentence on the page reads alone. Written by an agent from the prompt and read by a person; in a pull request of its own if it's large.
3. The summary function shared between `help.rs` and the language server.
4. The hover and the page rows, with language server tests; the Permalink row's tooltip, with the extension's unit and webview tests.
5. The rules, the skill, and the prompt, with the agents' tests.
6. The docs: the checks' reference (generated), the embedded help page on requiring summaries and keeping them short, `docs/content/editor/sidebar.md`'s Permalink row and the hover; `changes/language/summary-checks.md`, `changes/editor/section-summary.md`, and `changes/agents/section-summary.md`.

## Out of scope

Fetching, arms, and `resolve` (phases 5 to 7).

## Acceptance criteria

- `ascribe check --config docs` reports no `summary-missing` and no `summary-long`.
- The goal's hover and row, in tests.

## Verify

```sh
cargo test -p ascribe-conformance summary
cargo test -p ascribe-lsp summary
cargo run -p ascribe-cli -- check --config docs
```

## Commits

1. "Report a permalink with no summary, and a summary too long"
2. "Show a section's summary in the editor"
3. "Tell agents to give a permalinked section a summary"
4. "Give the docs' permalinked sections their summaries"
