# Phase 5 notes: supply chain and release

Hardening notes from [phase 5](../phase-5-supply-chain.md), reviewed on 27 October 2026 by one agent and checked by a second, adversarial one, with the numbers re-run. **No exploitable finding** (README decision 2). Each note says where it is, what it costs, one recommendation, and a size. The logs the review ran (`cargo deny`, `pnpm audit`, `npm audit` on `site/`, the install-script and dependency counts) were kept with the review's working files, not committed.

## H1: the daily link report restores an artifact any pull request run could have uploaded

- **Where:** `.github/workflows/report.yml`, the step "Download what the last run kept": it lists `actions/artifacts?name=ascribe-links` for the whole repository and takes the newest whose `workflow_run.head_branch` is `main`, then `gh run download` into `docs/.ascribe/`. The listing is cross-workflow and cross-event (verified: a `CI` `pull_request` run's artifact appears under its head branch name), and the filter reads neither the run's workflow, nor its event, nor `head_repository_id`.
- **What it costs:** a `pull_request` run from a fork whose branch is named `main`, uploading an artifact of that name with link-cache entries the binary's own format (`crates/ascribe-check/src/links/cache.rs` checks each entry's version, address, and `lychee.toml` hash, all public), makes the next report reuse the planted answers, so a broken external link goes unreported for up to the cache's limit (7 to 14 days), and the report re-uploads the planted state as its own. It reaches none of decision 2's kinds: the download extracts through `gh`'s safe paths into `docs/.ascribe/` only, nothing read from the files is a path or a command, the token is `issues: write` and `actions: read`, and `--recheck` undoes it. A first-time contributor's fork run needs a maintainer's approval first, by GitHub's default (the repository's setting wasn't readable from the session). No fork has opened a pull request yet.
- **Recommendation:** pick the run from the Report workflow's own successful runs on `main` (`actions/workflows/report.yml/runs?branch=main&status=success`), which a pull request can't produce and which also excludes a `workflow_dispatch` of Report from someone's branch (today's listing has one); or at least require `.workflow_run.head_repository_id == .workflow_run.repository_id` in the filter, both of which the artifact object carries.
- **Size:** small.

## H2: nothing audits the JavaScript dependencies

- **Where:** `rust.yml` runs `cargo deny`; `js.yml` has no counterpart. `pnpm audit` in the root on 27 October: 11 advisories (4 high, 6 moderate, 1 low; 13 on 29 October with the lockfile unchanged, `qs` and `deepmerge` added by the database, both under development trees) over 1,196 packages, all under development or optional trees (`@stoplight/elements` in `examples/astro-site`, `mocha`, `vite`, `jsdom`, `@vscode/vsce`); `pnpm audit --prod`: 2, both through `astro` as a peer; `npm audit` in `site/`: 0. The published packages' own production closures hold no advisory: `@ascribed/cli` 0 dependencies, `@ascribed/elements` 0, `@ascribed/review` 1, `@ascribed/astro` 3, the extension's bundle 10.
- **What it costs:** an advisory in a package that ships, or in a tool that runs with a token (H3, H4), is found by whoever runs `pnpm audit` by hand, or by nobody.
- **Recommendation:** a step beside `cargo deny check advisories`, with `continue-on-error: true` as it has: `pnpm audit --prod --audit-level moderate` in the root and `npm audit --package-lock-only --omit=dev` in `site/`; or OSV-Scanner over the four lockfiles. Report, don't fail, as the Rust step does; phase 6 decides whether either becomes required.
- **Size:** small.

## H3: `cargo deny check advisories` fails on every run, and the one advisory has an issue but no `ignore` entry

- **Where:** `Cargo.lock` and `fuzz/Cargo.lock`, `unsafe-libyaml` 0.2.11 through `serde_yaml_ng`, which reads frontmatter and the OpenAPI specs: RUSTSEC-2024-0449, unmaintained. `rust.yml` runs the step with `continue-on-error: true`; `.cargo/deny.toml` asks for an issue and an `ignore` entry when one appears. Issue #1157 exists; the `ignore` entry doesn't, so the step is red on every run.
- **What it costs:** a second advisory, a real vulnerability, looks the same as the one everyone passes. The crate itself is a C-to-Rust translation of libyaml, archived, reading untrusted YAML; a memory-safety bug found in it won't be fixed upstream.
- **Recommendation:** the `ignore` entry naming #1157 now, so a new advisory shows; the `openapi` and `frontmatter` fuzz targets phase 1 added are the right test of this parser meanwhile; moving to a maintained YAML parser is its own decision.
- **Size:** small (the entry); medium (the parser).

## H4: the Marketplace job's `@vscode/vsce` pulls `@azure/msal-node` with a high advisory

- **Where:** `pnpm-lock.yaml`: `@vscode/vsce` 4.0.0 → `@azure/identity` 4.13.3 → `@azure/msal-node` 6.0.1 (GHSA-xcx6-jx8p-x3h5, fixed in 7.1.0); `release.yml`'s `marketplace` job runs it after `azure/login`.
- **What it costs:** the advisory is in the authorization-code flow, which the job's federated sign-in doesn't use, so nothing exploitable; the job that holds the Entra sign-in runs a tree with a known-bad package and nothing says so.
- **Recommendation:** a `pnpm.overrides` entry for `@azure/msal-node` at `>=7.1.0`, or `@vscode/vsce` at a version that takes it.
- **Size:** small.

## H5: tools installed at a floating version in jobs that hold a token

- **Where:** `release.yml` and `canary.yml` install `npm@^11.x` globally before publishing; `site.yml` runs `npx --yes netlify-cli@27.11.0` (pinned, but its tree is resolved fresh each run, with no lockfile, holding `NETLIFY_AUTH_TOKEN`); `report.yml`, `review.yml`, and `drift.yml` install `@ascribed/cli`, `@ascribed/cli@next`, and `afdocs@0.22.2` globally, the report job with `issues: write`.
- **What it costs:** a compromised transitive package of any of these runs with the job's token. For `npm` it's npm's own package and the publishing token is short-lived; the Netlify token is long-lived and personal.
- **Recommendation:** pin `npm` to an exact version in the publishing jobs; put `netlify-cli` in `site/`'s `devDependencies` so `npm ci` pins its tree; install `afdocs` from a small folder with its own lockfile rather than `--global`.
- **Size:** small.

## H6: every checkout keeps the job's token in `.git/config`, including jobs with `contents: write`

- **Where:** no workflow sets `persist-credentials: false`. `templates.yml` (`contents: write`, `pull-requests: write`) then runs `cargo build`, `pnpm install`, the tests, and `scripts/templates/update.ts` over content fetched from GitLab; `release.yml`'s `github` job (`contents: write`) runs `scripts/release/version.ts` and `gh release`.
- **What it costs:** code that runs in those jobs (a crate's build script, a dev dependency, a test) can push any unprotected branch and open pull requests without reading a secret. `main` requires `all checks` and tags have a ruleset, so neither is pushable directly.
- **Recommendation:** `persist-credentials: false` on every checkout, and the push step's token through `git -c http.extraheader`, as `board.yml` does.
- **Size:** small.

## H7: Approve protects nothing branch protection asks for

- **Where:** `approve.yml` (`workflow_dispatch`, `pull-requests: write`); `project-docs/outside.md`: `main` has one required check and no required review, and admins can bypass; confirmed from the API on 29 October (no `required_pull_request_reviews`, `enforce_admins` false, and the merge-queue ruleset adds no review). The `board` and `review` skills have an agent dispatch Approve and then merge.
- **What it costs:** the workflow's own checks hold (the number is digits, the `sha` must equal the live head, both quoted), but no review is required, so anyone with write access, and every agent holding the maintainer's token, can merge an open pull request with `all checks` green without it. The hold against a prompt-injected reviewing agent is the skill's procedure alone.
- **Recommendation:** require one approving review on `main`, with stale approvals dismissed, so a merge needs the approval Approve gives for the reviewed head. A setting, not a workflow change.
- **Size:** small.

## H8: `templates.yml` uses the commit id GitLab returns in a path and a URL unchecked

- **Where:** `scripts/templates/update.ts` takes a tag's `commit.id` as any string and writes it into `scripts/templates/upstream.toml`; `scripts/templates/convert.ts` joins it into a cache path and a raw-file URL. Only the tag name is held to `^v\d+\.\d+\.\d+$`.
- **What it costs:** a compromised GitLab answer, with a `commit` of `../../..`, writes a template file at a chosen folder of the runner's checkout, in a job with `contents: write` (H6). Nothing executes what's written, and the written files aren't among the paths the job commits.
- **Recommendation:** accept `commit` only when it matches `^[0-9a-f]{40}$`, where the pin is moved and where it's read.
- **Size:** small.

## H9: the fuzz crate's lockfile drifts from the workspace's

- **Where:** `fuzz/Cargo.lock` against `Cargo.lock`, as committed with the three new targets: of 105 crates in both, 30 are at another version than the binary ships (`toml` 1.1.8 against 1.1.6 and its family, `syn`, the `icu_*` crates, `zerocopy`, `libc`, `sha2`, `smallvec`). Decision #1020 keeps the lockfiles apart so no nightly reaches the workspace, which doesn't need their shared crates to differ; Dependabot doesn't watch `fuzz/`.
- **What it costs:** the nightly fuzzing tests different parser and TOML crates than `ascribe` ships: a crash in the shipped version is missed, and one found is in a version nothing ships.
- **Recommendation:** a check in `scripts/check.ts` or a test that fails when a crate in both lockfiles has a different version, fixed with `cargo update -p <crate> --precise <version>` in `fuzz/`.
- **Size:** small.

## H10: `outside.md` lists eight third-party actions; the workflows use nine

- **Where:** `project-docs/outside.md`'s list is missing `taiki-e/install-action`, which `rust.yml` uses to download the `cargo-deny` and `cargo-machete` binaries. All nine are pinned to a commit.
- **What it costs:** the page is the inventory of what the repository trusts outside itself, and a reader doesn't learn that a third action downloads and runs prebuilt binaries.
- **Recommendation:** generate the list from the workflows, as `scripts/surface/inventory.ts` generates `surface.md`, or a test that compares the two ("one home per fact").
- **Size:** small.

## H11: the release's binaries and `SHA256SUMS` prove integrity in transit, not origin

- **Where:** `site/public/install.sh` and `scripts/release/formula.ts` download `SHA256SUMS` from the same release address as the archives and compare; `release.yml`'s `github` job uploads both with `contents: write`. npm's packages have provenance; the GitHub release and the Homebrew tap have nothing equivalent. The README's "Later" item.
- **What it costs:** anyone who can replace a release's assets (the maintainer's account, a `contents: write` token, GitHub itself) replaces `SHA256SUMS` with them, and the installer and the formula follow.
- **Recommendation:** `actions/attest-build-provenance` on the release's files in the `github` job, which already has the OIDC identity npm uses, with `install.sh` verifying through `gh attestation verify` when `gh` is present; or a `cosign sign-blob` of `SHA256SUMS`.
- **Size:** medium.

## H12: `main` reaches npm `next`, the Marketplace's users' tooling, and the production site with no person in the loop

- **Where:** `canary.yml` publishes nightly from `main` with an environment that has no reviewers; `site/netlify.toml` and `site/scripts/follow-next.mjs` build production from whatever `@ascribed/astro@next` names; `report.yml`, `review.yml`, and `drift.yml` install `@ascribed/cli@next`.
- **What it costs:** a pull request merged with `all checks` green, by an agent or a person, is on npm under `next` and on the site by the next morning. The pipeline does what RELEASING.md says; the hold is the merge, which is H7.
- **Recommendation:** H7's review requirement; nothing in the canary itself.
- **Size:** small.

## H13: the tap job clones with the token in the remote URL

- **Where:** `release.yml`'s `homebrew` job: `git clone "https://x-access-token:$GH_TOKEN@github.com/$TAP.git"`.
- **What it costs:** the token is in the clone's `.git/config` for the job's life and in any error `git` prints with the URL; Actions masks it in the log, not on disk. The job runs only this repository's scripts after it, on a runner that's discarded.
- **Recommendation:** the `http.extraheader` form `board.yml` uses, with the token only in the push's environment.
- **Size:** small.

## Checked, nothing found

- **Install scripts.** None of Ascribe's packages has one. In the workspace's store three dependencies declare one (`esbuild`, `@vscode/vsce-sign`, `@scarf/scarf`), all set to `false` in `pnpm-workspace.yaml`'s `allowBuilds`, and pnpm 12 runs no other without a listing. `site/`'s lockfile has one, `fsevents`, optional and macOS only. `@ascribed/cli` finds its binary only through `require.resolve` of its platform package, never the `PATH` or the project.
- **The extension's bundle.** `.vscodeignore` is an allowlist; `scripts/release/pack.ts` packs with `vsce package --no-dependencies` and refuses a package with other than exactly one binary under `bin/`, a skill that differs from `@ascribed/cli`'s, or no webview bundle. The binary is built per platform in `release-build.yml` with no build cache, `--locked`, Linux through a pinned `cargo-zigbuild` and a hash-pinned `ziglang` wheel, then checked on glibc 2.28 in a digest-pinned image.
- **`cargo deny`.** Bans, licences, and sources pass; `[sources]` allows crates.io only, and `Cargo.lock` has no git or outside-workspace source. Advisories are H3. Three `phf` skips in `deny.toml` are stale (upstream resolves `phf` 0.14 now).
- **`site-npm.yml`.** `workflow_run` on Canary, filtered to `main`, checks out the default branch's head with `contents: read`, downloads no artifact, and installs from npm. Canary runs only from `main`, under an environment that admits only `main`. A fork can't start it.
- **What a fork's pull request gets.** `ci.yml`, `review.yml`, and `drift.yml` run the pull request's own workflow files and code (`cargo test`, `pnpm test`, `ascribe observe` on `examples/tested/code`, the canary script, `brew install` from the checkout's formula script) with `contents: read` and no secret: `secrets: inherit` on the `site` job gives a fork nothing, and the deploy steps are gated on the head repository being this one. No job uses `pull_request_target`.
- **Every `${{ }}` in a `run:` step.** Inventoried across all sixteen workflows: event names and SHAs, matrix and input literals, step outputs, and tokens, each through `env` and quoted; the inputs of `approve.yml`, `board.yml`, and `templates.yml` are checked (digits, the live head, a tag of the upstream's). The two fork-controlled values that reach a `run:` (the pull request's body, read into `--accept` patterns for the outputs comparison, and the checkout itself) do so under the read-only token.
- **Scheduled jobs.** `report.yml` is H1; its `issues: write` is used only to create or edit the "Docs report" issue with the binary's report text. `canary.yml`'s one outside input, `npm view`'s `gitHead`, is held to forty hex digits. `corpora.yml` fetches three documentation repositories at pinned commits with `contents: read`. `fuzz.yml`'s issue body is a crash's signature, its input as base64, and the log. `templates.yml` is H6 and H8.
- **Caches.** `Swatinem/rust-cache`, `setup-node`'s pnpm cache keyed by the lockfile's hash, a fixed-key `vscode-test` cache, and the fuzz corpus. GitHub's branch scoping holds: a pull request's cache lives under its merge ref and is restorable only there; `main`, a tag, and a schedule restore from their own ref or `main`. The release's binaries use no cache, and pnpm checks each linked file against the lockfile's integrity hash.
- **`workflow_dispatch`.** Anyone with write access can dispatch, which `outside.md` says is the maintainer, and so every agent with the maintainer's token; each input is validated or quoted. H7 is the note.
- **npm trusted publishing.** `id-token: write`, no `registry-url` on `setup-node` so no token line, `--provenance` only under Actions, and `checkRepository()` refuses a manifest whose repository isn't this one. No workflow reads `NPM_TOKEN`.
- **The Marketplace.** No token: `azure/login` trades the job's OIDC token for a managed identity whose federated credential trusts only this repository's `release` environment, which admits only `v*` tags, which only an admin can create. H4 is about the tool.
- **The canary.** From `main`'s head on the schedule, stamped and never committed, built by `release-build.yml` with the run's SHA, tested, published under `next-pending` and moved to `next` only when every package shows. A pull request influences it only by being merged (H12).
- **`release.yml` from a tag.** The `version` job requires a passing `all checks` on the commit, and `scripts/release/version.ts --check --tag` holds the tag and every manifest to one version.

## Re-checked on 29 October 2026, against `main` at `85c4c85c`

All thirteen notes hold. Of this phase's files, only five workflows, `Cargo.lock` (three workspace crates, no new external dependency), `fuzz/Cargo.lock` (this branch's targets), and `outside.md` (two path renames) moved; `pnpm-lock.yaml`, `deny.toml`, `report.yml`, `release.yml`, `templates.yml`, the release scripts, and `install.sh` are byte-identical. `cargo deny` still fails on RUSTSEC-2024-0449 alone, with #1157 open and no `ignore` entry; the fuzz lockfile's drift is 30 of 105 shared crates; H1's planted-cache reproduction behaves the same with the moved cache code (`crates/ascribe-report/src/links/`), and `report` is outside the git guard. No issue or pull request addresses a note beyond #1157; the four open Dependabot pull requests touch none of the packages named. Every "nothing found" claim holds. Added to the surface, each looked at: `ci.yml`'s `record` job (the pull request's body reaches `scripts/ci/change-record.ts` as a string compared line by line, one `git diff` with a fixed base, read-only token); `rust.yml`'s `agents sync --check` (a literal `--command`, nothing written) and `observe --yes` (the same exposure as before); `site.yml`'s `check --base` and the git guard (new `git` starts on a fork's checkout, with decision 76's options); and `pnpm bless`, which no workflow runs.
