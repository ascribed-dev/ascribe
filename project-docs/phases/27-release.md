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
2. **Names (human).** Confirm availability of the GitHub organization, the npm scope and package names, and the VS Code Marketplace publisher. Prepare a report; don't register anything. As of the 2026-09-29 rename: the npm org `@ascribed` and the GitHub org `ascribed-dev` are claimed; the Marketplace publisher is still open.
3. **Versioning.** One version across the binary, npm packages, and extension. A changelog.
4. **Extension packaging.** Platform-specific VSIX packages (`vsce package --target` for each supported platform), each with the matching binary under `bin/`. Confirm the extension still prefers a project's own binary.
5. **Release workflows.** GitHub Actions that, on a version tag, build binaries for every platform, attach them to a GitHub release, publish npm packages (with provenance), and publish the extension packages. Leave them requiring manual approval.
6. **Documentation.** A getting-started guide (installing, `ascribe.toml`, a first page, Astro setup), a directive reference drawn from SPEC.md, the diagnostics list with codes and fixes, and the editor features.
7. **Release checklist.** Every step the person runs, in order, with how to verify each and how to roll back.

## Acceptance criteria

- [ ] Every release gate passes, with evidence in the report.
- [ ] Every open question is resolved or deferred with a reason.
- [ ] Release workflows pass a dry run.
- [ ] Each platform's extension package installs locally and works with `examples/quill`.
- [ ] Documentation covers installing, writing, building, and editing.
- [ ] The release checklist is complete and reviewed by a person.

## Out of scope

- Actually publishing. A person does that, following the checklist.

## Handoff notes

_To be filled in by the implementing agent._
