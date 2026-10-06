# Phase 7: Speed

Part of [Optimization](README.md). Needs phase 2, parts A and B: the comparison, and the baselines for `diff`, `drift`, and memory. Rust and the release configuration.

**Runs with:** phases 4 and 6. Part B is independent of everything else. Part A touches `crates/tessera-cli/src/commands/diff.rs`, `drift.rs`, and `crates/tessera-diff`; run it after phase 5's part B, which moves code in the same commands.

## Goal

The two things the [inventory](inventory.md#speed) found worth making faster or smaller, each with a number before and after. Nothing else is near a limit, and [decision 9](README.md#proposed-decisions) says nothing changes because it looks slow.

## Context

- The inventory: "Speed", findings 4 and 7. On 3,000 pages, on a laptop: `diff` takes 1.3 s whether one page changed or none, and 3.2 s when every page has a snippet; `drift` takes 0.6 s with nothing changed and 2.5 s with one region changed; `check` takes 0.48 s.
- `tests/corpora/benches/perf.rs`: the benchmark, including the `diff` and `drift` cases. `tests/corpora/RESULTS.md`: where the time went last time someone looked, and how it was found. `baselines/perf.json`: the recorded numbers after phase 2.
- `crates/tessera-diff/src/`: `lib.rs` (`compare_builds`, `compare_page_in`), `gitfs.rs` (reading a revision), `align.rs`, `tree.rs`, `drift.rs`.
- `crates/tessera-cli/src/commands/diff.rs`: loads the working tree's project, reads the base revision, resolves every build on both sides, then compares.
- `crates/tessera-resolve/src/incremental/`: how the language server avoids redoing work; `crates/tessera-lsp/src/review.rs` uses `compare_page_in` for one page.
- Pull request #79 and its review: the change that doubled `diff`'s time, and what the reviewer measured.
- `Cargo.toml`: no `[profile.release]`. `.github/workflows/release-build.yml` and `canary.yml`: where release binaries are built, for four platforms, with `zig` for the Linux targets (`scripts/release/zig-requirements.txt`). `scripts/release/smoke/`: the tests a built binary must pass.

## Design

### Part A: `diff` and `drift`

One pull request per change, each with the benchmark's numbers.

1. **Find where the time goes first.** Profile the four `diff` cases and the two `drift` cases on the 3,000-page project, with and without snippets, and write what you find into `RESULTS.md` before changing anything: how much is reading the base revision from `git`, loading and indexing each side, resolving, rendering, and comparing.
2. **A short path for what didn't change.** `git` already knows which files differ between the base and the working tree. A page can only differ if its own file changed, or a fragment it includes, a code file it shows, the content model, or the lock file. Everything else can skip resolving and comparing. The index of who includes what already exists for the language server; reuse it, don't build another.
3. **Snippets.** `diff` goes from 1.3 s to 3.2 s when pages have snippets, far more than `check` does (0.48 s to 0.78 s). Find why before fixing; reading each code file once per page that shows it, on both sides, is the first thing to rule out.
4. **`drift` with one region changed** takes four times as long as with none. The same question.

Targets, on the CI machine's baseline: `diff` with nothing changed and with one page changed, at most a quarter of today's time; `diff` with snippets, at most twice `diff` without. Record the new baselines with `corpora compare --record` so the weekly check holds the gain.

Results must be identical: the comparison script covers `build` and `check`, so for this part also compare `diff --format json`, `diff --format html`'s embedded data, and `drift --format json` before and after, on every benchmark case and on `docs/` against a base 20 commits back.

### Part B: a release profile

One pull request. Independent.

Add `[profile.release]` and measure each setting on its own, then together:

| Setting | Expect |
|---|---|
| `lto = "fat"` or `"thin"` | A smaller, somewhat faster binary; a slower release build |
| `codegen-units = 1` | The same direction |
| `strip = "symbols"` | Smaller; check that a panic's message is still useful |
| `panic = "abort"` | **Don't use it.** The language server catches a panic in a handler and keeps running (`catch_unwind` in `crates/tessera-lsp/src/server.rs`), which needs unwinding |
| `opt-level = "s"` | Smaller and usually slower; only if the size gain is large and `check` doesn't slow by more than 5% |

Report a table in the pull request: binary size for each of the four platforms, `check` and `build` time on the 3,000-page project, and release build time, before and after (today: 10.5 MB on macOS arm64, 0.48 s, and 28 s on a laptop). Choose the settings that cut size most without slowing `check`, and say what the canary and release workflows' build time becomes.

The binary ships in four npm packages on every install, so size is the gain. The cross-compiled Linux builds use `zig` as the linker: confirm link-time optimization works there by running the release workflow's build job from the branch, and that the smoke tests pass on every platform.

## Tasks

1. Part A: the profile written up; the short path; the snippet cost; `drift`; new baselines.
2. Part B: the measurements, the profile, the workflows' build confirmed.
3. `RESULTS.md` and `CHANGELOG.md` (faster `diff`; smaller downloads).

## Out of scope

- Parallelism in `build` or `check`. Neither misses a budget.
- A pass over `.clone()` calls or allocations.
- A cache on disk between runs.
- Making `check` faster. It's ten times inside its target.
- Loading the project once where the commands load it twice, unless the profile shows it's a large share of `diff`; if so, it's part of step 2.

## Acceptance criteria

- `diff` and `drift` give byte-identical results before and after, on every benchmark case and on `docs/`.
- The targets above are met on the CI machine, or the pull request says which wasn't and why.
- `baselines/perf.json` holds the new numbers, and the Corpora workflow passes.
- The release binary is smaller on all four platforms, `check` on 3,000 pages is no slower, and every smoke test passes.
- The language server still answers after a request that panics.

## Stop and report if

- The short path would need `diff` to trust something `git` reports that the project can't verify (a file outside the repository, a source in another repository).
- The time is mostly in `git` itself.
- Link-time optimization fails on a cross-compiled target.

## Verify

```sh
cargo test --workspace --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo bench -p tessera-corpora --bench perf --locked
node scripts/compare/outputs.ts --base origin/main
```

## Commits

1. "Record where diff and drift spend their time"
2. "Skip pages that can't have changed when comparing revisions"
3. "Read each code file once when comparing revisions"
4. "Record the new baselines for diff and drift"
5. "Add a release profile"
