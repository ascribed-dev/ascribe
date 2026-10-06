# Optimization

> **Finished** on 6 October 2026. Every measure below has its final number, and each one that missed its target says why. The truth now lives in [ARCHITECTURE.md](../../ARCHITECTURE.md) and [AGENTS.md](../../AGENTS.md), the decisions that still bind in [decisions.md](../decisions.md#clean-ups-and-checks), the lints in [lints.md](../lints.md), and what lives outside the repository in [outside.md](../outside.md). This folder is history.
>
> The crates were renamed from `tessera-*` to `ascribe-*` in phase 8A ([#151](https://github.com/ascribed-dev/ascribe/pull/151)). This plan and its phase files keep the old names, as they were written.

A clean-up of Ascribe's code and infrastructure, done on purpose and in order: find out what we have, make it safe to change, then remove what's repeated, slow, or left over. It also leaves trails, so the next round of growth is easier to follow than this one was.

Ascribe grew fast because the problems are hard, and most of that growth is worth keeping. This plan isn't a rewrite and adds no features. It's the work of making what exists smaller to hold in your head.

This is a high-level plan. Phase 1 is done, and its [inventory](inventory.md) decided what the later phases contain. Each of those gets its own file before it starts.

## Why now

Three reasons, in order of weight:

1. **The agents plan multiplies whatever shape the code has.** It adds an MCP server, more commands, and editor tools, and its own rule is that every tool wraps a command ([agents, decision 2](../agents/README.md#decisions)). If a command isn't already a thin wrapper over one core function, that plan builds a second and third copy of the logic around it.
2. **Reviews keep finding the same kinds of problem.** See [What reviews have shown](#what-reviews-have-shown). Fixing them one pull request at a time treats each as new.
3. **It's cheap before 1.0.** Names, crate boundaries, and file layouts can still change without breaking anyone.

## What exists today

The [inventory](inventory.md) holds the numbers, and this plan doesn't repeat them. Its summary: builds, tests, and CI are fast; speed already has budgets; the usual lint rules are already enforced; and the work worth doing is narrower than this plan first assumed.

## What reviews have shown

Patterns from reviewing pull requests #62 to #118, which are why this plan exists. Phase 1 counted how wide each runs, and two turned out small: workarounds that outlive their cause (none are open today) and unmeasured costs (budgets exist, with two commands missing from them). The inventory's [findings](inventory.md#findings) are the current list; this table is the history.

| Pattern | Example |
|---|---|
| The same fact is written in several places | The list of source folders is in `ascribe.toml`, `ci.yml`, Netlify's `ignore`, and `site-npm.yml` (#109). The back-test found six stale passages in the docs (#112). |
| The same bug is fixed once per surface | Symbolic links and paths were fixed in the editor (#73), in sources (#99), and in snippets and includes (#118), and two gaps remain. Nothing owns "read a project file safely". |
| A language change doesn't reach every surface | Links to fragment headings shipped (#115) without rename or completion following. No list says what a language change must touch. |
| A workaround outlives its cause | The docs site kept overrides for bugs fixed upstream (#114, #117). Nothing records when each can go. |
| One change breaks another surface | Code titles removed the source anchor that review needs (#117). Dark defaults break the live site until the next canary (#114). |
| A cost appears unmeasured | Diff time doubled (#79). A quadratic path in comment rendering (#62). A new heading lookup with no number (#115). |
| A decision is made and not recorded | A pull request cited a decision on #82 that the issue doesn't hold (#112). |
| Things are left behind | A one-time npm token, a scratch repository, and a `latest` tag pointing at a canary. |
| Some platforms are checked less | Link tests are Unix-only; most hand checks skip Windows. |

One likely cause sits behind several of these: much of the code arrives as separate pull requests written in parallel, each sensible alone, each adding the helper it needs. A rule holds there only if it's written where an implementor reads it, or enforced by a test.

## Proposed decisions

Proposed, not settled. Once agreed, they bind every phase, as in the other plans. Decisions 2, 4, 6, 7, and 9 held, and bind the work after the plan: they're decisions 37 to 41 in [decisions.md](../decisions.md#clean-ups-and-checks).

1. **Measure first.** Nothing is cleaned up before phase 1's inventory exists. Every later phase names the number it means to move.
2. **A clean-up changes no behavior.** A pull request in this plan produces the same outputs as before, and shows it with phase 2's comparison. A bug found along the way is fixed in its own pull request, or filed.
3. **No rewrites.** A crate or package is reshaped in steps that each pass the comparison. If something seems to need replacing whole, stop and report.
4. **Contracts don't move, and each has a check.** `SPEC.md`, the output contracts in `docs/content/contracts/`, the CLI's JSON, and published package APIs are fixed lines. What is free to change is everything behind them. A contract with no test behind it gets one in this plan. The one exception is the JSON output's page format, `"tessera-page"`, which phase 8A renames with the crates ([decisions.md](../decisions.md#the-optimization-plan), decision 32).
5. **Every finding ends in one of three places:** fixed, filed as an issue, or written down as accepted, with the reason. Nothing is left as "noted".
6. **A rule is enforced where it can be.** A test, a lint, or a generated file beats a sentence in a guide. A written rule is for what can't be checked.
7. **Each fact has one home.** Where a fact must appear twice, one copy is generated from the other, or a test compares them.
8. **This runs ahead of the agents plan.** That plan starts once all of this one is done ([decisions.md](../decisions.md#the-optimization-plan), decision 31), so phase 5's work on [command paths](inventory.md#command-paths) is in place before it.
9. **Speed has budgets.** Performance work targets a stated number on a named corpus. No change is made because code looks slow.

## Phases

Each phase has its own file, written for an implementor who hasn't seen the others. A phase is split into parts, and a part is one pull request or a short series.

| Phase | File | What it leaves behind |
|---|---|---|
| 1 | [Inventory](inventory.md) | Done |
| 2 | [Safety net](phase-2-safety-net.md) | Proof that a change altered no output; baselines for what has none |
| 3 | [Trails](phase-3-trails.md) | A map, a first-read file, a decision log, change checklists, and three lints tried |
| 4 | [One home per fact](phase-4-one-home-per-fact.md) | Shared names and JSON shapes generated; repeated facts tested |
| 5 | [One home per job](phase-5-one-home-per-job.md) | File reading through one place; every command a thin wrapper with typed failures |
| 6 | [Infrastructure](phase-6-infrastructure.md) | `main` stays green between canaries; dependencies audited; leftovers gone |
| 7 | [Speed](phase-7-speed.md) | `diff` with a short path; a release profile, with numbers |
| 8 | [Names and close](phase-8-close.md) | The `tessera` question settled, the measures reported, the plan closed |

### What can run at the same time

```text
now ──┬─ 2A comparison ───────┬─ 4A names ── 4B shapes
      ├─ 2B baselines ──────┐ ├─ 4C packages
      ├─ 2C Windows links ─┐│ ├─ 4D facts
      ├─ 3A map            ││ ├─ 5B commands ── 5C failures ──┐
      ├─ 3B decisions      │└─┼──────────────────────────────┴─ 7A diff
      ├─ 3C lints          └──┼─ 5A file reading
      ├─ 6A canary wait *     ├─ 5D one parser
      ├─ 6B audits            └─ 7B release profile
      └─ 6C leftovers *                                  all ── 8
```

`*` needs the maintainer: 6A a choice (made: [open question 4](#open-questions)), 6C the accounts.

**Can start today, all at once (up to nine implementors):** 2A, 2B, 2C, 3A, 3B, 3C, 6A, 6B, and 6C's preparation.

**After 2A merges:** 4A, 4C, 4D, 5B, 5D, and 7B, all at once. 5A too, once 2C has merged.

**Chains, where one waits for another:**

| First | Then | Why |
|---|---|---|
| 2A | Every part of 4, 5, and 7 | They prove themselves with the comparison |
| 2B | 7A | It needs the baselines it will beat |
| 2C | 5A | The Windows link tests catch what file reading changes |
| 4A | 4B | The same TypeScript files |
| 5B | 5C | The same functions in the CLI's commands |
| 5B | 7A | Both change `diff` and `drift` |
| Everything | The agents plan | [Decision 8](#proposed-decisions) |
| Everything | 8 | It measures the result |

**Not at the same time, though neither waits for the other:**

| These two | Share |
|---|---|
| 4A and 5D | `crates/tessera-emit/src/render/` |
| 5A and 5B | `crates/tessera-cli/src/commands/fmt.rs`; 5B has it, 5A takes it last |
| A lint kept in 3C, and any part of 5 | Many Rust files. The lint merges before phase 5 starts, or after it ends |
| 8A (a rename), and any Rust pull request | Every Rust file |

**The longest chain:** 2A, then 5B, 5C, and 7A. The agents plan waits for all of this plan, not only that chain ([open question 1](#open-questions)).

## Measures

Taken from the inventory, and again at the end. Each is a count a finding names; lines of code isn't one, since size isn't the problem.

| Measure | Before | Target | After (6 October 2026) |
|---|--:|--:|--:|
| Pushes to `main` that fail the site check, of the last 10 | 4 | 0 | **0**: 8 passed and 2 were cancelled by a newer push (#129 to #149) |
| File reads outside `FileSystem` | 34 | 0, or each with a stated reason | **25, each with its reason** in an `Outside FileSystem:` comment, in 12 files; `crates/ascribe-resolve/tests/file_reads.rs` fails on one without |
| Source files holding `data-ascribe-source` as a literal | 14 | 2 (one per language, generated) | **5**: its one home, `crates/ascribe-core/src/names.rs`, and a generated `names.ts` in each of the four packages that use it. Accepted: phase 4A generated one module per package so no package gains a dependency, and `crates/ascribe-core/tests/names.rs` fails on a literal anywhere else |
| TypeScript files declaring a JSON shape Rust writes, untested | 7 | 0 | **0**: the three `shapes.ts` are generated from the Rust types (4B) |
| Facts in several files with no test between them | 4 | 0 | **0**: the Node and Rust versions, the glibc floor, and the docs' source folders (`scripts/docs-site/facts.test.ts`, 4D) |
| Ways the commands load a project | 2, plus the language server's | 1, plus the language server's | **1**, plus the language server's (5B) |
| Commands whose core is in the CLI crate | 2 | 0 | **0** (5B) |
| Functions returning an error as a string | 10 | 0 | **0** (5C) |
| Lines in library crates that print | 7 | 0 | **1**: the language server's log, `crates/ascribe-lsp/src/log.rs`, since standard error is its log by protocol. Accepted; a lint denies any other (5C) |
| Commands timed with no baseline | 2 | 0 | **0** (2B) |
| `diff` on 3,000 pages, nothing changed | 1.3 s | At most 1.3 times `check`'s time (phase 7A; [RESULTS.md](../../tests/corpora/RESULTS.md#where-diff-and-drift-spend-their-time)) | **0.82 s against `check`'s 0.66 s, 1.24 times**, on the runner. Phase 7A's first target, a quarter of the time, was missed and accepted when #150 merged: `check`'s time is the floor, since `diff` counts the working tree's errors |
| Release binary, macOS arm64 | 10.5 MB | 7.0 MB, without a slower `check` (phase 7B; every platform in [RESULTS.md](../../tests/corpora/RESULTS.md#the-release-profile)) | **6.99 MB**, with `check` 3 percent faster (7B); 5D's one parser has made it smaller since, by 4.5 percent on Linux |
| Peak memory on 3,000 pages | Unmeasured | Recorded in phase 2 (`tests/corpora/baselines/perf.json`, `memory/*`); stays within the baseline's margin | **Recorded, and within it**: every `memory/*` metric within 2 percent of its baseline; `diff` with nothing changed fell from 547 MB to 371 MB (7A) |

The "After" column was taken on `main` at `b27d9f8`, the way the inventory took each: the commands on the Corpora workflow's runner ([run 37542947220](https://github.com/ascribed-dev/ascribe/actions/runs/37542947220)), and the counts from the code. The rest of the inventory's headline numbers are in its [After](inventory.md#after) section.

## Out of scope

- New features, and changes to what the language allows.
- Changes to anything [decision 4](#proposed-decisions) calls a contract.
- A general lint pass, a pass over `.clone()` calls, or parallelism in `build`. [Decision 9](#proposed-decisions) covers the last two: nothing there misses a budget.
- Splitting large files for its own sake.
- Replacing a tool we depend on (the parser, Astro, the test runners), unless a measure shows one costs more than it gives. That would be its own plan.
- The plans in `project-docs/agents/` and `project-docs/editor-ui/` themselves. This plan may say they should change; it doesn't change them.

## Open questions

Answered by the maintainer on 6 October 2026, and recorded in [decisions.md](../decisions.md#the-optimization-plan) as decisions 31 to 34.

1. Does the agents plan wait for phase 5's command work ([decision 8](#proposed-decisions)), or do the two run side by side? **It waits for all of this plan**, not only phase 5.
2. Is renaming the crates on the table, or is `tessera` staying? **The crates are renamed** in phase 8A, and the JSON output's page format, `"tessera-page"`, becomes `"ascribe-page"`: Ascribe hasn't launched, so that contract can still change. Nothing is published under the name, so no new npm package or crate is needed.
3. How much time is this worth: a fixed budget, or until the measures are met? **Until the measures are met.** A target that turns into a slog is accepted with the reason ([decision 5](#proposed-decisions)), so it doesn't hold up the close.
4. Which of phase 6's three options for the canary wait do you prefer? **Option 1**, with option 2 as a manual button, as phase 6 recommends. Actions jobs can't end neutral, so while waiting the check passes with a warning annotation and a summary line saying it's waiting.

Answered by phase 1: the parser fork is small to maintain (76 added lines) and stays. Its only work is the "one parser" test in phase 5.

## What we'd do differently

**What the first sketch got wrong.** It assumed old workarounds, slow CI, no performance budgets, a costly fork, and 2,600 lines of workflows. The inventory found none of those ([What the first sketch got wrong](inventory.md#what-the-first-sketch-got-wrong)), and the plan shrank to what it measured. Measure before planning the phases, not after.

**What the reviews added.**

- **The safety net needed its own test.** For the first pull requests of phase 4, `outputs unchanged` built the base and the change into one target folder, so Cargo reused the base's crates and the job passed without comparing anything (#145). Every green result before the fix had to be taken again. A comparison should prove it compared two different things, from its first day.
- **An exception has to be as narrow as the decision.** The first way to accept a recorded output change let every difference through; the review sent it back to name each file (#143). The same rule served 8A.
- **Parallel pull requests collide on the guards, not only on the code.** 5A's check on file reads and 5B's move of `fmt` broke each other (#142, #144). The "Not at the same time" table named the shared file but not the shared check.
- **The `optimization` label belongs on a pull request when it's opened.** Adding it later cancels the first CI run and leaves a red "all checks".

**What took longer than expected.**

- **Phase 5,** because 5A and 5B both needed `fmt` and had to merge in turn, and 5A merged `main` twice.
- **Phase 4A,** which needed a decision partway through (decision 35, on the bundled scripts' bytes).
- **Phase 7A** missed its first target, a quarter of the time, because `diff` must count the working tree's errors, which takes as long as `check`. The target was restated against `check` ([Measures](#measures)).

**What we'd measure next time.** CI time wasn't in the Measures table, and it grew: a pull request takes a minute longer and a quarter more machine time, mostly in the Windows Rust job and in phase 2A's own `determinism` test ([#154](https://github.com/ascribed-dev/ascribe/issues/154)). A plan that adds checks should budget their time like any other cost (decision 9).

## Left open

Each is filed, so nothing here is only "noted" ([decision 5](#proposed-decisions)):

- [#130](https://github.com/ascribed-dev/ascribe/issues/130): library crates' public functions that no other crate uses.
- [#137](https://github.com/ascribed-dev/ascribe/issues/137): the output comparison fails when run twice into the same `--out`.
- [#139](https://github.com/ascribed-dev/ascribe/issues/139): `@ascribed/astro`'s declarations import types from its dev dependencies. Waiting on the maintainer's call.
- [#152](https://github.com/ascribed-dev/ascribe/issues/152): `ascribe fmt` follows a link out of the content root, the gap phase 5A left in `crates/ascribe-fmt/src/files.rs`.
- [#153](https://github.com/ascribed-dev/ascribe/issues/153): a conformance case that doesn't test what it says, found during the rename.
- [#154](https://github.com/ascribed-dev/ascribe/issues/154): CI on pull requests got slower.
- [#155](https://github.com/ascribed-dev/ascribe/issues/155): `pnpm test` fails locally after `pnpm build:all` on Linux x64.
