# Phase 1: The contract

Part of [Content checks](README.md). Needs no other phase of this plan; shares work with the agents plan's phase 1. Rust, and the generated docs.

## Goal

Every diagnostic says what kind of next step it has, and a project can say how loudly each advisory check speaks. No new checks: this phase gives the 141 that exist the shape the new ones will arrive in.

## Context

- `tests/conformance/diagnostics.toml`: the registry. Each entry has `code`, `slug`, `severity` (`error` or `warning`), `level` (`file` or `page`), `message`, and `fix`. `crates/ascribe-core/src/diagnostics.rs` lists them in the same order; a test compares the two.
- `crates/ascribe-check/src/registry.rs`: the registry embedded in the binary. Its `Entry` doesn't read `fix` yet.
- [The agents plan's phase 1](../agents/phase-1-check.md): `help`, `docs`, and each fix's `applicability` in `check`'s JSON, and `codeDescription` in the server. If it has landed, build on it. If not, build those fields as that file describes, here, and say so in the pull request.
- `tests/conformance/tests/docs.rs`: generates the diagnostics reference's fragments in `docs/content/_generated/`.
- `docs/content/reference/cli.md` and `docs/content/contracts/json-reports.md`: `check`'s options, exit codes, and JSON. `crates/ascribe-cli/src/shapes.rs`: the JSON Schemas generated from the Rust types.
- `crates/ascribe-lsp/README.md`, "Diagnostics": what the server publishes, and how severities map to LSP's.
- `docs/content/reference/content-model.md`: how a section of `ascribe.toml` is specified, and §1.4's rule that unknown keys are errors. `docs/content/contracts/content-model.md`: the contract.
- `project-docs/checklists.md`: the lists for a change to a command and to the content model.

## Design

### The kind of next step

A new registry field, `next`, one of `fix`, `choose`, `write`, `outside`, `review` (README decision 2). Classify all 141:

- `fix` when every instance has a fix Ascribe can apply. Check against the fixes the checks actually offer, not against the `fix` paragraph.
- `choose` when the author picks among things Ascribe can list: a link's target, an attribute's allowed values, a declared phrase.
- `write` when neither holds.
- None of today's diagnostics should need `outside` or `review`. If one seems to, stop and report: it may be a quality check filed as an error.

`next` reaches `check`'s JSON on each diagnostic, the diagnostics reference (a column, or a line in each entry), and the server (in the diagnostic's `data`, for the extension and the agents plan's prompts).

### What a prompt carries

A second registry field, `evidence`: a list of the named pieces of context this check's prompt includes beyond the message and the line, such as `allowed-values` or `close-matches`. The agents plan's phase 5 reads it. For today's diagnostics most lists are empty or one item; the field exists so new checks have somewhere to say it (README decision 8).

### The `advice` level

`severity` gains `advice`, below `warning`.

- `ascribe check` reports advice after errors and warnings, counts it separately in the summary (`0 errors, 2 warnings, 5 advice`), and never exits 1 for it, with or without `--deny-warnings`.
- The JSON's `severity` gains the value. That's an addition to an enumeration readers were told may grow; confirm the docs say so, and if they don't, this needs a new `schema_version`.
- The server publishes advice as LSP `Information`.
- No existing diagnostic changes its severity in this phase.

### `[checks]` in `ascribe.toml`

```toml
[checks]
page-size = "warning"
orphan-page = "off"
```

- A key is a check's slug; a value is `off`, `advice`, `warning`, or `error`.
- Only checks the registry marks `configurable = true` may appear. Every existing diagnostic is `configurable = false`, so in this phase the section accepts no key, and says why when given one: an invalid page isn't a matter of taste (README decision 3).
- An unknown slug is an error with a did-you-mean, as unknown keys are everywhere.
- The section's own settings for tools (`[checks.vale]`) come in later phases.

### The test that holds decision 1

One test over the registry: every entry has `next`, a `fix` paragraph, an entry in the generated reference, and an `evidence` list (which may be empty); every entry whose `next` is `fix` has at least one fix in the code, and every `review` entry is `configurable`.

## Tasks

1. `next` and `evidence` in the registry and in `Entry`; all 141 classified, with the table of counts by kind in the pull request, and the ten least obvious classifications explained.
2. `advice`: the registry, `check`'s text and JSON, the exit code, the server.
3. `[checks]`: the model, its validation, the reference, and the contract.
4. `next` in the JSON, the reference's generated fragments, and the server's diagnostic data. Bless the schemas and fragments.
5. The registry test.
6. Conformance cases: an advice-level diagnostic doesn't fail `check` under `--deny-warnings` (use a test-only entry if no real one exists yet); `[checks]` with an unknown slug; `[checks]` naming a check that isn't configurable.
7. `docs/content/reference/diagnostics.md` (what the kinds mean), `cli.md`, `content-model.md`, the contracts, and `CHANGELOG.md`. A decision in `project-docs/decisions.md` for the two contract changes.

## Out of scope

Any new check; acknowledging (phase 2); how prompts are delivered (the agents plan).

## Acceptance criteria

- Every diagnostic has a kind, and the reference shows it.
- `ascribe check --deny-warnings` on a project whose only findings are advice exits 0, and says how many there are.
- The outputs comparison (`node scripts/compare/outputs.ts --base main`) differs only in `check`'s JSON gaining fields. Name those files; nothing else changes.

## Verify

```sh
cargo test --workspace --locked
ASCRIBE_BLESS=1 cargo test -p ascribe-cli shapes && git diff --stat schemas docs/content/_generated
node scripts/compare/outputs.ts --base main
```

## Commits

1. "Say what kind of next step each diagnostic has"
2. "Add the advice level"
3. "Let a project set a check's level: [checks]"
