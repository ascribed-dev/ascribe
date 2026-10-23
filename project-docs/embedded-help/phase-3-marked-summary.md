# Phase 3: The marked summary

Part of [Embedded help](README.md). Needs phases 1 and 2.

## Goal

A writer marks a section's summary, and the docs' diagnostics show their **When** paragraphs.

In the docs, after `ascribe build --config docs`, `help.json`'s `ASC149` entry has the summary "A page's Markdown, as a build writes it for agents to read, is as long as the limit or longer: …": not the metadata line, and not "When: …" with the paragraph's label, which phase 2 settled how to leave out. In a page, the form phase 2 settled, around a paragraph under a permalink's heading, makes that paragraph the entry's summary, wherever it is in the section.

## Context

- Phase 2's decision file: the form and its rules.
- README decision 10, which the marker now comes before.
- SPEC §4 and Appendix A; `crates/ascribe-syntax` (the directive's parse and its canonical form), `crates/ascribe-resolve` (the directive kept or dropped), `crates/ascribe-emit` (each emitter writing its content), `crates/ascribe-lsp` (completion and hover of directives).
- `tests/conformance/diagnostics.toml` and `crates/ascribe-core/src/diagnostics.rs`, for the errors phase 2 named.
- `tests/conformance/tests/docs.rs`, and `docs/content/_generated/diagnostics-*.md`.

## Design

- **The directive,** as phase 2 settled: parsed, formatted canonically, resolved, and written by each emitter as phase 2 says: its content in the plain output, what `site-render.md` now says in the site output, with review's source anchors on the marked paragraphs, and its shape in the JSON output. The two renderers agree (`tests/render/`).
- **The summary:** the marked text when the section has a marker; else decision 10. `help.json`'s keys don't change.
- **The diagnostics:** the errors and the advice phase 2 named, in `diagnostics.toml` and `diagnostics.rs`, in the same order.
- **The generator** marks each diagnostic's **When** paragraph, as phase 2 settled for its `**When:**` label, and the reference is blessed again.

## Tasks

1. The directive in the syntax, `fmt`, resolve, and the three emitters, with conformance cases for its parse, its canonical form, a marked paragraph's summary, and each diagnostic.
2. The summary rule in `help.rs`, with emit tests for a marker in the middle of a section, in a fragment, and with none.
3. The language server: completion of the directive, and its hover.
4. The generator, and the diagnostics reference blessed again, its diff read.
5. Phase 1's measurements taken again on the docs' file, in the pull request.
6. The docs: the directive in `docs/content/writing/directives.md`, `@available: ascribe planned`; the embedded help page's summary section; SPEC §4 and Appendix A; `site-render.md` and the JSON output's reference; `changes/language/summary-marker.md`; the surface map.

## Out of scope

The check of a summary's length, and the editor's preview (phase 4).

## Acceptance criteria

- The docs' `help.json` has every diagnostic's **When** sentence as its summary, without the `When:` label.
- Conformance cases for the directive and each of its diagnostics.
- The site and plain outputs of every project are byte for byte the same where no marker is written; the diagnostics reference's are its marked paragraphs, named with `--accept`.

## Verify

```sh
cargo test -p ascribe-conformance
cargo test -p ascribe-emit help
cargo run -p ascribe-cli -- build --config docs
node scripts/compare/outputs.ts --base main
```

## Commits

1. "Mark a section's summary"
2. "Take a section's summary from its marker"
3. "Mark each diagnostic's When paragraph as its summary"
