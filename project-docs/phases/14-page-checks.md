# Phase 14: Page-level checks

**Track:** Check · **Start after:** 10, 12 · **Finish after:** 03 · **Parallel with:** 13, 18 · **Unblocks:** 15 and 18 (to finish), 24, 26

## Goal

Report every page-level diagnostic, per build, at the source location that causes it. Provide the single entry point that `ascribe check`, `ascribe build`, and the language server all use, so they can't disagree.

## Read first

- [SPEC.md](../../SPEC.md): §8.1, §8.2 (rows marked page level), §4.2, §5.5, and §9.3.
- `tests/conformance/diagnostics.toml`, and handoff notes from phases 10, 11, and 12.

## Deliverables

- `crates/tessera-check/src/page/`: page-level checks.
- `check_project(&project, build) -> Vec<Diagnostic>`: file-level plus page-level diagnostics for one build. This is the one function every tool calls.
- `ascribe check` running it for every build, or one build with `--build`.

## Tasks

1. **Checks.** Implement every §8.2 row with level `page`, from phases 11 and 12's recorded problems and the resolved pages. This includes:
   - duplicate page ids, including ids from included content;
   - links to source ids that don't exist, to fragments, or to ids that exist only inside a fragment;
   - links whose target a build removes;
   - variant groups where no arm survives a build's selection (a warning);
   - availability specs that exceed their enclosing scope once inheritance and includes are applied;
   - include targets whose `#id` doesn't exist;
   - duplicate heading text without `@id` once includes are expanded (a warning).
2. **Locations.** A problem inside included content is reported at the include site, and also in the fragment (SPEC §8.1). A problem that appears only in some builds names those builds. Report each distinct problem once, not once per build.
3. **Entry point.** Implement `check_project`, and make `ascribe check` use it.
4. **Conformance.** Remove skip entries for page-level cases, and make them pass.

## Acceptance criteria

- [ ] Every page-level row of §8.2 has a passing conformance case.
- [ ] `ascribe check` on `examples/quill` exits `0`.
- [ ] A test shows a problem inside a fragment reported at the include site and in the fragment.
- [ ] A test shows a build-specific problem reported once, naming the build.

## Out of scope

- The language server's use of builds (phase 15) and the build command (phase 18). Both call `check_project`.

## Handoff notes

### What was built

- **`crates/tessera-check/src/page/`**:
  - `mod.rs`: the entry points and `PageChecker`. `check_project(&Project, &Build)` is the file-level diagnostics (`check_files`) followed by the build's page-level ones; `check_pages` is the page-level ones alone; `check_all_builds(&Project)` is the file-level ones once, then every build's page-level ones with each distinct problem once, then the problems in content no build publishes. `PageChecker::new(&Project)` indexes the project once, for several `check(build)` calls or one `check_all()`; the free functions build one each call.
  - `bridge.rs`: `tessera_resolve::Project` over a checked `Project`: source texts from memory (so the language server's buffers work), everything else (images) through the project's own file system, and a map from the index's `FileId`s to the checked project's, applied to every location before it leaves the crate.
  - `collect.rs`: one resolved page's problems, from three sources. (1) `ResolvedPage::problems`, relocated to the include site. (2) Duplicates from `ResolvedPage::headings()`: `id-duplicate` (page ids, at the `@id` line for an explicit id and at the heading for a slug, the `include` message variant when the later one came through an include) and `heading-duplicate-without-id` (Q103). (3) The source index's `link-id-missing` and `link-id-in-fragment` for the links the build publishes.
- **`Diagnostic`** gained `builds: Vec<String>` (the builds a page-level diagnostic appears in, in the model's order; empty for file-level ones) and `unpublished: bool`, and `builds_note(total_builds)`, the text `ascribe check` adds to a message.
- **`ascribe check`** (`commands/check.rs` only): runs `check_all_builds`, or `check_project` for `--build <name>` (an unknown name is exit code 2, listing the builds), and appends `builds_note` to each message. `report/` is untouched, so the JSON has the builds in the message, not as a field.
- **Conformance**: the `page-check` skip entry is removed and `tests/adapters/page_check.rs` handles the tag; the `resolve` adapter's build diagnostics are `check_pages`, no longer read off the resolved pages. `tests/file_checks.rs`, `resolve_rows.rs`, and `source_index_rows.rs` are deleted (see below). **363 passed, 0 failed, 0 skipped.**

### Interfaces later phases use

- **15, 18:** `tessera_check::check_project(&project, build)` for a build, `check_all_builds(&project)` for all of them (the language server builds a `Project` with `Project::from_parts`). `Diagnostic::builds` and `Diagnostic::unpublished` say which builds a diagnostic is in. Phase 18 fails a build on the errors of `check_project` for its own build; the parity tests compare `check_project` per build with `check_all_builds` (a diagnostic of a build is in `check_all_builds` with that build in its `builds`).
- **13, 15:** `PageChecker` indexes every time it's built. When phase 13's incremental index exists, `page/bridge.rs` is the one place that builds `tessera_resolve::Project` from a checked `Project`.
- **24:** page-level diagnostics carry no fixes yet. The ones with an obvious edit (a numbered heading gets `@id`, a duplicate id renamed) are quick fixes for phase 24.

### Decisions

- **Where a problem is reported** (SPEC §8.1, Q20): at the cause; inside included content at `via[0]`, the outermost include, with the cause in the fragment as a `related` entry ("the cause is in the included file `_f.md`"), never a second diagnostic. A cycle is at the include that closes it. A problem in a fragment included twice is reported at each include site, since each is where a reader meets it; the duplicate ids of a fragment included twice are one diagnostic, at the later include.
- **Once across builds**: two diagnostics are the same problem when the row, the location, the message variant, and the message arguments other than `build` agree. The builds are then merged in the model's order, and a message that names its build (`variant-no-arm-survives`, `link-id-removed`, `link-page-dropped`) names all of them (Q102).
- **Q81 and links**: `link-id-missing` and `link-id-in-fragment` are found once per file by the source index, and reported for each build only where the link is in a block the build publishes.
- **Content no build publishes** (Q101): one extra resolution with a build that keeps everything, only when no build already does. A problem it finds is reported when its cause sits in no block any real build publishes (a block is the page, the file and span, and the includes it came through). A problem that only exists because arms that no build keeps together are all kept isn't reported. Such diagnostics have `builds` empty and `unpublished` set.
- **A case corrected**: `headings/explicit-id-is-stable-source-and-page-id` expected `heading-duplicate-without-id` at line 5, the page's own heading with an `@id`. The row is about a heading *without* `@id` that repeats another's text, and Q20 says once, at the later occurrence, at the include site: the fragment's `## Setup`, so line 8 (`@include: _f.md`). The expectation and description are fixed.
- **`harness.rs`**: the test that ran `projects/quill` with stand-in adapters to see a skip has nothing left to be skipped for (no case has the `output` tag), so it now checks that a tag with neither an adapter nor a skip entry fails the case, naming the tag.

### The three temporary runners

`file_checks.rs`, `source_index_rows.rs`, and `resolve_rows.rs` are deleted. Everything they asserted is now the runner's, with the adapters running each case whole and comparing exactly:

- `file_checks.rs` compared the top-level diagnostics of `check` cases (columns only where the case gives one): the `check` and `page-check` adapters give the same list from `check_files`, and the runner compares slug, file, line, and any column, exactly.
- `source_index_rows.rs` compared nine rows in every case that expects diagnostics, by slug, file, and line: each is now in the case's own top-level or per-build diagnostics, compared exactly. Its check that every row has a case is `tests/suite.rs`'s `every_section_8_2_row_has_a_case`.
- `resolve_rows.rs` compared each build's published pages, outlines, assets, and the six rows resolution records: the runner does the same for every case that lists them (`pages` and `assets` sets exactly, outlines, and diagnostics), through the same `resolve_build`, and the diagnostics are now the checks' own.

What they had that the runner doesn't: minimum counts (`cases_run >= 30`) that guard against a runner silently running nothing. The runner prints `363 passed, 0 skipped`; a case with no adapter fails, so nothing can be dropped quietly. The adapters' helper functions for those tests (`source_problems` and its slug lists) are gone with them.

### Acceptance criteria

See the pull request for the status and evidence of each.

### Left open

- **Q101 to Q104** were resolved on 2026-09-28. Q101, Q102, and Q104 as implemented, with Q102's `builds` message variants added to the registry. Q103 changed: duplicates are found by slug among headings without `@id`, not by text, so `Set up` and `Set-up` are duplicates and a heading after one with an `@id` isn't (`collect.rs`).
- **Fragments no page includes** have no page-level diagnostics (Q104): they aren't in any page.
- **Performance.** `PageChecker` indexes the project (parsing every file a second time, after `check_files`) and resolves every page once per build, plus once for the unpublished pass when it's needed. `Project::problems` expands the target page for each link whose id isn't among the target's own, and its results are cached across builds. Phases 13 and 26 can cache all of it per file.
- **The JSON report** has the builds only in the message, because `report/` is shared with phase 18. A `builds` array and an `unpublished` flag on each entry are additive (no schema version change) and are the next change there.
- **Check that `ascribe build` uses `check_project`.** Phase 18 adds the command; the per-build parity is for its tests.
