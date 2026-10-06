# Phase 6: Infrastructure

Part of [Optimization](README.md). Needs phase 1 (done). Workflows and configuration.

**Runs with:** everything. No part touches product code. Parts A, B, and C are independent of each other. Part A waits on a decision from the maintainer ([open question 4](README.md#open-questions)). Part C is the maintainer's to do; an agent can only prepare it.

## Goal

`main` stays green when nothing is wrong, the dependency count can't drift up unnoticed, and what's left over from earlier setups is gone.

## Context

- The [inventory](inventory.md): finding 1, "Third-party code", and "Outside the repository".
- `.github/workflows/site-npm.yml`: "Site from npm", which builds `docs/` and `site/` with the published canary (`@ascribed/cli@next`) on each push to `main` and after each canary. `canary.yml`: the nightly publish. `drift.yml` and `review.yml`: report-only jobs on pull requests that also install the canary.
- `project-docs/docs/README.md`, decisions 1, 4, and 5: the docs are customer #1 and production builds from npm. Part A mustn't undo that.
- `.github/dependabot.yml`; `Cargo.lock` (158 third-party packages); `pnpm-lock.yaml`. The project's license is MPL-2.0; `crates/comrak-tessera` is BSD-2-Clause; `scripts/release/notices.ts` writes third-party notices for releases.
- Known duplicates: upstream `comrak` beside the fork (phase 5, part D, removes it); `serde_yaml_ng` and `yaml-rust2` (on purpose: one reads values, one gives positions); two versions of `phf`.
- `RELEASING.md`: trusted publishing, the `release` environment, and tags.

## Design

### Part A: `main` stays green between canaries

One pull request, once the maintainer has chosen.

**The problem:** a merge that documents something the canary doesn't have yet fails "Site from npm" until the next nightly: 4 of the last 10 pushes. The failure means "wait", and a red check that means "wait" teaches people to ignore red.

**The options:**

1. **Say "waiting", don't fail.** When the site check fails, the workflow compares the canary's commit (published in its package metadata, or add it) with the merge commit. If the canary is older, the job ends neutral with a summary line, "waiting for a canary that includes this commit", and the run after the next canary is the real check. A failure with a canary that already includes the commit stays red.
2. **Publish a canary on demand.** The same test, and when the canary is older, start `canary.yml`, whose completion already reruns the site check. More publishes to npm, and `main` goes green within about ten minutes.
3. **Build from the checkout.** The check on push uses the repository's own binary. It never waits, and it no longer tests what a user installs, which is the point of the workflow.

**Recommended:** option 1, with option 2 as a manual button the summary links to. It keeps the check honest, publishes nothing extra, and makes red mean broken. Option 3 gives up what the docs plan built the workflow for.

Whichever is chosen, apply the same rule to `drift.yml` and `review.yml`, which fail on pull requests for the same reason, and update `CONTRIBUTING.md`'s section on the docs site.

### Part B: dependencies, audited

One pull request per tool; they're independent.

- **`cargo deny`,** with a `deny.toml`: advisories; licenses (allow what's in the tree today, listed by name, and check that list against `scripts/release/notices.ts`); sources (crates.io only); and duplicate versions set to warn, with the known duplicates listed and explained. It runs in `rust.yml`'s lint job.
- **`cargo-machete`** for unused Rust dependencies, in the same job. Remove what it finds; where it's wrong (a dependency used only through a macro or a feature), ignore it by name with a comment.
- **`knip`** for unused files, exports, and dependencies in the JS workspace, in `js.yml`'s main job. Remove what it finds, except public exports of the four published packages, which are contracts: configure those as entry points.
- Each tool is pinned to a version, as the actions are.

Removals here change no output; show it with the comparison where a removed dependency was linked into the binary.

### Part C: leftovers

For the maintainer; an agent prepares a checklist in the pull request or an issue and changes nothing itself.

| Leftover | Action | Who |
|---|---|---|
| `NPM_TOKEN` in the `release` environment | `canary.yml` still reads it, in a step that publishes a package's first version and already stops with an error once every package is on npm. Remove that step and the variable in a pull request, and say in `RELEASING.md` how a new package gets its first publish. Then delete the secret | Agent: the pull request. Maintainer: the secret |
| `KyleBlankRollins/ascribe-review-scratch` | Delete or archive; `ascribed-dev/review-fixture` replaced it | Maintainer |
| `@ascribed/review`'s `latest` tag is `0.1.2-next.2` | Fixed by the next release, which moves `latest`. Until then, note it in `RELEASING.md`'s checklist so the release checks every package's `latest` | Agent: the note. Maintainer: the release |
| The `copilot` environment, and `AZURE_CLIENT_ID` / `AZURE_TENANT_ID` | Say what each is for in `RELEASING.md`, or remove them | Maintainer says; agent writes |

Also in this part, for an agent: `project-docs/optimization/outside.md`, a page listing everything outside the repository (the inventory's table, kept current), with what uses each thing and who can change it. Link it from `RELEASING.md`.

## Tasks

1. Part A, after the decision: the workflow change, the same for Drift and Review, the documentation.
2. Part B: three pull requests.
3. Part C: `canary.yml`'s token step removed; the `RELEASING.md` notes; `outside.md`; the checklist for the maintainer.

## Out of scope

- Changing how releases or canaries are built.
- Merging or renaming workflows. The inventory found 11 files and 1,302 lines, with CI at 5.3 minutes; there's nothing to gain.
- Upgrading dependencies. Dependabot does that.
- Deleting anything outside the repository.

## Acceptance criteria

- A merge to `main` that needs a newer canary doesn't leave a failed run, and a merge that breaks the site with a current canary still does. Show both, on a fork or with a dry run described in the pull request.
- `cargo deny check`, `cargo machete`, and `knip` pass in CI, and each fails when a test case is introduced (an unused dependency, an unlisted license).
- No workflow mentions `NPM_TOKEN`.
- `outside.md` lists every repository, environment, secret name, package, and tag the inventory found. It holds no secret values.

## Stop and report if

- `cargo deny` reports an advisory or a license that isn't already known. Don't suppress it; list it.
- `knip` wants to remove something a published package exports.
- Part A's test for "the canary is older" can't be made reliable from the package metadata.

## Verify

```sh
cargo deny check && cargo machete
pnpm exec knip
cargo test --workspace --locked
pnpm format:check && pnpm lint && pnpm typecheck && pnpm test
```

## Commits

1. "Don't fail the site check while it waits for a canary"
2. "Audit Rust dependencies for advisories, licenses, and duplicates"
3. "Remove unused Rust dependencies, and check for them"
4. "Remove unused JS code and dependencies, and check for them"
5. "Stop reading the npm token trusted publishing replaced"
6. "List what lives outside the repository"
