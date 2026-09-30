# Phase 27: Release

**Track:** Release · **Start after:** all other phases · **Parallel with:** none · **Human-led: an agent prepares, a person publishes**

## Goal

Prepare Ascribe v1 for release, and verify the release gates. A person makes every outward-facing move: claiming names, publishing packages, creating releases.

## Read first

- [phases/README.md](README.md): Release gates.
- [PLAN.md](../PLAN.md): Scope, Key decisions (distribution, extension's binary).
- Every phase's handoff notes, for anything left open.
- `project-docs/questions.md`: every open question must be resolved or explicitly deferred.

## Deliverables

- A report on each release gate.
- Release workflows, ready to run but not run.
- Platform-specific VS Code extension packages, each bundling its binary.
- User documentation.
- A release checklist for the person publishing.

## Tasks

1. **Release gates.** Verify each gate in the phases README and report the evidence:
   - **No unintentional skips:** run the full conformance suite; `SKIPS.toml` is empty, or each remaining entry has a human-approved reason.
   - **Diagnostic parity:** the parity tests from phases 15 and 18 pass on every platform, for every build of every example project.
   - **End to end:** phase 21's end-to-end test passes on every platform (phase 22's CI).
   - **Performance:** phase 26's targets are met, or each miss is accepted by a human.
2. **Names (human).** Confirm availability of the GitHub organization, the npm scope and package names, and the VS Code Marketplace publisher. Prepare a report; don't register anything. As of the 2026-09-29 rename: the npm org `@ascribed` and the GitHub org `ascribed-dev` are claimed, and the VS Code Marketplace publisher `Ascribe` is claimed (2026-09-30; `packages/vscode/package.json` uses it). Every package is still `private: true`; this phase removes the flag from the packages it publishes.
3. **Versioning.** One version across the binary, npm packages, and extension. A changelog.
4. **Extension packaging.** Platform-specific VSIX packages (`vsce package --target` for each supported platform), each with the matching binary under `bin/`. Confirm the extension still prefers a project's own binary.
5. **Release workflows.** GitHub Actions that, on a version tag, build binaries for every platform, attach them to a GitHub release, publish npm packages (with provenance), and publish the extension packages. Leave them requiring manual approval.
6. **Documentation.** A getting-started guide (installing, `ascribe.toml`, a first page, Astro setup), a directive reference drawn from SPEC.md, the diagnostics list with codes and fixes, and the editor features.
7. **Release checklist.** Every step the person runs, in order, with how to verify each and how to roll back.

## Acceptance criteria

- [ ] Every release gate passes, with evidence in the report. (Every gate passes on macOS arm64; parity and end to end on the other platforms wait for manual CI runs.)
- [ ] Every open question is resolved or deferred with a reason. (Q171 to Q174 await approval.)
- [ ] Release workflows pass a dry run. (Their scripts pass locally for both macOS targets; the workflow's own dry run waits for Actions.)
- [ ] Each platform's extension package installs locally and works with `examples/quill`. (darwin-arm64 does; the other three are the workflow's smoke jobs.)
- [x] Documentation covers installing, writing, building, and editing.
- [ ] The release checklist is complete and reviewed by a person. (Complete; waits for review.)

## Out of scope

- Actually publishing. A person does that, following the checklist.

## Handoff notes

The report is [release-report.md](../release-report.md), and the checklist is [RELEASING.md](../../RELEASING.md).

- **Done:** version 0.1.0 everywhere (`scripts/release/version.mjs`); `CHANGELOG.md`; `private` removed from the seven npm packages, with repository metadata; Intel Macs (`darwin-x64`) dropped, at the owner's request; per-platform VSIX packaging; `scripts/release/{pack,publish,smoke-vsix}.mjs`; `.github/workflows/release.yml`; `docs/` and a root README; the diagnostics reference generated from the registry (a `fix` per diagnostic); `SKIPS.toml` emptied; the Q173 gap closed (`astro dev` watches assets outside the content root).
- **Verified locally (macOS arm64):** every gate, the pack and publish dry runs, and the extension smoke test (bundled binary, then the project's binary preferred).
- **Waiting on a person:** approving Q171 to Q174; making the repository public, since npm provenance needs it; creating the `release` environment and its two secrets; running `rust.yml`, `js.yml`, and the release workflow's dry run for Linux and Windows; reviewing the checklist; publishing.
