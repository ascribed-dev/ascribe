# Phase 11: Source index and includes

**Track:** Resolve · **Start after:** 06, 07, 08, 09 · **Finish after:** 03 · **Parallel with:** 10, 23 · **Unblocks:** 12

## Goal

Index every source file in the project, independent of any build, and expand includes. This is the first of three resolution phases: this one handles what's true of the source; phase 12 applies builds; phase 13 keeps both current as files change.

## Read first

- [SPEC.md](../../SPEC.md): §2.2, §4.2, §5.2, §5.5 (source ids and page ids), and §9.2 step 1.
- `project-docs/contracts/assets.md` (phase 02).
- Handoff notes from phases 06, 07, 08, and 09.

## Deliverables

- `crates/tessera-resolve`: the crate's structure, the `Project` source index, and include expansion.

## Tasks

1. **Discovery.** Find every `.md` file under the content root and classify it as a page or a fragment (SPEC §2.2, including the model's fragment patterns).
2. **Per-file index.** For each file, record:
   - headings, their sections, and their **source ids** (SPEC §5.5): the `@id`, or the slug of the heading's text with phrases substituted, numbered within the file (phase 09, one scope per file);
   - titles, includes with their targets, links with their targets, phrase candidates, and availability markers;
   - **asset references** (image sources, and link targets that aren't pages), each resolved from the file it's written in to a path under the project, following the asset contract.
3. **Reverse edges.** Which files include a fragment, which files link to a file or source id, and which files reference an asset.
4. **Include expansion.** Expand a page's includes recursively into a build-independent **expanded page**. `@include` with `#id` selects the section of the heading with that source id in the target file (SPEC §4.2); `{heading=false}` drops that heading. Detect cycles. Every node in the expanded page keeps the file and span it came from, so relative references resolve against the right file (SPEC §4.2).
5. **Problems.** Record resolution problems for phase 14 to report: missing include targets or source ids, cycles, and references to missing files.
6. **Lookups.** Title for a file or source id; pages including a fragment; files linking to a source id; the resolved asset list for a page, with provenance.
7. **Conformance.** Remove skip entries for include and source-id cases, and make them pass.

## Acceptance criteria

- [ ] `cargo test -p tessera-resolve` passes.
- [ ] Tests cover: include cycles; an include selecting a section by source id; `{heading=false}`; the same fragment included twice in one page; a fragment's relative link and image resolving from the fragment's location, not the including page's.
- [ ] The Quill project indexes and expands with no problems.

## Out of scope

- Anything build-specific: availability, variant selection, phrase substitution in content, page ids, routes (phase 12).
- Updating the index as files change (phase 13).

## Notes

- Keep index construction a pure function of a file's contents, the model, and the file's path. Phase 13 depends on that to cache and invalidate correctly.

## Handoff notes

_To be filled in by the implementing agent._
