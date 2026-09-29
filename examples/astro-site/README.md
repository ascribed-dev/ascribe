# examples/astro-site

A plain Astro site (no Starlight) built from a small Tessera project with `@tessera/astro`: phase 21's end-to-end slice. Its content covers the integrations most likely to break.

```
ascribe.toml                      base-path /docs/, trailing-slash never; the `site` build
content/
  index.md                        the root page: served at /docs
  Guides/My Setup.md              entry id guides/my-setup; page-level availability
  Reference/Options.md            links to an explicit-id heading; an image beside the page
  _fragments/requirements.md      included by My Setup; its image sits beside the fragment
  downloads/loom.yaml             linked to, so published under _ascribe/files/
src/content.config.ts             the collection
src/pages/[...slug].astro         the route: base + entry id, and `index` at the base
src/layouts/Docs.astro            the page, its availability badge, and a table of contents
test/e2e/                         the end-to-end tests
```

| Risk | Where |
|---|---|
| A page includes a fragment whose image sits beside the fragment | `My Setup.md` includes `_fragments/requirements.md` |
| A heading with an explicit `@id`, and a link to it from another page | `## Weave configuration` (`@id: weave-config`), linked from `Options.md` |
| A partial variant selection: one group reduced to one arm, another left as tabs | build `site` selects `deployment = cloud`; the `pm` group stays tabs |
| An availability annotation | page-level `available` in the frontmatter, and `@available` on `## Streaming` |
| Routes | links to `guides/my-setup` and to the root page, which only work if Tessera's router and Astro's agree |

## Running it

```sh
cargo build -p tessera-cli
pnpm install
pnpm --filter @tessera/elements build
pnpm --filter @tessera/astro build
pnpm --filter @tessera/example-astro-site build     # or: astro build
pnpm --filter @tessera/example-astro-site test:e2e
```

The integration finds the binary in `target/` (or `TESSERA_BIN`). `test:e2e` runs a real `astro build`, serves it with `astro preview`, and checks the built HTML in Chromium (`TESSERA_CHROMIUM`, or `/opt/pw-browsers/chromium`, or Playwright's own):

- the root page is at `/docs`, and the links between pages, including the one to the root, land;
- the explicit-id link lands on its heading;
- the fragment's image, and an image beside a page, are processed by Astro (a hashed `.webp` under `/docs/_astro/`) and keep their `width`;
- the reduced group shows one arm's content and the other group works as tabs;
- the page and section availability badges render, with the element library's stylesheet;
- a linked file is served at `/docs/_ascribe/files/`;
- every heading's id, and Astro's table of contents, equal the ids in `ascribe build --emit json`, at the route the JSON gives;
- the build fails on a Tessera error and on routing that disagrees with `ascribe.toml`; the dev server serves `_ascribe/files/`; the plugin works under `unified()` too; and the root base with trailing slashes routes correctly.

`pnpm test` runs nothing here: the end-to-end test needs the compiler and a browser, so it has its own script (and its own job in `.github/workflows/js.yml`).
