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

These numbers are separate from the decisions in [content-model.md](content-model.md) §21, which that document calls Q1–Q21. A bare Qn in code, in a conformance case's `questions`, or in the diagnostics registry's `provisional` means an entry here. Q1 and Q2 come from phase 04.

### Q1: Extra indentation before a directive line

- **Section:** SPEC §1.5, §3.2, §3.9
- **Raised by:** phase 04
- **Status:** resolved (2026-09-28)
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
- **Resolution:** option 1, as proposed and implemented. SPEC §1.5 and §3.9 rule 5 now say so.

### Q2: A setext underline, table delimiter row, or link reference definition in a text primary

- **Section:** SPEC §3.4
- **Raised by:** phase 04
- **Status:** resolved (2026-09-28)
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
- **Resolution:** option 1, as proposed and implemented. SPEC §3.4 now says so.

### Q3: §8.2 rows that join a file-level and a page-level check

- **Section:** SPEC §8.1, §8.2
- **Raised by:** phase 02
- **Status:** resolved (2026-09-28)
- **Ambiguity:** the diagnostics registry gives every diagnostic one level, because the level decides where it's reported (once per file, or per page and build) and where a conformance case expects it. Four §8.2 rows join two checks at different levels in one condition:
  - `@include` "Target file or id doesn't exist": whether the file exists is file level (§8.1, "whether referenced files exist"); whether the id exists needs the target's headings.
  - Links "Target file or id doesn't exist": the same, and §8.1 names "link targets that are ids" as page level.
  - Links "Target is a fragment, or an id that exists only inside a fragment": a fragment path is known from the path and the model; an id inside an included fragment needs the expanded page.
  - Headings "No `@id`, and the heading contains a phrase or duplicates another heading's text": a phrase is visible in the file; a duplicate "on the same page" (§5.5) needs the expanded page.
- **Options:**
  1. One registry entry per row and level: eight entries for these four rows, each with its own slug and code. Every problem is reported at the level where it can be found, including in fragments that no page includes.
  2. One entry per row, at page level. Problems in unused fragments are never reported, and a missing file is reported once per build instead of once.
  3. One entry per row, at file level. File-level checks can't see includes, so duplicate headings from fragments and fragment-only ids are never reported.
- **Proposed resolution:** option 1, which reports every error and gives each problem a precise message and fix. It changes phase 02's acceptance criterion from "exactly one entry per §8.2 row" to "exactly one entry per row and level"; the registry test (`tests/conformance/tests/registry.rs`) lists the four split rows. Implemented now: option 1.
- **Affects:** `tests/conformance/diagnostics.toml` (`include-target-missing`, `include-id-missing`, `link-target-missing`, `link-id-missing`, `link-to-fragment`, `link-id-in-fragment`, `heading-phrase-without-id`, and `heading-duplicate-without-id`, each `provisional` on Q3); phases 03, 10, 11, and 14.
- **Resolution:** option 1, as proposed and implemented. SPEC §8.2 now splits each of the four rows into a file-level row and a page-level row, so the registry has exactly one entry per row, and none is provisional.

### Q4: The kind of `@available`'s primary

- **Section:** SPEC §3.4, §4.4
- **Raised by:** phase 02
- **Status:** resolved (2026-09-28)
- **Ambiguity:** §3.4 defines two primary kinds: an identifier, which ends at the first whitespace, and text, which is inline content that continues onto following lines like a paragraph. `@available`'s primary, an availability spec such as `cloud, self-managed preview 3.4`, contains spaces, so it isn't an identifier, but it isn't markdown either. If it's a text primary, a paragraph directly below a block-level `@available` becomes part of the spec:

  ```
  @available: cloud
  Streaming sync pushes changes as you save.
  ```
- **Options:**
  1. A third kind: the rest of the directive line, trimmed, not parsed as inline content and not continued. The paragraph is the bound block.
  2. A text primary. The paragraph joins the spec, which then fails to parse.
- **Proposed resolution:** option 1. It keeps the paragraph as content, and it fits every example in §4.4, which are all one line. Implemented now: option 1, as `Primary::Availability` in `tessera-core`.
- **Affects:** `crates/tessera-core/src/schema.rs` (`SPEC-QUESTION(Q4)`); phases 04, 05, and 08.
- **Resolution:** option 1, as proposed and implemented. SPEC §3.4 now defines a line primary, and §4.4 gives `@available` one.

### Q5: Problems §8.2 has no row for

- **Section:** SPEC §3.3, §4.1, §4.4, §5.3, §8.2
- **Raised by:** phase 02
- **Status:** resolved (2026-09-28)
- **Ambiguity:** the spec states these rules, but §8.2 has no diagnostic for breaking them:
  - an attribute block that doesn't parse, such as an unclosed quote or brace, or `=` with no value (§3.3 grammar);
  - the same attribute key twice in one block (`{type=tip, type=note}`);
  - an availability spec that doesn't parse (`@available: cloud (preview)`), in a directive or in `available` frontmatter (§4.4 grammar);
  - an `@id` value with characters other than letters, digits, and hyphens (§4.1);
  - an image missing a required image attribute. content-model.md §6.1 makes an attribute without `?` required; §8.2's "Project widget" row covers widgets, but nothing covers images.
- **Options:**
  1. Add a row for each, as an error, with provisional registry entries until the spec has them.
  2. Report them under the nearest existing row, such as "Unquoted value containing a reserved character", with a misleading message.
  3. Don't report them. A malformed spec or id would then be silently accepted or dropped.
- **Proposed resolution:** option 1. Implemented now: registry entries `attribute-syntax`, `attribute-duplicate-key`, `available-syntax`, `id-invalid`, and `image-attribute-missing`, each `provisional` on Q5.
- **Affects:** `tests/conformance/diagnostics.toml`; `crates/tessera-core/src/diagnostics.rs`; phases 03, 05, 07, 08, and 10.
- **Resolution:** option 1, as proposed and implemented. SPEC §8.2 now has the five rows, as errors, and their registry entries aren't provisional.

### Q6: What "an id that exists only inside a fragment" means

- **Section:** SPEC §4.2, §5.2, §5.5, §8.2
- **Raised by:** phase 02
- **Status:** resolved (2026-09-28)
- **Ambiguity:** §4.2 says "a link to a fragment file, or to an id that exists only inside one, is an error. Link to the page that includes it." But a link's `#id` names "a heading in the target file by its source id" (§5.2), and source ids "depend only on the file" (§5.5). So for `[x](setup.md#prereq)`, where `prereq` is a heading in `_fragments/prereq.md` that `setup.md` includes, `setup.md` has no source id `prereq`, and the link already fails as "id doesn't exist". It's unclear whether:
  - the fragment rule gives that case a clearer error (the id exists, but only in an included fragment), so no link can name a heading that comes from a fragment; or
  - links to `setup.md#prereq` are meant to work, with a page's ids including its fragments' headings, and the error is only for `_fragments/prereq.md#prereq`.
- **Options:**
  1. Source ids are per file, as §5.5 says. A link to an id that only an included fragment has is an error (`link-id-in-fragment`, page level), whose message names the fragment; authors link to the page.
  2. A page's linkable ids include the source ids of the fragments it includes, so `setup.md#prereq` works. §5.2 and §5.5 would need rewording, and ids from a fragment included twice would be ambiguous.
- **Proposed resolution:** option 1, which reports an error rather than guessing a target, and follows §5.5's definition. Implemented now: option 1.
- **Affects:** `tests/conformance/diagnostics.toml` (`link-id-in-fragment`, `provisional` on Q3 and Q6); phases 03, 11, 12, and 14.
- **Resolution:** option 1, as proposed and implemented. SPEC §4.2 and §5.2 now say a page's linkable ids are its own source ids, and a link to an id that exists only in an included fragment is an error that names the fragment.

### Q7: Whether an explicit `@id` takes part in slug numbering

- **Section:** SPEC §4.1, §5.5
- **Raised by:** phase 02
- **Status:** resolved (2026-09-28)
- **Ambiguity:** duplicate slugs are numbered "the way that algorithm numbers them". With

  ```
  ## Intro
  @id: intro

  ## Intro
  ```

  the second heading's slug is either `intro` (the explicit id isn't a slug, so the slugger never saw it), which duplicates the first heading's id, or `intro-1` (explicit ids count as used).
- **Options:**
  1. Explicit ids don't take part in numbering. The second heading is `intro`, a duplicate id on the page, which is an error. This is what Astro does: its heading-id plugin skips headings that already have an id without recording them.
  2. Explicit ids are recorded first, so the second heading becomes `intro-1`. No error, but the numbering no longer matches the consumer's algorithm, which §5.5 requires.
- **Proposed resolution:** option 1. It reports an error rather than silently choosing an id, and it matches the consumer. Implemented now: option 1, stated on `tessera_core::SlugScope::slug`.
- **Affects:** `crates/tessera-core/src/consumer.rs`; phases 09, 11, and 12.
- **Resolution:** option 1, as proposed and implemented. SPEC §5.5 now says explicit ids don't take part in slug numbering, so a slug equal to an explicit id is a duplicate id.

### Q8: `<tessera-note title>` is also HTML's `title` attribute

- **Section:** SPEC §9.4
- **Raised by:** phase 02
- **Status:** resolved (2026-09-28)
- **Ambiguity:** §9.4's table gives a note's title as `<tessera-note type="tip" title="…">`. `title` is a global HTML attribute: browsers show it as a tooltip whenever the pointer is anywhere over the element, so every titled note shows its title as a tooltip over its whole body, and assistive technology may announce it as the element's description.
- **Options:**
  1. Keep `title`, as the spec says, and accept the tooltip.
  2. Use another name, such as `heading`. Needs a spec change.
- **Proposed resolution:** option 2, `heading`, which the element library would show the same way. Implemented now: option 1, since the spec names the attribute; the element contract marks it provisional.
- **Affects:** `packages/elements/CONTRACT.md`; phases 19 and 20.
- **Resolution:** option 2: the attribute is `heading` (`<tessera-note type="tip" heading="…">`). SPEC §9.4, the element contract, and the render fixtures use it, and so do widget elements, for their title lines. The note's `label` attribute stays.

### Q9: Attribute names that clash with HTML

- **Section:** SPEC §5.3, §6, §9.4
- **Raised by:** phase 02
- **Status:** resolved (2026-09-28)
- **Ambiguity:** the site output writes a widget as "a custom element with the widget's name and attributes", and the site-render contract sets image attributes on the `<img>`. Nothing stops a content model from declaring attribute keys that mean something else in HTML:
  - on widgets: `title`, which the element contract uses for the widget's title line, and HTML's global attributes (`id`, `class`, `style`, `hidden`, `slot`, and any `on…` event handler);
  - on images: `src`, `alt`, and `title`, which CommonMark's image syntax already supplies, and the same global attributes.

  An image attribute `style` or a widget attribute `onclick` would pass straight into the published HTML.
- **Options:**
  1. The loader rejects these keys, with a new loader rule (for example `model-attribute-reserved`) in content-model.md §20.
  2. The emitter prefixes them (`data-…`), which changes what "the widget's attributes" means.
  3. Allow them. A marker attribute replaces the image's own attribute of the same name.
- **Proposed resolution:** option 1. Implemented now: nothing rejects them. The site-render contract says a marker attribute replaces an existing attribute of the same name (option 3), so no author content is dropped, and marks that rule provisional.
- **Affects:** `project-docs/content-model.md` §14, §15, §20; `tests/conformance/diagnostics.toml`; `project-docs/contracts/site-render.md`; `packages/elements/CONTRACT.md`; phases 08, 19, 20, and 21.
- **Resolution:** option 1, as proposed. SPEC §7.2 now forbids these keys, and content-model.md §20.3 has the loader rule `model-attribute-reserved` (registry TSR119). For widgets the reserved names are `heading` and `primary` (Q8 renamed the title attribute), plus HTML's global attributes, which include `title`, ARIA attributes, and event handlers. Amended at the human's request: the event handlers are HTML's event-handler attribute names as an explicit list (`tessera_core::reserved::HTML_EVENT_HANDLER_ATTRIBUTES`), not every key starting with `on`, so keys such as `online` and `only-if` are allowed. The site-render contract no longer depends on replacing an image's own attributes.

### Q10: Which local files an output may copy

- **Section:** SPEC §4.2, §5.2, §5.3, §9.4 (Assets)
- **Raised by:** phase 02
- **Status:** resolved (2026-09-28)
- **Ambiguity:** relative paths may lead out of the content root (`../../shared/logo.png`), and the spec doesn't limit where. Copying any file a destination can reach would let a page publish files from anywhere on the machine, or copy a previous build's output back into the build. The spec also doesn't say whether a reference whose case differs from the file's (`Logo.png` for `logo.png`) exists, which differs between file systems, or whether references inside raw HTML (`<img src="x.png">`) are assets.
- **Options:**
  1. A reference must resolve to a file inside the project root (the directory of `tessera.toml`) or the content root, and not inside the output directory; otherwise it's reported as not existing. Names must match exactly, on every platform. References in raw HTML aren't assets and pass through unchanged.
  2. Allow any file the process can read.
  3. Allow only files inside the content root.
- **Proposed resolution:** option 1. It supports shared images in a monorepo, keeps builds the same on macOS, Windows, and Linux, and reports an error rather than copying an unexpected file. Implemented now: option 1, in the asset contract, with the `outside` and `case` message variants of `image-source-missing` and `link-target-missing`.
- **Affects:** `project-docs/contracts/assets.md`; `tests/conformance/diagnostics.toml`; phases 10, 11, 18, 20, and 25.
- **Resolution:** option 1, as proposed and implemented. SPEC §9.4 (Assets) now says so, and the asset contract and registry entries aren't provisional.

### Q11: A title line above a directive that doesn't take one

- **Section:** SPEC §3.7, §8.2
- **Raised by:** phase 02
- **Status:** resolved (2026-09-28)
- **Ambiguity:** §3.7 says "If the next line isn't a directive that accepts a title, the `.` line is ordinary text", but §8.2 has the error "Title given to a directive that doesn't accept one". For

  ```
  .NET 8 is required.
  @steps
  1. Install the SDK.
  ```

  the first line is either a paragraph (§3.7) or an error (§8.2).
- **Options:**
  1. The line is a paragraph for structure, and the error is still reported, since a `.` line touching a directive is almost always a misplaced title; `\.` silences it.
  2. The line is ordinary text and nothing is reported, so the §8.2 row never applies.
- **Proposed resolution:** option 1: the tree follows §3.7 and the diagnostic follows §8.2, which reports a probable mistake rather than hiding it. Implemented now: nothing yet; the registry entry `title-not-accepted` exists, and phases 06 and 10 report it.
- **Affects:** `crates/tessera-core/src/schema.rs` (`TitleRule::None`); phases 03, 06, and 10.
- **Resolution:** a variant of option 1: the `.` line stays a paragraph (§3.7), and the §8.2 row "Title given to a directive that doesn't accept one" is a warning, not an error, because a real sentence such as `.NET 8 is required.` directly above `@steps` mustn't fail a build. `\.` silences it. SPEC §3.7 and §8.2 and the registry entry `title-not-accepted` now say so.

### Q12: A titled note shows its type only by color

- **Section:** SPEC §4.5, §9.7; `packages/elements/CONTRACT.md` §1
- **Raised by:** phase 19
- **Status:** resolved (2026-09-28)
- **Ambiguity:** The contract says a note's heading line shows `heading`, "or `label` when there's no heading". A note with a `heading`, such as `<tessera-note type="warning" label="Warning" heading="Back up your database first">`, then shows only "Back up your database first", and its type reaches the reader only through the accent color. A reader who can't tell the colors apart can't tell a warning from a tip (WCAG 1.4.1, Use of Color), and assistive technology gets no type either, because generated content is the only place the label appears.
- **Options:**
  1. Keep the contract: `heading`, or `label` without one. The type is color-only when there's a heading.
  2. Show both when there's a heading: `content: attr(label) ": " attr(heading)`, giving "Warning: Back up your database first". A note with no heading still shows the label alone. Only the element library's CSS changes; the emitter's markup is unchanged.
  3. Show the label as a separate visually distinct line or badge above the heading. More layout, same information.
- **Proposed resolution:** Option 2. It keeps the type readable without color and needs one CSS rule, with no change to the markup phase 20 emits. Implemented: option 2 (approved).
- **Affects:** `packages/elements/css/style.css` (`tessera-note[heading]::before`); contract §1 (its rendering paragraph); phase 19's tests of the note heading; no other phase, since the markup is unchanged. Plain-markdown output already shows the type (`**Tip: …**`).
- **Resolution:** Approved by the repository owner: option 2. `CONTRACT.md` §1 and `css/style.css` now show `label: heading`; the tests were updated.

### Q13: A directive line inside a code span that started on an earlier line

- **Section:** SPEC §3.2, §3.9
- **Raised by:** phase 03
- **Status:** resolved (2026-09-28)
- **Ambiguity:** SPEC §3.2 says a line is a directive line only if it "is not inside a code span, fenced code block, indented code block, or raw HTML block". Fences, indented code, and HTML blocks are block structure, so a block parser knows about them. A code span is inline: it can span lines of one paragraph, and a block parser reads line starts before it knows whether a span is open:

  ```
  Use `code
  @note: A note.
  more` here.
  ```

  Read literally, the `@note` line is inside the span, so it's text and the span closes on the third line. But §3.9 rule 4 says directive lines "interrupt paragraphs and never continue them, like a heading", and CommonMark lets a heading interrupt a paragraph even inside an open code span (block structure has priority over inline structure).
- **Options:**
  1. Block structure wins, as for a heading: the directive line interrupts the paragraph, the span never closes, and its backticks are literal. The first line is a paragraph `Use \`code`, then a note whose text primary is `A note. more\` here.`
  2. The code span wins: the line is text, and the paragraph is `Use \`code @note: A note. more\` here.` A block parser can't do this without an inline pass first; it would make a directive line's meaning depend on later lines.
- **Proposed resolution:** option 1. It keeps recognition a property of the line and its block context, which is what the block parser (phase 04) does, and it matches headings. SPEC §3.2 should say "inside a fenced code block, an indented code block, or a raw HTML block" and drop "code span", which can only apply within a line. Implemented in the fork now: option 1 (see `crates/comrak-tessera/SPIKE.md`).
- **Affects:** conformance cases: `recognition/code-span-across-lines`; phases 05, 06.
- **Phase 06:** the structure pass needs nothing here: recognition happens in the fork, and the structure pass only sees the lines it produced. The proposal is what the case expects.
- **Resolution:** approved by the repository owner: option 1: block structure wins, as for a heading. SPEC §3.2 now lists only fenced code, indented code, and raw HTML blocks, and says a directive line interrupts a paragraph even inside an unclosed code span.

### Q14: Which diagnostic a malformed attribute value gets

- **Section:** SPEC §3.3, §8.2
- **Raised by:** phase 03
- **Status:** resolved (2026-09-28)
- **Ambiguity:** §8.2 has two rows for attribute blocks that break §3.3's grammar: "Unquoted value containing a reserved character" and "Attribute block that doesn't parse (such as an unclosed quote or brace, or `=` with no value)". Some values satisfy both readings:

  ```
  @quill-labspace {lab=a=b}
  @quill-labspace {lab=Using other images}
  @quill-labspace {lab=say"hi"}
  ```

  The first has a `=` inside a token; the second has whitespace inside what should have been a quoted string; the third has a `"` inside a token. Each is also "an attribute block that doesn't parse" under the ABNF.
- **Options:**
  1. A value that is a run of non-whitespace characters containing a reserved character (`=`, `"`, `{`), or words separated by whitespace where §3.3 says a value must be quoted, is `attribute-unquoted-reserved`; `attribute-syntax` is for structure that can't be read as key, `=`, value at all: an unclosed quote or brace, a missing value, junk between pairs.
  2. Everything that fails the grammar is `attribute-syntax`; `attribute-unquoted-reserved` is only for reserved characters that the block would otherwise have accepted (there are none, so the row is unreachable).
- **Proposed resolution:** option 1. It gives the author the more specific message ("quote this value") whenever that is the fix, and keeps both rows reachable. Cases expect exactly one of the two per line. Implemented now: nothing yet; phase 05 chooses when it parses attribute blocks.
- **Affects:** conformance cases: `attributes/unquoted-equals`, `attributes/unquoted-quote-char`, `attributes/unquoted-whitespace`; phases 05, 06.
- **Resolution:** approved by the repository owner: option 1: a value that breaks only the quoting rule is `attribute-unquoted-reserved`, and structure that can't be read is `attribute-syntax`. SPEC §3.3 now says so.

### Q15: Text after an identifier primary

- **Section:** SPEC §3.4, §8.2
- **Raised by:** phase 03
- **Status:** resolved (2026-09-28)
- **Ambiguity:** §3.4: an identifier primary "is a single token that ends at the first whitespace". It doesn't say what happens to what follows:

  ```
  @include: guides/setup.md and more words
  @id: my id
  ```

  The token is `guides/setup.md`, but `and more words` isn't a primary, an attribute, or a comment.
- **Options:**
  1. It is an error: `directive-primary`, at the line, since the directive was given more primary than it takes.
  2. It is ignored, so a typo silently changes nothing.
  3. It is part of the primary (the token rule is dropped), which turns `@include: my file.md` into a path with a space.
- **Proposed resolution:** option 1, which reports rather than drops. `directive-primary`'s message would need a variant for it (for example, "`@include`'s primary is a single word; remove `{extra}`"), a change to the registry that this question's approval would cover. Cases expect `directive-primary` at the line.
- **See also:** Q30 (phase 05), which covers this shape and four others (`@note hello: text`, `@steps foo`, `@end: later`, `@id: two words`) and proposes a new `directive-extra-text` error instead. Resolve the two together.
- **Affects:** conformance cases: `primary/identifier-with-trailing-text`; phases 05, 06.
- **Resolution:** approved by the repository owner: settled with Q30: text after an identifier primary is an error, reported as the new `directive-extra-text` rather than `directive-primary`. SPEC §3.1 and §3.4 now say so, and the case `primary/identifier-with-trailing-text` expects the new slug.

### Q16: What a container-form error does to the container

- **Section:** SPEC §3.5, §8.2
- **Raised by:** phase 03
- **Status:** resolved (2026-09-28)
- **Ambiguity:** §3.5 makes two container-form mistakes errors: a trailing colon on a directive with no container form (`@steps:`), and a container-only directive (`@variant`) without its colon. It doesn't say whether, after the error, the line still opens a container (or a group arm) that the author's `@end` will close:

  ```
  @steps:
  1. One.
  @end
  ```

  If the colon line is still an opener, the `@end` closes it and one error is reported. If it isn't, the `@end` is an end line with no open container, a second error. For `@variant {deployment=cloud}` with no colon, if the line isn't an opener, its arm doesn't exist and the `@end` is again unmatched.
- **Options:**
  1. The form is decided by the line's colon, as §3.5 says ("a directive's form is decided by its own line"): a colon line opens a container even where that's an error, and a missing colon on a container-only directive is treated as if it were there. One error is reported; the `@end` matches.
  2. The line isn't a container after an error; `@end` is then reported too.
- **Proposed resolution:** option 1. It reports each mistake once, at the line where it happens, and keeps the rest of the file's structure intact, which is the point of §3.5's last paragraph. Cases expect the one error and include the `@end`.
- **Affects:** conformance cases: `forms/container-colon-on-line-only-directive`, `forms/container-only-without-colon`, `widgets/container-widget-line-form`, `widgets/line-widget-container-form`; phases 05, 06.
- **Phase 06:** implemented as proposed (`// SPEC-QUESTION(Q16)` in `structure/nest.rs`). A container-only directive that has a colon followed by text (`@variant: text`) is an opener and only `directive-primary` is reported, not `container-colon-missing`.
- **Resolution:** approved by the repository owner: option 1: the line keeps the form its colon gives it, one error is reported, and the `@end` still closes it. SPEC §3.5 now says so.

### Q17: Where diagnostics about a whole group are reported, and how groups count toward nesting depth

- **Section:** SPEC §3.6, §3.10, §4.3, §8.2
- **Raised by:** phase 03
- **Status:** resolved (2026-09-28)
- **Ambiguity:** A group is several openers with one end line, and §8.2 has rows about the group as a whole: a group never closed ("Container not closed before its enclosing block ends"), one that "mixes labeled and dimensional arms", one whose "dimensional arms share no dimension key", and (page level) one where "no arm survives a build's selection". The spec says where only for the case of an open container at an arm's opener. Nesting depth (§3.10) has a related gap: a group is a container, but its arms could count as a second level.
- **Options:**
  1. Group-level diagnostics are reported at the group's first opener (its directive line, not its title line); arm-level ones (`variant-arm-kind`, `variant-unknown`) at the arm's opener. A group is one container level, its arms are not another.
  2. Group-level diagnostics at the offending arm (the first arm that differs from the first), and arms count as a level.
  3. Group-level diagnostics at the last opener or at `@end`.
- **Proposed resolution:** option 1: the group is one container, opened once; the first opener is where it's written and where a fix starts. Cases carry `provisional` for the group-level diagnostics and for the nesting case that puts a group at the first level.
- **Affects:** conformance cases: `builds/selection/no-arm-survives`, `directives/variant/arms-share-no-dimension`, `directives/variant/mixed-labeled-and-dimensional`, `groups/same-directive-does-not-nest`, `groups/unclosed-group`, `nesting/group-counts-as-one-level`; phases 10, 11, 12, 14.
- **Phase 06:** implemented as proposed (`// SPEC-QUESTION(Q17)` in `structure/nest.rs`). An unclosed group is one `container-unclosed`, at its first opener. `variant-arm-kind` is reported at the arm's opener, `variant-mixed-arms` and `variant-no-shared-dimension` at the first opener; the shared-dimension check skips a group that's already mixed, and uses the keys common to every dimensional arm. The `@variant` rules apply to `@variant` only, not to other groupable widgets. Q33 covers how enclosing list items count.
- **Resolution:** approved by the repository owner: option 1: group-level diagnostics at the group's first opener, arm-level ones at the arm's opener, and a group counts as one nesting level. SPEC §3.6 and §3.10 now say so.

### Q18: A heading-bound directive with no heading above it

- **Section:** SPEC §3.8, §4.1, §4.4, §8.2
- **Raised by:** phase 03
- **Status:** resolved (2026-09-28)
- **Ambiguity:** §3.8: "at the top of a section, a directive describes the section; anywhere else, it describes the block it touches", and heading-bound directives "go at the top of their section, under the heading". Before a document's first heading there is no section:

  ```
  @id: orphan

  Text before any heading.
  ```

  `@id` is heading-bound only, so it has nothing to bind. `@available` is heading-or-block: before any heading, is it at the top of a section (the page), or "anywhere else"?
- **Options:**
  1. `@id` before any heading is `binding-not-section-top`, at the directive (it isn't under a heading). `@available` before any heading binds the block it touches: page-wide availability belongs in frontmatter (§4.4), so a section-level reading would duplicate it.
  2. `@available` before any heading describes the whole page.
  3. Both are errors when there's no heading.
- **Proposed resolution:** option 1. Cases carry `provisional` for both.
- **Affects:** conformance cases: `binding/available-before-first-heading`, `binding/id-with-no-heading`; phases 05, 06.
- **Phase 06:** implemented as proposed (`// SPEC-QUESTION(Q18)` in `structure/bind.rs`). Q32 asks the same question for the start of every container.
- **Resolution:** approved by the repository owner: option 1: `@id` before any heading is an error, and `@available` there binds the block it touches. SPEC §3.8 now says so.

### Q19: An end line in a different container from its opener, or indented differently within one

- **Section:** SPEC §3.9, §8.2
- **Raised by:** phase 03
- **Status:** resolved (2026-09-28)
- **Ambiguity:** §3.9 rule 3: "A container or group opens and closes within one list item... An end line indented differently from its opener doesn't close that opener; it's an error." §8.2 has the row "End line indented differently from its opener" and the row "End line with no open container". Which applies, and what else is reported, isn't stated for:

  ```
  - Item

    @note:
    Inside.
  - Two

    @end
  ```

  The `@end` is in another list item (a different CommonMark container), so it can't close the note. Is it an end line with no open container, or one indented differently from its opener, and is the note also reported unclosed? A second question is the same container with different spaces: `@note:` at column 1 and `   @end` (three extra spaces, which §1.5 allows before any block).
- **Options:**
  1. An end line that is in a different CommonMark container from an open opener that it would otherwise close is `end-indent-mismatch`, and the opener is also `container-unclosed` (it stays open until its own container ends). `end-unmatched` is for an end line when no container is open anywhere it could reach. An end line in the same container as its opener closes it whatever its extra spaces (up to three); the formatter (§8.3) removes them.
  2. Any difference in indentation is `end-indent-mismatch`, even within one container.
  3. Different container: `end-unmatched` only.
- **Proposed resolution:** option 1. It follows the container rule the section is about and doesn't punish spacing the grammar accepts. Cases expect both diagnostics for the cross-container case.
- **Affects:** conformance cases: `lists/end-in-next-item`, `lists/end-with-extra-indent-same-container`; phases 05, 06.
- **Phase 06:** implemented as proposed (`// SPEC-QUESTION(Q19)` in `structure/nest.rs`). An end line with no open container in its own list of blocks is `end-indent-mismatch` when a container is open in an enclosing scope, or when an earlier container was left unclosed when its scope ended (each such container is claimed by one end line, innermost first); otherwise `end-unmatched`. The container is reported unclosed where its own scope ends.
- **Resolution:** approved by the repository owner: option 1: an end line in a different container is `end-indent-mismatch`, and the opener is also `container-unclosed`; up to three extra spaces within one container don't matter. SPEC §3.9 rule 3 now says so.

### Q20: Where page-level, model, and frontmatter diagnostics are reported when several places cause them

- **Section:** SPEC §4.1, §4.2, §5.5, §7.2, §8.1, §8.2
- **Raised by:** phase 03
- **Status:** resolved (2026-09-28)
- **Ambiguity:** §8.1 says a page-level diagnostic "is reported at the source location that causes it", and that when the cause is in a fragment it "is reported at the include site". A conformance case needs one line. The spec doesn't choose it when two places are involved:

  - a duplicate id (`id-duplicate`) or a repeated heading (`heading-duplicate-without-id`): the first or the later occurrence, and for a slug that collides with an `@id`, the heading or the `@id` line;
  - an id from an include that collides with the page's own: the include site (stated), but whether the second of two includes of one fragment or both;
  - an include cycle: which include closes it;
  - a required frontmatter field missing, or a page matching several content types or none: no line names the problem;
  - a model name used in two roles: the first or the later declaration.

  §8.1's "SHOULD also report it in the fragment" is optional, so an expected list can't require it.
- **Options:**
  1. Report at the later occurrence: the second of two duplicate ids or headings (an `@id` line for an explicit id, the heading line for a slug); the second include of a fragment included twice; the include that closes the cycle, written in the file containing it; the first line of the file for frontmatter problems that have no line; the later declaration in `tessera.toml`. Report only at the include site, never also in the fragment (a processor may add the fragment report as a note, not as a conformance diagnostic).
  2. Report at every occurrence.
  3. Report at the first occurrence.
- **Proposed resolution:** option 1: the later occurrence is the one that made the earlier one a duplicate, and the first of a set is unchanged by whatever the author added. Cases that depend on a choice carry `provisional`.
- **Affects:** conformance cases: `directives/id/duplicate-explicit`, `directives/id/duplicate-through-include`, `directives/id/same-fragment-twice`, `directives/id/slug-equals-explicit-id`, `directives/include/cycle`, `directives/include/self-include`, `frontmatter/missing-required-field`, `frontmatter/no-frontmatter-at-all`, `frontmatter/no-type-and-no-default`, `frontmatter/reference-type-requires-api-version`, `frontmatter/two-types-match`, `headings/duplicate-across-included-fragment`, `headings/duplicate-heading-on-page`, `headings/explicit-id-is-stable-source-and-page-id`, `headings/page-id-differs-from-source-id`, `model/name-is-a-feature-key`, `model/name-is-a-lifecycle-state`; phases 10, 11, 12, 14.
- **Resolution:** approved by the repository owner: option 1: report once, at the later occurrence; at the include site only, with the fragment as related information; frontmatter problems with no line at the file's first line. SPEC §8.1 now says so.

### Q21: A link to a page that a build drops

- **Section:** SPEC §5.2, §8.2, §9.3
- **Raised by:** phase 03
- **Status:** resolved (2026-09-28)
- **Ambiguity:** §9.3 drops a page whose `variant` frontmatter conflicts with a selection. §8.2 has a row for a link whose target *id* a build removes ("Target id is removed by a build"), but none for a link to a page (with or without an id) that the build doesn't publish. Such a link is valid in the `site` build and would have no target in `cloud-only`:

  ```
  [Self-managed setup](sm.md)   ← sm.md has variant: {deployment: self-managed}
  ```
- **Options:**
  1. Nothing is reported: the link is valid in the source, and the consumer profile could route it to a page that another deployment publishes.
  2. An error in that build (a new row, or `link-id-removed` extended to pages), since the compiled link would be dead.
  3. A warning.
- **Proposed resolution:** option 2 is what §8.1's page-level checking is for, but it needs a row, so this is a request for one. No case depends on it yet, because the registry has nothing to expect; add cases when a slug exists.
- **Affects:** conformance cases: none; phases 10, 11, 12, 14.
- **Resolution:** approved by the repository owner: option 2: an error in that build, with a new §8.2 row and registry entry, `link-page-dropped` (TSR121, page level). SPEC §5.2 now says so, and the case `links/page-dropped-by-selection` expects it.

### Q22: What a destination that "looks like a published route" is

- **Section:** SPEC §5.2, §8.2
- **Raised by:** phase 03
- **Status:** resolved (2026-09-28)
- **Ambiguity:** §5.2: "A destination that looks like a published route rather than a file path produces a warning offering conversion." Paths are file paths; a `/` prefix means content-root-relative. The spec doesn't say how a route is recognized:

  ```
  [Install](/guides/install/)
  [Install](/guides/install)
  [Install](../guides/install)
  ```
- **Options:**
  1. A local destination (not external, not `#id` alone) whose path has no file extension in its last segment, or ends in `/`, and that doesn't name an existing file or directory-index page, is route-like: `link-route` is reported, and `link-target-missing` is not, so the author gets one message with the conversion offered.
  2. Only destinations that end in `/`.
  3. Any destination that doesn't exist and has no extension.
- **Proposed resolution:** option 1. The case expects `link-route` alone for `/guides/install/`.
- **Affects:** conformance cases: `links/route-destination`; phases 10, 11, 12, 14.
- **Resolution:** approved by the repository owner: option 1: a local destination that names no existing file, and whose last segment has no extension or ends in `/`, gets `link-route` instead of a missing-file error. SPEC §5.2 now says so.

### Q23: The source of a reference-style image in an outline

- **Section:** SPEC §5.3
- **Raised by:** phase 03
- **Status:** resolved (2026-09-28)
- **Ambiguity:** The outline's `image` block has a required main value: "source, as written". For an inline image, `![alt](settings.png)`, that is `settings.png`. For a reference image, `![alt][ref]`, `![alt][]`, and `![alt]`, the text as written is the label (`ref`), and the source is in a definition (`[ref]: settings.png`), which outlines omit.
- **Options:**
  1. The source is the definition's destination, `settings.png`: an image's source is where its file is, and adapters must resolve labels anyway to check that the file exists.
  2. The label as written.
- **Proposed resolution:** option 1. Cases for the three reference forms carry `provisional`.
- **Implemented now (phase 07):** option 1 (`// SPEC-QUESTION(Q23)` in `tests/conformance/tests/adapters/inline.rs`). Nothing in the proposal turned out to be wrong. It costs nothing in the tree: `Image::destination` is already the definition's destination for every reference form (comrak resolves it), and the label is `Image::label` (full form) or the alt text (collapsed and shortcut), so option 2 would be as cheap if a human chooses it. The one thing to note is that a reference image whose label has no definition isn't an image at all (CommonMark), so it never has a source of either kind.
- **Affects:** conformance cases: `images/collapsed-reference`, `images/full-reference`, `images/shortcut-reference`; phases 07.
- **Resolution:** approved by the repository owner: option 1: a reference image's source is its definition's destination. SPEC §5.3 now says so.

### Q24: A page whose page-level availability isn't available in a filter build

- **Section:** SPEC §4.4, §9.3
- **Raised by:** phase 03
- **Status:** resolved (2026-09-28)
- **Ambiguity:** §9.3: `filter` "removes content that isn't available for" the target, and a page's frontmatter `available` "applies to the whole page". §9.3 says which pages a variant selection drops, but not what a filter does with a page whose spec makes all of it unavailable: it could be dropped, like a conflicting page, or published with no content.
- **Options:**
  1. The page isn't published in that build: nothing on it is available, and an empty page in navigation is worse than none.
  2. The page is published with only its title and frontmatter.
- **Proposed resolution:** option 1. The case lists the pages each build publishes.
- **Affects:** conformance cases: `builds/filter/page-level-availability`; phases 10, 11, 12, 14.
- **Resolution:** approved by the repository owner: option 1: the page isn't published in that build. SPEC §9.3 now says so.

### Q25: What a retained `@available` shows when its primary was a feature key

- **Section:** SPEC §4.4, §9.2
- **Raised by:** phase 03
- **Status:** resolved (2026-09-28)
- **Ambiguity:** §4.4: a feature key "is replaced by that feature's declared spec". §9.2 step 2 resolves feature keys before build modes, and §9.3 says content that remains "is annotated as in `badge`". A resolved outline keeps the `available` directive as the annotation. Its primary is either the key as the author wrote it or the spec it stands for.
- **Options:**
  1. The declared spec (`cloud, self-managed preview 3.4`), as §4.4 says: the key is replaced, and an annotation the emitters render needs the targets and states, not the key.
  2. The key, with the registry consulted again by each emitter.
- **Proposed resolution:** option 1. The case is provisional.
- **Affects:** conformance cases: `builds/filter/feature-key`; phases 10, 11, 12, 14.
- **Resolution:** approved by the repository owner: option 1: the declared spec, not the key. SPEC §4.4 (Feature keys) now says so.

### Q26: Heading levels in an included section

- **Section:** SPEC §4.2, §9.2
- **Raised by:** phase 03
- **Status:** resolved (2026-09-28)
- **Ambiguity:** §4.2 includes "a heading's section", the heading and its content up to the next heading of the same or a higher level. It says nothing about the level of the included headings in the including page. A fragment's `## Install` included under a page's `### Steps` could keep level 2, which breaks the page's outline, or be shifted to level 4.
- **Options:**
  1. Levels are kept as written: an include is transclusion, and the author chooses where to include it. A tool may warn about a skipped level.
  2. Levels are shifted so the included top heading sits one below the enclosing heading.
- **Proposed resolution:** option 1, since the spec has no level-shifting syntax or attribute and options 2's rule would have to guess what "enclosing" means inside lists and containers. The cases put includes where the levels agree, and are provisional.
- **Affects:** conformance cases: `directives/include/heading-true-keeps-heading`, `directives/include/section-by-explicit-id`, `directives/include/section-by-source-id`; phases 10, 11, 12, 14.
- **Resolution:** approved by the repository owner: option 1: included headings keep the levels they're written with. SPEC §4.2 now says so.

### Q27: `role` is reserved, but the full example model declares it

- **Section:** SPEC §7.2 (reserved attribute keys); content-model.md §15, §20.3
- **Raised by:** phase 08
- **Status:** resolved (2026-09-28)
- **Ambiguity:** `tessera_core::reserved::HTML_GLOBAL_ATTRIBUTES` includes `role` (an ARIA global attribute), so `model-attribute-reserved` rejects it. Phase 01's `examples/content-models/full.toml` declared a widget attribute `role = "set(enum(admin, developer, writer))"` and, as an acceptance criterion, must load with no issues. The two contradict.
- **Options:**
  1. Keep `role` reserved and rename the example's attribute. The site output writes widget attributes onto the widget's custom element, where `role` would change its accessibility role.
  2. Remove `role` from the reserved list. Authors could then declare a `role` attribute whose value (such as `admin`) becomes an invalid ARIA role on the element.
- **Proposed resolution:** option 1. Implemented now: the example's attribute is renamed `audience` (in `full.toml` and its comment); no other file mentions the old spelling.
- **Affects:** `examples/content-models/full.toml`; phase 03 fixtures that copy from it.
- **Resolution:** approved by the repository owner: option 1: `role` stays reserved, and the example's attribute is `audience`.

### Q28: `DefaultValue` has no number variant

- **Section:** SPEC §3.3; content-model.md §6.2; `tessera_core::DefaultValue` (phase 02 contract)
- **Raised by:** phase 08
- **Status:** resolved (2026-09-28)
- **Ambiguity:** an attribute of type `number` can have a default (`height = { type = "number", default = 600 }`), but `DefaultValue` is `Text`, `Boolean`, or `Set`. There is nowhere to put a number.
- **Options:**
  1. Add `DefaultValue::Number` (a contract change: it touches every `match` on `DefaultValue` in phases 05, 06, 10, and 23).
  2. Store the default's source text (`"600"`) in `DefaultValue::Text`. Consumers of a `number` attribute already read attribute values as text.
- **Proposed resolution:** option 2 for now, since it needs no contract change and loses nothing; option 1 if a consumer needs to tell `"600"` from `600`. Implemented now: option 2 (`// SPEC-QUESTION(Q28)` in `tessera-model/src/fields.rs`).
- **Affects:** `crates/tessera-core/src/schema.rs`; phases 05, 06, 10, 23.
- **Resolution:** approved by the repository owner: option 2: a number default is stored as its source text in `DefaultValue::Text`. Revisit if a consumer needs to tell `"600"` from `600`.

### Q29: A dimension name as a target of a versioned entry

- **Section:** SPEC §4.4
- **Raised by:** phase 08
- **Status:** resolved (2026-09-28)
- **Ambiguity:** a target may be a dimension name, "which stands for all of its values", and "a target that the content model declares as versionless takes a single state and no versions". For a dimension with both kinds of value (`deployment`: `cloud` versionless, `self-managed` versioned), `deployment 3.4` is neither clearly valid nor clearly invalid.
- **Options:**
  1. A dimension name is versionless only if all its values are. `deployment 3.4` is then accepted for a mixed dimension, and the version means nothing to `cloud`.
  2. A dimension name is versionless if any of its values is, so versions are never allowed on a mixed dimension name.
  3. Versions are never allowed on a dimension name.
- **Proposed resolution:** option 1: it rejects only what is certainly wrong (a version on a dimension whose values are all versionless) and never rejects a spec that has a sensible meaning. **Implemented now: option 3**, the most conservative, at the reviewer's request while the question is open (`// SPEC-QUESTION(Q29)` in `tessera-model/src/model.rs`). A version on a dimension name is reported as `model-availability-versionless` (`available-versionless` in documents) with the base message; its wording ("`deployment` is versionless") doesn't quite fit, and a message variant naming the dimension would need a registry change, so none was added.
- **Affects:** `tessera-model` (`check_availability`); phases 10, 12 (which decide what the version means for versionless members).
- **Resolution:** approved by the repository owner: option 3: a version on a dimension name is always an error, since the dimension's values don't share one version line. SPEC §4.4 now says so. It's reported with a new `dimension` message variant of `model-availability-versionless` and `available-versionless`, which suggests naming a value instead (`AvailabilityProblem::DimensionVersion`).

### Q30: Text on a directive line that fits no part of the directive

- **Section:** SPEC §3.1, §3.4, §8.2, Appendix A
- **Raised by:** phase 05
- **Status:** resolved (2026-09-28)
- **Ambiguity:** the grammar (`directive-line`) allows only a name, an attribute block, a colon, and a primary, but §3.2 recognizes a line as a directive from its keyword alone, so a known keyword can be followed by text that fits no part of the grammar, and §8.2 has no row for it. Five shapes come up:

  ```
  @note hello: text          (attributes written without braces; no colon in the right place)
  @steps foo                 (a name, then text, on a directive with no primary)
  @end: later                (anything after `@end`)
  @include: my file.md       (an identifier primary ends at whitespace; ` file.md` is left over)
  @id: two words
  ```

  The first three aren't valid directive lines, and the fourth and fifth leave text after the identifier that the spec doesn't assign to anything.
- **Options:**
  1. Add one error to §8.2, for example `directive-extra-text` ("`@include` takes one identifier; ` file.md` isn't part of it"), covering all five shapes.
  2. Report each under the nearest existing row: `attribute-syntax` for text where an attribute block or colon should be, `directive-primary` for text after `@end`'s colon, and nothing for text after an identifier, which the identifier's own checks (`id-invalid`, `include-target-missing`) then catch in most cases.
  3. Treat an identifier primary's leftover text as part of it (so `@include: my file.md` names a file with a space in it), and report the other shapes as in option 2. This contradicts §3.4's "ends at the first whitespace".
- **Proposed resolution:** option 1. It reports every case, with a message that says what to fix, and needs one new registry entry (a contract change: phase 02's `diagnostics.toml`, with a `Fix` that removes the text). Implemented now: option 2 for the head junk and `@end`; leftover text after an identifier is kept in the tree (`IdentifierPrimary::trailing`) and reported as `directive-primary`, as Q15 proposes, with that entry's existing message.
- **See also:** Q15 (phase 03), which covers text after an identifier primary and proposes reporting it as `directive-primary`. Resolve the two together.
- **Affects:** `crates/tessera-syntax/src/convert.rs` (`SPEC-QUESTION(Q30)`), `crates/tessera-syntax/src/tree.rs` (`DirectiveLine::unexpected`, `IdentifierPrimary::trailing`, `EndLine::extra`); `tests/conformance/diagnostics.toml`; phases 03, 05, and 10. No conformance case should depend on these shapes until this is resolved; tag any that do `provisional`.
- **Resolution:** approved by the repository owner: option 1, together with Q15: a new §8.2 row and registry entry, `directive-extra-text` (TSR120), covers all five shapes, with message variants for text in the head (`head`) and after `@end` (`end`). SPEC §3.1 now says so, the parser reports it, and the cases `primary/extra-text-after-name`, `primary/extra-text-where-attributes-go`, and `primary/extra-text-after-end` expect it.

### Q41: `{key}` directly after an image

- **Section:** SPEC §5.1, §5.3
- **Raised by:** phase 07
- **Status:** resolved (2026-09-28)
- **Ambiguity:** §5.1 says "`{…}` containing `=` directly after … an image is an attribute block, not a phrase", which implies that a `{…}` *without* `=` directly after an image is a phrase. But §5.3 says an attribute block goes "directly after the image", and the conformance case `images/attribute-bare-key` (`![A](p.png){width}`) expects `attribute-bare-key`, which treats `{width}` as an attribute block with a bare key. Both can't hold:

  ```
  ![Logo](logo.png){cloud}
  ```
- **Options:**
  1. Any `{…}` that closes on the same line and comes directly after an image is its attribute block, so `{cloud}` there is a bare key (an error). To write a phrase after an image, put a space before it or escape it (`\{cloud}`).
  2. Only a block containing `=` (or an empty one) is an attribute block; `{cloud}` after an image is a phrase, and `![A](p.png){width}` is a phrase candidate, not a bare-key error.
- **Proposed resolution:** option 1. It matches the case, needs no `=` test, and turns the likeliest mistake (a forgotten `=value`) into an error instead of a literal `{width}` in the output. A phrase directly after an image is rare, and both escapes are available. Implemented now: option 1 (`// SPEC-QUESTION(Q41)` in `crates/tessera-syntax/src/inline/image.rs`). Under option 2 the change is one test in `comrak_tessera::tessera::image_attributes_len`, and the case `images/attribute-bare-key` would change. A `{` directly after an image that has no `}` on its line is also reported (`attribute-syntax`) and left as text, under either option.
- **Affects:** `crates/comrak-tessera/src/tessera.rs` (`image_attributes_len`), `crates/tessera-syntax/src/inline/image.rs`; conformance case `images/attribute-bare-key` (tagged `provisional`); phases 10 and 12.
- **Resolution:** approved by the repository owner: option 1: any `{…}` directly after an image and closed on its line is the image's attribute block; `{cloud}` there is a bare key. SPEC §5.1 and §5.3 now say so.

### Q42: Escapes in a fenced block that opts in to phrases

- **Section:** SPEC §2.3, §5.1
- **Raised by:** phase 07
- **Status:** resolved (2026-09-28)
- **Ambiguity:** a fence with `phrases=true` in its info string substitutes phrases in its code. In code, CommonMark's backslash escapes don't apply, so there's no way to write a literal `{key}` (say, a Mustache or Helm template that happens to use a declared key) in such a block. §2.3 says `\{` prevents a phrase, and doesn't say where.
- **Options:**
  1. Backslash doesn't escape in code, so `\{key}` in an opted-in fence is a candidate (with a backslash before it). A literal `{key}` needs a fence without `phrases=true`, or a key that isn't declared.
  2. `\{` escapes in an opted-in fence, and the substituted output drops the backslash (`\{key}` becomes `{key}`). The code's content changes from what the author sees in the source.
- **Proposed resolution:** option 2 is friendlier to authors, but it makes a code block's content differ from its source, which is the thing fences avoid. Option 1 is the conservative choice, since it changes nothing about code. Implemented now: option 1 (`// SPEC-QUESTION(Q42)` in `crates/tessera-syntax/src/inline/phrase.rs`); `CodeBlock::phrases` lists every `{key}`, with no escapes.
- **Affects:** `crates/tessera-syntax/src/inline/phrase.rs`; phases 10 and 12.
- **Resolution:** approved by the repository owner: option 1: a backslash doesn't escape in code, so a phrases fence's content never differs from its source except where phrases are substituted. SPEC §5.1 now says so.

### Q43: Phrases in link reference definitions and autolinks

- **Section:** SPEC §5.1, §5.2
- **Raised by:** phase 07
- **Status:** resolved (2026-09-28)
- **Ambiguity:** §5.1 lists "link destinations" among the places phrases apply. An inline link's destination is part of the link (`[text]({api}streaming)`). But a reference link's destination is in a definition (`[ref]: {api}streaming`), which isn't a node in the parser's tree, and an autolink's text is its destination (`<https://{host}/x>`, which CommonMark accepts, since `{` is allowed).
- **Options:**
  1. Recognize candidates in definitions and autolinks too.
  2. Only in inline links and images, where the destination is inside the node.
- **Proposed resolution:** option 1 for definitions (the destination is a link destination, and `[ref]: {api}streaming` is the natural way to write it once); autolinks are a corner, and either reading is fine as long as it's stated. Implemented now: option 2, the conservative one (the text stays literal, nothing is dropped), with `// SPEC-QUESTION(Q43)` in `crates/tessera-syntax/src/inline/mod.rs`. Option 1 for definitions needs the parser to expose definitions (phase 23 needs them too; phase 05 left this open).
- **Affects:** `crates/tessera-syntax/src/inline/`; phases 10, 12, and 23.
- **Resolution:** approved by the repository owner: option 1 for both: phrases apply in the destinations of link reference definitions and of autolinks. SPEC §5.1 now says so. Autolinks are implemented (`Link::destination_phrases`, with no backslash escapes, as CommonMark has none in autolinks). Definitions are implemented too (phase 23): `ParsedDocument::definitions` lists every link reference definition with exact spans for its label, destination, and title, and `LinkDefinition::destination_phrases` holds the candidates in the destination as written, with backslash escapes applying as in an inline destination (`\{key}` is text, and is in `ParsedDocument::escaped_phrases`). They are a side list, not blocks: comrak consumes them, and a small change to the fork (marked `// TESSERA:`, listed in `FORK.md`) reports them.
### Q31: Directives that stack in front of a block, and a block that isn't there

- **Section:** SPEC §3.8, §4.4, §4.5
- **Raised by:** phase 06
- **Status:** resolved (2026-09-28)
- **Ambiguity:** §3.8 says a following-block directive binds "the next block (paragraph, list, code block, blockquote, table, or container)". It doesn't say what "next" means when another directive line comes first:

  ```
  @available: cloud
  @note
  Text.
  ```

  `@available: cloud` directly above a one-line note (`@note: Preview.`), which to a reader and to the site output is the same note as the other two forms (`@note` with a block below it, and `@note:` with `@end`), and `@available: cloud` directly above `@include: x.md`, which stands alone and isn't in the list of blocks. Nor does it say whether a thematic break or a raw HTML block, which the list omits, can be bound.
- **Options:**
  1. Following-block directives stack, as heading-bound ones do: they all bind the block that the last of them touches. A following-block directive above a directive that isn't itself following-block (`@include`, `@id`, an end line) has no block: `binding-no-block`. A line-form directive that is its own text (`@note: text`) renders as a block, so a following-block directive binds it, and so the three spellings of a cloud-only note behave alike. Every CommonMark block except a heading can be bound.
  2. A following-block directive binds the next directive line too (including one that stands alone), so `@available` above `@note` annotates the note, and above `@include` annotates the include.
  3. Only the blocks §3.8 lists can be bound, so a thematic break or an HTML block is `binding-no-block`.
- **Proposed resolution:** option 1. It matches how stacked heading-bound directives read, and reports rather than guesses when what follows is a directive that stands alone. Implemented now: option 1, including the one-line note (`// SPEC-QUESTION(Q31)` in `structure/bind.rs`; `@available` above `@include`, `@id`, or a widget that stands alone still reports `binding-no-block`); `tessera_syntax::bound_block` finds the block a stack binds. If `@available` above `@include` should work, phase 12 needs option 2's reading of an include's content.
- **Affects:** `crates/tessera-syntax/src/structure/bind.rs`; phases 10, 11, 12, 23. No conformance case depends on it.
- **Resolution:** approved by the repository owner: option 1, extended: following-block directives stack and bind the block the last of them touches, and any block but a heading can be bound. A line-form directive whose content is its own text primary (a one-line `@note: …`) is a block, so `@available` above it binds the note; `@id`, `@include`, and end lines aren't blocks. SPEC §3.8 now says so.

### Q32: Which sections a heading-bound directive can be at the top of

- **Section:** SPEC §3.8, §3.9, §8.2
- **Raised by:** phase 06
- **Status:** resolved (2026-09-28)
- **Ambiguity:** §3.8 defines a section as a heading and the content up to the next heading of the same or a higher level. Headings are blocks in a container (the document, a list item, a blockquote, a directive container), so it's unclear whether a heading-bound directive in one container can bind a heading in another:

  ```
  ## Setup
  @note:
  @id: x
  @end
  ```

  Q18 covers the document before its first heading; this is the same question for the start of every container.
- **Options:**
  1. A section is found among the siblings in the same container, so a heading-bound directive binds a heading of its own container, and a container's first blocks have no heading above them (`binding-not-section-top`).
  2. A directive at the top of a container binds the heading above the container.
- **Proposed resolution:** option 1, which follows §3.9's rule that binding stays inside the container. Implemented now: option 1 (`// SPEC-QUESTION(Q18)` in `structure/bind.rs`; the pass has no other notion of a section).
- **Affects:** conformance cases: none beyond Q18's; phases 10, 11, 12.
- **Resolution:** approved by the repository owner: option 1: sections are found within one container, so a heading-bound directive never binds a heading outside its own document, list item, blockquote, container, or arm. SPEC §3.8 now says so.

### Q33: Whether containers in list items and block quotes count toward nesting depth

- **Section:** SPEC §3.10, §8.2
- **Raised by:** phase 06
- **Status:** resolved (2026-09-28)
- **Ambiguity:** §3.10 warns "when containers nest more than two levels deep". A container can sit in a list item that sits in a container, so the levels can be split by CommonMark containers:

  ```
  @note:
  - item

    @note:
    @details:
    ...
  ```
- **Options:**
  1. Depth counts every open container (directive containers and groups) around a directive, through list items and block quotes. The list item and the block quote themselves aren't levels.
  2. Depth restarts in each list item and block quote, since a container never straddles them (§3.9).
- **Proposed resolution:** option 1: what a reader sees as nesting is the whole stack, and §3.10's reason (hard to follow) doesn't depend on where a list item sits. Implemented now: option 1 (`// SPEC-QUESTION(Q33)` in `structure/nest.rs`).
- **Affects:** conformance cases: none; phases 10 and 23.
- **Resolution:** approved by the repository owner: option 1: depth counts every open container around a directive, through list items and blockquotes. SPEC §3.10 now says so.

### Q34: Exactly what triggers the three list warnings

- **Section:** SPEC §3.9, §4.6, §8.2 ("Lists")
- **Raised by:** phase 06
- **Status:** resolved (2026-09-28)
- **Ambiguity:** the three "Lists" rows say when they apply in a sentence each, which leaves several edges open:
  - `list-ended-by-directive`: "an unindented directive line ends a list". Does a directive after a blank line count (the blank line already ends the list in CommonMark), an `@end` line (it often ends a list that sits in a container, correctly), or a block quote (rule 6 says quotes work the same way)?
  - `directive-indented-code`: is it every directive-shaped line in an indented code block, or the first?
  - `steps-numbering-continued`: how much may sit between the two lists (only directive lines, or blocks too), and must there be at least one?
- **Options:** any combination of the above.
- **Proposed resolution:** the narrowest triggers that catch the cases the spec describes. `list-ended-by-directive`: a directive line (including a container or group opener) directly after a list with no blank line, in the same list of blocks; not an end line, and not after a block quote. `directive-indented-code`: every directive-shaped line (`@`, a known keyword, then a space, tab, `{`, `:`, or the end) of an indented code block, at the `@name`. `steps-numbering-continued`: an ordered list whose start number is the `@steps` list's start plus its item count, after one or more line-form directive lines and nothing else. Implemented now: these (`// SPEC-QUESTION(Q34)` in `structure/lists.rs`).
- **Affects:** `crates/tessera-syntax/src/structure/lists.rs`; phases 10 and 23.
- **Resolution:** approved by the repository owner: as proposed: the narrowest triggers. SPEC §3.9 now states them.

### Q35: A text primary followed by an ordered list that doesn't start at 1

- **Section:** SPEC §3.4, §8.2
- **Raised by:** phase 06
- **Status:** resolved (2026-09-28)
- **Ambiguity:** the conformance case `lists/steps-numbering-continued` (phase 03) wrote

  ```
  @steps
  1. One.
  @note: A note.
  2. Two.
  ```

  and expected the note's primary to be `A note.` and `2. Two.` to be a list starting at 2. §3.4 says a text primary continues "until a blank line, a directive line, or any other line that would interrupt a paragraph", and CommonMark's rule is that an ordered list can interrupt a paragraph only if it starts with 1. So `2. Two.` continues the primary, and the note's text is `A note. 2. Two.` (this is what the fork and the syntax tree do).
- **Options:**
  1. Keep §3.4 and CommonMark: the case is wrong. Its input gets a blank line after the note (so `2. Two.` starts a list), which is what an author who wants the warning has to write.
  2. A text primary also ends at any line that starts a list, of any number. That departs from CommonMark's paragraph rule, which §3.4 says a primary follows "exactly".
- **Proposed resolution:** option 1. Implemented now: option 1; the case's input has the blank line, its diagnostics move to line 9, and the case is `provisional` on this question (the expected outline and diagnostics are otherwise as phase 03 wrote them).
- **Affects:** conformance case `lists/steps-numbering-continued`; `crates/comrak-tessera` (option 2 would change the fork's paragraph rules); phases 05 and 06.
- **Resolution:** approved by the repository owner: option 1: a text primary follows CommonMark's paragraph rule, so `2. Two.` continues it; the case's corrected input stands. SPEC §3.4 now gives the example.

### Q36: What else is reported about a directive whose attribute block never closes

- **Section:** SPEC §3.3, §3.5, §3.8, §8.2
- **Raised by:** phase 06
- **Status:** resolved (2026-09-28)
- **Ambiguity:** an unclosed `{` takes the rest of the line (phase 05), so the line's colon and primary, and therefore whether it's line form, a container opener, or has its own text, are unknowable:

  ```
  @note {type=tip
  ```

  §8.2 reports the unclosed block, and the same line is also a following-block directive with nothing after it, which §8.2 also reports.
- **Options:**
  1. Report only the malformed block. The line's binding is `unbound` and nothing more is said about it, since every other conclusion depends on guessing what the author meant.
  2. Report everything the guessed reading would.
- **Proposed resolution:** option 1, so one mistake gives one diagnostic; the case `attributes/unclosed-brace` expects exactly `attribute-syntax`. Implemented now: option 1 (`Class::Unreadable` in `structure/bind.rs`). An unclosed block's line is still an opener when its directive is container-only, so its `@end` matches, but it gets no `container-colon-missing` either.
- **Affects:** conformance case `attributes/unclosed-brace`; phases 05, 06, and 10.
- **Resolution:** approved by the repository owner: option 1: only the unclosed block is reported for that line. SPEC §3.3 now says so.

### Q71: What "a construct with errors" is, for the formatter

- **Section:** SPEC §8.3, §8.2
- **Raised by:** phase 23
- **Status:** resolved (2026-09-28)
- **Ambiguity:** §8.3 says canonical form is what a formatter writes, but a directive line that is wrong (an unclosed block, a bare key, text that fits no part, a container-only directive without its colon, a stray end line) has no reading the formatter can be sure of. Phase 23 leaves such a construct alone. The spec doesn't say which reported diagnostics count. One of them, `binding-blank-line`, is a warning the formatter itself fixes; the other warnings (`container-nesting-deep`, `title-not-accepted`, `title-dot-space`, `list-ended-by-directive`, `directive-indented-code`, `steps-numbering-continued`, `directive-unknown`) say nothing about the bytes it edits.
- **Options:**
  1. Errors stop the formatter for the construct they are reported on, warnings don't. A construct is a directive line (and its title line), an end line, or an image's attribute block; "on" means the diagnostic's location is inside it.
  2. Any diagnostic stops it. Then `binding-blank-line` could never be fixed.
  3. Format a construct with an error as well as it can be read.
- **Proposed resolution:** option 1. Implemented now (`tessera-fmt/src/skip.rs`, with a test that the warning list agrees with the diagnostics registry). Diagnostics that need the content model (unknown attribute keys, value types) aren't run by the formatter, so a block with an undeclared key is respaced and unquoted but keeps its order (Q75). The other options can't be less conservative than leaving a wrong construct as the author wrote it.
- **Affects:** `crates/tessera-fmt/src/skip.rs`; conformance cases `format/errors-left-alone`, `format/errors-container-structure`, `format/errors-binding`; phase 24 (format on save).
- **Resolution:** approved by the repository owner: as proposed: an error stops the formatter for the construct it's on; warnings don't. SPEC §8.3 now says so.

### Q72: Trailing whitespace on directive lines and end lines

- **Section:** SPEC §8.3, §3.1
- **Raised by:** phase 23
- **Status:** resolved (2026-09-28)
- **Ambiguity:** §8.3 removes trailing whitespace "after a container's `:`" and lists nothing else. §3.1 says processors ignore whitespace at the end of a directive line. So `@note {type=tip}  `, `@steps  `, `@available: cloud  `, and `@end  ` are non-canonical or not?
- **Options:**
  1. Only what §8.3 lists: whitespace after a container's colon goes; other trailing whitespace stays. A text primary's trailing spaces can be a hard break, so removing them there would change the page.
  2. Remove trailing whitespace from every directive line and end line, except a text primary's.
- **Proposed resolution:** option 1, the conservative one, implemented now. Option 2 is a one-rule addition if the owner wants it.
- **Affects:** `crates/tessera-fmt/src/head.rs`; conformance case `format/trailing-space-elsewhere`.
- **Resolution:** approved by the repository owner: as proposed: trailing whitespace is removed only after a container's colon. SPEC §8.3 now says so.

### Q73: Indentation of directive lines: block quotes, the marker's line, and tabs

- **Section:** SPEC §8.3, §3.9, §1.5
- **Raised by:** phase 23
- **Status:** resolved (2026-09-28)
- **Ambiguity:** §8.3 says a directive line in a list item is indented exactly to the item's content column, and Q1 and Q19 remove up to three extra spaces before a directive or end line. It says nothing about:
  - a block quote (`>   @note`): where is the content column, and is `>@note` canonical?
  - a directive on the marker's own line (`-   @note: x`), where the spaces after the marker are the marker's spacing;
  - indentation made of tabs, whose width depends on the tab stop.
- **Options:**
  1. In a block quote the content starts after the marker and one space; extra spaces after that are removed, and no space after `>` is left as written. A directive on a marker's line, and a line whose indentation holds a tab, are left alone.
  2. Also normalize `>@note` to `> @note`, marker spacing (`-   @note` to `- @note`), and tabs.
- **Proposed resolution:** option 1, implemented now (`tessera-fmt/src/indent.rs`). Marker spacing is markdown formatting, not Tessera's, and a tab's width can't be known from the source alone.
- **Affects:** `crates/tessera-fmt/src/indent.rs`; conformance cases `format/indent-block-quote`, `format/indent-left-alone`.
- **Resolution:** approved by the repository owner: as proposed: block quotes keep `>` and one space; a list marker's line and tab indentation are left alone. SPEC §8.3 now says so.

### Q74: A blank line between a following-block directive and its block, in the awkward cases

- **Section:** SPEC §8.3, §3.8, §3.4
- **Raised by:** phase 23
- **Status:** resolved (2026-09-28)
- **Ambiguity:** canonical form removes the blank line between a following-block directive and its block. Three cases where doing so isn't safe or isn't what the author sees:
  - the directive has a text primary (a widget that binds a following block and takes text): the blank line ends its text, so removing it would make the block part of the primary;
  - a link reference definition sits in the gap (`@steps`, a definition, a blank line, the list). comrak consumes definitions, so the tree shows only a gap, and a definition must never be deleted or moved;
  - the gap is between two blocks of a list item. A blank line there can be what makes the list loose, and closing it can make the list tight, which changes how every item renders (paragraph wrapping and spacing). Reproduced by the reviewer with `1. First step.`, `   @note`, a blank line, `   Text of the note.`, `2. Second step.`.
- **Options:**
  1. Leave a gap alone when the directive has a text primary, when any line in it isn't blank (this includes definitions), or when it is between blocks of a list item; otherwise remove the blank lines. `binding-blank-line` still reports a gap that stays.
  2. Also leave gaps alone in the first two cases only, and close gaps in list items whatever that does to tightness (the author wrote the directive there).
  3. Close a gap in a list item only when the list's tightness, worked out again, doesn't change.
- **Proposed resolution:** option 1, the conservative one; a formatter that runs on save (phase 24) mustn't change how ordinary markdown renders. Implemented now (`tessera-fmt/src/blank.rs`, using `ParsedDocument::definitions` and the block's owner). Option 2 is the previous reading; option 3 is exact but has to reason about every gap in a list together.
- **Affects:** `crates/tessera-fmt/src/blank.rs`; conformance cases `format/blank-line-definition`, `format/blank-line-in-containers`, `format/blank-line-list-tightness`.
- **Resolution:** approved by the repository owner: option 1 as implemented after review: the blank line is kept when removing it would change a list's tightness, when the directive has a text primary, or when a non-blank line sits in the gap. SPEC §8.3 now says so.

### Q75: Attribute blocks the formatter can't put in order or safely rewrite

- **Section:** SPEC §8.3, §3.3
- **Raised by:** phase 23
- **Status:** resolved (2026-09-28)
- **Ambiguity:** §8.3 orders attributes as the schema declares them and quotes values "only when necessary". Not settled:
  - a block with a key the schema doesn't declare (`attribute-unknown-key` is reported by phase 10, not by the parser): where does it go?
  - a repeated key, or a bare key: there is no single reading.
  - whether `label="Setup"` (a quoted value that could be a token) is non-canonical, given that types come from the schema and never from the value's spelling (§3.3).
- **Options:**
  1. Unknown keys: if every key is declared, sort; otherwise keep the block's order (respacing and unquoting still happen). A repeated or bare key, or anything the attribute parser reports, leaves the block alone. A quoted string that can be a token loses its quotes; one that needs them keeps its exact spelling.
  2. Unknown keys sort after known ones.
  3. Leave any block with an unknown key entirely alone.
- **Proposed resolution:** option 1, implemented now (`tessera-fmt/src/attributes.rs`). Before a block is rewritten its canonical text is parsed again and must have the same keys, values, and forms, or the block is left alone.
- **Affects:** `crates/tessera-fmt/src/attributes.rs`; conformance case `format/order-unknown-key`.
- **Resolution:** approved by the repository owner: as proposed: bare or repeated keys leave the block alone; an undeclared key keeps its place; an unneeded quote is dropped. SPEC §8.3 now says so.

### Q76: Image attribute blocks: an empty block, and a block in a table cell

- **Section:** SPEC §8.3, §5.3, §5.1
- **Raised by:** phase 23
- **Status:** resolved (2026-09-28)
- **Ambiguity:** §8.3 says "no empty attribute block", written for directives (`@note`, not `@note {}`); §5.3 uses the same grammar for images. Two edges for images:
  - `![a](b){}{cloud}`: removing the `{}` makes `{cloud}` the image's attribute block (§5.1), which changes what the text means.
  - a block inside a table cell: a `|` in a cell is written `\|`, and the canonical value-set spelling (`a|b`) would end the cell.
- **Options:**
  1. An empty image block is removed unless the next character is `{`; a block in a table cell is left as written.
  2. Also format blocks in table cells, keeping `\|`.
- **Proposed resolution:** option 1, implemented now (`tessera-fmt/src/attributes.rs`, `lib.rs`).
- **Affects:** `crates/tessera-fmt`; conformance case `format/empty-block-before-brace`.
- **Resolution:** approved by the repository owner: as proposed: an empty image block is removed; a block in a table cell is left alone. SPEC §8.3 now says so.

### Q77: Title lines

- **Section:** SPEC §8.3, §3.7
- **Raised by:** phase 23
- **Status:** resolved (2026-09-28)
- **Ambiguity:** a title line (`.Try it`) is part of a directive's construct, but §8.3 lists no rule for it: its indentation (up to three spaces are allowed before it as before any paragraph line), the spaces inside its text (which is inline markdown), and trailing whitespace.
- **Options:**
  1. Title lines are left as written; only the directive line below them is formatted.
  2. Indent title lines like the directive below them.
- **Proposed resolution:** option 1, implemented now. The title's text is prose, and §3.7 only requires that it touch the directive.
- **Affects:** `crates/tessera-fmt`; conformance case `format/title-lines-untouched`.
- **Resolution:** approved by the repository owner: as proposed: title lines are left as written. SPEC §8.3 now says so.

### Q61: A heading with no `@id` whose slug is empty

- **Section:** SPEC §5.5, §8.2
- **Raised by:** phase 11
- **Status:** resolved (2026-09-28)
- **Ambiguity:** a heading made only of characters the slugger removes (punctuation, emoji) slugs to the empty string, and a second one to `-1`, a third to `-2` (phase 09's hand-off; `github-slugger` and so Astro do the same):

  ```
  ## ???
  ## 🎉
  ```

  These headings' source ids are `""` and `-1`. Nothing can link to `""` usefully (`page.md#` names no heading), and an author who moves a heading's emoji sees its id change from `-1` to `""` or the reverse. §5.5 asks processors to warn about a phrase in an id-less heading and about a repeated one, but says nothing about an empty slug, and §8.2 has no row for it.
- **Options:**
  1. A warning on every heading without `@id` whose slug is empty (including the numbered repeats), telling the author to give it an `@id`. A new §8.2 row and registry entry, file level.
  2. An error, since the heading can't be linked to or included.
  3. Nothing: report what the consumer does.
- **Proposed resolution:** option 1. The page still builds, and the consumer publishes some id for it, but the author can't rely on it. Also unverified: what Astro does with an empty id (phase 21 should check whether it writes `id=""`, or none).
- **Implemented now:** option 3, with the facts recorded, so a check can report it once the row exists: `Heading::empty_slug` and `Project::empty_slug_headings()`. An empty source id names no heading (`FileIndex::heading_by_id("")` is `None`), and an empty fragment after `#` in a link or include means no id at all. `// SPEC-QUESTION(Q61)` in `crates/tessera-resolve/src/index/headings.rs` and `project.rs`.
- **Affects:** `crates/tessera-resolve`; the registry and SPEC §8.2 (option 1 adds a row); phases 10 and 14 (to report it); conformance cases: none yet.
- **Resolution:** approved by the repository owner: option 1: a warning, a new §8.2 row and registry entry, `heading-empty-slug` (TSR124), reported by `tessera check` using the source index's heading text. SPEC §5.5 now says so. Case: `headings/empty-slug`. Whether Astro writes `id=""` or no id stays for phase 21 to check.

### Q62: Include paths: percent-encoding, and an empty id

- **Section:** SPEC §4.2, §5.2
- **Raised by:** phase 11
- **Status:** resolved (2026-09-28)
- **Ambiguity:** §4.2 calls an include's primary a path, "optionally followed by `#` and an id", and the primary is an identifier that ends at the first whitespace (§3.4). A link destination is percent-decoded (`my%20diagram.png` names `my diagram.png`, asset contract §2), but an include isn't a destination, and the spec doesn't say. So a fragment named `my snippet.md` can't be included at all unless `%20` decodes. Separately, `@include: file.md#` has an empty id.
- **Options:**
  1. An include path is literal: `%20` is three characters, and a file whose name has a space can't be included.
  2. An include path is decoded the same way a link destination is, so `my%20snippet.md` names `my snippet.md`, and a literal `%` in a name is written `%25`.
  Empty id: (a) it includes the whole file, (b) it's an error like a missing id.
- **Proposed resolution:** option 2, for consistency: every path an author writes in Tessera resolves the same way, and a name with a space stays includable. And (a).
- **Implemented now:** option 1 and (a) (`// SPEC-QUESTION(Q62)` in `crates/tessera-resolve/src/index/mod.rs`): a missing file is reported either way (`include-target-missing`), so the conservative reading loses nothing silently.
- **Affects:** `crates/tessera-resolve`; phases 10, 12, 15 (include completion).
- **Resolution:** approved by the repository owner: option 2: an include path is percent-decoded as a link destination is, and an empty `#` includes the whole file. SPEC §4.2 now says so.

### Q63: An include of something that isn't a source file of the project

- **Section:** SPEC §4.2, §2.1, §2.2
- **Raised by:** phase 11
- **Status:** resolved (2026-09-28)
- **Ambiguity:** an include names "a file", and §2.1 says a source file is a `.md` file under the content root. A path can also name a file that isn't one: a `.md` file outside the content root (`../README.md`), a file with another extension (`data.yaml`, `snippet.txt`), or a directory.
- **Options:**
  1. Only source files can be included. Anything else is `include-target-missing`, whether or not it exists on disk.
  2. Any file inside the project or content root (the asset contract's boundary) can be included, and non-Markdown content is inserted as text.
- **Proposed resolution:** option 1: the source index knows only its own files, expansion needs them parsed, and inserting arbitrary text would bypass every check. `include-target-missing`'s message says "doesn't exist", which is untrue of a `README.md` outside the content root, so a new variant (`not-a-source`, or `outside`, as image and link have) would be kind to the author.
- **Implemented now:** option 1, with the message as it is (`// SPEC-QUESTION(Q63)` in `crates/tessera-resolve/src/project.rs`).
- **Affects:** `crates/tessera-resolve`; the registry (a new message variant); phases 10 and 14.
- **Resolution:** approved by the repository owner: as proposed: only a source file can be included, reported as `include-target-missing` with a new `not-source` message variant for a file outside the content root or not Markdown. SPEC §4.2 now says so.

### Q64: A link with only `#id` inside a fragment

- **Section:** SPEC §4.2, §5.2
- **Raised by:** phase 11
- **Status:** resolved (2026-09-28)
- **Ambiguity:** a link's path is relative to the file it's written in, and an empty path names that file. In a page, `[Setup](#setup)` links to a heading of the page (its own source ids, Q6). In a fragment, the same link names the fragment, and §4.2 says a link to a fragment file is an error, because a fragment isn't published:

  ```
  _fragments/prerequisites.md:
  See [the agent section](#install-the-agent).
  ```

  A fragment is written to be included, and a cross-reference to a heading beside it is natural, but where it lands is only known once a page includes it.
- **Options:**
  1. It's `link-to-fragment` like any other link to a fragment. Authors link to the page that includes the fragment instead.
  2. In a fragment, `#id` names a heading **in that fragment**, by the fragment's own source id, consistent with Q6 (a file's linkable ids are its own source ids, never those of another file, and never resolved against the including page's). Each page that includes the fragment compiles the link to that heading's page id *on that page* (phase 12). A `#id` that isn't a heading of the fragment itself is `link-id-missing`, as in a page. On a page where the heading isn't included (a section include that leaves it out, or a build that removes it), the link has no target there: a page-level error like `link-id-removed`.
- **Proposed resolution:** option 2, which fits Q6 and is what authors want (cross-references inside a fragment), but it needs that page-level row and a rule for links to other fragments' ids, so it isn't a small change. Until a human chooses, option 1.
- **Implemented now:** option 1 (`// SPEC-QUESTION(Q64)` in `crates/tessera-resolve/src/project.rs`).
- **Affects:** `crates/tessera-resolve`; phases 10, 12, 14; conformance cases: none.
- **Resolution:** approved by the repository owner: option 2 as restated: a `#id` alone in a fragment names a heading of the fragment itself and compiles to that heading's page id on each including page, so it isn't `link-to-fragment`; an id the fragment doesn't have is `link-id-missing`. SPEC §5.2 now says so.

### Q65: `{heading=false}` on an include with no `#id`

- **Section:** SPEC §4.2
- **Raised by:** phase 11
- **Status:** resolved (2026-09-28)
- **Ambiguity:** `heading=false` "omits the included section's own heading". An include with no `#id` includes a whole file, which has no "section's own heading", although it usually starts with one:

  ```
  @include {heading=false}: _snippets/prerequisites.md
  ```
- **Options:**
  1. The attribute does nothing without an id.
  2. It drops the file's first block when that block is a heading.
  3. It's an error or a warning: the attribute means nothing here.
- **Proposed resolution:** option 3 as a warning, since the author expects something to disappear, with option 1 as the behavior so nothing is dropped by a guess. That needs a registry entry.
- **Implemented now:** option 1 (`// SPEC-QUESTION(Q65)` in `crates/tessera-resolve/src/expand.rs`).
- **Affects:** `crates/tessera-resolve`; the registry; phases 10 and 14.
- **Resolution:** approved by the repository owner: option 3 as a warning, with option 1's behavior: `{heading=false}` without `#id` has no effect and is warned about, as a new §8.2 row and registry entry, `include-heading-without-id` (TSR125). SPEC §4.2 now says so. Case: `directives/include/heading-false-without-id`. SPEC Appendix B had exactly this (`@include {heading=false}: _fragments/prerequisites.md`, a fragment with no heading), so the attribute was removed there and from its copies in `examples/`, `tests/conformance/cases/samples/appendix-b`, `projects/quill`, and the conformance README.

### Q66: What makes an include a cycle

- **Section:** SPEC §4.2, §8.2
- **Raised by:** phase 11
- **Status:** resolved (2026-09-28)
- **Ambiguity:** "Include cycles are an error." With section includes, a file can be included in part:

  ```
  _a.md:            ## Y  ...            ## X  @include: _a.md#y
  _b.md:            @include: _a.md#glossary        (a section of _a.md with no includes)
  _a.md also has:   @include: _b.md      (in another section)
  ```

  Is `_a.md → _b.md → _a.md#glossary` a cycle because it comes back to the file `_a.md`, or only if expansion would repeat?
- **Options:**
  1. A cycle is an include of a file that is already being expanded, whichever section.
  2. A cycle is an include that would expand the same file, or the same section of it, again while it's still being expanded; expansion of anything else ends.
- **Proposed resolution:** option 2: it reports exactly the includes whose expansion would never end, and it doesn't reject a fragment whose sections refer to each other's files, which option 1 would. A whole-file include and a section of the same file are different keys; a section that includes itself, or includes a file that includes it, is a cycle.
- **Implemented now:** option 2 (`// SPEC-QUESTION(Q66)` in `crates/tessera-resolve/src/expand.rs`). Either way it's reported at the include that closes the cycle (Q20).
- **Affects:** `crates/tessera-resolve`; phases 12 and 14.
- **Resolution:** approved by the repository owner: as proposed: a cycle is an include whose expansion would include the same section of the same file again. SPEC §4.2 now says so.

### Q67: What a heading's text is, for its slug

- **Section:** SPEC §5.5
- **Raised by:** phase 11
- **Status:** resolved (2026-09-28)
- **Ambiguity:** the slug is "computed from the heading's text (with phrases substituted)". A heading can hold more than text:

  ```
  ## Use `npm install` on **all** [hosts](hosts.md) ![logo](l.png)<br>now
  ```

  Whether code, link text, emphasis, an image's alt text, and inline HTML count, and what a line break is, decides the id, and the id has to be the one the consumer publishes (Astro takes the text of the rendered heading).
- **Options:**
  1. The text content of the rendered heading: text, code spans, link text, and emphasis count; an image and raw inline HTML contribute nothing; a line break is a newline.
  2. The heading's raw source text.
  3. Like option 1, but an image contributes its alt text.
- **Proposed resolution:** option 1, as Astro's heading pass reads a heading's text nodes, and verify it against Astro in phase 21. The `github` slugger and its Unicode handling are phase 09's; this is only what text goes in.
- **Implemented now:** option 1 (`// SPEC-QUESTION(Q67)` in `crates/tessera-resolve/src/index/headings.rs`).
- **Affects:** `crates/tessera-resolve`; phases 12 and 21.
- **Resolution:** approved by the repository owner: as proposed: the heading's rendered text content (text, code, link text, emphasis; not images or raw HTML), with phrases substituted. SPEC §5.5 now says so. Phase 21 checks it against Astro.

### Q68: An `@id` value that isn't valid, or a second `@id` on a heading

- **Section:** SPEC §4.1, §5.5
- **Raised by:** phase 11
- **Status:** resolved (2026-09-28)
- **Ambiguity:** an `@id` with characters other than letters, digits, and hyphens is an error (`id-invalid`), and "the id replaces the heading's slug". Does an invalid id still replace it? And what if a heading has two `@id` lines, or an `@id` with no value?

  ```
  ## Setup
  @id: my_id!
  ```
- **Options:**
  1. An `@id` with a value replaces the slug even if invalid, and the first of two wins; the error is reported once, and links that use the id the author wrote work.
  2. An invalid `@id` is ignored, so the heading keeps its slug, and a link to `my_id!` also fails.
- **Proposed resolution:** option 1: one mistake gives one diagnostic, not a cascade of missing-id errors (the same reasoning as Q36). An `@id` with no value has nothing to replace the slug with, so it's ignored.
- **Implemented now:** option 1 (`// SPEC-QUESTION(Q68)` in `crates/tessera-resolve/src/index/headings.rs`).
- **Affects:** `crates/tessera-resolve`; phases 10 and 14.
- **Resolution:** approved by the repository owner: option 1: an invalid `@id` still replaces the slug, and the first of several wins; each mistake is reported once. SPEC §4.1 now says so.

### Q51: Frontmatter that isn't valid YAML

- **Section:** SPEC §2.1, §8.2
- **Raised by:** phase 10
- **Status:** resolved (2026-09-28)
- **Ambiguity:** §8.2 has rows for frontmatter keys, missing fields, and types, but none for frontmatter that isn't YAML at all:

  ```
  ---
  title: [oops
  ---
  ```

  Nothing can be validated against the content type's schema until the YAML reads.
- **Options:**
  1. A new registry entry, `frontmatter-syntax` (an error, file level), whose message says what the YAML parser found.
  2. Report it as `frontmatter-type-mismatch` for the whole frontmatter (`frontmatter` must be a mapping of fields written in valid YAML, but it's invalid YAML).
  3. Don't report it. The page would then look as if it had no frontmatter.
- **Proposed resolution:** option 1, since a syntax error isn't a type mismatch and deserves its own message and code. **Implemented now: option 2**, the conservative one, because a new entry is a contract change (`SPEC-QUESTION(Q51)` in `crates/tessera-check/src/checks/frontmatter.rs`). The diagnostic is at the YAML parser's position, and no other frontmatter check runs on that file.
- **Affects:** `tests/conformance/diagnostics.toml` (a new entry), SPEC §8.2 (a new row); phases 10, 15.
- **Resolution:** approved by the repository owner: option 1: a new §8.2 row and registry entry, `frontmatter-syntax` (TSR122), at the YAML parser's position. SPEC §2.1 now requires valid YAML. Case: `frontmatter/not-valid-yaml`.

### Q52: Which files under the content root are source files

- **Section:** SPEC §2.1, §2.2
- **Raised by:** phase 10
- **Status:** resolved (2026-09-28)
- **Ambiguity:** §2.1 says a source file is "a CommonMark file with the extension `.md`", and §2.2 that the source files live under the content root. Neither says what to do with `.md` files in directories such as `.git/`, `.github/`, or `.tessera/`, with `.markdown` or `.MD` files, or with a `.md` file that isn't UTF-8.
- **Options:**
  1. Every file under the content root whose name ends in exactly `.md` is a source file, except those in a directory or with a name that begins with `.`. A source file that isn't UTF-8 stops the command (exit code 2) instead of being skipped.
  2. Every `.md` file, dot-directories included.
  3. Also accept `.markdown` and case variants.
- **Proposed resolution:** option 1. Dot-directories hold tool state, not documentation, and reading them would report problems in files the author doesn't own. Exactly `.md` follows §2.1 and the exact-case rule for names (§9.4). A file that can't be read is a failure of the command, like a missing `tessera.toml`, not a diagnostic, because there's no text to point at. Implemented now: option 1 (`SPEC-QUESTION(Q52)` in `crates/tessera-check/src/project.rs`).
- **Affects:** `tessera-check`'s `Project::load`; phases 11, 12, and 15 (which need the same set).
- **Resolution:** approved by the repository owner: option 1 for which files are sources (exactly `.md`, and a name beginning with `.` is skipped with everything in it), but a source that can't be read, or isn't UTF-8, is an error on that file, and the rest is still checked, rather than stopping the command: one bad file mustn't block the language server for a whole project. A new §8.2 row and registry entry, `source-unreadable` (TSR123, with an `encoding` variant). SPEC §2.1 now says so. Case: `files/source-not-utf8`.

### Q53: Where a link, image, or include diagnostic is reported

- **Section:** SPEC §5.2, §5.3, §8.1, §8.2
- **Raised by:** phase 10
- **Status:** resolved (2026-09-28)
- **Ambiguity:** §8.1 says a diagnostic is "reported at the source location that causes it". For a missing link target that could be the whole link (`[text](gone.md)`), its destination, or, for a reference-style link or image, the link reference definition that holds the destination. The conformance cases give only the line.
- **Options:**
  1. The destination as written, for an inline link or image; the whole link or image for a reference form, since the tree has no node for definitions (phase 05).
  2. Always the whole link or image.
  3. The definition, for a reference form.
- **Proposed resolution:** option 1 now, moving to option 3 when the parser exposes definitions (phases 12 and 23 need them anyway). The squiggle then covers the words the author has to change, and a fix (`link-route`) has an exact span to replace. The case `images/source-missing-reference` expects the image's line, which option 3 would change, so its expectation would need to name the definition's line at that point. Implemented now: option 1 (`SPEC-QUESTION(Q53)` in `checks/refs.rs`).
- **Affects:** conformance cases `images/source-missing-reference`, `images/attributes-on-reference-forms`; phases 12, 15, 23.
- **Resolution:** approved by the repository owner: option 1: at the destination as written for inline forms, and at the link or image for reference forms. SPEC §8.1 now says so.

### Q54: Phrases in a destination, and the file checks

- **Section:** SPEC §5.1, §5.2, §5.3
- **Raised by:** phase 10
- **Status:** resolved (2026-09-28)
- **Ambiguity:** §5.1 says phrases apply in link destinations, and §9.2 substitutes them (step 4) before links are resolved (step 6). §8.1 puts "whether referenced files exist" at file level. So does `[x]({api}streaming)` name the file `{api}streaming` or the URL `https://api.quill.dev/v3/streaming`? Read literally, the file-level check sees the source text and would report a missing file.
- **Options:**
  1. File-level checks substitute the declared phrases in a destination first, as the build does, so `{api}streaming` is external and needs no file. An undeclared `{key}` stays literal.
  2. File-level checks see the text as written and report `{api}streaming` as a missing file, or as a route.
- **Proposed resolution:** option 1: the SPEC's own example (`[streaming API reference]({api}streaming)`) must check cleanly, and Appendix B's page does. Implemented now: option 1, for inline and reference forms alike (`SPEC-QUESTION(Q54)` in `checks/refs.rs`). A phrase whose value contains `#` or a scheme therefore changes the destination's kind, as it does in the build.
- **Affects:** `projects/quill` (the Appendix B link); phases 11, 12.
- **Resolution:** approved by the repository owner: option 1: phrases in a destination are substituted before it's checked. SPEC §5.1 now says so.

### Q55: Which page a route names, and when to offer the fix

- **Section:** SPEC §5.2, §8.2, §9.5
- **Raised by:** phase 10
- **Status:** resolved (2026-09-28)
- **Ambiguity:** `link-route` says "this looks like the published route of `{page}`; link to the file instead: `{suggestion}`". The mapping from a route to a page belongs to the consumer profile's router (phase 12), but the warning is a file-level check and has to name a page now, including when no such page exists (`/guides/install/` with no `guides/install.md`).
- **Options:**
  1. The conventional mapping: `route.md`, else `route/index.md`, whichever exists in the project; else `route.md`. The warning offers a fix (an edit to the destination) only when a page exists.
  2. Ask the router (phase 12): a file-level check would then depend on the resolve crate.
  3. Warn without naming a page.
- **Proposed resolution:** option 1 until the router exists, then option 2 for the page and the suggestion. Implemented now: option 1 (`SPEC-QUESTION(Q55)` in `checks/refs.rs`). The suggestion keeps the destination's form (root-relative or relative to the file) and its `#id`.
- **Affects:** `link-route` diagnostics and their fixes; phases 12, 14, 24 (quick fixes).
- **Resolution:** approved by the repository owner: option 1, as an interim rule until the consumer profile's router (phase 12) can name the page: `route.md`, else `route/index.md`, whichever is a source file; the fix is offered only when that page exists. No spec change: the spec leaves routing to the consumer profile.

### Q56: Where `phrase-undeclared` applies

- **Section:** SPEC §5.1, §8.2
- **Raised by:** phase 10
- **Status:** resolved (2026-09-28)
- **Ambiguity:** the warning is for "`{key}` in prose whose key isn't declared". §5.1 lists where phrases apply: prose, headings, link text, link destinations, fences that opt in, and frontmatter fields. Which of those count as prose for this warning?
- **Options:**
  1. Every inline position where a phrase candidate is recorded: paragraphs, headings, link text, image alt text, table cells, titles, and text primaries. Not destinations, `phrases=true` fences, or frontmatter.
  2. Also destinations and opted-in fences.
  3. Only paragraphs and headings.
- **Proposed resolution:** option 1. A destination's `{key}` is checked as part of the destination (Q54), a fence that opts in has said it wants substitution and probably knows its keys, and frontmatter phrases depend on the content model's declarations (phase 08). Implemented now: option 1 (`SPEC-QUESTION(Q56)` in `checks/mod.rs`).
- **Affects:** phase 11 (frontmatter phrases), phase 12 (the registry-change report of §5.1).
- **Resolution:** approved by the repository owner: option 1: every inline position, but not destinations, `phrases=true` fences, or frontmatter. SPEC §5.1 now says so.

### Q57: `available` and `variant` frontmatter values that aren't the right shape

- **Section:** SPEC §2.1, §4.3, §4.4, §8.2
- **Raised by:** phase 10
- **Status:** resolved (2026-09-28)
- **Ambiguity:** §4.3 says `variant` is "a mapping from dimension names to a value or a list of values" and §4.4 that `available` holds a spec. §8.2 has no row for `available: 3` (a number) or `variant: cloud` (not a mapping) or `variant: {pm: [npm, 3]}`. The content model can't declare these keys, so `validate_frontmatter` accepts them as they are.
- **Options:**
  1. `frontmatter-type-mismatch`, at the value, with the expected shape in the message.
  2. `available-syntax` for `available`, and `variant-unknown` for `variant`.
  3. Not reported.
- **Proposed resolution:** option 1: the value has the wrong type, which is what that entry is for, and the message can name the right shape. Implemented now: option 1 (`SPEC-QUESTION(Q57)` in `checks/frontmatter.rs`).
- **Affects:** phases 12 and 14 (they read these keys after this check passes).
- **Resolution:** approved by the repository owner: option 1: a type mismatch, with the expected shape in the message. SPEC §2.1 now says so.

### Q58: A content model with errors, and the content model's warnings

- **Section:** SPEC §8.1; phase 10's exit codes
- **Raised by:** phase 10
- **Status:** resolved (2026-09-28)
- **Ambiguity:** `tessera check` exits `0`, `1`, or `2`, and `2` is for "usage or configuration failures". A `tessera.toml` with errors (the loader's rules, SPEC §7.2) isn't a usage error, but no source file can be checked against a model that doesn't load. And the loader's warnings (`model-name-case`, `model-build-filter-excluded`) belong to no source file.
- **Options:**
  1. A model with errors is a configuration failure: the model's diagnostics are shown, nothing else is checked, and the exit code is `2`. A model that loads has its warnings in the file-level list, so they count under `--deny-warnings` and appear in the same output, JSON included.
  2. A model with errors exits `1`, like any other errors.
  3. The model's warnings are shown by the build only.
- **Proposed resolution:** option 1. Exit code `1` means "the documentation has problems you can fix by editing it", and `2` means "the tool can't tell". Implemented now: option 1 (`SPEC-QUESTION(Q58)` in `commands/check.rs` and `check_files`).
- **Affects:** `tessera check`; phases 15 (the server reports model diagnostics on `tessera.toml`) and 18.
- **Resolution:** approved by the repository owner: option 1: a content model with errors is reported and nothing else is checked (exit code 2); a model's warnings are reported with the rest. SPEC §7.1 now says so.

### Q59: A link or image whose destination is only a fragment, or empty

- **Section:** SPEC §5.2, §5.3
- **Raised by:** phase 10
- **Status:** resolved (2026-09-28)
- **Ambiguity:** `[here](#install)` names a heading in the file it's written in (page level checks the id). `![a](#x)` and `![a]()` have no file to look for. §5.3 says "a local image source MUST exist".
- **Options:**
  1. File-level checks skip a destination with no path (the fragment-only link names the file itself, and an empty destination names nothing), for links and images alike.
  2. Report an image with no path as `image-source-missing`; skip links.
- **Proposed resolution:** option 2 for images, and option 1 for links: an image with no source is certainly wrong. **Implemented now: exactly that** (`SPEC-QUESTION(Q59)` in `checks/refs.rs`). The registry's message for `image-source-missing` reads badly for an empty path ("the image `(no source)` doesn't exist"; a `#id`-only source shows as written). A dedicated variant, such as `messages.empty = "this image has no source; give it a path between the parentheses"`, needs a registry change, which is the human's to approve when resolving this question.
- **Affects:** `image-source-missing`; phases 12 and 14.
- **Resolution:** approved by the repository owner: option 2 for images and option 1 for links: an image with an empty source is `image-source-missing` with a new `empty` message variant ("this image has no source"); a link with only `#id` names its own file. SPEC §5.2 and §5.3 now say so.

### Q81: Page-level problems in content a build removes

- **Section:** SPEC §8.1, §9.3, §5.2
- **Raised by:** phase 12
- **Status:** resolved (2026-09-28)
- **Ambiguity:** page-level validation runs "once per build" on the resolved page. The spec says a link to a page a build drops is an error in that build, and that to link to such a page from shared content, put the link in a `@variant` arm the same build removes, which implies a problem in content a build removes isn't reported for that build. It doesn't say the same for the other page-level rows. Example: `@available: self-managed` under a page whose `available: cloud, self-managed 3.3` exceeds its scope, inside an arm `@variant {deployment=self-managed}` that the `cloud-only` build removes; or an `@include: _f.md#missing` in the same arm.
- **Options:**
  1. Every page-level problem found on the expanded page is reported in every build, whether or not the build keeps the content. A build then fails for content it doesn't publish.
  2. A build reports only problems about content it publishes: the same rule the spec gives for links. A problem in content that every build removes is reported by no build, so the phase that runs the page-level checks should include a build that keeps everything (the editor's build, or `switch` builds).
- **Proposed resolution:** option 2, for every page-level row that depends on what survives: `variant-no-arm-survives` (recorded only when the group itself survives), `available-exceeds-scope`, `include-id-missing`, `include-cycle`, `link-id-removed`, and `link-page-dropped`. Implemented now: option 2 (`SPEC-QUESTION(Q81)` in `crates/tessera-resolve/src/build/mod.rs`; the link rows are in `links.rs`). Problems that don't depend on a build's content (`id-duplicate`, `heading-duplicate-without-id`, computed from the resolved page's headings) follow from the same rule, since they're about the surviving headings.
- **Affects:** `crates/tessera-resolve/src/build/`; phase 14 (which builds it checks); phases 15 and 18.
- **Resolution:** approved by the repository owner: as proposed: a build reports page-level problems only in content it publishes. SPEC §8.1 now says so. Content that no build publishes is phase 14's to handle.

### Q82: A spec that lists a target both directly and through its dimension name

- **Section:** SPEC §4.4, §9.3
- **Raised by:** phase 12
- **Status:** resolved (2026-09-28)
- **Ambiguity:** content is available for target *T* when its spec "lists *T*, directly or through *T*'s dimension name", and the state in effect for *T* counts as available. A spec can list both: `deployment, cloud removed`. Which entry decides the state for `cloud`: `deployment` (generally available) or `cloud removed`?
- **Options:**
  1. The direct entry, which is the more specific.
  2. Available if any entry that lists *T* says so.
  3. Not available if any entry that lists *T* says so.
- **Proposed resolution:** option 1: a more specific entry overrides a general one, the way a value overrides a dimension name elsewhere in the spec (`@variant`). Implemented now: option 1 (`SPEC-QUESTION(Q82)` in `crates/tessera-resolve/src/build/availability.rs`). Two entries for the same target (`cloud, cloud removed`) take the first.
- **Affects:** `crates/tessera-resolve/src/build/availability.rs`; conformance cases: none.
- **Resolution:** approved by the repository owner: option 1: the direct entry decides; of two entries for the same target, the first. SPEC §4.4 now says so.

### Q83: Several `@available` lines for one heading or block

- **Section:** SPEC §3.8, §4.4
- **Raised by:** phase 12
- **Status:** resolved (2026-09-28)
- **Ambiguity:** heading-bound and following-block directives "stack". Two `@available` lines at the top of a section, or above one block, are each valid. What are they together?

  ```
  ## Streaming
  @available: cloud
  @available: cloud beta
  ```
- **Options:**
  1. Content must be available under every one (each narrows the previous), and each is checked against the scope it sits in, in order.
  2. The first wins; the rest are ignored.
  3. It's an error.
- **Proposed resolution:** option 1, with the scope check applied to each in turn, so each mistake is reported once and nothing is dropped by a guess. Implemented now: option 1 (`SPEC-QUESTION(Q83)` in `availability.rs`). Option 3 would need a registry entry.
- **Affects:** `crates/tessera-resolve/src/build/availability.rs`; the registry, if option 3.
- **Resolution:** approved by the repository owner: option 1: every `@available` on a scope applies, and each is checked against its enclosing scope. SPEC §4.4 now says so.

### Q84: A heading-bound `@available` whose heading an include left out

- **Section:** SPEC §3.8, §4.2, §4.4
- **Raised by:** phase 12
- **Status:** resolved (2026-09-28)
- **Ambiguity:** bindings are decided per source file. `@include {heading=false}: _f.md#install` leaves out the section's heading but not the `@id` and `@available` lines under it, which were bound to that heading. After expansion they sit under whatever heading the including page has above the include (or none).

  ```
  _f.md:                              index.md:
  ## Install                          ## Setup
  @available: cloud                   @include {heading=false}: _f.md#install
  Run it.
  ```
- **Options:**
  1. Bindings stay per source file. A heading-bound `@available` whose heading is gone describes what is left of its section: the rest of the included content.
  2. It re-binds to the including page's heading above (`Setup`), so `Run it.` and the rest of `Setup` are `cloud`-only.
  3. It's dropped: nothing is annotated.
- **Proposed resolution:** option 1: a fragment means the same wherever it's included, which is the point of SPEC §4.2's per-file rules, and nothing is silently dropped or widened. Implemented now: option 1 (`SPEC-QUESTION(Q84)` in `availability.rs`). The same reading applies to a fragment whose first lines are `@available` with no heading above: it binds the block it touches, as at the start of a document (§3.8), not the including page's heading.
- **Affects:** `crates/tessera-resolve/src/build/availability.rs`; conformance cases: none.
- **Resolution:** approved by the repository owner: option 1: bindings are decided per source file, so the directive describes the rest of the included section. SPEC §4.2 now says so.

### Q85: Glossary matching, beyond the content model's rules

- **Section:** SPEC §5.4, content-model.md §13
- **Raised by:** phase 12
- **Status:** resolved (2026-09-28)
- **Ambiguity:** content-model.md §13 fixes whole-word matching, longest match, prose only, and `first` or `every`. It doesn't say: whether emphasis inside prose counts as prose; where the text is matched, the source or the page after phrases are substituted; what `first` counts when a fragment is included twice or a build removes the first occurrence; whether a term links from its own page; and what happens to a term whose page or `#id` a build doesn't publish.
- **Options:** (a) match in the resolved text, in document order across the resolved page, emphasis included; a term is never linked on the page it links to; a term whose page the build doesn't publish, or whose id it removes, isn't linked there. (b) The same, but a term whose target isn't published is an error (`link-page-dropped`-like). (c) Match the source text.
- **Proposed resolution:** option (a). A glossary link is a convenience, not something an author wrote, so a target the build lacks should skip the link rather than fail the build; matching the resolved text is what a reader sees, and "the first occurrence on each page" means the resolved page (content-model.md §13.1). Implemented now: option (a) (`SPEC-QUESTION(Q85)` in `crates/tessera-resolve/src/build/glossary.rs`).
- **Affects:** `crates/tessera-resolve/src/build/glossary.rs`; phases 18 and 20 (they render the links); phase 15 (hover).
- **Resolution:** approved by the repository owner: option (a): matched in the resolved text, emphasis counts as prose, no self-links, and unpublished targets aren't linked. SPEC §5.4 now says so.

### Q86: A state with no version on a versioned target

- **Section:** SPEC §4.4, §9.3
- **Raised by:** phase 12
- **Status:** resolved (2026-09-28)
- **Ambiguity:** "A bare target with no version is in effect at every version", and "each state names only the version where it begins". A versioned target given a state with no version (`self-managed preview`) parses, and `check_availability` doesn't report it. In a filter build at `self-managed 3.3`, is the content in preview from the start of time, or never in effect?

  ```
  @available: self-managed preview
  ```
- **Options:**
  1. The state is in effect at every version (like a bare target).
  2. No version is at or after "no version", so it's never in effect: the content is never available.
  3. It's an error (`available-versionless` reads the wrong way round: this is a *missing* version).
- **Proposed resolution:** option 1, which keeps content and matches "a bare target … at every version". Option 3 needs a registry entry and is the better long-term rule, since the lifecycle then always says when it began. Implemented now: option 1 (`SPEC-QUESTION(Q86)` in `availability.rs`).
- **Affects:** `crates/tessera-resolve/src/build/availability.rs`; possibly the registry and phase 10.
- **Resolution:** approved by the repository owner: option 1: a state with no version on a versioned target is in effect at every version, as a bare target is. SPEC §4.4 now says so.

### Q91: File ids when files are created, deleted, or renamed

- **Section:** SPEC §8.1 (locations), phases 10, 11, and 15
- **Raised by:** phase 13
- **Status:** resolved (2026-09-28)
- **Ambiguity:** Every location names its file by a `FileId`. Phases 10 and 11 number source files 1, 2, … in path order at load, and `tessera.toml` is 0. That is stable only while the file set is: create a file whose path sorts first and, renumbered, every other file's id (and every diagnostic located in it) changes. The language server (phase 15) keeps a project alive across edits and publishes diagnostics by file, so it needs ids that mean the same thing from one update to the next, and `tessera-check` and `tessera-resolve` must agree.
- **Options:**
  1. Renumber in path order after every change. Simple; every id can change on any create or delete, so no result can be kept.
  2. An id names a **path**: assigned when a file first appears at that path, never changed, never reused for another path. A deleted file's id names no live file; a file that returns at the same path gets its id back. A rename is a deletion and a creation.
  3. An id names a **file**: a rename keeps the id.
- **Proposed resolution:** option 2. Ids are stable per path for the life of one project (one `FileIds` table); a fresh load numbers in path order, as before; a stale result is one located at an id no live file has. Option 3 needs the project to guess that a deletion and a creation are one move, and everything about the file is re-derived after a rename anyway (relative references resolve from its path). Implemented now: option 2. `tessera_check::Project` accepts any ids (it finds a file by id, not position), so the language server builds one from a snapshot's ids.
- **Affects:** `crates/tessera-resolve/src/incremental/ids.rs` (`SPEC-QUESTION(Q91)`), `Project::load_with_ids`; phase 15; `tessera-check`'s documentation of ids.
- **Resolution:** approved by the repository owner: as proposed: a file id names a path; an edit never changes it and it's never reused for another path; a rename is a deletion plus a creation; a file that returns at the same path gets its id back. Fresh loads still number in path order from 1, with `tessera.toml` as 0. Implementation behavior, not language: no SPEC change.

### Q92: A change to the content root or output directory

- **Section:** `content-model.md` §2 (`[project]`), PLAN.md (VS Code extension)
- **Raised by:** phase 13
- **Status:** resolved (2026-09-28)
- **Ambiguity:** A content path is relative to the content root, and the asset boundary depends on the output directory. If `tessera.toml` changes either, every path means something else, and every file's identity changes. The phase says a model change re-parses, re-indexes, or re-resolves, but not what to do with this one.
- **Options:**
  1. Apply it in place: re-enumerate the files from the file system.
  2. Refuse it (`ApplyError::LayoutChanged`) and leave the caller to load a new project.
- **Proposed resolution:** option 2. PLAN.md already says the client restarts the server "when `tessera.toml` changes in ways the server can't reload in place". Nothing is applied when the error is returned. Implemented now: option 2.
- **Affects:** `crates/tessera-resolve/src/incremental/mod.rs` (`SPEC-QUESTION(Q92)`); phase 15.
- **Resolution:** approved by the repository owner: as proposed: a model change that moves the content root or output directory is refused (`ApplyError::LayoutChanged`), and the caller loads a new project. Implementation behavior, not language: no SPEC change.

### Q93: What of a target can change how a link resolves

- **Section:** SPEC §5.2, §5.5, §9.2 step 6
- **Raised by:** phase 13
- **Status:** resolved (2026-09-28)
- **Ambiguity:** When a file changes, which *other* pages must be re-resolved? A link's resolved form is its target's route, page id, and, when it has no text, its title or heading text, and whether the build publishes the target at all and keeps the heading. The spec lists these inputs but never says that nothing else of the target matters, and that is what lets an edit to a paragraph leave the pages that link to it alone.
- **Options:**
  1. Any change to a file re-resolves the pages that link to it (always correct, never fast).
  2. Only a change to what the spec lists: the file's kind, its frontmatter (title, `variant`, `available`), its headings (level, text, `@id`), and every directive line and how blocks nest (`@available`, `@variant`, `@include` decide which headings survive a build).
- **Proposed resolution:** option 2. The differential test (every step of thousands of random sequences) compares what a consumer holds after redoing only the listed pages with a from-scratch build, so an input missing from the list fails a test. If a later phase adds something links depend on (a new directive that affects heading ids, say), the signature in `signature.rs` must include it. Implemented now: option 2.
- **Affects:** `crates/tessera-resolve/src/incremental/signature.rs` (`SPEC-QUESTION(Q93)`); phases 14, 18, 20 if they add link inputs.
- **Resolution:** approved by the repository owner: as proposed: only a file's kind, frontmatter, headings, and directive structure can change how a link to it resolves; editing prose leaves the pages that link to it alone. Implementation behavior, not language: no SPEC change.

### Q94: Batches, and updates that change nothing

- **Section:** phase 13 tasks 1 and 3
- **Raised by:** phase 13
- **Status:** resolved (2026-09-28)
- **Ambiguity:** An editor and a file watcher report changes in bursts (a `git checkout` is dozens; a save is a delete and a create). Is each change its own version, or is a batch one? And is a version needed when nothing anyone can observe changed (the same text again; an image no one references appearing)?
- **Options:**
  1. One version per change.
  2. One version per call to `apply`, taking the batch's **net effect** per path (created then deleted is nothing; edited back is nothing); none when nothing observable changed.
- **Proposed resolution:** option 2. A consumer that publishes diagnostics per snapshot would otherwise publish for states no one saw, and would drop results for updates that changed nothing. Ids for files created in a batch are assigned in path order, whatever order the batch lists them. Implemented now: option 2.
- **Affects:** `crates/tessera-resolve/src/incremental/mod.rs` (`SPEC-QUESTION(Q94)`); phase 15.
- **Resolution:** approved by the repository owner: as proposed: a batch is applied by its net effect per path, and an update that changes nothing observable produces no new version. Implementation behavior, not language: no SPEC change.

### Q95: How long a result stays comparable across snapshots

- **Section:** phase 13 task 3
- **Raised by:** phase 13
- **Status:** resolved (2026-09-28)
- **Ambiguity:** The language server must not publish a result computed from an out-of-date snapshot. "Out of date" for a file means a later update affected it (its text, or a file it depends on), which needs a record of what each update affected. How much record is kept?
- **Options:**
  1. Keep every update's `Affected` forever (unbounded).
  2. Keep the last 256; a snapshot older than that is treated as stale for every file.
- **Proposed resolution:** option 2. A consumer has never been 256 updates behind unless something is wrong, and "stale" is the safe answer. Implemented now: option 2 (`IncrementalProject::is_file_current`).
- **Affects:** `crates/tessera-resolve/src/incremental/mod.rs` (`SPEC-QUESTION(Q95)`); phase 15.
- **Resolution:** approved by the repository owner: as proposed: the last 256 updates are kept for `is_file_current`; a snapshot older than that is treated as stale. Implementation behavior, not language: no SPEC change.

### Q96: A model that changes only what tessera.toml says about itself

- **Section:** `content-model.md` §20
- **Raised by:** phase 13
- **Status:** resolved (2026-09-28)
- **Ambiguity:** The model's warnings carry spans in `tessera.toml`. Reformatting the file, or adding a comment, changes those spans without changing what the model means. Does that count as a model change?
- **Options:**
  1. Yes: every file is re-checked.
  2. No for files: the model is swapped in (so `Project::model().warnings` is current) and `Affected::model` is `Some(ModelImpact::Warnings)`, which lists no file; the caller refreshes the diagnostics of `tessera.toml` (file id 0) itself.
- **Proposed resolution:** option 2. Every other model change reaches every file, at the tier of the stage that reads what changed (`ModelImpact`): directive keywords and note types reparse, phrases, fragment patterns, and the slugger re-index, anything else re-checks and re-resolves. The classification destructures `ContentModel` without `..`, so adding a field to the model is a compile error until someone decides which stage reads it. Implemented now: option 2.
- **Affects:** `crates/tessera-resolve/src/incremental/signature.rs` (`SPEC-QUESTION(Q96)`); phase 15.
- **Resolution:** approved by the repository owner: as proposed: a model change that affects only the model's own warnings changes no file. Implementation behavior, not language: no SPEC change.

### Q97: A diagnostic that names a file spelled with different case

- **Section:** SPEC §9.4
- **Raised by:** phase 13
- **Status:** resolved (2026-09-28)
- **Ambiguity:** SPEC §9.4 says names match exactly on every platform, and the diagnostics for `include-target-missing` and `link-target-missing` have a `case` variant that names the file whose name differs only in case. So the diagnostic in `a.md` depends on a file `Guide.md` even though `a.md` names `guide.md`: creating or deleting `Guide.md` changes `a.md`'s diagnostics. The spec doesn't say the dependency exists.
- **Options:**
  1. Track it: creating or deleting a file re-checks every file with a reference (link, image, or include) whose target differs from it only in case.
  2. Don't: the diagnostic goes stale until `a.md` is next edited.
- **Proposed resolution:** option 1, which costs a lowercased-path index. Implemented now: option 1. The differential test found it.
- **Affects:** `crates/tessera-resolve/src/incremental/mod.rs` (`SPEC-QUESTION(Q97)`).
- **Resolution:** approved by the repository owner: as proposed: a diagnostic that names a case-differing twin file depends on that file, so creating, deleting, or renaming the twin re-checks it. Implementation behavior, not language: no SPEC change.

### Q101: Page-level problems in content that no build publishes

- **Section:** SPEC §8.1, §9.3
- **Raised by:** phase 14 (from phase 12's note on Q81)
- **Status:** resolved (2026-09-28)
- **Ambiguity:** with Q81 each build reports only problems in content it publishes. Content that *no* build publishes (an arm none of the builds selects, a page every build drops) is then never checked at page level: a duplicate id, a bad include, or a link to a missing id in it is never reported, and it would surface only when someone adds a build that publishes it. The spec doesn't say whether page-level validation covers it.
- **Options:**
  1. Report nothing: the content isn't published, so it isn't the build's business.
  2. Check it with one extra resolution that keeps everything (`switch` and `badge`), and report the problems whose cause is in content no build publishes, as belonging to no build. A problem that exists only because arms that no build keeps together are kept together isn't reported.
  3. Add a warning (a new registry entry, "content that no build publishes") and check nothing else in it.
- **Proposed resolution:** option 2. It needs no new diagnostic, it finds what a later build would, and it costs nothing when a build already keeps everything (a `switch` and `badge` build, which most projects have). The diagnostic keeps its own row and severity; `Diagnostic::builds` is empty and `Diagnostic::unpublished` is set, and `tessera check` adds "in content that no build publishes" to its message. Option 3 is a reasonable addition on top, since dead content is worth knowing about. Implemented now: option 2 (`SPEC-QUESTION(Q101)` in `crates/tessera-check/src/page/mod.rs`).
- **Affects:** `crates/tessera-check/src/page/`; phases 15 and 18 (they show or fail on these); the registry only if option 3 is added.
- **Resolution:** approved by the repository owner: as proposed: content no build publishes is checked with one extra pass that keeps everything, and its problems are reported as belonging to no build (`Diagnostic::unpublished`). SPEC §8.1 now says so.

### Q102: One problem in several builds

- **Section:** SPEC §8.1, §8.2
- **Raised by:** phase 14
- **Status:** resolved (2026-09-28)
- **Ambiguity:** page-level validation runs once per build, and the rows `variant-no-arm-survives`, `link-id-removed`, and `link-page-dropped` name the build in their message ("build `{build}` removes …"). When two builds have the same problem, one diagnostic can't name a single build, and the spec says a problem is reported once.
- **Options:**
  1. One diagnostic per build, each naming its build.
  2. One diagnostic, its message naming every build, and the builds listed as data.
  3. One diagnostic whose message doesn't name a build, with the builds as data only (a registry change to the three messages).
- **Proposed resolution:** option 2 now, reading "build `cloud`, `self-managed` removes …": accurate, but not good English. The better fix is a `builds` message variant on each of the three rows, chosen when a diagnostic is in several builds, with the placeholder `{builds}` filled with the names quoted and joined (`` `cloud`, `cloud-pdf` ``). The exact texts, to approve and add in one step to `tests/conformance/diagnostics.toml`:

  ```toml
  # variant-no-arm-survives
  messages.builds = "builds {builds} remove every arm of this `@variant` group, so none of its content is published in those builds"
  # link-id-removed
  messages.builds = "builds {builds} remove the heading `{id}` from `{path}`, so this link would be broken in those builds"
  # link-page-dropped
  messages.builds = "builds {builds} don't publish `{path}`, so this link would be broken in those builds; move the link into a `@variant` arm those builds remove"
  ```

  With them, `check_all_builds` sets `variant = "builds"` and a `builds` argument when a merged diagnostic has more than one build, and nothing else changes. Until then the merged `{build}` is the builds joined by `` `, ` ``. Every diagnostic that appears in some but not all builds also gets "(only in build `a`)" from `tessera check`, and the builds are `Diagnostic::builds` for every tool. Implemented now: option 2 (`SPEC-QUESTION(Q102)` in `crates/tessera-check/src/page/mod.rs`).
- **Affects:** `tests/conformance/diagnostics.toml` (three messages, if option 3 or the plural variants are chosen); phases 15 and 18.
- **Resolution:** approved by the repository owner: option 2 with the plural wording: one diagnostic per problem, naming every build it appears in (`Diagnostic::builds`, and `builds` in the JSON output). The three rows whose message names the build gained a `builds` message variant, with the texts proposed here, used when the problem is in more than one build. SPEC §8.1 now says so.

### Q103: What "duplicates another heading's text" compares

- **Section:** SPEC §5.5, §8.2
- **Raised by:** phase 14
- **Status:** resolved (2026-09-28)
- **Ambiguity:** `heading-duplicate-without-id` is reported for a heading with no `@id` that "duplicates another heading's text on the page". Two headings can differ in text and still get the same slug (`Options` and `options`, `Set up` and `Set-up`), so the later one's id is numbered and can change when headings move, which is what the warning is about. The row says text, not slug.
- **Options:**
  1. Exactly equal text (after phrases are substituted, as the slug is computed): the row as worded.
  2. Equal slugs: every heading whose id would be numbered.
- **Proposed resolution:** option 1, which is the row's wording and reports fewer warnings. Option 2 is the more useful rule, and a change to the row's wording. The earlier heading may have an `@id` or not; the later one must not. An empty text is never a duplicate (its own row, `heading-empty-slug`, covers it). Implemented now: option 1 (`SPEC-QUESTION(Q103)` in `crates/tessera-check/src/page/collect.rs`).
- **Affects:** `crates/tessera-check/src/page/collect.rs`; possibly the row's wording.
- **Resolution:** approved by the repository owner: option 2: equal slugs, not equal text, among headings without `@id` (the ones numbered against each other, Q7), so `Set up` and `Set-up` are duplicates. A heading whose slug equals an earlier heading's explicit id is `id-duplicate`, not this warning. The §8.2 row, its registry row and message, and SPEC §5.5 now say slug. Cases: `headings/duplicate-slug-different-text` (new); `directives/id/slug-equals-explicit-id` and `headings/explicit-id-is-stable-source-and-page-id` no longer expect this warning.

### Q104: Fragments that no page includes

- **Section:** SPEC §8.1
- **Raised by:** phase 14
- **Status:** resolved (2026-09-28)
- **Ambiguity:** page-level validation runs on pages "after includes are expanded". A fragment that no page includes is never part of a page, so the rows that need one (a link to a missing id in it, a duplicate id inside it) are never reported for it, and neither are the page-level rows about content it holds; only its file-level rows are. This is the same gap as Q101 for a whole file, and one an author meets while writing a fragment before including it.
- **Options:**
  1. Report nothing for it at page level.
  2. Check each such fragment as if it were a page of its own.
  3. Warn that nothing includes it (a new registry entry).
- **Proposed resolution:** option 1 for now, because the rows are defined on pages and a fragment's ids may legitimately collide with the page's own once it's included. Option 3 is cheap and useful, and needs a registry entry. Implemented now: option 1 (`SPEC-QUESTION(Q104)` in `crates/tessera-check/src/page/mod.rs`).
- **Affects:** `crates/tessera-check/src/page/`; phase 15 (a fragment open in the editor).
- **Resolution:** approved by the repository owner: as proposed: a fragment no page includes has no page-level diagnostics; its file-level ones are still reported. A "nothing includes this" warning can come later. SPEC §8.1 now says so.

### Q111: What a plain-markdown page starts with

- **Section:** SPEC §9.4
- **Raised by:** phase 18
- **Status:** resolved (2026-09-28)
- **Ambiguity:** SPEC §9.4 says the plain output is "fully resolved CommonMark" for language models, search indexing, and export, and its table covers the body's constructs. It doesn't say what becomes of the frontmatter. A Tessera page's title lives in the frontmatter (`title`, required by content-model.md decision 3), and the body usually starts at a level-2 heading, so a page written out as its body alone has no title. Page-level `available` is frontmatter too, and the table's `@available` row covers only the directive.
- **Options:**
  1. The body only. The output loses the page's title and page-level availability.
  2. The frontmatter as YAML at the top. A plain CommonMark parser reads it as a thematic break and a setext heading, and "parses as ordinary CommonMark" no longer means what it says.
  3. The title as a level-1 heading, then the page-level availability as the same `Available:` line the directive gets, then the body. The rest of the frontmatter (`description`, custom fields) isn't in the plain output; the JSON output has it.
- **Proposed resolution:** option 3. Implemented now: option 3 (`SPEC-QUESTION(Q111)` in `plain/mod.rs`). A page with no `title` gets no heading.
- **Affects:** `crates/tessera-emit/src/plain/`; the plain snapshots; phase 20 (the site output passes availability through as frontmatter and doesn't need this).
- **Resolution:** approved by the repository owner: as proposed: a plain page begins with its title as a level-1 heading and its page-level availability line; other frontmatter isn't included. SPEC §9.4 now says so.

### Q112: Raw HTML in the source, in the plain output

- **Section:** SPEC §9.4, §5.3
- **Raised by:** phase 18
- **Status:** resolved (2026-09-28)
- **Ambiguity:** the plain output is "fully resolved CommonMark with no HTML". SPEC §9.4 also says raw HTML in a source page passes through unchanged (for references inside it, "they pass through unchanged"). Both can't hold for a page that has an HTML block or inline HTML, such as `<kbd>Ctrl</kbd>` or a `<div>`. Example: `Press <kbd>Ctrl</kbd>+C.`
- **Options:**
  1. Pass the HTML through. The output then has HTML, and the acceptance test for "no HTML" fails on such a page.
  2. Drop it, tags and content. Content is lost silently.
  3. Write the HTML as literal text (`Press \<kbd>Ctrl\</kbd>+C.`): the characters stay, no tag reaches a parser, and a reader sees what was written.
  4. Strip the tags and keep the text between them (`Press Ctrl+C.`), which reads best but needs an HTML parser, and can't handle a `<div>` full of markup.
- **Proposed resolution:** option 3, the conservative one (it keeps content and meets "no HTML"). Option 4 is friendlier for the inline case and is a possible refinement. Implemented now: option 3 (`SPEC-QUESTION(Q112)` in `plain/mod.rs`; inline HTML goes through the same text escaping).
- **Affects:** `crates/tessera-emit/src/plain/`; the JSON output keeps the raw HTML as `html` nodes.
- **Resolution:** approved by the repository owner: changed from the proposal: raw HTML keeps its text and loses its tags (`<kbd>Ctrl</kbd>` becomes `Ctrl`), and comments and `<script>`/`<style>` contents are dropped, rather than the tags being written as escaped text. SPEC §9.4 now says so; `tessera_emit`'s plain emitter implements it (`html_text`).

### Q113: What plain markdown shows of a widget besides its fallback and content

- **Section:** SPEC §6, §9.4; content-model.md §15, decision 8
- **Raised by:** phase 18
- **Status:** resolved (2026-09-28)
- **Ambiguity:** a project widget becomes "its plain fallback, or nothing", and content-model.md keeps wrapped content unless `plain-content = "drop"`. Not settled: (a) a widget's own title line, text primary, and attributes: `@quill-aside {tone=x}: Text` and `.Heading` above a titled widget; (b) a groupable widget (`@quill-compare`), whose arms are alternatives; (c) whether a following-block widget's bound block counts as wrapped content.
- **Options:**
  1. (a) Show the widget's title and primary as text. (b) One fallback per arm. (c) The bound block isn't content.
  2. (a) The title, primary, and attributes are the widget's own parameters, which the fallback stands for, and aren't shown. (b) The fallback once, then the arms as labeled sections (as for `@variant`), unless `plain-content = "drop"`. (c) A block bound by `binding = "block"` is wrapped content, kept or dropped like a container's.
- **Proposed resolution:** option 2, since the fallback is "a static string, no attribute interpolation" and stands in for the whole widget. A widget with no fallback shows only its content, so nothing is lost silently unless the model says `drop`. Implemented now: option 2 (`SPEC-QUESTION(Q113)` in `plain/mod.rs`).
- **Affects:** `crates/tessera-emit/src/plain/`; content-model.md §15 if it should say so.
- **Resolution:** approved by the repository owner: as proposed: a widget's plain output is its fallback followed by its content (unless `plain-content = "drop"`); its title, primary, and attributes aren't shown; a widget group shows the fallback once, then the arms. SPEC §9.4 now says so.

### Q114: The wording of the availability line

- **Section:** SPEC §9.4
- **Raised by:** phase 18
- **Status:** resolved (2026-09-28)
- **Ambiguity:** the spec's example, "Available: Quill Cloud (GA); self-managed (preview, 3.4+)", fixes the shape for two entries, and not the rest: an entry with no state (`cloud`), a bare version (`self-managed 3.2`), a history (`self-managed (preview 3.3, ga 3.5)`), a target that is a dimension name (`deployment`), a state with no version, and where the line goes for a page-level or heading-bound spec.
- **Options:** for each entry: label, then in parentheses the state's display label, then `, N+` for the version where it begins.
- **Proposed resolution:** an entry with no state is `GA` (it's generally available, SPEC §9.3); a bare version is `GA, 3.2+`; a history lists each step, `preview 3.3+, GA 3.5+`, inside one pair of parentheses; a dimension name shows the dimension's label; a state with no version shows only the state; entries are joined by `; `. The line is a paragraph of its own: under the title for page-level availability, where the `@available` line was for a heading-bound one (after the heading), and before the block for a block-bound one. State labels are the model's (`ga` is `GA` by default). Implemented now: exactly that (`SPEC-QUESTION(Q114)` in `labels.rs`).
- **Affects:** `crates/tessera-emit/src/labels.rs`; the plain snapshots. The JSON output gives the same text as `display`.
- **Resolution:** approved by the repository owner: changed to match the element contract (`packages/elements/CONTRACT.md` §4), which says the plain output uses its text per target: a history names each state and the version it begins at, with no `+` (`Self-managed (preview 3.3, GA 3.5, deprecated 4.0)`); a single state or bare version keeps `3.4+`. SPEC §9.4 now says so.

### Q115: Labels of a group's arms, and a note with no title

- **Section:** SPEC §9.4, §4.3
- **Raised by:** phase 18
- **Status:** resolved (2026-09-28)
- **Ambiguity:** "each arm as a section with a bold label" and "labels for dimension values come from the content model's display labels". Not settled: an arm that names a value set (`{platform=cloud|on-prem}`) or several dimensions (`{deployment=cloud, pm=npm}`); and what `**Tip: …**` is when the note has no title.
- **Proposed resolution:** the labels of one attribute's values are joined with ` / `, and several attributes with `, ` (`Quill Cloud / self-managed, npm`); a labeled arm's label is its title. A note with no title is `**Tip**` (the type's label alone). Sections are separated by blank lines, and an arm's content follows its label. Implemented now: exactly that (`SPEC-QUESTION(Q115)` in `labels.rs` and `plain/mod.rs`).
- **Affects:** `crates/tessera-emit/src/labels.rs`, `plain/mod.rs`.
- **Resolution:** approved by the repository owner: as proposed: a set's labels joined with ` / `, several attributes with `, `, and a note with no title is `**Tip**`. SPEC §9.4 now says so.

### Q116: What plain markdown loses: image attributes and table alignment

- **Section:** SPEC §9.4, §5.3
- **Raised by:** phase 18
- **Status:** resolved (2026-09-28)
- **Ambiguity:** the plain output is CommonMark. CommonMark has no place for an image's attribute block (`{width=600}`), and tables are GitHub-flavored markdown, not CommonMark. The syntax tree doesn't keep a table's column alignment (the delimiter row isn't a node).
- **Proposed resolution:** image attributes aren't in the plain output (the JSON output has them, as `attributes`). Tables are written as GFM tables, which every markdown reader for language models handles, with every column left-aligned by default (`---`); an unmodified CommonMark parser reads one as a paragraph of text. Keeping alignment needs the parser to record it. Implemented now: exactly that (`SPEC-QUESTION(Q116)`).
- **Affects:** `crates/tessera-emit/src/plain/`; `tessera-syntax` if alignment should be kept.
- **Resolution:** approved by the repository owner: image attributes aren't part of plain markdown, as proposed; but table alignment is kept, changed from the proposal: `tessera-syntax`'s `Table` now records each column's `Alignment`, the plain output writes it in the delimiter row, and the JSON output has an `alignments` field. SPEC §9.4 now says so.

### Q117: A file where a directory goes, and the reverse, in a replaced output

- **Section:** output-layout contract §4 (steps 4 and 6)
- **Raised by:** phase 18
- **Status:** resolved (2026-09-28)
- **Ambiguity:** step 4 fails the build when "a file would be written where a directory exists, or a directory is needed where a file exists", to avoid destroying what Tessera doesn't own. It doesn't say what happens when the thing in the way is Tessera's own: the previous output wrote a *file* `a`, and this build needs a directory `a/` (a page or asset `a/b.md`); or the previous output had the directory `a/` (with files Tessera wrote) and this build writes a file `a`.
- **Proposed resolution:** a file the previous manifest lists, in the way of a directory, is removed just before the directory is made (it's Tessera's, and the manifest already lists it, so ownership holds at every moment). A directory in the way of a file fails the build, even when every file in it is Tessera's: making that work means deleting a tree, and the contract says a directory is removed only when a build empties it. A build can then be retried after removing the directory. Implemented now: exactly that (`SPEC-QUESTION(Q117)` in `store.rs`).
- **Affects:** `crates/tessera-emit/src/store.rs`; output-layout.md if it should say so.
- **Resolution:** approved by the repository owner: as proposed: a Tessera-owned file in the way of a directory is removed first; a directory in the way of a file fails the build. `project-docs/contracts/output-layout.md` §4 now says so.

### Q118: The JSON output's shape

- **Section:** SPEC §9.4; phase 18, task 3
- **Raised by:** phase 18
- **Status:** resolved (2026-09-28)
- **Ambiguity:** SPEC §9.4 says only "the resolved tree, for custom consumers", and the phase asks for a documented, versioned schema with each node's source file and span. Every choice of shape is open.
- **Proposed resolution:** one document per page (`<path>.json`) with `schemaVersion` (`1`), the page's `path`, `route`, `title`, `frontmatter`, page `availability`, `headings`, `assets`, and `blocks`. Every block has a `type`, its `source` (file, byte `span`, first and last `lines`, and the includes it came through as `via`), and where it applies its effective `availability` below page level, its `links`, `glossary` uses, and phrase `substitutions`; inline nodes have a `span` in the block's file. Links carry their resolved `target`, with routes root-relative as the router made them (and the site origin as the page's `site`), and asset targets with the copy's path and the reference written for that page. Spans are UTF-8 byte offsets from the start of the file, and lines count from 1. The schema is in `crates/tessera-emit/README.md`. A field added later doesn't change the version; removing one or changing its meaning does.
- **Affects:** `crates/tessera-emit/src/json.rs` and README; phase 20 if the site output shares any of it; consumers.
- **Resolution:** approved by the repository owner: as proposed: the JSON shape documented in `crates/tessera-emit/README.md`, `schemaVersion` 1, now with each table's `alignments` (an additive field).

### Q119: `tessera build`: several builds, the site output, and routes

- **Section:** SPEC §8.2, §9.1; phase 18, task 6
- **Raised by:** phase 18
- **Status:** resolved (2026-09-28)
- **Ambiguity:** (a) "A build MUST fail on errors", and `tessera build` builds every build by default, but the spec doesn't say whether one build's errors stop the others, or how a diagnostic that appears in every build is shown. (b) `--emit site` names an output phase 20 builds. (c) Phase 12's resolution needs a router; phase 20 supplies Astro's.
- **Proposed resolution:** (a) The checks run for every selected build, before anything is written, and their diagnostics are shown once each (a file-level diagnostic isn't repeated per build), in the same format as `tessera check`, on standard output, so the report of `tessera check` and `tessera build` is identical. If any build has errors, nothing is written for any build (exit code 1). Progress and warnings go to standard error. (b) `--emit site` is refused with exit code 2 until phase 20, and the default is `plain,json`. (c) `DefaultRouter::from_consumer` until phase 20; the plain output's links then come from the `[consumer]` base path and trailing-slash policy, which is what the `astro` profile does by default. A link the resolution couldn't resolve (an error the checks report) is written as its text if it gets to an emitter anyway. Implemented now: exactly that (`SPEC-QUESTION(Q119)` in `commands/build.rs`).
- **Affects:** `crates/tessera-cli/src/commands/build.rs`; phase 14 (`check_project`, whose per-build diagnostics this merges), phase 20.
- **Resolution:** approved by the repository owner: as proposed: checks run for every selected build before any output is written, reported once as `tessera check` reports them (one shared `diagnose`); `--emit site` is refused until phase 20; routes come from `DefaultRouter` until then.

### Q120: The missing-site-origin warning has no registry entry

- **Section:** content-model.md decision 10; SPEC §8.2
- **Raised by:** phase 18
- **Status:** resolved (2026-09-28)
- **Ambiguity:** without `[consumer] site`, plain-markdown links are root-relative "and `tessera build` warns". SPEC §8.2's table and the diagnostics registry have no row for this, and it isn't about a source location. Registry entries need a code, and the checks are the same across the command line and the editor.
- **Proposed resolution:** the warning is the build's own message on standard error (`warning: [consumer] site isn't set …`), printed once per build run when the plain output is written, and not a diagnostic (it has no location in a source file, doesn't change the exit code, and isn't part of `tessera check`). If it should be a registry warning (`model-consumer-site-missing`, at the `[consumer]` table, reported only by `tessera build --emit plain`), that's the human's to approve. Implemented now: the message (`SPEC-QUESTION(Q120)` in `plain/mod.rs`).
- **Affects:** `crates/tessera-emit/src/plain/mod.rs`; the registry if the human prefers a diagnostic.
- **Resolution:** approved by the repository owner: as proposed: the missing `[consumer] site` warning is a build message, not a registry diagnostic, since it's about the configuration of an output, not the source.

### Q121: What the TextMate grammar highlights without the content model

- **Section:** SPEC §3.1, §3.2, §3.9, §5.1
- **Raised by:** phase 17
- **Status:** resolved (2026-09-29)
- **Ambiguity:** §3.2 makes a line a directive only if its name is a known keyword: a built-in or a project widget the content model declares. A TextMate grammar can't read the content model, so it must decide from the spelling. §3.9 and Q1 allow up to three extra spaces of indentation beyond the container's, but TextMate can't see a list item's content column or a quote's marker width, so "extra" is unknowable there. A directive's text primary continues onto later lines (§3.4), and TextMate can't see past a line.
- **Options:** (a) highlight only the built-in names and leave every widget to the server; (b) also highlight any hyphenated name (§3.2: widget names always contain a hyphen), so `@api-endpoint` is colored at once, and let the server's `tesseraWidget` tokens confirm it, at the cost that an undeclared `@my-thing` at line start is colored until the server answers (the server can't remove a TextMate color, only add tokens); (c) accept any indentation everywhere, at the cost of coloring indented code.
- **Proposed resolution:** (b), plus this for indentation: at top level, up to three spaces (four spaces or a tab is indented code and is left alone; the guard rule claims such a line because the markdown grammar's own rule starts at column 0, where an injection wins a tie); inside a list or blockquote, any indentation (TextMate can't tell what the container consumed). Unknown built-in-shaped names (`@warning:`) are not colored. Every `{key}` is colored as a candidate phrase, whether or not declared (`tesseraPhraseUndeclared` marks the undeclared ones once the server answers), except `\{key}`; phrases inside link destinations aren't colored by TextMate, because the markdown grammar tokenizes the destination in a capture the injection doesn't reach. Only the first line of a text primary is colored. Title lines (§3.7) are left to the server. Implemented now: exactly that (`packages/vscode/syntaxes/`, tested in `test/unit/grammar.test.ts` against VS Code's own markdown grammar).
- **Affects:** `packages/vscode/syntaxes/*.tmLanguage.json`.
- **Resolution:** approved by the repository owner: as proposed, option (b): the grammar colors built-in names and any hyphenated (widget-shaped) name; up to three spaces of indentation at top level (four or a tab is indented code), any inside lists and quotes; every `{key}` outside code and link destinations as a candidate phrase; only a text primary's first line; title lines left to the server's semantic tokens. Editor-extension behavior, not language: no SPEC change.

### Q122: A `tessera.path` that doesn't work

- **Section:** phase 17, task 2
- **Raised by:** phase 17
- **Status:** resolved (2026-09-29)
- **Ambiguity:** the binary is found "in order: the `tessera.path` setting if set; the project's `node_modules/.bin/tessera`; the bundled binary". It doesn't say whether a `tessera.path` that names no working binary falls through to the next.
- **Options:** (a) fall through, so the extension still works; (b) fail with an error that names the setting.
- **Proposed resolution:** (b). The author asked for that binary; using another silently would hide a typo and run a different version from the one they meant. A project or bundled candidate that doesn't run (missing, not executable, no version in `--version`) is skipped and listed in the output when none works. Implemented now: (b) (`SPEC-QUESTION(Q122)` in `src/binary.ts`).
- **Affects:** `packages/vscode/src/binary.ts`.
- **Resolution:** approved by the repository owner: as proposed, option (b): a `tessera.path` that doesn't name a working binary is an error that names the setting, and resolution doesn't fall through. A project or bundled candidate that doesn't run is skipped and listed. Editor-extension behavior, not language: no SPEC change.

### Q123: What counts as a crash, and when the count resets

- **Section:** phase 17, task 3
- **Raised by:** phase 17
- **Status:** resolved (2026-09-29)
- **Ambiguity:** "After a configured number of crashes, stop and explain." It doesn't say what the count covers (a session, a time window, since the last successful start), what the default is, or whether the limit is the crash that stops restarts.
- **Options:** (a) count all crashes in the window's lifetime; (b) count crashes in a sliding time window (`vscode-languageclient`'s own default is five in three minutes); (c) count since the last manual restart.
- **Proposed resolution:** (c), with the setting `tessera.maxCrashes` (default 5, at least 1): the server is restarted after crashes 1 to `max - 1`, and the crash number `max` stops it, with a message that offers **Show Output** and **Restart Server**. Restarting by hand (the command, or a `tessera.path` change) resets the count. A time window would restart a server that dies once an hour forever, which is a bug worth reporting; a lifetime count is the simplest to explain. Implemented now: (c) (`SPEC-QUESTION(Q123)` in `src/crash.ts`).
- **Affects:** `packages/vscode/src/crash.ts`, `client.ts`, the `tessera.maxCrashes` setting.
- **Resolution:** approved by the repository owner: as proposed, option (c): crashes count since the last manual restart (the command, or a `tessera.path` change); `tessera.maxCrashes` (default 5, at least 1) is the crash that stops restarts, with a message offering Show Output and Restart Server. Editor-extension behavior, not language: no SPEC change.

### Q124: Who registers file watchers

- **Section:** phase 17, task 3; phase 15, task 3
- **Raised by:** phase 17
- **Status:** resolved (2026-09-29)
- **Ambiguity:** phase 17 says to "forward file-watching registrations (phase 15 relies on them for files that aren't open)". Phase 15 registers `workspace/didChangeWatchedFiles` dynamically after `initialized`. The client could also watch `**/*.md` and `**/tessera.toml` itself (`synchronize.fileEvents`), but a server that registers as well would then receive every event twice.
- **Options:** (a) the client only forwards what the server registers; (b) the client also watches statically.
- **Proposed resolution:** (a). `vscode-languageclient` implements dynamic registration of `didChangeWatchedFiles` on its own, so the extension adds nothing, and what is watched (the content root, assets, `tessera.toml`) stays the server's decision. The integration tests cover it: a file created, changed, and deleted on disk while not open produces updated diagnostics. Implemented now: (a).
- **Affects:** `packages/vscode/src/client.ts`; phase 15 must keep registering the watchers.
- **Resolution:** approved by the repository owner: as proposed, option (a): the client only forwards the server's dynamic `workspace/didChangeWatchedFiles` registration, so what is watched stays the server's decision (phase 15, Q135) and no event arrives twice. Editor-extension behavior, not language: no SPEC change.

### Q125: The version the extension expects

- **Section:** phase 17, task 2; PLAN.md, Packaging
- **Raised by:** phase 17
- **Status:** resolved (2026-09-29)
- **Ambiguity:** "warn when the project's binary is older than the version the extension expects" doesn't say where that version is written down, or what it is before the first release. The compiler is `0.0.0` (`Cargo.toml`).
- **Proposed resolution:** it's `tessera.minServerVersion` in `packages/vscode/package.json`, `0.0.0` for now (so nothing warns yet), and phase 27 sets it to the release it ships with. The warning applies to a binary from the setting or the project, not to the bundled one (which is the extension's own). A pre-release sorts before its release. Implemented now: exactly that (`SPEC-QUESTION(Q125)` in `src/client.ts`).
- **Affects:** `packages/vscode/package.json`; phase 27.
- **Resolution:** approved by the repository owner: as proposed: the expected version is `tessera.minServerVersion` in `packages/vscode/package.json`, `0.0.0` until phase 27 sets it to the release it ships with; the warning applies to a binary from the setting or the project, not the bundled one. Editor-extension behavior, not language: no SPEC change.

### Q126: Extension files outside `packages/vscode`

- **Section:** phases/README.md, Ownership
- **Raised by:** phase 17
- **Status:** resolved (2026-09-29)
- **Ambiguity:** the extension's tests and install need three things in files other phases own: the pnpm 11 build-script policy for `esbuild` (`allowBuilds` in `pnpm-workspace.yaml`; without a decision `pnpm install` exits 1), a `vscode` job in `.github/workflows/js.yml` (integration tests need Rust, a display, and a VS Code download), and `test/fixtures/markdown.tmLanguage.json`, a copy of VS Code's MIT-licensed markdown grammar.
- **Proposed resolution:** `allowBuilds: esbuild: false` (esbuild ships its binary in a per-platform package; its postinstall only swaps in a faster launcher); the workflow stays manual-only (`workflow_dispatch`); the grammar is a test fixture, reformatted by Prettier only, with its MIT license notice beside it (`test/fixtures/markdown.tmLanguage.LICENSE.txt`). Implemented now: exactly that.
- **Affects:** `pnpm-workspace.yaml`, `.github/workflows/js.yml`.
- **Resolution:** approved by the repository owner: as proposed: `allowBuilds: esbuild: false` in `pnpm-workspace.yaml`; a `vscode` job in `.github/workflows/js.yml`, manual-only (`workflow_dispatch`) like the rest of CI; VS Code's MIT-licensed markdown grammar as a test fixture, with its license beside it. Repository layout, not language: no SPEC change.

### Q131: A `tessera.toml` that doesn't load, in a running editor

- **Section:** SPEC §8.2 (content model rows), §10
- **Raised by:** phase 15
- **Status:** resolved (2026-09-29)
- **Ambiguity:** `tessera check` stops on a model with errors (exit code 2, Q58). An editor session can't stop: the author is typing the model, and it's invalid at most keystrokes. The spec doesn't say what the editor reports for the rest of the project meanwhile.
- **Options:** (1) Keep the last model that loaded, keep checking the sources with it, and report the model's problems on `tessera.toml`. (2) Stop reporting anything for the sources until the model loads. (3) Report the sources with an empty model (every widget and phrase unknown), which floods the project with false positives.
- **Proposed resolution:** option 1, implemented: the model's own problems (with its warnings, as `tessera check` lists them) are published on `tessera.toml`; every other file's diagnostics stay those of the last model that loaded, and update again as soon as one loads. A project whose model has never loaded has no source diagnostics; it loads when the model does.
- **Affects:** `crates/tessera-lsp/src/core.rs` (`sync_model`), the README.
- **Resolution:** approved by the repository owner: as proposed: a `tessera.toml` that stops loading keeps the last model that loaded in use, and the model's problems are reported as diagnostics on `tessera.toml`. Language-server behavior, not language: no SPEC change.

### Q132: More than one project in a workspace

- **Section:** SPEC §10; PLAN.md, Editor integration
- **Raised by:** phase 15
- **Status:** resolved (2026-09-29)
- **Ambiguity:** the phase says to find "the project's `tessera.toml` from the workspace folders". A workspace can have several folders, and a folder can hold several projects (a monorepo).
- **Options:** (1) One server serves one project: the first `tessera.toml` found, looking at each folder and its parents in order, then a few levels below each folder. (2) One project per workspace folder. (3) One project per `tessera.toml` found.
- **Proposed resolution:** option 1, implemented (the chosen path is logged to standard error when there are several folders); the extension can start one server per folder for options 2 and 3, which needs no server change. Folders added later (`workspace/didChangeWorkspaceFolders`) aren't followed.
- **Affects:** `crates/tessera-lsp/src/core.rs` (`find_config`); phase 17.
- **Resolution:** approved by the repository owner: as proposed, for now: one server serves the first project it finds in the workspace. Serving several projects can come later. Language-server behavior, not language: no SPEC change.

### Q133: A source file that becomes unreadable while the server runs

- **Section:** SPEC §8.2 (Files: a source file that can't be read, or isn't valid UTF-8)
- **Raised by:** phase 15
- **Status:** resolved (2026-09-29)
- **Ambiguity:** at load, an unreadable source is reported as `source-unreadable` and the rest is checked (Q52). `tessera_resolve::IncrementalProject` has no change for "this file exists but can't be read": `Change::Edited` needs text, and a file that stops being valid UTF-8 (or is created that way) can't give it.
- **Options:** (1) Treat it as deleted until it's readable again, and log it. (2) Add a change to phase 13's API that records an unreadable source, so the editor reports `source-unreadable` as `tessera check` does. (3) Keep the last readable text.
- **Proposed resolution:** option 2 is right in the end (the editor and the command line then agree), and needs an additive `Change` in `tessera-resolve`; option 1 is implemented until then: a file that becomes unreadable is treated as deleted (its diagnostics are cleared and links to it break), and logged to standard error. A file that is unreadable when the project loads is reported as `tessera check` does, which the parity test covers.
- **Affects:** `crates/tessera-lsp/src/core.rs` (`collect_present`); `tessera-resolve`'s `Change`.
- **Resolution:** approved by the repository owner: option 1, as an interim only: a source that becomes unreadable while the server runs is treated as deleted. **Follow-up (not yet scheduled):** add an additive "unreadable" `Change` to `tessera_resolve::IncrementalProject::apply` (option 2), so the editor reports `source-unreadable` on the file, as `tessera check` does (Q52), instead of clearing its diagnostics and breaking the links to it. Language-server behavior, not language: no SPEC change.

### Q134: What the editor reports for content no build publishes

- **Section:** SPEC §8.1, §10
- **Raised by:** phase 15
- **Status:** resolved (2026-09-29)
- **Ambiguity:** §8.1 checks content that no build publishes, as if one build kept everything, and reports its problems as belonging to no build (Q101). §10 wants the diagnostics of §8 as the author types, and the content model has one `[editor] build`. `check_project` for that build doesn't run the pass over unpublished content (`check_all_builds` does).
- **Options:** (1) The editor reports the editor build's diagnostics only, as `tessera check --build <editor build>` does. (2) It also reports the unpublished-content problems, as plain `tessera check` does. (3) It reports every build's diagnostics.
- **Proposed resolution:** option 1, implemented, because the phase says to compute the editor's build with `check_project` and the parity test compares with `tessera check --build <name>`. A problem that only shows in another build, or in content no build publishes, appears in `tessera check` and `tessera build`, not in the editor. Option 2 is a one-line change (`check_all_builds` for the editor build's pages plus the unpublished pass) if the human wants it.
- **Affects:** `crates/tessera-lsp/src/compute.rs`; the parity test.
- **Resolution:** approved by the repository owner: as proposed: the editor reports its own build (`[editor] build`) only, not content that no build publishes; `tessera check` covers that. Language-server behavior, not language: no SPEC change.

### Q135: Which files the server follows

- **Section:** SPEC §2.1, §10; asset contract §1
- **Raised by:** phase 15
- **Status:** resolved (2026-09-29)
- **Ambiguity:** the editor and the file watcher report every file, not only the project's. The spec says which files are sources (Q52) and which files a reference can name (asset contract, boundary), not which events a server should act on.
- **Options:** act on every event under the workspace; or only on files that can matter: sources, and files a reference can probe (inside the project root or content root, outside the output directory).
- **Proposed resolution:** the second, implemented: a document that isn't a `file:` URI, isn't `tessera.toml`, and isn't a source or under the project root is ignored (no tokens, no diagnostics); events under `.git`, `.hg`, and `node_modules` are ignored; other files under the project root are assets, whose appearing and disappearing re-checks the references to them. Assets outside the project root and content root can't be named by a reference, so they aren't watched. The watcher is registered as `**/*` and filtered by the server.
- **Affects:** `crates/tessera-lsp/src/core.rs` (`classify`, `IGNORED_DIRS`).
- **Resolution:** approved by the repository owner: as proposed: the server follows `file:` documents that are `tessera.toml`, sources, or under the project root; it ignores events under `.git`, `.hg`, and `node_modules`; other files under the project root are assets. Language-server behavior, not language: no SPEC change.

### Q136: Diagnostics of files that aren't open

- **Section:** SPEC §10; phase 15 task 4
- **Raised by:** phase 15
- **Status:** resolved (2026-09-29)
- **Ambiguity:** the phase says diagnostics are published for every affected file, including ones that aren't open, and cleared for deleted files. It doesn't say what closing a file does to its diagnostics, or whether files nobody has opened get any at load.
- **Options:** (1) Diagnostics are the project's: published for every file at load and kept when a file closes, so the Problems panel shows the whole project. (2) Only open files and files an open file's change affects have them; closing clears. 
- **Proposed resolution:** option 1, implemented: at load every file with diagnostics gets a publication (a file with none gets nothing), a closed file keeps its diagnostics, and a publication is skipped when nothing changed since the last one for that file and version. A large project with thousands of problems publishes them all at startup; an extension setting could limit that to open files, which needs no server change to the checks.
- **Affects:** `crates/tessera-lsp/src/core.rs` (`publish`, `did_close`).
- **Resolution:** approved by the repository owner: as proposed: at load every file with diagnostics gets a publication, a closed file keeps its diagnostics, and a publication is skipped when nothing changed for that file and version. Language-server behavior, not language: no SPEC change.

### Q137: The page-level checks are whole-project on every keystroke

- **Section:** PLAN.md, Performance targets ("language server responses on a keystroke within about 50 ms"); phase 15 acceptance criteria
- **Raised by:** phase 15
- **Status:** resolved (2026-09-29) (implemented in phase 15, awaiting resolution)
- **Ambiguity:** not a spec question, an architecture one that needs a human's decision because it crosses a phase's ownership. `tessera_check::PageChecker::new` indexes the whole project from source texts, and `check` resolves every page of the build, so page-level diagnostics cost O(project) per keystroke. The file-level checks are incremental already (`check_file` for the files in `Affected::recheck`). Phase 15's brief keeps changes to `tessera-check` to the `FileSystem` constructor, so the server can't hand the page checks the incremental index it already has.
- **Options:** (1) Leave it: the keystroke latency is under 50 ms up to about 300 pages and grows linearly after (`cargo bench -p tessera-lsp --bench keystroke`; the numbers are in the handoff notes). (2) An additive change in `tessera-check`'s `page/` module (phase 14's): `PageChecker` built from a `tessera_resolve::Snapshot`'s index, and a check of only the pages in `Affected::re_resolve`, with `ResolvedCache` supplying resolved pages. Every diagnostic and its place stay as they are; the parity and differential tests hold it to that.
- **Proposed resolution:** option 2.
- **Implemented now:** option 2, approved by the repository owner in the review of the phase 15 pull request. `tessera_check::PageChecker::with_index` (a checker over a caller's index) and `PageChecker::check_resolved` (the diagnostics of given resolved pages) are additive; `check_project`, `check_builds`, `check_all_builds`, `check_pages`, and `PageChecker::new`/`check` are unchanged. The server checks the pages that can have a diagnostic located in a file of the round, over its snapshot's index and a `ResolvedCache`. Keystroke latency at 3,000 pages: about 3 ms for a page, 9 ms for a fragment with 30 includers (`cargo bench -p tessera-lsp --bench keystroke`).
- **Affects:** `crates/tessera-check/src/page/` (`bridge.rs`, `mod.rs`); `crates/tessera-lsp/src/compute.rs`; phase 26.
- **Resolution:** approved by the repository owner: implemented in phase 15 after review: `PageChecker::with_index` over the snapshot's index checks only the pages a round can affect, with a `ResolvedCache` updated from each `Affected`. A keystroke publishes in about 3 ms (page) and 9 ms (fragment) at 3,000 pages, from 501 ms before. `check_project`, `check_builds`, and `check_all_builds` are unchanged. Language-server behavior, not language: no SPEC change.

### Q141: Image attribute defaults in the site output

- **Section:** site-render contract §4; content-model.md §14; SPEC §5.3
- **Raised by:** phase 20
- **Status:** resolved (2026-09-29)
- **Ambiguity:** the contract says an image's marker holds "the image's attributes in canonical order", and "an image without attributes has no marker". content-model.md §14 lets `[images.attributes]` declare defaults (`loading = { type = "enum(lazy, eager)", default = "lazy" }`). It doesn't say whether a default reaches the `<img>` of an image that doesn't write the attribute. The element contract says widgets get their declared defaults; images aren't mentioned.
- **Options:** (1) Written attributes only: a default is validation help and never appears in output. (2) Written attributes and declared defaults: every image carries what the model declares, so `loading="lazy"` applies to every image.
- **Proposed resolution:** option 2. A default the output ignores would make declaring it pointless, and it's the rule for widgets. So an image gets a marker holding, in declaration order, every declared attribute that has a value or a default, and then any attribute written that the model doesn't declare (already an error the checks report). An image with none has no marker. Implemented now: exactly that (`SPEC-QUESTION(Q141)` in `site/inline.rs`). Quill declares no defaults, so its output doesn't change.
- **Affects:** `crates/tessera-emit/src/site/inline.rs`; site-render.md §4's wording ("An image without attributes" would read "with neither attributes nor defaults"); phase 21's plugin needs nothing, since it applies whatever the marker holds.
- **Resolution:** approved by the repository owner: as proposed, option 2: a declared image attribute's default reaches every image's marker, as widgets' defaults do. Stated in the site-render contract §4 and content-model.md §14.

### Q142: How page-level `available` reaches the layout

- **Section:** SPEC §9.4, §9.6; element contract §4
- **Raised by:** phase 20
- **Status:** resolved (2026-09-29)
- **Ambiguity:** SPEC §9.4 says page-level availability is "passed through as frontmatter", and the element contract leaves "a form the layout can read" to phase 20. The source's `available` is a spec string (`cloud, self-managed preview 3.3`). A layout that shows it needs each target's `dimension`, states, versions, and display text, which the layout can't compute without the content model, and the generated Zod schema must describe whatever is written.
- **Options:** (1) Pass the string through, and let the layout parse it. (2) Write a list of targets with the attributes of a `<tessera-availability-target>` and its text.
- **Proposed resolution:** option 2, with the spec resolved (a feature key replaced by the spec it stands for, Q25). Each entry is `{ target, dimension, states: [...], versions: [...] (left out with none), text }`, in the spec's order, so a layout writes `<tessera-availability scope="page">` with one `<tessera-availability-target target dimension states versions>text</…>` per entry, with no model knowledge. The Zod schema's `available` is that list (`availableSchema`), not a string. A page with no `available` has no key. `variant` passes through as written. Implemented now: exactly that (`SPEC-QUESTION(Q142)` in `site/frontmatter.rs`).
- **Affects:** `crates/tessera-emit/src/site/frontmatter.rs` and `zod/`; phase 21's collection configuration (the schema is the site output's, not the source's); the element contract's §4 could name this form.
- **Resolution:** approved by the repository owner: as proposed, option 2: page-level `available` in the site output is a list of targets (`target`, `dimension`, `states`, `versions` when given, `text`), with feature keys resolved, and the Zod schema describes that list. Stated in SPEC §9.6 and the element contract §4.

### Q143: Two pages with the same route

- **Section:** SPEC §9.5; content-model.md §16; phase 02's handoff ("Route collisions have no diagnostic")
- **Raised by:** phase 20
- **Status:** resolved (2026-09-29)
- **Ambiguity:** Astro's `glob` loader slugs each path segment, so `My File.md` and `my-file.md` have the same entry id and the same route, and `index.md` and `index/index.md` both have the id `index`, which Tessera routes to the base path. Neither the spec nor the registry says what a build does.
- **Options:** (1) Publish both and let Astro pick (its loader warns of a duplicate id and keeps one). (2) Fail the build with a message. (3) A registry diagnostic at page level.
- **Proposed resolution:** option 2 for now: the site emitter refuses the build ("the site output can't publish two pages at one route: /docs/my-file is the route of My File.md and my-file.md. Rename one of the files"), before writing anything, and the plain and JSON outputs are unaffected. It isn't a registry diagnostic because it's a property of an output (the routes are the consumer's), like Q120; if it should be one (`page-route-duplicate`, reported by `tessera check` too), that's the human's call. Implemented now: the emitter error (`SPEC-QUESTION(Q143)` in `site/mod.rs`).
- **Affects:** `crates/tessera-emit/src/site/mod.rs`; the registry if a diagnostic is preferred.
- **Resolution:** approved by the repository owner: as proposed, option 2: two pages with one Astro entry id fail `tessera build --emit site` before anything is written, with a message naming the route and both files; the plain and JSON outputs are unaffected. An emitter error, not a registry diagnostic, since routes are a property of the consumer profile (as with Q120). Stated in content-model.md §16. If authors need to see it in the editor, a `page-route-duplicate` diagnostic can be added later.

### Q144: Which router `tessera build` uses, and which outputs it builds by default

- **Section:** SPEC §9.5; Q119
- **Raised by:** phase 20
- **Status:** resolved (2026-09-29)
- **Ambiguity:** Q119 used `DefaultRouter` for every output "until phase 20 supplies the `astro` profile's", and defaulted `--emit` to `plain,json`. The plain output's links must be the URLs the site publishes (SPEC §9.4), and the site output is the toolchain's main output.
- **Proposed resolution:** every output is resolved with the `astro` profile's router (`AstroRouter`, in `tessera-resolve`, which also answers "which page has this route"), so a link in the plain or JSON output is the URL the site publishes, including the entry-id slugging (`Guides/My Setup.md` is `/guides/my-setup/`). `DefaultRouter` stays for tests and callers with no profile; `tessera check`'s page-level pass still uses it, because a route's text never changes a diagnostic. `--emit` defaults to `site,plain,json`.
- **Affects:** `crates/tessera-cli/src/commands/build.rs`, `crates/tessera-resolve/src/astro.rs`; `tessera-check`'s page pass could switch to `AstroRouter` too.
- **Resolution:** approved by the repository owner: as proposed: every output of `tessera build` is resolved with the `astro` profile's router, so plain and JSON links are the URLs the site publishes; `tessera check`'s page pass keeps `DefaultRouter`, since a route's text never changes a diagnostic; `--emit` defaults to `site,plain,json`. Tool behavior, not language: no SPEC change.

### Q145: A marker after an image that ends a heading

- **Section:** site-render contract §2.1, §2.2
- **Raised by:** phase 20
- **Status:** resolved (2026-09-29)
- **Ambiguity:** `## ![Icon](./icon.png)<tessera-attributes width="16"></tessera-attributes>`: the marker is the last inline content of the heading (§2.1) and also directly follows an image (§2.2).
- **Proposed resolution:** it applies to the image; the heading gets no id from it. §2.2 names the position exactly, and the emitter never writes this (its heading marker follows a space, so a heading `## Logo ![Logo](./logo.png){width=32}` is `… ![Logo](./logo.png)<marker> <marker>`). A fixture, `tests/render/image-ends-heading`, records it for both implementations. Implemented now: exactly that (`SPEC-QUESTION(Q145)` in `render/mod.rs`).
- **Affects:** `render_site_html`; phase 21's plugin must agree, through the new fixture; site-render.md §2.
- **Resolution:** approved by the repository owner: as proposed: a marker directly after an image that also ends a heading applies to the image, and the heading gets no id from it. Stated in the site-render contract §2.2, with the fixture `image-ends-heading` in §6, which phase 21's plugin must also pass.

### Q146: Markdown in a tight list item that holds an element

- **Section:** element contract §0; SPEC §9.4
- **Raised by:** phase 20
- **Status:** resolved (2026-09-29)
- **Ambiguity:** an HTML block of the kind the elements are can't interrupt a paragraph, so a list item written tight (`- item` then a `@note` on the next line) needs a blank line before the element, and CommonMark then reads the list as loose: its items render `<p>`. The contract doesn't say.
- **Proposed resolution:** accept it. The emitter writes the blank line the element needs, whatever the list's tightness in the source, and a tight list whose items hold no element stays tight. The only difference is spacing of paragraphs in that list.
- **Affects:** `crates/tessera-emit/src/site/blocks.rs`.
- **Resolution:** approved by the repository owner: as proposed: a list whose items hold an element is written loose, because the element needs a blank line before it; a list whose items hold none keeps its tightness. Stated in the element contract §0.

### Q147: Images in a `@details` title

- **Section:** element contract §5; SPEC §9.4
- **Raised by:** phase 20
- **Status:** resolved (2026-09-29)
- **Ambiguity:** the summary is raw HTML (`<summary>Show <code>x</code></summary>`), so an image in the title would be a raw `<img>`, which a consumer doesn't process, and whose relative `src` resolves against the page's URL, not the file. Assets in raw HTML aren't copied or rewritten (asset contract §1).
- **Proposed resolution:** an image in a `@details` title is written as its alt text in the summary. Its file is still copied, since the resolved page lists it. Implemented now (`SPEC-QUESTION(Q147)` in `site/blocks.rs`). If images in summaries matter, the contract could allow a published reference.
- **Affects:** `crates/tessera-emit/src/site/blocks.rs`.
- **Resolution:** approved by the repository owner: as proposed: an image in a `@details` title is written as its alt text in the `<summary>`; its file is still copied. Stated in the element contract §5.

### Q148: Which page a route-like link names

- **Section:** SPEC §5.2; Q22, Q55
- **Raised by:** phase 20
- **Status:** resolved (2026-09-29)
- **Ambiguity:** Q55's interim rule guessed a route's page as `route.md`, else `route/index.md`, on the destination read as a content path. That misses a route the way a published site spells it: with the base path (`/docs/guides/install/`), or with the segments Astro slugs (`/guides/my-setup/` for `Guides/My Setup.md`).
- **Proposed resolution:** keep Q55's mapping first, since it never gets a project wrong that it got right, and when neither candidate is a source file, ask the profile's router which page has the route (`AstroRouter::page_for_route`, through a new `SourceSet::pages`, whose default is empty). A destination that starts with `/` is read as written, with or without the base path; any other is the route of the content path it names. A page that's found makes the `link-route` fix available (its `page_exists`). The change is additive: no destination that resolved before resolves differently, and `tessera check` and the source index share it, so `crates/tessera-check/tests/parity.rs` still passes.
- **Affects:** `crates/tessera-resolve/src/references.rs`, `crates/tessera-check/src/project.rs` (`SourceSet::pages`), `link-route` and its fix.
- **Resolution:** approved by the repository owner: as proposed: Q55's conventional mapping first (`route.md`, else `route/index.md`), then the profile's router names the page whose route it is (with or without the base path, and with Astro's entry-id slugging), for the `link-route` warning and its fix. `tessera check` and the source index share it. Stated in content-model.md §16; SPEC leaves routing to the consumer profile, so no SPEC change.

### Q149: The generated Zod module

- **Section:** SPEC §9.6; content-model.md §5.1, §6
- **Raised by:** phase 20
- **Status:** resolved (2026-09-29)
- **Ambiguity:** SPEC §9.6 says the collection's schema MUST be generated as a Zod schema, and the phase asks for one per content type with the reserved keys. Where the module lives, what it imports, how each field type maps, and what it exports aren't set.
- **Proposed resolution:** `_tessera/schema.ts` in the site emitter root (a `generated` file in the manifest), importing `z` from `astro/zod` (Zod 4 in Astro 7.3, the targeted version). Each type is `z.strictObject` (unknown keys are errors, as in Tessera), with `title` and every field per content-model.md §6: `string`, `number`, `boolean`; `date` as `z.coerce.date()` (Astro's own advice, since YAML readers may return a string or a `Date`); `enum` as `z.enum`; `list` as `z.array`; `object` as a nested strict object; optional fields `.optional()`; defaults `.default(…)`; `description` as `.describe(…)`. `available` is `availableSchema` (Q142) and `variant` is a record of strings or lists of strings, on every type. Exports, per type `<camelName>Schema` and its type, and `schemas` (by name), `contentTypes` (each type's `files` and `default`), and `schema` (a union of every type's, for one collection). `tests/zod/` (a pnpm workspace package) type-checks the generated files under the workspace's strict settings, and validates the Quill pages' frontmatter with them.
- **Affects:** `crates/tessera-emit/src/zod/`, `tests/zod/`; phase 21's collection configuration.
- **Resolution:** approved by the repository owner: as proposed: `_tessera/schema.ts` in the site output, importing `z` from `astro/zod` (Zod 4, Astro 7.3); one `z.strictObject` per content type with `title`, its fields mapped per content-model.md §6, and the reserved `available` and `variant`; exports per type, plus `schemas`, `contentTypes`, and `schema`. `tests/zod/` type-checks it and validates Quill's site frontmatter. Output format, not language: no SPEC change.

### Q150: A frontmatter `slug` under the `astro` profile

- **Section:** content-model.md §5.1, §16; SPEC §9.5
- **Raised by:** phase 20 (review of the pull request)
- **Status:** resolved (2026-09-29)
- **Ambiguity:** Astro's `glob` loader gives a page whose frontmatter has a `slug` that value as its entry id, in place of the id computed from its path (`generateIdDefault`, Astro 7.3.5). A content model can declare a `slug` field today, and a page that sets it would be published at an id Tessera never computed, so every route, link, and `link-route` answer for it would be wrong.
- **Options:** (1) The router honors the frontmatter `slug`, so Tessera's routes depend on frontmatter, and the source index and every route answer need each page's frontmatter. (2) Under the `astro` profile `slug` is reserved: a content type can't declare it.
- **Proposed resolution:** option 2, the simpler. A page that sets `slug` without a declaration is already an unknown key (`frontmatter-unknown-key`), and a declaration is now a model error, `model-field-reserved` with a third message variant that names the profile's rule. A later profile that doesn't have this rule doesn't reserve it. Implemented now: exactly that (`SPEC-QUESTION(Q150)` in `tessera-model/src/sections.rs`), with the message in the registry and content-model.md §20.
- **Affects:** `crates/tessera-model/src/sections.rs`, `tests/conformance/diagnostics.toml`, content-model.md §5.1 and §20.
- **Resolution:** approved by the repository owner: as proposed, option 2: under the `astro` profile, `slug` is a reserved frontmatter key that no content type may declare (`model-field-reserved`, a third message variant), because Astro's loader uses it as the entry id in place of the path. Stated in content-model.md §5.1 and §20, and in the registry.

### Q151: The markdown plugin is a hast plugin, for both of Astro 7.3's markdown processors

- **Section:** site-render contract §3, §5; SPEC §9.5, §9.6
- **Raised by:** phase 21
- **Status:** resolved (2026-09-29)
- **Ambiguity:** Phase 20 wrote the site output assuming a remark plugin setting `hProperties`, and `tests/render/README.md` says to run the plugin in "a unified pipeline". Astro 7.3.5 (the targeted version) doesn't run remark by default. Its default `markdown.processor` is **Sätteri** (`@astrojs/markdown-satteri`), a Rust parser with its own mdast and hast plugin lists; the remark/rehype pipeline is the `unified()` processor of `@astrojs/markdown-remark`, which is no longer installed with Astro. So "a remark plugin" would work only for a site that installs `@astrojs/markdown-remark` and switches its processor. Where the plugin sits, and what it is, is not settled by the contract.
- **Options:** (1) A remark plugin only, and the integration switches the site to `unified()`. (2) A plugin for Astro's default processor only. (3) One rule set on the hast tree, with an adapter for each processor.
- **Proposed resolution:** option 3. In both processors the markers reach a user hast plugin as two adjacent `raw` nodes (an open tag and a close tag), after markdown has become hast and before Astro's own image and heading-id passes (unified: user rehype plugins run before `rehypeImages` and `rehypeHeadingIds`; Sätteri: user hast plugins run before its image marker and heading ids). So `packages/astro/src/attributes.ts` finds the edits once (a pure function over a hast tree), `rehype.ts` applies them to a unified tree, and `satteri.ts` queues them as Sätteri commands. The integration adds the right one to whichever processor the site uses (`processor.options.hastPlugins` or `.rehypePlugins`, which Astro documents as the way integrations extend a processor), and fails the build for any other processor. The rules are the contract's, unchanged; `tests/render/` runs against both pipelines. Verified in real builds of `examples/astro-site` under each processor: an explicit id survives Astro's heading-id pass and reaches its table of contents (`weave-config`, where Astro's own slug would be `weave-configuration`), and an image's `width` reaches Astro's image processing (`width="300" height="188"`, a hashed `.webp`). Contract wording only: the rationale in §1 says "user remark plugins" and `tests/render/README.md` says "a unified pipeline"; both could name the hast stage instead. No behavior changes.
- **Affects:** `packages/astro/src/{attributes,rehype,satteri}.ts`; site-render.md §1 (wording), `tests/render/README.md` (wording); phase 22 (the published package's peer dependencies: none on `@astrojs/markdown-remark`).
- **Resolution:** approved by the repository owner: as proposed, option 3: the site-render contract's rules are applied once on the hast tree, where a marker is two adjacent `raw` nodes, with an adapter for each of Astro 7.3's markdown processors (Sätteri's `hastPlugins`, `unified()`'s `rehypePlugins`); the integration adds the right one and fails the build for any other processor. No behavior changes. The contract's wording now names the processors and the hast stage: site-render.md §1 ("Why a marker") and `tests/render/README.md` ("Running a fixture").

### Q152: How the integration finds the `tessera` binary

- **Section:** SPEC §9.6; phases 21 and 22
- **Raised by:** phase 21
- **Status:** resolved (2026-09-29)
- **Ambiguity:** Phase 21 runs `tessera build --emit site` from a locally built binary; npm distribution is phase 22's. How the integration finds it isn't specified.
- **Options:** (1) An integration option only. (2) An environment variable only. (3) The workspace's `target/`. (4) `tessera` on `PATH`.
- **Proposed resolution:** the first three, in that order: the `binary` option (relative to the Astro root), then `TESSERA_BIN`, then `target/release/tessera` or `target/debug/tessera` (the newer) in the project's directory or the nearest parent that has one. Nothing found is an error that says how to point at a binary. `PATH` is left out: a stray older `tessera` there would build the site with the wrong compiler without saying so. Phase 22 replaces the last step with the npm-installed binary and keeps the option and variable as overrides.
- **Affects:** `packages/astro/src/binary.ts`; phase 22.
- **Resolution:** approved by the repository owner: as proposed: the `binary` option, then `TESSERA_BIN`, then the newer of `target/release/tessera` and `target/debug/tessera` in the project's directory or the nearest parent that has one; never `PATH`. Phase 22 replaces the last step with the npm-installed binary and keeps the first two as overrides. Integration behavior, not language: no SPEC change.

### Q153: Loading the element library: a component, not an injected script

- **Section:** SPEC §9.6, §9.7
- **Raised by:** phase 21
- **Status:** resolved (2026-09-29)
- **Ambiguity:** The integration should "load the element library". The obvious way is `injectScript("page", 'import "@tessera/elements"; import "@tessera/elements/style.css"')`. In Astro 7.3.5 that bundles the script but drops the CSS it imports: the built pages have no stylesheet.
- **Options:** (1) Inject the script, and tell sites to import the CSS in their layout. (2) A component, `@tessera/astro/Elements.astro`, that imports the stylesheet in its frontmatter and the script in a `<script>`, which a layout puts in its `<head>`. (3) Emit the CSS as a static file and inject a `<link>`.
- **Proposed resolution:** option 2. Astro links the stylesheet and bundles the script as it does for any component, on exactly the pages whose layout includes it ("pages that render Tessera content", as the phase says), and the site's other pages don't get them. The integration sets `vite.ssr.noExternal` for `@tessera/astro` so an installed copy's `.astro` file is compiled. There is no `elements` option.
- **Affects:** `packages/astro/src/Elements.astro`; phase 22 (the package's `files`).
- **Resolution:** approved by the repository owner: as proposed, option 2: a `<Elements />` component (`@tessera/astro/Elements.astro`) that a layout puts in its `<head>` loads the stylesheet and the script, on the pages whose layout includes it, because `injectScript` drops the imported CSS in Astro 7.3.5. Integration behavior, not language: no SPEC change.

### Q154: `tessera.toml` and `astro.config` must agree on routing, and the integration checks it

- **Section:** content-model.md §16; SPEC §9.5
- **Raised by:** phase 21
- **Status:** resolved (2026-09-29)
- **Ambiguity:** content-model.md §16 says the integration SHOULD check that `site`, `base-path`, and `trailing-slash` agree with `astro.config`. It doesn't say what a disagreement does. Tessera writes every link with its own settings, so a site that routes differently has broken links, and nothing else would notice.
- **Options:** (1) Warn. (2) Fail the build.
- **Proposed resolution:** fail, before running `tessera build`, naming each difference. `base` is compared as a path with leading and trailing `/`; Astro's `trailingSlash: "ignore"` agrees with either value (both forms are served); `site` is compared by origin, and only when both sides set it. The integration reads `tessera.toml` itself (`smol-toml`) for this, and for `[project] output-dir`, which it needs to find the site output.
- **Affects:** `packages/astro/src/project.ts`; content-model.md §16 (states the result).
- **Resolution:** approved by the repository owner: as proposed, option 2: a disagreement between `tessera.toml`'s `[consumer]` and `astro.config` fails the Astro build before `tessera build` runs, naming each difference. Stated in content-model.md §16.

### Q155: The collection helper reads the site root from the integration

- **Section:** SPEC §9.6
- **Raised by:** phase 21
- **Status:** resolved (2026-09-29)
- **Ambiguity:** The integration should "provide the collection configuration and generated schema". The collection is defined in `content.config.ts`, which Astro loads through Vite, before any page exists, and the generated schema is a TypeScript file inside the build output. How does the helper know where the output is, and how does the schema reach it?
- **Options:** (1) The helper finds `tessera.toml` from the working directory. (2) The helper takes the project and build again as arguments. (3) The integration passes the resolved site root through a virtual module.
- **Proposed resolution:** option 3, so the integration's options are the one place the project and build are named (a first version used option 1, and a test that built a copy of the site read the original's output). `tesseraCollection({ schema })` from `@tessera/astro/content` is a `glob` loader (`**/*.md`, not `_tessera/**`) over `virtual:tessera/site`'s `siteRoot`; the site imports `schema` from the generated `_tessera/schema.ts` itself, because Vite must compile that TypeScript file and resolve its `astro/zod` import from the site. The helper is a separate entry from the integration because the integration runs in Node and the helper in Vite.
- **Affects:** `packages/astro/src/content.ts`; `examples/astro-site/src/content.config.ts`; phase 22.
- **Resolution:** approved by the repository owner: as proposed, option 3: the integration passes the resolved site root to `tesseraCollection` through `virtual:tessera/site`, so the integration's options are the one place the project and build are named; the site imports `schema` from the generated `_tessera/schema.ts` by path, to keep its exact inferred types (the trade-off is documented in `packages/astro/README.md`). Integration behavior, not language: no SPEC change.
