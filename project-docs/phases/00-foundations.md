# Phase 00: Foundations

**Track:** Setup · **Start after:** none · **Parallel with:** 01 · **Unblocks:** 02, 04, 17

## Goal

Create the repository skeleton every other phase builds in: the Rust workspace, the JavaScript workspace, CI, and the conformance test harness. When this phase is done, other agents can start work without making structural decisions.

## Read first

- [PLAN.md](../PLAN.md): Key decisions, Repository layout, Testing.
- [SPEC.md](../../SPEC.md): skim the whole document; read §8 and Appendix B closely.
- [phases/README.md](README.md): Conventions, Ownership, and Release gates.

## Deliverables

- Root `Cargo.toml` defining a workspace with these crates as empty libraries (plus `tessera-cli` as a binary named `tessera`): `tessera-core`, `tessera-syntax`, `tessera-model`, `tessera-check`, `tessera-resolve`, `tessera-emit`, `tessera-fmt`, `tessera-lsp`, `tessera-cli`. Don't create `comrak-tessera`; phase 04 does.
- `rust-toolchain.toml`, `rustfmt.toml`, and workspace-level clippy settings.
- A pnpm workspace (`pnpm-workspace.yaml`, root `package.json`) with empty `packages/elements`, `packages/astro`, `packages/cli`, and `packages/vscode`, plus shared TypeScript, ESLint, and Prettier configs.
- CI (GitHub Actions): Rust fmt, clippy, and tests on Linux, macOS, and Windows; JS lint and tests.
- The conformance harness in `tests/conformance/`, with `SKIPS.toml`.
- The CommonMark spec suite in `tests/commonmark/`, running against *unmodified* comrak as a baseline.
- `project-docs/questions.md`.

## Tasks

1. **Workspace.** Create the Rust and JS workspaces above. `cargo build` and `pnpm install` succeed on a fresh clone.
2. **CI.** One workflow for Rust (fmt, clippy with `-D warnings`, test) across three operating systems, and one for JS. Cache dependencies.
3. **CommonMark baseline.** Add the official CommonMark spec tests (the `spec.json` for the version comrak targets) under `tests/commonmark/`, with a runner that parses each example with the comrak crate from crates.io and compares HTML. Record the pass count. Phase 04 reruns this against the fork.
4. **Conformance case format.** Define and document, in `tests/conformance/README.md`, how a conformance case is written:
   - A case is a directory. Single-file cases have `input.md`. Project cases have a `files/` tree.
   - A case may include `tessera.toml`. Otherwise it uses the shared fixture model at `tests/conformance/_model/tessera.toml`, which phase 03 writes.
   - `expect.yaml` holds the expected results:
     - `outline`: an abstract, implementation-independent description of the parsed structure. It describes directives (name, form, attributes, primary, title, binding), containers and groups with their children, and CommonMark block kinds (paragraph, list, list item, code, heading, blockquote, image). It must *not* depend on internal node types or spans.
     - `diagnostics`: expected diagnostics by slug (from the phase 02 registry), with a line number and optionally a column.
     - `builds`: per-build expectations for project cases (for example, which arms, pages, and assets survive).
     - Slots for expected outputs (`plain`, `site`, `json`), usually filled with `insta` snapshots by later phases.
   - Tags, so a runner can select cases by area (`parser`, `structure`, `check`, `resolve`, `output`, and so on).
5. **Harness.** A Rust test crate that discovers cases, deserializes `expect.yaml`, and exposes an adapter trait later phases implement, for example `trait ConformanceAdapter { fn outline(&self, case: &Case) -> Outline; fn diagnostics(&self, case: &Case) -> Vec<ExpectedDiagnostic>; }`.
6. **Skips.** `tests/conformance/SKIPS.toml` lists skipped cases or tags, each with a reason. The harness fails a case that has no adapter and no skip entry, and prints a summary of every skip, so skips are always deliberate and visible. Start with skip entries for every tag.
7. **Samples.** Two hand-written sample cases that exercise the format.
8. **Questions file.** Create `project-docs/questions.md` with a short header explaining the protocol from the phases README.

## Acceptance criteria

- [ ] `cargo build --workspace`, `cargo test --workspace`, `cargo fmt --all --check`, and `cargo clippy --workspace --all-targets -- -D warnings` pass.
- [ ] `pnpm install` and `pnpm -r lint` pass.
- [ ] CI runs both workflows on a pull request and passes.
- [ ] The CommonMark baseline runs and reports its pass count.
- [ ] The harness discovers the sample cases, reports them as skipped with the recorded reason, and fails a case whose tag has neither an adapter nor a skip entry.
- [ ] `tests/conformance/README.md` fully documents the case format, including the `outline` schema, with a worked example for the Appendix B page.

## Out of scope

- Any parsing or validation logic.
- The content-model format (phase 01), the contracts (phase 02), and the real conformance cases (phase 03).

## Notes

- The `outline` format is the contract between the test authors in phase 03 and the parser phases. Keep it small and readable; test authors write it by hand.
- Choose crate names and directory paths exactly as the plan lists them. Other phases assume them.

## Handoff notes

_To be filled in by the implementing agent._
