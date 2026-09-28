# Phase 08: Content model crate

**Track:** Model · **Start after:** 01 (after human review), 02 · **Parallel with:** 03, 05, 09, 19 · **Unblocks:** 10, 11, 23

## Goal

Implement `tessera-model`: load `tessera.toml`, validate it against the phase 01 reference, and expose typed data to every other crate. Also implement the availability-spec parser in `tessera-core`.

## Read first

- `project-docs/content-model.md` and `examples/content-models/` (phase 01).
- [SPEC.md](../../SPEC.md): §4.4 (availability specs), §7, and Appendix A (`availability`, `name-word`).
- `crates/tessera-core` (phase 02), especially `DirectiveSchema` and `Issue`.

## Deliverables

- `crates/tessera-model`: `load(path) -> Result<ContentModel, Vec<Issue>>`, and typed structures for every section of the reference.
- `crates/tessera-core/src/availability.rs`: a parser for availability specs, with spans.

## Tasks

1. **Loading.** Parse `tessera.toml` with a TOML library that keeps source spans (for example `toml_edit` or `toml-span`), so every problem points at a line in the file.
2. **Typed model.** Structures for each section: project, content types and frontmatter schemas, fragments, dimensions, versions, lifecycle states, features, note types, phrases, glossary, images, widgets, consumer profile settings, builds, editor settings. Apply defaults from the reference.
3. **Widgets.** Convert each declared widget into a `tessera-core` `DirectiveSchema`, exactly as phase 02 defined the type.
4. **Frontmatter schemas.** Parse the field-type syntax from the reference into a schema representation, and provide `validate_frontmatter(&schema, &yaml_value) -> Vec<Issue>` for phase 10 to call.
5. **Availability specs.** Parse the SPEC §4.4 grammar: targets, states, bare versions, parenthesized histories, feature keys. Return a typed spec with spans, or issues. Resolving feature keys and checking against the model is phase 12's job; this phase parses and checks syntax.
6. **Validation.** Enforce every rule in the reference, including the one-role rule for names (SPEC §7.2) and the output directory not being inside the content root. Each problem is an `Issue` with a span, using a registry slug where one applies.
7. **Queries.** Lookups other crates need: whether a phrase key is declared, a dimension's values and labels, whether a value is versionless, whether a lifecycle state counts as available, a widget's schema, the directive keyword set (built-ins plus widgets), a build by name, the editor's default build, and the consumer settings.
8. **Tests.** Every phase 01 example file loads cleanly. Every validation rule has a failing fixture with the expected error.

## Acceptance criteria

- [ ] `cargo test -p tessera-model -p tessera-core` passes.
- [ ] All three phase 01 example files load with no issues.
- [ ] Each validation rule in `content-model.md` has a test producing the documented error at the right line.
- [ ] The availability parser handles every example in SPEC §4.4 and rejects malformed specs with spans.

## Out of scope

- Validating documents against the model (phase 10).
- Watching the file for changes (phases 13 and 15).

## Notes

- The model is read on every language-server start and every CLI run, so keep loading fast.
- If the reference turns out to be ambiguous, follow the questions protocol. Don't invent format.

## Handoff notes

### What was built

- **`tessera_core::availability`** (`crates/tessera-core/src/availability.rs`): `parse_availability(text, offset) -> Result<AvailabilitySpec, AvailabilityError>`. It parses SPEC Appendix A's `availability` rule (targets, states, bare versions, parenthesized histories) with a span on every node, `offset` being where the text starts in its file. It reports the first error, with the offending span (an empty span where something is missing) and a `detail` for the `{detail}` placeholder of `available-syntax` and `model-availability-syntax`. `AvailabilitySpec::bare_name()` says when the whole spec could be a feature key. `Version::compare` and `compare_components` implement the `numeric` scheme. It doesn't know which names the model declares. `lib.rs` gained one line (`pub mod availability;`).
- **`tessera-model`**:
  - `load(path)`, `load_with_file(path, FileId)`, `load_str(text, FileId)`, and `load_str_in(text, FileId, project_dir)` return `Result<ContentModel, Vec<Issue>>`. Issues are sorted by position. Warnings (`model-name-case`, `model-build-filter-excluded`) are in `ContentModel::warnings`; when loading fails, the error vector includes the warnings found.
  - `ContentModel` has typed data for every section, with §19's defaults applied (the implicit `page` type and `site` build, the built-in lifecycle states and note types). Collections keep declaration order.
  - Queries: `has_phrase`, `phrase`, `dimension`, `dimension_of_value`, `dimension_values`, `value_label`, `is_versionless`, `lifecycle_state`, `state_is_available`, `feature`, `note_type`, `widget_schema`, `widget`, `directive_schemas` (built-ins, then widgets), `directive_keywords`, `build`, `editor_default_build`, `consumer`, `is_fragment`, `type_for` (§5.1: one, ambiguous, or none), `is_target`, `check_availability`.
  - Widgets are `Widget { schema: DirectiveSchema, plain_fallback, plain_content }`.
  - `validate_frontmatter(&FrontmatterSchema, &serde_yaml::Value, Location) -> Vec<Issue>` reports `frontmatter-unknown-key` (with the `suggestion` variant), `-missing-field`, `-type-mismatch`, and `-reserved-in-fragment`. Every issue is located at the `Location` passed in and names the field in its `field` or `key` argument (`author.name`, `tags[1]`), so phase 10 can relocate it to the key's span. `available` and `variant` are accepted on pages and rejected in fragments. YAML uses the core schema (`yes` is a string, `3.10` is a number).
  - `check_availability(&AvailabilitySpec) -> Vec<AvailabilityProblem>` checks targets, states, versionless targets given versions, and history order. The loader turns problems into `model-availability-*`; phases 10 and 12 turn them into `available-unknown`, `available-versionless`, and `available-history-order`. A bare name that is a feature key has no problems.
  - `Pattern` implements §1.3's globs (`*`, `**`, `?`, `{a,b}`, `\`).
- **Tests**: `tests/rules.rs` has a failing fixture for every one of the 60 §20 rules (and each message variant), checking the slug, the variant, and the line marked `#!`. It also checks that every `model-` slug in the registry has a fixture, and that each issue carries every placeholder of the message template it uses. `tests/load.rs` loads the three examples with no issues, checks the queries, order preservation, widget conversion, and the filesystem rules (with a real directory, and with a symbolic link on Unix). 32 tests in `tessera-model` (13 unit, 16 loading, 3 rule-coverage) and 14 new ones in `tessera-core`.

### Interfaces later phases use

- **10:** `load`, then `ContentModel::type_for(path)` / `is_fragment(path)` to pick the schema, then `validate_frontmatter`. `directive_schemas()` is the parser's input. Map slugs to codes through `diagnostics.toml`; message arguments use the registry's placeholder names.
- **11, 12:** `ContentModel::consumer()`, `build(name)`, `editor_default_build()`, `check_availability`, `feature(key)`, `is_versionless`, `state_is_available`, `dimension_values`, and `parse_availability` for `@available` primaries and the `available` frontmatter value. Resolving a feature key and the scope-exceeds check are phase 12's.
- **15:** `load_str_in` for an unsaved buffer plus the project directory; `load_str` skips the filesystem rules.
- **23:** `ContentModel::dimensions` and `image_attributes` keep declaration order; widgets' `Attributes::Declared` does too.

### Decisions

- **Warnings ride in the model**, so `load`'s signature stays `Result<ContentModel, Vec<Issue>>` as the phase file says.
- **Issue locations** are byte spans in `tessera.toml`, in the `FileId` the caller passes (`FileId::new(0)` for `load`). A missing top-level `spec` is reported at offset 0; a missing key inside a table is reported at the table's header.
- **`toml` 0.9 with `preserve_order`**, not `toml_edit`: `toml::de::DeTable` keeps spans and, with the feature, order. The workspace already depends on `toml` 0.9.
- **`model-glossary-link`'s fragment half** doesn't need the file system (it's a path check), so it runs on `load_str` too; only the existence half is skipped there.
- **An empty `[types]` or `[builds]` table** counts as absent: the implicit `page` type and `site` build apply.
- **`model-output-overlaps-content`** compares lexically normalized paths, and, when the project directory is known, resolves symbolic links for the part of each path that exists.
- **A `number` attribute default** is stored as its source text in `DefaultValue::Text` (Q28); a `set` default is `DefaultValue::Set`.
- **A dimension name as a versioned target** (Q29): any version on a dimension name is an error (`VersionlessVersion`, so `model-availability-versionless` / `available-versionless`, with the base message, whose wording says the dimension is "versionless"; a message variant naming the dimension would need a registry change).
- **`role` is reserved** by phase 02's lists, so `full.toml`'s `quill-audience` attribute `role` became `audience` (Q27).
- **`model-editor-build-required`** is reported at the `[editor]` table if there is one, else at the `[builds]` table.
- **Feature spec spans** assume the TOML string has no escape sequences before the error; with escapes, the reported column can shift.

### Left open

- Q27, Q28, and Q29 in `questions.md`.
- `validate_frontmatter` gets a parsed `serde_yaml::Value`, which has lost the original spelling of `3.10`, so it uses the plain `frontmatter-type-mismatch` message, not the registry's `quote` variant. Phase 10 can build that variant, since it has the source text.
- The `model` conformance tag still has its skip entry in `SKIPS.toml`: phase 03 writes the cases and the shared `_model/`, and `tests/conformance` has no `tessera-model` adapter yet. The rule fixtures live in `crates/tessera-model/tests/rules.rs` until then.
- Frontmatter defaults are kept as `serde_yaml::Value` but aren't applied by `validate_frontmatter`; a consumer that wants defaults (the Zod generator, hover) reads `Field::default`.

