# Phase 01: Content model format

**Track:** Model · **Start after:** none · **Parallel with:** 00 · **Unblocks:** 02, 03, 08 · **Human checkpoint after this phase**

## Goal

Specify `tessera.toml`, the content model file, completely enough that phase 08 can implement a loader and phase 03 can write fixtures without making design decisions. This is a design and documentation phase; it produces no code.

## Read first

- [SPEC.md](../../SPEC.md): §2 (documents), §4 (built-in directives), §5 (inline constructs), §6 (project widgets), §7 (content model), §9.3–§9.5 (build modes, outputs, consumer profile).
- [PLAN.md](../PLAN.md): the `tessera.toml` sketch under Content model.

## Deliverables

- `project-docs/content-model.md`: the reference for `tessera.toml`.
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

- [ ] Every declaration in SPEC §7.2 maps to a documented section of `content-model.md`.
- [ ] Every key has a type, a default or "required", and a description.
- [ ] The three example files exist, and each follows the reference exactly.
- [ ] `quill.toml` supports every construct used in SPEC Appendix B.
- [ ] Every loader validation rule has an error message.

## Out of scope

- Implementing the loader (phase 08).
- Changing SPEC.md. Record needed spec changes as questions.

## Notes

- Readability matters as much as in the markup itself: technical writers will edit this file. TOML strings are always quoted, which is one reason it was chosen (SPEC §7.1).
- Keep the format flat where possible. Deeply nested tables are hard to read and hard to diff.

## Handoff notes

_To be filled in by the implementing agent._
