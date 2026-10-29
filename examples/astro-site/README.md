# examples/astro-site

A plain Astro site (no Starlight) built from a small Ascribe project with `@ascribed/astro` and `@ascribed/cli`. Its content covers the integrations most likely to break.

```
ascribe.toml                      base-path /docs/, trailing-slash never; the `site` build
content/
  index.md                        the root page: served at /docs
  Guides/My Setup.md              entry id guides/my-setup; page-level availability
  Guides/Teams/Share a loom.md    a nested section's page, in the sidebar below Guides
  Reference/Options.md            links to an explicit-id heading; an image beside the page
  _fragments/requirements.md      included by My Setup; its image sits beside the fragment
  downloads/loom.yaml             linked to, so published under _ascribe/files/
src/content.config.ts             the collection
src/pages/[...slug].astro         the route: base + entry id, and `index` at the base
src/pages/404.astro               the not-found page, which explains a permalink that isn't here
src/layouts/Docs.astro            the page, the sidebar from the project's navigation, its availability badge, its versions, and a table of contents
src/styles/docs.css               the layout's styles, which the 404 page imports too
test/e2e/                         the end-to-end tests
scripts/screenshot.ts             the docs' pictures of the site and the report, from copies of this example (../../site/README.md#screenshots)
```

`src/content.config.ts`, the route, the layout, and its stylesheet are what `npx @ascribed/astro init` writes for this project, from `packages/astro/starter/`: `packages/astro/test/init.test.ts` fails when they differ, and rewrites them with `ASCRIBE_BLESS=1`. Change the starter, not these. The layout's sidebar has no list of its own: `sidebar(await getCollection("docs"), { entry })` draws it from the project's navigation, the default order, since the project has no `ascribe.nav`.

| Risk | Where |
|---|---|
| A page includes a fragment whose image sits beside the fragment | `My Setup.md` includes `_fragments/requirements.md` |
| A heading with an explicit `@id`, and a link to it from another page | `## Weave configuration` (`@id: weave-config`), linked from `Options.md` |
| A partial variant selection: one group reduced to one arm, another left as tabs | build `site` selects `deployment = cloud`; the `pm` group stays tabs |
| An availability annotation | page-level `available` in the frontmatter, and `@available` on `## Streaming` |
| A permalink | `@id {permalink=true}: weave-config`, at `/docs/go/weave-config`, which with `?pm=pnpm` opens the `pnpm` tab; a name that isn't there gets the 404 page's explanation |
| A nested section | `Guides/Teams/`, whose page the sidebar lists after `My Setup.md`, one level deeper |
| Routes | links to `guides/my-setup` and to the root page, which only work if Ascribe's router and Astro's agree |

## Running it

```sh
pnpm install
# For local workspace development only, stage a native binary into the optional package:
cargo build -p ascribe-cli
ASCRIBE_BIN_DARWIN_ARM64="$PWD/target/debug/ascribe" pnpm --filter @ascribed/cli stage-native darwin-arm64
pnpm --filter @ascribed/cli build
pnpm --filter @ascribed/elements build
pnpm --filter @ascribed/astro build
pnpm --filter @ascribed/example-astro-site build     # or: astro build
pnpm --filter @ascribed/example-astro-site test:e2e
```

Use the matching target name and environment variable from `packages/cli/README.md`
on other platforms. The integration resolves the project's installed `@ascribed/cli`
native package by default; `binary` and `ASCRIBE_BIN` can override it.
`astro dev` rebuilds and refreshes when a source, asset, or `ascribe.toml`
changes, and returns 503 while an invalid source is being fixed.

`test:e2e` runs a real `astro build`, serves it with `astro preview`, and checks the built HTML in Chromium (`ASCRIBE_CHROMIUM`, or `/opt/pw-browsers/chromium`, or Playwright's own):

- the root page is at `/docs`, and the links between pages, including the one to the root, land;
- the explicit-id link lands on its heading;
- the fragment's image, and an image beside a page, are processed by Astro (a hashed `.webp` under `/docs/_astro/`) and keep their `width`;
- the reduced group shows one arm's content and the other group works as tabs;
- the page and section availability badges render, with the element library's stylesheet;
- a linked file is served at `/docs/_ascribe/files/`;
- every heading's id, and Astro's table of contents, equal the ids in `ascribe build --emit json`, at the route the JSON gives;
- the build fails on an Ascribe error and on routing that disagrees with `ascribe.toml`; the dev server serves `_ascribe/files/`; the plugin works under `unified()` too; and the root base with trailing slashes routes correctly.
- on a copy whose build is versioned, with two live lines of `self-managed` (`test/e2e/versions.test.ts`): 3.5's pages at `/docs/` and 3.4's at `/docs/3.4/`, each listing the other; a page only 3.5 has; an older line's sidebar, links, image, and files staying in its line, and every link landing; a permalink forwarding to the current line; no older-line pages in the sitemap and no older-line Markdown; and `astro dev` rebuilding both lines.
- on copies of the site with Astro's `base` at `/` (`test/e2e/projects.test.ts`), in a build and under `astro dev`: a home page at `/` with the docs at `/docs/`, their files, `llms.txt`, and permalinks there; a second project, `home/`, at `/` beside the docs, with a permalink from each and a `.ascribe/dev.json` for each; two entries for one project with two builds; and the four refusals: two projects at one base path, a page under another project's path, a permalink both declare, and an entry's name used twice.

`pnpm test` runs nothing here: the end-to-end test needs the compiler and a browser, so it has its own script (and its own job in `.github/workflows/js.yml`).
