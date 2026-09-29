# Phase 20: Site output and Astro profile

**Track:** Output · **Start after:** 18 · **Parallel with:** 15 · **Unblocks:** 21, 25

## Goal

Emit the site output (markdown plus web components), implement the Astro consumer profile, generate the Zod schema Astro's content collections need, and implement Tessera's own renderer for the site-render contract, which the preview uses.

## Read first

- [SPEC.md](../../SPEC.md): §9.4–§9.6, including Assets.
- `packages/elements/CONTRACT.md`, `project-docs/contracts/site-render.md`, `tests/render/`, `project-docs/contracts/assets.md`, and `project-docs/contracts/output-layout.md` (phase 02).
- Handoff notes from phases 09, 12, and 18.

## Deliverables

- `crates/tessera-emit/src/site/`: the site-markdown emitter.
- The Astro implementation of `tessera-core`'s `ConsumerProfile` and `Router` traits.
- `crates/tessera-emit/src/render/`: `render_site_html()`, which renders site markdown to HTML following the site-render contract.
- `crates/tessera-emit/src/zod/`: Zod schema generation.

## Tasks

1. **Astro profile.**
   - **Routing:** from the model's consumer settings (base path, trailing slash, how collection entries map to URLs). Implement the `Router` trait so phase 12's link resolution produces Astro URLs.
   - **Slugger:** `github` (phase 09).
   - **Heading ids and image attributes:** emit exactly the syntax the site-render contract defines. Phase 21's Astro plugin applies it.
   - **Assets:** place copies where Astro's image processing handles them, as the asset contract's Astro rules say (for example, beside the compiled page, with a relative reference), so Astro still optimizes images.
2. **Site emitter.** Emit from phase 12's resolved tree, following SPEC §9.4's table and the element contract:
   - A group with several surviving arms becomes `<tessera-tabs>`; a group with one becomes plain content.
   - Availability annotations become `<tessera-availability>` elements, in badge and filter builds alike.
   - Put a blank line after each opening tag and before each closing tag that wraps markdown (SPEC §9.4).
   - Pass page-level frontmatter through, including `available` in a form the layout can read.
   - Copy and rewrite assets, and write output, through phase 18's shared handling.
3. **Renderer.** `render_site_html()` turns site markdown into HTML: comrak's HTML rendering with raw HTML allowed, plus the heading-id and image-attribute transforms the site-render contract defines. It must pass every `tests/render/` fixture. The preview (phase 25) uses it, and the fixtures are what keep it equal to the Astro plugin.
4. **Zod generation.** From the content model's frontmatter schemas, generate a TypeScript module exporting one Zod schema per content type, including the reserved `available` and `variant` keys. Verify the generated code type-checks and validates the Quill frontmatter.
5. **`tessera build --emit site`.** Add the site emitter to the build command.
6. **Snapshots.** Site output for the Quill project under each build.

## Acceptance criteria

- [ ] `tessera build --emit site` builds `examples/quill` under every build.
- [ ] Every element and attribute in the output matches `CONTRACT.md`; the cloud build's output keeps the `pm` group as `<tessera-tabs>`.
- [ ] Links in the output use Astro routes; a test checks them against the routing settings.
- [ ] Assets in the output follow the asset contract's Astro rules, and the fragment-image case resolves with the source directory removed.
- [ ] `render_site_html()` passes every `tests/render/` fixture.
- [ ] The generated Zod module type-checks with `tsc` and accepts the Quill project's frontmatter.
- [ ] Snapshots are reviewed and committed.

## Out of scope

- The Astro integration package and its markdown plugin (phase 21).

## Notes

- Verify Astro specifics (content collection loaders, where schemas live, how images in collections are processed) against the current Astro documentation, and record the Astro version targeted. Astro's APIs change between major versions.

## Handoff notes

### What was built

- **`crates/tessera-emit/src/site/`**: `SiteEmitter` (`Emitter`), and `AstroProfile` (`ConsumerProfile`: routing, the `github` slugger, HTML passthrough, asset placement). `blocks.rs` writes the elements of `packages/elements/CONTRACT.md`, `inline.rs` the inline content and the attribute markers, `frontmatter.rs` the page's frontmatter, `element.rs` the escaping and layout rules.
- **`crates/tessera-emit/src/render/`**: `render_site_html()`, passing all twelve fixtures in `tests/render/` (eleven from phase 02, one I added: `image-ends-heading`).
- **`crates/tessera-emit/src/zod/`**: `zod::generate(model)`, written as `_tessera/schema.ts` in the site output.
- **`crates/tessera-resolve/src/astro.rs`** (new file, in the resolve crate so the source index can use it without a dependency on the emitters): `AstroRouter`, the Astro profile's `Router`, plus `page_for_route` (which page has a route) and `collisions`.
- **`tessera build --emit site`**, and every output now resolved with `AstroRouter` (Q144); `--emit` defaults to `site,plain,json`.
- **`tests/zod/`**: a pnpm workspace package (added to `pnpm-workspace.yaml`) that type-checks the generated Zod modules with `tsc` and validates the Quill pages' frontmatter with them. It depends on `zod` (4.6) and aliases `astro/zod` to `zod/v4`, which is what Astro 7.3.5's `astro/zod` re-exports, so it doesn't pull Astro's dependency tree into the workspace; phase 21 runs the real one.
- **Interim route mapping replaced, in the way that adds to it** (Q148): `references::route` still tries `route.md` and `route/index.md` first, then asks `AstroRouter::page_for_route` through the new `SourceSet::pages` (default: empty, so nothing else that implements the trait changes). `tessera check` and the source index agree (`crates/tessera-check/tests/parity.rs` passes).

### Interfaces later phases use

- **Phase 21 (the Astro plugin):** the marker rules are the site-render contract's; the fixtures in `tests/render/` are the shared test, and `image-ends-heading` is new (Q145). Read `_tessera/schema.ts` (`schema`, `schemas`, `contentTypes`, `availableSchema`) for the collection; the site output root is `<output-dir>/<build>/site/`; `_tessera/files/` must be served at `<base-path>_tessera/files/`. Images stay relative (`./img/a.png`) so Astro processes them. Page-level `available` is a list of targets (Q142); a layout writes `<tessera-availability scope="page">` from it.
- **Phase 25 (preview):** `tessera_emit::render_site_html(markdown)`; `SiteEmitter::new(model)` with `emit`, or the whole pipeline through `tessera build`.
- **Everyone:** `tessera_resolve::AstroRouter` (`from_consumer`, `route`, `entry_id`, `page_for_route`, `collisions`); `tessera_emit::AstroProfile`.

### Astro version

Written against **Astro 7.3.5** (`astro/zod` is Zod 4.6). Verified in `node_modules` of that release: the `glob` loader's entry id (`getContentEntryIdAndSlug`: each path segment slugged by `github-slugger` 2.0.0's pure `slug()`, joined with `/`, a final `/index` removed, one extension removed) is what `AstroRouter::entry_id` computes; `astro/zod` re-exports `zod/v4`, where `z.strictObject`, `z.coerce.date()`, and `.default()` are the forms the generated module uses. **Not verified here, and left for phase 21** (as the asset and site-render contracts say): that Astro processes a relative `./img/a.png` in a collection entry's markdown, that a remark plugin setting `hProperties` reaches both Astro's heading-id pass and its image processing, and how the integration serves `_tessera/files/`.

### Decisions

Every choice the spec leaves open is a question with the implemented answer: Q141 (image defaults reach every image), Q142 (page-level `available` is a list of targets), Q143 (two pages with one route fail the site output), Q144 (`AstroRouter` for every output; all three outputs by default), Q145 (a marker after an image that ends a heading applies to the image), Q146 (a list that holds an element is loose), Q147 (an image in a `@details` title is its alt text), Q148 (the router finds the page a route names), Q149 (the Zod module). All are `open`, implemented as proposed.

Also:

- **The site markdown is a tree walk like the plain emitter's** (`blocks.rs`), sharing its list, quote, fence, and escaping code (now `pub(crate)`), and its handling of following-block directives (the tree keeps them as siblings, and the emitter wraps the block they bind, in stacking order).
- **Markers are applied to comrak's HTML** (`render/`): comrak writes raw HTML unchanged and escapes `<`, `>`, `&`, and `"` in every value it writes, so a marker is found exactly, and the rules (§2) are decided on the original HTML, so a marker that follows another marker never applies to the image.
- **`labels.rs`** gained `availability_target_text`, one target's text; `availability_display` uses it, so the plain line is unchanged.
- **A bug in the plain emitter was fixed on the way**: a heading whose text ends in an already escaped `#` (`## \#`) got `\\#`, which reads as an escaped backslash and a `#`. Both emitters use `escape_closing_hash`; `tests/plain.rs` has a case.
- **Attributes on a tab label** follow the contract's canonical order (the content model's dimension order), where the plain output labels in written order (phase 18, Q115): the two differ only for an arm whose attributes are written out of order.
- **`tessera-check`'s page pass** still uses `DefaultRouter` (Q144): a route's text never changes a diagnostic.

### Conformance

`cargo test -p tessera-conformance`: **364 passed, 0 failed, 0 skipped**. No case carries the `output` tag, so the `output` entry in `SKIPS.toml` doesn't gate this phase (phase 18 said the same). `tests/conformance/tests/render_fixtures.rs` covers the new fixture. The site output's own tests are in `crates/tessera-emit/tests/` (`site.rs`, `site_quill.rs` with snapshots for every page of Quill under each build, `site_assets.rs`, `render_fixtures.rs`, `zod.rs`) and `crates/tessera-cli/tests/build.rs`.

### Left open

- **Q141 to Q149** (above).
- **A tight list that holds an element becomes loose** (Q146), so its items render `<p>`.
- **Route collisions** are an emitter error, not a registry diagnostic (Q143): `tessera check` doesn't report them, only `tessera build --emit site`.
- **Raw HTML in a `<summary>`** can't hold a processed image (Q147).
- **Prettier doesn't format the generated Zod module** (`tests/` is ignored), so its layout is whatever `zod::generate` writes; it is stable, so the fixtures don't churn.
- **The default of `tessera build`** now includes `site`, so a project with two pages of one route can't build with the defaults; `--emit plain,json` is the way around it.
