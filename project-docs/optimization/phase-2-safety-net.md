# Phase 2: Safety net

Part of [Optimization](README.md). Needs phase 1 (done). Rust, TypeScript, and CI.

**Runs with:** phase 3, at the same time. Part A must merge before any pull request of phases 4, 5, or 7 opens. Parts B and C can run beside part A.

## Goal

A way to prove that a clean-up changed nothing a user can see, and numbers for the things nothing measures yet. Every later phase leans on part A: [decision 2](README.md#proposed-decisions) says a clean-up changes no behavior, and this is how a pull request shows it.

## Context

- The [inventory](inventory.md), sections "Speed", "Rules in force", and "Tests".
- `crates/tessera-cli/src/commands/build.rs`: `ascribe build` and its `--emit site|plain|json` outputs. A project's output directory comes from `[project] output-dir` in its `ascribe.toml`.
- The projects to build: every folder under `examples/` that has an `ascribe.toml` (some are nested, as in `examples/monorepo/`), and `docs/`. `examples/getting-started` has a broken link on purpose, so `build` stops there with an error; compare its `check` output instead.
- `tests/corpora/`: `benches/perf.rs` times the commands, `src/perf.rs` and `corpora compare [--record]` check them against `baselines/perf.json`, and `RESULTS.md` explains the margin. `.github/workflows/corpora.yml` runs it weekly.
- `tests/corpora/synthetic/`: the 3,000-page project every benchmark shares.
- Files in the output path that use `HashMap` or `HashSet`: in `tessera-diff`, `align.rs`, `tree.rs`, `gitfs.rs`, `html/mod.rs`; in `tessera-emit`, `emitter.rs`; in `tessera-resolve`, `project.rs`, `fs.rs`, `slug/github.rs`, `incremental/cache.rs`, `build/modes.rs`, `build/mod.rs`, `build/availability.rs`, `snippet/mod.rs`.
- Rust tests that run only on Unix: `crates/tessera-cli/tests/sources.rs`, `crates/tessera-model/tests/load.rs`, `crates/tessera-resolve/src/fs.rs`, `crates/tessera-resolve/tests/includes.rs`, `crates/tessera-resolve/tests/snippets.rs`, `crates/tessera-sources/tests/fetch.rs`. All are about symbolic links except the ones in `sources.rs` (a fake `git` script) and `fs.rs` (a folder locked by its Unix mode).
- `.github/workflows/ci.yml` and `rust.yml`: how jobs are added, and that Rust runs on Windows for pull requests.

## Design

### Part A: the same output, twice and across two binaries

One pull request, in this order.

1. **Determinism.** A test that builds each project twice with one binary, into two directories, and fails on any byte that differs: every file of the site, plain, and JSON outputs, and the text and JSON of `check`, `diff`, and `drift` (the last two against a fixed base in a temporary repository). If it fails, find the iteration order that reaches the output and fix it there: sort before writing, or use an ordered map. Fix only what the test shows; don't replace hash maps that never reach an output.
2. **The comparison.** `scripts/compare/outputs.ts`, run like the other scripts under `scripts/`:
   - It takes two binaries (`--before <path> --after <path>`), or `--base <revision>`, which builds the `before` binary from that revision in a temporary worktree.
   - For each project it runs `build` with every output and `check --format json` with both binaries, and compares the results file by file.
   - It prints each file that differs with a short diff, then one line per project (`same` or `N files differ`), and exits `1` when anything differs.
   - Paths inside outputs that hold the build's own directory are normalized, so two checkouts compare equal.
3. **In CI.** A job in `ci.yml` that runs the comparison against the pull request's base when the pull request carries the label `optimization`. It reports and fails on a difference; it isn't part of `all checks` for other pull requests.
4. **For the JS packages:** the same idea, one level up. Build `examples/astro-site` with the packages at both revisions and compare `dist/`. If hashes in file names make that impractical, compare the HTML with hashes stripped, and say so in the pull request.

### Part B: numbers nothing records

One pull request. It can run beside part A.

1. **Baselines for `diff` and `drift`.** `benches/perf.rs` already times them; `baselines/perf.json` has no entry for them, so the weekly check can't fail on them. Record them with `corpora compare --record` on the CI machine (run the Corpora workflow from the branch and take its results; a laptop's numbers don't belong in the file, whose `machine` field names the runner). Add no absolute target; the margin is enough.
2. **Memory.** Add peak memory to the benchmark for `check`, `build` (first), and `diff` on the 3,000-page project, and for the language server after loading it and after 100 edits. Record them as metrics in `perf.json` with the same margin rule. Use the platform's own measure of peak resident memory; don't add an allocator.
3. **`RESULTS.md`** gains the new numbers.

### Part C: link tests on Windows

One pull request. It can run beside parts A and B.

For each Unix-only test in the six files above, add a Windows variant that makes the same kind of link the Windows way: a directory junction for a linked folder, and a symbolic link for a file where the runner allows one (GitHub's Windows runners do; skip with a printed reason where creation fails with a privilege error, don't fail). Share one helper for making links, in a test support module, in place of per-file `cfg` blocks.

A test that fails on Windows has found a bug. Don't change the product in this pull request: mark the test ignored with the issue's number, file the issue, and list it in the pull request. Phase 5's file-reading work fixes it.

## Tasks

1. Part A: the determinism test and any ordering fixes; the script; the CI job; the JS comparison.
2. Part B: the two baselines; the memory metrics; `RESULTS.md`.
3. Part C: the shared link helper; the Windows variants; issues for what they find.
4. `CONTRIBUTING.md`: a short section on running the comparison before a clean-up pull request.

## Out of scope

- Speeding anything up (phase 7).
- Merging test programs or changing the test runner.
- Fixing bugs part C finds.
- Windows runs of the JS suites beyond what CI has.

## Acceptance criteria

- Building any project twice with one binary gives identical bytes, and a test holds that.
- `scripts/compare/outputs.ts --base HEAD` reports `same` for every project.
- Changing one word an emitter writes makes the script exit `1` and name the files.
- `baselines/perf.json` has entries for `diff`, `drift`, and peak memory, and the Corpora workflow passes with them.
- Each of the six files has link tests that run on Windows, or a skip with a printed reason.

## Stop and report if

- Determinism fails for a reason that sorting doesn't fix, such as an output that depends on the clock or the machine.
- A difference between two runs is in a published contract's content, not its order.

## Verify

```sh
cargo test --workspace --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo fmt --all --check
pnpm format:check && pnpm lint && pnpm typecheck && pnpm test
node scripts/compare/outputs.ts --base HEAD
```

## Commits

1. "Test that every output is the same from one run to the next"
2. "Add a script that compares two binaries' outputs"
3. "Record baselines for diff, drift, and peak memory"
4. "Run the link tests on Windows"
