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

_To be filled in by the implementing agent._
