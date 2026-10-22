# Proposal: keep the CI gate, make the nightly and release gates real

A proposal, not a plan. It says what would change and in what order, so the idea can be judged before anything is designed in detail. It answers the question of whether the cross-platform test matrix should move from every pull request to release testing.

## The goal

Shorten the time from a pull request to a merge, without letting more breakage reach users or `next`.

Two things are out of scope:

- **Replacing required CI with release testing.** The Claude Code team's [write-up on code review](https://claude.dev/blog/solving-code-review-at-anthropic/) keeps "tests on every platform" as a required check and adds review, the merge queue, flags, and rollback around it. Ascribe should do the same.
- **Dropping platform coverage in one step.** Windows catches the path, link, and case bugs that AGENTS.md calls platform-specific. It stays until the data says it catches nothing that would otherwise reach users.

## What exists

Checked on 22 October 2026 against the workflows in `.github/workflows/`.

| Trigger | What runs |
|---|---|
| Pull request | fmt, `cargo deny` and `cargo machete`. Clippy and the test suite on Ubuntu and Windows, plus the determinism test on both. JavaScript on Ubuntu. VS Code integration on Windows. Astro end-to-end on Linux x64 only. Site when docs or code change. Install when the install script changes. Outputs-unchanged only when labeled `optimization`. |
| Push to `main` | The same, with macOS in place of Windows for Rust and VS Code. |
| Nightly CI, 05:43 UTC | Everything, on all platforms. Astro end-to-end on all four targets. |
| Nightly canary, 04:17 UTC | Build, pack, and smoke on four platforms. Publishes to `next` if those pass. It does not run `cargo test` or the JavaScript test suite. |
| Release | Build, pack, and smoke on four platforms, then publish. The dry run does the same without publishing. |

Three facts follow from the table:

1. **The canary publishes before the full matrix runs on the same commit.** The canary runs at 04:17 UTC and the nightly CI at 05:43 UTC. A commit whose Windows tests fail, but whose binary smokes fine, reaches `next` before anyone sees the failure.
2. **Release testing doesn't run the test suite.** The release checks the tagged commit by building and smoking it. I found nothing in RELEASING.md or the release workflow that requires a passing CI run on that commit. I haven't read every line of `release.yml` to confirm the absence.
3. **The PR gate's extra OS is Windows only.** macOS already runs only on `main` and nightly. Dropping macOS from the PR gate is already done.

Step 0 measured the first two (see [Results of step 0](#results-of-step-0)). The third, how often a change needed a fix or revert afterwards, is only a lower bound from commit messages.

## The problem

Pull request wall-clock time is the cost people feel. The parallel jobs mean the time is set by the slowest one, which is probably Windows clippy and test or the determinism test (its own comment calls it the longest test program), but that's a guess.

The release and nightly gates are weaker than their names suggest. A test failure can reach `next` before anyone sees it, and a release can be tagged from a commit the test suite has never run on.

## Results of step 0

Collected on 22 October 2026, by reading the Actions API and first-parent `main` history. The proposal asked for two weeks; the data covers one day for runs and 90 days of history for fixes, and the sample is small. Read the numbers as rough.

**Limits:** the 20 pull-request runs span 05:35 to 17:49 UTC on one day, from about eight branches (one branch has four of them). The 20 `main` runs span one day too. Seventeen runs per event completed; the rest were cancelled and are excluded. `ci.yml` changed during the window: #837 removed a Chromium install step from the Linux Astro job and the site job, which shortens those jobs by about 15 to 25 seconds. Four sampled runs failed in that Chromium install step, which was an Azure apt mirror slowdown, not a code failure.

**Run wall-clock, median and 90th percentile:** pull requests 9.9 and 10.6 minutes; `main` 7.8 and 10.3 minutes.

**Critical path on pull requests** (the last job to finish, out of 17 completed runs), with median job durations:

| Job | Median (min) | Last to finish |
|---|---|---|
| clippy and test, Windows | 9.7 | 14 of 17 |
| Astro end-to-end, Linux x64 | 7.1 | 2 |
| VS Code integration, Linux | 5.0 | 0 |
| clippy and test, Linux | 4.8 | 0 |
| VS Code multi-project, Windows | 3.8 | 0 |
| determinism, Windows | 3.6 | 0 |
| site | 3.1 | 1 (a failed run) |
| JavaScript format, lint, typecheck, and test | 1.9 | 0 |

Inside the Windows clippy-and-test job, `cargo test --workspace` takes 6.6 minutes of its median 9.7.

On `main`, the Linux Astro end-to-end job is last to finish in 15 of 17 runs, with a median of 7.5 minutes. The macOS clippy-and-test job is next at 5.3 minutes.

**What this changes:**

- Windows clippy and test is the critical path on pull requests. Step 0's guess (Windows clippy and test or determinism) was half right: determinism is not on the path.
- If Windows were removed from pull requests, the critical path would move to the Linux Astro job at about 7 minutes. That figure comes from job medians, not from a run without Windows, so treat it as an estimate. Against a median of 9.9 minutes, the saving is about 3 minutes. Step 2 doesn't remove Windows from every change, so its saving depends on how often a change touches the selected paths.

**Post-merge fixes** (first-parent `main`, 612 commits in the history, which begins 9 September 2026). Classified by commit subject and body only; no CI logs or review threads were read. The count is a lower bound, because fixes whose messages don't use the search words were not enumerated:

| Class | Count | Examples |
|---|---|---|
| Windows | 2 | A retry around folder deletion in the VS Code tests, on Windows. A release smoke test fix whose title names Windows. Both are guesses. |
| macOS | 0 | |
| Linux or arm64 | 4 | The Chromium install change, which was infrastructure. An arm64 Astro e2e fix. Two others, one of them weak. |
| Platform-independent | 10 | Canary `next` logic, repo-map path tests, and an editor bug. |
| Unclear | 9 | Test races and flakes, and vague subjects. |

Two Windows fixes in 90 days is a weak case for either keeping or dropping Windows on every pull request. It does support Step 2's selection, since a change to path and link code is the one most likely to break there.

## Proposal

Each step is cheaper than the one after it, and each stands on its own. Step 0 is done and its results are above.

### Step 0: measure (done)

Pulled the job durations for recent pull-request and `main` runs, and logged fixes and reverts on `main`. Results are in [Results of step 0](#results-of-step-0). A standing log of post-merge fixes, by platform and by the check that should have caught each, is still not kept; a follow-up is needed to make that count exact.

### Step 1: make the nightly and release gates real (no coverage lost)

- **Canary.** Run the test suite before `next` moves: Linux and Windows `cargo test`, and the JavaScript tests. The canary already tolerates a late night, so a slower publish costs nothing that matters.
- **Release.** Require that the tagged commit already has a passing CI run, and fail the release otherwise. A tag pushed from a commit nobody tested is the case this closes.
- **Failure signal.** A failing nightly run should notify whoever owns the release, not only appear in the Actions tab.

This step adds checks, so it can't reduce coverage. It's the first thing to do whatever the measurements show.

### Step 2: select the Windows jobs, don't drop them

Keep Linux on every pull request, complete. Run the Windows jobs only when the change touches code the platform can break: the filesystem, path, link, and case handling in `ascribe-resolve`, the workflow files, and the toolchain pins. The `changes` job in `ci.yml` already selects `site` this way, so this extends an existing mechanism.

Test impact analysis, which the Claude Code team's [scaling-CI article](https://claude.com/resources/articles/agentic-coding-is-straining-ci-heres-how-we-scaled-test-impact-analysis-at-anthropic) describes, would go further: run only the tests a change can reach. It's a larger tool and waits for the data from step 0.

### Step 3: merge queue on `main`

Once Step 1 holds, put `main` behind a merge queue that runs the full required set on the queued change. Two passing pull requests can't then combine into a broken `main`. The queue makes the expensive jobs run per merge instead of per push, which is what the pull-request gate needs.

Whether the repository's plan supports a merge queue is unchecked.

### Step 4: decide on the pull-request Windows job (conditional)

Remove the Windows job from pull requests outside the paths Step 2 selects only if both hold:

- Step 0 shows no post-merge Windows fix in the measurement window that the nightly run caught late, and
- Step 1's nightly gate has run for at least a month without a Windows-only failure reaching `next`.

Otherwise it stays.

### Not proposed

- Release-only testing, with no pre-merge test matrix.
- Dropping Windows or macOS entirely.
- An AI reviewer on this repository. The Claude Code team's write-up is a reason to consider one later, as a separate experiment with its own proposal. It doesn't replace any CI step.

## Decisions needed

1. **Accept a Windows-only failure landing on `main`** for paths outside Step 2's selection, as long as the nightly run catches it and the fix follows within a day.
2. **Enable the merge queue** if the plan supports it (Step 3).
3. **Name the owner** of branch protection and of the `all checks` gate in `ci.yml`, which is the only required check.
4. **Approve the two-week measurement** in Step 0.

## Measures

- Median time from pull request open to all-checks green, before and after Step 2.
- Total runner minutes per merged pull request.
- Post-merge fixes and reverts, by platform, per month. This is the guardrail: it must not rise.
- Nightly runs that fail, and how long each stays red.

## Risks

- **A Windows-only break reaches `main`** for a path outside Step 2's selection. Mitigation: the nightly run and its failure signal from Step 1. The guardrail measure would show a rise.
- **A slower canary delays `next`.** Acceptable, since `next` already moves at the canary's pace.
- **Merge-queue batch failures add waiting time** for the change that caused them. Mitigation: keep batches small, and measure the wait.

## Next

Step 0 needs no decision and no change to any workflow. It can start on request, and its output decides whether Steps 2 to 4 are worth writing up as a plan.
