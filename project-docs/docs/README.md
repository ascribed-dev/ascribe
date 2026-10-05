# Docs

Ascribe's own documentation, written with Ascribe, and treated as Ascribe's first customer: built, hosted, reviewed, and kept true to the code the way we expect a user to do it. Where our setup differs from a user's, that's a defect in the plan, not a convenience.

It has two halves:

- **The docs.** `docs/` becomes an Ascribe project and gets a site, which Netlify builds from the npm packages, as a user's host would.
- **Drift.** Features that tell a team when its docs have fallen behind its code, built here because this repository has the problem: generated reference that can't go stale, code examples taken from tested files, and a report of pages whose covered code changed without them.

This plan isn't tied to a release. Other work may ship before or between its phases; each phase says what it needs, and nothing here assumes a version number.

## Names

- **User docs** are for people who write documentation with Ascribe. The developer docs (contracts, READMEs, contributing) stay as they are for now; see [Later](#later-not-in-this-plan).
- **Drift** is a page saying something the code no longer does. The three checks are named for what they catch: **stale generated content**, **stale examples**, and **uncovered changes** (code a page covers changed, and the page didn't).
- **A source** is a named set of files outside a project's content that its pages refer to: code a snippet comes from, and code a page covers.
- **The canary** is a build of the npm packages from `main`, published nightly under the `next` tag.

## Who it's for

Engineers who document their own code in the same repository. Research (`reports/`) says they feel drift most, and keeping docs beside the code doesn't prevent it. Teams with a separate docs repository come later; the one thing this plan does for them is choose formats that won't have to change.

## What exists today

- `docs/` holds 13 plain Markdown pages, about 4,650 lines. None of it is Ascribe source, and none is published as a site.
- `docs/diagnostics.md` is generated from `tests/conformance/diagnostics.toml` by a test that fails when it's stale.
- A user installs `@ascribed/cli` and `@ascribed/astro` from npm; the platform's binary comes with them. Published versions are released by hand, each publish approved, and `RELEASING.md` says a release moves no dist-tags. Nothing is published from `main`.
- The release scripts already accept a prerelease version (`x.y.z-pre.n`) and publish one under the `next` tag (`scripts/release/manifests.ts`, `publish.ts`).
- This repository's own workflows build the binary with `cargo`, while the recipes in the docs install it from npm. We don't run what we tell users to run.
- `ascribe check`, `build`, and `fmt` need only the working tree. `ascribe diff` needs `git` history and says so clearly in a shallow clone.
- Site hosts differ. Netlify builds on Ubuntu 24.04 with what's reported to be a full, blobless clone; Vercel and Cloudflare Pages clone shallowly. The Linux binaries need glibc 2.28 or later, which every one of those images has, and the release workflow tests that floor ([#65](https://github.com/ascribed-dev/ascribe/issues/65)). These are as read in October 2026; confirm before relying on one.
- The plans for review, the editor UI, and agents tell each phase to update `docs/<name>.md` with the code. That habit is the main thing keeping the docs current.
- Brainstorm [section 1](../brainstorm.md#1-code-snippets-from-tested-code-bluehawk-built-in) describes snippets from tested code. It isn't built.

## Decisions

These are settled. Don't reopen them in a phase; if one can't be met, stop and report.

1. **The docs are customer #1.** They install Ascribe from npm, build on the host, and run in CI by the recipes the docs publish. Production never uses a binary from `target/`, a workspace link to `packages/`, or a step a user couldn't copy. The one exception is named in decision 4.
2. **The docs live in this repository,** in `docs/`, and change in the same pull request as the code.
3. **Ascribe source, not GitHub's rendering.** Pages use directives, variants, phrases, includes, and availability wherever they fit. How a page looks on GitHub isn't a constraint.
4. **Netlify builds production; Actions builds previews.** Production is Netlify's own build: `npm install` of the canary packages, then the site's build. A pull request's preview is built in GitHub Actions with that pull request's binary and packages, and handed to Netlify, because a pull request that adds a feature and documents it has no canary yet. Two build paths, on purpose.
5. **A nightly canary.** The npm packages are published from `main` every night under the `next` tag, unattended, and on demand. Releases stay manual and approved, and keep `latest`.
6. **One site, following `main`.** No versioned copies. What isn't in the latest release is marked on the page with Ascribe's own availability, so a reader on the released version isn't misled.
7. **`build` and `check` need only the working tree.** No `git` history and no network, so they run on any host. `diff` and `drift` read history and belong in CI, where the clone's depth is ours to set; they refuse a shallow clone with a message that says what to do.
8. **Generated content is a fragment.** What's generated from code is written as Ascribe source into `_generated/` fragments by a test that fails when they're stale, and pages include them.
9. **Sources are named.** `ascribe.toml` declares each source (`[sources.code]`), and pages address code as `<source>:<path>#<region>`, never by a relative path out of the project. A source is a folder in the repository today. The same address can mean another repository at a pinned commit later, with no change to any page.
10. **Snippets before coverage.** A snippet can't raise a false alarm; a coverage report can. Coverage is built only if a back-test on this repository's history shows it's worth reading.
11. **Drift is reported in the job summary, and nothing else.** No comment, no failing check, no notification. A team can opt into failing (`--exit-code`); this repository doesn't until the numbers support it.
12. **Drift checks need only `git`,** and keep no state outside the repository.
13. **What dogfooding finds is filed, not worked around.** A gap or a bug in Ascribe becomes an issue, listed in the pull request. Work around it in the docs only when the page would otherwise be wrong, with a comment naming the issue.

## The pieces

| Piece | Where | What it does |
|---|---|---|
| The user docs | `docs/` | An Ascribe project: `ascribe.toml`, `content/` |
| Generated reference | Tests beside the code; `docs/content/_generated/` | The diagnostics, each command's options, and the extension's settings, as fragments kept current by tests |
| The canary | `.github/workflows/`, `scripts/release/` | Publishes the npm packages from `main` nightly, under `next` |
| The site | `site/`, outside the pnpm workspace | An Astro site that installs Ascribe from npm, as a user's does |
| Publishing | Netlify, `site/netlify.toml`, `.github/workflows/` | Netlify builds production; Actions builds previews and the review report |
| Snippets | `SPEC.md`, the compiler crates | `@snippet`: code examples taken from tested files, from named sources |
| Coverage | `tessera-diff`, `tessera-cli` (`ascribe drift`) | Pages say which regions of code they cover; the report says which covered regions changed without them |

## Phases

Each phase leaves the repository green and can be its own pull request. A phase can start once the phases it needs are merged.

| Phase | Result | Needs phases |
|---|---|---|
| [1: The user docs as a project](phase-1-project.md) | `docs/` is an Ascribe project that passes `ascribe check` in CI. | Nothing |
| [2: Generated reference](phase-2-generated.md) | The diagnostics, each command's options, and the extension's settings are generated fragments, under test. | 1 |
| [3: The canary](phase-3-canary.md) | The npm packages are published from `main` every night under `next`. | Nothing |
| [4: The site](phase-4-site.md) | `site/` builds the user docs, installing Ascribe from npm. | 1, 3 |
| [5: Publishing](phase-5-publish.md) | Netlify builds and serves the site from `main`; pull requests get a preview and the review report. | 2, 4 |
| [6: The back-test](phase-6-back-test.md) | A script and a write-up: over past pull requests, how often would a coverage report have been right? | Nothing |
| [7: Snippets](phase-7-snippets.md) | Named sources, and `@snippet`: a code example taken from a tested file. | 1 |
| [8: Coverage](phase-8-coverage.md) | Pages say which regions of code they cover, and `ascribe drift` reports covered regions that changed without them. Region-level only, after the [back-test](back-test.md). | 6, 7 |
| [9: Our docs, kept current](phase-9-adopt.md) | Our docs take their examples from tested files and declare what they cover; CI shows the report; the measures are recorded. | 5, 7, and 8 if it was built |

Phases 1 to 5 are the docs as a user would run them. Phase 6 is a day's measurement. Phases 7 to 9 are the drift features and their use.

### What can run at the same time

- **Phases 1, 3, and 6.** They share nothing.
- **Phases 2 and 4**, once their needs are merged. Phase 2 is tests and fragments; phase 4 is `site/`.
- **Phase 7** with any of 2 to 5, once 1 is merged. It's the compiler and touches no page.

Phases 5, 8, and 9 run one at a time.

### Other plans' docs

The review, editor UI, and agents plans say "update `docs/<name>.md`". After phase 1, that file is under `docs/content/`, and it's Ascribe source: it has to pass `ascribe check`.

Phase 1 moves every page, so it conflicts with any open pull request that edits `docs/`. Start it when none is open, or tell the other pull request's author to rebase onto it; don't let it sit.

### The measures

"Customer #1" is checked by numbers, recorded in phase 5 and again in phase 9, in `project-docs/docs/measures.md`:

- **Minutes from an empty repository to a deployed site,** timed by one person following our own getting-started and Astro guides, with nothing from this repository on the machine.
- **The host's build time,** split into installing, `ascribe build`, and Astro.
- **Issues filed from dogfooding,** per phase.
- **The share of coverage reports that were right** (phase 6, then phase 9 on real pull requests).
- **Days from a code change to the docs fix,** for drift the back-test finds.

## Rules for every phase

- Branch before committing; never commit to `main`.
- Read the current code and docs before the phase file's pointers: they drift. If the phase file and the repository disagree, or a decision above can't be met, stop and report instead of choosing silently.
- **External facts change.** Netlify's build image, clone, CLI, and configuration; npm's publishing and provenance rules; GitHub Actions. Check each against its current documentation before designing against it, and say in the pull request what you checked.
- **Docs phases change wording as little as possible.** Converting a page means changing its form, not rewriting it. Where you do write, match the existing docs' voice: plain, direct, second person, sentence-case headings.
- **Every page passes `ascribe check --deny-warnings` and `ascribe fmt --check`.**
- **Links.** Between pages, link to files, never to published routes. To the repository, link to GitHub at `main` through a phrase, so the address is in one place.
- **Feature phases** (7 and 8) follow the other plans' rules for the compiler: a change to the language is made in `SPEC.md` first, with conformance cases; a new diagnostic gets a registry entry; a new command follows the command reference's conventions. `git` is real in tests (a temporary repository); nothing touches the network.
- Match the surrounding code's style, comment density, and naming. Libraries don't panic on user input; `unwrap` and `expect` are linted.
- Tests must be correct on Windows: no hard-coded `/` in filesystem paths, and `git` paths (always `/`) converted before use.
- **User-visible changes** to Ascribe update the docs and add a line to the unreleased section of `CHANGELOG.md`. Changes to the docs alone don't need a changelog line.
- No phase history in code or docs. Describe what's there now.
- Before finishing a phase, all of these pass:

  ```sh
  cargo fmt --all --check
  cargo clippy --workspace --all-targets --locked -- -D warnings
  cargo test --workspace --locked
  pnpm format:check && pnpm lint && pnpm typecheck && pnpm test
  cargo build -p tessera-cli && ./target/debug/ascribe check --deny-warnings --config docs/ascribe.toml
  ```

  (`corepack pnpm` where `pnpm` isn't on the path.)

## Later, not in this plan

- **The developer docs as a second project** (`dev-docs/`: contracts, architecture, the server's requests, contributing), with the READMEs shortened. Until then the contracts stay in the user docs and the READMEs stay as they are.
- **Sources in another repository.** A source that names a repository, pinned to a commit in a lock file; a command that moves the pin and copies in the files snippets use, so builds still need no network; drift between the old pin and the new. Decision 9 keeps pages' addresses ready for it. Build and test it on a pair of fixture repositories in the org.
- **Drift where code is changed:** a pull request comment, a lens on covered files in the editor, a line in the agents plan's stop hook, a second group in review's changed-pages list. All wait on phase 9's numbers.
- **Drift across history** (`ascribe drift --history`: pages whose covered code has changed since the page did), a way to say "I checked, it's still right" (recording the code commit that was checked, not a date), and review dates.
- **A GitHub Action** that runs check, the review report, and drift, and deepens the clone itself.
- **Snippets in the editor:** go to a snippet's source, problems shown in the code file, "used by" on the code, a preview that follows the code as it's edited.
- **A site that follows the latest release,** with `main` on its own address, once people outside the project read it.
- **A second host.** A build on Vercel or Cloudflare Pages, to catch what Netlify's image hides (after [#65](https://github.com/ascribed-dev/ascribe/issues/65)).
- **Setup in one step** (`npm create ascribe`, or `ascribe init`), informed by the minutes measured here.
- **Docs for agents to read** (brainstorm section 7), a custom domain, versioned docs, translation.
