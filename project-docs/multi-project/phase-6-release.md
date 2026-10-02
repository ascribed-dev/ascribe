# Phase 6: Release 0.1.1

Part of [multi-project workspaces](README.md). Requires phases 1-5 merged to `main`. Read the "Releasing" steps 1-9 in [RELEASING.md](../../RELEASING.md) first; it is the source of truth for the steps below.

## Goal

0.1.1 is published to npm (all seven packages), the VS Code Marketplace (four targets), and GitHub, and the extension works in a workspace with several projects.

## Important: ask before outward-facing steps

Tagging, pushing a tag, approving the `release` environment, and publishing are hard to undo. The maintainer approves the `release` environment deployment (a required reviewer is set), so an agent can prepare and dry-run, but must stop and ask before pushing the tag.

## Context

- Releases are lock-step: one version for every npm package, the Cargo workspace, and the extension. `scripts/release/version.ts` bumps and checks it.
- `.github/workflows/release.yml` runs on tag `v*`. A manual run (`workflow_dispatch`) with the `publish` input unchecked (the default) builds, packs, and smoke-tests on all four targets and runs the publish steps with `--dry-run`.
- npm publishing uses trusted publishing (OIDC), with no token, for all seven packages. This is the first release to publish that way; the `NPM_TOKEN` secret in the `release` environment is still present only as a fallback and is no longer used by the workflow.
- The Marketplace publishes through the Entra managed identity `ascribe-vscode-publisher`; no PAT.
- Tags are protected by an environment rule for `v*`; branch protection requires ten checks on `main`.

## Tasks

1. **Prepare.** From an up-to-date `main`, on a branch: bump to 0.1.1 with `node scripts/release/version.ts 0.1.1` (RELEASING.md step 2), update the extension's `minServerVersion` only if the server's contract changed in a way the extension depends on (it does: the extension now relies on the server not guessing, so set it to `0.1.1` and say so in the PR). Add the `0.1.1` heading and entries to the root `CHANGELOG.md`; `node scripts/release/version.ts --check` must pass.
2. **Open a PR**, get the ten required checks green, and ask the maintainer to merge it.
3. **Dry run** the release workflow from `main` (Actions → Release → Run workflow with **publish** unchecked, or `gh workflow run release.yml --ref main`; RELEASING.md step 5). All build, pack, and smoke jobs must pass; publish jobs are skipped.
4. **Ask the maintainer to confirm** before tagging. Then create an annotated tag `v0.1.1` on the merge commit and push it. Never move or delete a pushed release tag.
5. **Watch the run.** The maintainer approves the `release` environment. Publishing to npm must succeed through trusted publishing. If any package fails with 404 or an auth error, stop and report the exact message, and consult the failure table in `RELEASING.md`; don't try tokens.
6. **Verify**, as RELEASING.md steps 7-9 do. The GitHub release is created as a draft; the maintainer publishes it (step 9):
   - `npm view @ascribed/cli version` and the other six packages show 0.1.1; `npm audit signatures` verifies provenance.
   - In a clean directory, `npm install @ascribed/cli` and `ascribe --version` prints `ascribe 0.1.1`; `ascribe check` on a copy of `examples/quill` is clean.
   - The Marketplace shows 0.1.1 for each target.
   - The GitHub release has the archives with their notices.
   - Install the extension from the Marketplace, open the repository root, and confirm that `examples/quill` and `examples/astro-site` each get diagnostics and a preview. (A human check; the agent can only check the installed files and run the bundled binary.)
7. **Clean up the token.** After a successful trusted-publishing release, tell the maintainer they can delete the `NPM_TOKEN` secret from the `release` environment and set each package's publishing access to "Require two-factor authentication and disallow tokens". These are npm and GitHub settings changes the maintainer makes; don't attempt them.

## Acceptance criteria

- All seven npm packages, four Marketplace targets, and the GitHub release at 0.1.1.
- No `NPM_TOKEN` was used (check the publish job's log).
- The multi-project behaviour works in the installed extension.

## Do not

Force-push tags, rerun a half-published release without reading `RELEASING.md`'s failure table, or change repository, environment, or npm settings.
