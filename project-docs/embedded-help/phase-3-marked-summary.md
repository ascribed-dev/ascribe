# Phase 3: Section summaries

Part of [Embedded help](README.md). Needs phases 1 and 2.

## Goal

A writer gives a section a summary with `@summary`, and the docs' diagnostics have theirs.

In the docs, after `ascribe build --config docs`, `help.json`'s `ASC149` entry has the summary "A page's Markdown, as a build writes it for agents to read, is as long as the limit or longer.", from the `@summary:` line the generator writes, not the metadata line. `cli-versions-release`'s, with `@summary` before a first sentence made a paragraph of its own, is "Releases a line.", and the page shows that sentence where it was. A section with neither has no `summary`: nothing is guessed from its prose.

## Context

- Phase 2's decision file: the two bindings and each rule.
- README decision 10, which this phase replaces.
- SPEC §3 (the directive line, and a binding of "self (with primary), else following block", as `@note` has) and §4; Appendix A.
- `crates/ascribe-syntax` (the directive's parse and its canonical form), `crates/ascribe-resolve` (the summary attached to its section), `crates/ascribe-emit` (each emitter, and `site/help.rs`), `crates/ascribe-lsp` (completion and hover of directives).
- `tests/conformance/diagnostics.toml` and `crates/ascribe-core/src/diagnostics.rs`, for the errors phase 2 named.
- `tests/conformance/tests/docs.rs`, which writes `docs/content/_generated/diagnostics-*.md` from `diagnostics.toml`.

## Design

- **The directive,** as phase 2 settled: parsed with both bindings, formatted canonically, and resolved onto its section, so a section's summary is one value however it was given. The line form writes nothing in the site and plain outputs' bodies; the following form's block is written as it would be without it. The JSON output carries the summary on the section's heading. The two renderers agree (`tests/render/`).
- **The summary in `help.json`:** the section's `@summary`, as plain-output Markdown, or absent. Decision 10's first-paragraph rule is removed. For a page's own permalink, decision 2 holds: the page's description.
- **The diagnostics:** the errors phase 2 named, in `diagnostics.toml` and `diagnostics.rs`, in the same order.
- **The generator** writes `@summary: <when>` under each diagnostic's `@id`, and the reference is blessed again. Each diagnostic without a `when` gets one in `diagnostics.toml`: one sentence saying when it's reported, as the others do. If those 85 sentences make the pull request too large to read, they go first, in a pull request of their own, which changes the reference and nothing else.

## Tasks

1. The directive in the syntax, `fmt`, resolve, and the three emitters, with conformance cases for both bindings' parse and canonical form, a summary in a fragment, one in a variant arm, two in a section, and each diagnostic.
2. `help.rs` reads the section's summary, and the first-paragraph rule goes, with emit tests for each binding, a fragment, and a section with none.
3. The language server: completion of the directive, and its hover.
4. The 85 `when` sentences, and the generator, with the diagnostics reference blessed again and its diff read.
5. Phase 1's measurements taken again on the docs' file, in the pull request: how many entries have a summary, and how long.
6. The docs: the directive in `docs/content/writing/directives.md`, `@available: ascribe planned`, with both bindings; the embedded help page's summary section; SPEC §3, §4, and Appendix A; `site-render.md` and the JSON output's reference; a decision file is phase 2's; `changes/language/section-summary.md`; the surface map.

## Out of scope

The check of a missing or long summary, the editor's preview, and telling agents to write summaries (phase 4).

## Acceptance criteria

- The docs' `help.json` has every diagnostic's `when` as its summary.
- No entry's summary comes from a section without `@summary`.
- Conformance cases for both bindings and each diagnostic.
- The site and plain outputs of every project are byte for byte the same where no `@summary` is written, and the diagnostics reference's differ only by its `@summary` lines and its new `when` sentences, named with `--accept`.

## Verify

```sh
cargo test -p ascribe-conformance
cargo test -p ascribe-emit help
cargo run -p ascribe-cli -- build --config docs
node scripts/compare/outputs.ts --base main
```

## Commits

1. "Say when each diagnostic is reported"
2. "Give a section a summary with @summary"
3. "Take help's summary from the section's @summary"
4. "Give each diagnostic its when as its summary"
