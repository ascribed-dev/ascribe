# Phase 07: Inline extensions

**Track:** Parser · **Start after:** 05 · **Finish after:** 03 · **Parallel with:** 06 · **Unblocks:** 10, 11

## Goal

Recognize Tessera's two inline constructs, phrase candidates (`{key}`) and attribute blocks after images, with exact spans, in every place the spec allows them.

## Read first

- [SPEC.md](../../SPEC.md): §2.3 (escapes), §5.1 (phrases), §5.3 (images), §3.3 (attributes).
- Phase 05's handoff notes, for the tree types and the attribute parser.

## Deliverables

- Changes to the inline parser in `crates/comrak-tessera`, marked `// TESSERA:` and listed in `FORK.md`.
- `crates/tessera-syntax/src/inline/`: conversion of the new inline nodes into Tessera's tree.

## Tasks

1. **Phrase candidates.** In text (paragraphs, headings, link text, text primaries), recognize `{key}` where `key` matches the phrase-key syntax (SPEC Appendix A) as a phrase candidate node with its span. Whether a key is declared is decided later (phase 12); the parser records every candidate.
2. **Escapes.** `\{` must produce literal text, not a candidate. CommonMark's inline parser removes the backslash, so detect the escape before that happens, and record that the brace was escaped.
3. **Link destinations.** CommonMark treats a link destination as one string. Record the spans of phrase candidates inside destinations (`[text]({api}streaming)`) without changing how the destination parses.
4. **Code.** Never recognize candidates in code spans, indented code, or raw HTML. In fenced code, recognize them only when the info string contains the word `phrases=true`, and record which fences opted in.
5. **Image attributes.** An attribute block immediately after a recognized **image node**, with no space, attaches to the image (SPEC §5.3). This covers every CommonMark image form: inline (`![alt](src){…}`), full reference (`![alt][ref]{…}`), collapsed reference (`![alt][]{…}`), and shortcut reference (`![alt]{…}`). Attach based on the image node the inline parser produced, not on a particular closing delimiter. Parse the block with `tessera-core`'s attribute parser and keep spans. `{…}` after a space, or after anything but an image, is ordinary text.
6. **Distinguishing the two.** `{…}` containing `=` directly after an image is an attribute block; a bare `{key}` in text is a phrase candidate (SPEC §5.1).
7. **Conformance.** Remove the skip entries for phrase, escape, and image-attribute cases, and make them pass.

## Acceptance criteria

- [ ] Conformance cases for SPEC §5.1 recognition and §5.3 run and pass, including full, collapsed, and shortcut reference images with attributes.
- [ ] `\{key}`, `{key}` in a code span, and `{key}` in a fence without `phrases=true` produce no candidates; tests cover each.
- [ ] Candidates in link destinations have correct spans, and the link's destination is otherwise unchanged.
- [ ] The CommonMark suite still passes apart from the documented differences.

## Out of scope

- Deciding whether a key is declared, and substituting values (phase 12).
- The undeclared-phrase warning (phase 10 reports it from the candidates).

## Notes

- Keep the fork's inline changes as small as possible; most logic can live in `tessera-syntax` after conversion.
- Phase 06 runs in parallel in `tessera-syntax/src/structure/`. Don't edit files outside `src/inline/` and the fork's inline parser without raising it.

## Handoff notes

_To be filled in by the implementing agent._
