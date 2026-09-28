# Phase 12: Build resolution and linking

**Track:** Resolve · **Start after:** 11 · **Finish after:** 03 · **Parallel with:** 23 · **Unblocks:** 13, 14, 18

## Goal

Turn expanded pages into resolved pages for a given build: apply availability and build modes, substitute phrases, assign page ids, resolve links, and link glossary terms. The result is exactly what the emitters and page-level checks consume.

## Read first

- [SPEC.md](../../SPEC.md): §4.3, §4.4, §5, and §9.2–§9.4 (especially the selection rules, the "available at version V" rules, and Assets).
- `project-docs/contracts/assets.md` (phase 02), and `crates/tessera-core`'s `Router` trait.
- Handoff notes from phase 11.

## Deliverables

- `crates/tessera-resolve/src/build/`: the build resolution passes and `ResolvedPage`.

## Tasks

1. **Availability.** Resolve feature keys, apply inheritance from page to section to block, and compute each node's effective availability (SPEC §4.4). Record scope-nesting problems for phase 14.
2. **Build modes.** Apply the build's variant mode and availability mode exactly as SPEC §9.3 defines them:
   - **Selection:** remove conflicting arms and pages. A group keeps every surviving arm: one arm becomes plain content, several stay a group. Groups on unselected dimensions and labeled groups are untouched. Record groups where no arm survives.
   - **Filter:** remove content not available for the build's target and version, including versionless targets whose single state doesn't count as available (for example `cloud removed`). Content that remains keeps its availability annotations.
3. **Phrases.** Substitute declared phrase candidates in prose, headings, link text, link destinations, opted-in fences, and the frontmatter fields the model allows. Undeclared candidates stay literal.
4. **Page ids.** Assign every heading its page id (SPEC §5.5): `@id`, or the slug computed across the whole expanded page after build modes (phase 09, one scope per page).
5. **Links.** Resolve each file-path link from its source id to the target heading's page id. Fill empty link text from the target's title. Compute each link's route through `tessera-core`'s `Router`; provide a simple default router for tests, and let phase 20 supply Astro's. Record links whose target a build removes.
6. **Assets.** Carry each page's surviving asset references, with provenance, into the result, as the asset contract requires, so emitters can copy and rewrite them.
7. **Glossary.** Link glossary terms according to the model's matching settings.
8. **Results.** `ResolvedPage`: the resolved tree (every node keeping its source file and span), frontmatter, effective availability, assets, and resolution problems. The tree preserves structure: surviving groups stay groups and availability annotations stay attached, so emitters work from what survived rather than re-deriving it from the build's mode.
9. **Conformance.** Remove skip entries for resolution and build cases, and make them pass.

## Acceptance criteria

- [ ] The Quill project resolves under all three of its builds with no problems.
- [ ] Tests cover: every selection rule in SPEC §9.3, including the cloud build keeping the whole `pm` group; filtering before a history's first state, between states, at a state that doesn't count as available, and a versionless `removed` target; annotations kept in filter builds; page ids that differ from source ids because an include duplicates a heading; empty-text links to pages and to ids; phrases in link destinations.
- [ ] Every resolution and build conformance case runs and passes.

## Out of scope

- Reporting diagnostics (phase 14), emitting output (phases 18 and 20), incremental updates (phase 13), and Astro's routing rules (phase 20).

## Notes

- Keep the passes separate functions with clear inputs and outputs, in SPEC §9.2's order. They're easier to test, and to reorder if the spec changes.

## Handoff notes

_To be filled in by the implementing agent._
