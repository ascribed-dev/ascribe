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

_To be filled in by the implementing agent._
