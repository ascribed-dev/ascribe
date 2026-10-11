# site/

Ascribe's website: a landing page at the root and Why Ascribe at `/why/`, from `home/`, the user docs, `../docs`, under `/docs/`, and the changelog, `../CHANGELOG.md`, at `/changelog/`. Plain Astro with `@ascribed/astro`, building both projects into one site, and no documentation theme. It's set up as a user's site is, following the [getting-started](https://ascribed-dev.com/go/getting-started/) and [Astro](https://ascribed-dev.com/docs/guides/astro/) guides: its own folder and lockfile, outside the pnpm workspace, with Ascribe installed from npm.

```
astro.config.mjs            ascribe({ projects: [docs, home, demo] }); routing as in each project's [consumer]
home/                       the site's own pages, an Ascribe project: ascribe.toml (a page type, the pages' widgets, the screenshots' `scheme`, the docs' release tables) and content/
home/content/screenshots/   the pictures of review and the editor, light and dark, each with the release it was taken at (see Screenshots)
demo/                       the page the landing page demonstrates, an Ascribe project of its own, which the landing page draws beside its source; it has no address
netlify.toml                production's build on Netlify
netlify/edge-functions/     content negotiation: a page asked for as Markdown answers with its .md
public/_redirects           pages that moved, from their old address to their new one
public/*.png, favicon.*     the mark's favicons, touch icon, and social card, generated (../design/README.md)
public/cards/               a social card for each page outside the docs, generated (../design/README.md)
src/assets/logo.svg         the header's mark and wordmark, generated (../design/README.md)
src/content.config.ts       a collection per project, `docs`, `home`, and `demo`, each with the schema ascribe build generates, and `changelog`
src/changelog.ts            the changelog's collection, and how its links and headings are pointed
src/demo.ts                 where the landing page's demonstration draws demo/'s page: inside its widget, after the source
src/nav.ts                  the docs' sidebar: every page, grouped and in order
src/site.ts                 the site's name, repository, and the docs' route
src/pages/[...slug].astro   a route per page of home/, at the root, through render() so its images are Astro's, with demo/'s page drawn in each demonstration (src/components/Demonstrations.astro)
src/pages/docs/[...slug].astro  a route per docs page: the docs' base path plus its entry id
src/pages/changelog.astro   the changelog, in the frame of home/'s pages
src/pages/404.astro         the 404 page, in the docs' frame
src/layouts/                the shell (header; the docs' sidebar and search), a docs page (title, availability, edit link, contents), and a page of home/ (wide, no title of its own)
src/components/             search, the sidebar, the table of contents, the page's availability badge, and a page of home/ with its demonstrations
src/styles/site.css         light and dark, following the system, the element library's theme, and the landing page's widgets; its colors and type are generated from ../design/tokens.toml
src/theme.ts                the theme-color for each scheme, read from site.css
scripts/follow-next.mjs     production's extra step: the newest canary, without saving
scripts/checkout.mjs        a build with this checkout's Ascribe, for local work and previews
scripts/screenshot.ts       the review picture: a pull request on the docs in astro dev (see Screenshots)
test/                       navigation, links and anchors, redirects, the projects, the demonstration, content negotiation, and the built site in Chromium
```

## Publishing

The site is at <https://ascribed-dev.com>, the address in each project's `[consumer]`, and the docs at <https://ascribed-dev.com/docs/>, their `base-path`.

- **Production** is Netlify's own build of `main`, with `site/` as its base directory and `netlify.toml` saying how: the three commands below. It runs when `main` changes `docs/`, `site/`, a file the docs take code examples from (`[sources.code]` in `docs/ascribe.toml`), or `CHANGELOG.md`, and when the canary workflow calls the site's build hook after publishing a canary. A build that fails, such as one of a page documenting a feature the canary doesn't have yet, leaves the last good deploy up; the next canary's build hook rebuilds it.
- **A pull request's preview** is built by the **Site** workflow with the pull request's own Ascribe, and deployed with Netlify's CLI as a draft at `https://pr-<number>--ascribe-docs.netlify.app` (the Netlify site's name, `ascribe-docs`, is in the workflow), which the run's summary links. It's built with that address as `site`, in `astro.config.mjs` and both projects' `[consumer]` alike, and as `home/`'s `site` phrase. Netlify's own deploy previews are off. Pull requests from forks get the build and its tests, and no preview.

## Building

Production, as a host builds it, from npm alone (no Rust, nothing from the workspace, no `git` history):

```sh
npm ci
npm run follow-next   # the Ascribe packages at `next`, without saving
npm run build         # astro build, then Pagefind's index of dist/
```

**This is the one place the site's build differs from a user's.** A user's lockfile pins the Ascribe they installed, and they move it on purpose. This site follows `main`, so after `npm ci` it installs the canary that `next` names now: `@ascribed/astro`, `@ascribed/cli`, and `@ascribed/elements`, all at the version `next` gives for `@ascribed/astro`, which pins the others to its own. The lockfile and everything else in it stay as they are. Plain `npm ci && npm run build` builds with the canary in the lockfile. The **Site from npm** workflow builds this way, in a copy of the repository's committed files alone, after each canary and when `main` changes the site, the docs, a file the docs take code examples from, or the changelog.

With this checkout's Ascribe, for working on Ascribe and its docs together, and for a pull request's preview:

```sh
cargo build -p ascribe-cli          # or set ASCRIBE_BIN
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

- `test/nav.test.ts`: every published docs page is in `src/nav.ts`, and it names nothing else.
- `test/projects.test.ts`: the docs' route is their base path; every permalink `home/`'s pages link to is one the docs publish, by `ascribe permalinks verify`, which also fails on a retired name; and the landing page's install commands are getting started's.
- `test/negotiation.test.ts`: the edge function answers a docs page asked for as Markdown with its `.md`, and a page of `home/`, which has none, with the page.
- `test/changelog.test.ts`: the changelog's rules for links and anchors, on Markdown made in the test, with no build.
- `test/links.test.ts`: every link and anchor between the built pages lands, the changelog's included.
- `test/redirects.test.ts`: each redirect in `public/_redirects` leads from an address that isn't a page to one that is.
- `test/e2e.test.ts`, in Chromium (`ASCRIBE_CHROMIUM`, or `/opt/pw-browsers/chromium`, or Playwright's own: `npx playwright-core install chromium`): the landing page, its one heading in the hero, and its link to getting started; the favicons, touch icon, and social card resolve, and the card's address is absolute; each page outside the docs names its own card, which is served; one `theme-color` per scheme, each the page's background; the header's mark links home and has an accessible name; variants switch and the choice holds on the next page; the availability badge on a page that has one; notes and steps; links between pages and glossary links; the sidebar; search, with the network blocked; the edit link; the 404 page; the menu at phone width; the header's link to the changelog, and the changelog's link to a docs page; and Why Ascribe, from the header and the landing page, with its examples, its marks, its link to the comparison, and its screenshots, each the reader's scheme's.

## Addresses

A docs page's address is `/docs/` and its file's path under `docs/content/`, so moving a page moves its address. Don't move a published page without a reason; when one moves, add a line to `public/_redirects`. What links in from outside the docs, the binary's help and diagnostics and the editor's links, uses a [permalink](https://ascribed-dev.com/docs/reference/directives/#permalinks) instead, such as `/go/ASC036/`, which forwards to wherever its page or heading is, so moving a page that has one breaks none of them.

## Navigation

Ascribe's content model has no page order or grouping, so the site lists its pages in `src/nav.ts`, by source file, and the test keeps the list complete. A page added to `docs/content/` fails `npm test` until it's in the list.

For the content model to give a site its navigation, it would need:

- **an order and a group for each page**, either in `ascribe.toml` (a list of groups, each a label and its pages' files) or in each page's frontmatter (a group and a weight);
- **checking**: `ascribe check` reporting a published page that's in no group, and a group naming a file that isn't a page, which is what the test does now;
- **the result in the outputs**: the groups, in order, with each page's title and route, in the JSON output and as a module the site can import, so a layout renders the sidebar and previous and next links without its own list;
- **a label for a page in the navigation**, where its title is too long (`ascribe.toml reference`).

## The site's own pages

`home/` is a second Ascribe project, at the site's root: one page type, a hero widget (`@hero-banner {status=early}:` around the page's heading and first paragraph, drawn by `src/styles/site.css` with no script), `content/index.md`, the landing page, and `content/why.md`, Why Ascribe, whose hero has no status. It copies the docs' release tables and `[sources.code]`, so a claim there can be marked as the docs mark one; `scripts/docs-site/facts.test.ts` fails until the copies agree. It publishes nothing for agents.

A link into the docs is to a permalink, written `{site}/go/<name>/` with the `site` phrase: a link that starts with `/` names a file of `home/` itself, which `ascribe check` would report. `test/links.test.ts` follows it in the built site, and `test/projects.test.ts` runs `ascribe permalinks verify` on its name.

## The changelog

`/changelog/` is `../CHANGELOG.md`, which stays where it is and isn't an Ascribe page. The `changelog` collection (`src/changelog.ts`) reads it from the repository at build time, which Netlify's clone has, and renders it with Astro's Markdown and the site's settings; in `astro dev` an edit to it reloads it. The page then points its links, which are written relative to the repository's root, as GitHub reads them:

- a docs page's source, a `.md` file under `docs/content/`, becomes that page's route on this site, with its fragment kept. The route is the one `src/pages/docs/[...slug].astro` gives the page, from the `docs` collection, so a page the docs' build didn't publish has none;
- any other file or folder in the repository becomes its address on GitHub, on `main`, from `repo` in `src/site.ts`;
- an address, or a heading of the changelog, stays as it is.

Anything else, a docs file that isn't a published page, or a file that isn't in the repository, fails the build, naming the line of `CHANGELOG.md` it's on. `test/links.test.ts` then checks each fragment like any page's.

Each release's heading is anchored by `v` and its version, its tag (`/changelog/#v0.2.0`), so a release's notes can link to it; a heading inside a release is anchored by both (`#v0.2.0-the-language`). The unreleased section, `## Unreleased` or a version marked `(unreleased)`, is `#unreleased`, and says under its heading that it's in `main` and not yet released.

The header links to it. On a docs page at phone width, where the menu and search fill the header, the link is on the site's own pages only; and below 26rem, those pages' header leaves out GitHub to fit Docs, Why Ascribe, and Changelog beside the mark.

## Search

[Pagefind](https://pagefind.app) indexes `dist/` after `astro build`, and the index is served with the site, so search needs no service. Only a docs page's `<article>` (`data-pagefind-body`) is indexed, so the pages of `home/` aren't, and the search box is on docs pages only. `astro dev` has no index; build and `npm run preview` to search.

## Screenshots

What the built pages can't show, the editor and review, is shown in pictures, in `home/content/screenshots/`: `review` and `editor`, each a PNG for light and for dark, and a record, `<name>.json`, of the release it was taken at. A page shows them as two images marked `{scheme=light}` and `{scheme=dark}`, and `src/styles/site.css` shows the reader's.

Each is taken by a script, and retaken with one command at the repository's root, which also writes its record from `[versions] current` in `docs/ascribe.toml`:

```sh
node scripts/docs-site/screenshots.ts review   # or editor, or neither for both
```

- **review**, by `scripts/screenshot.ts`: the docs and this site as committed, copied into a repository of their own with a change on a branch, in `astro dev`, with a pull request's comments from the end-to-end suite's fake GitHub (`../examples/astro-site/test/e2e/fake-github.ts`). It needs the site installed with the Ascribe to show (`npm run build:checkout`, with `ASCRIBE_BIN`), and a POSIX system.
- **editor**, by `../packages/vscode/scripts/screenshot.ts`: a copy of `../examples/quill` in VS Code, at the version the script names, with the extension built from the checkout, a link written as an address, and its quick fix open. It needs `ASCRIBE_BIN` and Linux with Xvfb.

Both fix what would change between runs (the clock, the window's size, blinking and fading), so a retake with nothing changed writes the same bytes on the same machine; fonts differ between machines, so take them on Linux. `scripts/docs-site/screenshots.test.ts`, in the repository's tests, fails when a picture's release is more than one minor release behind the current one, and names the command.
