# Phase 3: The canary

Part of [Docs](README.md). Needs no other phase, and can run at the same time as phases 1 and 6. Workflows and the release scripts.

## Goal

Every night, the npm packages are built from `main` and published under the `next` tag, with nobody approving it. A site, ours first, can then install tomorrow's Ascribe the way it installs today's: from npm. Releases don't change.

## Context

- `RELEASING.md`: how a release works, that each publish is approved, that a release "moves no dist-tags", and the trusted-publishing setup. This phase adds a second, unattended way to publish, so read it whole.
- `scripts/release/` (`manifests.ts`, `version.ts`, `pack.ts`, `publish.ts`, their tests): versions are in lock step across every package; `checkVersion` wants a changelog section for the version; a version containing `-` is published under `next`.
- `.github/workflows/release.yml`: the build, pack, smoke, and publish jobs, and the `release` environment that requires approval.
- npm, to check against current documentation: trusted publishing from GitHub Actions and whether a package can trust more than one workflow; provenance for prereleases; how dist-tags move; unpublishing and deprecating.
- [Decision 5](README.md#decisions).

## Design

### What's published

The same npm packages a release publishes (the platform packages, `@ascribed/cli`, `@ascribed/elements`, `@ascribed/astro`, `@ascribed/review`), at one version, under `next`. Not the VS Code extension.

### The version

`<next version>-next.<n>`: the version after the latest release (a patch bump unless the unreleased changelog says otherwise), and a number that only goes up (the workflow's run number). Numbers only after `next`: a commit hash can make an invalid version. The commit is recorded where it can be read: in each package's `gitHead`, and in `ascribe --version`'s output for a canary.

The version is stamped in the workflow's working tree and never committed.

### The workflow

`canary.yml`, on a nightly schedule and by hand:

1. Skip, successfully, when `main` hasn't changed in `crates/`, `packages/`, or `scripts/release/` since the last canary.
2. Stamp the version (`version.ts`, with a flag that accepts the changelog's unreleased section in place of a section for the version).
3. Build, pack, and smoke-test, by the same jobs a release uses. Make them reusable; don't copy them.
4. Publish to npm under `next`, with provenance, from its own environment: limited to `main`, with no reviewers.
5. Only after the smoke test and the publish succeed, tell the site to rebuild (phase 5 adds the hook; this phase leaves the step ready and skipped when no hook is set).

A canary that fails publishes nothing and moves no tag. `next` always points at the newest canary that passed.

### What a user is told

`RELEASING.md` gains the canary: what it is, that it's unattended, and how to deprecate a bad one. The getting-started page says, in one line, that `@ascribed/cli@next` and `@ascribed/astro@next` install the nightly build, with no promise of stability.

## Tasks

1. The release scripts: stamping a canary version without a changelog section or a commit, with tests.
2. `release.yml`'s jobs made reusable, and `canary.yml`.
3. The npm side: whatever trusted publishing needs for a second workflow. This is the owner's to set up; say in the pull request exactly what to click, and don't merge until it's done.
4. One canary published by hand, and installed in a clean directory on Linux and macOS: `npx ascribe --version` reports the canary and its commit.
5. `RELEASING.md`, the getting-started line, `CHANGELOG.md`.

## Out of scope

A canary of the VS Code extension; a canary per push; publishing from pull requests; changing how releases work.

## Acceptance criteria

- A night with changes on `main` produces a new version under `next`; a night without produces none, and the run is green.
- `latest` never moves because of a canary.
- A failed build or smoke test publishes nothing.
- The release workflow still works as before; its approval is unchanged.

## Verify

```sh
pnpm typecheck && pnpm lint && pnpm format:check && pnpm test
node scripts/release/pack.ts --help
```

## Commits

1. "Stamp a canary version"
2. "Share the release's build, pack, and smoke jobs"
3. "Publish a nightly canary to npm"
