# site/

Ascribe's user docs, `../docs`, as a website: plain Astro with `@ascribed/astro`, no documentation theme. It's set up as a user's site is, following the [getting-started](https://ascribed-dev.com/getting-started/) and [Astro](https://ascribed-dev.com/guides/astro/) guides: its own folder and lockfile, outside the pnpm workspace, with Ascribe installed from npm.

```
astro.config.mjs            ascribe({ project: "../docs", build: "site" }); routing as in ../docs/ascribe.toml's [consumer]
netlify.toml                production's build on Netlify
public/_redirects           pages that moved, from their old address to their new one
src/content.config.ts       the collection, with the schema ascribe build generates in ../docs
src/nav.ts                  the sidebar: every page, grouped and in order
src/pages/[...slug].astro   a route per page: the base path plus its entry id
src/pages/404.astro         the 404 page
src/layouts/                the shell (header, sidebar) and a docs page (title, availability, edit link, contents)
src/components/             search, the sidebar, the table of contents, the page's availability badge
src/styles/site.css         light and dark, following the system, and the element library's theme
scripts/follow-next.mjs     production's extra step: the newest canary, without saving
scripts/checkout.mjs        a build with this checkout's Ascribe, for local work and previews
test/                       navigation, links and anchors, redirects, and the built site in Chromium
```

## Publishing

The site is at <https://ascribed-dev.com>, the address in `[consumer]`.

- **Production** is Netlify's own build of `main`, with `site/` as its base directory and `netlify.toml` saying how: the three commands below. It runs when `main` changes `docs/`, `site/`, or a file the docs take code examples from (`[sources.code]` in `docs/ascribe.toml`), and when the canary workflow calls the site's build hook after publishing a canary. A build that fails, such as one of a page documenting a feature the canary doesn't have yet, leaves the last good deploy up; the next canary's build hook rebuilds it.
- **A pull request's preview** is built by the **Site** workflow with the pull request's own Ascribe, and deployed with Netlify's CLI as a draft at `https://pr-<number>--ascribe-docs.netlify.app` (the Netlify site's name, `ascribe-docs`, is in the workflow), which the run's summary links. It's built with that address as `site`, in `astro.config.mjs` and `[consumer]` alike. Netlify's own deploy previews are off. Pull requests from forks get the build and its tests, and no preview.

## Building

Production, as a host builds it, from npm alone (no Rust, nothing from the workspace, no `git` history):

```sh
npm ci
npm run follow-next   # the Ascribe packages at `next`, without saving
npm run build         # astro build, then Pagefind's index of dist/
```

**This is the one place the site's build differs from a user's.** A user's lockfile pins the Ascribe they installed, and they move it on purpose. This site follows `main`, so after `npm ci` it installs the canary that `next` names now: `@ascribed/astro`, `@ascribed/cli`, and `@ascribed/elements`, all at the version `next` gives for `@ascribed/astro`, which pins the others to its own. The lockfile and everything else in it stay as they are. Plain `npm ci && npm run build` builds with the canary in the lockfile. The **Site from npm** workflow builds this way, in a copy of the repository's committed files alone, after each canary and when `main` changes the site, the docs, or a file the docs take code examples from.

With this checkout's Ascribe, for working on Ascribe and its docs together, and for a pull request's preview:

```sh
cargo build -p tessera-cli          # or set ASCRIBE_BIN
pnpm install                        # at the repository's root
npm ci
npm run build:checkout
```

`build:checkout` builds and packs the workspace's `cli`, `elements`, `review`, and `astro`, installs the packs without saving, and builds with `ASCRIBE_BIN`. `npm ci` puts the registry's packages back. The **Site** workflow runs it on pull requests that touch `docs/`, `site/`, or Ascribe.

## Checking

```sh
npm run typecheck
npm test              # after a build
```

- `test/nav.test.ts`: every published page is in `src/nav.ts`, and it names nothing else.
- `test/links.test.ts`: every link and anchor between the built pages lands.
- `test/redirects.test.ts`: each redirect in `public/_redirects` leads from an address that isn't a page to one that is.
- `test/e2e.test.ts`, in Chromium (`ASCRIBE_CHROMIUM`, or `/opt/pw-browsers/chromium`, or Playwright's own: `npx playwright-core install chromium`): variants switch and the choice holds on the next page; the availability badge on an unreleased page; notes and steps; links between pages and glossary links; the sidebar; search, with the network blocked; the edit link; the 404 page; and the menu at phone width.

## Addresses

A page's address is its file's path under `docs/content/`, so moving a page moves its address. Don't move a published page without a reason; when one moves, add a line to `public/_redirects`. Each diagnostic's address, `/reference/diagnostics/#asc036-link-target-missing`, stays the same, so tools can link to a diagnostic.

## Navigation

Ascribe's content model has no page order or grouping, so the site lists its pages in `src/nav.ts`, by source file, and the test keeps the list complete. A page added to `docs/content/` fails `npm test` until it's in the list.

For the content model to give a site its navigation, it would need:

- **an order and a group for each page**, either in `ascribe.toml` (a list of groups, each a label and its pages' files) or in each page's frontmatter (a group and a weight);
- **checking**: `ascribe check` reporting a published page that's in no group, and a group naming a file that isn't a page, which is what the test does now;
- **the result in the outputs**: the groups, in order, with each page's title and route, in the JSON output and as a module the site can import, so a layout renders the sidebar and previous and next links without its own list;
- **a label for a page in the navigation**, where its title is too long (`ascribe.toml reference`).

## Search

[Pagefind](https://pagefind.app) indexes `dist/` after `astro build`, and the index is served with the site, so search needs no service. Only the page's `<article>` (`data-pagefind-body`) is indexed. `astro dev` has no index; build and `npm run preview` to search.
