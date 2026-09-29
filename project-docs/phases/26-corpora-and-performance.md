# Phase 26: Corpora and performance

**Track:** Tests · **Start after:** 14, 15, 18 · **Parallel with:** 21 and later waves · **Unblocks:** 27

## Goal

Test Ascribe against real documentation at scale: catch false positives in recognition, and prove the performance targets.

## Read first

- [PLAN.md](../PLAN.md): Testing (real-world corpora, performance targets).
- Handoff notes from phases 13, 14, 15, and 18.

## Deliverables

- `tests/corpora/`: scripts that fetch the corpora and convert parts of them to Ascribe, plus the tests that use them.
- Benchmarks (for example with `criterion`) for `ascribe check`, `ascribe build`, and language-server response times.
- A generated synthetic project of about 3,000 pages for repeatable performance tests.

## Tasks

1. **Fetching, not vendoring.** Fetch the corpora at test time (shallow, sparse clones at pinned commits): `withastro/docs` (`src/content/docs/en`), `elastic/docs-content`, and `docker/docs` (`content`). Don't commit their content. Check each repository's license, and record the result, before using its content in any committed fixture.
2. **Recognition checks.** Run Ascribe's parser over the unconverted markdown and assert it finds no directives, titles, or phrases where none were intended. Prose `@` (`@astrojs/react`, `@timestamp`), braces in prose, and lines starting with `.` are the main risks.
3. **Conversion.** Scripts converting a sample of each corpus's constructs to Ascribe: asides and admonitions to `@note`, tabs to `@variant` groups, substitutions to phrases, includes to `@include`, `applies_to` to `@available`. They don't need to be complete; they need to produce realistic Ascribe at volume.
4. **Performance.**
   - A full `ascribe check` of the 3,000-page synthetic project and of the converted Elastic sample, targeting a few seconds on a developer laptop.
   - Language-server diagnostics after a keystroke, and completion responses, targeting about 50 ms.
   - Record results in `tests/corpora/RESULTS.md`, and add a CI job that fails on large regressions.
5. **Findings.** File each false positive or performance problem as a question or issue, with a minimal reproduction, for the owning phase.

## Acceptance criteria

- [ ] Recognition checks run over all three corpora with no unexplained false positives.
- [ ] Performance results are recorded and meet the targets, or each miss has a documented cause and a follow-up.
- [ ] CI runs the benchmarks and catches regressions.

## Out of scope

- Fixing the problems found, beyond small fixes in this phase's own code.

## Handoff notes

### What was built

`tests/corpora/` (crate `tessera-corpora`, with a README) and `tests/corpora/synthetic/` (crate `tessera-synthetic`):

- **`tessera-synthetic`**: the shared 3,000-page project (`Synthetic::standard()`: pages, fragments, images, model; `write_to`, `sources`, `page_text` with a typing line) and `report::record`, which appends a benchmark's median and p95 as a JSON line when `ASCRIBE_BENCH_OUT` is set. It has no dependencies, so any crate can use it as a dev-dependency without a cycle. The keystroke bench (phase 15) and the incremental bench (phase 13) now use it; their numbers are unchanged (RESULTS.md).
- **Corpora** (`src/corpus.rs`): shallow, sparse, blobless clones at pinned commits into `target/corpora/`; `ASCRIBE_CORPORA` = unset or `skip` (cache only; a `SKIPPED:` line when a corpus isn't cached, so `cargo test --workspace` stays offline and fast), `fetch` (fetch, skip on failure), `require` (fetch, fail on failure; the CI job). Licenses are in `LICENSES.md`: Astro MIT, Docker Apache-2.0, **Elastic CC BY-NC-ND 4.0, so nothing of it or derived from it is committed**.
- **Recognition** (`src/recognize.rs`, `tests/recognition.rs`, `recognition.toml`, `baselines/recognition-*.json`): classes of recognition with a recorded verdict each; a class with none fails; counts are pinned; an independent line scan checks prose `@` and `.` lines. `tests/edge.rs` has the hand-written reproductions, offline.
- **Conversion** (`src/convert/`): Elastic (asides, dropdowns, tab sets, steppers, includes, images, substitutions to phrases with values from `docset.yml`, `applies_to` to `@available`, heading anchors to `@id` with links rewritten), Astro, Docker. `tests/conversion.rs`: fixtures offline; each corpus converted and checked, with ceilings on error and warning counts.
- **Benchmarks and regression check**: `benches/perf.rs` (CLI wall time and library phases, synthetic and Elastic), `corpora compare` against `baselines/perf.json` (3x plus 25 ms, and the spec's targets), `.github/workflows/corpora.yml` (`workflow_dispatch` only).
- **Questions Q191 to Q196**, and `FINDINGS.md` (recognition R1 to R5, language F1 to F5, performance P1 to P4).

### Interfaces later phases use

- **Phase 16's completion benchmark**: `crates/tessera-lsp/benches/completion.rs`, using `tessera_synthetic::Synthetic::standard()` (already a dev-dependency of `tessera-lsp`) and `tessera_synthetic::report::record("lsp/completion-3000", &mut times)`. The workflow runs it when the file exists and `baselines/perf.json` already has its 50 ms target. Then add its row to RESULTS.md. Not measured here: phase 16 hadn't merged.
- **Re-recording baselines**: `corpora compare results.jsonl --record`, on the machine the job runs on.
- **A new benchmark** calls `report::record` and, if it must not go missing, is added to `REQUIRED` in `src/perf.rs`.

### Results against the targets

3,000-page synthetic `ascribe check` 0.78 s (met); keystroke 3.0 ms page, 9.0 ms fragment (met, 50 ms). **Converted Elastic sample (3,008 pages, 21 MB) 7.4 s: a miss on this container**, with the cause and follow-ups in RESULTS.md and FINDINGS P1 to P3. The text report (the default output) takes over 100 s on that sample: FINDINGS P1, a real problem in phase 10's `report/text.rs`.

### Decisions

- **No recognition false positives were found**; the finding is that the corpora hardly contain the risky cases (one prose line starting with `@`, two with `.`), so the edge tests carry the weight.
- **The Elastic converter does not widen a page's `available`** to cover its blocks (F3/Q194) and leaves unconvertible constructs as written; the errors that remain are findings or listed converter limits, not hidden.
- **Ceilings, not exact counts**, on converted diagnostics (a tenth plus five), because the checks of other phases change; the recognition counts are exact because the parser's recognition shouldn't move unnoticed.
- The benchmarks call the command's **JSON** report for Elastic (the text report is P1).

### Left open

- **P1 to P4** and **F1 to F5** (Q191 to Q196) are for their owners; nothing in the compiler, `crates/tessera-lsp/src`, or `packages/` was changed.
- **Completion isn't measured** (above).
- **CI has not run**: it's manual-only, and the job was exercised by running its commands locally on Linux only. The Windows and macOS runners aren't in it.
- **Elastic in CI** depends on the license reading in `LICENSES.md`: the tests use the text without committing anything derived from it. If that's in doubt, leave `ASCRIBE_CORPORA` unset for it.
