# Phase 2: Decide the marker

Part of [Embedded help](README.md). Needs phase 1's measurements. A decision by the maintainer; no code.

## Goal

The maintainer settles how a section marks its summary, so phase 3 can build it. Phase 3 ships a change to the language, which is a contract (SPEC §4, Appendix A), so the syntax is settled first, not chosen while building.

## Context

- The proposal's "Why these choices", and its open question on the marker.
- Phase 1's measurements of the docs' file: how often the first paragraph is the right summary, too long, or missing, and the diagnostics' metadata lines.
- SPEC §3 (the directive line and its forms), §4 (the built-in directives), and `@steps` and `@available`, which are the nearest: a directive that wraps content and changes how it's read, not what it says.
- `tests/conformance/tests/docs.rs`: how the diagnostics reference writes each section, which phase 3's generator changes.

## The question

How does a section mark the paragraph a product shows as its summary?

1. **A container around the text,** `@summary:` … `@end`, which every output writes as its content. It marks the text itself, survives the paragraph moving within the section, and needs no counting. The site output writes nothing for it; the plain output writes its content; the JSON output carries it as a container, as `@steps` is.
2. **An attribute on the section's heading,** `@id {permalink=true, summary=2}`, naming a paragraph by its place. It adds no directive, and breaks when a paragraph is added above.
3. **A line form before the paragraph,** `@summary`, wrapping the block after it, as `@available` does. Shorter than the container, and limited to one block.

**Recommended:** 1, the container, or 3 if one block is always enough: the measurements say whether any section wants two paragraphs.

Also settled here, whichever form:

- **More than one marker in a section** is an error, at the second.
- **A marker in a section with no permalink** is advice: it marks nothing anyone reads.
- **A marker in a fragment** marks the summary of the section it lands in, on each page that includes it, as a fragment's heading permalink is cut from its recorded page (decision 7).
- **What the marker may hold:** paragraphs only, or any block. Recommended: paragraphs, since a summary is shown as a sentence or two.
- **What the site output writes for it,** a contract (`docs/content/contracts/site-render.md`). Every container directive writes an element today, and §7's source anchors are built on that. A marker that writes nothing needs a sentence there, and a rule for what review anchors the marked paragraphs to: recommended, their own source, as if the marker weren't there.
- **Its shape in the JSON output,** a contract too: a container, as `@steps` is, for option 1; an annotation on the block, for option 3.
- **A marker inside a variant arm:** the summary of the section for that arm, which phase 6's `choices` would carry, or an error. Recommended: an error until phase 6, which can allow it.
- **A label at the start of the marked paragraph.** The diagnostics reference writes each paragraph as `**When:** …`, so marking it as it is gives every diagnostic the summary "When: …". Either the summary drops a leading bold run that ends in a colon, a rule that also serves a writer's `**Note:** …`, and the reference's pages read as they do; or the generator writes the label outside the marker and the sentence inside, and the reference's look changes. Recommended: the rule, since it leaves the generated pages as they are.

## Output

A decision file in `project-docs/decisions/`, numbered by this phase's issue, with the form, each rule above, and why. Phase 3 starts from it. Nothing else changes.
