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

### Q27: `role` is reserved, but the full example model declares it

- **Section:** SPEC §7.2 (reserved attribute keys); content-model.md §15, §20.3
- **Raised by:** phase 08
- **Status:** open
- **Ambiguity:** `tessera_core::reserved::HTML_GLOBAL_ATTRIBUTES` includes `role` (an ARIA global attribute), so `model-attribute-reserved` rejects it. Phase 01's `examples/content-models/full.toml` declared a widget attribute `role = "set(enum(admin, developer, writer))"` and, as an acceptance criterion, must load with no issues. The two contradict.
- **Options:**
  1. Keep `role` reserved and rename the example's attribute. The site output writes widget attributes onto the widget's custom element, where `role` would change its accessibility role.
  2. Remove `role` from the reserved list. Authors could then declare a `role` attribute whose value (such as `admin`) becomes an invalid ARIA role on the element.
- **Proposed resolution:** option 1. Implemented now: the example's attribute is renamed `audience` (in `full.toml` and its comment); no other file mentions the old spelling.
- **Affects:** `examples/content-models/full.toml`; phase 03 fixtures that copy from it.
- **Resolution:** _to be filled in by a human._

### Q28: `DefaultValue` has no number variant

- **Section:** SPEC §3.3; content-model.md §6.2; `tessera_core::DefaultValue` (phase 02 contract)
- **Raised by:** phase 08
- **Status:** open
- **Ambiguity:** an attribute of type `number` can have a default (`height = { type = "number", default = 600 }`), but `DefaultValue` is `Text`, `Boolean`, or `Set`. There is nowhere to put a number.
- **Options:**
  1. Add `DefaultValue::Number` (a contract change: it touches every `match` on `DefaultValue` in phases 05, 06, 10, and 23).
  2. Store the default's source text (`"600"`) in `DefaultValue::Text`. Consumers of a `number` attribute already read attribute values as text.
- **Proposed resolution:** option 2 for now, since it needs no contract change and loses nothing; option 1 if a consumer needs to tell `"600"` from `600`. Implemented now: option 2 (`// SPEC-QUESTION(Q28)` in `tessera-model/src/fields.rs`).
- **Affects:** `crates/tessera-core/src/schema.rs`; phases 05, 06, 10, 23.
- **Resolution:** _to be filled in by a human._

### Q29: A dimension name as a target of a versioned entry

- **Section:** SPEC §4.4
- **Raised by:** phase 08
- **Status:** open
- **Ambiguity:** a target may be a dimension name, "which stands for all of its values", and "a target that the content model declares as versionless takes a single state and no versions". For a dimension with both kinds of value (`deployment`: `cloud` versionless, `self-managed` versioned), `deployment 3.4` is neither clearly valid nor clearly invalid.
- **Options:**
  1. A dimension name is versionless only if all its values are. `deployment 3.4` is then accepted for a mixed dimension, and the version means nothing to `cloud`.
  2. A dimension name is versionless if any of its values is, so versions are never allowed on a mixed dimension name.
  3. Versions are never allowed on a dimension name.
- **Proposed resolution:** option 1: it rejects only what is certainly wrong (a version on a dimension whose values are all versionless) and never rejects a spec that has a sensible meaning. **Implemented now: option 3**, the most conservative, at the reviewer's request while the question is open (`// SPEC-QUESTION(Q29)` in `tessera-model/src/model.rs`). A version on a dimension name is reported as `model-availability-versionless` (`available-versionless` in documents) with the base message; its wording ("`deployment` is versionless") doesn't quite fit, and a message variant naming the dimension would need a registry change, so none was added.
- **Affects:** `tessera-model` (`check_availability`); phases 10, 12 (which decide what the version means for versionless members).
- **Resolution:** _to be filled in by a human._
