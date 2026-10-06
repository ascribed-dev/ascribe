# Releasing Ascribe

A release publishes one version of everything: the `ascribe` binaries (on a GitHub release), eight npm packages, and four VS Code extension packages, one per platform. The [release workflow](.github/workflows/release.yml) builds, packs, tests, and publishes; a person starts it, approves each publishing step, and publishes the GitHub release at the end. Nothing in a release is published without that person.

The [nightly canary](#the-nightly-canary) is the one exception: the npm packages, built from `main` and published under the `next` tag every night, unattended. It never moves `latest`.

| What | Where | Published by |
|---|---|---|
| `@ascribed/cli-darwin-arm64`, `-linux-arm64`, `-linux-x64`, `-win32-x64` | npm | the `npm` job |
| `@ascribed/cli`, `@ascribed/elements`, `@ascribed/astro`, `@ascribed/review` | npm | the `npm` job |
| `Ascribe.ascribe-vscode`, for each of the four platforms | VS Code Marketplace | the `marketplace` job |
| `ascribe-<version>-<platform>.tar.gz` (`.zip` for Windows), the `.vsix` files, `SHA256SUMS` | GitHub release | the `github` job drafts it; you publish it |

## How a release runs

A person starts the workflow by hand: Actions → **Release** → Run workflow, picking the version tag under "Use workflow from". Nothing runs automatically.

```
version ─→ build (4 platforms) ─→ pack ─→ smoke (4 platforms)
                                              │   with `publish` checked, each of these
                                              ▼   waits for approval in the `release` environment
                                            npm ─→ marketplace ─→ github (draft release)
```

Without `publish`, it's a dry run: everything up to `smoke` runs, and the publish steps run with `--dry-run`. Nothing leaves the workflow.

### Who owns what

| Thing | Where it lives | How it's reached |
|---|---|---|
| Source and CI | GitHub organization `ascribed-dev`, repository `ascribe` (organization id 335742967, repository id 1393013016) | The `release` environment requires a reviewer and allows only `v*` tags. |
| npm packages | npm organization `ascribed` | Trusted publishing: each package trusts the `release` environment of this repository's `release.yml`, and the `canary` environment of its `canary.yml`. There is no token. |
| VS Code extension | Marketplace publisher `Ascribe`, created by a personal Microsoft account (the owner) | The managed identity below is a Contributor member. |
| The managed identity `ascribe-vscode-publisher` | Resource group `ascribe-release`, region East US, in the Azure tenant and subscription that account's free Azure sign-up created | A federated credential trusts the workflow (below). |
| An Azure DevOps organization | Connected to that tenant | It exists only to give the identity an Azure DevOps profile. It has no projects or pipelines. |

### The Marketplace login

The `marketplace` job publishes without a token. Each hop checks the one before it:

1. The job runs in the `release` environment with `id-token: write`, so GitHub issues it an OIDC token. This repository issues *immutable* subject claims, so the subject is `repo:ascribed-dev@335742967/ascribe@1393013016:environment:release` (check with `gh api repos/ascribed-dev/ascribe/actions/oidc/customization/sub`).
2. `azure/login` sends that token to Microsoft Entra. Entra looks for a federated credential on the managed identity whose issuer, subject, and audience (`api://AzureADTokenExchange`) match exactly. If one does, the job is signed in as the identity.
3. `vsce publish --azure-credential` asks Entra for a token for the Marketplace and publishes with it.
4. The Marketplace checks that the identity's *Azure DevOps profile id* is a member of the `Ascribe` publisher with the Contributor role. That id is neither the Azure resource id, the client id, nor the object id.

The only values the workflow holds are `AZURE_CLIENT_ID` and `AZURE_TENANT_ID`, as variables on the `release` environment. They aren't secrets, and nothing in the repository can sign in without a token GitHub issues to a run in that environment.

[What lives outside the repository](project-docs/optimization/outside.md) lists every account, environment, secret, and package this page relies on, with who can change each.

## Before the first release

Do these once. Each says how to check it.

### 1. Make the repository public

npm refuses provenance from a private repository ("Only public source repositories are supported when publishing with provenance"), and a private repository on a free plan can't require reviewers on an environment. A public repository also runs Actions on standard runners at no cost. A release run hasn't been timed yet; expect under an hour of runner time across its thirteen jobs, two of them on macOS runners, which a private repository bills at ten times the Linux rate.

Decide where the repository lives first. If it moves (for example, to `ascribed-dev`), update the URLs that name it, which npm checks against the repository that publishes:

```sh
git grep -l "ascribed-dev/ascribe"    # package.json files, Cargo.toml, READMEs, docs
```

**Check:** the repository's Settings page says Public, and `git grep` finds the old location nowhere.

### 2. Create the `release` environment

Settings → Environments → **New environment** → `release`:

- **Required reviewers:** yourself (and anyone else who may approve a release).
- **Deployment branches and tags:** Selected branches and tags → add the tag rule `v*`.

**Check:** the environment lists you as a required reviewer and allows only `v*` tags.

### 3. npm: trusted publishing

The `npm` job publishes without a token. npm trades the job's GitHub OIDC token for a short-lived publish token, and signs a provenance statement. Each of the eight packages has to trust this repository's workflow. Do it once per package, with your own npm login (`npm login`, with two-factor authentication). A token that bypasses two-factor authentication can't change this setting.

```sh
npm trust github @ascribed/cli --file release.yml --repo ascribed-dev/ascribe --env release --allow-publish --dry-run
```

Drop `--dry-run`, and repeat for `@ascribed/cli-darwin-arm64`, `@ascribed/cli-linux-arm64`, `@ascribed/cli-linux-x64`, `@ascribed/cli-win32-x64`, `@ascribed/elements`, `@ascribed/astro`, and `@ascribed/review`. The same form is on each package's npm page: Settings → Trusted Publisher.

- Every field is case-sensitive and exact, and npm doesn't validate it when you save. A mistake appears only when you publish.
- The workflow file is `release.yml`, without its path. Renaming the workflow breaks publishing until each package is changed: a trusted publisher's organization, repository, and workflow can't be edited, so it has to be removed and added again. Its environment and what it's allowed to do can be edited in place.
- Allow `npm publish` only. The release stages nothing and moves no dist-tags.
- Once a release has published through it, set each package's Settings → Publishing access to **Require two-factor authentication and disallow tokens**. That stops any token from publishing, including a leaked one, and trusted publishing keeps working.
- In the workflow, leave `registry-url` out of `setup-node`. It writes an empty `_authToken` line, which stops npm from starting the OIDC exchange.

**Check:** `npm trust list @ascribed/cli` (for each package) shows the repository, `release.yml`, and the `release` environment. The real check is a release: the `npm` job succeeds, and each package's page shows a provenance badge.

**A package that doesn't exist yet** can't have a trusted publisher, because npm needs the package to exist first. Its first version is published by hand, from your own npm account, as [the nightly canary's setup](#setting-it-up) says in step 3. No workflow reads a token.

### 4. VS Code Marketplace: a managed identity

The publisher is `Ascribe`. Microsoft is retiring Azure DevOps personal access tokens on December 1, 2026, so the release signs in as a Microsoft Entra managed identity instead of using a token. The workflow's `azure/login` step trades the job's GitHub OIDC token for an Entra token, and `vsce publish --azure-credential` publishes with it. Nothing is stored. Set this up once:

1. **An Azure account** with its own Microsoft Entra tenant and a subscription. A personal Microsoft account has no tenant until it signs up at [azure.microsoft.com/free](https://azure.microsoft.com/free/).
2. **A user-assigned managed identity** (Azure portal → Managed Identities → Create), named `ascribe-vscode-publisher`, in a region that supports federated credentials, such as East US.
3. **A federated credential** on it (Settings → Federated credentials → Add credential → GitHub Actions deploying Azure resources): organization `ascribed-dev`, repository `ascribe`, entity type **Environment**, environment `release`. This repository issues immutable subject claims, so check that the subject reads `repo:ascribed-dev@335742967/ascribe@1393013016:environment:release`, with the audience `api://AzureADTokenExchange`. A wrong subject saves without an error and fails later.
4. **An Azure DevOps organization connected to that tenant** (dev.azure.com, created while signed in as a member user of the tenant, not a guest). Add the identity as a user: Organization settings → Users → Add users, by the identity's name. This gives it an Azure DevOps profile, which the Marketplace needs.
5. **The identity's Azure DevOps profile id.** Run `az rest -u https://app.vssps.visualstudio.com/_apis/profile/profiles/me --resource 499b84ac-1321-427f-aa17-267ca6975798` as the identity: a one-off workflow job in the `release` environment that signs in with `azure/login` and runs that command. The `id` in the response is the profile id. It isn't the client ID, object ID, or resource ID.
6. **Add the identity to the publisher.** At marketplace.visualstudio.com/manage, open `Ascribe` → Members → Add, paste the profile id, and give it the **Contributor** role.
7. **Two variables on the `release` environment** (Settings → Environments → release → Environment variables; they aren't secrets): `AZURE_CLIENT_ID`, the identity's client ID, and `AZURE_TENANT_ID`, the directory (tenant) ID.

**Check:** the identity is listed under the publisher's Members as Contributor, the two variables exist on the environment, and the `marketplace` job's login step succeeds on a real release.

## Each release

### 1. Check what's published

The changelog's section for the release lists what it ships, and nothing in it is unfinished.

### 2. Set the version

```sh
node scripts/release/version.ts 0.2.0
```

This sets the version in `Cargo.toml` (and `Cargo.lock`), every published `package.json`, and the extension's `ascribe.minServerVersion`, which makes the extension warn about a project binary older than itself. Lower `minServerVersion` by hand if the release still works with older binaries.

Set the `version` phrase in `docs/ascribe.toml` to the new version too; a test fails until it matches.

In `CHANGELOG.md`, write the release's section as `## 0.2.0 (YYYY-MM-DD)`, with today's date. For 0.1.0, replace `(unreleased)` with the date.

**Check:** `node scripts/release/version.ts --check` prints the version.

### 3. Check everything locally

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
pnpm install --frozen-lockfile
pnpm format:check && pnpm lint
pnpm --filter @ascribed/cli build && pnpm --filter @ascribed/elements build
pnpm typecheck && pnpm test
```

### 4. Commit and push

```sh
git commit -am "Release 0.2.0"
git push origin main
```

### 5. Dry run

Actions → **Release** → Run workflow, from `main`, with **publish** unchecked. It builds the binary on each platform, packs everything, installs the npm packages and each extension package on its platform and checks them against `examples/quill`, and runs the publish steps with `--dry-run`.

**Check:** every job passes, and the **CI** workflow is green on the same commit. CI on `main` runs Linux and macOS; every platform runs in its nightly run, so check that the latest one passed, or start one from `main` (Actions → CI → Run workflow). A failing job on any platform stops the release: don't tag until it's fixed. Download the `release` artifact and look it over: `npm/` has eight tarballs, `vsix/` four packages, `github/` four archives.

### 6. Tag

```sh
git tag -a v0.2.0 -m "Ascribe 0.2.0"
git push origin v0.2.0
```

### 7. Publish

Actions → **Release** → Run workflow, from the tag `v0.2.0` (under "Use workflow from", choose Tags), with **publish** checked. The workflow checks that the tag matches the version. The build, pack, and smoke jobs run as in the dry run; then each publishing job waits for your approval.

1. **Approve `npm`.** **Check:** `npm view @ascribed/cli@0.2.0 version` and `npm view @ascribed/astro@0.2.0 version` print `0.2.0`, and each package's npm page shows a provenance badge. `npm view <package> dist-tags` shows `latest` at `0.2.0` for all eight packages. A package whose first version was a canary has that canary as `latest` until a release; `@ascribed/review`'s `latest` is `0.1.2-next.2` until the release after 0.1.1.
2. **Approve `marketplace`.** **Check:** the Marketplace page for `Ascribe.ascribe-vscode` shows 0.2.0 (it can take a few minutes to appear).
3. **Approve `github`.** **Check:** a draft release `v0.2.0` exists with eight archives and packages and `SHA256SUMS`, and the changelog section as its notes.

### 8. Try it as a user

On a machine that hasn't built Ascribe:

```sh
mkdir try-ascribe && cd try-ascribe && npm init -y
npm install --save-dev @ascribed/cli
npx ascribe --version
```

Install the extension from the Marketplace, open a copy of `examples/quill`, and check that diagnostics appear on a page with a mistake. **Ascribe: Show Server Output** says which binary it uses.

### 9. Publish the GitHub release

Review the draft, then publish it.

### 10. Check the docs site

The [docs site](#the-docs-site) follows `main`, not the release, so a release doesn't change it by itself. In the pull request after the release, set `version` in `docs/ascribe.toml`'s `[phrases]` to the release, and change `[features.next]` to name it, as the comment above `[dimensions.release]` says. Once that's merged and Netlify has built it, open <https://ascribed-dev.com>: **Check:** getting-started installs the new version, and a page the release shipped reads "since" the release, not "in main, not yet released".

## When something goes wrong

**Before any publishing job ran:** nothing is public. Fix the problem on `main`, then move the tag to the fixed commit and run again:

```sh
git tag -d v0.2.0 && git push --delete origin v0.2.0
git tag -a v0.2.0 -m "Ascribe 0.2.0" && git push origin v0.2.0
```

**A publishing job failed partway:** run the workflow again from the same tag with **publish** checked. Each step skips what's already published: npm packages at this version, Marketplace targets already there (`--skip-duplicate`), and a draft release that exists (its files are replaced).

**A published npm version is bad:** npm never lets a version number be reused, even after unpublishing, so fix forward with a new patch version. Meanwhile:

```sh
npm deprecate @ascribed/cli@0.2.0 "Broken; use 0.2.1"          # for each package
npm dist-tag add @ascribed/cli@0.1.0 latest                     # point `latest` back
```

Unpublishing (`npm unpublish @ascribed/cli@0.2.0`) is allowed only within 72 hours and when nothing depends on the version; prefer deprecating. Keep the eight packages at one version: move the tags of all of them together.

**A published extension version is bad:** the Marketplace has no rollback to an earlier version. Publish a fixed patch version. Unpublishing (`vsce unpublish`, or from the publisher's management page) removes the whole extension and its install count, so reserve it for emergencies.

### The Marketplace identity: failures we've seen, and a one-off check

| Where | What you see | What it means | What to do |
|---|---|---|---|
| Azure portal sign-in | `AADSTS16000 … does not exist in tenant 'Microsoft Services'` | A personal Microsoft account has no Entra tenant of its own. | Sign up at [azure.microsoft.com/free](https://azure.microsoft.com/free/) in a private window. That creates the tenant. |
| Marketplace → Members → Add | `Not a valid User Id` | You entered the Azure resource id, client id, or object id. | Enter the Azure DevOps profile id (item 5 of the identity setup). |
| The profile call in a one-off job | `VSS011031: There is no profile for the authenticated user` | The identity isn't a user of an Azure DevOps organization connected to the tenant. | Item 4 of the identity setup. Organization settings → Microsoft Entra must show the tenant. An organization created with a personal account isn't connected, and its Add users box won't find the identity. Reportedly an Entra guest can't add a service principal by name, so use a member user of the tenant. |
| Any job in the `release` environment | The job never starts, or the deployment is rejected | "Selected branches and tags" is on with no rules, which blocks every ref. | Add the `v*` tag rule ("Before the first release", step 2). A one-off run from a branch needs a temporary branch rule for that branch, removed afterward. |
| `azure/login` | An error about no matching federated identity record | The credential's subject, issuer, or audience differs from the token's. A wrong subject saves without an error. | The login step prints the token's `subject claim`. Compare it with the credential character by character. |
| `vsce publish`, after a good login | A 401 or 403 from the Marketplace | The usual cause is that the identity isn't a Contributor on the publisher, or the profile id entered was wrong. | Check Members on the publisher page. |

If you recreate the identity, repeat items 2 to 7 of the identity setup: the new identity has a new client id and a new profile id.

**Reading the identity's profile id (item 5 of the identity setup).** Only the identity can ask for it, so it takes a one-off job on GitHub. A workflow can only be dispatched if its file exists on `main`, so do it on a scratch branch that replaces `release.yml` for the run, and never merge it:

1. Create a branch, replace `.github/workflows/release.yml` with the file below, and push it.
2. In the `release` environment's deployment rules, add a temporary branch rule for that branch.
3. Run `gh workflow run release.yml --ref <branch>`, then approve the deployment on the run's page.
4. Read the `id` in the log, then delete the branch and the rule.

```yaml
name: Release
on:
  workflow_dispatch:
permissions:
  id-token: write
  contents: read
jobs:
  profile:
    runs-on: ubuntu-24.04
    environment: release
    steps:
      - uses: azure/login@<the commit SHA release.yml uses>
        with:
          client-id: <the identity's client id>
          tenant-id: <the tenant id>
          allow-no-subscriptions: true
      - run: az rest -u https://app.vssps.visualstudio.com/_apis/profile/profiles/me --resource 499b84ac-1321-427f-aa17-267ca6975798
```

**The GitHub release is bad:** while it's a draft, `gh release delete v0.2.0` removes it. After publishing, edit it, or mark it a pre-release while a fix is prepared.

## The nightly canary

The [canary workflow](.github/workflows/canary.yml) publishes the npm packages (the four platform packages, `@ascribed/cli`, `@ascribed/elements`, `@ascribed/review`, and `@ascribed/astro`) from `main` every night at 04:17 UTC, under the `next` tag, with nobody approving it. It's for sites that follow `main`, ours first: `npm install @ascribed/cli@next @ascribed/astro@next`. The VS Code extension has no canary.

- **The version** is `<next>-next.<n>`: the version after the latest release, and the workflow's run number. The next version is the one the changelog's unreleased section names (`## 0.2.0 (unreleased)`), or a patch bump of the workspace's version under `## Unreleased`. `node scripts/release/version.ts --canary <n>` stamps it in the working tree; it's never committed. A canary sorts before the release it leads to, so `npm install` of a range never picks one.
- **Run numbers have to keep rising.** They restart if `canary.yml` is renamed or deleted and recreated, and then each canary's version may already be on npm, so it publishes nothing and passes. If that happens, name the unreleased section's version (`## 0.2.0 (unreleased)`), or wait for the next release, so the versions are new.
- **Its commit** is each package's `gitHead` (`npm view @ascribed/cli@next gitHead`), and `ascribe --version` prints it: `ascribe 0.1.2-next.42 (3f9c2a1b7d4e)`.
- **It runs only when something ships.** It compares `main` with the last canary's commit, in the files that ship, which `node scripts/release/canary.ts paths` lists. When nothing there has changed, it publishes nothing, and the run passes.
- **It builds, packs, and smoke-tests as a release does,** with the same jobs (`release-build.yml`). If any of them fails, nothing is published and `next` stays where it was.
- **It publishes from the `canary` environment,** with provenance. Every package goes under the holding tag `next-pending` first, and `next` moves to the new canary on each package only once all of them are on npm, so `next` never mixes two nights. Then it calls the [docs site](#the-docs-site)'s build hook, so the site rebuilds with the new canary.

To publish one by hand (after a fix, or when a night's wait matters): Actions → **Canary** → Run workflow, from `main`. **force** publishes even when nothing has changed.

### Setting it up

Do these once, after the release setup above.

1. **The `canary` environment.** Settings → Environments → **New environment** → `canary`. No required reviewers. **Deployment branches and tags:** Selected branches and tags → add the branch rule `main`. **Check:** the environment has no reviewers and allows only `main`.
2. **A second trusted publisher on each npm package.** A package can trust up to ten workflows. On each package's npm page, Settings → Trusted Publisher → add a GitHub Actions publisher: organization or user `ascribed-dev`, repository `ascribe`, workflow filename `canary.yml`, environment `canary`, with **Allow npm publish** and **Allow npm dist-tag** both ticked (the canary moves `next` itself). Do it for every package that exists. Leave the `release.yml` one as it is. The workflow filename must be `canary.yml`, not `release-build.yml`: npm checks the workflow that was started, not a reusable one it calls. **Check:** each package's Trusted Publisher settings list both `release.yml` (environment `release`) and `canary.yml` (environment `canary`). The real check is a canary: the `npm` job succeeds, and `npm view @ascribed/cli dist-tags` shows `next` on it and `latest` unchanged.
3. **A package that doesn't exist yet** (such as a new one no release has published) can't have a trusted publisher, because npm needs the package to exist first. The canary's `npm` job stops before publishing anything, and its error names the package. Publish that package's first version by hand, signed in to npm (`npm login`) as an owner of the `@ascribed` organization, so npm asks for your two-factor code and no token is made:
   1. Download the failed run's `release` artifact, and unzip it.
   2. `npm publish npm/<tarball> --access public --tag next`, with the new package's tarball from the artifact's `npm/` folder.
   3. Add the package's trusted publishers: `canary.yml`, as in step 2, and `release.yml`, as in [npm: trusted publishing](#3-npm-trusted-publishing).
   4. Re-run the failed job. It skips the version you published, publishes the rest, and moves `next`.

   That first version has no provenance badge. npm makes it the package's `latest` too, so until a release publishes the package, `latest` is that canary; the release's [publish check](#7-publish) catches it. **Check:** the new package is on npm under `next`, and its Trusted Publisher settings list both workflows.

### A bad canary

A canary can't be replaced: npm never reuses a version. Fix `main`, then run the workflow by hand; the new canary takes `next`. Meanwhile, deprecate the bad one, and point `next` back at the last good one:

```sh
npm deprecate @ascribed/cli@0.1.2-next.42 "Broken; use 0.1.2-next.43"   # for each package
npm dist-tag add @ascribed/cli@0.1.2-next.41 next                         # for each package
```

**A publish that failed partway:** `next` hasn't moved, and the packages that were published are under `next-pending`. Re-run the failed job: a re-run keeps the run number, so the version is the same, packages already published are skipped, and then `next` moves. If you don't, the next night's canary replaces it.

## The docs site

The user docs are published at <https://ascribed-dev.com>, the address in `[consumer]` in `docs/ascribe.toml`. Netlify builds it from `main`, as a user's host would: `site/netlify.toml` installs the site's packages, moves Ascribe's to the canary `next` names (`npm run follow-next`), and builds. No Rust, no workspace, and no secret, so the site follows `main` through the canary, a night behind it at most. It rebuilds when `main` changes `docs/` or `site/`, and when the canary calls its build hook. A pull request gets a preview from the **Site** workflow instead, built with its own Ascribe. [site/README.md](site/README.md) has the details.

**A build that fails** leaves the last good deploy up, and Netlify emails the owner. The usual cause is a page documenting a feature merged that day, which the canary doesn't have until the night's run; the canary's build hook then rebuilds it. When waiting a night matters, run the canary by hand. The **Site from npm** workflow, which builds the site the same way on each push to `main`, doesn't fail for that cause: while the canary lacks what the commit ships, a failure ends with a warning, "Waiting for a canary that includes …", and a link to run the canary, and the run after the canary is the real check ([CONTRIBUTING.md](CONTRIBUTING.md#the-docs-site)). Netlify's deploy log (Deploys, in the site's dashboard) shows the error; `cd site && npm ci && npm run follow-next && npm run build` reproduces it.

### Setting it up

Once, in Netlify (the site `ascribe-docs`, linked to this repository) and GitHub:

1. **Build settings.** Project configuration → Developer settings → Continuous deployment → Build settings → Configure: **Base directory** `site`, and leave the build command and publish directory empty (`site/netlify.toml` sets them). **Production branch** `main`. **Check:** a deploy's log says it read `site/netlify.toml` and ran `npm ci && npm run follow-next && npm run build`.
2. **No deploy previews or branch deploys.** Same page, Branches and deploy contexts → Configure: deploy previews off, branch deploys off. **Check:** a new pull request gets no Netlify checks, only the **Site** workflow's preview.
3. **Failure notifications.** Project configuration → Notifications → Emails: add **Deploy failed** to the owner's address.
4. **The build hook.** Continuous deployment → Build hooks → Add build hook, named `canary`, on `main`. In GitHub, Settings → Secrets and variables → Actions → New repository secret: `SITE_BUILD_HOOK`, the hook's URL. **Check:** run the canary by hand with **force**; its **rebuild the docs site** job passes, and a deploy titled `canary <version>` appears in Netlify.
5. **The preview's secrets.** In Netlify, user settings → Applications → Personal access tokens → New access token (give it an expiration, and put a reminder where you'll see it). In GitHub, two repository secrets: `NETLIFY_AUTH_TOKEN`, the token, and `NETLIFY_SITE_ID`, the Project ID from Project configuration → General → Project details. **Check:** a pull request that changes `docs/` gets a **Preview** link in the **Site** workflow's summary, at `https://pr-<number>--ascribe-docs.netlify.app`.

## Known limitations

- **Intel Macs:** not supported. There's no `darwin-x64` npm package or extension package, so npm installs no binary there (`ascribe` says the platform isn't supported) and the Marketplace doesn't offer the extension.
- **Linux:** the binaries link against glibc 2.28 (built with `cargo zigbuild --target <triple>.2.28`), so they run on glibc 2.28 or later: Debian 10, Ubuntu 18.10, RHEL 8, Amazon Linux 2023, and newer. That's Node.js 24's floor too. The `smoke` job runs each Linux binary in AlmaLinux 8 (glibc 2.28) and fails if it asks for more. Zig comes from PyPI, pinned by hash in `scripts/release/zig-requirements.txt`. cargo-zigbuild runs Cargo with its unstable features turned on, so after bumping Rust, Zig, or cargo-zigbuild, do a dry run before releasing. To move the floor, change `GLIBC` at the top of `release-build.yml`, the smoke image, the `glibc` phrase in `docs/ascribe.toml`, this section, and `packages/cli/README.md`; `scripts/docs-site/facts.test.ts` fails until they agree. Alpine and other musl systems aren't supported; npm won't install the Linux packages there.
- **macOS and Windows downloads:** the binaries in the GitHub release aren't signed with a Developer ID or Authenticode certificate. A binary downloaded with a browser is quarantined on macOS (`xattr -d com.apple.quarantine ascribe` clears it) and may trigger SmartScreen on Windows. Binaries installed from npm or inside the extension aren't affected.
- **Actions:** the workflow uses `macos-26`, `ubuntu-24.04`, `ubuntu-24.04-arm`, and `windows-2025` runners. If GitHub retires one, update both `build` and `smoke`.
