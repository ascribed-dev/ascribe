---
type: reference
title: "`ascribe.toml` format contract: what pages hold"
nav-label: "What pages hold"
description: "The normative contract for the content model's note types, phrases, glossary, images, and widgets."
---

Part of the [`ascribe.toml` format contract](../content-model.md), whose [contents](../content-model.md#contents) list every section. What these keys are for, with examples, is in the reference's [what pages hold](../../content-model/content.md).

## 6. `[notes.<type>]`

@include: ../../_fragments/content-model/notes.md

**Rules** (§25.4): a new note type MUST set `label` (`model-note-label-required`).

---

## 7. `[phrases]`

@include: ../../_fragments/content-model/phrases.md

**Rules** (§25.5): a phrase's value MUST be a TOML string (`model-phrase-value-type`), or a table whose one key, `line`, names a target with a live line in `[releases]` (`model-phrase-line`). A field's `phrases = true` MUST be on a `string` or `list(string)` field (`model-phrases-field-type`).

---

## 8. `[glossary]`

@include: ../../_fragments/content-model/glossary.md

**Matching.** A term MUST match only whole words, and where two terms overlap, only the longer (`API key`, not `API`). Only prose MUST be matched: never a heading, link text, code, raw HTML, or a directive's primary identifier. A term MUST NOT be linked on a glossary page that shows it, nor in a build that doesn't publish its heading or gives it no definition. SPEC §5.4 gives the rest: what a term is, its definition, and what an author's link to its heading is.

**Rules** (§25.5): each path in `pages` MUST be a page under the content root, not a fragment, with its name's case as on disk (`model-glossary-page`). Terms are read from pages, so the rest are checked with the project, not by the loader: two terms or aliases MUST NOT have the same text, compared ignoring case unless both terms are case-sensitive, nor two terms the same id, and the second of either isn't a term (`glossary-term-duplicate`); a term's section MUST open with a paragraph, its definition (`glossary-definition-missing`), of at most 300 characters in a build, or it's advice (`glossary-definition-long`); and `@term` is a term only on a page `pages` lists, or in a fragment such a page includes (`term-off-glossary-page`), whose fix MUST NOT be applied by `ascribe check --fix`.

---

## 9. `[images]`

@include: ../../_fragments/content-model/images.md

---

## 10. `[widgets.<name>]`

@include: ../../_fragments/content-model/widgets.md

**Rules** (§25.6): a widget with container form MUST NOT have a required primary, and a container-only widget's primary MUST be `"none"` (`model-widget-container-primary`). A groupable widget MUST be container-only (`model-widget-groupable-form`). `binding` MUST be set with line form, and only then (`model-widget-binding`). The element a widget becomes in the site output, its tag name and attributes, is the [element contract]({repo}/blob/main/packages/elements/CONTRACT.md)'s.
