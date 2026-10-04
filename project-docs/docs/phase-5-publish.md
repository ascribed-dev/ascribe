# Phase 5: Publishing

Part of [Docs](README.md). Requires phases 2 and 4. Netlify, workflows, and links.

## Goal

The site is live. Netlify builds it from `main`, itself, from npm, as a user's host would. A pull request that changes the docs gets a preview of the site and the review report. Everything that points readers at the docs points at the site.

## Context

- Netlify, each to check against current documentation: linking a site to a repository; the base directory, build command, and publish directory (`netlify.toml`); its build image (the Node version it gives, and the glibc the binary needs: [#65](https://github.com/ascribed-dev/ascribe/issues/65)); how it clones (reported as full and blobless; the production build must not depend on it); build hooks; deploying built files with its CLI, as production or as a named preview; `_redirects` and `_headers`; what it does with a failed build.
- Phase 3's canary workflow and its last step, which calls a build hook. Phase 4's two ways to build.
- `.github/workflows/review.yml`: the review report for `examples/quill`, built with `cargo`. The recipe in the review guide installs from npm.
- Links to the docs: the README, each package's and crate's README, the extension's manifest and README, the binary's help and messages.
- Diagnostics' links: the agents plan's phase 1 gives each diagnostic a link to its entry, with the URL's base as one constant. If that's merged, change the constant; if not, leave a note in that phase's file.
- [Decisions 1, 4, 6, and 7](README.md#decisions).

## Design

### Production: Netlify's own build

A Netlify site linked to this repository, with `site/` as its base directory. `site/netlify.toml` says how to build: `npm ci`, then the site's build, publishing its output. Nothing else: no Rust, no workspace install, no secret.

It builds when:

- `main` changes under `docs/` or `site/`;
- the canary workflow calls its build hook, after a new canary passes its smoke test.

**The lag, and what happens in it.** A pull request can add a feature and document it. When it merges, the canary doesn't have the feature until that night, so Netlify's build of `main` may fail until then. That's acceptable: Netlify keeps serving the last good deploy, and the nightly canary's hook rebuilds. Two things keep it tidy: a failed production build notifies the maintainer, and the canary workflow can be run by hand when waiting a night matters. If this happens more than occasionally, say so in `measures.md`; it's a signal about the canary's schedule.

`ascribe build` needs no `git` history (decision 7). Check it: the production build must pass with the history removed.

### Previews: built in Actions

On a pull request that touches `docs/`, `site/`, or the packages, a workflow builds the site with that pull request's Ascribe (phase 4's script) and deploys it to Netlify as a preview named for the pull request, then puts the preview's address in the job summary. Netlify's own deploy previews are turned off, so a pull request has one preview, and it's the right one.

The preview deploy needs two repository secrets (a Netlify access token and the site's id). Pull requests from forks don't get secrets, so they get the build check and the review report without a preview; the summary says so.

A preview is built with its own address as `site`, so its links are right. Check that the integration's agreement check between `astro.config.mjs` and `[consumer]` allows that; if it doesn't, stop and report.

### What the owner sets up

The Netlify site (linked to the repository, base directory `site/`, deploy previews off, failure notifications on), a build hook (stored as a secret for the canary workflow), and the two secrets for previews. The pull request says exactly what to set, and isn't merged until it's done. The site's address replaces phase 4's placeholder in `[consumer]`.

### Our CI runs our recipes

The review report workflow follows the recipe the review guide gives users: Ascribe installed from npm (`next`), a full clone, `ascribe diff --format html`, the upload, the summary link. It also runs for `docs/` when a pull request touches it. Where the recipe had to change to work here, change the guide.

The exception is decision 4's: a job that has to judge a pull request's own Ascribe (the docs check in phase 1, the preview) uses the checkout's binary.

### Stable addresses

A page's address comes from its file path. Don't move a published page without a reason; when one moves, add a line to `site/public/_redirects`. A test fails when a redirect's target isn't a page of the site. Each diagnostic keeps a stable address (`…/reference/diagnostics/#asc036-link-target-missing`), since the editor and `check`'s JSON link to them.

### Links to the site

The README, each package's README, the extension's manifest and README, and the binary's help and messages link to the site's pages, not to files under `docs/`. The address is written in one place per language (a constant in the binary, a phrase in the docs, a field in a manifest), each checked against `[consumer]` by a test.

### The measures

Record in `measures.md`: the production build's time on Netlify, split into installing, `ascribe build`, and Astro; phase 4's time to a working site; and the issues filed so far.

## Tasks

1. `site/netlify.toml`, and the production build passing on Netlify, with history removed locally as a check.
2. The build hook step in the canary workflow.
3. The preview workflow.
4. The review workflow, following the published recipe, for `examples/quill` and `docs/`.
5. Redirects and their test; the links and their tests; the diagnostics' link base, or the note.
6. `RELEASING.md`: that the site follows `main` through the canary, and what to check on it after a release. `measures.md`.

## Out of scope

A custom domain; a second host; a GitHub Action for users; Netlify's forms, functions, or analytics; versioned docs.

## Acceptance criteria

- The site is reachable at its Netlify address, built by Netlify from npm, with no Rust and no secret in that build.
- A docs change merged to `main` is live within a few minutes; a change that needs a newer canary is live after the next canary, and the site stays up in between.
- A pull request from a branch of this repository gets one preview address in its summary, built with its own Ascribe.
- The review workflow installs Ascribe as the guide says to.
- No README, manifest, or message links to a docs file on GitHub where a page on the site exists.

## Verify

```sh
cd site && npm ci && npm run build && npm test
cargo test --workspace --locked
pnpm format:check && pnpm lint && pnpm test
```

## Commits

1. "Build the docs site on Netlify"
2. "Rebuild the docs site after each canary"
3. "Preview the docs site on pull requests"
4. "Run the review report from npm, as the guide says"
5. "Link to the docs site"
