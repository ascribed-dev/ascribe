---
title: ascribe.toml reference
description: "The content model: content types, dimensions, availability, phrases, widgets, and builds."
---

`ascribe.toml` is a project's content model: its schema (SPEC §7). It sits at the project root and declares the content types and their frontmatter, the dimensions content varies along, phrases, the glossary, project widgets, how the site output fits Astro, and the builds. The editor, `ascribe check`, and `ascribe build` all read it, so they can't disagree about what's valid.

A file containing only `spec = "0.1"` is valid; every section has a default ([§19](#19-defaults-what-an-absent-section-means)). Example files:

- [`examples/content-models/minimal.toml`]({repo}/blob/main/examples/content-models/minimal.toml): the smallest valid model.
- [`examples/quill/ascribe.toml`]({repo}/blob/main/examples/quill/ascribe.toml): a small, complete project.
- [`examples/content-models/full.toml`]({repo}/blob/main/examples/content-models/full.toml): every section and key.

Every problem the loader reports is in the [diagnostics reference](diagnostics.md#the-content-model), with its fix. References to "SPEC" are to the [Ascribe specification]({repo}/blob/main/SPEC.md).

## Contents

1. [Conventions](#1-conventions)
2. [Top level: `spec`](#2-top-level-spec)
3. [`[project]`](#3-project)
4. [`[types.<name>]` and `[fragments]`](#4-typesname-and-fragments)
5. [Field and attribute types](#5-field-and-attribute-types)
6. [`[dimensions.<name>]`](#6-dimensionsname)
7. [`[versions]`](#7-versions)
8. [`[lifecycle.<state>]`](#8-lifecyclestate)
9. [`[features.<key>]`](#9-featureskey)
10. [`[notes.<type>]`](#10-notestype)
11. [`[phrases]`](#11-phrases)
12. [`[glossary]`](#12-glossary)
13. [`[images]`](#13-images)
14. [`[widgets.<name>]`](#14-widgetsname)
15. [`[consumer]`](#15-consumer)
16. [`[builds.<name>]`](#16-buildsname)
17. [`[editor]`](#17-editor)
18. [`[sources.<name>]`](#18-sourcesname)
19. [Defaults: what an absent section means](#19-defaults-what-an-absent-section-means)
20. [Validation](#20-validation)

---

## 1. Conventions

### 1.1 The file

- The content model is a TOML 1.0 file named `ascribe.toml`. The directory that contains it is the **project root**. Every path in the file is relative to the project root unless a key says otherwise.
- The file is UTF-8.
- TOML's equivalent spellings are all accepted: a table can be written as a `[header]` section, as an inline table (`key = { … }`), or with dotted keys. This reference shows the most readable form for each section. For example, these are the same:

  ```toml
  [dimensions.pm]
  values = ["npm", "pnpm", "yarn"]

  # same as
  dimensions.pm.values = ["npm", "pnpm", "yarn"]
  ```

- **Keys are kebab-case** (`content-root`, `trailing-slash`), matching Ascribe's attribute keys. TOML allows hyphens in bare keys, so they need no quotes.
- **Unknown keys are errors**, everywhere, with a did-you-mean suggestion. A misspelled key is otherwise a silently ignored setting. The exceptions are the tables whose keys are names the project chooses (`[phrases]`, `[types]`, `[dimensions]`, and so on); their keys are validated as names instead.
- **Declaration order is kept where it matters.** The loader preserves the order in which attribute keys are declared (`[images.attributes]`, `[widgets.<name>.attributes]`), because canonical form writes attributes in declared order ([SPEC §8.3]({repo}/blob/main/SPEC.md#83-canonical-form)). It also preserves the order of the `[dimensions]` tables, which is the canonical order of `@variant` attributes and decides which dimension a tab group syncs on (the [element contract]({repo}/blob/main/packages/elements/CONTRACT.md), §2). Order is kept elsewhere too, for stable output (for example, the order of dimension values in a tab switcher is the order of `values`). Nothing else in the model depends on order.

### 1.2 Names

Several kinds of name appear in the file. Each has a grammar from SPEC Appendix A or from this reference:

| Name | Grammar | Rule | Used for |
|---|---|---|---|
| name-word | SPEC A `name-word` | A letter, then letters, digits, `_`, or `-` | Dimension values, lifecycle states, feature keys, frontmatter field names |
| key | SPEC A `key` | A lowercase letter, then lowercase letters, digits, or `-` | Dimension names, phrase keys, note types, attribute keys, content type names, glossary term ids, source names |
| widget name | SPEC A `widget-name` | Lowercase words of letters and digits joined by single hyphens, with at least one hyphen; starting with a letter | Widget names |
| build name | This reference | A letter, then letters, digits, `_`, `-`, or `.` | Build names |

Dimension names follow the stricter `key` rule, not just `name-word`, because they're also written as attribute keys in `@variant {deployment=cloud}` ([SPEC §3.3]({repo}/blob/main/SPEC.md#33-attributes), §3.3).

Build names allow `.` so that names like `self-managed-3.3` work. A name containing `.` must be quoted in a TOML header: `[builds."self-managed-3.3"]`.

Names are case-sensitive and spelled exactly as declared ([SPEC §4.4]({repo}/blob/main/SPEC.md#44-available)).

### 1.3 Paths and patterns

- Paths use `/` as the separator on every platform.
- Filesystem paths in `[project]` and `[sources]` are relative to the project root. They must not be absolute, and they may use `..`.
- A glossary term's `link` (§12) is written like a link destination in a document ([SPEC §5.2]({repo}/blob/main/SPEC.md#52-links)), relative to the content root.
- **Patterns** (globs) are matched against a source file's path relative to the content root, including its extension, with `/` separators. They are case-sensitive. The syntax:

  | Syntax | Matches |
  |---|---|
  | `*` | Any run of characters within one path segment, not including `/` |
  | `**` | Any number of whole segments, including none. It must be a whole segment (`a/**/b`, `a/**`, `**/b`) |
  | `?` | Any one character except `/` |
  | `{a,b}` | Either alternative. Alternatives don't nest |
  | `\` | Escapes the next character |

  Every other character matches itself. A pattern must not start with `/` and must not contain a `..` segment. Patterns only ever apply to Markdown source files (`*.md`); other files under the content root are assets. The exception is a source's `include` and `ignore` (§18), which are matched against any file's path relative to the source's folder.

### 1.4 How this reference describes keys

Each section has a table of keys with these columns:

- **Type:** the TOML type. "Field type", "attribute type", and "availability spec" are strings with the grammars in §5 and [SPEC §4.4]({repo}/blob/main/SPEC.md#44-available).
- **Default:** the value when the key is absent, or **required**.
- **Description.**

---

## 2. Top level: `spec`

```toml
spec = "0.1"
```

| Key | Type | Default | Description |
|---|---|---|---|
| `spec` | string | **required** | The version of the Ascribe specification this project targets (SPEC §11). It must be quoted: `spec = 0.1` is a TOML float and is an error. A processor accepts only the spec versions it implements, compared as exact strings; this reference defines `"0.1"`. |

The only other top-level keys are the tables in §3–§17. Anything else is an unknown key.

---

## 3. `[project]`

Where the source lives and where builds write.

@snippet: code:examples/content-models/full.toml#project

| Key | Type | Default | Description |
|---|---|---|---|
| `content-root` | string (path) | `"docs"` | The content root ([SPEC §2.2]({repo}/blob/main/SPEC.md#22-pages-and-fragments)): the directory holding every source file. Paths in links and includes that begin with `/` are relative to it. It must exist and be a directory. |
| `output-dir` | string (path) | `".ascribe/build"` | Where `ascribe build` writes output. Each build and emitter writes under `<output-dir>/<build>/<emitter>/`. It need not exist. |

**Rules:** both paths are relative (`model-path-absolute`). The output directory must not be inside the content root, the content root must not be inside the output directory, and they must not be the same directory (`model-output-overlaps-content`); otherwise a build would read its own output as source, or delete source as stale output. Paths are compared after normalizing `.` and `..` segments and, when both exist, after resolving symbolic links.

The defaults are `".ascribe/build"` keeps generated output out of the way of both the source and a consumer's own `dist/`.

---

## 4. `[types.<name>]` and `[fragments]`

### 4.1 Page types

A **content type** is a frontmatter schema for a kind of page, plus the files it applies to.

```toml
[types.guide]
default = true

[types.guide.frontmatter]
title = "string"
description = "string?"

[types.reference]
files = ["reference/**"]

[types.reference.frontmatter]
title = "string"
api-version = "string"
```

`<name>` is the type's name (`key` rule). It appears in diagnostics and in generated code (the Zod schema's name).

| Key | Type | Default | Description |
|---|---|---|---|
| `files` | array of patterns | `[]` | The pages this type applies to (§1.3). |
| `default` | boolean | `false` | Makes this the **default type**: it applies to every page that no type's `files` match. At most one type may be the default. |
| `frontmatter` | table of fields | **required** | The frontmatter schema: one entry per field, keyed by field name (`name-word`). Values are field types (§5). It must declare `title` as a required string (below). |

**Which type applies to a page.** For each page (not fragments; see §4.3):

1. If exactly one type's `files` match the page, that type applies.
2. If several types' `files` match, it's an error on the page: the page is ambiguous. There's no precedence between types.
3. If none match and a default type exists, the default applies.
4. If none match and there's no default type, it's an error on the page.

A type with neither `files` nor `default = true` could never apply, and is an error (`model-type-unreachable`). The rules in steps 2 and 4 are document diagnostics, listed in [SPEC §8.2]({repo}/blob/main/SPEC.md#82-diagnostics).

**The page title.** Every page type must declare `title` as a required `string` field (`model-type-title`). The frontmatter `title` is the page's title wherever the spec needs one, such as the replacement text of an empty link to a page ([SPEC §5.2]({repo}/blob/main/SPEC.md#52-links)). The field may accept phrases (§11).

**Reserved keys.** `available` ([SPEC §4.4]({repo}/blob/main/SPEC.md#44-available)) and `variant` ([SPEC §4.3]({repo}/blob/main/SPEC.md#43-variant)) are reserved frontmatter keys. Every page accepts them, with the meaning the spec gives, whether or not its type mentions them, and a type must not declare them (`model-field-reserved`). Generated consumer schemas include them automatically. Under the `astro` profile, `slug` is reserved too: Astro's content loader uses a page's frontmatter `slug` as its entry id in place of its path, which would publish the page at a URL Ascribe never computed, so a type must not declare it (`model-field-reserved`).

**Unknown frontmatter keys.** A page whose frontmatter has a key its type doesn't declare (and that isn't reserved) is an error on the page.

### 4.2 Frontmatter values

Frontmatter is YAML. For type checking, processors parse it with the YAML 1.2 **core schema**: `true` and `false` are booleans (`yes`, `no`, `on`, and `off` are strings), and `3.10` is the number 3.1. So a `string` field whose value is `3.10` unquoted is a type error, and the message suggests quoting it.

### 4.3 Fragments

@snippet: code:examples/content-models/full.toml#fragments

| Key | Type | Default | Description |
|---|---|---|---|
| `patterns` | array of patterns | `[]` | Additional fragment patterns ([SPEC §2.2]({repo}/blob/main/SPEC.md#22-pages-and-fragments)). A file is a fragment if any segment of its path begins with `_`, or its path matches one of these patterns. |
| `frontmatter` | table of fields | `{}` (no fields) | The fragment schema ([SPEC §2.2]({repo}/blob/main/SPEC.md#22-pages-and-fragments)): the frontmatter every fragment is validated against. Fields as in §5. `title` is not required. |

Content types never apply to fragments, even when a type's `files` match a fragment's path.

**Reserved keys on fragments.** The spec defines `available` and `variant` for pages only. A fragment's frontmatter must not use them (an error on the fragment), and `[fragments.frontmatter]` must not declare them (`model-field-reserved`). Use `@available` inside the fragment instead.

---

## 5. Field and attribute types

Two related type languages:

- **Field types** describe frontmatter fields (`[types.<name>.frontmatter]`, `[fragments.frontmatter]`). They cover the [SPEC §7.2]({repo}/blob/main/SPEC.md#72-declarations) set: string, number, boolean, date, enumeration, list, and object, each optional or with a default.
- **Attribute types** describe attributes on images and widgets (`[images.attributes]`, `[widgets.<name>.attributes]`). They cover [SPEC §3.3]({repo}/blob/main/SPEC.md#33-attributes)'s set: string, enumeration, boolean, and number, plus set-valued keys.

Each entry is written either in **short form**, a string, or in **table form**, when it needs a default, a description, phrases, enumeration values that aren't simple words, or nested fields.

```toml
[types.guide.frontmatter]
title = "string"                                   # required string
description = "string?"                            # optional string
level = "enum(beginner, intermediate, advanced)"   # required, one of three
tags = "list(string)?"                             # optional list of strings
updated = "date?"                                  # optional date
status = { type = "enum(draft, published)", default = "published" }
author = { type = "object?", fields = { name = "string", url = "string?" } }
```

### 5.1 Short form

A field is **required** unless its type ends in `?` or it has a default. The same holds for attributes: `lab = "string"` means every use of the widget must give `lab`.

| Short form | Field types | Attribute types | Accepts |
|---|---|---|---|
| `string` | yes | yes | Frontmatter: a YAML string. Attribute: a token or quoted string. |
| `number` | yes | yes | Frontmatter: a YAML integer or float. Attribute: a token matching `["-"] 1*DIGIT ["." 1*DIGIT]` (`600`, `-2`, `1.5`; not `600px`). |
| `boolean` | yes | yes | Frontmatter: YAML `true` or `false`. Attribute: the token `true` or `false` ([SPEC §3.3]({repo}/blob/main/SPEC.md#33-attributes)). |
| `date` | yes | no | A calendar date written `YYYY-MM-DD` that exists (`2026-02-30` is an error), quoted or not. Times aren't accepted. |
| `enum(a, b, …)` | yes | yes | One of the listed values, compared exactly. Frontmatter: a YAML string. Attribute: a token or quoted string. |
| `list(T)` | yes | no | A YAML sequence whose items are all of type `T`, where `T` is `string`, `number`, `boolean`, `date`, `enum(…)`, or (table form only) `object`. An empty sequence is allowed. |
| `set(T)` | no | yes | A value set ([SPEC §3.3]({repo}/blob/main/SPEC.md#33-attributes)), such as `platform=cloud\|on-prem`, or a single token. `T` is `string` or `enum(…)`. Members are tokens. Only keys typed `set(…)` accept `\|`. |
| `object` | table form only | no | A YAML mapping whose keys are the object's `fields` (nested field types). Unknown keys are errors. |
| any of the above + `?` | yes | yes | The same type, optional. |

Grammar (ABNF, with the rules of SPEC Appendix A):

@include: ../_fragments/field-type-grammar.md

### 5.2 Table form

| Key | Type | Default | Description |
|---|---|---|---|
| `type` | string | **required** | A field type or attribute type (§5.1). In table form it may also be `object`, `list(object)`, or bare `enum`, with the keys below. |
| `fields` | table of fields | required when `type` is `object`, `object?`, `list(object)`, or `list(object)?`; not allowed otherwise | The nested fields of an object, in the same syntax (short or table form). Field types only. |
| `values` | array of strings | required when `type` uses bare `enum`; not allowed otherwise | The enumeration's values, for values that aren't simple words, such as `["Getting started", "How-to"]`. Values must be distinct. Values of a `set(enum)` attribute must be tokens. |
| `default` | any TOML value | none | The value used when the field or attribute is absent. It must have the declared type (a TOML string for `string` and `enum`, integer or float for `number`, boolean for `boolean`, local date for `date`, array for `list`, inline table for `object`, a string or array of strings for `set`). A field with a default is optional; adding `?` as well is allowed and changes nothing. |
| `phrases` | boolean | `false` | Whether phrases ([SPEC §5.1]({repo}/blob/main/SPEC.md#51-phrases)) are substituted in this field's value. Field types only; allowed only on `string` and `list(string)` fields (and their optional forms), including fields nested in objects. See §11. |
| `description` | string | none | Help text shown by the editor (hover and completion) and emitted into generated schemas as documentation. |

Nesting beyond one level of `fields` is allowed but discouraged; keep frontmatter flat.

---

## 6. `[dimensions.<name>]`

A **dimension** is an axis content varies along ([SPEC §4.3]({repo}/blob/main/SPEC.md#43-variant)), and whose values are availability targets ([SPEC §4.4]({repo}/blob/main/SPEC.md#44-available)).

@snippet: code:examples/content-models/full.toml#deployment

`<name>` is the dimension's name (`key` rule, §1.2). It's written as an attribute key in `@variant`, as a key in `variant` frontmatter and build selections, and as a target in availability specs, where it stands for all its values.

| Key | Type | Default | Description |
|---|---|---|---|
| `values` | array of strings | **required** | The dimension's values (`name-word`), in display order. At least one; no duplicates. |
| `labels` | table of strings | `{}` | Display labels for values, keyed by value ([SPEC §9.4]({repo}/blob/main/SPEC.md#94-outputs): "Labels for dimension values come from the content model's display labels"). A value without a label is displayed as the value itself. Every key must be a declared value. |
| `label` | string | the dimension's name | The display label for the dimension itself, used where a dimension name appears as an availability target or names a tab group. |
| `versionless` | array of strings | `[]` | Values that are versionless ([SPEC §4.4]({repo}/blob/main/SPEC.md#44-available)): availability for them takes a single state and no versions. Every entry must be a declared value. Values not listed are versioned. |

**Rules.** A value must belong to only one dimension (`model-dimension-value-shared`); otherwise a bare target in an availability spec would be ambiguous. Dimension names and values also take part in the one-role rule: a name can be a dimension name, a dimension value, a lifecycle state, or a feature key, but only one of them (`model-name-multiple-roles`).

---

## 7. `[versions]`

How versions in availability specs are compared ([SPEC §4.4]({repo}/blob/main/SPEC.md#44-available)).

@snippet: code:examples/content-models/full.toml#versions

| Key | Type | Default | Description |
|---|---|---|---|
| `scheme` | string | `"numeric"` | The version scheme. Spec 0.1 defines one: `"numeric"`. |

**The `numeric` scheme.** A version is any string matching SPEC Appendix A's `version` rule: numbers separated by dots (`3`, `3.4`, `3.4.1`). Versions compare component by component, numerically, from the left, with missing trailing components treated as `0`: `3.4` equals `3.4.0`, `3.10` is later than `3.9`, and `4` is later than `3.99.1`. Leading zeros don't matter (`3.04` equals `3.4`). This is semantic versioning's `major.minor.patch` ordering. It has no pre-release or build suffixes, because the spec's grammar doesn't allow them; express a pre-release with a lifecycle state (`preview 3.4`) instead.

No other scheme is defined. The table exists so a later spec version can add one without changing the file's shape.

---

## 8. `[lifecycle.<state>]`

Lifecycle states ([SPEC §4.4]({repo}/blob/main/SPEC.md#44-available)). Five are built in:

| State | Counts as available | Default label |
|---|---|---|
| `preview` | yes | `preview` |
| `beta` | yes | `beta` |
| `ga` | yes | `GA` |
| `deprecated` | yes | `deprecated` |
| `removed` | no | `removed` |

A project adds states by declaring them, and may change the `available` flag or `label` of a built-in state. Built-in states can't be removed.

@snippet: code:examples/content-models/full.toml#lifecycle

`<state>` is the state's name (`name-word`).

| Key | Type | Default | Description |
|---|---|---|---|
| `available` | boolean | built-in states: as in the table above; new states: **required** | Whether content in this state counts as available ([SPEC §4.4]({repo}/blob/main/SPEC.md#44-available), §8.3). A new state must say so explicitly. |
| `label` | string | built-in states: as in the table above; new states: the state's name | The display label used in availability annotations, such as "Available: Quill Cloud (GA); self-managed (preview, 3.4+)" ([SPEC §9.4]({repo}/blob/main/SPEC.md#94-outputs)). |

**Rules.** `ga` must count as available (`model-lifecycle-ga-unavailable`), because content with no state is `ga` ([SPEC §4.4]({repo}/blob/main/SPEC.md#44-available)). Lifecycle states take part in the one-role rule, including the built-in ones: a dimension value named `beta` is an error.

---

## 9. `[features.<key>]`

The features registry ([SPEC §4.4]({repo}/blob/main/SPEC.md#44-available)): named availability specs, so a feature going generally available takes one edit.

@snippet: code:examples/content-models/full.toml#features

`<key>` is the feature key (`name-word`). Writing it as a whole `@available` primary, or as the whole `available` frontmatter value, stands for the spec.

| Key | Type | Default | Description |
|---|---|---|---|
| `name` | string | **required** | The feature's display name, shown on hover and available to emitters. |
| `available` | string (availability spec) | **required** | The feature's availability, in [SPEC §4.4]({repo}/blob/main/SPEC.md#44-available)'s syntax. |

**Rules.** The spec is checked like one written in a document: it must parse (`model-availability-syntax`); every target must be a declared dimension value or dimension name, and every state a declared state (`model-availability-unknown-name`); versionless targets and dimension names take no versions (`model-availability-versionless`); histories must be in chronological order (`model-availability-history-order`). It must not be a feature key itself (`model-feature-nested`), so features never refer to each other. Feature keys take part in the one-role rule.

---

## 10. `[notes.<type>]`

Note types ([SPEC §4.5]({repo}/blob/main/SPEC.md#45-note)), the values of `@note`'s `type` attribute. Five are built in: `note`, `tip`, `important`, `warning`, and `caution`. A project adds types by declaring them, and may relabel a built-in type. Built-in types can't be removed.

@snippet: code:examples/content-models/full.toml#notes

`<type>` is the note type (`key` rule).

| Key | Type | Default | Description |
|---|---|---|---|
| `label` | string | built-in types: `Note`, `Tip`, `Important`, `Warning`, `Caution`; new types: **required** | The display label, used where a note's type is shown as text, such as the plain-markdown output's `**Tip: …**` ([SPEC §9.4]({repo}/blob/main/SPEC.md#94-outputs)). |

Under `[notes]`, the inline form `security = { label = "Security" }` is equivalent.

---

## 11. `[phrases]`

The phrases registry ([SPEC §5.1]({repo}/blob/main/SPEC.md#51-phrases)).

@snippet: code:examples/content-models/full.toml#phrases

Every key is a phrase key (`key` rule, the same rule as SPEC Appendix A's `phrase`), and every value must be a TOML string (`model-phrase-value-type`). `version = 3.4` is a float and an error; that's the kind of silent type change TOML exists to prevent ([SPEC §7.1]({repo}/blob/main/SPEC.md#71-role)). Values are inserted as literal text ([SPEC §5.1]({repo}/blob/main/SPEC.md#51-phrases)).

**Phrases in frontmatter.** [SPEC §5.1]({repo}/blob/main/SPEC.md#51-phrases) lets the content model decide which frontmatter fields accept phrases. A field accepts them when its table form sets `phrases = true` (§5.2):

```toml
[types.guide.frontmatter]
title = { type = "string", phrases = true }
```

Only `string` and `list(string)` fields can accept phrases (`model-phrases-field-type`). By default, no field does. The setting lives on the field, not in `[phrases]`, so that a phrase key can never collide with a setting's name.

---

## 12. `[glossary]`

The glossary ([SPEC §5.4]({repo}/blob/main/SPEC.md#54-glossary-terms)): terms, their definitions, and how occurrences are matched. Authors don't mark terms in source; processors link them ([SPEC §9.2]({repo}/blob/main/SPEC.md#92-resolution) step 7).

@snippet: code:examples/content-models/full.toml#glossary

### 12.1 Settings

| Key | Type | Default | Description |
|---|---|---|---|
| `match` | string | `"first"` | Which occurrences are linked: `"first"`, the first occurrence of each term on each page, or `"every"`, every occurrence. "Page" means the resolved page for a build, after includes and build modes. |
| `case-sensitive` | boolean | `false` | Whether occurrences must match a term's case exactly. When `false`, matching ignores case. A term can override it. |
| `terms` | table of terms | `{}` | The terms, keyed by term id (`key` rule). |

### 12.2 Terms

| Key | Type | Default | Description |
|---|---|---|---|
| `term` | string | **required** | The term as it appears in prose. Non-empty. |
| `aliases` | array of strings | `[]` | Other forms that count as occurrences, such as plurals. |
| `definition` | string | **required** | A short plain-text definition, shown on hover in the editor and included in the JSON output. |
| `link` | string (path) | none | Where the full definition lives: a source file path relative to the content root, with an optional `#id`, written as in a link ([SPEC §5.2]({repo}/blob/main/SPEC.md#52-links); a leading `/` is allowed and means the same). It must name a page, not a fragment. Occurrences are linked here. |
| `case-sensitive` | boolean | the `[glossary]` setting | Overrides `case-sensitive` for this term, for terms like `Go` that collide with ordinary words. |

**Matching:** occurrences match whole words only; the longest matching term wins where terms overlap (`API key` over `API`); matching applies to prose only, never to headings, link text, code, raw HTML, or text inside a directive's primary identifier. A term with no `link` isn't linked in the site or plain-markdown output; its definition still reaches the editor and the JSON output.

**Rules.** No two terms or aliases may be the same text (compared ignoring case when either is case-insensitive; `model-glossary-duplicate-term`). The `link` file must exist and not be a fragment (`model-glossary-link`); its `#id` is checked with the page-level link checks, since ids depend on parsing.

---

## 13. `[images]`

Image attributes ([SPEC §5.3]({repo}/blob/main/SPEC.md#53-images)): which keys may appear in an attribute block after an image, and their types.

@snippet: code:examples/content-models/full.toml#images

| Key | Type | Default | Description |
|---|---|---|---|
| `attributes` | table of attribute types | `{}` | Accepted image attribute keys (`key` rule) and their types (§5). With no entries, an image accepts no attributes, and any attribute block after an image is an error ([SPEC §8.2]({repo}/blob/main/SPEC.md#82-diagnostics), "Unknown key"). Declaration order is canonical order ([SPEC §8.3]({repo}/blob/main/SPEC.md#83-canonical-form)). A declared default reaches the output: in the site output, every image carries the defaults of attributes it doesn't write. |

Alt text and titles aren't attributes; they use CommonMark's syntax ([SPEC §5.3]({repo}/blob/main/SPEC.md#53-images)). Presentation choices aren't image attributes either. Keys HTML already uses on `<img>` (`src`, `alt`, `title`, and its global attributes) are rejected (`model-attribute-reserved`), since the site output writes image attributes onto the `<img>` element ([SPEC §7.2]({repo}/blob/main/SPEC.md#72-declarations)).

---

## 14. `[widgets.<name>]`

Project widgets (SPEC §6): directives a documentation set defines. Each declaration is a directive schema with the same parts a built-in directive's schema has (SPEC §3, §3).

@snippet: code:examples/content-models/full.toml#widgets

`<name>` is the widget's name (SPEC A `widget-name`: lowercase, with at least one hyphen). Names starting with `ascribe-` are reserved for Ascribe's element library, and the names HTML reserves for itself (`annotation-xml`, `color-profile`, `font-face`, `font-face-src`, `font-face-uri`, `font-face-format`, `font-face-name`, `missing-glyph`) aren't allowed, because the site output emits a widget as a custom element with the widget's name ([SPEC §9.4]({repo}/blob/main/SPEC.md#94-outputs)).

| Key | Type | Default | Description |
|---|---|---|---|
| `forms` | array of strings | **required** | The permitted forms ([SPEC §3.5]({repo}/blob/main/SPEC.md#35-forms)): `["line"]`, `["container"]`, or both. |
| `primary` | string | `"none"` | The primary ([SPEC §3.4]({repo}/blob/main/SPEC.md#34-the-primary)): `"none"`; `"identifier"` or `"text"`, which are required; or `"identifier?"` or `"text?"`, which are optional. |
| `binding` | string | required when `forms` includes `"line"`; not allowed otherwise | What the line form applies to ([SPEC §3.8]({repo}/blob/main/SPEC.md#38-binding)), one of the values below. |
| `title` | string | `"none"` | Whether the widget takes a title line ([SPEC §3.7]({repo}/blob/main/SPEC.md#37-titles)): `"none"`, `"accepted"`, or `"required"`. |
| `groupable` | boolean | `false` | Whether a run of the widget's openers forms a group of arms ([SPEC §3.6]({repo}/blob/main/SPEC.md#36-groups)). |
| `attributes` | table of attribute types | `{}` | The attribute schema: keys (`key` rule) and types (§5). Declaration order is canonical order ([SPEC §8.3]({repo}/blob/main/SPEC.md#83-canonical-form)). The keys `heading` and `primary`, HTML's global attributes (including `title`), `aria-` keys, and event-handler attributes such as `onclick`, are rejected (`model-attribute-reserved`): the site output writes these attributes onto the widget's element ([SPEC §7.2]({repo}/blob/main/SPEC.md#72-declarations)). |
| `plain-fallback` | string | none | Plain-text fallback for the plain-markdown output (SPEC §6, §8.4): CommonMark text written in place of the widget. Phrases in it are substituted. It isn't a template: attribute values aren't inserted. Without it, the widget itself emits nothing. |
| `plain-content` | string | `"keep"` | For a widget that wraps content, whether the plain-markdown output keeps that content (`"keep"`) after the fallback, or drops it (`"drop"`). Allowed only when the widget wraps content: it has container form, or its binding is `block` or `heading-or-block`. |
| `description` | string | none | Help text for the editor's hover and completion. |

**Binding values:**

| Value | [SPEC §3.8]({repo}/blob/main/SPEC.md#38-binding) binding | Behaves like |
|---|---|---|
| `"self"` | Self: its own primary, or nothing | `@include` |
| `"heading"` | Preceding heading: at the top of its section | `@id` |
| `"block"` | Following block. If the primary is a text primary and it's given, the primary is the content and nothing else is bound | `@steps`; with `primary = "text?"`, `@note` |
| `"heading-or-block"` | At the top of a section, the section; anywhere else, the following block | `@available` |

**Rules:**

- A widget with container form must not have a required primary, since a container opener's primary is empty ([SPEC §3.5]({repo}/blob/main/SPEC.md#35-forms)). A container-only widget's primary must be `"none"` (`model-widget-container-primary`).
- A groupable widget must be container-only, since group arms are containers ([SPEC §3.6]({repo}/blob/main/SPEC.md#36-groups)) (`model-widget-groupable-form`).
- `binding` is required with line form and not allowed without it (`model-widget-binding`).

The site output's element for a widget (tag name and attributes) is defined by the [element contract]({repo}/blob/main/packages/elements/CONTRACT.md), not here.

---

## 15. `[consumer]`

The consumer profile ([SPEC §9.5]({repo}/blob/main/SPEC.md#95-consumer-profile)): how the site output fits a specific consumer. Spec 0.1's toolchain implements one profile, `astro` ([SPEC §9.6]({repo}/blob/main/SPEC.md#96-astro)). The profile sets defaults for every other key; a key given here overrides them, within the values the profile supports.

```toml
[consumer]
profile = "astro"
site = "https://docs.quill.dev"
base-path = "/"
trailing-slash = "always"
slugger = "github"
```

| Key | Type | Default | Description |
|---|---|---|---|
| `profile` | string | `"astro"` | The consumer profile. Supported: `"astro"`. |
| `site` | string (URL) | none | The published site's origin, such as `"https://docs.quill.dev"`: an `http` or `https` URL with no path, query, or fragment. The plain-markdown output needs it to write absolute links ([SPEC §9.4]({repo}/blob/main/SPEC.md#94-outputs)). Without it, plain-markdown links are root-relative (they start with `base-path`) and `ascribe build` warns. |
| `base-path` | string | `"/"` | **Routing.** The URL path every route starts with, such as `"/docs/"`, including any locale prefix (`"/en/"`). It must start with `/`. A trailing `/` is optional and doesn't change the meaning. |
| `trailing-slash` | string | `"always"` | **Routing.** Whether page URLs end in `/`: `"always"` (`/guides/setup/`) or `"never"` (`/guides/setup`). Match the consumer's own setting (Astro's `trailingSlash` and `build.format`). |
| `slugger` | string | `"github"` | **Slugging.** The algorithm for heading ids ([SPEC §5.5]({repo}/blob/main/SPEC.md#55-heading-ids)), which must be the one the consumer uses. Supported: `"github"`, a port of `github-slugger`, which Astro uses. |
| `html` | boolean | `true` | **HTML passthrough.** Whether the consumer renders raw HTML in markdown. The site output's custom elements need it, so the `astro` profile supports only `true`. |

**Heading ids, image attributes, and assets** ([SPEC §9.5]({repo}/blob/main/SPEC.md#95-consumer-profile)) are part of the profile, not keys. The `astro` profile has exactly one way to do each:

- **Heading ids and image attributes:** the site output writes each as a `<ascribe-attributes>` marker that the consumer's markdown plugin applies, so the consumer keeps its own heading, table-of-contents, and image processing.
- **Assets:** copies mirror their source paths inside each output, and images are referenced relatively so Astro's image processing still applies. Other files a page links to are published under `_ascribe/files/`.

A later profile that offers a choice will add a key for it.

**How file paths become routes** (the `astro` profile): a page's route is `base-path`, then its path relative to the content root with the `.md` extension removed and each segment slugged the way Astro's content loader computes entry ids; a final `index` segment is dropped (`guides/index.md` → `/guides/`); then the trailing slash per `trailing-slash`. The root `index.md` (Astro's entry id `index`) is at `base-path`, which under `trailing-slash = "never"` loses its final `/` unless it's `/`. Two pages with one entry id (`My File.md` and `my-file.md`, or `index.md` and `index/index.md`) can't both be published, and `ascribe build --emit site` fails, naming them. The same router answers the reverse question, which page a route-like link names, for the `link-route` warning and its fix. Source files never contain routes ([SPEC §5.2]({repo}/blob/main/SPEC.md#52-links)).

The `astro` profile's `site`, `base-path`, and `trailing-slash` repeat settings from `astro.config`. The Astro integration checks that they agree, and fails the build, naming each difference, when they don't. It compares `base-path` as a path with a leading and a trailing `/`, treats Astro's `trailingSlash: "ignore"` as agreeing with either value, and compares `site` by origin when both sides set it.

---

## 16. `[builds.<name>]`

Named builds ([SPEC §9.3]({repo}/blob/main/SPEC.md#93-build-modes)). Each sets a variant mode and an availability mode. These are [SPEC §9.3]({repo}/blob/main/SPEC.md#93-build-modes)'s examples, in `ascribe.toml`:

```toml
[builds.site]
variants = "switch"
availability = "badge"

[builds.cloud-pdf]
variants = { deployment = "cloud" }
availability = { filter = "cloud" }

[builds."sm-3.3"]
variants = "switch"
availability = { filter = "self-managed 3.3" }
```

`<name>` is the build name (§1.2). It names the build on the command line (`ascribe build --build cloud`) and in output paths.

| Key | Type | Default | Description |
|---|---|---|---|
| `variants` | string or table | `"switch"` | The variant mode. `"switch"` keeps every arm and page. A **selection** is a table from dimension names to a value or an array of values: `{ deployment = "cloud" }`, `{ pm = ["npm", "pnpm"] }`. Every dimension must be declared and every value a value of that dimension. An empty table is an error; write `"switch"`. |
| `availability` | string or table | `"badge"` | The availability mode. `"badge"` keeps everything and annotates it. `{ filter = "<target> [<version>]" }` removes content not available for the target at the version. The target must be a declared dimension value, not a dimension name. A versioned target must have a version, and a versionless one must not. The version follows the version scheme (§7). |

**Rules.** Build names must be unique ignoring case (`model-build-name-case`), since they become directory names and some file systems ignore case. A build that filters for a target its own selection excludes (for example, selecting `deployment = "cloud"` and filtering for `self-managed 3.3`) is legal but almost certainly a mistake, so the loader warns (`model-build-filter-excluded`).

---

## 17. `[editor]`

Settings for the authoring environment (SPEC §10).

@snippet: code:examples/content-models/full.toml#editor

| Key | Type | Default | Description |
|---|---|---|---|
| `build` | string | see below | The build whose page-level diagnostics the language server reports by default ([SPEC §8.1]({repo}/blob/main/SPEC.md#81-validation-levels) checks pages once per build). It must name a declared build. |

**Default.** If only one build exists, that build. Otherwise, the build named `site`, if there is one. Otherwise the key is required (`model-editor-build-required`).

---

## 18. `[sources.<name>]`
@available: next

A source is a folder outside the project's content that its pages may take code examples from, with [`@snippet`](directives.md#snippet) ([SPEC §7.3]({repo}/blob/main/SPEC.md#73-sources)). It's the only way a page can read a file outside the project's folder.

```toml
[sources.code]
path = ".."                               # relative to the folder ascribe.toml is in
include = ["crates/**", "examples/**"]    # what's readable; everything else isn't
ignore = ["**/target/**"]
```

A page names a file through its source as `<source>:<path>`, with the path relative to the source's folder: `@snippet: code:examples/quill/ascribe.toml#dimensions`.

| Key | Type | Default | Description |
|---|---|---|---|
| `path` | string (path) | **required** | The source's folder, relative to the project root. It must exist, be a directory, and be inside the git repository the project is in, when the project is in one. |
| `include` | array of patterns | every file | The files a snippet may read, matched against their paths relative to `path` (§1.3). |
| `ignore` | array of patterns | none | Files left out even when `include` matches them. |

`<name>` follows the rules for keys (§1.2). A project can declare several sources.

**Rules.** `path` is relative (`model-path-absolute`); its folder must exist and be a directory (`model-source-path-missing`) inside the project's repository (`model-source-outside-repository`). `git` and `branch` are reserved for a source in another repository, and are errors for now (`model-source-remote`).

---

## 19. Defaults: what an absent section means

A file containing only `spec = "0.1"` is valid. It means:

| Section | When absent |
|---|---|
| `[project]` | `content-root = "docs"`, `output-dir = ".ascribe/build"` |
| `[types]` | One implicit default page type, named `page`, with frontmatter `title = "string"` and nothing else. If `[types]` declares any type, there's no implicit type. |
| `[fragments]` | Only paths with a `_` segment are fragments. The fragment schema has no fields, so a fragment's frontmatter can't have any keys. |
| `[dimensions]` | No dimensions. Any `@variant` arm with attributes, or `variant` frontmatter, is an error. |
| `[versions]` | `scheme = "numeric"` |
| `[lifecycle]` | The five built-in states. |
| `[features]` | No features. |
| `[notes]` | The five built-in note types. |
| `[phrases]` | No phrases. Every `{…}` is literal text. |
| `[glossary]` | No glossary. |
| `[images]` | No image attributes. |
| `[widgets]` | No project widgets. |
| `[consumer]` | `profile = "astro"` with its defaults. |
| `[builds]` | One implicit build, `site`, with `variants = "switch"` and `availability = "badge"`. If `[builds]` declares any build, there's no implicit one. |
| `[editor]` | `build` as in §17. |
| `[sources]` | No sources. A page can't read anything outside the project's folder. |

---

## 20. Validation

A content model with errors is reported, and nothing else is checked, since every other check depends on it. Its warnings are reported with the rest of the diagnostics. Every rule the loader enforces is listed in the [diagnostics reference](diagnostics.md#the-content-model), with its code and how to fix it.
