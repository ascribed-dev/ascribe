# Phase 03: Conformance cases

**Track:** Tests · **Start after:** 00, 01, 02 · **Parallel with:** 05, 08, 09, 19 · **Unblocks:** the acceptance tests of every parser, check, and resolution phase

## Goal

Turn SPEC.md into executable expectations: a conformance suite covering every rule in §2–§6, the resolution and build rules in §9.2–§9.4, and every row of the §8.2 diagnostics table. Later phases prove themselves by making these cases pass.

## Read first

- [SPEC.md](../../SPEC.md), all of it. §3–§6, §8, and §9.2–§9.4 closely.
- `tests/conformance/README.md` (phase 00) for the case format.
- `project-docs/content-model.md` (phase 01), and the contracts and `diagnostics.toml` (phase 02).

## Deliverables

- `tests/conformance/_model/ascribe.toml`: the shared fixture model, based on `examples/content-models/quill.toml`.
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
6. **The Quill project.** Build `examples/quill/`: `ascribe.toml`, the Appendix B page, and the files it references (`_fragments/prerequisites.md`, `quickstart.md` with a `try-in-browser` id, `keys.md` with a `rotate-keys` id), plus at least one image, including one beside the fragment, so the project is free of errors under every build.
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

### What was built

- **310 conformance cases** in `tests/conformance/cases/`, written by hand from SPEC.md (see the table in `tests/conformance/README.md`, "The suite"), plus the two phase 00 samples. All load, validate against `diagnostics.toml`, and run as skipped under `SKIPS.toml`; the skip entries are unchanged, since no adapter exists yet.
  - §3 and §4: `recognition/` (19), `attributes/` (25), `primary/` (17), `forms/` (14), `groups/` (12), `titles/` (17), `binding/` (15), `lists/` (13), `nesting/` (3), `directives/` (65: `id`, `include`, `variant`, `available`, `note`, `steps`, `details`), `widgets/` (14).
  - §5: `phrases/` (7), `links/` (16), `images/` (25, every reference form with attributes), `headings/` (5), `glossary/` (2).
  - §2, §7.2: `frontmatter/` (13), `model/` (2).
  - §9.2 to §9.4: `builds/selection/` (8), `builds/filter/` (9), `builds/assets/` (7), `builds/pages/` (1).
  - `projects/quill/`: the whole Quill project as one case, with no diagnostics under any of its three builds.
- **`tests/conformance/_model/ascribe.toml`**: the shared fixture model, from `quill.toml`, extended with a fragment pattern and schema, three page types (two overlap on `reference/api/**`), features (one with a history), a `sunset` state that isn't available, a `security` note type, image attributes, one widget of each shape (from `full.toml`), and the builds `site`, `cloud-pdf`, `cloud-only`, `npm-only`, `sm-3.3`, `sm-3.4`, and `sm-3.5`. It declares `site` and `cloud-pdf`, as phase 00's sample expects.
- **`examples/quill/`**: `ascribe.toml`, `docs/install-agent.md` (byte for byte the Appendix B page), `quickstart.md` (`try-in-browser`), `keys.md` (`rotate-keys`), `_fragments/prerequisites.md`, and two images, one beside the fragment.
- **`tests/conformance/tests/suite.rs`** (runs now, in `cargo test`): every SPEC §8.2 row has a case that expects it; every case has a description and `spec`; every question a case names is open in `questions.md`; the Quill example page equals SPEC Appendix B and the `appendix-b` sample; the example equals its project case; every file the example's pages include, link to, or embed exists.
- **`tests/conformance/INTERPRETATIONS.md`**: the readings the cases lock in, and what the suite doesn't cover. **README** gained "The suite", "Resolved outlines", and "The shared model and the content root".
- **`project-docs/questions.md`**: Q13 to Q26, each with a proposed resolution. All were resolved on 2026-09-28 as proposed (Q15 through Q30), and the 45 cases that were `provisional` on them no longer are.

### Interfaces later phases use

- **Adapters** (05 to 14, 18, 20): expectations use the format in the README. Points an adapter has to know, all in the README or `INTERPRETATIONS.md`:
  - a resolved outline is source text after includes, availability, build modes, and phrases; links, heading ids, and glossary aren't in it, and a surviving `@available` stays as the annotation;
  - `Case::content_root()` is the content root whatever the shared model's `content-root` says, and the case directory is the project root, which is where `../shared/...` and `../.tessera/...` files sit;
  - a loader diagnostic is `file: ascribe.toml` in a single-file case.
- **Tags**: `parser` cases carry only line-form directives, so they need no structure; `structure` cases carry outlines; `check` cases carry top-level diagnostics; page-level diagnostics under a build always come with `page-check`. A case with several tags waits for all of them. Build expectations for a `resolve`-only case list `pages` and `assets`, never `diagnostics`.
- **Phase 08**: `tests/conformance/cases/model/` has the two `model-name-multiple-roles` cases, tagged `model`. The rest of §20 needs fixtures there; `model-dimension-value-shared` overlaps that row (`INTERPRETATIONS.md`).

### Decisions

- **Outline-only and diagnostics-only cases are separate**, so each implementation phase's cases run when its own adapter lands, not when all of them have.
- **Every provisional choice is the proposal in `questions.md`**, and each provisional case implements it. When a question resolves differently, its cases change and lose the tag (the suite test fails until they do).
- **Small cases**: 310 cases, most with one input file and a handful of lines.
- **A project case copy of the example** rather than a pointer, because a case's files must sit under it; the test in `suite.rs` keeps them identical.
- **No output expectations**: `plain`, `site`, and `json` are phases 18 and 20's.

### Left open

- **Q13 to Q26** were resolved on 2026-09-28. Q21's resolution added `link-page-dropped`, with the case `links/page-dropped-by-selection`.
- **Not covered** (see `INTERPRETATIONS.md`): canonical form (§8.3, the `format` tag), outputs, registry-change reports (§5.1), phrases in frontmatter, most loader rules, and editor features.
- **Tag coverage**: no case carries `output` or `format`, so those `SKIPS.toml` entries stay until phases 18, 20, and 23 add cases and adapters.
- **Finish after**: this phase's cases can't be shown to pass until the implementation phases connect adapters; until then they're reported as skipped.
