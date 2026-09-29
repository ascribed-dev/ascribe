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

### What was built

`crates/tessera-resolve/src/incremental/` (plus small, additive changes to `Project`; see Changes outside the module):

- `mod.rs`: `IncrementalProject` (`load`, `apply`, `snapshot`, `version`, `ids`, `stats`, `is_file_current`), `Change`, `Affected`, `Snapshot`, `Version`, `ApplyError`. The module documentation is the specification of the invalidation rules, the caches, and the versions.
- `ids.rs`: `FileIds` and the rules for file ids (below).
- `overlay.rs`: the file system a project probes: a base (disk or memory) with files added and removed on top.
- `signature.rs`: `structure_signature` (what other files can see of a file) and the model fingerprints behind `ModelImpact`.
- `cache.rs`: the parse cache, and `ResolvedCache` (resolved pages per build).
- Tests: `tests/incremental.rs` (19: the acceptance cases, caching, model tiers, snapshots, ids, assets, batches), `tests/incremental_differential.rs` (the property test), `tessera-check/tests/ids.rs` (2), and `benches/incremental.rs`.

### Interfaces later phases use

- **`IncrementalProject::load(model, layout, fs)`** then **`apply(changes) -> Result<Affected, ApplyError>`**. `Change`: `Created`/`Edited`/`Deleted`/`Renamed` (content paths), `AssetCreated`/`AssetDeleted` (project paths, for files that aren't sources), `Model(Arc<ContentModel>)`. A batch is taken by its net effect per path. `snapshot()` gives a `Snapshot`, which dereferences to `Project`, so `expand`, `resolve_page`, `problems`, and every lookup work on it.
- **`Affected`**: `version`/`previous`, `parsed`, `indexed`, `recheck` (files that exist and whose file-level diagnostics, or the page-level ones located in them, may differ), `removed`, `re_resolve` (pages whose resolved form, in any build, may differ), `model: Option<ModelImpact>`. Everything not listed is unchanged. `is_empty()` when the update changed nothing (no new version).
- **For phase 15, to avoid stale diagnostics:** compute from a `Snapshot`; before publishing, `snapshot.is_current()`, or, for one file, `inc.is_file_current(&snapshot, &path)` (true unless a later update listed the file in `recheck`/`removed`, or changed the model beyond warnings). `Snapshot::file_version(path)` and `model_revision()` say when a file's text and the model last changed. `Affected::removed` names files whose diagnostics to clear; file id 0 (`ascribe.toml`) is the caller's (a `ModelImpact::Warnings` change lists no file).
- **`ResolvedCache`** keeps `ResolvedPage`s per build; call `cache.apply(&affected)` after each update, and `cache.resolve_page(&snapshot, path, build, router)`. One router per cache.
- **`Project::expansion(path) -> Option<Arc<ExpandedPage>>`** shares the cached expansion; `expand` still returns an owned copy.
- **`Project::load_with_ids(model, layout, fs, &mut FileIds)`**: a from-scratch load numbered like an incremental one. It's what the differential test compares with.
- **`index::parse_source` and `index_parsed`** are `index_file` split in two, so a parse can be reused.
- **`is_source_path(&RelPath)`** is the discovery rule (Q52) for one path.

### File ids (task, and Q91)

An id names a **path**: given the first time a file is there, never changed by an edit, never reused for another path. Deleting a file leaves no live file with its id (`path_of` returns `None`); a file back at the same path gets the same id. A rename is a deletion and a creation. A fresh load numbers files 1, 2, … in path order, and `ascribe.toml` is 0, as before. `tessera_check::Project` finds a file by id, not position, so the language server builds one from a snapshot's ids (`Project::from_parts` with `SourceFile { id: file.file, .. }`); `tessera-check/tests/ids.rs` checks that a fresh load numbers files as `Project::from_sources` does and that check works with an updated snapshot's gaps.

### Invalidation, in brief (the module documentation has the table)

Content of a file: its parse and index; the pages that include it; and, if what others see of it changed (`structure_signature`: kind, frontmatter as written, headings, every directive line with nesting), the pages that link to it or to a page that includes it, and the files that include it. Creation, deletion, and rename: the same, plus every reference that names the path, a case twin of it, or the `.md` or `index.md` a route-like link maps to, found through an index from each file to the paths its references can depend on. Assets: the references to them. Model: `ModelImpact::Keywords` (directive keywords or note types) reparses every file; `Index` (phrases, fragment patterns, slugger) re-indexes every file and reuses each parse; `Resolution` re-checks and re-resolves everything with no reparse; `Warnings` changes no file.

### Decisions

- **Snapshots are values.** `Project` is copied on write (`Arc::make_mut`): an update in place when no snapshot is held, a copy of the maps (files, indexes, resolutions and edges are `Arc`s or small) when one is. The reverse edges are updated by removing and adding what the files involved contribute, kept sorted by (file, span) so the result doesn't depend on order; `Project::load` builds them the same way.
- **`Project` gained state.** `files` and resolutions hold `Arc`s, ids are a map (`path_of` no longer indexes a `Vec`), and expansions are cached behind a `Mutex`. Public signatures are unchanged.
- **The differential test checks two things:** the incremental `Project` equals a from-scratch load (every file, resolution, edge, problem, expansion, asset list), and a *consumer* that recomputes only what `Affected` lists holds what a from-scratch consumer would (file-level results, and resolved pages for every build). The second is what makes `Affected` trustworthy. Two properties (one from an empty base, one from a non-empty one where case-twin names are left out, because the base and the overlay may pick different twins) × 300 cases by default; `TESSERA_INCREMENTAL_CASES=4000` was run clean for both. Nine deliberate breakages of the invalidation rules (each rule removed in turn) are each caught.
- **A file appearing that no reference names** produces no new version. Nor does an edit that restores the text, or a batch that nets to nothing (Q94).
- **`ApplyError::LayoutChanged`** for a model that moves the content root or output directory (Q92).

### Benchmark (`cargo bench -p tessera-resolve --bench incremental`)

3,000 pages in 30 directories, 100 fragments each included by 30 pages, 60 images; each page has an include, three links, an image. Release build, one run on the CI-class container this phase was written on:

| | min | median | p95 |
|---|---|---|---|
| from-scratch load (3,100 files) | | 152 ms | |
| expanding every page once | | 36 ms | |
| **apply: edit a paragraph of one page** | 73 µs | **87 µs** | 194 µs |
| the same with a snapshot held (copies the maps) | 1.4 ms | 1.9 ms | 2.4 ms |
| edit a fragment's text (30 pages re-resolved) | 107 µs | 127 µs | 159 µs |
| rename a heading in a fragment (120 pages, through links) | 183 µs | 223 µs | 283 µs |
| create / delete a page | 21 µs | 61 µs | 105 µs |
| appear / disappear an image (50 pages) | 224 µs | 284 µs | 423 µs |
| model change to a phrase (re-indexes all 3,100, reparses none) | | 145 ms | |

`salsa` isn't needed. The one cost that scales with project size is the copy when a snapshot is held; it's about 2 ms here.

### Changes outside the module (all additive to `Project`'s public API)

- `project.rs`: `load_with_ids`, `expansion`; private fields changed as above. No public signature changed.
- `index/mod.rs`: `index_file` is `parse_source` plus `index_parsed`; both are public.
- `fs.rs`: `is_source_path`, used by `MemoryFs::sources` (same behavior).
- `tessera-check/src/project.rs`: documentation only (ids need not be consecutive).

### Conformance

Untouched by this phase: `cargo test -p tessera-conformance --test conformance` was 330 passed, 0 failed, 33 skipped before merging `main`, and is 364 passed, 0 failed, 0 skipped after merging phase 14. The parity test (`tessera-check/tests/parity.rs`) passes unchanged.

### Left open

- **Q91 to Q97** in `questions.md`, all implemented as proposed: ids (Q91), a layout change (Q92), what of a target a link can depend on (Q93), batches and no-op updates (Q94), the history that `is_file_current` looks back over, 256 updates (Q95), a warnings-only model change (Q96), a diagnostic that names a case twin (Q97). Q98 to Q100 are unused.
- **Not incremental:** the file-level checks. `tessera_check::check_files` runs over a whole project; phase 15 should call `check_file` per file in `Affected::recheck` (its `check_file` is per file already) with a `tessera_check::Project` built from the snapshot. That project probes the disk for non-source files (`DiskFs`), so an unsaved image isn't seen; phase 15 needs a constructor that takes a `FileSystem` (a change in `tessera-check`, which this phase left alone).
- **The interim route mapping** (phase 12's note) is still `route.md`, else `route/index.md`; the dependency index covers exactly those two names. A router that answers "which page has this route" needs to add the paths it can answer with.
- **Glossary targets:** a change to what others see of a page a glossary term links to re-resolves every page (any page may contain the term). Fine at this size; a term-to-pages index would narrow it.
- **A model change reaches every file.** Narrowing it by which dimensions, states, or builds changed is possible but wasn't needed.
- **`Renamed` from a non-source path to a source path** can't know the text; it deletes the old file and expects a `Created`.
- **The overlay's case-twin rule** (files added since load come before the base's, first in path order) is a corner: a disk with two files differing in case is out of scope (SPEC §9.4).
