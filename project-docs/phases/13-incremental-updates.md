# Phase 13: Incremental updates

**Track:** Resolve · **Start after:** 12 · **Parallel with:** 14, 18 · **Unblocks:** 15

## Goal

Keep the project index and resolved pages correct as files change, re-doing only the work each change affects. The language server depends on this for speed; correctness means the result always equals rebuilding from scratch.

## Read first

- [PLAN.md](../PLAN.md): Project graph and resolution.
- Handoff notes from phases 11 and 12.

## Deliverables

- `crates/tessera-resolve/src/incremental/`: an API such as `project.apply(changes) -> Affected`, where changes are file contents updated, files created, deleted, or renamed, and the content model changed; and `Affected` lists the files to re-check and the pages to re-resolve.

## Tasks

1. **Invalidation rules.** Define and implement what each kind of change invalidates:
   - **A source file's contents:** that file's parse and index; pages that include it (directly or transitively); files whose links target its source ids; and, when its title or headings change, empty-text links that show them.
   - **A file created, deleted, or renamed:** page and fragment classification; every reference that resolves, or used to resolve, to that path, including links, includes, and assets.
   - **An asset file created or deleted:** every page that references it.
   - **The content model:** the directive keyword set can change, which changes how *every* file parses, including files no one has open. Reparse all files. Changes to phrases can change source ids and substitutions, so re-index all files. Changes to dimensions, lifecycle states, features, or builds re-resolve all pages.
2. **Caching.** Cache per-file parse and index results by content hash and model version, so unchanged files are never reparsed.
3. **Snapshot consistency.** Each update produces a consistent snapshot tagged with the file versions it reflects, so a consumer can tell whether results are current (phase 15 uses this to avoid publishing stale diagnostics).
4. **Differential testing.** A property test that applies random sequences of edits, creations, deletions, renames, and model changes, and after each step compares the incremental result with a from-scratch build of the same files. They must match exactly.

## Acceptance criteria

- [ ] The differential property test passes over a large number of random sequences, including model changes.
- [ ] Tests show: editing a fragment re-resolves the pages that include it and no others; declaring a new widget in the model changes the parse of an unopened fragment that uses its name; deleting a file produces broken-reference problems in the files that referenced it; renaming a heading updates the empty-text links that show it.
- [ ] A benchmark records the time to apply a one-file edit on a 3,000-page project.

## Out of scope

- Watching the filesystem and talking to editors (phase 15). This phase provides the API they call.

## Notes

- If profiling shows this approach is too slow, `salsa` is the planned alternative (PLAN.md). Keep the invalidation rules above as the specification of correctness either way.

## Handoff notes

_To be filled in by the implementing agent._
