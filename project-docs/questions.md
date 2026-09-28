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

### Q3: §8.2 rows that join a file-level and a page-level check

- **Section:** SPEC §8.1, §8.2
- **Raised by:** phase 02
- **Status:** open
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
- **Resolution:**

### Q4: The kind of `@available`'s primary

- **Section:** SPEC §3.4, §4.4
- **Raised by:** phase 02
- **Status:** open
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
- **Resolution:**

### Q5: Problems §8.2 has no row for

- **Section:** SPEC §3.3, §4.1, §4.4, §5.3, §8.2
- **Raised by:** phase 02
- **Status:** open
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
- **Resolution:**

### Q6: What "an id that exists only inside a fragment" means

- **Section:** SPEC §4.2, §5.2, §5.5, §8.2
- **Raised by:** phase 02
- **Status:** open
- **Ambiguity:** §4.2 says "a link to a fragment file, or to an id that exists only inside one, is an error. Link to the page that includes it." But a link's `#id` names "a heading in the target file by its source id" (§5.2), and source ids "depend only on the file" (§5.5). So for `[x](setup.md#prereq)`, where `prereq` is a heading in `_fragments/prereq.md` that `setup.md` includes, `setup.md` has no source id `prereq`, and the link already fails as "id doesn't exist". It's unclear whether:
  - the fragment rule gives that case a clearer error (the id exists, but only in an included fragment), so no link can name a heading that comes from a fragment; or
  - links to `setup.md#prereq` are meant to work, with a page's ids including its fragments' headings, and the error is only for `_fragments/prereq.md#prereq`.
- **Options:**
  1. Source ids are per file, as §5.5 says. A link to an id that only an included fragment has is an error (`link-id-in-fragment`, page level), whose message names the fragment; authors link to the page.
  2. A page's linkable ids include the source ids of the fragments it includes, so `setup.md#prereq` works. §5.2 and §5.5 would need rewording, and ids from a fragment included twice would be ambiguous.
- **Proposed resolution:** option 1, which reports an error rather than guessing a target, and follows §5.5's definition. Implemented now: option 1.
- **Affects:** `tests/conformance/diagnostics.toml` (`link-id-in-fragment`, `provisional` on Q3 and Q6); phases 03, 11, 12, and 14.
- **Resolution:**

### Q7: Whether an explicit `@id` takes part in slug numbering

- **Section:** SPEC §4.1, §5.5
- **Raised by:** phase 02
- **Status:** open
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
- **Resolution:**

### Q8: `<tessera-note title>` is also HTML's `title` attribute

- **Section:** SPEC §9.4
- **Raised by:** phase 02
- **Status:** open
- **Ambiguity:** §9.4's table gives a note's title as `<tessera-note type="tip" title="…">`. `title` is a global HTML attribute: browsers show it as a tooltip whenever the pointer is anywhere over the element, so every titled note shows its title as a tooltip over its whole body, and assistive technology may announce it as the element's description.
- **Options:**
  1. Keep `title`, as the spec says, and accept the tooltip.
  2. Use another name, such as `heading`. Needs a spec change.
- **Proposed resolution:** option 2, `heading`, which the element library would show the same way. Implemented now: option 1, since the spec names the attribute; the element contract marks it provisional.
- **Affects:** `packages/elements/CONTRACT.md`; phases 19 and 20.
- **Resolution:**

### Q9: Attribute names that clash with HTML

- **Section:** SPEC §5.3, §6, §9.4
- **Raised by:** phase 02
- **Status:** open
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
- **Resolution:**

### Q10: Which local files an output may copy

- **Section:** SPEC §4.2, §5.2, §5.3, §9.4 (Assets)
- **Raised by:** phase 02
- **Status:** open
- **Ambiguity:** relative paths may lead out of the content root (`../../shared/logo.png`), and the spec doesn't limit where. Copying any file a destination can reach would let a page publish files from anywhere on the machine, or copy a previous build's output back into the build. The spec also doesn't say whether a reference whose case differs from the file's (`Logo.png` for `logo.png`) exists, which differs between file systems, or whether references inside raw HTML (`<img src="x.png">`) are assets.
- **Options:**
  1. A reference must resolve to a file inside the project root (the directory of `tessera.toml`) or the content root, and not inside the output directory; otherwise it's reported as not existing. Names must match exactly, on every platform. References in raw HTML aren't assets and pass through unchanged.
  2. Allow any file the process can read.
  3. Allow only files inside the content root.
- **Proposed resolution:** option 1. It supports shared images in a monorepo, keeps builds the same on macOS, Windows, and Linux, and reports an error rather than copying an unexpected file. Implemented now: option 1, in the asset contract, with the `outside` and `case` message variants of `image-source-missing` and `link-target-missing`.
- **Affects:** `project-docs/contracts/assets.md`; `tests/conformance/diagnostics.toml`; phases 10, 11, 18, 20, and 25.
- **Resolution:**

### Q11: A title line above a directive that doesn't take one

- **Section:** SPEC §3.7, §8.2
- **Raised by:** phase 02
- **Status:** open
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
- **Resolution:**
