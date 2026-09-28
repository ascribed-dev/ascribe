# Phase 05: Tessera lines and syntax tree

**Track:** Parser · **Start after:** 02, 04 (with a go decision) · **Finish after:** 03 · **Parallel with:** 08, 09, 19 · **Unblocks:** 06, 07

## Goal

Build `tessera-syntax`'s public parsing API and its own syntax tree, and parse each directive line's head (name, attributes, primary, trailing colon) with exact source spans.

## Read first

- [SPEC.md](../../SPEC.md): §3.1–§3.4, §8.3, and Appendix A.
- `crates/comrak-tessera/SPIKE.md` and `FORK.md` (phase 04).
- `crates/tessera-core` and `tests/conformance/diagnostics.toml` (phase 02).

## Deliverables

- `crates/tessera-core/src/attributes.rs`: the attribute-block parser for SPEC §3.3, producing phase 02's attribute-value type. Phase 07 reuses it for images.
- In `crates/tessera-syntax`:
  - `parse(source, &ParseOptions) -> ParsedDocument`, where `ParseOptions` carries the directive schemas (built-ins plus project widgets, as `tessera-core` `DirectiveSchema`s).
  - Tessera's own syntax tree, independent of comrak's types. It covers every CommonMark block and inline kind Tessera needs, including images of every form, plus Tessera nodes: directive lines, end lines, and the node kinds for containers, groups, titles, phrase candidates, and image attributes, which phases 06 and 07 fill in.
  - Conversion from comrak's tree to Tessera's.
  - The directive-head parser.

## Tasks

1. **Attribute parser.** Implement SPEC §3.3 and the Appendix A rules exactly: tokens, quoted strings with `\"` and `\\`, value sets, any spaces or tabs, empty `{}`. Return each pair with spans for its key and value. Report malformed input as `Issue`s using registry slugs, never by panicking.
2. **The syntax tree.** Design node types for the whole of Tessera. Every node has a span. Tessera nodes keep sub-spans (name, attribute block and each pair, colon, primary). The formatter (23) and language server (15) depend on these, so document them. Define the node kinds phases 06 and 07 fill in now, so those phases can run in parallel without both editing the tree types.
3. **Directive heads.** For each `TesseraLine`, parse `@name`, optional attributes, optional `:` and primary. A `:` with no primary after it, only whitespace to the end of the line, is a container opener (SPEC §3.1, §3.5); a text primary that ends in `:` is still a primary. Classify the primary by schema: an identifier ends at whitespace, and text is the child paragraph from the fork. Report issues for a primary where the schema allows none, a missing required primary, and malformed attributes. Leave form and container errors to phase 06.
4. **Misspelled directives.** Lines shaped like directives with unknown names (`@word` followed by `{`, `:`, or the end of the line; SPEC §3.2) stay text in the tree. Record an issue with the closest known name for the "did you mean" warning.
5. **Conformance.** Implement the adapter's `outline` for what this phase produces (directive lines and CommonMark blocks, before containers exist). Remove the skip entries for the directive-head and attribute cases, and make them pass.

## Acceptance criteria

- [ ] `cargo test -p tessera-core -p tessera-syntax` passes.
- [ ] Conformance cases for recognition, attributes, and primaries (SPEC §3.1–§3.4) run and pass, including spacing tolerance, every quoting rule, and `@note: Important:`.
- [ ] Property tests (for example with `proptest`) show the attribute parser and head parser never panic on arbitrary input.
- [ ] Every span in the tree covers exactly its source text; a test checks this across all conformance inputs.

## Out of scope

- Containers, groups, titles, and bindings (phase 06).
- Phrases and image attributes (phase 07); leave the node kinds in place.
- Diagnostic codes, severities, and rendering (phase 10). Report slugs only.

## Notes

- Downstream crates use Tessera's tree, never comrak's. Keep comrak an implementation detail of `tessera-syntax`.

## Handoff notes

_To be filled in by the implementing agent._
