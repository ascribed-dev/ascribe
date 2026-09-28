# Phase 03: Conformance cases

**Track:** Tests · **Start after:** 00, 01, 02 · **Parallel with:** 05, 08, 09, 19 · **Unblocks:** the acceptance tests of every parser, check, and resolution phase

## Goal

Turn SPEC.md into executable expectations: a conformance suite covering every rule in §2–§6, the resolution and build rules in §9.2–§9.4, and every row of the §8.2 diagnostics table. Later phases prove themselves by making these cases pass.

## Read first

- [SPEC.md](../../SPEC.md), all of it. §3–§6, §8, and §9.2–§9.4 closely.
- `tests/conformance/README.md` (phase 00) for the case format.
- `project-docs/content-model.md` (phase 01), and the contracts and `diagnostics.toml` (phase 02).

## Deliverables

- `tests/conformance/_model/tessera.toml`: the shared fixture model, based on `examples/content-models/quill.toml`.
- Conformance cases under `tests/conformance/`, grouped by area.
- `examples/quill/`: a complete, valid Tessera project built around the SPEC Appendix B page.

## Tasks

1. **Settle interpretations first.** Before writing cases, list every spec interpretation the cases will lock in. Anything the spec doesn't settle goes into `project-docs/questions.md`, and its cases are tagged `provisional` until a human resolves it. Hand-written expectations become permanent quickly, so don't bake in a guess.
2. **Fixture model.** Copy the Quill model and extend it only as needed to exercise rules (for example, a project widget, a feature key, an extra note type, a lifecycle state that doesn't count as available).
3. **Cases for §2–§6.** For each normative rule, write at least one passing case and, where the rule can be broken, at least one failing case. Put every example from SPEC.md in a case. Cover in particular:
   - Recognition (§3.2): known and unknown keywords, `@` in code, mid-word `@`, escapes, misspelled directive shapes.
   - Attributes (§3.3): each value form, quoting rules, value sets, spacing tolerance, `{}`.
   - Primaries (§3.4): identifier versus text, text continuing across lines.
   - Forms (§3.5): a container opener has an empty primary after its `:`; a text primary that ends in `:` (`@note: Important:`) is a one-line note, not a container.
   - Groups and the arm rule (§3.6), titles including `. Title` (§3.7), binding including the top-of-section rule and blank-line warnings (§3.8), lists and blockquotes (§3.9), nesting (§3.10).
   - Each built-in directive (§4) and project widgets (§6).
   - Phrases, links, images (inline and every reference form: full `![a][r]`, collapsed `![a][]`, shortcut `![a]`, each with attributes), glossary, and heading ids (§5).
   - Source ids and page ids (§5.5): an include selecting a section by source id; a link to a heading whose page id differs from its source id because an included fragment duplicates its text.
4. **Cases for §9.2–§9.4 (project cases with `builds` expectations).**
   - A selection build that keeps several arms of a group (the cloud build keeps the `pm` group whole while reducing the `deployment` group to one arm).
   - Filter builds: before a history's first state, between states, at a state that doesn't count as available, and a versionless target marked `removed`.
   - Availability annotations retained in filter builds.
   - Assets: an image beside an included fragment, resolved from the fragment's location.
5. **Cases for §8.2.** At least one case per row that triggers exactly that diagnostic, at the expected line. Page-level rows need project cases with several files.
6. **The Quill project.** Build `examples/quill/`: `tessera.toml`, the Appendix B page, and the files it references (`_fragments/prerequisites.md`, `quickstart.md` with a `try-in-browser` id, `keys.md` with a `rotate-keys` id), plus at least one image, including one beside the fragment, so the project is free of errors under every build.
7. **Skips.** Keep a skip entry in `SKIPS.toml` for every tag until the phase that implements it removes the entry.

## Acceptance criteria

- [ ] Every SPEC.md example appears in at least one case.
- [ ] Every §8.2 row has at least one case that expects it.
- [ ] Every item in tasks 3 and 4 has at least one case.
- [ ] Every `expect.yaml` is valid under the phase 00 format, and the harness discovers every case (reported as skipped with a reason).
- [ ] Every interpretation the spec doesn't settle is recorded in `questions.md`, and its cases are tagged `provisional`.
- [ ] `examples/quill/` contains every file the Appendix B page references.

## Out of scope

- Implementing anything that makes the cases pass.
- Expected compiler outputs (`plain`, `site`, `json`). Phases 18 and 20 add those as snapshots.

## Notes

- Write outlines by hand from the spec, not from any implementation. The point is an independent check.
- Aim for many small cases over a few large ones. A failing small case points straight at the broken rule.

## Handoff notes

_To be filled in by the implementing agent._
