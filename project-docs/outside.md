# Outside the repository

Everything Ascribe relies on that doesn't live in this repository: what it is, what uses it, and who can change it. It started as the [optimization plan](optimization/README.md)'s phase 6, part C, from the [inventory's table](optimization/inventory.md#outside-the-repository) as found on 6 October 2026, and moved here when that plan finished.

It holds names, never values. When you add, rename, or remove one of these, change this page in the same pull request. [RELEASING.md](../RELEASING.md) says how to set each up.

"The maintainer" is Kyle (`KyleBlankRollins`), the owner of the `ascribed-dev` organization and of every account below. Nobody else can change any of it, and an agent can't reach the settings pages: it can only say what to change.

## GitHub

| Thing | What uses it | Who can change it |
|---|---|---|
| `ascribed-dev/ascribe` (public) | Everything. Branch `main` only; one required check, `all checks`, no required review, and admins can bypass | Maintainer |
| `ascribed-dev/ascribe`'s description, website, and topics (the repository's page, About) | What GitHub shows beside the code and in search. They're set by hand from the words' home, `design/words.toml`: the description is its `short`, the website <https://ascribed-dev.com>, and the topics its `keywords`. Nothing in the repository sets or checks them, so they're set again when the home changes | Maintainer, in the repository's settings |
| `ascribed-dev/homebrew-tap` (public) | `brew install ascribed-dev/tap/ascribe`: its `Formula/ascribe.rb`, written for each release by `scripts/release/formula.ts`. It needs a default branch for a pull request to target | Maintainer, who merges each release's pull request ([decision #463b](decisions/463b-homebrew-tap-pull-request.md)) |
| `ascribed-dev/review-fixture` (private) | `scripts/review-fixture/setup.ts` rebuilds it, for trying review by hand | Maintainer, or anyone running the script with access |
| `ascribed-dev/sources-fixture-code`, `ascribed-dev/sources-fixture-docs` (private) | `scripts/sources-fixture/setup.ts`, for running the sources update workflow for real. The docs repository runs `examples/docs-repository`'s workflow on a schedule. The code repository is `CODE` in `scripts/sources-fixture/changes.ts`: the client library, its quickstart, and the quickstart's tested examples config, test, and observation, which a scripted change (`untested`) makes stale. A change to `CODE` reaches GitHub when the maintainer runs `setup.ts ascribed-dev` again | Maintainer, or anyone running the script with access |
| `ascribed-dev/tested-examples-node`, `ascribed-dev/tested-examples-python` (public) | The repositories to copy that [Show what an example was tested with](../docs/content/examples/tested.md#start-from-a-repository) points at. Each one's `examples.yml` runs its tests, checks its observations, and builds its docs on every pull request and weekly, on Mondays at 06:23 UTC; a failed scheduled run is emailed to the maintainer, who owns the workflow. Each pins a release of `ascribe`, which [RELEASING.md](../RELEASING.md#8-try-it-as-a-user) builds their docs with | Maintainer |
| A GitHub App with read access to `sources-fixture-code` and write access to `sources-fixture-docs` | That workflow's tokens: the variable `SOURCES_APP_CLIENT_ID` and the secret `SOURCES_APP_PRIVATE_KEY` in `sources-fixture-docs` ([Keep the docs in another repository](../docs/content/examples/docs-repository.md) says how) | Maintainer |
| `KyleBlankRollins/ascribe-review-scratch` (private) | Nothing. `review-fixture` replaced it | Maintainer: **to delete or archive** |
| Repository secrets `NETLIFY_AUTH_TOKEN`, `NETLIFY_SITE_ID` | `site.yml`: a pull request's docs preview | Maintainer |
| Repository secret `SITE_BUILD_HOOK` | `canary.yml`'s `site` job: rebuilds the docs site after a canary | Maintainer |
| Repository secret `PROJECT_BOARD_TOKEN`: a fine-grained token of the maintainer's, owned by `ascribed-dev`, with Organization → Projects: read and write | `board.yml`: reads [the project board](https://github.com/orgs/ascribed-dev/projects/1) into `board.json` on the branch `board`, and moves issues on it, for agents that can't reach the board | Maintainer; it stops working when the token expires |
| Repository setting Actions → General → "Allow GitHub Actions to create and approve pull requests" | `approve.yml`: approves a pull request as `github-actions[bot]` for a reviewing agent, since agents post as the pull request's author; `templates.yml`: opens the templates' update pull request | Maintainer |
| Environment `release` (no reviewers; `v*` tags only) | `release.yml`'s `npm`, `marketplace`, `github`, and `tap` jobs | Maintainer |
| Tag ruleset `Release tags`: only a repository admin creates, moves, or deletes a `v*` tag | What makes pushing a tag the release's sign-off, since `release` has no reviewer ([decision #489](decisions/489-release-tag-is-the-sign-off.md)) | Maintainer |
| Secret `HOMEBREW_TAP_TOKEN`, on `release` (**to add**): a fine-grained token of the maintainer's, resource owner `ascribed-dev`, for the repository `ascribed-dev/homebrew-tap` only, with Contents: read and write and Pull requests: read and write | `release.yml`'s `tap` job: pushes a branch to the tap and opens its pull request. Without it, the job writes the formula and opens nothing | Maintainer; it stops working when the token expires |
| Variables `AZURE_CLIENT_ID`, `AZURE_TENANT_ID`, on `release` | `release.yml`'s `azure/login`: the Marketplace managed identity below. Not secrets | Maintainer |
| Environment `canary` (no reviewers; `main` only) | `canary.yml`'s `npm` job | Maintainer |
| Secret `NPM_TOKEN` | Nothing, once phase 6 part C merges. The inventory found it on `release`; RELEASING.md had it on `canary` | Maintainer: **to delete**, from whichever environment holds it, and revoke the token on npmjs.com |
| Environment `copilot` | Nothing in this repository's workflows. Probably made by GitHub when Copilot's coding agent was tried here (inferred, not checked) | Maintainer: **to say**, or delete |
| Dependabot | Weekly updates for cargo, npm, and Actions (`.github/dependabot.yml`) | Anyone, by pull request |
| Third-party actions | 8 in this repository's workflows (`actions/cache`, `checkout`, `download-artifact`, `setup-node`, `upload-artifact`; `Swatinem/rust-cache`, `pnpm/action-setup`, `azure/login`), plus `actions/create-github-app-token` in `examples/docs-repository`. Each pinned to a commit | Anyone, by pull request; Dependabot moves them |
| Vale's Linux release, from `vale-cli/vale` on GitHub | `rust.yml`'s `check` job downloads it, at the version and checksum written there, for the tests that run a real Vale (`crates/ascribe-report/tests/all/vale.rs`). Ascribe itself never downloads Vale | Anyone, by pull request: a new version needs its checksum from the release's `checksums.txt` |
| A dated Rust nightly, and `cargo-fuzz` and `libfuzzer-sys` from crates.io | `fuzz.yml`'s nightly fuzzing only, never a pull request: the nightly is `fuzz/rust-toolchain.toml`'s, `cargo-fuzz` is the version written in `fuzz.yml`, and `libfuzzer-sys` is in `fuzz/Cargo.lock`, outside the workspace's (decision [#1020](decisions/1020-fuzzing-outside-the-workspace.md)). Dependabot doesn't watch `fuzz/` | Anyone, by pull request, after a local `cargo fuzz run` with the new versions |
| lychee's Linux release, from `lycheeverse/lychee` on GitHub | `report.yml` downloads it, at the version and checksum written there, for the weekly `ascribe report links` on the docs. Ascribe itself never downloads it | Anyone, by pull request: a new version needs its checksum, worked out from the release's archive |
| Pull requests from the branch `templates/update` | `templates.yml` opens one when The Good Docs Project releases, with the page templates written again, and updates it while it's open, with the job's own token, so its checks wait for someone to close and reopen it. A GitHub App's token would start them, and would be one more secret | Anyone: read it and merge or close it. The token is the maintainer's to change |
| Issues titled "Fuzz: <target> crashes (<signature>)" | `fuzz.yml` opens one for each distinct crash the nightly fuzzing finds, and comments on it while it's open, with the job's own token | Anyone: close it once the crash is fixed |
| Issues titled "Docs report: links and the published site" | `report.yml` opens one when the weekly report finds something, and updates it while it's open, with the job's own token | Anyone: close it once it's dealt with |

## npm

| Thing | What uses it | Who can change it |
|---|---|---|
| Organization `ascribed` | Owns the packages below | Maintainer |
| `@ascribed/cli`, `@ascribed/astro`, `@ascribed/elements`, `@ascribed/review`, and `@ascribed/cli-darwin-arm64`, `-linux-arm64`, `-linux-x64`, `-win32-x64` | Users; `site/` and the Drift and Review jobs install `@next` | `release.yml` and `canary.yml`; a new package's first version by hand, by the maintainer ([RELEASING.md](../RELEASING.md#setting-it-up)) |
| Trusted publishers on each package: `release.yml` with environment `release`, and `canary.yml` with environment `canary` | Publishing without a token | Maintainer, on each package's npm page |
| Tag `latest` | `npm install`. 0.1.1, except **`@ascribed/review`'s, which is `0.1.2-next.2`**, a canary, until the next release moves it | `release.yml` |
| Tags `next` and `next-pending` | The nightly canary (0.1.2-next.6 when the inventory ran) | `canary.yml` |
| `afdocs`, the delivery spec's checker, at 0.22.2 | `examples/astro-site`'s end-to-end job, and `report.yml`'s weekly `ascribe report delivery` on <https://ascribed-dev.com>. The version is written in both, and in `ascribe_report::site::VERSION`, which says it in the report's install hint | Anyone, by pull request, after reading the new version's checks: a check it adds is listed in `ascribe_report::site::CHECKS` |

## Microsoft

| Thing | What uses it | Who can change it |
|---|---|---|
| VS Code Marketplace publisher `Ascribe`, extension `Ascribe.ascribe-vscode` | Users of the extension. It stays `Ascribe`, the one place Ascribe is named as a maker: a publisher can't be renamed (decision [#833h](decisions/833h-bryo-studio-maker.md)) | Maintainer, through a personal Microsoft account |
| Managed identity `ascribe-vscode-publisher` (Azure, resource group `ascribe-release`, East US), with a federated credential trusting this repository's `release` environment | `release.yml`'s `marketplace` job | Maintainer |
| An Azure DevOps organization connected to that tenant | Gives the identity the profile the Marketplace needs. No projects or pipelines | Maintainer |

## The Good Docs Project

| Thing | What uses it | Who can change it |
|---|---|---|
| Their templates' repository, <https://gitlab.com/tgdp/templates>, public on GitLab | `scripts/templates/convert.ts` fetches the release `scripts/templates/upstream.toml` pins, from their raw files, to write Ascribe's page templates, and `rust.yml` runs it. `templates.yml` reads their tags and the diff between two through GitLab's API every week (`scripts/templates/update.ts`). Neither signs in: their tags and files answer without it, and their releases answer 403 | The Good Docs Project. A moved or renamed repository is a change to `repository` in `upstream.toml` |

## Netlify and the domain

| Thing | What uses it | Who can change it |
|---|---|---|
| Site `ascribe-docs` | Serves <https://ascribed-dev.com> from `main`, built with the canary (`site/netlify.toml`) | Maintainer |
| Build hook `canary` | `SITE_BUILD_HOOK`, above | Maintainer |
| A Netlify personal access token, with an expiry | `NETLIFY_AUTH_TOKEN`, above | Maintainer; it needs replacing when it expires |
| The install script's address, <https://ascribed-dev.com/install.sh> | `curl -fsSL https://ascribed-dev.com/install.sh \| sh`, in the docs' getting started. The site serves `site/public/install.sh` as it is ([decision #463a](decisions/463a-install-script-address.md)) | Anyone, by pull request; it changes only through this repository |
| The domain `ascribed-dev.com` | The docs site, and `[consumer]` in `docs/ascribe.toml` | Maintainer. Its registrar and DNS weren't checked |

## Images

The mark's images that an outside account shows, generated in this repository ([design/README.md](../design/README.md#the-assets)). Each is uploaded by hand on the account's settings page, and again when the image changes; nothing in the repository can.

| Where it shows | The file | Who can change it |
|---|---|---|
| The `ascribed-dev` organization's avatar on GitHub | `design/out/avatar.png` | Maintainer, in the organization's settings |
| The repository's social preview, shown when a link to it is shared | `site/public/social-card.png` | Maintainer, in the repository's settings (General, Social preview) |
| The Marketplace publisher `Ascribe`'s logo | `design/out/avatar.png` | Maintainer, on the publisher's management page |
| The npm organization `ascribed` | Nothing yet: npm shows a Gravatar for an account. Whether an organization can take one wasn't checked | Maintainer |

The extension's own icon and the docs site's favicon and social card aren't here: the extension package and the site carry them.

## Not checked

These need the maintainer's accounts, so neither the inventory nor this page could look: Netlify's settings, the npm trusted-publisher entries, the GitHub App's permissions, the Marketplace publisher's members, the domain's registrar, and what the `copilot` environment is for.
