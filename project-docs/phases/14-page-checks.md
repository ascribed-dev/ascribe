# Phase 14: Page-level checks

**Track:** Check · **Start after:** 10, 12 · **Finish after:** 03 · **Parallel with:** 13, 18 · **Unblocks:** 15 and 18 (to finish), 24, 26

## Goal

Report every page-level diagnostic, per build, at the source location that causes it. Provide the single entry point that `tessera check`, `tessera build`, and the language server all use, so they can't disagree.

## Read first

- [SPEC.md](../../SPEC.md): §8.1, §8.2 (rows marked page level), §4.2, §5.5, and §9.3.
- `tests/conformance/diagnostics.toml`, and handoff notes from phases 10, 11, and 12.

## Deliverables

- `crates/tessera-check/src/page/`: page-level checks.
- `check_project(&project, build) -> Vec<Diagnostic>`: file-level plus page-level diagnostics for one build. This is the one function every tool calls.
- `tessera check` running it for every build, or one build with `--build`.

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
3. **Entry point.** Implement `check_project`, and make `tessera check` use it.
4. **Conformance.** Remove skip entries for page-level cases, and make them pass.

## Acceptance criteria

- [ ] Every page-level row of §8.2 has a passing conformance case.
- [ ] `tessera check` on `examples/quill` exits `0`.
- [ ] A test shows a problem inside a fragment reported at the include site and in the fragment.
- [ ] A test shows a build-specific problem reported once, naming the build.

## Out of scope

- The language server's use of builds (phase 15) and the build command (phase 18). Both call `check_project`.

## Handoff notes

_To be filled in by the implementing agent._
