# Phase 8: Names and close

Part of [Optimization](README.md). Needs phases 2 to 7. Mostly docs; part A may be a large mechanical change.

**Runs with:** nothing. It's last, and part A must have no other pull request open against the Rust crates while it's in flight.

## Goal

The one naming question is settled, the measures are taken again, and the plan says it's finished and what it left behind.

## Context

- The [plan](README.md): the measures table, the decisions, and the open questions.
- The [inventory](inventory.md): the numbers to take again, and how each was taken.
- The crates are `tessera-*` and the product is Ascribe. The maintainer decided to rename them, and the JSON output's page format with them ([open question 2](README.md#open-questions); [decisions.md](../decisions.md#the-optimization-plan), decision 31). The name appears in crate names and paths, `use` lines, the fork (`comrak-tessera`, with `// TESSERA:` markers and a test that counts them), `Cargo.toml`, workflows, scripts, docs, and the plans.
- Nothing is published under the `tessera` name: every crate has `publish = false`. The npm packages are `@ascribed/*` and the binary is `ascribe`.
- `ARCHITECTURE.md`, `AGENTS.md`, `project-docs/decisions.md`, and `checklists.md`, from phase 3.

## Design

### Part A: the crate names

The maintainer decided to rename ([decision 31](../decisions.md#the-optimization-plan)).

- One pull request, nothing else in it, merged when no other Rust pull request is open.
- Crate names, folder names, `use` paths, workspace dependency keys, the binary's package name, and every mention in workflows, scripts, READMEs, and docs.
- The fork: rename the crate and the cargo feature or option names that we chose; keep the `// TESSERA:` markers only if the maintainer wants the patch to stay easy to compare with history, and update `FORK.md` and its counting test either way.
- Finished plans under `project-docs/` keep the old name: they're history. Add one line to each plan's top note saying the crates were renamed.
- The JSON output's page format, `format: "tessera-page"` (`crates/tessera-emit/src/json.rs`), is renamed too (to `"ascribe-page"`, unless the maintainer names another), with its documentation in `crates/tessera-emit/README.md`. Ascribe hasn't launched, so this contract can change ([decision 29](../decisions.md#process)). Any TypeScript that checks the value changes with it, and the pull request says whether the schema version rises. It's the one change in output: the comparison reports `same` for everything else.
- Otherwise no behavior changes, and no output or diagnostic contains the crate name. Check first: if another output, a diagnostic, or a published file prints it, stop and report.

### Part B: the measures

- Take every measure in the plan's table again, the way the inventory took it, on `main`.
- Add the results as a third column in the plan's table, and a short "After" section at the end of the inventory with the same headline numbers as its summary: build and test times, CI time, `check` and `diff` on 3,000 pages, binary size, peak memory.
- For each measure that missed its target: fixed late, filed as an issue, or accepted with the reason ([decision 5](README.md#proposed-decisions)).
- Count the review findings of the kinds in the plan's "What reviews have shown" table over the ten pull requests before phase 2 began and the ten most recent, from the reviews on GitHub. It's a rough count; say how each finding was classed.

### Part C: close

- The plan's `README.md` gets the top note every finished plan has: finished, when, and where the truth now lives.
- Proposed decisions that held become entries in `project-docs/decisions.md`: a clean-up changes no output; contracts have checks; each fact has one home; a rule is enforced where it can be; speed has budgets.
- `ARCHITECTURE.md`, `AGENTS.md`, and the crate READMEs are read once more against the code as it ended up, and corrected.
- `lints.md` and `outside.md` move out of this folder to where they'll be kept current (`project-docs/`, or beside `RELEASING.md`), since this folder becomes history.
- What this plan learned that the next one should know goes in a short "What we'd do differently" section at the end of the plan: what the first sketch got wrong, what the review added, and what phase took longer than expected.

## Tasks

1. Part A: the rename.
2. Part B: the measures, the "After" section, the misses dealt with.
3. Part C: the closing notes, the decisions carried over, the map corrected.

## Out of scope

- New clean-up work. What's found here is filed.
- Renaming anything users see, apart from the page format above.

## Acceptance criteria

- Every measure has a second number, taken the same way as the first.
- Every miss is fixed, filed, or accepted in writing.
- After the rename: `cargo test --workspace --locked` passes, the comparison reports `same` apart from the page format, and a search for the old name finds it only in history (finished plans, the changelog, the fork's upstream-facing notes).
- Every path and command in `ARCHITECTURE.md` and `AGENTS.md` exists.

## Stop and report if

- The crate name turns out to appear in an output, a diagnostic, or a published file, other than the page format.
- A measure can't be taken the way the inventory took it.

## Verify

```sh
cargo test --workspace --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
pnpm format:check && pnpm lint && pnpm typecheck && pnpm test
node scripts/compare/outputs.ts --base origin/main
```

## Commits

1. "Rename the crates"
2. "Report the optimization plan's measures"
3. "Close the optimization plan"
