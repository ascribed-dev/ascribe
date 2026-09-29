# Phase 18: `ascribe build`, plain markdown, and JSON

**Track:** Output · **Start after:** 12 · **Finish after:** 14 · **Parallel with:** 13, 14 · **Unblocks:** 20, 26

## Goal

Ship `ascribe build` with the two simplest outputs, plain markdown and JSON, including self-contained assets and safe replacement of previous output. Together they prove the resolution passes end to end.

## Read first

- [SPEC.md](../../SPEC.md): §9.1–§9.4, including Assets.
- `project-docs/contracts/output-layout.md` and `project-docs/contracts/assets.md` (phase 02).
- Handoff notes from phases 12 and 14.

## Deliverables

- `crates/tessera-emit`: the crate's structure, an `Emitter` trait, the plain-markdown emitter, the JSON emitter, and shared output handling (assets, manifests, staging).
- `crates/tessera-cli/src/build.rs`: the `ascribe build` subcommand.

## Tasks

1. **Emit from what survived.** Emitters walk phase 12's resolved tree and render what's there. They never re-derive what a build mode would keep. So a selection build that keeps several arms of a group emits them all, and a filter build emits the availability annotations still attached.
2. **Plain markdown.** Follow SPEC §9.4's table:
   - Notes become blockquotes starting with a bold type label and title (`**Tip: …**`).
   - `@steps` becomes the plain ordered list.
   - A group with one surviving arm becomes that arm's content. A group with several becomes one section per arm, each led by the arm's bold display label.
   - `@details` becomes the bold title followed by the content.
   - Availability, in badge and filter builds alike, becomes a line such as "Available: Quill Cloud (GA); self-managed (preview, 3.4+)", using display labels.
   - Project widgets become their declared plain fallback, or nothing.
   - Phrases, includes, and glossary links are resolved. Links become absolute URLs.
3. **JSON.** Serialize the resolved tree with a documented, versioned schema (a `schemaVersion` field). Include each node's source file and span, so custom consumers can point back at the source.
4. **Assets.** Copy every surviving asset reference into the output and rewrite references, exactly as the asset contract says. Output must work with the source directory removed.
5. **Output ownership.** Implement the output-layout contract: write to a staging directory, replace the previous output only on success, delete files the previous manifest listed that this build didn't produce, never delete a file the manifest never listed, and fail with an error instead of overwriting a file the build doesn't own.
6. **`ascribe build`.** Options: `--build <name>` (default: every build) and `--emit plain,json,site` (site comes in phase 20). Before emitting, call phase 14's `check_project`, and fail on errors with the same output as `ascribe check`.
7. **Parity test.** For the fixture projects with known problems, `ascribe build` reports exactly the diagnostics `ascribe check` reports for the same build.
8. **Snapshots.** `insta` snapshots of both outputs for the Quill project under each of its builds.

## Acceptance criteria

- [ ] `ascribe build` builds `examples/quill` under every build with both emitters.
- [ ] The cloud build's plain output keeps the whole package-manager group as labeled sections and shows the availability line.
- [ ] A test builds a page that includes a fragment whose image sits beside the fragment, deletes the source directory, and confirms the output's image reference resolves.
- [ ] A two-build test: build, remove a page from the source (and, separately, change a page so a selection build drops it), build again, and confirm the old output is gone while a user file placed in the output directory beforehand is untouched.
- [ ] Plain-markdown output contains no HTML and parses as ordinary CommonMark with an unmodified parser.
- [ ] The parity test passes, and snapshots are reviewed and committed.
- [ ] The JSON schema is documented in `crates/tessera-emit/README.md`.

## Out of scope

- Site output, the consumer profile, and web components (phase 20).

## Handoff notes

### What was built

- **`crates/tessera-emit`**:
  - `Emitter` (`emitter.rs`): `name`, `page_path`, `render_page(&PageContext, &ResolvedPage)`, and defaults for `place_asset` (mirrored path, relative reference), `generated` (files under `_ascribe/`), and `warnings`. `EmitContext::new(&Project, project_root, &Build)` holds the model, the build, file paths and line indexes (for JSON's `source`), and `absolute_url`. `emit(&dyn Emitter, &EmitContext, &ResolvedBuild) -> Emission` renders every page and lists each asset the pages use once, from `ResolvedPage::assets`.
  - `PlainEmitter` (`plain/`): follows SPEC §9.4's table; see the crate README. It walks the resolved tree and never looks at the build's mode.
  - `JsonEmitter` (`json.rs`): `schemaVersion` 1, one document per page (`.md` becomes `.json`), documented in the crate README.
  - `assets.rs`: `mirrored_path`, `relative_reference`, `encode_path`, `markdown_destination` (asset contract §3, §4). `labels.rs`: the availability line, arm labels, plain text of inlines.
  - `OutputDir` (`store.rs`): the whole output-layout contract, §4: lock (`File::try_lock`, so the OS releases it), previous manifest (refuses a file that isn't one, an unknown version, and entries that could leave the emitter root), conflict checks, ownership recorded before files move, unchanged files left alone, stale files removed with the directories they empty, final manifest, staging removed.
- **`crates/tessera-cli/src/commands/build.rs`**: `ascribe build [--build NAME]... [--emit plain,json] [--format text|json]`, plus the wiring (`Command::Build` and its arm in `cli.rs`, `pub mod build;`). `tessera-cli` gained `tessera-emit` and `tessera-resolve` as dependencies. Its README documents the command.
- **Questions Q111 to Q120** (below).

### Interfaces later phases use

- **Phase 20 (site output)**: implement `Emitter` for the site emitter and override `place_asset` (link targets under `_ascribe/files/`, with `Placement::url`, which the store already writes to the manifest) and `generated` (the Zod schema, under `_ascribe/`). `emit` and `OutputDir::replace` need no change; add `Emit::Site` to `commands/build.rs` (it's refused now) and swap `DefaultRouter::from_consumer` for the profile's router in `write_outputs` (`SPEC-QUESTION(Q119)`). `EmitContext::absolute_url` and `labels::availability_display` are reusable.
- **Phase 26**: `emit` plus `OutputDir::replace` per build and emitter is the pipeline to time; `EmitContext::new` indexes every file's lines for each call, so build one per build and share it across emitters.
- **Phase 14**: `commands/diagnose.rs::diagnose` is the one place `ascribe build` calls the checks.

### Decisions

- Every choice the spec leaves open is a question with the implemented answer: Q111 (a page starts with its title and page-level availability), Q112 (raw HTML is written as literal text), Q113 (widgets), Q114 (the availability line's wording), Q115 (arm labels; a note with no title), Q116 (image attributes and table alignment are lost in plain markdown), Q117 (an Ascribe file where a directory goes is removed; a directory where a file goes fails), Q118 (the JSON shape), Q119 (`ascribe build`: checks for every build first, one report, `--emit site` refused, default router), Q120 (the missing-origin warning isn't a registry diagnostic).
- **Following-block directives are applied by the plain emitter, not by the tree.** The tree keeps `@note`, `@details`, `@available`, and widgets that bind the next block as sibling directive blocks (`Bound::FollowingBlock`); the emitter wraps the block they bind, in stacking order (`plain::Renderer::blocks`).
- **Availability is rendered from `Annotation::text`** (the spec as shown, Q25) parsed again, so the line shows labels without re-deriving anything from the build. A page's own line comes from `Availability::spec`.
- **JSON omits the page's availability from each block's chain**: it's in the page's `availability`, and repeating it made every node several times larger.
- **Files are written as text or copied from the source** (`Contents::Text`, `Contents::Copy`): assets are never read into memory.

### Acceptance criteria

See the pull request for the status and evidence of each.

### Conformance

No case has `outputs:` yet, and none carries the `output` tag, so the `output` skip in `SKIPS.toml` stays: it covers phase 20's site output as well. The tests for this phase are in `crates/tessera-emit/tests/` (`plain.rs`, `store.rs`, `assets.rs`, `quill.rs` with the `insta` snapshots in `tests/snapshots/`) and `crates/tessera-cli/tests/build.rs`.

### Left open

- **Phase 14 is wired in, through one implementation.** `commands/diagnose.rs` (build selection, the unknown-build error, `check_all_builds` or `tessera_check::check_builds`, and the `builds_note` decoration) is called by both `ascribe check` and `ascribe build`, and both take `--build` more than once. `check_builds(project, &[&Build])` (new in `tessera-check`, at the reviewer's request) merges a subset of builds as `check_all_builds` merges all of them.
- **Q111 to Q120** are open, all implemented as proposed.
- **A case-only rename on a case-insensitive file system** (`Guide.md` to `guide.md`) makes the build fail with "isn't a file Ascribe wrote", since the manifest lists the old spelling: safe, and confusing. Fixing it needs telling a case twin of Ascribe's file from a user's file, which a case-insensitive listing can't do portably.
- **An output of a build removed from `ascribe.toml`** stays on disk: a build only replaces the outputs it writes, and nothing lists the ones it doesn't. An `ascribe clean` (or pruning the manifests of unknown builds) could remove them; it isn't in the phase.
- **Interrupted builds**: the invariant is tested by making a copy fail while staging (nothing changes), and by reading the code's ordering (manifest with old and new files, then moves, then removals, then the final manifest). There is no test that kills the process between steps.
- **Table alignment** is kept since Q116's resolution (`tessera_syntax::Table::alignments`); image attributes aren't part of plain markdown.
- **Route mapping** is `DefaultRouter`; Q55's inverse mapping (which page a route names) is still in `tessera-resolve`.
