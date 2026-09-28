# Phase 04: Parser spike

**Track:** Parser · **Start after:** 00 · **Parallel with:** 02, 17, and wave C · **Unblocks:** 05 · **Human checkpoint after this phase (go or no-go)**

## Goal

Prove, with a small amount of code, that comrak can be forked to support Tessera's block-level changes to CommonMark. End with a recommendation: continue with the fork, or switch to the fallback (markdown-rs).

## Read first

- [SPEC.md](../../SPEC.md): §1.4, §3.2, §3.4, §3.9.
- [PLAN.md](../PLAN.md): Parser.
- comrak's source, especially its block parser and how it adds extension block types such as tables and front matter.

## Deliverables

- `crates/comrak-tessera/`: comrak, vendored at a pinned release tag, with its license. Add `FORK.md` recording the upstream version, the reason for the fork, the procedure for merging upstream releases, and every changed location.
- A minimal Tessera-line block in the fork.
- Spike tests in the fork's test directory.
- `crates/comrak-tessera/SPIKE.md`: findings and a recommendation.

## Tasks

1. **Vendor comrak.** Copy the source at the latest release tag into `crates/comrak-tessera`, rename the crate, keep the license, and add it to the workspace. Confirm the CommonMark baseline from phase 00 passes unchanged.
2. **Options.** Add a Tessera option to comrak's extension options, carrying the set of known directive keywords (including `end`) and, for each keyword, whether it takes a text primary. Hardcode the built-in set in tests; later phases supply it from the content model.
3. **The Tessera-line block.** Add a leaf block node, `TesseraLine`, holding the raw line and its source position. It's recognized at line start (after container indentation) when the line is `@` followed by a known keyword and then whitespace, `{`, `:`, or the end of the line. It must:
   - **interrupt a paragraph**, as an ATX heading does;
   - **never be a lazy continuation line**, so an unindented directive line after a list item ends the list;
   - be recognized inside list items and blockquotes according to CommonMark's container rules, and be literal text when over-indented into an indented code block;
   - when the keyword takes a text primary and the line has a non-empty one, **continue onto following lines as a paragraph does**. One approach is to give the node a child paragraph holding the primary text, so comrak's inline parser handles it.
4. **Spike tests.** Show that each of these parses correctly:
   - A directive line directly after a paragraph line starts a new block.
   - `1. Install\n@note` ends the list at `@note`.
   - A directive line indented to a list item's content column belongs to the item.
   - A directive line indented four or more spaces past the content column is code.
   - `@unknown: text` and `@astrojs/react` stay paragraph text.
   - `@note {type=caution}: Back up your database` followed by `before you upgrade.` keeps both lines in the primary.
   - A directive line inside a fenced code block is code.
5. **Rerun the CommonMark suite** against the fork with the Tessera option on. Record every newly failing example and why. The only acceptable failures involve lines that are valid directive lines.
6. **Write SPIKE.md**: what worked, what was hard, how intrusive the change is (lines changed and where), how merging future upstream releases would go, and a clear recommendation.

## Acceptance criteria

- [ ] Every spike test in task 4 passes.
- [ ] The CommonMark suite passes with the Tessera option off, and with it on apart from documented, justified exceptions.
- [ ] `FORK.md` lists every changed location, each marked in the code with a `// TESSERA:` comment.
- [ ] `SPIKE.md` ends with a go or no-go recommendation and its reasons.

## Out of scope

- Parsing the directive head (phase 05), structure (phase 06), and inline constructs (phase 07).

## Notes

- Keep changes to comrak small and confined. Every changed line is a future merge conflict.
- If the fork proves impractical, stop, document why in SPIKE.md, and recommend the fallback. The phase is still a success.

## Handoff notes

_To be filled in by the implementing agent._
