# Phase 5: Supply chain and release

Part of [Security review](README.md). Needs phase 0.

## Goal

Find how someone could get code into what Ascribe ships, or a secret out of its CI.

## Questions

### Dependencies

- `cargo deny` covers the Rust side. Nothing audits the JavaScript side: what would (`pnpm audit`, OSV-Scanner), and should it fail CI or report?
- How many dependencies each published package pulls in, and which run code at install: none of Ascribe's packages has an install script; do their dependencies?
- The VS Code extension's bundled dependencies and the binary inside it.

### CI

- `report.yml` and `site-npm.yml`: triggered by a schedule and by `workflow_run`. Does either download an artifact or check out code from a run a fork could start, and then run it with secrets or a write token?
- Every workflow that runs on `pull_request`: what secrets and token permissions a fork's pull request gets, and whether any job runs code from the pull request with more.
- Caches: can a pull request poison a cache that a release or a push to `main` restores?
- The board, approve, and other workflows that act on the repository: who can trigger them, with what input, and whether an input reaches a shell unquoted (`${{ github.event.* }}` in a `run:` step).

### Releases

- `release.yml` and `canary.yml`: npm's trusted publishing and provenance are in place. The VS Code Marketplace token: where it lives and who can use it. The GitHub release's binaries: checksums published, and whether to sign them (the README's "Later").
- The canary: what it publishes, from which commit, and whether a pull request can influence it.

## Method

Read each workflow with the questions above; for each `${{ }}` in a `run:` step, check where its value comes from. Run the candidate JavaScript audit tool once and record what it finds.

## Output

Draft advisories for exploitable findings; `notes/phase-5.md` for hardening.
