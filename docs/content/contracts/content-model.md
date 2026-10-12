---
title: "`ascribe.toml` format contract"
description: The normative contract for the content model file, with the rules a loader enforces and the decisions behind them.
---

This is the normative contract for `ascribe.toml`, the content model file (SPEC §7). It defines every table and key, the short syntax for field and attribute types, and every rule a loader enforces. The loader in `ascribe-model` implements it. For a guide to writing one, see the [`ascribe.toml` reference](../reference/content-model.md).

SPEC.md is normative for the language. This document is normative for the file format. Where it settles something the spec leaves open, the item is listed in [Decisions](content-model/validation.md#26-decisions).

Example files, each valid under this reference:

- [`examples/content-models/minimal.toml`]({repo}/blob/main/examples/content-models/minimal.toml): the smallest valid model.
- [`examples/content-models/quill.toml`]({repo}/blob/main/examples/content-models/quill.toml): the model SPEC Appendix B assumes.
- [`examples/content-models/full.toml`]({repo}/blob/main/examples/content-models/full.toml): every section and key.

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
26. [Decisions](content-model/validation.md#26-decisions)

---

## 1. Conventions

@include: ../_fragments/content-model/conventions.md

---

## 2. Top level: `spec`

@include: ../_fragments/content-model/spec.md

---

## 3. `[project]`

@include: ../_fragments/content-model/project.md

| Key | Type | Default | Description |
|---|---|---|---|
| `content-root` | string (path) | `"docs"` | The content root (SPEC §2.2): the directory holding every source file. Paths in links and includes that begin with `/` are relative to it. It MUST exist and be a directory. |
| `output-dir` | string (path) | `".ascribe/build"` | Where `ascribe build` writes output. Each build and emitter writes under `<output-dir>/<build>/<emitter>/` (see the [output-layout contract](output-layout.md)). It need not exist. |

**Rules** (§24): both paths are relative (`model-path-absolute`). The output directory MUST NOT be inside the content root, the content root MUST NOT be inside the output directory, and they MUST NOT be the same directory (`model-output-overlaps-content`); otherwise a build would read its own output as source, or delete source as stale output. Paths are compared after normalizing `.` and `..` segments and, when both exist, after resolving symbolic links.

@include: ../_fragments/content-model/project-defaults.md

---

## 4. `[types.<name>]` and `[fragments]`

@include: ../_fragments/content-model/types.md

**The page title.** Every page type MUST declare `title` as a required `string` field (`model-type-title`). The frontmatter `title` is the page's title wherever the spec needs one, such as the replacement text of an empty link to a page (SPEC §5.2). The field MAY accept phrases (§11), and MAY set `inline = "code"` for code spans in the title (§5.3).

**Reserved keys.** `available` (SPEC §4.4), `variant` (SPEC §4.3), `permalink` (SPEC §2.1), and `listing` (SPEC §2.1) are reserved frontmatter keys. Every page accepts them, with the meaning the spec gives, whether or not its type mentions them, and a type MUST NOT declare them (`model-field-reserved`). Generated consumer schemas include them automatically, except `listing`, which reaches a site in the page's record (output-layout contract §1.3). `intended` (SPEC §4.9) is reserved too, and every page and every fragment accepts it; no output has it, so generated consumer schemas leave it out, and a type MUST NOT declare it (`model-field-reserved`). Under the `astro` profile, `slug` is reserved too: Astro's content loader uses a page's frontmatter `slug` as its entry id in place of its path, which would publish the page at a URL Ascribe never computed, so a type MUST NOT declare it (`model-field-reserved`). `formatted` is reserved on page types as well: the site output writes the formatted form of `inline` fields under it (§5.3), so a type MUST NOT declare it (`model-field-reserved`). A page's own frontmatter can't use it either, since no type declares it. `source` is reserved on page types for the same reason: the site output writes each page's path from the content root under it (SPEC §9.6), so a type MUST NOT declare it (`model-field-reserved`), and a page can't write it.

@include: ../_fragments/content-model/types-frontmatter-and-fragments.md

**Reserved keys on fragments.** The spec defines `available`, `variant`, `permalink`, and `listing` for pages only. A fragment's frontmatter MUST NOT use them (an error on the fragment), and `[fragments.frontmatter]` MUST NOT declare them (`model-field-reserved`). Use `@available` inside the fragment instead. A fragment MAY use `intended`, and `[fragments.frontmatter]` MUST NOT declare it (`model-field-reserved`).

---

## 5. Field and attribute types

@include: ../_fragments/content-model/field-types.md

| Key | Type | Default | Description |
|---|---|---|---|
| `type` | string | **required** | A field type or attribute type (§5.1). In table form it may also be `object`, `list(object)`, or bare `enum`, with the keys below. |
| `fields` | table of fields | required when `type` is `object`, `object?`, `list(object)`, or `list(object)?`; not allowed otherwise | The nested fields of an object, in the same syntax (short or table form). Field types only. |
| `values` | array of strings | required when `type` uses bare `enum`; not allowed otherwise | The enumeration's values, for values that aren't simple words, such as `["Getting started", "How-to"]`. Values MUST be distinct. Values of a `set(enum)` attribute MUST be tokens. |
| `default` | any TOML value | none | The value used when the field or attribute is absent. It MUST have the declared type (a TOML string for `string` and `enum`, integer or float for `number`, boolean for `boolean`, local date for `date`, array for `list`, inline table for `object`, a string or array of strings for `set`). A field with a default is optional; adding `?` as well is allowed and changes nothing. |
| `phrases` | boolean | `false` | Whether phrases (SPEC §5.1) are substituted in this field's value. Field types only; allowed only on `string` and `list(string)` fields (and their optional forms), including fields nested in objects. See §11. |
| `inline` | string | none | The inline markup the value is read with. The one value is `"code"`: code spans. Allowed only on a content type's top-level `string` fields (and `string?`), not in objects or the fragment schema. See §5.3. |
| `description` | string | none | Help text shown by the editor (hover and completion) and emitted into generated schemas as documentation. |
| `role` | string | none | What the field is for, for the checks that read it (SPEC §7.2): `"description"`, `"review-date"`, or `"owner"`. Allowed only on a content type's top-level fields, not in objects or the fragment schema. See §5.4. |

@include: ../_fragments/content-model/field-types-nesting.md

### 5.3 Inline code in a field

A `string` field is plain text: a backtick in it is a backtick. A field that sets `inline = "code"` is read as inline Markdown that knows only code spans, so a page title can show code:

@include: ../_fragments/content-model/inline-code-example.md

```yaml
title: "`ascribe.toml` reference"
```

- **Code spans** are CommonMark's (CommonMark §6.1): a run of backticks opens one, the next run of the same length closes it, and a run with no closer is literal. Their content is normalized as CommonMark does: line endings become spaces, and one space is stripped from each end when both ends have one.
- **Escapes.** Outside code spans, a backslash before ASCII punctuation escapes it, as in CommonMark, so `` \` `` is a literal backtick. Inside them, a backslash is literal.
- **Nothing else is markup.** Emphasis, links, and HTML stay literal text. A title becomes the text of other links (empty link text, SPEC §5.2), where a link or emphasis would cause trouble.
- **Phrases** (`phrases = true`) are substituted in the text, never inside code spans, as in prose (SPEC §5.1).

Each output carries two forms of the value. The **plain text** is the value without markup: the code spans' content, with no backticks (`ascribe.toml reference`). The **formatted form** keeps the code spans. Where each goes:

| Output | Plain text | Formatted form |
|---|---|---|
| Site output (its frontmatter) | The field's frontmatter value, for a layout's `<title>`, search, and sorting | `formatted.<field>`, as HTML (`<code>ascribe.toml</code> reference`), for a layout's heading and navigation |
| Empty link text (SPEC §5.2) | Not used | The link text, with the title's code spans |
| Plain-markdown output | Not used | The page's level-1 heading |
| JSON output | `title` and `frontmatter.<field>` | `formatted.<field>`, a list of `{ "type": "text" \| "code", "value": … }` |
| Generated Zod schema | The field, as `z.string()` | `formatted`, with a `z.string()` for each field |

A field the page leaves out gets its default's formatted form, so `formatted.<field>` is there whenever the field has a value. The setting is per field, not on for every `string`, so a backtick in an existing field keeps its meaning.

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
