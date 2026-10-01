# Releasing Ascribe

A release publishes one version of everything: the `ascribe` binaries (on a GitHub release), seven npm packages, and four VS Code extension packages, one per platform. The [release workflow](.github/workflows/release.yml) builds, packs, tests, and publishes; a person starts it, approves each publishing step, and publishes the GitHub release at the end. Nothing is published without that person.

| What | Where | Published by |
|---|---|---|
| `@ascribed/cli-darwin-arm64`, `-linux-arm64`, `-linux-x64`, `-win32-x64` | npm | the `npm` job |
| `@ascribed/cli`, `@ascribed/elements`, `@ascribed/astro` | npm | the `npm` job |
| `Ascribe.ascribe-vscode`, for each of the four platforms | VS Code Marketplace | the `marketplace` job |
| `ascribe-<version>-<platform>.tar.gz` (`.zip` for Windows), the `.vsix` files, `SHA256SUMS` | GitHub release | the `github` job drafts it; you publish it |

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

### 3. npm: a publishing token

On npmjs.com, as an owner of the `ascribed` organization: Access Tokens → **Generate New Token** → Granular Access Token:

- **Packages and scopes:** Read and write, for the `@ascribed` scope.
- **Organizations:** no access.
- **Expiration:** as short as your release schedule allows.

Add it to the `release` environment as the secret `NPM_TOKEN`.

**Check:** `npm whoami --//registry.npmjs.org/:_authToken=<token>` prints your user name.

After the first release, you can replace the token with npm's trusted publishing: for each of the seven packages, Settings → Trusted publishing → GitHub Actions, with this repository, the workflow `release.yml`, and the environment `release`. Then delete the token and the secret. The workflow already uses an npm that supports it.

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

**Check:** every job passes, and the Rust and JavaScript workflows are green on the same commit. A failing job on any platform stops the release: don't tag until it's fixed. Download the `release` artifact and look it over: `npm/` has seven tarballs, `vsix/` four packages, `github/` four archives.

### 6. Tag

```sh
git tag -a v0.2.0 -m "Ascribe 0.2.0"
git push origin v0.2.0
```

### 7. Publish

Actions → **Release** → Run workflow, from the tag `v0.2.0` (under "Use workflow from", choose Tags), with **publish** checked. The workflow checks that the tag matches the version. The build, pack, and smoke jobs run as in the dry run; then each publishing job waits for your approval.

1. **Approve `npm`.** **Check:** `npm view @ascribed/cli@0.2.0 version` and `npm view @ascribed/astro@0.2.0 version` print `0.2.0`, and each package's npm page shows a provenance badge.
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

Unpublishing (`npm unpublish @ascribed/cli@0.2.0`) is allowed only within 72 hours and when nothing depends on the version; prefer deprecating. Keep the seven packages at one version: move the tags of all of them together.

**A published extension version is bad:** the Marketplace has no rollback to an earlier version. Publish a fixed patch version. Unpublishing (`vsce unpublish`, or from the publisher's management page) removes the whole extension and its install count, so reserve it for emergencies.

**The GitHub release is bad:** while it's a draft, `gh release delete v0.2.0` removes it. After publishing, edit it, or mark it a pre-release while a fix is prepared.

## Known limitations

- **Intel Macs:** not supported. There's no `darwin-x64` npm package or extension package, so npm installs no binary there (`ascribe` says the platform isn't supported) and the Marketplace doesn't offer the extension.
- **Linux:** the binaries are built on Ubuntu 24.04 and need glibc 2.39 or later. Alpine and other musl systems aren't supported; npm won't install the Linux packages there.
- **macOS and Windows downloads:** the binaries in the GitHub release aren't signed with a Developer ID or Authenticode certificate. A binary downloaded with a browser is quarantined on macOS (`xattr -d com.apple.quarantine ascribe` clears it) and may trigger SmartScreen on Windows. Binaries installed from npm or inside the extension aren't affected.
- **Actions:** the workflow uses `macos-26`, `ubuntu-24.04`, `ubuntu-24.04-arm`, and `windows-2025` runners. If GitHub retires one, update both `build` and `smoke`.
