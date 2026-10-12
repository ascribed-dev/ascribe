---
title: "`ascribe.toml` format contract: dimensions, phrases, and widgets"
description: "The normative contract for the content model's dimensions, versions, lifecycle states, features, notes, phrases, glossary, images, and widgets."
---

Part of the [`ascribe.toml` format contract](../content-model.md), whose [contents](../content-model.md#contents) list every section.

## 6. `[dimensions.<name>]`

@include: ../../_fragments/content-model/dimensions.md

**Rules.** A value MUST belong to only one dimension (`model-dimension-value-shared`); otherwise a bare target in an availability spec would be ambiguous. Dimension names and values also take part in the one-role rule (§24).

---

## 7. `[versions]`

@include: ../../_fragments/content-model/versions.md

| Key | Type | Default | Description |
|---|---|---|---|
| `scheme` | string | `"numeric"` | The version scheme. Spec 0.1 defines one: `"numeric"`. |
| `current` | string | none | The latest release: a version (SPEC Appendix A rule `version`), compared under `scheme`. Nothing in an output depends on it. The content check `availability-left-behind` reports a feature whose spec names a version for every target, none of them later than `current`; without `current` it reports nothing. |

@include: ../../_fragments/content-model/versions-numeric.md

No other scheme is defined. The table exists so a later spec version can add one without changing the file's shape. The scheme is named `numeric`, not "semantic versioning", because it accepts any number of components and has no pre-release syntax.

---

## 8. `[lifecycle.<state>]`

@include: ../../_fragments/content-model/lifecycle.md

---

## 9. `[features.<key>]`

@include: ../../_fragments/content-model/features.md

---

## 10. `[notes.<type>]`

@include: ../../_fragments/content-model/notes.md

---

## 11. `[phrases]`

@include: ../../_fragments/content-model/phrases.md

---

## 12. `[glossary]`

@include: ../../_fragments/content-model/glossary.md

**Matching**: occurrences match whole words only; the longest matching term wins where terms overlap (`API key` over `API`); matching applies to prose only, never to headings, link text, code, raw HTML, or text inside a directive's primary identifier. A term with no `link` isn't linked in the site or plain-markdown output; its definition still reaches the editor and the JSON output. SPEC §5.4 says only that occurrences are linked to definitions.

@include: ../../_fragments/content-model/glossary-rules.md

---

## 13. `[images]`

@include: ../../_fragments/content-model/images.md

---

## 14. `[widgets.<name>]`

@include: ../../_fragments/content-model/widgets.md

**Rules** (§24):

@include: ../../_fragments/content-model/widgets-rules.md
