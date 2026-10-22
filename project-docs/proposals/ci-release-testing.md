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

Not measured yet: how long each job takes, which job is on the critical path, and how often a change that passed CI needed a fix or revert afterwards. Without those three numbers, no step below can be sized.

## The problem

Pull request wall-clock time is the cost people feel. The parallel jobs mean the time is set by the slowest one, which is probably Windows clippy and test or the determinism test (its own comment calls it the longest test program), but that's a guess.

The release and nightly gates are weaker than their names suggest. A test failure can reach `next` before anyone sees it, and a release can be tagged from a commit the test suite has never run on.

## Proposal

Each step is cheaper than the one after it, and each stands on its own. Later steps need the data from step 0.

### Step 0: measure (two weeks)

- Pull the job durations for the last 20 pull-request runs and the last 20 `main` runs from the Actions API. Record, for each run, which job finished last.
- Log every fix, revert, or hotfix merged to `main`. For each, record the check that should have caught it, its platform, and whether it came from a pull request, a `main` push, or a nightly run.

Output: a table of time per job, and a table of post-merge fixes by platform. Both go into this document as a new section.

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
