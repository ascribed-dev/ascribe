# Phase 01: Content model format

**Track:** Model · **Start after:** none · **Parallel with:** 00 · **Unblocks:** 02, 03, 08 · **Human checkpoint after this phase**

## Goal

Specify `ascribe.toml`, the content model file, completely enough that phase 08 can implement a loader and phase 03 can write fixtures without making design decisions. This is a design and documentation phase; it produces no code.

## Read first

- [SPEC.md](../../SPEC.md): §2 (documents), §4 (built-in directives), §5 (inline constructs), §6 (project widgets), §7 (content model), §9.3–§9.5 (build modes, outputs, consumer profile).
- [PLAN.md](../PLAN.md): the `ascribe.toml` sketch under Content model.

## Deliverables

- `project-docs/content-model.md`: the reference for `ascribe.toml`.
- `examples/content-models/`: at least three example files, each valid under the reference:
  - `minimal.toml`: the smallest valid content model.
  - `quill.toml`: the model Appendix B assumes (see below).
  - `full.toml`: every section and option used at least once.

## Tasks

1. **Structure.** Define every top-level table and key, with its type, whether it's required, its default, and a short description. Cover each declaration in SPEC §7.2:
   - `spec`: the spec version the project targets.
   - `[project]`: content root and output directory. The output directory must not be inside the content root; say so as a validation rule.
   - `[types.<name>]`: frontmatter schemas per content type, plus the fragment schema, and which type applies to which files (for example, by glob or a default).
   - `[fragments]`: extra fragment patterns.
   - `[dimensions.<name>]`: values, display labels, versionless values.
   - `[versions]`: the version scheme. Semantic versioning (`major.minor.patch`, compared numerically) is the default; say what else is allowed, if anything.
   - `[lifecycle.<state>]`: extra states and whether each counts as available. List the defaults from SPEC §4.4 and §7.2.
   - `[features.<key>]`: name and availability spec.
   - `[notes]`: extra note types.
   - `[phrases]`: keys and values, and which frontmatter fields accept phrases (SPEC §5.1).
   - `[glossary]`: terms, definitions, and matching settings (for example, first occurrence per page or every occurrence, and case sensitivity).
   - `[images]`: accepted image attributes and their types.
   - `[widgets.<name>]`: forms, primary kind, binding, title (none, accepted, or required), groupable, attribute schema, plain-text fallback.
   - `[consumer]`: profile name and its settings (SPEC §9.5): routing (base path, trailing slash, how file paths map to URLs), slugger, heading-id emission, HTML passthrough, and image and asset placement.
   - `[builds.<name>]`: variant mode and availability mode, in the shapes SPEC §9.3 shows.
   - `[editor]`: which build the language server checks by default.
2. **Frontmatter field types.** Define the syntax for field types: string, number, boolean, date, enumeration, list, object, optional, and default (SPEC §7.2). Prefer readable forms (such as `"string?"` for an optional string) over nested tables where they stay unambiguous, and give the full grammar for the short forms.
3. **Validation rules.** List every rule a loader must enforce, each with an error message. Include:
   - A name used in more than one role (dimension name, dimension value, lifecycle state, feature key) is an error (SPEC §7.2).
   - Names follow the `name-word` rule (SPEC Appendix A).
   - Labels refer to declared values; versionless values are declared values; feature availability specs are valid; widget names contain a hyphen; build selections refer to declared dimensions and values; the output directory isn't inside the content root.
4. **Quill model.** Write `quill.toml` to match SPEC Appendix B: dimensions `pm` (`npm`, `pnpm`, `yarn`) and `deployment` (`cloud`, `self-managed`, with `cloud` versionless); phrases `product`, `cloud`, `version`, and `api`; builds `site` (switch and badge), `cloud` (selecting `deployment=cloud`, filtering for `cloud`), and `self-managed-3.3` (switch, filtering for `self-managed 3.3`); the Astro consumer profile.
5. **Open decisions.** Anything you can't settle from the spec goes into `project-docs/questions.md` with your recommendation, and the reference marks it as provisional.

## Acceptance criteria

- [x] Every declaration in SPEC §7.2 maps to a documented section of `content-model.md`.
- [x] Every key has a type, a default or "required", and a description.
- [x] The three example files exist, and each follows the reference exactly.
- [x] `quill.toml` supports every construct used in SPEC Appendix B.
- [x] Every loader validation rule has an error message.

## Out of scope

- Implementing the loader (phase 08).
- Changing SPEC.md. Record needed spec changes as questions.

## Notes

- Readability matters as much as in the markup itself: technical writers will edit this file. TOML strings are always quoted, which is one reason it was chosen (SPEC §7.1).
- Keep the format flat where possible. Deeply nested tables are hard to read and hard to diff.

## Handoff notes

### What was built

- **`project-docs/content-model.md`**, the `ascribe.toml` reference:
  - §1: conventions. Kebab-case keys, unknown keys are errors, name grammars, path and glob syntax, and which declaration orders are significant.
  - §2: a table mapping every SPEC §7.2 declaration (plus §9.3, §9.5, §11, and the editor setting) to its section.
  - §3–§18: one section per table, each with a key table (type, default or **required**, description) and an example.
  - §6: the field-type and attribute-type short forms, with an ABNF grammar and a table form for defaults, descriptions, phrases, nested objects, and enumerations with arbitrary values.
  - §19: what each absent section means.
  - §20: 60 loader validation rules, each with a stable slug (`model-…`) and a message template.
  - §21: 21 numbered open questions, each with a recommendation. Every provisional item in the body points at one.
- **`examples/content-models/`**:
  - `minimal.toml`: only `spec = "0.1"`.
  - `quill.toml`: the SPEC Appendix B model, with dimensions `pm` and `deployment` (`cloud` versionless), the four phrases, the builds `site`, `cloud`, and `self-managed-3.3`, and the Astro profile.
  - `full.toml`: every section, every key, and every short-form type.
  - `docs/reference/glossary.md`: a one-page content root, so all three models pass the filesystem rules (the content root must exist; `full.toml`'s glossary link must name a page). It's valid frontmatter under all three models.

### For phase 08 (loader)

- Load all three example files from `examples/content-models/` with no issues; `docs/` is their shared content root.
- Every rule in §20 needs a failing fixture producing its message at the offending key's span. Two rules are warnings: `model-name-case` and `model-build-filter-excluded`.
- The filesystem rules (`model-content-root-missing`, `model-glossary-link`) are skipped when loading from text alone.
- Preserve the declaration order of attribute tables; canonical form depends on it (§1.1). `toml_edit` and `toml-span` both keep order.
- Widgets map onto phase 02's `DirectiveSchema`. Binding values are `self`, `heading`, `block`, and `heading-or-block` (§15). `block` with a `text?` primary behaves like `@note`: a given primary is the content.
- The implicit `page` type and `site` build (§19) exist only when `[types]` or `[builds]` is absent.

### For phase 02 (contracts)

- `model-name-multiple-roles` is the only loader rule that is a SPEC §8.2 row. Q1 added document-level rows for frontmatter and content-type assignment to SPEC §8.2; the registry needs them.
- The `[consumer]` values `heading-ids`, `image-attributes`, `assets`, and `assets-dir` are provisional (Q12). The site-render and asset contracts define the exact output, and may rename these values.
- Glossary rendering (Q7) may need an element in the element contract.

### For phase 03 (fixtures)

- Base `tests/conformance/_model/ascribe.toml` on `quill.toml`. `full.toml` has ready-made widgets covering every binding, primary kind, title setting, and groupable, plus a feature with a history, a lifecycle state that doesn't count as available (`sunset`), and an extra note type.
- Q1–Q21 are decided, so cases that depend on them are ordinary cases. Only cases depending on Q12's consumer values should be tagged `provisional`.

### Decisions (the checkpoint)

All 21 items in `content-model.md` §21 were decided on 2026-09-28 at the human checkpoint, each as recommended. Items 1, 2, 3, 4, and 6 are also written into SPEC.md (§2.1, §5.2, §7.2, §8.2). Q12's consumer values stay provisional until phase 02's contracts. The decisions stay in §21 rather than moving to `project-docs/questions.md`, since none is open.

### Left open

- A JSON Schema for `ascribe.toml` would give writers completion in any TOML editor (for example, through Taplo). It isn't in any phase yet.
- The Astro routing rule (§16) says routes follow how Astro's content loader computes entry ids. Phase 20 must verify this against the Astro version it targets, including Astro's handling of a `slug` frontmatter field.
