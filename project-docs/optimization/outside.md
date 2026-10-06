# Outside the repository

Part of [Optimization](README.md), phase 6, part C. Everything Ascribe relies on that doesn't live in this repository: what it is, what uses it, and who can change it. It starts from the [inventory's table](inventory.md#outside-the-repository), as found on 6 October 2026.

It holds names, never values. When you add, rename, or remove one of these, change this page in the same pull request. [RELEASING.md](../../RELEASING.md) says how to set each up.

"The maintainer" is Kyle (`KyleBlankRollins`), the owner of the `ascribed-dev` organization and of every account below. Nobody else can change any of it, and an agent can't reach the settings pages: it can only say what to change.

## GitHub

| Thing | What uses it | Who can change it |
|---|---|---|
| `ascribed-dev/ascribe` (public) | Everything. Branch `main` only; one required check, `all checks`, no required review, and admins can bypass | Maintainer |
| `ascribed-dev/review-fixture` (private) | `scripts/review-fixture/setup.ts` rebuilds it, for trying review by hand | Maintainer, or anyone running the script with access |
| `ascribed-dev/sources-fixture-code`, `ascribed-dev/sources-fixture-docs` (private) | `scripts/sources-fixture/setup.ts`, for running the sources update workflow for real. The docs repository runs `examples/docs-repository`'s workflow on a schedule | Maintainer, or anyone running the script with access |
| A GitHub App with read access to `sources-fixture-code` and write access to `sources-fixture-docs` | That workflow's tokens: the variable `SOURCES_APP_CLIENT_ID` and the secret `SOURCES_APP_PRIVATE_KEY` in `sources-fixture-docs` ([the drift guide](../../docs/content/guides/drift.md) says how) | Maintainer |
| `KyleBlankRollins/ascribe-review-scratch` (private) | Nothing. `review-fixture` replaced it | Maintainer: **to delete or archive** |
| Repository secrets `NETLIFY_AUTH_TOKEN`, `NETLIFY_SITE_ID` | `site.yml`: a pull request's docs preview | Maintainer |
| Repository secret `SITE_BUILD_HOOK` | `canary.yml`'s `site` job: rebuilds the docs site after a canary | Maintainer |
| Environment `release` (required reviewer; `v*` tags only) | `release.yml`'s `npm`, `marketplace`, and `github` jobs | Maintainer |
| Variables `AZURE_CLIENT_ID`, `AZURE_TENANT_ID`, on `release` | `release.yml`'s `azure/login`: the Marketplace managed identity below. Not secrets | Maintainer |
| Environment `canary` (no reviewers; `main` only) | `canary.yml`'s `npm` job | Maintainer |
| Secret `NPM_TOKEN` | Nothing, once phase 6 part C merges. The inventory found it on `release`; RELEASING.md had it on `canary` | Maintainer: **to delete**, from whichever environment holds it, and revoke the token on npmjs.com |
| Environment `copilot` | Nothing in this repository's workflows. Probably made by GitHub when Copilot's coding agent was tried here (inferred, not checked) | Maintainer: **to say**, or delete |
| Dependabot | Weekly updates for cargo, npm, and Actions (`.github/dependabot.yml`) | Anyone, by pull request |
| Third-party actions | 8 in this repository's workflows (`actions/cache`, `checkout`, `download-artifact`, `setup-node`, `upload-artifact`; `Swatinem/rust-cache`, `pnpm/action-setup`, `azure/login`), plus `actions/create-github-app-token` in `examples/docs-repository`. Each pinned to a commit | Anyone, by pull request; Dependabot moves them |

## npm

| Thing | What uses it | Who can change it |
|---|---|---|
| Organization `ascribed` | Owns the packages below | Maintainer |
| `@ascribed/cli`, `@ascribed/astro`, `@ascribed/elements`, `@ascribed/review`, and `@ascribed/cli-darwin-arm64`, `-linux-arm64`, `-linux-x64`, `-win32-x64` | Users; `site/` and the Drift and Review jobs install `@next` | `release.yml` and `canary.yml`; a new package's first version by hand, by the maintainer ([RELEASING.md](../../RELEASING.md#setting-it-up)) |
| Trusted publishers on each package: `release.yml` with environment `release`, and `canary.yml` with environment `canary` | Publishing without a token | Maintainer, on each package's npm page |
| Tag `latest` | `npm install`. 0.1.1, except **`@ascribed/review`'s, which is `0.1.2-next.2`**, a canary, until the next release moves it | `release.yml` |
| Tags `next` and `next-pending` | The nightly canary (0.1.2-next.6 when the inventory ran) | `canary.yml` |

## Microsoft

| Thing | What uses it | Who can change it |
|---|---|---|
| VS Code Marketplace publisher `Ascribe`, extension `Ascribe.ascribe-vscode` | Users of the extension | Maintainer, through a personal Microsoft account |
| Managed identity `ascribe-vscode-publisher` (Azure, resource group `ascribe-release`, East US), with a federated credential trusting this repository's `release` environment | `release.yml`'s `marketplace` job | Maintainer |
| An Azure DevOps organization connected to that tenant | Gives the identity the profile the Marketplace needs. No projects or pipelines | Maintainer |

## Netlify and the domain

| Thing | What uses it | Who can change it |
|---|---|---|
| Site `ascribe-docs` | Serves <https://ascribed-dev.com> from `main`, built with the canary (`site/netlify.toml`) | Maintainer |
| Build hook `canary` | `SITE_BUILD_HOOK`, above | Maintainer |
| A Netlify personal access token, with an expiry | `NETLIFY_AUTH_TOKEN`, above | Maintainer; it needs replacing when it expires |
| The domain `ascribed-dev.com` | The docs site, and `[consumer]` in `docs/ascribe.toml` | Maintainer. Its registrar and DNS weren't checked |

## Not checked

These need the maintainer's accounts, so neither the inventory nor this page could look: Netlify's settings, the npm trusted-publisher entries, the GitHub App's permissions, the Marketplace publisher's members, the domain's registrar, and what the `copilot` environment is for.
