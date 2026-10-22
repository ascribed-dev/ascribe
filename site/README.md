# site/

Ascribe's website: a landing page at the root, Why Ascribe at `/why/`, and Credits at `/credits/`, from `home/`, the user docs, `../docs`, under `/docs/`, and the changelog, `../CHANGELOG.md` with the entries in `../changes/` not yet gathered into it, at `/changelog/`. Plain Astro with `@ascribed/astro`, building both projects into one site, and no documentation theme. It's set up as a user's site is, following the [getting-started](https://ascribed-dev.com/go/getting-started/) and [Astro](https://ascribed-dev.com/docs/astro/) guides: its own folder and lockfile, outside the pnpm workspace, with Ascribe installed from npm.

```
astro.config.mjs            ascribe({ projects: [docs, home, demo] }); routing as in each project's [consumer]; the sitemap's exclusions
home/                       the site's own pages, an Ascribe project: ascribe.toml (a page type, the pages' widgets, the screenshots' `scheme`, the docs' release tables) and content/
home/content/screenshots/   the pictures of review and the editor, light and dark, each with the release it was taken at (see Screenshots)
demo/                       the page the landing page demonstrates, an Ascribe project of its own, which the landing page draws beside its source; it has no address
netlify.toml                production's build on Netlify
netlify/edge-functions/     content negotiation: a page asked for as Markdown answers with its .md, and the .md's headers
public/*.png, favicon.*     the mark's favicons, touch icon, and social card, generated (../design/README.md)
public/cards/               a social card for each page outside the docs, generated (../design/README.md)
src/assets/logo.svg         the header's mark and wordmark, generated (../design/README.md)
src/content.config.ts       a collection per project, `docs`, `home`, and `demo`, each with the schema ascribe build generates, and `changelog`
src/changelog.ts            the changelog's collection, how its links and headings are pointed, and how its entries are collapsed and grouped
src/demo.ts                 where the landing page's demonstration draws demo/'s page: inside its widget, after the source
src/site.ts                 the site's name, repository, the docs' route, and the site's own pages left out of the sitemap
src/pages/[...slug].astro   a route per page of home/, at the root, through render() so its images are Astro's, with demo/'s page drawn in each demonstration (src/components/Demonstrations.astro)
src/pages/docs/[...slug].astro  a route per docs page: the docs' base path plus its entry id
src/pages/changelog.astro   the changelog, in the frame of home/'s pages
src/pages/404.astro         the 404 page, in the docs' frame
src/layouts/                the shell (the head, with each project page's from its record; header; footer; the docs' sidebar and search), a docs page (title, availability, edit link, contents), and a page of home/ (wide, no title of its own)
src/components/             search, the sidebar, the table of contents, the page's availability badge, and a page of home/ with its demonstrations
src/styles/site.css         light and dark, following the system, the element library's theme, and the landing page's widgets; its colors and type are generated from ../design/tokens.toml
src/theme.ts                the theme-color for each scheme, read from site.css
scripts/follow-next.mjs     production's extra step: the newest canary, without saving
scripts/checkout.mjs        a build with this checkout's Ascribe, for local work and previews
scripts/screenshot.ts       the review picture: a pull request on the docs in astro dev (see Screenshots)
test/                       links and anchors, the projects, the demonstration, content negotiation, the sitemap and robots.txt, and the built site in Chromium
test/fixtures/sidebar.json  the docs' sidebar as the site publishes it
test/fixtures/crawled/      a small built site with a page that says noindex, for the sitemap's rules
```

## Publishing

The site is at <https://ascribed-dev.com>, the address in each project's `[consumer]`, and the docs at <https://ascribed-dev.com/docs/>, their `base-path`.

- **Production** is Netlify's own build of `main`, with `site/` as its base directory and `netlify.toml` saying how: the three commands below. It runs when `main` changes `docs/`, `site/`, a file the docs take code examples from (`[sources.code]` in `docs/ascribe.toml`), or `CHANGELOG.md`, `changes/`, or the script that gathers them (`scripts/release/changes.ts`), and when the canary workflow calls the site's build hook after publishing a canary. A build that fails, such as one of a page documenting a feature the canary doesn't have yet, leaves the last good deploy up; the next canary's build hook rebuilds it.
- **A pull request's preview** is built by the **Site** workflow with the pull request's own Ascribe, and deployed with Netlify's CLI as a draft at `https://pr-<number>--ascribe-docs.netlify.app` (the Netlify site's name, `ascribe-docs`, is in the workflow), which the run's summary links. It's built with that address as `site` in both projects' `[consumer]`, which the Astro integration gives Astro, and as `home/`'s `site` phrase. Netlify's own deploy previews are off. Pull requests from forks get the build and its tests, and no preview.

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

- `test/projects.test.ts`: the docs' route is their base path; every permalink `home/`'s pages link to is one the docs publish, by `ascribe permalinks verify`, which also fails on a retired name; and the landing page's install commands are getting started's.
- `test/negotiation.test.ts`: the edge function answers a docs page asked for as Markdown with its `.md`, passing on the `.md` file's `x-robots-tag` and `link`, and a page of `home/`, which has none, with the page.
- `test/changelog.test.ts`: the changelog's rules for links, anchors, and collapsed entries, on Markdown made in the test, with no build.
- `test/links.test.ts`: every link and anchor between the built pages lands, the changelog's included.
- `test/crawlers.test.ts`: the sitemap lists every built page that doesn't say `noindex`, other than a permalink's forwarding page, the 404 page, and `unindexed` in `src/site.ts`, and nothing else; a page that says `noindex` is in neither the sitemap nor `llms.txt`; `robots.txt` names the sitemap; and the changelog has its canonical link. No docs page is marked `listing: unlisted`, so the rules are also run on `test/fixtures/crawled/`, which has such a page.
- `test/e2e.test.ts`, in Chromium (`ASCRIBE_CHROMIUM`, or `/opt/pw-browsers/chromium`, or Playwright's own: `npx playwright-core install chromium`): the landing page, its one heading in the hero, its call to action above the fold, its tools, its mark in the reader's scheme, and its link to getting started; each pillar, with its split bar and its proofs beside their examples, and every link in them a permalink that lands; the audiences, each linking to a pillar and to where it starts; the favicons, touch icon, and social card resolve, and the card's address is absolute; each page outside the docs names its own card, which is served; one `theme-color` per scheme, each the page's background; the header's mark links home and has an accessible name; variants switch and the choice holds on the next page; the availability badge on a page that has one; notes and steps; links between pages and glossary links; the sidebar, group by group and page by page, against `test/fixtures/sidebar.json`; search, with the network blocked; the edit link; the 404 page; the menu at phone width; the footer, on every kind of page, with its links to Credits, the changelog, the license, and the repository, and the changelog's link to a docs page, its entries opening one at a time and all at once, its area counts, Behavior changes only, and its contents, beside it and, at phone width, above it; Why Ascribe, from the header and the landing page, with its examples, its marks, its link to the comparison, and its screenshots, each the reader's scheme's; and Credits, from the footer and the landing page, with comrak's release and no sideways scroll at phone width.

## Addresses

A docs page's address is `/docs/` and its file's path under `docs/content/`, so moving a page moves its address, and nothing forwards from the old one. What links in from outside the docs, the binary's help and diagnostics, the editor's links, and the published READMEs, uses a [permalink](https://ascribed-dev.com/docs/writing/directives/#permalinks) instead, such as `/go/ASC036/`, which forwards to wherever its page or heading is, so moving a page that has one breaks none of them.

## Navigation

The docs' sidebar is their navigation: `../docs/ascribe.nav` orders the sections, and the pages where the default order isn't the site's, and labels the sections whose own page's title isn't the sidebar's ([Put your pages in order](https://ascribed-dev.com/docs/navigation/order/)). `src/components/Sidebar.astro` draws it with `sidebar()` from `@ascribed/astro/content`, with a nested section's pages right after its own page. A page added to `docs/content/` is in the sidebar where the default order puts it, and `ascribe check` reports a name in `ascribe.nav` that isn't a page. `test/e2e.test.ts` holds the published sidebar to `test/fixtures/sidebar.json`, so adding, moving, or relabeling a page changes the fixture with it.

## The site's own pages

`home/` is a second Ascribe project, at the site's root: one page type, a hero widget (`@hero-banner {status=early}:` around the page's heading and first paragraph, drawn by `src/styles/site.css` with no script), `content/index.md`, the landing page, `content/why.md`, Why Ascribe, and `content/credits.md`, Credits, whose heroes have no status. The landing page is laid out by widgets of its own, each described in `home/ascribe.toml` and drawn by `src/styles/site.css`: the hero's `@tool-list` and `@call-to-action`, a `@story-pillar` under each of the story's pillars' headings, with a `@pillar-proof` and the `@proof-example` beside it, the `@audience-list` of `@audience-entry`s, and the `@quiet-pillar` that closes it. Its mark, beside the headline, is a picture for each scheme in `home/content/images/`, generated (`../design/README.md`). It copies the docs' release tables and `[sources.code]`, so a claim there can be marked as the docs mark one; `scripts/docs-site/facts.test.ts` fails until the copies agree. It publishes nothing for agents.

Credits names who builds Ascribe, the projects it's built on, and where its ideas came from. `scripts/docs-site/credits.test.ts`, in the repository's tests, holds it to what it's about: each project it names as a dependency is one, in a `Cargo.toml` or a `package.json`, so a dependency that's removed fails until the page follows; comrak's release, the `comrak-release` phrase, is the one in `../crates/comrak-ascribe/FORK.md`; and each link to a file of the repository on GitHub, which `test/links.test.ts` doesn't follow, leads to the file and the heading. Who builds it nothing can test, so the maintainer reads the page when it changes.

A link into the docs is to a permalink, written `{site}/go/<name>/` with the `site` phrase: a link that starts with `/` names a file of `home/` itself, which `ascribe check` would report. `test/links.test.ts` follows it in the built site, and `test/projects.test.ts` runs `ascribe permalinks verify` on its name.

## The changelog

`/changelog/` is `../CHANGELOG.md`, which stays where it is and isn't an Ascribe page. The `changelog` collection (`src/changelog.ts`) reads it from the repository at build time, which Netlify's clone has, and puts the entries in `../changes/` under their areas' headings in its unreleased section as a release would gather them (`withChanges` in `../scripts/release/changes.ts`, which the site imports, so `main`'s unreleased changes are on the page before a release), and renders it with Astro's Markdown and the site's settings; in `astro dev` an edit to either reloads it. The page then points its links, which are written relative to the repository's root, as GitHub reads them:

- a docs page's source, a `.md` file under `docs/content/`, becomes that page's route on this site, with its fragment kept. The route is the one `src/pages/docs/[...slug].astro` gives the page, from the `docs` collection, so a page the docs' build didn't publish has none;
- any other file or folder in the repository becomes its address on GitHub, on `main`, from `repo` in `src/site.ts`;
- an address, or a heading of the changelog, stays as it is.

Anything else, a docs file that isn't a published page, or a file that isn't in the repository, fails the build, naming the line it's on, counted in `CHANGELOG.md` with the entries gathered. (`pnpm test` checks an entry's links to files when it's added; see `changes/README.md`.) `test/links.test.ts` then checks each fragment like any page's.

Each entry shows only what it says first until it's opened (`collapseEntries`): a `<details>` whose `<summary>` is its bold sentence, or, for a fix or an entry whose bold text starts a sentence, its first sentence (decision #382c, in `../project-docs/decisions/`), with a badge on a behavior change. Each release and each of its areas is a `<section>`, and an area's heading counts its entries. **Expand all**, **Collapse all**, and **Behavior changes only** above the releases need the page's script, and stay hidden without it; the script also opens an entry that a link leads into, and every entry for printing and, in a browser without the `beforematch` event, when find-in-page is opened from the keyboard: Chrome and Firefox open a closed `<details>` that find-in-page matches in on their own. The contents list each release and the areas of the newest two, beside the page where they fit and above it, closed, below 60rem.

Each release's heading is anchored by `v` and its version, its tag (`/changelog/#v0.2.0`), so a release's notes can link to it; a heading inside a release is anchored by both (`#v0.2.0-the-language`). The unreleased section, `## Unreleased` or a version marked `(unreleased)`, is `#unreleased`, and says under its heading that it's in `main` and not yet released.

The footer links to it, on every page, beside the license and the repository: the header is for the site's main pages, Docs and Why Ascribe, and GitHub. On a docs page at phone width, where the menu and search fill the header, Why Ascribe is on the site's own pages only; and below 26rem, those pages' header leaves out GitHub to fit Docs and Why Ascribe beside the mark.

## Search engines

Each page of a project, the docs' and home/'s, has everything its head says about it from its record in its build's `_ascribe/pages.json`, through `@ascribed/astro/Head.astro` in `src/layouts/Base.astro`: its `<title>`, description, canonical link, robots rule, link to its Markdown version, link preview, and breadcrumbs, with the site's name and its social card passed in. The layout keeps the rest of the head: the icons, the theme color, and `twitter:card`. `test/head.test.ts` holds each kind of page's head to the tags it had when the layout wrote them, `test/fixtures/head.json`. A docs page marked `listing: unlisted` says `noindex`, and is in neither the sitemap nor `llms.txt`. The changelog and the 404 page have no record and write their own title, description, and link preview, and the changelog its canonical link.

The integration writes one `sitemap.xml` and one `robots.txt` at the site's root, from the three projects together ([the head, sitemap, `robots.txt`, and headers](https://ascribed-dev.com/go/search-engines/)): the docs' listed pages, home/'s, and every other page Astro built, the changelog among them, but not a permalink's forwarding page, the 404 page, or `unindexed` in `src/site.ts`. A page of the site's own that should say `noindex` writes it and goes in `unindexed`, since the integration reads no HTML. The demo's page has no route, so it isn't in the sitemap. There's no `llms.txt` at the root: the delivery spec's checker finds the docs' at `/docs/llms.txt` from there too, and `home/` publishes nothing for agents. The Report workflow's `--site` stays at `/docs/`: the delivery spec's checks are about the docs' pages, and at the root they add only that `home/`'s pages aren't in the docs' `llms.txt` ([decision 32](https://github.com/ascribed-dev/ascribe/blob/5b22286fd2f370e796219e68a7c9d7cdb2ba8acc/project-docs/search-and-answers/README.md#decisions) of the search and answers plan).

## Search

[Pagefind](https://pagefind.app) indexes `dist/` after `astro build`, and the index is served with the site, so search needs no service. Only a docs page's `<article>` (`data-pagefind-body`) is indexed, so the pages of `home/` aren't, and the search box is on docs pages only. `astro dev` has no index; build and `npm run preview` to search.

## Screenshots

What the built pages can't show, the editor and review, is shown in pictures: the site's own, `review` and `editor`, in `home/content/screenshots/`, and the docs' scenes of the editor, the site, and the report in `../docs/content/images/`, each a PNG for light and for dark, and a record, `<name>.json`, of the release it was taken at. `PICTURES` in `../scripts/docs-site/screenshots.ts` names them all. A page shows them as two images marked `{scheme=light}` and `{scheme=dark}`, and `src/styles/site.css` shows the reader's.

Each is taken by a script, and retaken with one command at the repository's root, which also writes its record from the current line of `[releases.ascribe]` in `docs/ascribe.toml`:

```sh
node scripts/docs-site/screenshots.ts review   # or another picture's name, or none for every one
```

- **review**, by `scripts/screenshot.ts`: the docs and this site as committed, copied into a repository of their own with a change on a branch, in `astro dev`, with a pull request's comments from the end-to-end suite's fake GitHub (`../examples/astro-site/test/e2e/fake-github.ts`). It needs the site installed with the Ascribe to show (`npm run build:checkout`, with `ASCRIBE_BIN`), and a POSIX system.
- **editor**, by `../packages/vscode/scripts/screenshot.ts`: a copy of `../examples/quill` in VS Code, at the version the script names, with the extension built from the checkout, a link written as an address, and its quick fix open. It needs `ASCRIBE_BIN`. On Linux it draws VS Code on a virtual screen with Xvfb (`xvfb-run`); elsewhere VS Code opens a window for a few seconds for each scheme, and quits when the picture is taken.
- **review-panel**, by `scripts/screenshot.ts review-panel`: the same page as **review**, cropped to the review panel, for the docs.
- **the site's and the report's scenes**, `html-report`, `elements`, `starter-sidebar`, and `versions`, by `../examples/astro-site/scripts/screenshot.ts`: each a copy of that example, as its end-to-end tests make one, built and served by Astro in Chromium, its starter layout given the few rules a site's stylesheet would (both schemes, a measure, the sidebar's indent); and the report, `ascribe diff --format html` on a change to the example's setup page that changes, adds, removes, and moves a block. They need the example installed and `ASCRIBE_BIN`.
- **the docs' scenes of the editor**, by the editor's script: each opens a workspace of its own, made from an example, and `../packages/vscode/scripts/scenes.ts`, run in VS Code's extension host, stages it through the extension's API and VS Code's commands, a step at a time, while the script takes the picture of each step. The Review view's pull request is on the integration tests' fake GitHub. They need `ASCRIBE_BIN`, and the status bar's scene, which starts `ascribe` through a shell script to catch it starting and failing, a POSIX system.

Each fixes what would change between runs (the clock, the window's size, blinking and fading), so a retake with nothing changed writes the same bytes on the same machine. Take them on whatever machine you have: fonts and a window's frame differ a little between systems, and nothing on the page depends on it. `scripts/docs-site/screenshots.test.ts`, in the repository's tests, fails when a picture's release is more than one minor release behind the current one, and names the command.
