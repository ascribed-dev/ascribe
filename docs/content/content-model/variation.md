---
type: reference
title: "`ascribe.toml`: how content varies"
description: "The content model's dimensions, versions and releases, lifecycle states, and features: what content varies along, and how a page says where it applies."
---

Part of the [`ascribe.toml` reference](index.md), whose [contents](index.md#contents) list every section. The rules for a loader are in the contract's [how content varies](../contracts/content-model/variation.md).

## 11. `[dimensions.<name>]`
@id {permalink=true}: dimensions
@summary: A dimension is an axis content varies along, such as a platform or a deployment, and whose values are availability targets.

@include: ../_fragments/content-model/dimensions.md

A target `[releases.<target>]` names, when no dimension has it, is a value of the implicit dimension `release` ([§12.1](#121-releasestarget)).

**Rules.** A value can be in one dimension only (`model-dimension-value-shared`), since a bare target such as `cloud` in an availability spec has to mean one thing. Dimension names and values also take part in the one-role rule: a name can be a dimension name, a dimension value, a lifecycle state, or a feature key, but only one of them (`model-name-multiple-roles`), and none of them can be `planned`, the version word for a target's planned line.

---

## 12. `[versions]`

@include: ../_fragments/content-model/versions.md

@include: ../_generated/content-model-versions.md

@include: ../_fragments/content-model/versions-numeric.md

No other scheme is defined. The table exists so a later spec version can add one without changing the file's shape.

**Rules.** `scheme` is one of the schemes (`model-invalid-value`). `current` isn't a key: a target's current release is its newest live line in [`[releases.<target>]`](#121-releasestarget), and a project that sets it gets an error that says so (`model-versions-current`).

### 12.1 `[releases.<target>]`
@id {permalink=true}: 121-releasestarget

@include: ../_fragments/content-model/releases.md

@include: ../_generated/content-model-releases-target.md

`[releases]` itself takes two keys beside the targets' tables:

@include: ../_generated/content-model-releases.md

**Rules.** `<target>` is a declared dimension value that isn't versionless, or a name (`name-word`) no dimension has, which makes it a value of the implicit dimension `release`. While a target is left to the implicit dimension, `release` is reserved: a declared `[dimensions.release]` is an error at each such target, naming both. A target a declared dimension has takes no `label`, since that dimension's `labels` labels it (`model-releases-target`). The implicit dimension's name and values take part in the one-role rule, as a declared dimension's do (`model-name-multiple-roles`).

Each line is a version, quoted, listed once, and every line of one target, in every list, has the same number of parts (`model-releases-line`). A line is in one list, planned lines are newer than every live line, and archived lines older (`model-releases-order`). A line in `ends` is a live line (`model-releases-ends`), and a date is a TOML date (`model-wrong-type`). Any other key is an error (`model-unknown-key`).

**What the list checks.** For a target with a list, each version a mark, a feature, or a build's filter names is placed among the declared lines:

| The version is | Reported |
|---|---|
| In a declared line | Nothing |
| Newer than every declared line | An error, `available-version-ahead`, with the newest line offered as a fix |
| In a line between two declared ones that isn't declared | Advice, `available-line-undeclared`: declare the line, in `archived` with no address when nothing publishes it, or correct the number, with the declared lines either side offered as fixes |
| Older than every declared line | Nothing: it's before the window, and a project doesn't declare every line it ever shipped |

A mark's [`planned`](../writing/directives.md#the-planned-line) is the target's one planned line, so it's placed as that line's version; a target with no planned line, or more than one, makes it an error at the word, `available-planned-line`. [`ascribe versions release`](../versions/release.md#cli-versions-release) rewrites it to the version when the line ships.

**A mark that does nothing.** A mark does nothing only where no output shows it: when every build that publishes its content is versioned by the mark's target or filtered for it, since a line's output leaves out a version at or before the oldest live line. Then two content checks report a mark:

| Check | Reports |
|---|---|
| `available-same-in-every-line` (advice) | A version every live line has, so the mark can drop it and keep its state |
| `available-in-no-line` (a warning) | Content removed before the oldest live line with nothing after, which no output has |

**A badge.** A build that badges availability and isn't versioned, such as one site, shows every version. So there:

- `ascribe 0.1.1` still reads "since 0.1.1", and content in no line is still published, so `available-in-no-line` stays quiet.
- When every version a mark names is before the oldest live line and every live line has the content, though, that badge says only that the content is older than any reader's line, so `available-same-in-every-line` reports it too, saying the badge is shown, with an unsafe fix, since removing the versions removes the badge.
- A project that keeps such badges, as "new in 0.2", sets `available-same-in-every-line = "off"` in `[checks]`.

**When the fix removes the mark.** When the mark would then be its target alone, generally available, and the target is the only value of its dimension, the mark says nothing even as a badge ("Available: Ascribe (GA)" in a project with one product), so the fix removes the mark instead of rewriting it: the `@available` line, the page's `available` key, or the row's `{available=…}`. It does so only when the mark names nothing else and no build filters for another target, since content with no mark applies everywhere, and there that's the one target. Content with no mark reads as the scope around it, though, so the fix rewrites the mark instead of removing it when it's:

- Inside a mark that says something, such as a section marked `ascribe preview`: the mark keeps its content generally available.
- In a fragment, which reads as wherever it's included.

**What neither check reports.**

- A version in the oldest live line still says what came with it, so `ascribe 0.4.0` with 0.4.0 the only live line isn't reported there.
- A build filtered for the target at a version before the oldest live line still decides by the mark's versions, so neither check reports a mark while one publishes it.

[`availability-left-behind`](../checks/diagnostics/content-checks.md) compares a feature's versions with each target's current line, and a live line whose `ends` date has passed is advice, `release-line-ended`, as of [the day the checks run as](../checks/check.md#checking-as-of-another-day).

---

## 13. `[lifecycle.<state>]`

@include: ../_fragments/content-model/lifecycle.md

**Rules.** `ga` always counts as available (`model-lifecycle-ga-unavailable`): content with no state is `ga` ([SPEC §4.4]({repo}/blob/main/SPEC.md#44-available)), so otherwise unmarked content would be unavailable. A state you add says whether it counts as available (`model-lifecycle-available-required`). The built-in states take part in the one-role rule too: a dimension value named `beta` is an error.

---

## 14. `[features.<key>]`

@include: ../_fragments/content-model/features.md

**Rules.** A feature's spec is checked as it would be in a page: it has to parse (`model-availability-syntax`), name declared targets and states (`model-availability-unknown-name`), give no versions to a versionless target or a dimension name (`model-availability-versionless`), and list a history oldest first (`model-availability-history-order`). It can't be another feature's key (`model-feature-nested`), so features never refer to each other, and feature keys take part in the one-role rule. It names its versions: [`planned`](../writing/directives.md#the-planned-line) is for a page's mark, and in a feature's spec it's `model-availability-syntax`.
