# Phase 21: Astro end-to-end slice

**Track:** Web · **Start after:** 19, 20 · **Parallel with:** 26 · **Unblocks:** 16, 22, 25 (to finish)

## Goal

Prove the whole pipeline in a real Astro site as early as possible, with the smallest integration that works. The full editor experience (phase 16 onward) waits for this slice, because the integration failures it catches, such as asset paths, heading ids, and partial variant selection, are the ones snapshot tests of Tessera's own output miss.

## Read first

- [SPEC.md](../../SPEC.md): §9.4–§9.7.
- `project-docs/contracts/site-render.md`, `tests/render/`, and `project-docs/contracts/assets.md` (phase 02).
- Handoff notes from phases 19 and 20.
- The current Astro documentation for integrations, content collections, and markdown plugins.

## Deliverables

- `packages/astro`: a minimal `@tessera/astro`.
- `examples/astro-site`: a plain Astro site (no Starlight) that uses it.
- An end-to-end test in CI on Linux.

## Tasks

1. **Minimal integration.** `@tessera/astro`:
   - Runs `tessera build --emit site` for a configured build before Astro loads content. For this phase, it uses the locally built binary; npm distribution is phase 22.
   - Writes the generated Zod schema (phase 20), and exports a helper that defines the content collection over the build output with that schema.
   - Registers a markdown plugin implementing the site-render contract (heading ids and image attributes), so Astro keeps its own heading and table-of-contents handling. The plugin must pass every `tests/render/` fixture, the same fixtures phase 20's renderer passes.
   - Loads `@tessera/elements` (script and CSS) on pages that render Tessera content.
   - Fails the Astro build when Tessera reports errors.
2. **Sample site.** `examples/astro-site`: a layout that renders a page, its availability badge from frontmatter, and the elements.
3. **The slice's content.** A small project in the sample site covering exactly the risky integrations:
   - a page that includes a fragment whose image sits beside the fragment;
   - a heading with an explicit `@id`, and a link to it from another page;
   - a build with a partial variant selection that reduces one group to one arm and leaves another group as tabs;
   - an availability annotation.
4. **End-to-end test.** Build the site in CI and check the built HTML in a browser: the fragment's image loads (and Astro processed it), the explicit-id link lands on the heading, the reduced group shows one arm's content and the other group works as tabs, and the badge renders.

## Acceptance criteria

- [ ] The Astro plugin passes every `tests/render/` fixture.
- [ ] The end-to-end test passes in CI on Linux, checking each item in task 3.
- [ ] Heading ids in the built HTML match the page ids `tessera check` validated.

## Out of scope

- npm distribution of the binary, dev-mode rebuilding, and other platforms (phase 22).

## Notes

- Verify every Astro API you use against the current documentation and record the Astro version targeted. Don't rely on memory for integration hook names or loader APIs.

## Handoff notes

### What was built

- **`packages/astro`** (`@tessera/astro`, private): the integration (`src/index.ts`), `tesseraCollection` (`src/content.ts`, exported as `@tessera/astro/content`), the `<Elements />` component (`src/Elements.astro`), and the markdown plugin. The plugin is `src/attributes.ts` (the site-render contract's rules, once, as a pure function from a hast tree to a list of edits) with an adapter for each of Astro's markdown processors: `src/rehype.ts` (unified) and `src/satteri.ts` (Sätteri). Also `src/project.ts` (reads `ascribe.toml`; checks routing agreement), `src/binary.ts`, `src/run.ts`, and `src/files.ts` (`_ascribe/files/`). `packages/astro/README.md` documents the interface. `dist/` is git-ignored: `pnpm --filter @tessera/astro build` (and `--filter @tessera/elements build`) first.
- **`examples/astro-site`**: a plain Astro site over a small Tessera project (`ascribe.toml`, `content/`) with the four risky integrations of task 3, `base: "/docs"`, `trailingSlash: "never"`. Its README maps each risk to where it is.
- **The end-to-end test**, `examples/astro-site/test/e2e/` (`pnpm --filter @tessera/example-astro-site test:e2e`), and a manual-dispatch job for it, `astro`, in `.github/workflows/js.yml` (which stays `workflow_dispatch` only).
- **Questions Q151 to Q155** (below). No Rust changes.

### Interfaces later phases use

- **Phase 22:** everything above is the interface. What is left for it: the binary from npm (`src/binary.ts`'s last step), dev-mode rebuilds (the integration builds once, in `astro:config:setup`, for `dev`, `build`, and `sync`, and not for `preview`), publishing, and the peer-dependency story (`astro` `^7.3.5`; nothing on `@astrojs/markdown-remark`).
- **Phase 25 (preview):** `render_site_html` and this plugin agree through `tests/render/`; nothing new.
- **Phase 16:** nothing.

### Astro version, and what was checked against it

Written against **Astro 7.3.5** (`@astrojs/markdown-satteri` 0.4.2, `satteri` 0.10.5, `@astrojs/markdown-remark` 7.3.1, Vite 8.3.1). Nothing was taken from memory: each API was read in the installed package's source (`node_modules/.pnpm/astro@7.3.5…/dist`) and then exercised in a real build.

| API | Read in | Result |
|---|---|---|
| Integration hooks: `astro:config:setup` (`config`, `command`, `updateConfig`, `logger`), `astro:build:done` (`dir`) | `dist/types/public/integrations.d.ts` | Used as documented. `config.root`, `config.base` (normalized to a trailing `/`), `config.trailingSlash`, and `config.site` are the resolved values. |
| `glob` loader (`pattern`, `base` as a URL) and its entry id | `dist/content/loaders/glob.js` | `**/*.md` with `!_ascribe/**`. The id is what `AstroRouter::entry_id` computes; `Guides/My Setup.md` is `guides/my-setup`, the root page `index`. |
| `markdown.processor` | `dist/core/config/schemas/base.js`, `validate.js`, `@astrojs/internal-helpers/markdown.d.ts` | **The default is Sätteri, not remark** (Q151). `processor.options.hastPlugins` / `.rehypePlugins` is the documented way for an integration to extend it. |
| Both processors' pass order | `@astrojs/markdown-satteri/dist/satteri-processor.js`, `@astrojs/markdown-remark/dist/index.js` | Sätteri: user hast plugins, then the image marker, then heading ids. Unified: user rehype plugins, then `rehypeImages`, then `rehypeHeadingIds`, then `rehype-raw`. So a plugin at the user hast stage runs first in both, with markers still `raw` nodes. |
| `astro/zod`, `defineCollection`, `render()` and its `headings` | the generated schema, `content.config.ts`, the page | `schema.ts` from phase 20 compiles and validates the site's frontmatter in the real Astro. |
| `build`, `preview`, and `dev` (JavaScript API) | `dist/core/index.d.ts` | Used by the tests. `dev()` behaves differently when `NODE_ENV=test` or `VITEST*` are set (it 404s every page), so the test runs it in a child process without them. |

**The three things phase 20 left for a real build:**

1. **Astro processes a relative image in a collection entry.** Yes. `![…](../_fragments/requirements.png)<marker width=300>` in `Guides/My Setup.md`, a fragment's image mirrored beside the fragment, becomes `/docs/_astro/requirements.<hash>.webp` with `width="300" height="188"` and `loading="lazy"`; the `./weave.png` form beside a page does the same. The asset contract §3 holds as written. (`sharp` must be resolvable from the site: it is an optional dependency of Astro that pnpm doesn't hoist, so the example lists it.)
2. **The plugin's ids and attributes reach Astro's own passes.** Yes, under both processors. An explicit id is kept by the heading-id pass and appears in Astro's `headings` (the layout's table of contents): `@id: weave-config` on "Weave configuration", where Astro's own slug would be `weave-configuration`. An image's `width` reaches the image processing. Verified in the default (Sätteri) build and in a `unified()` build. The plugin is a hast plugin, not the remark plugin phase 20 assumed (Q151).
3. **Serving `_ascribe/files/`.** Copied into the build output in `astro:build:done` (so `astro preview` and any static host serve it) and served in dev by a Vite middleware, both at `<base>/_ascribe/files/`. The sample links to `/docs/_ascribe/files/downloads/loom.yaml` and the tests fetch it in build and dev.

**Contract consequences.** None change behavior. Wording: site-render.md §1 says "user remark plugins" and `tests/render/README.md` says "a unified pipeline"; Q151 proposes naming the hast stage (both are wording, and neither is edited here).

### Route agreement, and the root page

Confirmed in real builds. The sample's page route is `base + entry.id`, with `params.slug` undefined for `index`, and Astro writes the root page at `/docs` under `trailingSlash: "never"` (`astro preview` answers `/docs` and `/docs/guides/my-setup`, and 404s `/docs/guides/my-setup/`). The e2e test follows `[setting up Loom](<Guides/My Setup.md>)` (a slugged, lower-cased id) and `[documentation home](../index.md)` (the root page) by clicking them, and a second build with `base: "/"` and `trailingSlash: "always"` does the same. `AstroRouter` needed no change.

### Decisions

Q151 (a hast plugin for both processors), Q152 (finding the binary), Q153 (`<Elements />`, not `injectScript`, which drops the CSS it imports in Astro 7.3.5), Q154 (routing disagreement fails the build), Q155 (the collection helper reads the site root from a virtual module). All are `open`, implemented as proposed.

Also:

- **The e2e tests build copies of the site under `examples/astro-site/.e2e-tmp/`** (git-ignored, removed afterwards) for the failure and variant cases, so they resolve the same `node_modules`. They run one file at a time (`fileParallelism: false`): the copies and the site share Tessera's locked output directory.
- **`pnpm test` in the example runs nothing** (`--passWithNoTests`); `test:e2e` is separate because it needs a built compiler and a browser, so the `check` job's `pnpm -r test` still needs neither.
- **The example has its own `eslint.config.js`** that adds `.astro/`, `.ascribe/`, and `.e2e-tmp/` to the workspace's ignores. `examples/` is outside Prettier's scope (`.prettierignore`), as the other examples are.
- **The generated `_ascribe/schema.ts` is imported by `content.config.ts` by path**, so it has to exist when Astro loads the config; the integration builds first, in `astro:config:setup`, which is before that.

### How the end-to-end test was run

Locally, in this container, with Chromium at `/opt/pw-browsers/chromium` (no `TESSERA_CHROMIUM` set), Node 22.22, and a debug `tessera` from `cargo build -p tessera-cli`:

```sh
cargo build -p tessera-cli
pnpm install
pnpm --filter @tessera/elements build
pnpm --filter @tessera/astro build
pnpm --filter @tessera/astro test                           # the tests/render/ fixtures under both processors, and unit tests
pnpm --filter @tessera/example-astro-site test:e2e          # real `astro build`, `astro preview`, Chromium
```

The acceptance criterion says "in CI on Linux". The `astro` job in `.github/workflows/js.yml` runs the same steps, and CI here is manual-only, so **that run waits for a manual dispatch**. Only Chromium is used (Firefox and WebKit aren't part of this test).

To check that the test can fail, the Sätteri plugin's registration was disabled in `dist/index.js`: four of its tests failed (ids became `requirements-`, `install-`, …; widths `64` and `48`, not `300` and `120`).

### Acceptance criteria

See the pull request for the status and evidence of each.

### Left open

- **Q151 to Q155.**
- **Dev-mode rebuilds** (phase 22): in `astro dev`, editing a `.md` source doesn't re-run `tessera build`.
- **The binary from npm** (phase 22).
- **A mismatch between the two sides' `output-dir`**: the site imports the schema from a path that repeats `[project] output-dir` (`../.ascribe/build/site/site/_ascribe/schema.ts`), while the integration reads it from `ascribe.toml`. A project with another output directory changes both. A helper could re-export the schema from the virtual module if a site wants a single place.
- **`astro check`** isn't run: `typecheck` for the example covers `test/` only, since `src/content.config.ts` needs Astro's generated types (`astro sync`), which need the compiler.
- **The sample's `available` text** for `self-managed` needed a label (`labels = { "self-managed" = "Self-managed" }`); without one, the text is lower-case (`self-managed (preview, 3.4+)`), which is the emitter's documented behavior.

