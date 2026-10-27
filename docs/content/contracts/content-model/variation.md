---
type: reference
title: "`ascribe.toml` format contract: how content varies"
nav-label: "How content varies"
description: "The normative contract for the content model's dimensions, versions and releases, lifecycle states, and features."
---

Part of the [`ascribe.toml` format contract](../content-model.md), whose [contents](../content-model.md#contents) list every section. What these keys are for, with examples, is in the reference's [how content varies](../../content-model/variation.md).

## 11. `[dimensions.<name>]`

@include: ../../_fragments/content-model/dimensions.md

**Rules** (§25.4): a value MUST NOT belong to two dimensions (`model-dimension-value-shared`), and dimension names and values are subject to the one-role rule (`model-name-multiple-roles`).

---

## 12. `[versions]`

@include: ../../_fragments/content-model/versions.md

@include: ../../_generated/content-model-versions.md

@include: ../../_fragments/content-model/versions-numeric.md

**Rules** (§25.5): `scheme` MUST be `"numeric"` (`model-invalid-value`). `current` MUST NOT be set (`model-versions-current`): a target's current release is its newest live line in `[releases.<target>]`.

### 12.1 `[releases.<target>]`

@include: ../../_fragments/content-model/releases.md

@include: ../../_generated/content-model-releases-target.md

`[releases]` takes two keys of its own, beside the targets' tables:

@include: ../../_generated/content-model-releases.md

**Rules.** `<target>` MUST be a declared dimension value that isn't versionless, or a name (`name-word`) that's no declared dimension's value or name, which declares it a value of the implicit dimension `release`. A loader MUST add the implicit dimension, after the declared ones, when at least one target is left to it, and only then. While one is, a declared dimension MUST NOT be named `release`, and each such target is the error (`model-releases-target`). `[releases.<target>] label` MUST NOT be set for a target a declared dimension has (`model-releases-target`). The implicit dimension's name and values are subject to the one-role rule (`model-name-multiple-roles`). Each line MUST be a version (SPEC Appendix A rule `version`), listed once, and every line of one target, in every list, MUST have the same number of parts (`model-releases-line`). A line MUST be in one list, every planned line MUST be newer than every live line, and every archived line older (`model-releases-order`). A key of `ends` MUST be a live line, as written (`model-releases-ends`), and its value a TOML local date (`model-wrong-type`). A key of `archived` is a line as written, and its table MAY have `url`, a non-empty string, and `ended`, a TOML local date, and no other key. `planned-label` MUST NOT be empty. The order of a list carries no meaning.

---

## 13. `[lifecycle.<state>]`

@include: ../../_fragments/content-model/lifecycle.md

**Rules** (§25.4): `ga` MUST count as available (`model-lifecycle-ga-unavailable`). A new state MUST set `available` (`model-lifecycle-available-required`). Lifecycle states, built-in ones included, are subject to the one-role rule (`model-name-multiple-roles`).

---

## 14. `[features.<key>]`

@include: ../../_fragments/content-model/features.md

**Rules** (§25.4): a feature's spec MUST parse as an availability spec (`model-availability-syntax`); each of its targets MUST be a declared dimension value or dimension name, and each state a declared state (`model-availability-unknown-name`); a versionless target or a dimension name MUST NOT have versions (`model-availability-versionless`); a history MUST be in chronological order (`model-availability-history-order`); and the spec MUST NOT be a feature key (`model-feature-nested`). Feature keys are subject to the one-role rule (`model-name-multiple-roles`).
