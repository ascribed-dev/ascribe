---
type: reference
title: "`ascribe.toml` format contract"
description: The normative contract for the content model file, with the rules a loader enforces.
---

This is the normative contract for `ascribe.toml`, the content model file (SPEC §7): its tables and keys, the short syntax for field and attribute types, and every rule a loader enforces. The loader in `ascribe-model` implements it. What each key is for, with examples, is in the [`ascribe.toml` reference](../content-model/index.md), which shares this page's key tables and section numbers.

SPEC.md is normative for the language, and this contract for the file format. The decisions that settle what the spec leaves open are files in [`project-docs/decisions/`]({repo}/tree/main/project-docs/decisions), listed in [decision #689a]({repo}/blob/main/project-docs/decisions/689a-what-a-contract-page-holds.md).

Every file in [`examples/content-models/`]({repo}/tree/main/examples/content-models) is valid under this contract: `minimal.toml` is the smallest valid model, `quill.toml` the model SPEC Appendix B assumes, and `full.toml` has every section and key.

## Contents

1. [Conventions](#1-conventions)
2. [Top level: `spec`](#2-top-level-spec)
3. [`[project]`](#3-project)
4. [`[types.<name>]` and `[fragments]`](#4-typesname-and-fragments)
5. [Field and attribute types](#5-field-and-attribute-types)
6. [`[dimensions.<name>]`](content-model/content.md#6-dimensionsname)
7. [`[versions]`](content-model/content.md#7-versions)
8. [`[lifecycle.<state>]`](content-model/content.md#8-lifecyclestate)
9. [`[features.<key>]`](content-model/content.md#9-featureskey)
10. [`[notes.<type>]`](content-model/content.md#10-notestype)
11. [`[phrases]`](content-model/content.md#11-phrases)
12. [`[glossary]`](content-model/content.md#12-glossary)
13. [`[images]`](content-model/content.md#13-images)
14. [`[widgets.<name>]`](content-model/content.md#14-widgetsname)
15. [`[consumer]`](content-model/output.md#15-consumer)
16. [`[builds.<name>]`](content-model/output.md#16-buildsname)
17. [`[editor]`](content-model/output.md#17-editor)
18. [`[sources.<name>]`](content-model/output.md#18-sourcesname)
19. [`[checks]`](content-model/checks.md#19-checks)
20. [`[permalinks]`](content-model/checks.md#20-permalinks)
21. [`[crawlers]`](content-model/checks.md#21-crawlers)
22. [`[listing]`](content-model/checks.md#22-listing)
23. [Defaults: what an absent section means](content-model/checks.md#23-defaults-what-an-absent-section-means)
24. [Validation rules](content-model/validation.md#24-validation-rules)
25. [Map of SPEC §7.2 to this file](content-model/validation.md#25-map-of-spec-72-to-this-file)

---

## 1. Conventions

@include: ../_fragments/content-model/conventions.md

**Rules.** A loader MUST preserve the order in which attribute keys (`[images.attributes]`, `[widgets.<name>.attributes]`) and the `[dimensions]` tables are declared, and SHOULD preserve the order of everything else. A filesystem path in `[project]` or `[sources]` MUST NOT be absolute, and MAY contain `..` segments (`model-path-absolute`). A pattern's `**` MUST be a whole segment, and a pattern MUST NOT start with `/` or contain a `..` segment (`model-pattern-syntax`).

---

## 2. Top level: `spec`

@include: ../_fragments/content-model/spec.md

---

## 3. `[project]`

@include: ../_fragments/content-model/project.md

@include: ../_generated/content-model-project.md

**Rules** (§24.2): `content-root` and `output-dir` MUST be relative (`model-path-absolute`). The content root MUST exist and be a directory (`model-content-root-missing`); the output directory need not exist. The output directory MUST NOT be inside the content root, the content root MUST NOT be inside the output directory, and the two MUST differ (`model-output-overlaps-content`), each compared after its `.` and `..` segments are normalized and, when both exist, its symbolic links resolved.

@include: ../_fragments/content-model/project-defaults.md

---

## 4. `[types.<name>]` and `[fragments]`

@include: ../_fragments/content-model/types.md

**The page title.** Every page type MUST declare `title` as a required `string` field (`model-type-title`), and a page's frontmatter `title` is its title wherever SPEC needs one (SPEC §5.2). The field MAY set `phrases = true` (§11) and `inline = "code"` (§5.3).

**Reserved keys.** `type`, `available`, `variant`, `permalink`, `listing`, `replaces`, `nav-label` (SPEC §2.1, §4.3, §4.4, §7.2), and `intended` (SPEC §4.9) are reserved frontmatter keys. Every page MUST accept them, with the meaning SPEC gives them, whatever its type declares, and a type MUST NOT declare one (`model-field-reserved`). A generated consumer schema MUST include `type`, as the literal of the type's name on a type the project declares, `available`, `variant`, and `permalink`, and MUST NOT include `listing` (the page's record has it, output-layout contract §1.3), `replaces` (the versions file has it, §1.5), `nav-label` (the navigation has it, §1.6), or `intended`, which no output has. A page type MUST NOT declare `formatted` (§5.3) or `source` (SPEC §9.6), which the site output writes, nor, under the `astro` profile, `slug`, which Astro's content loader reads as an entry id (`model-field-reserved`); a page's frontmatter can't use `formatted` or `source`, since no type declares them.

@include: ../_fragments/content-model/types-frontmatter-and-fragments.md

**Reserved keys on fragments.** A fragment's frontmatter MUST NOT use `type`, `available`, `variant`, `permalink`, `listing`, `replaces`, or `nav-label`, which SPEC defines for pages only (`frontmatter-reserved-in-fragment`, an error on the fragment), and MAY use `intended`. `[fragments.frontmatter]` MUST NOT declare any of the eight (`model-field-reserved`).

---

## 5. Field and attribute types

@include: ../_fragments/content-model/field-types.md

@include: ../_generated/content-model-fields.md

**Rules** (§24.3): `fields` MUST be given exactly when the type is `object` or `list(object)`, optional or not (`model-type-fields`). `values` MUST appear only with bare `enum`, with at least one value and no two alike (`model-enum-values`), and a `set(enum)` attribute's values MUST be tokens (`model-set-token`). A `default` MUST have the declared type, and an enumeration's MUST be one of its values (`model-default-type`). `phrases = true` MUST be set only on `string` and `list(string)` fields, optional or not, including fields in objects (`model-phrases-field-type`). `inline` and `role` MUST be set only on a content type's top-level fields, not in an object or the fragment schema (`model-inline-field`, `model-field-role`).

@include: ../_fragments/content-model/field-types-nesting.md

### 5.3 Inline code in a field

A field that sets `inline = "code"` MUST be read as inline Markdown that knows only code spans; any other `string` field's value is its text, backticks and all.

- **Code spans** are CommonMark's (CommonMark §6.1): a run of backticks opens one, the next run of the same length closes it, and a run with no closer is literal. Their content is normalized as CommonMark does: line endings become spaces, and one space is stripped from each end when both ends have one.
- **Escapes.** Outside code spans, a backslash before ASCII punctuation escapes it, as in CommonMark, so `` \` `` is a literal backtick. Inside them, a backslash is literal.
- **Nothing else is markup.** Emphasis, links, and HTML stay literal text.
- **Phrases** (`phrases = true`) are substituted in the text, never inside code spans, as in prose (SPEC §5.1).

Each output carries two forms of the value. The **plain text** is the value without markup: the code spans' content, with no backticks (`ascribe.toml reference`). The **formatted form** keeps the code spans. Where each goes:

| Output | Plain text | Formatted form |
|---|---|---|
| Site output (its frontmatter) | The field's frontmatter value, for a layout's `<title>`, search, and sorting | `formatted.<field>`, as HTML (`<code>ascribe.toml</code> reference`), for a layout's heading and navigation |
| Empty link text (SPEC §5.2) | Not used | The link text, with the title's code spans |
| Plain-markdown output | Not used | The page's level-1 heading |
| JSON output | `title` and `frontmatter.<field>` | `formatted.<field>`, a list of `{ "type": "text" \| "code", "value": … }` |
| Generated Zod schema | The field, as `z.string()` | `formatted`, with a `z.string()` for each field |

A field the page leaves out gets its default's formatted form, so `formatted.<field>` is there whenever the field has a value.

**Rules** (§24.3): `inline` MUST be `"code"`, on a content type's top-level `string` or `string?` field (`model-inline-field`).

---

### 5.4 Field roles
@available: ascribe 0.3.0

A page type's top-level field MAY have a `role`, which says what the field is for, so a check or the report reads it whatever the field's name (SPEC §7.2):

| Role | The field's type | Read by |
|---|---|---|
| `description` | `string` or `string?` | `page-description-missing`, which reports a page of the type whose field is absent, null, or blank. A field that's required or has a default is never absent, so the check skips it. |
| `review-date` | `date` or `date?` | `review-overdue`, which reports a page whose date is before the day the check runs. |
| `owner` | `string`, `string?`, or an `enum`, optional or not | `ascribe report inventory`, which counts pages by the field's value; a page without one counts as having no owner. |

A type without a field with the role has none of that check's diagnostics. A role changes no output: not the generated schemas, the JSON, or the pages.

**Rules** (§24.3): the role is one of the three, on a field of its type, in a content type's own fields, and at most one field of a type has each role (`model-field-role`). A second field with a role keeps its type, and loses the role.
