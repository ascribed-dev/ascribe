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
2. **`tests/conformance/diagnostics.toml`**: the diagnostics registry. One entry per row of SPEC §8.2, with a stable `code` (`ASC001` onward, never reused), a kebab-case `slug`, the `severity`, the `level` (`file` or `page`, SPEC §8.1), the `spec` section, and a `message` template with named placeholders. Every crate reports problems by slug; phase 10 maps slugs to codes and severities.
3. **`packages/elements/CONTRACT.md`**: the element contract between the site emitter (phase 20) and the element library (phase 19). For each element, its tag name, attributes and their meaning, expected children, and rendering with and without JavaScript: `<ascribe-note type title>`, `<ascribe-steps>` wrapping an ordered list, `<ascribe-tabs sync>` containing `<ascribe-tab value label>`, and `<ascribe-availability>`, whose attributes SPEC §9.4 leaves open (it needs targets, states, versions, and display labels). `@details` compiles to the native `<details>` element.
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

- [x] `cargo test -p tessera-core` passes, and `cargo doc -p tessera-core` documents every public item.
- [x] `diagnostics.toml` has exactly one entry per §8.2 row, each with all six fields. (SPEC §8.2 split its four mixed-level rows when Q3 was resolved.)
- [x] The element, asset, output-layout, and site-render contracts each answer every question listed for them above.
- [x] `tests/render/` has fixtures covering every construct the site-render contract defines.
- [x] A human has reviewed the contracts (the checkpoint, 2026-09-28).

## Out of scope

- Implementations: parsers, emitters, elements, and the Astro plugin.

## Notes

- Contracts are expensive to change once parallel work starts. Prefer small, explicit definitions, and write down the reason behind each non-obvious choice.

## Handoff notes

### What was built

- **`crates/tessera-core`**, types and traits with doc comments on every public item and 27 unit, integration, and doc tests:
  - `span`: `FileId` (opaque, assigned by whoever owns the file table), `Span` (UTF-8 byte range, `start <= end` enforced), `Location`.
  - `line_index`: `LineIndex`, converting offsets to 0-based lines and UTF-8 columns, and to UTF-16 or scalar-value columns (`WideEncoding`), and back. Lines end at `\n`, `\r\n`, or a lone `\r`, as in CommonMark. Tests cover characters outside the Basic Multilingual Plane and round-trip every character boundary.
  - `text_edit`: `TextEdit` (simultaneous edits, as in LSP) and `apply_edits`, which rejects overlaps and spans off character boundaries.
  - `issue`: `Issue` (slug, location, message variant, named arguments, related locations, fixes as `TextEdit`s), and `DiagnosticSlug`, which only this crate can construct.
  - `diagnostics`: one `DiagnosticSlug` constant per registry entry, plus `ALL`. A test keeps them equal to `diagnostics.toml`, in order.
  - `schema`: `DirectiveSchema` (forms, primary, binding, title rule, groupable, attributes, description) and `builtin_schemas()` for the seven SPEC §4 directives; `END_KEYWORD`.
  - `attribute_block`: `AttributeBlock`, `Attribute`, `AttributeValue` (token, quoted, set), `Token`, all with spans. Phase 05's parser goes in `attributes.rs` and produces these.
  - `path`: `RelPath` (normalized, `/`-separated, leading `..` allowed), `classify_destination` and `LocalDestination::resolve`: the asset contract's resolution steps 1–4.
  - `reserved`: the attribute keys a content model can't declare (SPEC §7.2), as explicit lists, with `is_reserved_image_attribute` and `is_reserved_widget_attribute` for phase 08's `model-attribute-reserved`.
  - `consumer`: the `Slugger` and `SlugScope`, `Router`, and `ConsumerProfile` traits, and `AssetUse` and `AssetPlacement`.
- **`tests/conformance/diagnostics.toml`**: 119 entries, one per SPEC §8.2 row and one per loader rule. ASC001–ASC054 cover SPEC §8.2's original rows (the four rows Q3 split have one entry per half); ASC055–ASC059 cover the five rows Q5 added; ASC060–ASC118 are content-model.md §20's original 59 loader rules, with `model-name-multiple-roles` as §8.2's "Content model" row; ASC119 is `model-attribute-reserved`, added by Q9. No entry is provisional. Every entry has `code`, `slug`, `severity`, `level`, `spec`, and `message`; optional fields are `messages` (named variants), `row`, `rule`, `provisional`, and `retired`. The header documents the format.
- **Registry checks**: `tests/conformance/tests/registry.rs` checks the registry against SPEC §8.2 (exactly one entry per row, severities, rows marked page level), content-model.md §20 (every rule, with the same severity and exactly the same messages), SPEC's section numbers, and questions.md; codes are sequential and slugs unique and kebab-case; templates are well formed.
- **Slug validation in the harness** (the phase 00 follow-up): `tessera_conformance::registry` loads `diagnostics.toml`, and the runner fails a case, even a skipped one, that expects an unregistered slug, a slug at the wrong level, or a provisional slug without the `provisional` tag and its questions. Self-tests cover each.
- **`packages/elements/CONTRACT.md`**: `ascribe-note`, `ascribe-steps`, `ascribe-tabs` and `ascribe-tab`, `ascribe-availability` and `ascribe-availability-target`, `details`, project-widget elements with `ascribe-group`, and glossary links.
- **`project-docs/contracts/assets.md`**, **`output-layout.md`**, and **`site-render.md`**.
- **`tests/render/`**: 11 fixtures (`input.md`, `expected.html`), indexed in `fixtures.toml`, covering all 17 constructs in the site-render contract's §6 table; `tests/conformance/tests/render_fixtures.rs` checks coverage. The expected HTML is comrak 0.55's rendering with the marker rules applied.
- **content-model.md**: Q12 settled (below); cross-references to the contracts; `[dimensions]` order is now a MUST. `examples/content-models/full.toml` drops the removed keys.
- **questions.md**: Q3–Q11, all resolved at the checkpoint (phase 04 has Q1 and Q2).
- **SPEC.md**, edited with the human's authorization to record the Q3–Q11 resolutions: §3.4 and §4.4 (line primary), §3.7 and §8.2 (a title above a directive that takes none is a warning), §4.2 and §5.2 (linkable ids), §5.5 (explicit ids and slug numbering), §7.2 (reserved attribute keys), §8.2 (split and added rows), and §9.4 (`heading`, asset boundaries).

### Interfaces later phases use

- **05:** parse attributes into `tessera_core::AttributeBlock`; take `ParseOptions` schemas as `DirectiveSchema`s, starting from `builtin_schemas()`. `Primary::Availability` is SPEC §3.4's line primary: the rest of the line, never continued. Report with `Issue::new(diagnostics::…, location)`.
- **08:** build a `DirectiveSchema` per widget (`Origin::Widget`); widget `binding` values map one-to-one onto `Binding`. Loader rules report the `diagnostics::MODEL_*` slugs, with the registry's message variants.
- **09:** implement `Slugger` (`name() == "github"`) and `SlugScope`; explicit ids don't go through the scope (SPEC §5.5).
- **10:** read `diagnostics.toml` for codes, severities, and templates (`{name}` placeholders, `{{`/`}}` literal braces, variants selected by `Issue::variant`). Columns for people and conformance: `LineIndex::wide_line_col(WideEncoding::Utf32, …)` plus 1.
- **11, 12:** resolve references with `classify_destination` and `LocalDestination::resolve`, then the asset contract's boundary and exact-case checks; route links with `Router`.
- **15:** `LineIndex` with `WideEncoding::Utf16` for LSP positions.
- **18, 20:** the output-layout contract (manifest, staging, ownership) and the asset contract's placement; `ConsumerProfile::asset_placement` for the site output.
- **19:** `packages/elements/CONTRACT.md`.
- **20, 21:** the site-render contract and `tests/render/`.
- **03:** expect diagnostics by registry slug, at their level. No registry entry is provisional now; if one becomes provisional later, its cases must be tagged `provisional`.

### Decisions (approved at the checkpoint, 2026-09-28)

1. **One registry entry per §8.2 row.** SPEC §8.2 now splits the four rows that joined a file-level and a page-level check (Q3), so every slug has one level.
2. **Diagnostics §8.2 lacked** (Q5), now rows: malformed attribute blocks, duplicate attribute keys, malformed availability specs, invalid `@id` values, and missing required image attributes.
3. **The attribute marker** (`<ascribe-attributes …></ascribe-attributes>`) for heading ids and image attributes, instead of `{#id}` or `{width=600}` blocks, because Astro's GFM and smartypants passes run before user plugins and rewrite text-based blocks.
4. **Every heading in the site output carries its page id.**
5. **Q12 settled by removing `heading-ids`, `image-attributes`, `assets`, and `assets-dir`** (and `model-consumer-assets-dir`).
6. **Assets mirror their source paths**; under `astro`, images stay relative and other linked files are published under `_ascribe/files/`.
7. **Asset boundary and case** (Q10, now SPEC §9.4).
8. **Output ownership through a manifest** that always lists every file Tessera wrote.
9. **Tabs sync on one dimension**; labeled groups don't sync.
10. **Availability is a badge**, with `<ascribe-availability-target>` children.
11. **The note's title attribute is `heading`** (Q8, now SPEC §9.4), beside `label`, the type's display label. Widget elements use `heading` for their title lines too.
12. **Reserved attribute keys** (Q9): the loader rejects image keys `src`, `alt`, and `title`, widget keys `heading` and `primary`, and on both HTML's global attributes, `aria-` keys, and HTML's event-handler attributes, from explicit lists in `tessera_core::reserved`, so `online` is allowed (`model-attribute-reserved`, SPEC §7.2).
13. **A title line above a directive that takes none** stays a paragraph, with a warning, not an error (Q11, SPEC §3.7, §8.2).
14. **Line primaries** (Q4), **per-file linkable ids** (Q6), and **explicit ids outside slug numbering** (Q7), now in SPEC.
15. **No glossary element**; **`Issue` carries no severity or text**, and only `tessera-core` can create a `DiagnosticSlug`.

### Left open

- **Astro specifics** to verify in phase 21: that relative images in collection entries are processed from the entry's file; that a remark plugin setting `data.hProperties` reaches both Astro's heading-id pass and its image processing; how the integration serves `_ascribe/files/`. The contracts say what must happen; phase 21 records how.
- **Page-level availability in frontmatter**: phase 20 defines the shape layouts read.
- **Route collisions** (two pages with the same route) have no diagnostic; phase 20 should raise one if its router can produce them.
- **Merged with main after phase 04**: `project-docs/questions.md` keeps Q1–Q2 then Q3–Q11, and SPEC §3.4 keeps both phases' additions. `Cargo.lock` was regenerated for the second `toml` version phase 04 brought in.

### CI

Both workflows passed on pull request #4 (run 36457842168 for Rust, 36457842178 for JavaScript): `fmt`; clippy, build, and test on Linux, macOS, and Windows; and the JavaScript format, lint, typecheck, and test job. After the checkpoint resolutions and the merge of main, they passed again (runs 36467638093 and 36467638008). They haven't run on a push to `main`, which happens only after merge.
