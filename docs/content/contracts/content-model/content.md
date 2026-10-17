---
type: reference
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
| `scheme` | string | `"numeric"` | The version scheme. It MUST be one Spec 0.1 defines, and it defines one: `"numeric"`. |

@include: ../../_fragments/content-model/versions-numeric.md

No other scheme is defined. The table exists so a later spec version can add one without changing the file's shape. The scheme is named `numeric`, not "semantic versioning", because it accepts any number of components and has no pre-release syntax.

`current` MUST NOT be set: a target's current release is its newest live line in `[releases.<target>]` (`model-versions-current`).

### 7.1 `[releases.<target>]`

@include: ../../_fragments/content-model/releases.md

| Key | Type | Default | Description |
|---|---|---|---|
| `live` | array of strings (versions) | `[]` | The lines a reader can be on, in any order. A build [versioned](output.md#16-buildsname) by the target gives one output for each. |
| `planned` | array of strings (versions) | `[]` | Lines that haven't shipped, in any order. A version in a planned line is planned (SPEC §9.4). |
| `ends` | table of dates | none | For a live line, keyed by the line as written, the last day it's supported: a TOML local date. |
| `archived` | table of tables | none | Lines that have left the window, keyed by the line as written. Each table takes `url`, a non-empty string, and `ended`, a TOML local date, both optional. |

`[releases]` takes one key of its own, beside the targets' tables:

| Key | Type | Default | Description |
|---|---|---|---|
| `planned-label` | string | `"planned"` | The label of a planned version, shown in place of its state. Not empty. |

**Rules.** `<target>` MUST be a declared dimension value that isn't versionless (`model-releases-target`). Each line MUST be a version (SPEC Appendix A rule `version`), listed once, and every line of one target, in every list, MUST have the same number of parts (`model-releases-line`). A line MUST be in one list, every planned line MUST be newer than every live line, and every archived line older (`model-releases-order`). A line in `ends` MUST be a live line (`model-releases-ends`), and a date MUST be a TOML local date (`model-wrong-type`). A list in any other order means the same.

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

**Matching**: occurrences match whole words only; the longest matching term wins where terms overlap (`API key` over `API`); matching applies to prose only, never to headings, link text, code, raw HTML, or text inside a directive's primary identifier. A term isn't linked on a glossary page that shows it, and a term whose heading a build doesn't publish isn't linked in that build. SPEC §5.4 gives the rest: what a term is, its definition, and what an author's link to its heading is.

@include: ../../_fragments/content-model/glossary-rules.md

---

## 13. `[images]`

@include: ../../_fragments/content-model/images.md

---

## 14. `[widgets.<name>]`

@include: ../../_fragments/content-model/widgets.md

**Rules** (§24):

@include: ../../_fragments/content-model/widgets-rules.md
