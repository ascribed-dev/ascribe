# Phase 23: Formatter

**Track:** Format · **Start after:** 06, 08 · **Parallel with:** 10, 11, and later waves · **Unblocks:** 24

## Goal

Build `tessera-fmt` and `tessera fmt`: rewrite Tessera constructs into canonical form with minimal text edits, never touching the author's ordinary markdown.

## Read first

- [SPEC.md](../../SPEC.md): §8.3, §3.1, §3.3, §3.8, and §10's note on general-purpose formatters.
- [PLAN.md](../PLAN.md): Formatter.
- Phase 05's handoff notes, for the sub-spans on directive lines.

## Deliverables

- `crates/tessera-fmt`: `format(source, &ParseOptions, &ContentModel) -> Vec<TextEdit>`.
- `crates/tessera-cli/src/fmt.rs`: `tessera fmt [paths] [--check]`.

## Tasks

1. **Rules.** Implement every canonical-form rule in SPEC §8.3, each as its own function producing edits:
   - one space between a directive name and `{`;
   - no space inside braces, none around `=` or `|`, and `, ` between pairs;
   - attributes in the order the directive's schema declares them (built-ins from `tessera-core`, widgets from the model);
   - values quoted only when necessary;
   - no empty `{}`;
   - `:` directly after the name or attribute block, then one space before a primary; nothing, including trailing whitespace, after a container's colon;
   - no blank line between a following-block directive and its block;
   - directive lines in list items indented to the item's content column.
2. **Minimal edits.** Only change bytes inside Tessera constructs, the blank line between a directive and its block, and indentation of directive lines. Never reflow or re-render markdown.
3. **Safety.** Skip formatting a construct that has parse errors, rather than guessing.
4. **`tessera fmt`.** Formats files in place; `--check` reports files that would change and exits `1`, for CI.
5. **Tests.** For every conformance input, formatting is idempotent (formatting twice equals formatting once), and the formatted file parses to the same outline as the original. Add focused tests for each rule.

## Acceptance criteria

- [ ] Every §8.3 rule has a test.
- [ ] Idempotence and outline preservation hold across all conformance inputs.
- [ ] A test with hand-wrapped prose, tables, and code shows ordinary markdown is untouched.
- [ ] `tessera fmt --check` exits `1` on unformatted input and `0` on formatted input.

## Out of scope

- Format on save in the editor (phase 24).

## Handoff notes

_To be filled in by the implementing agent._
