---
type: reference
title: "`ascribe.toml`: what pages hold"
description: "The content model's note types, phrases, glossary, images, and widgets: what a page can hold beyond its type's fields."
---

Part of the [`ascribe.toml` reference](index.md), whose [contents](index.md#contents) list every section. The rules for a loader are in the contract's [what pages hold](../contracts/content-model/content.md).

## 6. `[notes.<type>]`

@include: ../_fragments/content-model/notes.md

---

## 7. `[phrases]`
@id {permalink=true}: phrases
@summary: The phrases registry: each key a page writes in braces, and the text it stands for, or a target's line.

@include: ../_fragments/content-model/phrases.md

---

## 8. `[glossary]`

@include: ../_fragments/content-model/glossary.md

To set one up, see [Set up a glossary](glossary.md).

**Matching:** a term is found as a whole word, never inside one, and where two terms overlap, as `API key` and `API` do, the longer is linked. Only prose is searched, so a term in a heading, link text, code, raw HTML, or a directive's primary identifier stays as it is. A glossary page doesn't link the terms it shows, and a build doesn't link a term whose heading it leaves out, or that has no definition in it.

**Rules.** A path in `pages` names a page that's there, with its name's case as it is on disk (`model-glossary-page`). The rest are checked with the project, since terms are read from pages:

- Two terms or aliases with the same text, compared ignoring case unless both terms are case-sensitive, are reported at the second, which isn't a term (`glossary-term-duplicate`); so are two terms with the same id, which two headings with the same text on different pages give them.
- A term's definition is the paragraph that opens its section, so a term whose section opens with anything else has none (`glossary-definition-missing`). One over 300 characters in a build, the limit a page's `description` has, is advice (`glossary-definition-long`).
- `@term` on a page `pages` doesn't list, or in a fragment no such page includes, isn't a term (`term-off-glossary-page`). Its fix lists the page in `pages`; the editor applies it, and `ascribe check --fix` doesn't, since it links the page's terms on every page.

---

## 9. `[images]`

@include: ../_fragments/content-model/images.md

---

## 10. `[widgets.<name>]`

@include: ../_fragments/content-model/widgets.md

**Rules:**

- A widget with container form can't have a required primary, since a container opener's primary is empty ([SPEC §3.5]({repo}/blob/main/SPEC.md#35-forms)), and a container-only widget's primary is `"none"` (`model-widget-container-primary`).
- A groupable widget is container-only, since the arms of a group are containers ([SPEC §3.6]({repo}/blob/main/SPEC.md#36-groups)) (`model-widget-groupable-form`).
- A widget with line form needs a `binding`, and one without it can't have one (`model-widget-binding`).

What the site output writes for a widget, its element's tag name and attributes, is set by the [element contract]({repo}/blob/main/packages/elements/CONTRACT.md).

What a widget's attributes mean is the site's to check: Ascribe doesn't know that an attribute names a page, so it doesn't check that the page exists. Ascribe's own landing page is laid out with widgets that [`site/home/ascribe.toml`]({repo}/blob/main/site/home/ascribe.toml) declares.
