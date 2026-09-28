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

_To be filled in by the implementing agent._
