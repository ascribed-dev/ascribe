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

### What was built

- **One fork change** (`crates/comrak-tessera`, listed in `FORK.md`): after an image, `close_bracket_match` skips the attribute block that follows it directly, so the block's contents never become emphasis, links, or code (`![a](b){t="*x"} *y*`). It's one marked hunk, 4 lines of code, in `parser/inlines.rs`. The scan is `tessera::image_attributes_len` (in Tessera's own file), and `tessera::unescape_entities` exposes upstream's entity decoder. Nothing else in the inline parser changed: phrases and escapes needed no fork change (see Decisions).
- **`crates/tessera-syntax/src/inline/`**: `extend`, one pass over the finished tree (after every span is final, table cells included), in three files: `mod.rs` (the walk over every block and inline, including phase 06's containers, groups, and `DirectiveLine::title`; it runs after the structure pass and keeps `Arm::title` equal to `opener.title`), `phrase.rs` (candidates, escapes, destinations, fences), `image.rs` (attribute blocks). `convert.rs` has four small edits: it calls `extend`, turns on comrak's `escaped_char_spans`, reads an escape as text whose span includes the backslash, and initializes the new fields.
- **The conformance adapter**: `tests/conformance/tests/adapters/inline.rs` builds the outline's `image` block; `syntax.rs` calls it in one line and handles the `inline` tag. The `inline` skip entry is gone. 45 cases run and pass (the 36 `parser` cases and the 9 `inline` ones).
- **Tests**: `crates/tessera-syntax/tests/inline.rs` (32 tests), unit tests in `inline/phrase.rs` and `comrak_tessera::tessera`, and the span checker (`tests/support`) now checks phrases, destinations, fences, and image blocks over every input the span test reads. The phase 05 test that phrases are never produced now covers only phase 06's nodes.

### Interfaces later phases use

- **`InlineKind::Phrase(Phrase)`**: `{key}` in text (paragraphs, headings, emphasis, link text, alt text, table cells, text primaries, and titles once phase 06 fills them). `Phrase { key, key_span, span }`; the `Inline`'s span equals `span`. The `Text` around it is split, with its decoded value kept. Every candidate is recorded, declared or not.
- **Not inline nodes**: `Link::destination_phrases` and `Image::destination_phrases` (inline forms only), and `CodeBlock::phrases: Option<Vec<Phrase>>`, `Some` only for a fence with the word `phrases=true` in its info string. `Link::destination` and the text nodes are unchanged.
- **Escapes**: `\{key}` is plain `Text` (value `{key}`, span including the backslash) and is listed in `ParsedDocument::escaped_phrases` (spans from the backslash through the `}`), so the undeclared-phrase warning (phase 10) can skip it, and the formatter (phase 23) can find it. A `Text`'s span now always includes the backslash of an escape at its start; before, it didn't.
- **`Image::attributes: Option<ImageAttributes { block }>`** for every form. The image's span covers the block. Issues from the block (`attribute-syntax`, `attribute-bare-key`, `attribute-duplicate-key`, `attribute-unquoted-reserved`) are in `ParsedDocument::issues`; **phase 10 shouldn't report them again**, only unknown keys and value types. A `{` directly after an image with no `}` on its line is reported as `attribute-syntax` and stays text.
- **Reading an image**: `tests/conformance/tests/adapters/inline.rs`.

### Decisions

- **Recognition reads the source, not comrak's text.** comrak has decoded `\{` and `&#123;` to `{` by the time the converter sees text, and it merges adjacent text nodes, so `{key}` can't be recognized from the values. `inline/phrase.rs` scans each text's source span (backslash escapes, then `{key}`) and works out the decoded value of the pieces around a candidate with the same escape and entity rules (`decode`). A test checks values next to escapes and entities.
- **The fork skips the block; `tessera-syntax` finds it with the same function.** So there's no second copy of the "where a block ends" rule, and a test checks that what follows a block is neither lost nor repeated.
- **Autolinks** have no candidates (Q43); reference definitions aren't nodes, so `[ref]: {api}x` has none either (Q43).
- **Images of every form** come from one node, so nothing in the pass depends on how the image closed.

### Left open

- **Q41** (`{key}` directly after an image is an attribute block with a bare key, as `images/attribute-bare-key` expects, not a phrase as §5.1's wording suggests), **Q42** (backslash doesn't escape in a `phrases=true` fence), **Q43** (definitions and autolinks). Each is implemented in the conservative reading and marked `SPEC-QUESTION`. **Q23** has since been resolved as proposed (SPEC §5.3 says so), and nothing is left to do for it.
- **Frontmatter phrases** aren't parsed here (the content model decides which fields, phase 08 and 10).
- **Definitions** (`[ref]: {api}x`) need the parser to expose them (phases 12 and 23).
- The `phrase-undeclared` and `heading-phrase-without-id` cases (tag `check`) and the substitution cases (tag `resolve`) wait for their own phases; an outline can't show candidates, so this phase's coverage of §5.1 is the syntax tests.
