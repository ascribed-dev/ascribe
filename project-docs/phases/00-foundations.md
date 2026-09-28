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

- [x] `cargo build --workspace`, `cargo test --workspace`, `cargo fmt --all --check`, and `cargo clippy --workspace --all-targets -- -D warnings` pass.
- [x] `pnpm install` and `pnpm -r lint` pass.
- [x] CI runs both workflows on a pull request and passes.
- [x] The CommonMark baseline runs and reports its pass count.
- [x] The harness discovers the sample cases, reports them as skipped with the recorded reason, and fails a case whose tag has neither an adapter nor a skip entry.
- [x] `tests/conformance/README.md` fully documents the case format, including the `outline` schema, with a worked example for the Appendix B page.

## Out of scope

- Any parsing or validation logic.
- The content-model format (phase 01), the contracts (phase 02), and the real conformance cases (phase 03).

## Notes

- The `outline` format is the contract between the test authors in phase 03 and the parser phases. Keep it small and readable; test authors write it by hand.
- Choose crate names and directory paths exactly as the plan lists them. Other phases assume them.

## Handoff notes

### What was built

- **Rust workspace** (`Cargo.toml`, resolver 3, edition 2024): `crates/tessera-{core,syntax,model,check,resolve,emit,fmt,lsp}` as empty libraries, and `crates/tessera-cli`, whose binary is `tessera`. The binary is a dependency-free placeholder that answers only `--version` and `--help`; phase 10 replaces `main.rs` with the real command structure. Two test crates are also workspace members: `tests/commonmark` (`tessera-commonmark-suite`) and `tests/conformance` (`tessera-conformance`). Nothing is published (`publish = false` workspace-wide).
- **Toolchain and lints.** `rust-toolchain.toml` pins stable **1.90.0** with rustfmt and clippy; `rust-version = "1.90"` makes the resolver pick compatible dependency versions. `rustfmt.toml` uses the 2024 style. `[workspace.lints]` sets `missing_docs`, `unsafe_code = "forbid"`, and clippy's `unwrap_used`, `expect_used`, `panic`, `dbg_macro`, and `todo`, all denied in CI through `-D warnings`. `clippy.toml` allows unwrap, expect, and panic in tests. Every crate opts in with `[lints] workspace = true`; do the same in new crates. Shared dependency versions live in `[workspace.dependencies]` (comrak, insta, serde, serde_json, serde_yaml, thiserror, toml, plus the internal crates).
- **JS workspace.** pnpm 11 (`packageManager: pnpm@11.24.0`), Node 22 (`.nvmrc`). Empty packages `@tessera/elements`, `@tessera/astro`, `@tessera/cli`, and `tessera-vscode` (extensions aren't scoped), each with `src/index.ts`, a `tsconfig.json` extending `tsconfig.base.json` (strict, NodeNext), and scripts `lint` (`eslint .`), `typecheck` (`tsc --noEmit`), and `test` (`vitest run --passWithNoTests`). The tools are root devDependencies: ESLint 10 with typescript-eslint's `strict` and `stylistic` configs and `eslint-config-prettier` (`eslint.config.js`), Prettier 3 (`.prettierrc.json`), TypeScript 6.0, and Vitest 5. Root scripts: `pnpm lint`, `pnpm typecheck`, `pnpm test`, `pnpm format`, `pnpm format:check`.
- **CI.** `.github/workflows/rust.yml`: `cargo fmt --check` on Linux, then clippy (`-D warnings`), build, and test with `--locked` on Linux, macOS, and Windows, cached with `Swatinem/rust-cache`. `.github/workflows/js.yml`: frozen install, format check, lint, typecheck, and test, with the pnpm store cached. Both run on pull requests and pushes to `main`.
- **CommonMark baseline** (`tests/commonmark/`): the 652 examples of CommonMark 0.31.2 (the version comrak targets) in `spec.json`, a renderer-agnostic runner, and `baselines/comrak.toml`. **Unmodified comrak 0.55.0 passes 652 of 652**, with CommonMark options only and `render.unsafe = true` so raw HTML passes through as the spec expects. The baseline pins the exact failing set, so any change in either direction fails until the baseline is rewritten (`COMMONMARK_WRITE_BASELINE=1`). See `tests/commonmark/README.md`.
- **Conformance harness** (`tests/conformance/`): the case format and harness, documented in `tests/conformance/README.md`; `SKIPS.toml` with an entry for every area tag; two sample cases, `cases/samples/appendix-b` (the Appendix B page, outline only) and `cases/samples/include-and-selection` (a project case with `builds`); and self-tests (`tests/harness.rs`) that run a fixture suite through a fake adapter to cover every outcome. A test also keeps the README's worked example identical to the sample's `expect.yaml`.
- **`project-docs/questions.md`**, with the protocol and an entry template. No entries: nothing in this phase needed a spec interpretation.

### Interfaces later phases use

- **Case format.** `tests/conformance/README.md` is the contract between test authors (phase 03) and implementers. The pieces most likely to matter:
  - Cases live under `tests/conformance/cases/<area>/<case>/`, not directly under `tests/conformance/`. The shared model is `tests/conformance/_model/tessera.toml`; snapshots go in `tests/conformance/snapshots/`.
  - The outline has **structural** fields, always compared against a default (`form`, `attributes`, `primary`, `title`, `children`, a list's kind), and **content** fields, compared only when written (`text`, `binding`, `info`, `fenced`, `start`, `alt`). Text fields are raw source, whitespace-normalized.
  - A following-block directive and its bound block stay siblings; `binding: following-block` means "the next entry".
  - Top-level `diagnostics` are file-level only; page-level diagnostics go under `builds.<name>.diagnostics`. Columns count Unicode scalar values.
- **Area tags**, each skipped until its phase lands: `parser` (05), `structure` (06), `inline` (07), `model` (08), `slug` (09), `check` (10), `include` (11), `resolve` (12), `page-check` (14), `output` (18, 20), `format` (23). `provisional` is a marker tag and needs a `questions` list.
- **Adapters.** `tessera_conformance::ConformanceAdapter` (`name`, `handles_tag`, and optional `outline`, `diagnostics`, and `build`, each returning `Result<Option<T>, AdapterError>`), registered in `tests/conformance/tests/adapters/mod.rs`. Add Tessera crates as dev-dependencies of `tessera-conformance`; the harness library depends on none. When you register an adapter for a tag, delete that tag's `SKIPS.toml` entry, or the run fails with a stale-skip error. Use a `case` entry with `checks = [...]` for individual checks your phase can't produce yet.
- **CommonMark runner** for phase 04: `tessera_commonmark_suite::{load_bundled_examples, run, Baseline}`. Add a test with the fork's renderer and its own baseline file next to `baselines/comrak.toml`.
- **Running things:** `cargo test -p tessera-conformance --test conformance [-- --tag <tag>] [<case id substring>]`; `cargo test -p tessera-commonmark-suite --test commonmark`.

### Decisions

- **Toolchain 1.90.0** is the stable release installed where this phase was built, so every check was run on it. Bump it deliberately, in its own commit.
- **`serde_yaml` 0.9** reads `expect.yaml`. It's marked deprecated upstream but is stable and complete for this use; if it ever needs replacing, `serde_yaml_ng` has the same API. It follows YAML 1.2, so `no` stays a string; floats in attribute values are rejected so `3.10` can't silently become `3.1`.
- **TypeScript 6.0**, not 7: typescript-eslint 8 supports TypeScript below 6.1, and TypeScript is pinned with `~6.0.3` to stay inside that range.
- **Prettier ignores all markdown**, plus `crates/`, `tests/`, `examples/`, and `project-docs/`. SPEC §10 warns that general formatters break Tessera sources, and the docs are hand-wrapped.
- **All JS packages are `"private": true`** so nothing can be published by accident. Phases 22 and 27 remove the flag when publishing is approved.
- **`.gitattributes` forces LF** (`* text=auto eol=lf`), so byte-for-byte fixtures behave the same on Windows.
- **The sample cases' tags** are `structure` (Appendix B) and `include` plus `resolve` (the project sample). The project sample assumes the shared model declares the builds `site` and `cloud-pdf` from SPEC §9.3; phase 03 should rename them there if the model uses other names.

### Left open

- **CI** ran both workflows on the phase 00 pull request (#2) and passed on Linux, macOS, and Windows. The workflows haven't run on a push to `main` yet, since that happens only after merge.
- **Diagnostic slugs aren't validated** against `tests/conformance/diagnostics.toml`, which doesn't exist until phase 02. Once it does, phase 02 or 03 should make the harness reject unknown slugs in `expect.yaml` (a check in `Case::load` or the runner).
- **`project-docs/questions.md`** is created here with no entries. Phase 01, running in parallel, may also add it; if so, merge the two, keeping this header and every entry.
- **Intermediate commits.** The first commit lists `tests/commonmark` and `tests/conformance` as workspace members before the commits that add them, so the first two commits don't build on their own; the branch as a whole does.
