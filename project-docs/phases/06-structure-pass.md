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

### What was built

- **`crates/tessera-syntax/src/structure/`**, run by `parse()` after tree conversion (two lines in `convert.rs`, plus the new fields in `tree.rs`):
  - `mod.rs`: the walk. It works one *scope* at a time (the document, a list item, a block quote), recursing into list items and block quotes as it meets them.
  - `titles.rs`: attaches title lines, and reports `title-not-accepted` and `title-dot-space`.
  - `nest.rs`: forms, containers, groups, arms, end lines, nesting, and the `@variant` and required-title rules.
  - `bind.rs`: bindings, `@steps`, and the helpers `bound_heading` and `bound_block`.
  - `lists.rs`: `list-ended-by-directive`, `directive-indented-code`, `steps-numbering-continued`.
- **The tree**: `BlockKind::Container`, `Group` (with `Arm`s) are produced. A title is **attached** to its directive, as `DirectiveLine::title` (a `TitleLine` with `span`, `dot`, `content`, and `inlines`), and the title line stops being a block. `DirectiveLine::binding: Option<Bound>` (`Own`, `Heading`, `FollowingBlock`, `Unbound`). `BlockKind::Title` is kept but never produced (documented; removing it would have touched `match`es in phase 07's files).
- **The conformance adapter**: `syntax.rs` handles `structure` as well as `parser` and delegates the Tessera nodes to `tests/adapters/structure.rs`. The stand-in binding logic is gone. The `structure` skip entry is removed.
- **`tests/conformance/tests/structure_rows.rs`**: see "Conformance" below.
- **Questions** Q31 to Q36 (below), and implementation notes on Q13 and Q16 to Q19.

### Interfaces later phases use

- **What stays flat.** After the pass, a `BlockKind::Directive` is always a line-form directive (a sibling of the blocks around it); every opener is the `opener` of a `Container` or an `Arm`. A `BlockKind::End` left in the tree is one that closes nothing, and was reported. `Container::end` / `Group::end` hold the end line that closed them, `None` when unclosed (also reported).
- **Spans.** A container's block runs from its title line (if any) through its end line, or through its last block when it's unclosed. An arm runs from its title line or opener through its last block (not the group's end line). A line-form directive's block starts at its title line, but `DirectiveLine::span` still starts at the `@`. `tests/support/mod.rs` checks all of this across every input the phase 05 span test uses.
- **Bindings.** `Bound::Heading`'s heading is found with `bound_heading(blocks, i)`, and `Bound::FollowingBlock`'s block with `bound_block(blocks, i)`, both in the same list of blocks. Following-block directives stack (Q31): all of them describe the block the last touches, and a line-form directive with its own text primary (`@note: text`) counts as that block. `Bound::Unbound` means binding failed and was reported (or the head was unreadable, Q36). A container opener has no binding.
- **Issues** (in `ParsedDocument::issues`, sorted by position, with the registry's message arguments): `container-unclosed`, `container-colon-unexpected`, `container-colon-missing`, `container-open-at-arm`, `end-unmatched`, `end-indent-mismatch`, `container-nesting-deep`, `binding-no-block`, `binding-heading`, `binding-blank-line`, `binding-not-section-top`, `title-not-accepted`, `title-dot-space`, `variant-arm-kind`, `variant-mixed-arms`, `variant-no-shared-dimension`, `steps-not-ordered-list`, `details-title-missing`, `widget-schema` (only a missing required title on a widget), `list-ended-by-directive`, `directive-indented-code`, `steps-numbering-continued`. **Phase 10 shouldn't report these again**; it reports what needs content-model data (`variant-unknown`, attribute types and keys, `widget-schema` for attributes) and the phase 05 issues stay in the parser.
- **Locations.** Container, end-line, binding, and arm diagnostics are at the directive's `@name` (`container-colon-unexpected` at the colon, end-line ones at the `@end`, group ones at the group's first opener, title ones at the title line, list ones at the `@name` or the list's first marker).

### Decisions

- **One walk per scope, one stack of frames.** A group is one frame, and its arms live in it, so `@end` pops a whole group, nesting counts a group once, and "join the nearest open group of the same name in the same scope" is a search down the stack. Containers still open inside the arm when the next opener arrives are popped, reported, and left unclosed inside the arm they were opened in.
- **Forms** follow the line's colon (Q16 option 1). A container-only directive is an opener even without its colon.
- **Cross-scope end lines** (Q19): the pass keeps the stack of open container names across scopes, and a list of containers left unclosed by scopes that ended (the "orphans"), so a stray `@end` in the next list item is `end-indent-mismatch` and claims the orphan.
- **A directive with an unclosed attribute block** is `Unbound` and gets no binding or colon diagnostics (Q36); it's what makes `attributes/unclosed-brace` give exactly one diagnostic.
- **Heading sections are per list of blocks** (Q18, Q32). `@include`, `@note: text`, and other own-bound directives are content: `@id` after one isn't at the top of its section.
- **The case `lists/steps-numbering-continued` was corrected** (Q35): its `2. Two.` can't end the note's text primary under CommonMark's rule (and so under SPEC §3.4), so the input has a blank line and the diagnostic moved to line 9.
- **`directive-indented-code`** looks at the source lines of every indented code block, not at its decoded text, so it needs no line bookkeeping for tabs.

### Conformance

- `cargo test -p tessera-conformance --test conformance`: **85 passed, 0 failed, 227 skipped** (the 36 `parser` cases from phase 05 and 49 `structure` cases). Every `structure`-tagged case is either in that run or in the eight below.
- **Eight `structure` cases also carry the `check` tag** (they expect diagnostics), so the runner skips them until phase 10 has an adapter for `check`, and so does every `check` case for a structural row. `tests/conformance/tests/structure_rows.rs` runs them now: it runs the outline of every `structure` case, and for every case tagged only with `parser`, `structure`, and `check` it compares the **structural** diagnostics (the list in the test) with the expectation's, at the expected lines, and fails if a row in that list has no case. That's 50+ cases, including every §8.2 structural row, the provisional cases for Q16 to Q19, and the negative ones (`diagnostics: []`). When phase 10 lands, the runner covers these cases whole, so **phase 10 should delete `structure_rows.rs` (or fold it into its `check` adapter) when it removes the `check` skip entry**, so two runners don't drift.
- **Appendix B**: `samples/appendix-b` passes, and `appendix_b_has_no_issues` (in `crates/tessera-syntax/tests/structure.rs`) parses `examples/quill/docs/install-agent.md` with no issues and checks the group inside the list item.
- **Property tests** (`crates/tessera-syntax/tests/structure.rs`, 2,000 cases plus a text-soup run): the pass never panics; no line-form container opener stays flat; the number of unclosed containers and groups equals the number of `container-unclosed` plus `container-open-at-arm` issues; the number of end lines left over equals the number of `end-unmatched` plus `end-indent-mismatch` issues; issues stay in source order; every span is a valid range.
- The phase 05 tests were adapted for the new tree (`parse.rs`, `agreement.rs`, `support/mod.rs`), and `harness.rs`'s skipped-sample test now uses `samples/include-and-selection`, since `samples/appendix-b` is no longer skipped.

### Left open

- **Q31 to Q36** are new (stacked following-block directives; sections per container; nesting across list items; the exact triggers of the list warnings; text primaries and `2.`; unreadable heads). Each has an implemented proposal marked `SPEC-QUESTION`. All six were resolved on 2026-09-28 as proposed (Q31 extended to one-line notes, as implemented), and SPEC §3.3–§3.10 now state them.
- **Q13, Q16 to Q19** are implemented as proposed; none of the proposals proved wrong or unimplementable. Q13 needed nothing from the structure pass (the fork decides recognition).
- **`BlockKind::Title`** is never produced. Remove it when no other phase's code matches on it.
- **Not done here**: which arms a build keeps, `@variant`'s dimension names and values, `@available` specs, and `@id` characters are phase 10 and later; `@include` isn't followed (phase 11).

