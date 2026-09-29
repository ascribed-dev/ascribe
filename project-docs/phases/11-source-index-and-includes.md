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

### What was built

`crates/tessera-resolve` (`src/slug/` is untouched):

- `index/`: `index_file(file, path, source, model, slugger) -> FileIndex`, **a pure function** of the file's id, path, and text, the content model, and the slugger. It never touches the file system and doesn't know which other files exist. A `FileIndex` holds the parsed tree (`Arc<ParsedDocument>`) and text, the frontmatter (`serde_yaml::Value`) and `title`, `headings`, `includes`, `references` (links and images), `phrases`, and `availability`.
  - `Heading`: level, text (plain text, declared phrases substituted), `explicit_id`, **`source_id`**, `section` (a byte span), `has_phrase`, `empty_slug`. One `SlugScope` per file, only for headings without `@id`; explicit ids don't take part in numbering (Q7).
  - `Include`: the directive, the path as written, the resolved `target` content path, the `#id` section, and `heading` (`false` only for `{heading=false}`).
  - `Reference`: a link or image with its destination (a reference form's is its definition's, Q23), `LinkForm`, and a `Target`: `External`, `Local` (the file it names, resolved **from the file it's written in**, the fragment, whether it's a source file, whether it looks like a route), or `Deferred` (a reference form whose definition holds a declared phrase; see Left open).
  - `PhraseUse` (every candidate, declared or not, and where: text, heading, destination, code), `AvailabilityMarker` (primary, binding, parsed spec).
- `project.rs`: `Project::load(model, layout, &dyn FileSystem)` indexes every `.md` file under the content root, gives each a `FileId` (path order), and works out what every reference names once the files are known (`Resolution`), following the asset contract's steps 5 and 6 (boundary, exact case) through `Layout::is_allowed` and `FileSystem::probe`. It builds the reverse edges and answers the lookups below.
- `expand.rs`: `Project::expand(path) -> ExpandedPage`.
- `fs.rs`: `FileSystem` (`sources`, `read`, `probe`), `DiskFs`, `MemoryFs` (for tests and for the language server's unsaved buffers). `layout.rs`: `Layout` (content root and output dir relative to the project root).
- Tests: 63 in `crates/tessera-resolve/tests` (`includes.rs`, `headings.rs`, `references.rs`, `quill.rs`) and unit tests in `fs.rs` and `layout.rs`, plus a doctest. `tests/conformance/tests/source_index_rows.rs` runs the source-index rows of SPEC §8.2 over every conformance case that expects diagnostics (see Conformance).

### Interfaces later phases use

- **Lookups** on `Project`: `file(path)`, `files()`, `pages()`, `fragments()`, `path_of(FileId)`, `file_by_id`; `title_for(path, Option<id>)` (frontmatter `title`, or the heading text for a source id: what a link with no text shows); `heading(path, id)`; `resolutions(path)` / `resolution_at(path, span)`, parallel to `FileIndex::references`.
- **Reverse edges**: `includers(target)` (each with its span and section), `including_pages(target)` (transitive; a section include counts as including the file), `links_to(target)`, `links_to_id(target, id)`, `asset_users(asset)`.
- **`expand(path)`** returns `ExpandedPage { blocks, problems }`. An `ExpandedBlock` has `file`, `span`, `via` (the includes it came through, outermost first: the first is in the page's own file) and a kind: `Leaf(Block)`, `BlockQuote`, `List`, `Container`, `Group`, with nested blocks expanded. An `@include` is replaced by the included blocks (the whole file, or the section of the heading with that source id, or the section without its heading for `{heading=false}`); one that can't be expanded stays as its directive. `visit`, `headings(&project)`, and `ExpandedBlock::own_ranges` walk it. Bindings (`DirectiveLine::binding`) are the ones from each block's own file: phase 12 recomputes them where content was spliced next to different neighbors.
- **`assets(page) -> Vec<PageAsset>`**: every image and every link to a non-source file in the expanded page, in document order, each with its source path (starts with `..` outside the content root), kind, kept `#fragment`, the file it's written in, its location, and `via`. Missing files aren't assets; they're problems. Build modes aren't applied.
- **Problems**, all `Issue`s with registry slugs and the registry's message arguments (`path`, `id`, `actual`, `page`, `suggestion`, `cycle`, `fragment`):
  - `Project::problems(path)`, in source order, located where written: `include-target-missing` (variant `case` with `actual`), `link-target-missing` and `image-source-missing` (variants `case`, `outside`), `link-to-fragment`, `link-route`, and the page-level `link-id-missing` and `link-id-in-fragment` (checked against the target's **own** source ids, Q6; `fragment` names the fragment). The last two are located at the link, in the file it's written in: for a link inside an included fragment, phase 14 reports at the include site.
  - `ExpandedPage::problems` (`PageProblem { issue, via }`): `include-cycle` (located at the include that closes it, in the file containing it, Q20) and `include-id-missing`. A page-level report goes at `via[0]` (the outermost include site) with the issue's own location as related information, except a cycle.
  - Not reported here, for phase 14: `id-duplicate` and `heading-duplicate-without-id` (page ids, after build modes), `link-id-removed`, `link-page-dropped`. Not recorded at all: `id-invalid`, `image-alt-missing`, phrase warnings, frontmatter (phase 10).
- **`FileIndex` is a pure function of a file's contents, the model, and its path** (the phase's note), so phase 13 can cache it per file. What depends on other files is `Resolution` (which files exist, which assets exist) and the edges, both built in `Project` from the indexes; `Project::resolve_file` and `build_edges` are the parts to redo.

### Decisions

- **Asset boundary**: `Layout::is_allowed(content_path)`: inside the content root or the project root, and not inside the output directory. A file case-differing from the one asked for is `Missing::Case(actual)`, whatever the file system (`DiskFs::probe` compares directory entries, not `metadata`).
- **What a destination names** follows the asset contract §1: a link to a `.md` file inside the content root is a source file (a page or fragment) whether or not it exists; every other local destination is an asset, or, for a link that looks like a route and names no file, `link-route` instead of a missing-file error (Q22). A route-like extensionless file that exists (`LICENSE`) is an asset.
- **Phrases in destinations** (Q43): a declared phrase in an inline link or image destination, or an autolink, is substituted before the destination is classified (the parser records the candidates). A reference form's destination comes from a definition, which isn't a node yet: if it contains a declared `{key}`, its target is `Deferred` and nothing is reported or copied, rather than a false missing-file error. No workaround for definitions was attempted (as asked).
- **Cycles** are keyed by file *and section* (Q66); a cycle is found when an include would expand something already being expanded, reported once per expansion, at the include that closes it. A fragment included twice on a page is just expanded twice: uniqueness of ids is a page-level check.
- **Sections** are per list of blocks (Q18, Q32): a heading in a container ends its section at the end of the container. A section include of a heading inside a container gives the blocks of that container only.
- **Discovery** takes exactly the files whose name ends in `.md` and skips any file or directory whose name starts with `.` (Q52, raised by phase 10, still open: `// SPEC-QUESTION(Q52)` in `fs.rs`). It follows symbolic links to files and to directories (each directory once, by canonical path). Nothing is skipped silently: a directory or file that can't be read, and a source file that isn't UTF-8, are left out and listed in `Project::unreadable()`.
- **Heading text** for slugs (Q67): text, code, link text, and emphasis; images and raw HTML contribute nothing.
- **Empty fragments** (`page.md#`, `file.md#`) mean no id. **An image with no path** (`![a]()`, `![a](#x)`) is `image-source-missing`, and a `#id`-only link isn't a problem: the same reading phase 10 implements for Q59 (`SPEC-QUESTION(Q59)` in `project.rs`).
- **`FileId`s** are assigned in path order at load; phase 13 keeps them stable per path.
- **A change outside the crate**: `tests/conformance/tests/harness.rs`'s `bundled_samples_are_discovered_and_skipped_with_recorded_reasons` ran the bundled sample `include-and-selection` with an empty registry and expected its `include` tag to be skipped; now it registers a stand-in adapter for `include`, so the sample is skipped for `resolve` alone. Phase 12 will need the same kind of change.

### Conformance

- `SKIPS.toml`: the `include` and `slug` entries are removed, and `tests/conformance/tests/adapters/include.rs` handles both tags. **No case runs in the runner because of it**: all 26 cases with these tags also carry `check`, `page-check`, or `resolve`, which stay skipped until phases 10, 14, and 12. `cargo test -p tessera-conformance --test conformance`: **246 passed, 0 failed, 71 skipped** after merging `main` with phase 10, the same as `main` (97 passed before phase 10).
- So the adapter produces no outline, diagnostics, or build result (it would shadow the adapters that will), and offers `source_problems(case)` instead. **`tests/conformance/tests/source_index_rows.rs`** runs, over all 189 cases that expect diagnostics (file-level or per build), the nine rows the source index answers (`include-target-missing`, `link-target-missing`, `image-source-missing`, `link-to-fragment`, `link-route`, `include-cycle`, `include-id-missing`, `link-id-missing`, `link-id-in-fragment`) against the expectations by slug, file, and line, and fails if a row has no case. All pass, including `directives/include/cycle`, `self-include`, `id-missing`, `bare-filename-is-not-searched`, `builds/assets/fragment-image-not-beside-page`, the `links/` and `images/` cases, and `projects/quill`. **Phase 12 or phase 14 deletes it, whichever first runs the `include`, `resolve`, and `page-check` cases for real** (a case runs only when every one of its tags is handled, and these cases combine `check`, `include`, `resolve`, and `page-check`); that phase folds any row its adapter doesn't cover into the adapter instead.
- What the `resolve`-tagged include cases expect (resolved outlines, `builds.*.assets`) needs build modes and phrase substitution in content: phase 12. `Project::assets` already gives the assets before build filtering; the cases `builds/assets/beside-fragment` and `projects/quill` list the same sets.
- **The shared model didn't load with phase 08's loader (fixed by phase 10).** `tests/conformance/_model/ascribe.toml` declares a `role` attribute on `quill-audience`, which Q27 reserves (`model-attribute-reserved`), and its `content-root = "files"` doesn't exist for single-file cases. Phase 08 fixed `examples/content-models/full.toml` but not this file, and three cases use `role` in their inputs (`attributes/value-set`, `attributes/set-on-set-valued-key`, `widgets/audience-heading-or-block`). Phase 10 fixed the model and the three cases (PR #14), and the adapter here now reads the model as phase 10's does (`load_str`, the content root from the case's kind).

### One implementation of the reference rules (consolidated with phase 10)

After phases 10 and 11 merged, their two implementations of the rules below were made one, in `tessera_resolve::references` and `tessera_resolve::fs`. `tessera-check` depends on `tessera-resolve` and calls them; `tessera_check::tests::parity` builds both projects over one tree with every kind of reference problem and asserts they report the same problems at the same places, with the same file ids.

1. **Discovery of source files**: `FileSystem::sources` (exactly `.md`, skipping dot-names, unreadable entries listed). `tessera_check::Project::load` uses `DiskFs`, and still stops on an unreadable source (Q52).
2. **The exact-case probe**: `FileSystem::probe`. `tessera-check`'s own `Project::lookup` and `Lookup` are gone.
3. **The boundary**: `Layout::is_allowed` (Q10).
4. **What a destination names**: `reference_target` (phrase substitution in destinations, Q54; external or local; source or asset; route-like, Q22).
5. **Whether it's there**: `resolve_reference`, against a `SourceSet` (both projects implement it) and a `FileSystem`, including the route-to-page mapping (Q55: `route.md`, else `route/index.md`; `Resolution::Route::page_exists` says whether to offer the fix).
6. **The file-level issue**: `reference_issue` and `include_issue`, at the destination as written (Q53, `destination_span`, now also on `Reference::destination_span`), with the `link-route` fix. `Project::problems` adds only the page-level link-id checks.

Behavior that changed for `tessera check` in the consolidation: an `@include` of an existing file that isn't a source (`../README.md`) is now `include-target-missing` (Q63), a route-like link to a file whose name differs only in case is the case error rather than `link-route`, and a `#id`-only link in a fragment is `link-to-fragment` (Q64). Behavior that changed for the source index: source files have ids from 1 (id 0 is `ascribe.toml`, as in `tessera-check`), and its diagnostics point at the destination as written.

### Left open

- **Q61 to Q68** in `questions.md`: an empty slug (the requested warning); include paths and empty ids; includes of non-source files; a `#id`-only link in a fragment; `{heading=false}` with no id; what a cycle is; a heading's text for its slug; an invalid or repeated `@id`. Each has an implemented proposal marked `SPEC-QUESTION`. Q61 has no registry entry to report through, so `Heading::empty_slug` and `Project::empty_slug_headings()` record it.
- **Definitions' destinations** (`[r]: {api}x`) stay `Deferred` until the parser exposes definitions (phases 12 and 23).
- **`@include` depth**: expansion recurses once per nested include, with cycles cut, and no limit on depth. A chain of thousands of distinct files, each including the next, could overflow the stack; nothing in the spec bounds it.
- **Availability**: `AvailabilityMarker` parses each `@available` primary but doesn't check it against the model or resolve feature keys and scopes (phase 12).
- **Frontmatter** is parsed for `title` (and kept whole); which fields are validated, and `available` and `variant` on pages, are phases 10 and 12.
- **Expansion isn't cached**: `Project::problems` expands a target page for each link whose id isn't among the target's own, to look for the id in a fragment. Fine for now; phase 13 can cache per file.

