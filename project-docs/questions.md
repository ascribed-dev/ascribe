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
### Q31: Directives that stack in front of a block, and a block that isn't there

- **Section:** SPEC §3.8, §4.4, §4.5
- **Raised by:** phase 06
- **Status:** open
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
- **Resolution:** _open_

### Q32: Which sections a heading-bound directive can be at the top of

- **Section:** SPEC §3.8, §3.9, §8.2
- **Raised by:** phase 06
- **Status:** open
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
- **Resolution:** _open_

### Q33: Whether containers in list items and block quotes count toward nesting depth

- **Section:** SPEC §3.10, §8.2
- **Raised by:** phase 06
- **Status:** open
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
- **Resolution:** _open_

### Q34: Exactly what triggers the three list warnings

- **Section:** SPEC §3.9, §4.6, §8.2 ("Lists")
- **Raised by:** phase 06
- **Status:** open
- **Ambiguity:** the three "Lists" rows say when they apply in a sentence each, which leaves several edges open:
  - `list-ended-by-directive`: "an unindented directive line ends a list". Does a directive after a blank line count (the blank line already ends the list in CommonMark), an `@end` line (it often ends a list that sits in a container, correctly), or a block quote (rule 6 says quotes work the same way)?
  - `directive-indented-code`: is it every directive-shaped line in an indented code block, or the first?
  - `steps-numbering-continued`: how much may sit between the two lists (only directive lines, or blocks too), and must there be at least one?
- **Options:** any combination of the above.
- **Proposed resolution:** the narrowest triggers that catch the cases the spec describes. `list-ended-by-directive`: a directive line (including a container or group opener) directly after a list with no blank line, in the same list of blocks; not an end line, and not after a block quote. `directive-indented-code`: every directive-shaped line (`@`, a known keyword, then a space, tab, `{`, `:`, or the end) of an indented code block, at the `@name`. `steps-numbering-continued`: an ordered list whose start number is the `@steps` list's start plus its item count, after one or more line-form directive lines and nothing else. Implemented now: these (`// SPEC-QUESTION(Q34)` in `structure/lists.rs`).
- **Affects:** `crates/tessera-syntax/src/structure/lists.rs`; phases 10 and 23.
- **Resolution:** _open_

### Q35: A text primary followed by an ordered list that doesn't start at 1

- **Section:** SPEC §3.4, §8.2
- **Raised by:** phase 06
- **Status:** open
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
- **Resolution:** _open_

### Q36: What else is reported about a directive whose attribute block never closes

- **Section:** SPEC §3.3, §3.5, §3.8, §8.2
- **Raised by:** phase 06
- **Status:** open
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
- **Resolution:** _open_
