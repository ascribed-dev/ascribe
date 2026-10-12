---
title: "`ascribe.toml`: dimensions, phrases, and widgets"
description: "The content model's dimensions, versions, lifecycle states, features, notes, phrases, glossary, images, and widgets."
---

Part of the [`ascribe.toml` reference](../content-model.md), whose [contents](../content-model.md#contents) list every section.

## 6. `[dimensions.<name>]`
@id {permalink=true}: dimensions

@include: ../../_fragments/content-model/dimensions.md

**Rules.** A value must belong to only one dimension (`model-dimension-value-shared`); otherwise a bare target in an availability spec would be ambiguous. Dimension names and values also take part in the one-role rule: a name can be a dimension name, a dimension value, a lifecycle state, or a feature key, but only one of them (`model-name-multiple-roles`).

---

## 7. `[versions]`

@include: ../../_fragments/content-model/versions.md

| Key | Type | Default | Description |
|---|---|---|---|
| `scheme` | string | `"numeric"` | The version scheme. Spec 0.1 defines one: `"numeric"`. |
| `current` {available="ascribe 0.3.0"} | string (version) | none | The latest release, such as `"0.2.0"`. With it, the [`availability-left-behind`](../diagnostics/content-checks.md) check reports a feature whose versions are all at or before it. Set it to each release as it ships. |

@include: ../../_fragments/content-model/versions-numeric.md

No other scheme is defined. The table exists so a later spec version can add one without changing the file's shape.

**Rules.** `scheme` is one of the schemes (`model-invalid-value`), and `current` is a version, quoted (`model-wrong-type`).

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
@id {permalink=true}: phrases

@include: ../../_fragments/content-model/phrases.md

---

## 12. `[glossary]`

@include: ../../_fragments/content-model/glossary.md

**Matching:** occurrences match whole words only; the longest matching term wins where terms overlap (`API key` over `API`); matching applies to prose only, never to headings, link text, code, raw HTML, or text inside a directive's primary identifier. A term with no `link` isn't linked in the site or plain-markdown output; its definition still reaches the editor and the JSON output.

@include: ../../_fragments/content-model/glossary-rules.md

---

## 13. `[images]`

@include: ../../_fragments/content-model/images.md

---

## 14. `[widgets.<name>]`

@include: ../../_fragments/content-model/widgets.md

**Rules:**

@include: ../../_fragments/content-model/widgets-rules.md

**An example: a landing page.** Ascribe's own landing page is a page of Ascribe, laid out with three widgets that [`site/home/ascribe.toml`]({repo}/blob/main/site/home/ascribe.toml) declares and the site's stylesheet draws: `hero-banner`, the opening, with a `status` attribute the site shows as a badge; `claim-example`, a claim with its heading, its sentences, and the example under them, which can carry an [`@available`](../directives.md#available) mark like any other block; and `source-and-result`, whose content is a `@snippet` of a page's file and whose `page` attribute names that page, which the site draws beside it. The lists and the install commands needed no widget.

```markdown
@claim-example:
### Checked

A broken link is an error, in the editor as you type and in CI.

@snippet {lang=text}: code:crates/ascribe-cli/tests/output/getting-started-check.txt
@end
```

What a widget can't say is the site's to do: Ascribe doesn't know that `page` names a page, so it doesn't check that the page exists, and the site's own test does.
