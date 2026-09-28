# Phase 02: Core contracts

**Track:** Setup · **Start after:** 00, 01 · **Parallel with:** 04, 17 · **Unblocks:** 03, 05, 08, 09, 19 · **Human checkpoint after this phase**

## Goal

Write every interface that parallel phases share, as completed deliverables, before those phases start. Later phases implement against these contracts instead of coordinating informally. Each contract is small, precise, and reviewed by a human.

## Read first

- [SPEC.md](../../SPEC.md): all of it, with particular attention to §3 (directive schemas), §4.2 and §5.5 (source ids and page ids), §8 (diagnostics), §9 (compilation, outputs, assets, consumer profile, element library), and Appendix A.
- [PLAN.md](../PLAN.md) and `project-docs/content-model.md` (phase 01).

## Deliverables

1. **`crates/tessera-core`** (types and traits only, no parsing logic):
   - `FileId`, `Span`, and `LineIndex` (byte offset to line and column, with UTF-8 and UTF-16 columns).
   - `TextEdit`, used by diagnostics fixes, the formatter, and refactorings.
   - `Issue`: a problem found by any crate, carrying a diagnostics slug, a span, and message arguments.
   - `DirectiveSchema`: permitted forms, primary kind (none, identifier, text; optional or required), binding, title (none, accepted, required), groupable, attribute schema. Include the built-in schemas for every directive in SPEC §4.
   - An attribute-value type for parsed attributes (token, quoted string, value set, each with spans). Phase 05 writes the parser.
   - The `Slugger` trait (phase 09 implements it), and the `Router` and `ConsumerProfile` traits (SPEC §9.5; phases 12 and 20 implement them).
2. **`tests/conformance/diagnostics.toml`**: the diagnostics registry. One entry per row of SPEC §8.2, with a stable `code` (`TSR001` onward, never reused), a kebab-case `slug`, the `severity`, the `level` (`file` or `page`, SPEC §8.1), the `spec` section, and a `message` template with named placeholders. Every crate reports problems by slug; phase 10 maps slugs to codes and severities.
3. **`packages/elements/CONTRACT.md`**: the element contract between the site emitter (phase 20) and the element library (phase 19). For each element, its tag name, attributes and their meaning, expected children, and rendering with and without JavaScript: `<tessera-note type title>`, `<tessera-steps>` wrapping an ordered list, `<tessera-tabs sync>` containing `<tessera-tab value label>`, and `<tessera-availability>`, whose attributes SPEC §9.4 leaves open (it needs targets, states, versions, and display labels). `@details` compiles to the native `<details>` element.
4. **`project-docs/contracts/assets.md`**: the asset contract (SPEC §9.4, Assets). It defines:
   - Which references are assets: image sources, and link targets that aren't pages.
   - How a reference resolves: from the file it's written in, including inside included fragments (SPEC §4.2), to a file under the project.
   - Where copies go in each output, how their file names are chosen (and how collisions between different files with the same name are avoided), and how references are rewritten, including how the consumer profile changes placement so Astro's image processing still applies.
   - The guarantee: output works with the source directory removed.
5. **`project-docs/contracts/output-layout.md`**: the build output contract. It defines:
   - The directory layout: `<output>/<build>/<emitter>/`, mirroring source paths for pages, plus asset locations.
   - A manifest of generated files per build and emitter.
   - **Stale-output policy:** a build writes to a staging directory and replaces the previous output only on success. Files listed in the previous manifest but not produced now are removed. Files the manifest never listed are never deleted; if a user file sits where generated output would go, the build fails with an error rather than overwriting it.
6. **`project-docs/contracts/site-render.md`** and **`tests/render/`**: the site-render contract. The site output uses syntax a consumer must apply beyond plain CommonMark: explicit heading ids and image attributes (SPEC §9.5). This contract defines exactly what that syntax is and what HTML it must produce. It's implemented twice: by the Astro markdown plugin (phase 21) and by Tessera's own HTML renderer for the preview (phase 20). `tests/render/` holds shared fixtures, site-markdown input paired with expected HTML, that both implementations must pass, which is what keeps the preview and the published site in agreement.

## Tasks

1. Write each deliverable above.
2. For the Rust types, write doc comments explaining every field and invariant, and add small unit tests where behavior exists (for example `LineIndex` conversions, including characters outside the Basic Multilingual Plane).
3. For the registry, confirm there's exactly one entry per §8.2 row.
4. For `tests/render/`, write at least one fixture per construct the contract covers, including a heading with an explicit id, a heading whose slug needs duplicate numbering, an inline image with attributes, and a reference image with attributes.
5. Record anything the spec leaves open in `project-docs/questions.md`, and mark it provisional in the contract.

## Acceptance criteria

- [ ] `cargo test -p tessera-core` passes, and `cargo doc -p tessera-core` documents every public item.
- [ ] `diagnostics.toml` has exactly one entry per §8.2 row, each with all six fields.
- [ ] The element, asset, output-layout, and site-render contracts each answer every question listed for them above.
- [ ] `tests/render/` has fixtures covering every construct the site-render contract defines.
- [ ] A human has reviewed the contracts (the checkpoint).

## Out of scope

- Implementations: parsers, emitters, elements, and the Astro plugin.

## Notes

- Contracts are expensive to change once parallel work starts. Prefer small, explicit definitions, and write down the reason behind each non-obvious choice.

## Handoff notes

_To be filled in by the implementing agent._
