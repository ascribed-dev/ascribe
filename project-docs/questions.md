# Spec questions

[SPEC.md](../SPEC.md) is normative, but it has gaps. This file records every place where an implementing agent found the spec ambiguous or silent, so that a human can resolve it. The protocol is in [phases/README.md](phases/README.md#when-the-spec-is-unclear):

1. **Don't guess silently.** Add an entry below with the section, the ambiguity, the options, and your proposed resolution.
2. **Implement the most conservative option**: the one that reports an error or keeps content rather than dropping it.
3. **Mark the code** with a `// SPEC-QUESTION(Qn)` comment pointing at the entry, and tag any conformance case that depends on it `provisional`, listing the entry in the case's `questions` (see [tests/conformance/README.md](../tests/conformance/README.md)).
4. **Keep going.** Don't edit SPEC.md. A human resolves the question, updates the spec, and records the resolution here; then the `provisional` tags come off.

Contract changes (phase 02) also go through this file: a change to a contract needs an entry approved by a human, listing every phase it affects.

## Entry format

Number entries in order (`Q1`, `Q2`, …) and never reuse a number.

```markdown
### Q<n>: <short title>

- **Section:** SPEC §<n.n>
- **Raised by:** phase <NN>
- **Status:** open | resolved (<date>) | withdrawn
- **Ambiguity:** what the spec doesn't settle, with a minimal example.
- **Options:** the plausible readings, with their consequences.
- **Proposed resolution:** which option, and why. Note which one is implemented now.
- **Affects:** code (`SPEC-QUESTION(Q<n>)` locations), conformance cases, contracts, phases.
- **Resolution:** filled in by a human.
```

## Questions

### Q1: Extra indentation before a directive line

- **Section:** SPEC §1.5, §3.2, §3.9
- **Raised by:** phase 04
- **Status:** open
- **Ambiguity:** §3.2 requires `@` at "line start", which §1.5 defines as "the first character after any indentation or blockquote markers required by the enclosing CommonMark container". Read strictly, a directive line with one to three extra spaces of indentation isn't at line start, so it's ordinary text. But §3.9 says directives "follow CommonMark's container rules, just as headings and code fences do", and rule 5 makes only four or more extra spaces code; headings and fences allow up to three. The readings disagree on, for example:

  ```
  1. Install
    @note: Keep the key safe.
  ```

  The `@note` line is indented two spaces, less than the item's content column (3). As a heading would, it ends the list and is a directive; under the strict reading it's ordinary text and a lazy continuation of the item's paragraph. A top-level `  @end` is either an end line or text.
- **Options:**
  1. Allow up to three spaces of extra indentation, as CommonMark does for every other block start. Indentation that doesn't reach a list item's content column then puts the directive outside the item, which may surprise authors, but it's how headings behave.
  2. Require `@` at exactly the container's content column. Slightly indented directive lines become text, silently: nothing reports that the author's `@note` or `@end` was ignored (an `@end` that isn't recognized leaves its container unclosed, which is reported, but elsewhere).
- **Proposed resolution:** option 1. It's what "follow CommonMark's container rules, just as headings do" means, and it keeps the block parser's rule uniform. Phase 06 still reports an end line indented differently from its opener (§3.9 rule 3), and the formatter (phase 23) can remove the extra spaces. Implemented now: option 1.
- **Affects:** `crates/comrak-tessera` (recognition in `open_new_blocks`; the test `up_to_three_spaces_of_extra_indentation_are_allowed` in `tests/spike.rs`); phases 05, 06, and 23; conformance cases with indented directive lines.
- **Resolution:**

### Q2: A setext underline, table delimiter row, or link reference definition in a text primary

- **Section:** SPEC §3.4
- **Raised by:** phase 04
- **Status:** open
- **Ambiguity:** a text primary "continues onto the following lines exactly as a paragraph does: until a blank line, a directive line, or any other line that would interrupt a paragraph", and is "parsed as CommonMark inline content". Three CommonMark rules don't interrupt a paragraph but turn it into something else, and the spec doesn't say what they do to a primary:

  ```
  @note: Title
  ===
  @note: Title
  ---
  @note: a | b
  --|--
  @note: [label]: /url
  ```

  In a paragraph, `===` and `---` make a setext heading, a GFM delimiter row makes a table, and a leading `[label]: /url` is a link reference definition rather than text.
- **Options:**
  1. The primary is always inline content. `===` and a delimiter row are continuation text; `---` can't be an underline, so it interrupts the primary as a thematic break (as it would a paragraph with setext headings off); `[label]: /url` is text.
  2. Apply the paragraph rules: the primary becomes a heading or table, or loses its reference definitions. This contradicts "parsed as CommonMark inline content", and a note's content would become a heading inside the directive line.
  3. Treat an underline or delimiter row as ending the primary and starting a new block.
- **Proposed resolution:** option 1. It follows from the primary being inline content, and it keeps every line of content where the author wrote it. Implemented now: option 1.
- **Affects:** `crates/comrak-tessera` (`is_text_primary` in `src/parser/tessera.rs` and its three call sites in `src/parser/mod.rs`; the test `text_primary_never_becomes_a_block` in `tests/spike.rs`); phases 05 and 23; conformance cases with multi-line primaries.
- **Resolution:**
