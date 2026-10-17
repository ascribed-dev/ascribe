---
type: reference
title: "`ascribe.toml`: dimensions, phrases, and widgets"
description: "The content model's dimensions, versions, lifecycle states, features, notes, phrases, glossary, images, and widgets."
---

Part of the [`ascribe.toml` reference](index.md), whose [contents](index.md#contents) list every section.

## 6. `[dimensions.<name>]`
@id {permalink=true}: dimensions

@include: ../_fragments/content-model/dimensions.md

**Rules.** A value must belong to only one dimension (`model-dimension-value-shared`); otherwise a bare target in an availability spec would be ambiguous. Dimension names and values also take part in the one-role rule: a name can be a dimension name, a dimension value, a lifecycle state, or a feature key, but only one of them (`model-name-multiple-roles`).

---

## 7. `[versions]`

@include: ../_fragments/content-model/versions.md

| Key | Type | Default | Description |
|---|---|---|---|
| `scheme` | string | `"numeric"` | The version scheme. Spec 0.1 defines one: `"numeric"`. |

@include: ../_fragments/content-model/versions-numeric.md

No other scheme is defined. The table exists so a later spec version can add one without changing the file's shape.

**Rules.** `scheme` is one of the schemes (`model-invalid-value`). `current` isn't a key: a target's current release is its newest live line in [`[releases.<target>]`](#71-releasestarget), and a project that sets it gets an error that says so (`model-versions-current`).

### 7.1 `[releases.<target>]`

@include: ../_fragments/content-model/releases.md

| Key | Type | Default | Description |
|---|---|---|---|
| `live` {available="ascribe 0.3.2"} | array of strings (versions) | `[]` | The lines a reader can be on, in any order. A build [versioned](output.md#16-buildsname) by the target gives one output for each. |
| `planned` {available="ascribe 0.3.2"} | array of strings (versions) | `[]` | Lines that haven't shipped, each newer than every live line. A version in one is **planned**: its availability reads `Self-managed (planned, 3.6)` in every output, and stops being planned when its line moves to `live`, with no other edit. |
| `ends` {available="ascribe 0.3.2"} | table of dates | none | The last day each live line is supported, by line: `"3.4" = 2027-01-31`, a TOML date, unquoted. |
| `archived` {available="ascribe 0.3.2"} | table of tables | none | Lines that have left the window, each older than every live line, by line. Each takes `url`, where the line can still be read, and `ended`, the day its support ended, both optional: `"3.2" = {}` declares a line that's readable nowhere. |

`[releases]` itself takes one key beside the targets' tables:

| Key | Type | Default | Description |
|---|---|---|---|
| `planned-label` {available="ascribe 0.3.2"} | string | `"planned"` | The word a reader sees for a planned version, in place of its state: `Self-managed (planned, 3.6)`. |

**Rules.** `<target>` is a declared dimension value that isn't versionless (`model-releases-target`). Each line is a version, quoted, listed once, and every line of one target, in every list, has the same number of parts (`model-releases-line`). A line is in one list, planned lines are newer than every live line, and archived lines older (`model-releases-order`). A line in `ends` is a live line (`model-releases-ends`), and a date is a TOML date (`model-wrong-type`). Any other key is an error (`model-unknown-key`).

**What the list checks.** For a target with a list, each version a mark, a feature, or a build's filter names is placed among the declared lines:

| The version is | Reported |
|---|---|
| In a declared line | Nothing |
| Newer than every declared line | An error, `available-version-ahead`, with the newest line offered as a fix |
| In a line between two declared ones that isn't declared | Advice, `available-line-undeclared`: declare the line, in `archived` with no address when nothing publishes it, or correct the number, with the declared lines either side offered as fixes |
| Older than every declared line | Nothing: it's before the window, and a project doesn't declare every line it ever shipped |

A mark does nothing only where no output shows it: when every build that publishes its content is versioned by the mark's target or filtered for it, since a line's output leaves out a version at or before the oldest live line. Then two content checks report a mark: `available-same-in-every-line` (advice), a version every live line has, so the mark can drop it and keep its state; and `available-in-no-line` (a warning), content removed before the oldest live line with nothing after, which no output has. A project with a badge build that isn't versioned, such as one site, gets neither: there, `ascribe 0.1.1` still reads "since 0.1.1". [`availability-left-behind`](../checks/diagnostics/content-checks.md) compares a feature's versions with each target's current line, and a live line whose `ends` date has passed is advice, `release-line-ended`, as of today in UTC or the day `ASCRIBE_TODAY` gives.

---

## 8. `[lifecycle.<state>]`

@include: ../_fragments/content-model/lifecycle.md

---

## 9. `[features.<key>]`

@include: ../_fragments/content-model/features.md

---

## 10. `[notes.<type>]`

@include: ../_fragments/content-model/notes.md

---

## 11. `[phrases]`
@id {permalink=true}: phrases

@include: ../_fragments/content-model/phrases.md

---

## 12. `[glossary]`

@include: ../_fragments/content-model/glossary.md

**Matching:** occurrences match whole words only; the longest matching term wins where terms overlap (`API key` over `API`); matching applies to prose only, never to headings, link text, code, raw HTML, or text inside a directive's primary identifier. A term isn't linked on a glossary page that shows it, and a term whose heading a build doesn't publish, or that has no definition in the build, isn't linked in that build.

@include: ../_fragments/content-model/glossary-rules.md

---

## 13. `[images]`

@include: ../_fragments/content-model/images.md

---

## 14. `[widgets.<name>]`

@include: ../_fragments/content-model/widgets.md

**Rules:**

@include: ../_fragments/content-model/widgets-rules.md

**An example: a landing page.** Ascribe's own landing page is a page of Ascribe, laid out with three widgets that [`site/home/ascribe.toml`]({repo}/blob/main/site/home/ascribe.toml) declares and the site's stylesheet draws: `hero-banner`, the opening, with a `status` attribute the site shows as a badge; `claim-example`, a claim with its heading, its sentences, and the example under them, which can carry an [`@available`](../writing/directives.md#available) mark like any other block; and `source-and-result`, whose content is a `@snippet` of a page's file and whose `page` attribute names that page, which the site draws beside it. The lists and the install commands needed no widget.

```markdown
@claim-example:
### Checked

A broken link is an error, in the editor as you type and in CI.

@snippet {lang=text}: code:crates/ascribe-cli/tests/output/getting-started-check.txt
@end
```

What a widget can't say is the site's to do: Ascribe doesn't know that `page` names a page, so it doesn't check that the page exists, and the site's own test does.
