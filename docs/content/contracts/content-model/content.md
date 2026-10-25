---
type: reference
title: "`ascribe.toml` format contract: dimensions, phrases, and widgets"
description: "The normative contract for the content model's dimensions, versions, lifecycle states, features, notes, phrases, glossary, images, and widgets."
---

Part of the [`ascribe.toml` format contract](../content-model.md), whose [contents](../content-model.md#contents) list every section. What these keys are for, with examples, is in the reference's [dimensions, phrases, and widgets](../../content-model/content.md).

## 6. `[dimensions.<name>]`

@include: ../../_fragments/content-model/dimensions.md

**Rules** (§25.4): a value MUST NOT belong to two dimensions (`model-dimension-value-shared`), and dimension names and values are subject to the one-role rule (`model-name-multiple-roles`).

---

## 7. `[versions]`

@include: ../../_fragments/content-model/versions.md

@include: ../../_generated/content-model-versions.md

@include: ../../_fragments/content-model/versions-numeric.md

**Rules** (§25.5): `scheme` MUST be `"numeric"` (`model-invalid-value`). `current` MUST NOT be set (`model-versions-current`): a target's current release is its newest live line in `[releases.<target>]`.

### 7.1 `[releases.<target>]`

@include: ../../_fragments/content-model/releases.md

@include: ../../_generated/content-model-releases-target.md

`[releases]` takes two keys of its own, beside the targets' tables:

@include: ../../_generated/content-model-releases.md

**Rules.** `<target>` MUST be a declared dimension value that isn't versionless, or a name (`name-word`) that's no declared dimension's value or name, which declares it a value of the implicit dimension `release`. A loader MUST add the implicit dimension, after the declared ones, when at least one target is left to it, and only then. While one is, a declared dimension MUST NOT be named `release`, and each such target is the error (`model-releases-target`). `[releases.<target>] label` MUST NOT be set for a target a declared dimension has (`model-releases-target`). The implicit dimension's name and values are subject to the one-role rule (`model-name-multiple-roles`). Each line MUST be a version (SPEC Appendix A rule `version`), listed once, and every line of one target, in every list, MUST have the same number of parts (`model-releases-line`). A line MUST be in one list, every planned line MUST be newer than every live line, and every archived line older (`model-releases-order`). A key of `ends` MUST be a live line, as written (`model-releases-ends`), and its value a TOML local date (`model-wrong-type`). A key of `archived` is a line as written, and its table MAY have `url`, a non-empty string, and `ended`, a TOML local date, and no other key. `planned-label` MUST NOT be empty. The order of a list carries no meaning.

---

## 8. `[lifecycle.<state>]`

@include: ../../_fragments/content-model/lifecycle.md

**Rules** (§25.4): `ga` MUST count as available (`model-lifecycle-ga-unavailable`). A new state MUST set `available` (`model-lifecycle-available-required`). Lifecycle states, built-in ones included, are subject to the one-role rule (`model-name-multiple-roles`).

---

## 9. `[features.<key>]`

@include: ../../_fragments/content-model/features.md

**Rules** (§25.4): a feature's spec MUST parse as an availability spec (`model-availability-syntax`); each of its targets MUST be a declared dimension value or dimension name, and each state a declared state (`model-availability-unknown-name`); a versionless target or a dimension name MUST NOT have versions (`model-availability-versionless`); a history MUST be in chronological order (`model-availability-history-order`); and the spec MUST NOT be a feature key (`model-feature-nested`). Feature keys are subject to the one-role rule (`model-name-multiple-roles`).

---

## 10. `[notes.<type>]`

@include: ../../_fragments/content-model/notes.md

**Rules** (§25.4): a new note type MUST set `label` (`model-note-label-required`).

---

## 11. `[phrases]`

@include: ../../_fragments/content-model/phrases.md

**Rules** (§25.5): a phrase's value MUST be a TOML string (`model-phrase-value-type`), or a table whose one key, `line`, names a target with a live line in `[releases]` (`model-phrase-line`). A field's `phrases = true` MUST be on a `string` or `list(string)` field (`model-phrases-field-type`).

---

## 12. `[glossary]`

@include: ../../_fragments/content-model/glossary.md

**Matching.** A term MUST match only whole words, and where two terms overlap, only the longer (`API key`, not `API`). Only prose MUST be matched: never a heading, link text, code, raw HTML, or a directive's primary identifier. A term MUST NOT be linked on a glossary page that shows it, nor in a build that doesn't publish its heading or gives it no definition. SPEC §5.4 gives the rest: what a term is, its definition, and what an author's link to its heading is.

**Rules** (§25.5): each path in `pages` MUST be a page under the content root, not a fragment, with its name's case as on disk (`model-glossary-page`). Terms are read from pages, so the rest are checked with the project, not by the loader: two terms or aliases MUST NOT have the same text, compared ignoring case unless both terms are case-sensitive, nor two terms the same id, and the second of either isn't a term (`glossary-term-duplicate`); a term's section MUST open with a paragraph, its definition (`glossary-definition-missing`), of at most 300 characters in a build, or it's advice (`glossary-definition-long`); and `@term` is a term only on a page `pages` lists, or in a fragment such a page includes (`term-off-glossary-page`), whose fix MUST NOT be applied by `ascribe check --fix`.

---

## 13. `[images]`

@include: ../../_fragments/content-model/images.md

---

## 14. `[widgets.<name>]`

@include: ../../_fragments/content-model/widgets.md

**Rules** (§25.6): a widget with container form MUST NOT have a required primary, and a container-only widget's primary MUST be `"none"` (`model-widget-container-primary`). A groupable widget MUST be container-only (`model-widget-groupable-form`). `binding` MUST be set with line form, and only then (`model-widget-binding`). The element a widget becomes in the site output, its tag name and attributes, is the [element contract]({repo}/blob/main/packages/elements/CONTRACT.md)'s.
