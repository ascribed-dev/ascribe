# Phase 18: `tessera build`, plain markdown, and JSON

**Track:** Output · **Start after:** 12 · **Finish after:** 14 · **Parallel with:** 13, 14 · **Unblocks:** 20, 26

## Goal

Ship `tessera build` with the two simplest outputs, plain markdown and JSON, including self-contained assets and safe replacement of previous output. Together they prove the resolution passes end to end.

## Read first

- [SPEC.md](../../SPEC.md): §9.1–§9.4, including Assets.
- `project-docs/contracts/output-layout.md` and `project-docs/contracts/assets.md` (phase 02).
- Handoff notes from phases 12 and 14.

## Deliverables

- `crates/tessera-emit`: the crate's structure, an `Emitter` trait, the plain-markdown emitter, the JSON emitter, and shared output handling (assets, manifests, staging).
- `crates/tessera-cli/src/build.rs`: the `tessera build` subcommand.

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
6. **`tessera build`.** Options: `--build <name>` (default: every build) and `--emit plain,json,site` (site comes in phase 20). Before emitting, call phase 14's `check_project`, and fail on errors with the same output as `tessera check`.
7. **Parity test.** For the fixture projects with known problems, `tessera build` reports exactly the diagnostics `tessera check` reports for the same build.
8. **Snapshots.** `insta` snapshots of both outputs for the Quill project under each of its builds.

## Acceptance criteria

- [ ] `tessera build` builds `examples/quill` under every build with both emitters.
- [ ] The cloud build's plain output keeps the whole package-manager group as labeled sections and shows the availability line.
- [ ] A test builds a page that includes a fragment whose image sits beside the fragment, deletes the source directory, and confirms the output's image reference resolves.
- [ ] A two-build test: build, remove a page from the source (and, separately, change a page so a selection build drops it), build again, and confirm the old output is gone while a user file placed in the output directory beforehand is untouched.
- [ ] Plain-markdown output contains no HTML and parses as ordinary CommonMark with an unmodified parser.
- [ ] The parity test passes, and snapshots are reviewed and committed.
- [ ] The JSON schema is documented in `crates/tessera-emit/README.md`.

## Out of scope

- Site output, the consumer profile, and web components (phase 20).

## Handoff notes

_To be filled in by the implementing agent._
