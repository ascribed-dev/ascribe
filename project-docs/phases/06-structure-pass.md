# Phase 06: Structure pass

**Track:** Parser · **Start after:** 05 · **Finish after:** 03 · **Parallel with:** 07 · **Unblocks:** 10, 11, 23

## Goal

Turn the flat sequence of directive lines and blocks into Tessera's structure: containers, groups and arms, titles, and bindings. Report every structural error the spec defines.

## Read first

- [SPEC.md](../../SPEC.md): §3.5–§3.10 and §4 (each directive's forms, binding, and title rules).
- [PLAN.md](../PLAN.md): Parser, especially the structure-pass bullets.
- Phase 05's handoff notes, for the tree types.

## Deliverables

- `crates/tessera-syntax/src/structure/`: the structure pass, run by `parse()` after tree conversion.
- Filled-in container, group, arm, title, and binding nodes.
- Conformance adapter support for the full outline.

## Tasks

1. **Forms.** Decide each directive's form from its own line and its schema (SPEC §3.5): a `:` with an empty primary opens a container, and nothing else does. Report a trailing colon on a directive with no container form, and a container-only directive missing its colon.
2. **Containers.** Within each parent (the document, a list item, a blockquote), walk the children with a stack. An end line closes the innermost open container. Report an end line with no open container, a container still open when its parent ends, and an end line at a different indentation from its opener (SPEC §3.9).
3. **Groups.** Implement SPEC §3.6: consecutive openers of a groupable directive form a group, each opener ends the previous arm, and the end line belongs to the group. A new opener joins the nearest open group of the same directive within the same CommonMark container; report containers still open inside the previous arm at the new opener's line. Check the `@variant` rules: arms have attributes or a title but not both, arms are all labeled or all dimensional, and dimensional arms share a key.
4. **Titles.** A one-line paragraph that starts with `.` followed by neither whitespace nor `.`, and sits directly above a directive whose schema accepts a title, becomes that directive's title (SPEC §3.7, PLAN.md). A `.` line inside a longer paragraph stays text. Report a title on a directive that doesn't accept one, a missing required title (`@details`), and the `. Title` warning.
5. **Binding.** Implement SPEC §3.8:
   - Heading-bound directives bind their section when they're at the top of it, with blank lines between the heading and the directives allowed. Compute sections (a heading plus content up to the next heading of the same or a higher level).
   - Following-block directives bind the next block in the same container. Warn on a blank line between them. Report a missing block and an attempt to bind a heading.
   - `@available` binds its section at the top of a section and otherwise the following block (SPEC §4.4).
   - Report heading-bound directives that aren't at the top of a section.
6. **Directive-specific structure.** `@steps` must bind an ordered list. Enforce each built-in's forms and bindings from its schema, and project widgets' from theirs.
7. **Nesting.** Warn when containers nest more than two levels deep (SPEC §3.10).
8. **List heuristics.** Warn when an ordered list continues the numbering of a `@steps` list that just ended (SPEC §8.2), and when a directive line was over-indented into code.
9. **Conformance.** Extend the adapter to produce the full outline. Remove the skip entries for structure cases, and make them pass.

## Acceptance criteria

- [ ] Every conformance case tagged for SPEC §3.5–§3.10 and §4 structure runs and passes.
- [ ] Every structural row of the §8.2 table has a passing case that reports it at the expected line.
- [ ] Appendix B's page produces the expected outline with no issues.
- [ ] Property tests show the pass never panics and every opened container is either closed or reported.

## Out of scope

- Anything across files (phases 11 and 12).
- Checks that need content-model data, such as whether a dimension value exists (phase 10).

## Notes

- Everything here works from one file and the directive schemas. If a rule seems to need another file, it belongs in a later phase.
- Put the error for a missing `@end` where the reader can fix it: at the next arm when the arm rule applies, otherwise at the unclosed opener.

## Handoff notes

_To be filled in by the implementing agent._
