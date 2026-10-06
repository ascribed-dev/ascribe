# Optimization

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

Proposed, not settled. Once agreed, they bind every phase, as in the other plans.

1. **Measure first.** Nothing is cleaned up before phase 1's inventory exists. Every later phase names the number it means to move.
2. **A clean-up changes no behavior.** A pull request in this plan produces the same outputs as before, and shows it with phase 2's comparison. A bug found along the way is fixed in its own pull request, or filed.
3. **No rewrites.** A crate or package is reshaped in steps that each pass the comparison. If something seems to need replacing whole, stop and report.
4. **Contracts don't move, and each has a check.** `SPEC.md`, the output contracts in `docs/content/contracts/`, the CLI's JSON, and published package APIs are fixed lines. What is free to change is everything behind them. A contract with no test behind it gets one in this plan.
5. **Every finding ends in one of three places:** fixed, filed as an issue, or written down as accepted, with the reason. Nothing is left as "noted".
6. **A rule is enforced where it can be.** A test, a lint, or a generated file beats a sentence in a guide. A written rule is for what can't be checked.
7. **Each fact has one home.** Where a fact must appear twice, one copy is generated from the other, or a test compares them.
8. **This runs ahead of the agents plan.** Phase 5's work on [command paths](inventory.md#command-paths) finishes before that plan's phase 2 starts.
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

`*` needs the maintainer: 6A a choice, 6C the accounts.

**Can start today, all at once (up to nine implementors):** 2A, 2B, 2C, 3A, 3B, 3C, 6B, and 6C's preparation; 6A once its option is chosen.

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
| 5B | The agents plan's phase 2 | [Decision 8](#proposed-decisions) |
| Everything | 8 | It measures the result |

**Not at the same time, though neither waits for the other:**

| These two | Share |
|---|---|
| 4A and 5D | `crates/tessera-emit/src/render/` |
| 5A and 5B | `crates/tessera-cli/src/commands/fmt.rs`; 5B has it, 5A takes it last |
| A lint kept in 3C, and any part of 5 | Many Rust files. The lint merges before phase 5 starts, or after it ends |
| 8A (a rename), and any Rust pull request | Every Rust file |

**The shortest path to unblocking the agents plan:** 2A, then 5B. Everything else can go around them.

## Measures

Taken from the inventory, and again at the end. Each is a count a finding names; lines of code isn't one, since size isn't the problem.

| Measure | Now | Target |
|---|--:|--:|
| Pushes to `main` that fail the site check, of the last 10 | 4 | 0 |
| File reads outside `FileSystem` | 34 | 0, or each with a stated reason |
| Source files holding `data-ascribe-source` as a literal | 14 | 2 (one per language, generated) |
| TypeScript files declaring a JSON shape Rust writes, untested | 7 | 0 |
| Facts in several files with no test between them | 4 | 0 |
| Ways the commands load a project | 2, plus the language server's | 1, plus the language server's |
| Commands whose core is in the CLI crate | 2 | 0 |
| Functions returning an error as a string | 10 | 0 |
| Lines in library crates that print | 7 | 0 |
| Commands timed with no baseline | 2 | 0 |
| `diff` on 3,000 pages, nothing changed | 1.3 s | Set in phase 7 |
| Release binary | 10.5 MB | Set in phase 7 |
| Peak memory on 3,000 pages | Unmeasured | Recorded in phase 2 (`check` 221 MB, `build` 265 MB, `diff` 547 MB); stays within the baseline's margin |

## Out of scope

- New features, and changes to what the language allows.
- Changes to anything [decision 4](#proposed-decisions) calls a contract.
- A general lint pass, a pass over `.clone()` calls, or parallelism in `build`. [Decision 9](#proposed-decisions) covers the last two: nothing there misses a budget.
- Splitting large files for its own sake.
- Replacing a tool we depend on (the parser, Astro, the test runners), unless a measure shows one costs more than it gives. That would be its own plan.
- The plans in `project-docs/agents/` and `project-docs/editor-ui/` themselves. This plan may say they should change; it doesn't change them.

## Open questions

1. Does the agents plan wait for phase 5's command work ([decision 8](#proposed-decisions)), or do the two run side by side?
2. Is renaming the crates on the table, or is `tessera` staying?
3. How much time is this worth: a fixed budget, or until the measures are met?
4. Which of phase 6's three options for the canary wait do you prefer?

Answered by phase 1: the parser fork is small to maintain (76 added lines) and stays. Its only work is the "one parser" test in phase 5.
