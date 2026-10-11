# Phase 4: The changelog

Part of [The website](README.md). Requires phase 2. `site/` only.

## Goal

`https://ascribed-dev.com/changelog/` is `CHANGELOG.md`, in the site's frame, with every link in it leading somewhere that exists.

## Context

- README decision 6: the file stays where it is and isn't an Ascribe page.
- `CHANGELOG.md`: its sections, and the three kinds of link it holds: to files under `docs/content/`, to other files in the repository, and to addresses outside.
- `RELEASING.md` and the packages' READMEs, which point at the file.
- `docs/ascribe.toml`'s `[sources.code]` comment: the four places that list which paths rebuild the site. The changelog is a fifth path.
- `site/test/links.test.ts`: how the built site's links are checked.
- The docs' `_ascribe/permalinks.json` and the site output's pages: what a path under `docs/content/` becomes.

## Design

The site renders the file with Astro's own Markdown, on a page in the marketing frame.

- **A link to a file under `docs/content/`** becomes that page's route, with its anchor kept. The route comes from what the docs' build wrote, not from a rule about paths copied into the site.
- **A link to any other file in the repository** becomes its address on GitHub, on `main`, built from `repo` in `site/src/site.ts`, where the site already keeps the repository's address. It isn't written a third time.
- **A link that fits neither,** to a docs file that isn't a published page, fails the site's build with the line it's on.
- **Each release's heading has an anchor** a release's notes can link to, and the unreleased section is shown as unreleased.

The header gains Changelog.

A change to `CHANGELOG.md` on `main` rebuilds the site: it's added wherever the paths that do are listed, and `scripts/docs-site/facts.test.ts` keeps them agreeing.

Nothing is copied. The page reads the file from the repository at build time, which Netlify's clone has.

## Tasks

1. The page, the rendering, and the three rules for links, with unit tests for each rule.
2. The header's link.
3. The rebuild paths, in the workflows and `site/netlify.toml`, with the test.
4. `site/test/links.test.ts` covers the changelog's links like any page's.
5. `site/README.md`.

## Out of scope

A page per release. A feed. Changing how the changelog is written.

## Acceptance criteria

- Every link on the built page lands: the site's link test passes with the page included.
- A link in `CHANGELOG.md` to a docs file that doesn't exist fails the site's build and names the line.
- `CHANGELOG.md` is unchanged by this phase, apart from its own entry if the site's changes need one.

## Verify

```sh
pnpm test
cd site && npm run build:checkout && npm run typecheck && npm test
```

## Commits

1. "Publish the changelog as a page of the site"
