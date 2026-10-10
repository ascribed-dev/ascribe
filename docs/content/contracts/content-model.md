---
title: "`ascribe.toml` format contract"
description: The normative contract for the content model file, with the rules a loader enforces and the decisions behind them.
---

This is the normative contract for `ascribe.toml`, the content model file (SPEC §7). It defines every table and key, the short syntax for field and attribute types, and every rule a loader enforces. The loader in `ascribe-model` implements it. For a guide to writing one, see the [`ascribe.toml` reference](../reference/content-model.md).

SPEC.md is normative for the language. This document is normative for the file format. Where it settles something the spec leaves open, the item is listed in [Decisions](#23-decisions).

Example files, each valid under this reference:

- [`examples/content-models/minimal.toml`]({repo}/blob/main/examples/content-models/minimal.toml): the smallest valid model.
- [`examples/content-models/quill.toml`]({repo}/blob/main/examples/content-models/quill.toml): the model SPEC Appendix B assumes.
- [`examples/content-models/full.toml`]({repo}/blob/main/examples/content-models/full.toml): every section and key.

## Contents

1. [Conventions](#1-conventions)
2. [Map of SPEC §7.2 to this file](#2-map-of-spec-72-to-this-file)
3. [Top level: `spec`](#3-top-level-spec)
4. [`[project]`](#4-project)
5. [`[types.<name>]` and `[fragments]`](#5-typesname-and-fragments)
6. [Field and attribute types](#6-field-and-attribute-types)
7. [`[dimensions.<name>]`](#7-dimensionsname)
8. [`[versions]`](#8-versions)
9. [`[lifecycle.<state>]`](#9-lifecyclestate)
10. [`[features.<key>]`](#10-featureskey)
11. [`[notes.<type>]`](#11-notestype)
12. [`[phrases]`](#12-phrases)
13. [`[glossary]`](#13-glossary)
14. [`[images]`](#14-images)
15. [`[widgets.<name>]`](#15-widgetsname)
16. [`[consumer]`](#16-consumer)
17. [`[builds.<name>]`](#17-buildsname)
18. [`[editor]`](#18-editor)
19. [`[sources.<name>]`](#19-sourcesname)
20. [`[checks]`](#20-checks)
21. [Defaults: what an absent section means](#21-defaults-what-an-absent-section-means)
22. [Validation rules](#22-validation-rules)
23. [Decisions](#23-decisions)

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
- **Declaration order is kept where it matters.** The loader MUST preserve the order in which attribute keys are declared (`[images.attributes]`, `[widgets.<name>.attributes]`), because canonical form writes attributes in declared order (SPEC §8.3). It MUST also preserve the order of the `[dimensions]` tables, which is the canonical order of `@variant` attributes and decides which dimension a tab group syncs on (the [element contract]({repo}/blob/main/packages/elements/CONTRACT.md), §3). Loaders SHOULD also preserve order elsewhere, for stable output (for example, the order of dimension values in a tab switcher is the order of `values`). Nothing else in the model depends on order.

### 1.2 Names

Several kinds of name appear in the file. Each has a grammar from SPEC Appendix A or from this reference:

| Name | Grammar | Rule | Used for |
|---|---|---|---|
| name-word | SPEC A `name-word` | A letter, then letters, digits, `_`, or `-` | Dimension values, lifecycle states, feature keys, frontmatter field names |
| key | SPEC A `key` | A lowercase letter, then lowercase letters, digits, or `-` | Dimension names, phrase keys, note types, attribute keys, content type names, glossary term ids, source names |
| widget name | SPEC A `widget-name` | Lowercase words of letters and digits joined by single hyphens, with at least one hyphen; starting with a letter | Widget names |
| build name | This reference | A letter, then letters, digits, `_`, `-`, or `.` | Build names |

Dimension names follow the stricter `key` rule, not just `name-word`, because they're also written as attribute keys in `@variant {deployment=cloud}` (SPEC §3.3, §4.3).

Build names allow `.` so that names like `self-managed-3.3` work. A name containing `.` must be quoted in a TOML header: `[builds."self-managed-3.3"]`.

Names are case-sensitive and spelled exactly as declared (SPEC §4.4).

### 1.3 Paths and patterns

- Paths use `/` as the separator on every platform.
- Filesystem paths in `[project]` and `[sources]` are relative to the project root. They MUST NOT be absolute, and they MAY use `..`.
- A glossary term's `link` (§13) is written like a link destination in a document (SPEC §5.2), relative to the content root.
- **Patterns** (globs) are matched against a source file's path relative to the content root, including its extension, with `/` separators. They are case-sensitive. The syntax:

  | Syntax | Matches |
  |---|---|
  | `*` | Any run of characters within one path segment, not including `/` |
  | `**` | Any number of whole segments, including none. It MUST be a whole segment (`a/**/b`, `a/**`, `**/b`) |
  | `?` | Any one character except `/` |
  | `{a,b}` | Either alternative. Alternatives don't nest |
  | `\` | Escapes the next character |

  Every other character matches itself. A pattern MUST NOT start with `/` and MUST NOT contain a `..` segment. Patterns only ever apply to Markdown source files (`*.md`); other files under the content root are assets. The exception is a source's `include` and `ignore` (§19), which are matched against any file's path relative to the source's folder.

### 1.4 How this reference describes keys

Each section has a table of keys with these columns:

- **Type:** the TOML type. "Field type", "attribute type", and "availability spec" are strings with the grammars in §6 and SPEC §4.4.
- **Default:** the value when the key is absent, or **required**.
- **Description.**

---

## 2. Map of SPEC §7.2 to this file

Every declaration SPEC §7.2 lists, and every setting SPEC §9.3 and §9.5 need, maps to one section:

| SPEC declaration | Section of `ascribe.toml` | Here |
|---|---|---|
| Spec version (§11) | `spec` | [§3](#3-top-level-spec) |
| Content root (§2.2); output directory (§9.4) | `[project]` | [§4](#4-project) |
| Content types: page schemas and the fragment schema (§2.1, §2.2) | `[types.<name>]`, `[fragments.frontmatter]` | [§5](#5-typesname-and-fragments) |
| Field roles (§7.2) | The `role` field option | [§6.4](#64-field-roles) |
| Fragment patterns (§2.2) | `[fragments] patterns` | [§5.3](#53-fragments) |
| Field types (§7.2) | Field type syntax | [§6](#6-field-and-attribute-types) |
| Directive schemas for project widgets (§3, §6) | `[widgets.<name>]` | [§15](#15-widgetsname) |
| Dimensions: names, values, display labels, versionless values (§4.3, §4.4) | `[dimensions.<name>]` | [§7](#7-dimensionsname) |
| Version scheme (§4.4) | `[versions]` | [§8](#8-versions) |
| Lifecycle states (§4.4) | `[lifecycle.<state>]` | [§9](#9-lifecyclestate) |
| Features registry (§4.4) | `[features.<key>]` | [§10](#10-featureskey) |
| Note types (§4.5) | `[notes.<type>]` | [§11](#11-notestype) |
| Phrases registry; phrases in frontmatter (§5.1) | `[phrases]`; the `phrases` field option | [§12](#12-phrases) |
| Glossary (§5.4) | `[glossary]` | [§13](#13-glossary) |
| Image attributes (§5.3) | `[images.attributes]` | [§14](#14-images) |
| Consumer profile (§9.5) | `[consumer]` | [§16](#16-consumer) |
| Builds (§9.3) | `[builds.<name>]` | [§17](#17-buildsname) |
| Name roles (§7.2, last paragraph) | Validation rule `model-name-multiple-roles` | [§22](#22-validation-rules) |
| The editor's default build | `[editor]` | [§18](#18-editor) |
| Sources: the folders outside the content a snippet may read (§4.8, §7.3) | `[sources.<name>]` | [§19](#19-sourcesname) |
| Acknowledgements of a content model entry's or an image's problems (§4.9) | `[[intended]]` | [§20.1](#201-intended) |

---

## 3. Top level: `spec`

```toml
spec = "0.1"
```

| Key | Type | Default | Description |
|---|---|---|---|
| `spec` | string | **required** | The version of the Ascribe specification this project targets (SPEC §11). It MUST be quoted: `spec = 0.1` is a TOML float and is an error. A processor accepts only the spec versions it implements, compared as exact strings; this reference defines `"0.1"`. |

The only other top-level keys are the tables in §4–§20. Anything else is an unknown key.

---

## 4. `[project]`

Where the source lives and where builds write.

@snippet: code:examples/content-models/full.toml#project

| Key | Type | Default | Description |
|---|---|---|---|
| `content-root` | string (path) | `"docs"` | The content root (SPEC §2.2): the directory holding every source file. Paths in links and includes that begin with `/` are relative to it. It MUST exist and be a directory. |
| `output-dir` | string (path) | `".ascribe/build"` | Where `ascribe build` writes output. Each build and emitter writes under `<output-dir>/<build>/<emitter>/` (see the [output-layout contract](output-layout.md)). It need not exist. |

**Rules** (§22): both paths are relative (`model-path-absolute`). The output directory MUST NOT be inside the content root, the content root MUST NOT be inside the output directory, and they MUST NOT be the same directory (`model-output-overlaps-content`); otherwise a build would read its own output as source, or delete source as stale output. Paths are compared after normalizing `.` and `..` segments and, when both exist, after resolving symbolic links.

The defaults: `".ascribe/build"` keeps generated output out of the way of both the source and a consumer's own `dist/`.

---

## 5. `[types.<name>]` and `[fragments]`

### 5.1 Page types

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
| `frontmatter` | table of fields | **required** | The frontmatter schema: one entry per field, keyed by field name (`name-word`). Values are field types (§6). It MUST declare `title` as a required string (below). |

**Which type applies to a page.** For each page (not fragments; see §5.3):

1. If exactly one type's `files` match the page, that type applies.
2. If several types' `files` match, it's an error on the page: the page is ambiguous. There's no precedence between types.
3. If none match and a default type exists, the default applies.
4. If none match and there's no default type, it's an error on the page.

A type with neither `files` nor `default = true` could never apply, and is an error (`model-type-unreachable`). The rules in steps 2 and 4 are document diagnostics, listed in SPEC §8.2.

**The page title.** Every page type MUST declare `title` as a required `string` field (`model-type-title`). The frontmatter `title` is the page's title wherever the spec needs one, such as the replacement text of an empty link to a page (SPEC §5.2). The field MAY accept phrases (§12), and MAY set `inline = "code"` for code spans in the title (§6.3).

**Reserved keys.** `available` (SPEC §4.4) and `variant` (SPEC §4.3) are reserved frontmatter keys. Every page accepts them, with the meaning the spec gives, whether or not its type mentions them, and a type MUST NOT declare them (`model-field-reserved`). Generated consumer schemas include them automatically. `intended` (SPEC §4.9) is reserved too, and every page and every fragment accepts it; no output has it, so generated consumer schemas leave it out, and a type MUST NOT declare it (`model-field-reserved`). Under the `astro` profile, `slug` is reserved too: Astro's content loader uses a page's frontmatter `slug` as its entry id in place of its path, which would publish the page at a URL Ascribe never computed, so a type MUST NOT declare it (`model-field-reserved`). `formatted` is reserved on page types as well: the site output writes the formatted form of `inline` fields under it (§6.3), so a type MUST NOT declare it (`model-field-reserved`). A page's own frontmatter can't use it either, since no type declares it.

**Unknown frontmatter keys.** A page whose frontmatter has a key its type doesn't declare (and that isn't reserved) is an error on the page.

### 5.2 Frontmatter values

Frontmatter is YAML. For type checking, processors parse it with the YAML 1.2 **core schema**: `true` and `false` are booleans (`yes`, `no`, `on`, and `off` are strings), and `3.10` is the number 3.1. So a `string` field whose value is `3.10` unquoted is a type error, and the message suggests quoting it.

### 5.3 Fragments

@snippet: code:examples/content-models/full.toml#fragments

| Key | Type | Default | Description |
|---|---|---|---|
| `patterns` | array of patterns | `[]` | Additional fragment patterns (SPEC §2.2). A file is a fragment if any segment of its path begins with `_`, or its path matches one of these patterns. |
| `frontmatter` | table of fields | `{}` (no fields) | The fragment schema (SPEC §2.2): the frontmatter every fragment is validated against. Fields as in §6. `title` is not required. |

Content types never apply to fragments, even when a type's `files` match a fragment's path.

**Reserved keys on fragments.** The spec defines `available` and `variant` for pages only. A fragment's frontmatter MUST NOT use them (an error on the fragment), and `[fragments.frontmatter]` MUST NOT declare them (`model-field-reserved`). Use `@available` inside the fragment instead. A fragment MAY use `intended`, and `[fragments.frontmatter]` MUST NOT declare it (`model-field-reserved`).

---

## 6. Field and attribute types

Two related type languages:

- **Field types** describe frontmatter fields (`[types.<name>.frontmatter]`, `[fragments.frontmatter]`). They cover the SPEC §7.2 set: string, number, boolean, date, enumeration, list, and object, each optional or with a default.
- **Attribute types** describe attributes on images and widgets (`[images.attributes]`, `[widgets.<name>.attributes]`). They cover SPEC §3.3's set: string, enumeration, boolean, and number, plus set-valued keys.

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

### 6.1 Short form

A field is **required** unless its type ends in `?` or it has a default. The same holds for attributes: `lab = "string"` means every use of the widget must give `lab`.

| Short form | Field types | Attribute types | Accepts |
|---|---|---|---|
| `string` | yes | yes | Frontmatter: a YAML string. Attribute: a token or quoted string. |
| `number` | yes | yes | Frontmatter: a YAML integer or float. Attribute: a token matching `["-"] 1*DIGIT ["." 1*DIGIT]` (`600`, `-2`, `1.5`; not `600px`). |
| `boolean` | yes | yes | Frontmatter: YAML `true` or `false`. Attribute: the token `true` or `false` (SPEC §3.3). |
| `date` | yes | no | A calendar date written `YYYY-MM-DD` that exists (`2026-02-30` is an error), quoted or not. Times aren't accepted. |
| `enum(a, b, …)` | yes | yes | One of the listed values, compared exactly. Frontmatter: a YAML string. Attribute: a token or quoted string. |
| `list(T)` | yes | no | A YAML sequence whose items are all of type `T`, where `T` is `string`, `number`, `boolean`, `date`, `enum(…)`, or (table form only) `object`. An empty sequence is allowed. |
| `set(T)` | no | yes | A value set (SPEC §3.3), such as `platform=cloud\|on-prem`, or a single token. `T` is `string` or `enum(…)`. Members are tokens. Only keys typed `set(…)` accept `\|`. |
| `object` | table form only | no | A YAML mapping whose keys are the object's `fields` (nested field types). Unknown keys are errors. |
| any of the above + `?` | yes | yes | The same type, optional. |

Grammar (ABNF, with the rules of SPEC Appendix A):

@include: ../_fragments/field-type-grammar.md

### 6.2 Table form

| Key | Type | Default | Description |
|---|---|---|---|
| `type` | string | **required** | A field type or attribute type (§6.1). In table form it may also be `object`, `list(object)`, or bare `enum`, with the keys below. |
| `fields` | table of fields | required when `type` is `object`, `object?`, `list(object)`, or `list(object)?`; not allowed otherwise | The nested fields of an object, in the same syntax (short or table form). Field types only. |
| `values` | array of strings | required when `type` uses bare `enum`; not allowed otherwise | The enumeration's values, for values that aren't simple words, such as `["Getting started", "How-to"]`. Values MUST be distinct. Values of a `set(enum)` attribute MUST be tokens. |
| `default` | any TOML value | none | The value used when the field or attribute is absent. It MUST have the declared type (a TOML string for `string` and `enum`, integer or float for `number`, boolean for `boolean`, local date for `date`, array for `list`, inline table for `object`, a string or array of strings for `set`). A field with a default is optional; adding `?` as well is allowed and changes nothing. |
| `phrases` | boolean | `false` | Whether phrases (SPEC §5.1) are substituted in this field's value. Field types only; allowed only on `string` and `list(string)` fields (and their optional forms), including fields nested in objects. See §12. |
| `inline` | string | none | The inline markup the value is read with. The one value is `"code"`: code spans. Allowed only on a content type's top-level `string` fields (and `string?`), not in objects or the fragment schema. See §6.3. |
| `description` | string | none | Help text shown by the editor (hover and completion) and emitted into generated schemas as documentation. |
| `role` | string | none | What the field is for, for the checks that read it (SPEC §7.2): `"description"` or `"review-date"`. Allowed only on a content type's top-level fields, not in objects or the fragment schema. See §6.4. |

Nesting beyond one level of `fields` is allowed but discouraged; keep frontmatter flat.

### 6.3 Inline code in a field

A `string` field is plain text: a backtick in it is a backtick. A field that sets `inline = "code"` is read as inline Markdown that knows only code spans, so a page title can show code:

```toml
[types.reference.frontmatter]
title = { type = "string", inline = "code" }
```

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

### 6.4 Field roles
@available: next

A page type's top-level field MAY have a `role`, which says what the field is for, so a check reads it whatever the field's name (SPEC §7.2):

| Role | The field's type | Read by |
|---|---|---|
| `description` | `string` or `string?` | `page-description-missing`, which reports a page of the type whose field is absent, null, or blank. A field that's required or has a default is never absent, so the check skips it. |
| `review-date` | `date` or `date?` | `review-overdue`, which reports a page whose date is before the day the check runs. |

A type without a field with the role has none of that check's diagnostics. A role changes no output: not the generated schemas, the JSON, or the pages.

**Rules** (§22.3): the role is one of the two, on a field of its type, in a content type's own fields, and at most one field of a type has each role (`model-field-role`). A second field with a role keeps its type, and loses the role.

---

## 7. `[dimensions.<name>]`

A **dimension** is an axis content varies along (SPEC §4.3), and whose values are availability targets (SPEC §4.4).

@snippet: code:examples/content-models/full.toml#deployment

`<name>` is the dimension's name (`key` rule, §1.2). It's written as an attribute key in `@variant`, as a key in `variant` frontmatter and build selections, and as a target in availability specs, where it stands for all its values.

| Key | Type | Default | Description |
|---|---|---|---|
| `values` | array of strings | **required** | The dimension's values (`name-word`), in display order. At least one; no duplicates. |
| `labels` | table of strings | `{}` | Display labels for values, keyed by value (SPEC §9.4: "Labels for dimension values come from the content model's display labels"). A value without a label is displayed as the value itself. Every key MUST be a declared value. |
| `label` | string | the dimension's name | The display label for the dimension itself, used where a dimension name appears as an availability target or names a tab group. |
| `versionless` | array of strings | `[]` | Values that are versionless (SPEC §4.4): availability for them takes a single state and no versions. Every entry MUST be a declared value. Values not listed are versioned. |

**Rules.** A value MUST belong to only one dimension (`model-dimension-value-shared`); otherwise a bare target in an availability spec would be ambiguous. Dimension names and values also take part in the one-role rule (§22).

---

## 8. `[versions]`

How versions in availability specs are compared (SPEC §4.4).

@snippet: code:examples/content-models/full.toml#versions

| Key | Type | Default | Description |
|---|---|---|---|
| `scheme` | string | `"numeric"` | The version scheme. Spec 0.1 defines one: `"numeric"`. |

**The `numeric` scheme.** A version is any string matching SPEC Appendix A's `version` rule: numbers separated by dots (`3`, `3.4`, `3.4.1`). Versions compare component by component, numerically, from the left, with missing trailing components treated as `0`: `3.4` equals `3.4.0`, `3.10` is later than `3.9`, and `4` is later than `3.99.1`. Leading zeros don't matter (`3.04` equals `3.4`). This is semantic versioning's `major.minor.patch` ordering. It has no pre-release or build suffixes, because the spec's grammar doesn't allow them; express a pre-release with a lifecycle state (`preview 3.4`) instead.

No other scheme is defined. The table exists so a later spec version can add one without changing the file's shape. The scheme is named `numeric`, not "semantic versioning", because it accepts any number of components and has no pre-release syntax.

---

## 9. `[lifecycle.<state>]`

Lifecycle states (SPEC §4.4). Five are built in:

| State | Counts as available | Default label |
|---|---|---|
| `preview` | yes | `preview` |
| `beta` | yes | `beta` |
| `ga` | yes | `GA` |
| `deprecated` | yes | `deprecated` |
| `removed` | no | `removed` |

A project adds states by declaring them, and MAY change the `available` flag or `label` of a built-in state. Built-in states can't be removed.

@snippet: code:examples/content-models/full.toml#lifecycle

`<state>` is the state's name (`name-word`).

| Key | Type | Default | Description |
|---|---|---|---|
| `available` | boolean | built-in states: as in the table above; new states: **required** | Whether content in this state counts as available (SPEC §4.4, §9.3). A new state must say so explicitly. |
| `label` | string | built-in states: as in the table above; new states: the state's name | The display label used in availability annotations, such as "Available: Quill Cloud (GA); self-managed (preview, 3.4+)" (SPEC §9.4). |

**Rules.** `ga` MUST count as available (`model-lifecycle-ga-unavailable`), because content with no state is `ga` (SPEC §4.4). Lifecycle states take part in the one-role rule, including the built-in ones: a dimension value named `beta` is an error.

---

## 10. `[features.<key>]`

The features registry (SPEC §4.4): named availability specs, so a feature going generally available takes one edit.

@snippet: code:examples/content-models/full.toml#features

`<key>` is the feature key (`name-word`). Writing it as a whole `@available` primary, or as the whole `available` frontmatter value, stands for the spec.

| Key | Type | Default | Description |
|---|---|---|---|
| `name` | string | **required** | The feature's display name, shown on hover and available to emitters. |
| `available` | string (availability spec) | **required** | The feature's availability, in SPEC §4.4's syntax. |

**Rules.** The spec is checked like one written in a document: it MUST parse (`model-availability-syntax`); every target MUST be a declared dimension value or dimension name, and every state a declared state (`model-availability-unknown-name`); versionless targets and dimension names take no versions (`model-availability-versionless`); histories MUST be in chronological order (`model-availability-history-order`). It MUST NOT be a feature key itself (`model-feature-nested`), so features never refer to each other. Feature keys take part in the one-role rule.

---

## 11. `[notes.<type>]`

Note types (SPEC §4.5), the values of `@note`'s `type` attribute. Five are built in: `note`, `tip`, `important`, `warning`, and `caution`. A project adds types by declaring them, and MAY relabel a built-in type. Built-in types can't be removed.

@snippet: code:examples/content-models/full.toml#notes

`<type>` is the note type (`key` rule).

| Key | Type | Default | Description |
|---|---|---|---|
| `label` | string | built-in types: `Note`, `Tip`, `Important`, `Warning`, `Caution`; new types: **required** | The display label, used where a note's type is shown as text, such as the plain-markdown output's `**Tip: …**` (SPEC §9.4). |

Under `[notes]`, the inline form `security = { label = "Security" }` is equivalent.

---

## 12. `[phrases]`

The phrases registry (SPEC §5.1).

@snippet: code:examples/content-models/full.toml#phrases

Every key is a phrase key (`key` rule, the same rule as SPEC Appendix A's `phrase`), and every value MUST be a TOML string (`model-phrase-value-type`). `version = 3.4` is a float and an error; that's the kind of silent type change TOML exists to prevent (SPEC §7.1). Values are inserted as literal text (SPEC §5.1).

**Phrases in frontmatter.** SPEC §5.1 lets the content model decide which frontmatter fields accept phrases. A field accepts them when its table form sets `phrases = true` (§6.2):

```toml
[types.guide.frontmatter]
title = { type = "string", phrases = true }
```

Only `string` and `list(string)` fields can accept phrases (`model-phrases-field-type`). By default, no field does. The setting lives on the field, not in `[phrases]`, so that a phrase key can never collide with a setting's name.

---

## 13. `[glossary]`

The glossary (SPEC §5.4): terms, their definitions, and how occurrences are matched. Authors don't mark terms in source; processors link them (SPEC §9.2 step 7).

@snippet: code:examples/content-models/full.toml#glossary

### 13.1 Settings

| Key | Type | Default | Description |
|---|---|---|---|
| `match` | string | `"first"` | Which occurrences are linked: `"first"`, the first occurrence of each term on each page, or `"every"`, every occurrence. "Page" means the resolved page for a build, after includes and build modes. A term can override it. |
| `case-sensitive` | boolean | `false` | Whether occurrences must match a term's case exactly. When `false`, matching ignores case. A term can override it. |
| `terms` | table of terms | `{}` | The terms, keyed by term id (`key` rule). |

### 13.2 Terms

| Key | Type | Default | Description |
|---|---|---|---|
| `term` | string | **required** | The term as it appears in prose. Non-empty. |
| `aliases` | array of strings | `[]` | Other forms that count as occurrences, such as plurals. |
| `definition` | string | **required** | A short plain-text definition, shown on hover in the editor and included in the JSON output. |
| `link` | string (path) | none | Where the full definition lives: a source file path relative to the content root, with an optional `#id`, written as in a link (SPEC §5.2; a leading `/` is allowed and means the same). It MUST name a page, not a fragment. Occurrences are linked here. |
| `case-sensitive` | boolean | the `[glossary]` setting | Overrides `case-sensitive` for this term, for terms like `Go` that collide with ordinary words. |
| `match` | string | the `[glossary]` setting | Overrides `match` for this term: `"first"`, `"every"`, or `"marked"`. A `"marked"` term isn't linked automatically at all, for words like `build` that also have ordinary senses: link the occurrences that mean the term with an ordinary link to its page. `"marked"` is a term setting only; `[glossary]` doesn't accept it. |

**Matching**: occurrences match whole words only; the longest matching term wins where terms overlap (`API key` over `API`); matching applies to prose only, never to headings, link text, code, raw HTML, or text inside a directive's primary identifier. A term with no `link` isn't linked in the site or plain-markdown output; its definition still reaches the editor and the JSON output. SPEC §5.4 says only that occurrences are linked to definitions.

**Rules.** No two terms or aliases may be the same text (compared ignoring case when either is case-insensitive; `model-glossary-duplicate-term`). The `link` file MUST exist and not be a fragment (`model-glossary-link`); its `#id` is checked with the page-level link checks, since ids depend on parsing.

---

## 14. `[images]`

Image attributes (SPEC §5.3): which keys may appear in an attribute block after an image, and their types.

@snippet: code:examples/content-models/full.toml#images

| Key | Type | Default | Description |
|---|---|---|---|
| `attributes` | table of attribute types | `{}` | Accepted image attribute keys (`key` rule) and their types (§6). With no entries, an image accepts no attributes, and any attribute block after an image is an error (SPEC §8.2, "Unknown key"). Declaration order is canonical order (SPEC §8.3). A declared default reaches the output: in the site output, every image carries the defaults of attributes it doesn't write. |

Alt text and titles aren't attributes; they use CommonMark's syntax (SPEC §5.3). Presentation choices aren't image attributes either. Keys HTML already uses on `<img>` (`src`, `alt`, `title`, and its global attributes) are rejected (`model-attribute-reserved`), since the site output writes image attributes onto the `<img>` element (SPEC §7.2).

---

## 15. `[widgets.<name>]`

Project widgets (SPEC §6): directives a documentation set defines. Each declaration is a directive schema with the same parts a built-in directive's schema has (SPEC §3, §4).

@snippet: code:examples/content-models/full.toml#widgets

`<name>` is the widget's name (SPEC A `widget-name`: lowercase, with at least one hyphen). Names starting with `ascribe-` are reserved for Ascribe's element library, and the names HTML reserves for itself (`annotation-xml`, `color-profile`, `font-face`, `font-face-src`, `font-face-uri`, `font-face-format`, `font-face-name`, `missing-glyph`) aren't allowed, because the site output emits a widget as a custom element with the widget's name (SPEC §9.4).

| Key | Type | Default | Description |
|---|---|---|---|
| `forms` | array of strings | **required** | The permitted forms (SPEC §3.5): `["line"]`, `["container"]`, or both. |
| `primary` | string | `"none"` | The primary (SPEC §3.4): `"none"`; `"identifier"` or `"text"`, which are required; or `"identifier?"` or `"text?"`, which are optional. |
| `binding` | string | required when `forms` includes `"line"`; not allowed otherwise | What the line form applies to (SPEC §3.8), one of the values below. |
| `title` | string | `"none"` | Whether the widget takes a title line (SPEC §3.7): `"none"`, `"accepted"`, or `"required"`. |
| `groupable` | boolean | `false` | Whether a run of the widget's openers forms a group of arms (SPEC §3.6). |
| `attributes` | table of attribute types | `{}` | The attribute schema: keys (`key` rule) and types (§6). Declaration order is canonical order (SPEC §8.3). The keys `heading` and `primary`, HTML's global attributes (including `title`), `aria-` keys, and event-handler attributes such as `onclick`, are rejected (`model-attribute-reserved`): the site output writes these attributes onto the widget's element (SPEC §7.2). |
| `plain-fallback` | string | none | Plain-text fallback for the plain-markdown output (SPEC §6, §9.4): CommonMark text written in place of the widget. Phrases in it are substituted. It isn't a template: attribute values aren't inserted. Without it, the widget itself emits nothing. |
| `plain-content` | string | `"keep"` | For a widget that wraps content, whether the plain-markdown output keeps that content (`"keep"`) after the fallback, or drops it (`"drop"`). Allowed only when the widget wraps content: it has container form, or its binding is `block` or `heading-or-block`. |
| `description` | string | none | Help text for the editor's hover and completion. |

**Binding values:**

| Value | SPEC §3.8 binding | Behaves like |
|---|---|---|
| `"self"` | Self: its own primary, or nothing | `@include` |
| `"heading"` | Preceding heading: at the top of its section | `@id` |
| `"block"` | Following block. If the primary is a text primary and it's given, the primary is the content and nothing else is bound | `@steps`; with `primary = "text?"`, `@note` |
| `"heading-or-block"` | At the top of a section, the section; anywhere else, the following block | `@available` |

**Rules** (§22):

- A widget with container form MUST NOT have a required primary, since a container opener's primary is empty (SPEC §3.5). A container-only widget's primary MUST be `"none"` (`model-widget-container-primary`).
- A groupable widget MUST be container-only, since group arms are containers (SPEC §3.6) (`model-widget-groupable-form`).
- `binding` is required with line form and not allowed without it (`model-widget-binding`).

The site output's element for a widget (tag name and attributes) is defined by the [element contract]({repo}/blob/main/packages/elements/CONTRACT.md), not here.

---

## 16. `[consumer]`

The consumer profile (SPEC §9.5): how the site output fits a specific consumer. Spec 0.1's toolchain implements one profile, `astro` (SPEC §9.6). The profile sets defaults for every other key; a key given here overrides them, within the values the profile supports.

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
| `site` | string (URL) | none | The published site's origin, such as `"https://docs.quill.dev"`: an `http` or `https` URL with no path, query, or fragment. The plain-markdown output needs it to write absolute links (SPEC §9.4). Without it, plain-markdown links are root-relative (they start with `base-path`) and `ascribe build` warns. |
| `base-path` | string | `"/"` | **Routing.** The URL path every route starts with, such as `"/docs/"`, including any locale prefix (`"/en/"`). It MUST start with `/`. A trailing `/` is optional and doesn't change the meaning. |
| `trailing-slash` | string | `"always"` | **Routing.** Whether page URLs end in `/`: `"always"` (`/guides/setup/`) or `"never"` (`/guides/setup`). Match the consumer's own setting (Astro's `trailingSlash` and `build.format`). |
| `slugger` | string | `"github"` | **Slugging.** The algorithm for heading ids (SPEC §5.5), which must be the one the consumer uses. Supported: `"github"`, a port of `github-slugger`, which Astro uses. |
| `html` | boolean | `true` | **HTML passthrough.** Whether the consumer renders raw HTML in markdown. The site output's custom elements need it, so the `astro` profile supports only `true`. |
| `agents` {available=next} | boolean | `false` | **Outputs for agents.** Whether each build also publishes what AI agents read, following the Web Documentation Delivery Spec 0.6.0: `llms.txt` and each page's Markdown version, in the plain-markdown output laid out by URL, and a pointer to them on each page of the site output. It requires `site`. See below. |

**Heading ids, image attributes, and assets** (SPEC §9.5) are part of the profile, not keys. The `astro` profile has exactly one way to do each, defined by the contracts:

- **Heading ids and image attributes:** the site output writes each as a `<ascribe-attributes>` marker that the consumer's markdown plugin applies, so the consumer keeps its own heading, table-of-contents, and image processing. See [`contracts/site-render.md`](site-render.md).
- **Assets:** copies mirror their source paths inside each output, and images are referenced relatively so Astro's image processing still applies. Other files a page links to are published under `_ascribe/files/`. See [`contracts/assets.md`](assets.md).

A later profile that offers a choice adds a key for it then.

**How file paths become routes** (the `astro` profile, verified against Astro): a page's route is `base-path`, then its path relative to the content root with the `.md` extension removed and each segment slugged the way Astro's content loader computes entry ids; a final `index` segment is dropped (`guides/index.md` → `/guides/`); then the trailing slash per `trailing-slash`. The root `index.md` (Astro's entry id `index`) is at `base-path`, which under `trailing-slash = "never"` loses its final `/` unless it's `/`. Two pages with one entry id (`My File.md` and `my-file.md`, or `index.md` and `index/index.md`) can't both be published, and `ascribe build --emit site` fails, naming them. The same router answers the reverse question, which page a route-like link names, for the `link-route` warning and its fix. Source files never contain routes (SPEC §5.2).

**The outputs for agents** (`agents = true`; decision 64). The plain-markdown output becomes what a site publishes for agents, laid out so that copying it to `base-path` publishes it (contracts/output-layout.md):

- Each page's file is its route's path with `.md`: Astro's entry id with `.md` added (`Guides/My Setup.md` → `guides/my-setup.md`, `guides/index.md` → `guides.md`, the root `index.md` → `index.md`). It opens with the blockquote `> For the complete documentation index, see [llms.txt](<url>).` and a blank line, `<url>` being `llms.txt`'s absolute URL.
- Files the pages link to are copied under `_ascribe/files/`, mirroring their source paths, and linked by absolute URL; the manifest gives each its root-relative `url`.
- `llms.txt` is the index of every page the build publishes, in page order: `# <name>`, a blank line, `> <summary>`, then a section per folder, each `## <heading>` followed by a line per page, `- [<title>](<url>)` or `- [<title>](<url>): <description>`, with `<url>` the absolute URL of the page's Markdown. `<name>` and `<summary>` are the title and description of the page whose entry id is `index`; without one, the site's host and a fixed sentence. A page's description is the field its type marks `role = "description"` (§6.4), as plain text on one line (its Markdown's text: a link is its text). Pages directly in the content root are the section "Pages", first, with the `index` page first among them; the rest are grouped by their entry id's first segment, in order of first appearance, each headed by the title of the folder's index page (entry id equal to the segment), which is listed first, or else the folder's name with `-` and `_` as spaces and its first letter capitalized.
- When `llms.txt` would be longer than 50,000 characters, it's split: the root file keeps the header and "Pages", and adds `## Sections` with a line per folder linking to `<segment>/llms.txt`, described by the folder's index page's description or "N pages". Each `<segment>/llms.txt` is a whole index of its own: the section's heading as its name, the folder's index page's description (or a sentence naming the site and the section) as its summary, and the section. A file still over the limit is written, and `ascribe check` reports it (`llms-section-large`).

Without `site` the model is invalid (`model-consumer-agents-site`), since every URL above is absolute.

The `astro` profile's `site`, `base-path`, and `trailing-slash` repeat settings from `astro.config`. The Astro integration checks that they agree, and fails the build, naming each difference, when they don't. It compares `base-path` as a path with a leading and a trailing `/`, treats Astro's `trailingSlash: "ignore"` as agreeing with either value, and compares `site` by origin when both sides set it.

---

## 17. `[builds.<name>]`

Named builds (SPEC §9.3). Each sets a variant mode and an availability mode. These are SPEC §9.3's examples, in `ascribe.toml`:

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
| `variants` | string or table | `"switch"` | The variant mode. `"switch"` keeps every arm and page. A **selection** is a table from dimension names to a value or an array of values: `{ deployment = "cloud" }`, `{ pm = ["npm", "pnpm"] }`. Every dimension MUST be declared and every value a value of that dimension. An empty table is an error; write `"switch"`. |
| `availability` | string or table | `"badge"` | The availability mode. `"badge"` keeps everything and annotates it. `{ filter = "<target> [<version>]" }` removes content not available for the target at the version. The target MUST be a declared dimension value, not a dimension name. A versioned target MUST have a version, and a versionless one MUST NOT. The version follows the version scheme (§8). |

**Rules.** Build names MUST be unique ignoring case (`model-build-name-case`), since they become directory names and some file systems ignore case. A build that filters for a target its own selection excludes (for example, selecting `deployment = "cloud"` and filtering for `self-managed 3.3`) is legal but almost certainly a mistake, so the loader warns (`model-build-filter-excluded`).

---

## 18. `[editor]`

Settings for the authoring environment (SPEC §10).

@snippet: code:examples/content-models/full.toml#editor

| Key | Type | Default | Description |
|---|---|---|---|
| `build` | string | see below | The build whose page-level diagnostics the language server reports by default (SPEC §8.1 checks pages once per build). It MUST name a declared build. |

**Default.** If only one build exists, that build. Otherwise, the build named `site`, if there is one. Otherwise the key is required (`model-editor-build-required`).

---

## 19. `[sources.<name>]`
@available: next

A source is a folder outside the project's content, or another repository, that its pages may take code examples from (SPEC §4.8, §7.3, §7.4). It's the only way a page can read a file outside the project's folder.

```toml
[sources.code]
path = ".."                               # relative to the folder ascribe.toml is in
include = ["crates/**", "examples/**"]    # what's readable; everything else isn't
ignore = ["**/target/**"]

[sources.api]
git = "https://github.com/acme/api.git"
branch = "main"
include = ["src/**"]
```

`<name>` is the source's name (§1.2), which a snippet's address starts with: `code:examples/quill/ascribe.toml#dimensions`.

| Key | Type | Default | Description |
|---|---|---|---|
| `path` | string (path) | **required** without `git` | The source's folder, relative to the project root. It MUST exist and be a directory, and, when the project is in a git repository, it MUST be inside that repository. |
| `git` | string (URL) | **required** without `path` | Another repository: `https://`, `http://`, `ssh://`, `git://`, `file://`, or `user@host:path`. Its files are read from copies in `sources/<name>/`, beside `ascribe.toml`. |
| `branch` | string | the repository's default branch | The branch `ascribe sources update` moves the pin to. Only with `git`. |
| `include` | array of patterns | every file | The files a snippet may read, matched against their paths relative to the source's folder (§1.3). |
| `ignore` | array of patterns | none | Files left out even when `include` matches them. |

A source has `path` or `git`, never both.

### 19.1 `ascribe.lock` and the copies

A source with `git` is pinned to a commit in `ascribe.lock`, beside `ascribe.toml`, and the files its snippets use are copied into `sources/<name>/` at the paths they have in their repository. Both are written by `ascribe sources fetch` and `ascribe sources update`, and committed. `check` and `build` read only the copies.

```toml
# Written by `ascribe sources fetch` and `ascribe sources update`. Don't edit it by hand.
version = 1

[[source]]
name = "api"
git = "https://github.com/acme/api.git"
commit = "9f2c41d0e0c4a1b2c3d4e5f60718293a4b5c6d7e"
missing = ["examples/quickstart.ts"]

[source.files]
"src/auth.rs" = "sha256:5d41402abc4b2a76b9719d911017c592ae2fd2b1f6b3b0f0d5ce64b4c7d0e1a2"
```

| Key | Type | Description |
|---|---|---|
| `version` | integer | The lock's format: `1`. |
| `source[].name` | string | The source's name. Each source is pinned once. |
| `source[].git` | string | The source's `git` when it was pinned. |
| `source[].commit` | string | The commit, in full: 40 or 64 hexadecimal digits. |
| `source[].files` | table | Each copy's path, relative to the source's folder, and the SHA-256 of its bytes: `sha256:` and 64 lowercase hexadecimal digits. |
| `source[].missing` | array of strings | Optional. The paths, relative to the source's folder, that snippets named and that weren't in the repository at the commit, so weren't copied. No path is in both `files` and `missing`. |

`check` reads the lock and the copies with the source files, and reports what doesn't match: a lock it can't read (`lock-invalid`); a pin of a source `ascribe.toml` doesn't declare with `git`, or declares with another `git` (`lock-source-unknown`); a copy that's missing or doesn't have its hash (`source-copy-changed`); a file in `sources/<name>/` the lock doesn't list (`source-copy-unlocked`); and a copy no snippet uses (`source-copy-unused`, a warning). A snippet that names a file with no copy is `snippet-file-missing`: its message says to run `ascribe sources fetch`, or, for a file the lock lists in `missing`, that the file isn't in the repository at the pinned commit, which `fetch` can't change. These are source-file diagnostics; see [the diagnostics reference](../reference/diagnostics.md).

**Rules** (§22.8): `path` is relative (`model-path-absolute`), and names an existing directory (`model-source-path-missing`) inside the project's repository (`model-source-outside-repository`). A source has exactly one of `path` and `git`, `branch` only with `git`, a URL and a branch name `git` accepts, and copies outside the content root (`model-source-remote`).

---

## 20. `[checks]`
@available: next

How loudly each check about the content's quality speaks. A diagnostic about whether the project is valid can't be named here: its severity is the registry's. The two forms a key's value takes are:

```toml
[checks]
code-language-missing = "warning"     # a level

[checks.page-size]                    # or a table: the level, and the check's own settings
level = "off"
limit = 40_000
```

| Key | Type | Default | Description |
|---|---|---|---|
| `<check>` | string or table | the check's own level | A check's slug, as the [diagnostics reference](../reference/diagnostics.md) lists it. The value is a level: `"off"` (not reported), `"advice"`, `"warning"`, or `"error"`. Or it's a table with an optional `level` and the check's own settings, which the reference lists with the check; a table without `level` keeps the check's own level. |

Only a check the diagnostics registry marks configurable can be named; the [diagnostics reference](../reference/diagnostics.md#content-checks) lists them. The settings a check's table takes:

| Check | Setting | Type | Default | Description |
|---|---|---|---|---|
| `page-size` | `limit` | integer | `50000` | The size, in characters of a page's plain Markdown in a build, at which the page is reported. It MUST be above 0. |
| `image-large` | `limit` | integer or string | `"500 KB"` | The size over which an image file is reported: a whole number of bytes, or a number followed by `B`, `KB`, or `MB` (a kilobyte is 1,000 bytes, a megabyte 1,000,000), with or without a space. |

The names `vale` and `links` are reserved: they're tables of settings for the tools Ascribe runs, not checks, and no check has either name.

A level changes how a diagnostic is reported everywhere: by `ascribe check` and `ascribe build`, and in the editor. Advice is shown, and never fails `ascribe check`, even with `--deny-warnings`; a check set to `off` isn't reported at all.

**Rules** (§22.9): every key is a check's slug, with a did-you-mean suggestion for one that isn't (`model-unknown-key`); the check is configurable (`model-check-not-configurable`); a level is one of the four (`model-invalid-value`); a table has only `level` and the check's settings (`model-unknown-key`); a value is a string or a table, `page-size`'s `limit` a whole number above 0, and `image-large`'s a size (`model-wrong-type`).

### 20.1 `[[intended]]`
@available: next

Acknowledges a review check's problem with a content model entry or an image as intended (SPEC §4.9): the problem isn't reported, and is counted and listed with the reason instead. Each table is one acknowledgement.

```toml
[[intended]]
check = "phrase-unused"
phrase = "old-product-name"
reason = "Kept for the 2.x pages restored in the next release."
```

| Key | Type | Default | Description |
|---|---|---|---|
| `check` | string | required | A check's slug. The registry gives it `next = "review"` and `place = "entry"`. |
| `reason` | string | required | Why the problem is intended. Not empty. |
| `phrase` | string | | A phrase's key, declared in `[phrases]`. |
| `feature` | string | | A feature's key, declared in `[features]`. |
| `term` | string | | A glossary term's id, declared in `[glossary]`. |
| `image` | string | | An image's path under the content root. |

Exactly one of `phrase`, `feature`, `term`, and `image`. The acknowledgement covers the check's problems that name that entry, in every build. One that covers nothing when every build is checked is advice (`intended-unused`), reported at its `[[intended]]` header with a fix that removes the table.

**Rules** (§22.9): the check exists, with a did-you-mean suggestion for one that doesn't, is a review check, and is reported at an entry (`model-intended-check`); the table names exactly one declared entry (`model-intended-entry`); `check` and `reason` are present (`model-missing-key`) and `reason` isn't empty (`model-empty-text`); and the table has no other key (`model-unknown-key`).

---

## 21. Defaults: what an absent section means

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
| `[editor]` | `build` as in §18. |
| `[sources]` | No sources. Nothing outside the project's folder can be read. |
| `[checks]` | Every check at its own level. |

---

## 22. Validation rules

A loader MUST enforce every rule below when it loads `ascribe.toml`, and report each violation at the span of the offending key or value. Every rule is an error unless marked **warning**. A model with errors doesn't load, and no document is checked against it; warnings don't stop loading.

- **Slugs** are stable identifiers, for tests and for the diagnostics registry, [`tests/conformance/diagnostics.toml`]({repo}/blob/main/tests/conformance/diagnostics.toml), which gives each one a code. Only `model-name-multiple-roles` corresponds to a row of SPEC §8.2 ("Content model"); the rest are loader rules this reference adds.
- **Messages** are templates. `{name}` is a placeholder. Where a rule has several messages, each covers one case of it.
- Rules about documents, such as a page matching no content type or an unknown frontmatter key, aren't loader rules.
- **Filesystem rules** (`model-content-root-missing`, `model-glossary-link`, `model-source-path-missing`, `model-source-outside-repository`) need the project directory. A loader given only the file's text, such as an unsaved editor buffer or a unit test, skips them; `ascribe check` and the language server always run them.

### 22.1 File and structure

| Slug | Rule | Message |
|---|---|---|
| `model-toml-syntax` | The file is valid TOML 1.0. | `` ascribe.toml isn't valid TOML: {detail} `` |
| `model-unknown-key` | Every key is one this reference defines for its table, except in tables keyed by names the project chooses. | `` unknown key `{key}` in `[{table}]` ``<br>`` unknown key `{key}` in `[{table}]`; did you mean `{suggestion}`? `` |
| `model-missing-key` | Every required key is present. | `` `[{table}]` is missing the required key `{key}` `` |
| `model-wrong-type` | Every value has the TOML type its key requires. | `` `{key}` must be {expected}, but it's {found} ``<br>`` `{key}` must be a string, but it's a number; quote it: {key} = "{value}" `` |
| `model-invalid-value` | A key with a fixed set of values has one of them. | `` `{key}` can't be "{value}"; use one of: {values} `` |
| `model-spec-unsupported` | `spec` is a version this processor implements. | `` ascribe.toml targets spec version "{spec}", but this processor implements {supported} `` |
| `model-invalid-name` | Every name follows its grammar (§1.2). | `` `{name}` isn't a valid {role} name: {rule} ``, for example `` `Deployment` isn't a valid dimension name: use a lowercase letter, then lowercase letters, digits, or hyphens `` |
| `model-empty-text` | Every `label`, `name`, `term`, `definition`, and `description` is non-empty. | `` `{key}` can't be empty `` |

### 22.2 Project

| Slug | Rule | Message |
|---|---|---|
| `model-path-absolute` | `content-root`, `output-dir`, and each source's `path` are relative paths. | `` `{key}` must be a path relative to ascribe.toml, not an absolute path `` |
| `model-content-root-missing` | The content root exists and is a directory. | `` content root `{path}` doesn't exist ``<br>`` content root `{path}` isn't a directory `` |
| `model-output-overlaps-content` | The output directory isn't inside the content root, the content root isn't inside the output directory, and they differ (derived from the spec). | `` output directory `{output}` is inside content root `{content}`; move it outside, or builds will read their own output as source ``<br>`` content root `{content}` is inside output directory `{output}`; builds could delete source files as stale output ``<br>`` output directory and content root are both `{path}` `` |

### 22.3 Content types, fragments, and fields

| Slug | Rule | Message |
|---|---|---|
| `model-type-multiple-defaults` | At most one type sets `default = true`. | `` only one content type can be the default, but `{a}` and `{b}` both set default = true `` |
| `model-type-unreachable` | Every type has `files` or `default = true`. | `` content type `{type}` has no `files` and isn't the default, so no page can use it `` |
| `model-type-title` | Every page type declares `title` as a required `string`. | `` content type `{type}` must declare title = "string": it's the page title, used for empty link text ``<br>`` content type `{type}`: `title` must be a required string, not "{found}" `` |
| `model-field-reserved` | No content type or fragment schema declares `available` or `variant`, and no content type declares `slug` or `formatted`. | `` `{field}` is reserved by the Ascribe spec and every page accepts it; remove it from `[types.{type}.frontmatter]` ``<br>`` `{field}` is reserved by the Ascribe spec, and fragments can't use it ``<br>`` `slug` is reserved by the astro profile, which uses it as a page's URL id; remove it from `[types.{type}.frontmatter]` ``<br>`` `formatted` is reserved by the site output, which writes the formatted form of `inline` fields under it; remove it from `[types.{type}.frontmatter]` `` |
| `model-type-syntax` | Every field and attribute type parses under §6.1, and is allowed where it's used: no `date`, `list`, or `object` for attributes; no `set` for fields; `object`, `list(object)`, and bare `enum` only in table form. | `` "{type}" isn't a valid {kind} type: {detail} ``, for example `` "strng?" isn't a valid field type: expected string, number, boolean, date, enum(…), or list(…) `` |
| `model-type-fields` | `fields` is present exactly when the type is `object` or `list(object)`, optional or not. | `` field `{field}` is an object, so it needs `fields` ``<br>`` `fields` is only allowed on object fields, and `{field}` is "{type}" `` |
| `model-enum-values` | An enumeration has at least one value and no duplicates, and its values come from exactly one of `enum(…)` and `values`. `values` appears only with bare `enum`. | `` `{field}` has an empty enumeration ``<br>`` "{value}" appears twice in the enumeration for `{field}` ``<br>`` `{field}`: list enumeration values in enum(…) or in `values`, not both ``<br>`` `{field}` is a bare enum, so it needs `values` `` |
| `model-set-token` | Every value of a `set(enum)` attribute is a token (SPEC §3.3). | `` "{value}" can't be in a value set: members can't contain spaces or any of , \| { } = " `` |
| `model-default-type` | A `default` has the declared type, and an enumeration default is one of its values. | `` default for `{field}` must be {type}, but it's {found} ``<br>`` default "{value}" for `{field}` isn't one of: {values} `` |
| `model-phrases-field-type` | `phrases = true` is set only on `string` and `list(string)` fields. | `` phrases = true only works on string and list(string) fields, and `{field}` is "{type}" `` |
| `model-inline-field` | `inline` is `"code"`, and is set only on a content type's top-level `string` fields (§6.3). | `` inline = "code" only works on string fields, and `{field}` is "{type}" ``<br>`` inline must be "code", not "{value}" ``<br>`` inline = "code" only works on a content type's top-level fields, and `{field}` is in {place} `` |
| `model-pattern-syntax` | Every pattern parses under §1.3, doesn't start with `/`, and has no `..` segment. | `` "{pattern}" isn't a valid pattern: {detail} ``<br>`` pattern "{pattern}" is already relative to the content root; remove the leading / ``<br>`` pattern "{pattern}" can't contain .. `` |
| `model-field-role` | `role` is `"description"`, on a `string` field, or `"review-date"`, on a `date` field; only on a content type's top-level fields; and a type has at most one field with each role (§6.4). | `` role = "{role}" only works on a {expected} field, and `{field}` is "{type}" ``<br>`` role must be "description" or "review-date", not "{value}" ``<br>`` role only works on a content type's top-level fields, and `{field}` is in {place} ``<br>`` `{other}` already has role = "{role}"; a content type has one field with each role `` |
| `model-attribute-reserved` | No image or widget attribute key is one HTML already gives a meaning on that element (SPEC §7.2): `src`, `alt`, or `title` on images; `heading` or `primary` on widgets; and on both, HTML's global attributes (such as `id`, `class`, `style`, and `title`), any key starting with `aria-`, and HTML's event-handler attributes (such as `onclick`, `onload`, and `onerror`). The lists are explicit, in `ascribe_core::reserved`, so keys that merely begin with `on`, such as `online` or `only-if`, are allowed. | `` `{key}` can't be an image attribute: HTML already uses it on the <img> element ``<br>`` `{key}` can't be an attribute of widget `{name}`: the site output already uses it on the widget's element `` |

### 22.4 Dimensions, names, lifecycle, notes, and features

| Slug | Rule | Message |
|---|---|---|
| `model-name-multiple-roles` | A name is used in at most one of these roles: dimension name, dimension value, lifecycle state (including built-in states), feature key (SPEC §7.2, §8.2). | `` `{name}` is used as both {role-a} and {role-b}; a name can have only one role, so availability specs stay unambiguous ``, for example `` `beta` is used as both a dimension value (in dimensions.channel) and a lifecycle state (built in); a name can have only one role, so availability specs stay unambiguous `` |
| `model-name-case` (**warning**) | No two names in those roles differ only in case. | `` `{a}` and `{b}` differ only in case; names are case-sensitive, so they're easy to confuse `` |
| `model-dimension-empty` | `values` has at least one value. | `` dimension `{dimension}` has no values `` |
| `model-dimension-value-duplicate` | No value appears twice in one dimension. | `` `{value}` appears twice in dimensions.{dimension}.values `` |
| `model-dimension-value-shared` | No value belongs to two dimensions. | `` `{value}` is a value of both `{a}` and `{b}`; a value can belong to only one dimension `` |
| `model-label-undeclared` | Every key in a dimension's `labels` is one of its values. | `` dimensions.{dimension}.labels has a label for `{value}`, which isn't one of its values: {values} `` |
| `model-versionless-undeclared` | Every entry in a dimension's `versionless` is one of its values. | `` dimensions.{dimension}.versionless lists `{value}`, which isn't one of its values: {values} `` |
| `model-lifecycle-available-required` | A new lifecycle state sets `available`. | `` new lifecycle state `{state}` must set available = true or available = false `` |
| `model-lifecycle-ga-unavailable` | `ga` counts as available. | `` `ga` must count as available: content with no lifecycle state is ga `` |
| `model-note-label-required` | A new note type sets `label`. | `` new note type `{type}` needs a label, such as label = "{Type}" `` |
| `model-availability-syntax` | A feature's `available` parses as an availability spec (SPEC Appendix A `availability`). | `` feature `{key}`: "{spec}" isn't a valid availability spec: {detail} `` |
| `model-availability-unknown-name` | Every target in a feature's spec is a declared dimension value or dimension name, and every state is a declared lifecycle state. | `` feature `{key}`: `{name}` isn't a declared dimension value or dimension name ``<br>`` feature `{key}`: `{name}` isn't a declared lifecycle state `` |
| `model-availability-versionless` | A versionless target, or a dimension name, has no versions in a feature's spec (SPEC §4.4). | `` feature `{key}`: `{target}` is versionless, so it takes a state but no version ``<br>`` feature `{key}`: `{target}` is a dimension name, so it takes a state but no version; name one of its values, such as `{example}`, to give a version `` |
| `model-availability-history-order` | A history in a feature's spec is in chronological order under the version scheme. | `` feature `{key}`: the history for `{target}` must be in chronological order, but {later} comes before {earlier} `` |
| `model-feature-nested` | A feature's `available` isn't itself a feature key. | `` feature `{key}`: available must be an availability spec, not another feature (`{other}`) `` |

### 22.5 Versions, phrases, glossary, and images

| Slug | Rule | Message |
|---|---|---|
| `model-phrase-value-type` | Every phrase value is a string. | `` phrase `{key}` must be a quoted string, but it's {found}; write {key} = "{value}" `` |
| `model-glossary-duplicate-term` | No two terms or aliases are the same text, ignoring case when either is case-insensitive. | `` "{text}" is declared by both glossary terms `{a}` and `{b}` `` |
| `model-glossary-link` | A term's `link` file exists under the content root and isn't a fragment. | `` glossary term `{id}` links to {path}, which doesn't exist ``<br>`` glossary term `{id}` links to {path}, which is a fragment; link to a page that includes it `` |

`[versions] scheme` is covered by `model-invalid-value`. Image attribute types are covered by the rules in §22.3.

### 22.6 Widgets

| Slug | Rule | Message |
|---|---|---|
| `model-widget-reserved-name` | The name doesn't start with `ascribe-` and isn't one of HTML's reserved custom-element names. | `` widget name `{name}` is reserved: names starting with ascribe- belong to Ascribe's element library ``<br>`` widget name `{name}` is reserved by HTML and can't be a custom element `` |
| `model-widget-forms` | `forms` is non-empty, has no duplicates, and contains only `"line"` and `"container"`. | `` forms must be ["line"], ["container"], or ["line", "container"] `` |
| `model-widget-binding` | `binding` is present when `forms` includes `"line"`, and absent otherwise. | `` widget `{name}` has a line form, so it needs a binding: "self", "heading", "block", or "heading-or-block" ``<br>`` widget `{name}` is container-only, and a container holds its own content; remove binding `` |
| `model-widget-container-primary` | With container form, the primary isn't required; a container-only widget's primary is `"none"`. | `` widget `{name}` has a container form, whose opener has no primary, so its primary can't be required; use "{kind}?" ``<br>`` widget `{name}` is container-only, so it can't take a primary `` |
| `model-widget-groupable-form` | A groupable widget is container-only. | `` widget `{name}` is groupable, so it must be container-only: forms = ["container"] `` |
| `model-widget-plain-content` | `plain-content` is set only on widgets that wrap content: container form, or binding `block` or `heading-or-block`. | `` widget `{name}` doesn't wrap content, so plain-content has no effect; remove it `` |

`primary`, `title`, and `binding` values are covered by `model-invalid-value`; attribute types by §22.3.

### 22.7 Consumer, builds, and editor

| Slug | Rule | Message |
|---|---|---|
| `model-consumer-unsupported` | Every consumer key has a value the profile supports (for example, `html = false` with `astro`). | `` the {profile} profile doesn't support {key} = {value}; use {values} `` |
| `model-consumer-site` | `site` is an absolute `http` or `https` URL with no path, query, or fragment. | `` site must be an origin such as "https://docs.example.com"; put any path in base-path `` |
| `model-consumer-base-path` | `base-path` starts with `/`. | `` base-path must start with "/", such as "/docs/" `` |
| `model-consumer-agents-site` | `agents = true` comes with `site`. | `` agents = true needs `site`: every link in llms.txt and the Markdown pages is an absolute URL `` |
| `model-build-name-case` | Build names are unique ignoring case. | `` builds `{a}` and `{b}` differ only in case, so they'd share an output directory on some file systems `` |
| `model-build-variants` | `variants` is `"switch"` or a non-empty table of dimension selections, each a value or a non-empty array of values. | `` variants must be "switch" or a selection such as { deployment = "cloud" } ``<br>`` build `{build}` selects nothing; write variants = "switch" `` |
| `model-build-unknown-dimension` | Every dimension a selection names is declared. | `` build `{build}` selects dimension `{dimension}`, which isn't declared `` |
| `model-build-unknown-value` | Every selected value is a value of its dimension. | `` build `{build}`: `{value}` isn't a value of `{dimension}`; values: {values} `` |
| `model-build-availability` | `availability` is `"badge"` or a table with exactly one key, `filter`. | `` availability must be "badge" or { filter = "<target> <version>" } `` |
| `model-build-filter-target` | The filter's target is a declared dimension value. | `` build `{build}` filters for `{target}`, which isn't a declared dimension value ``<br>`` build `{build}` filters for `{target}`, which is a dimension; filter for one of its values: {values} `` |
| `model-build-filter-version` | The filter has a version exactly when its target is versioned, and the version follows SPEC Appendix A's `version` rule. | `` build `{build}` filters for `{target}`, which is versioned, so it needs a version, such as "{target} 3.3" ``<br>`` build `{build}`: `{target}` is versionless, so the filter can't name a version ``<br>`` build `{build}`: "{version}" isn't a valid version `` |
| `model-build-filter-excluded` (**warning**) | A build's filter target isn't a value its own selection drops. | `` build `{build}` filters for `{target}`, but its selection keeps only {dimension} = {values}, so pages marked for `{target}` are dropped `` |
| `model-editor-build-unknown` | `editor.build` names a declared build. | `` editor.build is `{build}`, which isn't a declared build; builds: {builds} `` |
| `model-editor-build-required` | When there are several builds and none is named `site`, `editor.build` is set. | `` there are several builds and none is named site; set [editor] build to the one the editor should check `` |

### 22.8 Sources
@available: next

| Slug | Rule | Message |
|---|---|---|
| `model-source-path-missing` | A source's `path` exists and is a directory. | `` the folder of source `{source}`, `{path}`, doesn't exist ``<br>`` the folder of source `{source}`, `{path}`, isn't a directory `` |
| `model-source-outside-repository` | A source's folder is inside the git repository the project is in, when it's in one. | `` the folder of source `{source}`, `{path}`, is outside the git repository the project is in `` |
| `model-source-remote` | A source has `path` or `git`, not both; `branch` only with `git`; a URL `git` can be given; a branch name `git` accepts; and, with `git`, copies outside the content root. | `` source `{source}` has both `path` and `git`: a source is a folder in this repository or another repository, not both ``<br>`` source `{source}` needs `path`, a folder in this repository, or `git`, another repository's URL ``<br>`` source `{source}` has `branch` but no `git`: a branch is only for a source in another repository ``<br>`` `{url}` isn't a URL Ascribe can give `git`: use `https://`, `http://`, `ssh://`, `git://`, `file://`, or `user@host:path` ``<br>`` `{branch}` isn't a branch name `git` accepts ``<br>`` the copies of source `{source}` go in `sources/{source}/`, which is inside the content root `{content}`, where they'd be read as pages; give the content root a folder of its own `` |

### 22.9 Checks
@available: next

| Slug | Rule | Message |
|---|---|---|
| `model-check-not-configurable` | `[checks]` names only checks the diagnostics registry marks configurable. | `` `{check}` can't be set in [checks]: it reports whether the project is valid, which a project can't lower or turn off `` |
| `model-intended-check` | `[[intended]]` names a review check that's acknowledged at a content model entry. | `` `{check}` isn't a check `` |
| `model-intended-entry` | `[[intended]]` names exactly one declared phrase, feature, glossary term, or image. | `` an acknowledgement in [[intended]] names what it's about with one of `phrase`, `feature`, `term`, or `image` `` |

---

## 23. Decisions

Each item settles a gap in SPEC.md. All 21 were decided on 2026-09-28 as recommended below, except item 12, which was settled when the site-render and asset contracts were written. Items 1, 2, 3, 4, and 6 are now also stated in SPEC.md (§2.1, §5.2, §7.2, §8.2). Each item keeps the alternatives that were considered.

1. **Frontmatter diagnostics (SPEC §8.2).** §8.1 says file-level validation covers frontmatter, but §8.2 has no rows for it. *Decision:* add file-level error rows: unknown frontmatter key; missing required field; value doesn't match the field's type; reserved key (`available`, `variant`) on a fragment; page matches more than one content type; page matches no content type and there's no default.
2. **Assigning content types to pages (SPEC §7.2).** The spec doesn't say how a page gets its type. *Decision:* `files` patterns per type plus at most one `default = true` type; a page matching several types is an error, with no precedence. *Considered:* first match in file order (TOML tables are formally unordered); most specific pattern (hard to define); a frontmatter `type` key (would need a new reserved key).
3. **Page titles (SPEC §5.2).** Empty link text uses "the page title", but the spec doesn't say where it comes from. *Decision:* the frontmatter `title`, which every page type must declare as a required string. *Considered:* fall back to the first level-1 heading.
4. **Reserved keys in fragments (SPEC §2.1, §4.3, §4.4).** `available` and `variant` are defined for pages. *Decision:* fragments can't use them in spec 0.1; use `@available` inside the fragment. *Considered:* fragment `available` applies to everything the fragment contributes, like a section spec.
5. **Version scheme name (SPEC §4.4).** *Decision:* call the one scheme `numeric`: dotted numbers of any length, compared numerically with missing components as 0. The grammar has no pre-release syntax, so "semver" would overpromise. *Considered:* call it `semver` and cap versions at three components, which needs a new document diagnostic.
6. **Name rules beyond the one-role rule (SPEC §4.3, §4.4, §7.2).** *Decision:* (a) a dimension value belongs to only one dimension, since `cloud` in a spec must mean one thing; (b) dimension names follow the `key` rule, since they're attribute keys; (c) warn on names that differ only in case. (a) and (b) are stated in SPEC §7.2.
7. **Glossary (SPEC §5.4).** The spec gives no format or matching rules. *Decision:* terms with a required plain-text `definition` and an optional `link`; occurrences link to `link`; terms without it aren't linked in site or plain output. Whole-word matching, longest term wins, prose only (not headings, link text, or code), `first` per resolved page by default. No term element is needed: in the site output an occurrence is an ordinary link whose title is the definition, marked with `data-ascribe-term` (see the [element contract]({repo}/blob/main/packages/elements/CONTRACT.md)).
8. **Widget plain fallback (SPEC §6, §9.4).** *Decision:* a static CommonMark string with phrases substituted and no attribute interpolation (no behavior); a widget that wraps content keeps that content in plain output unless `plain-content = "drop"`, since silently losing content is worse than showing it. The spec's "or nothing" then applies to the widget itself, not its content.
9. **Widget schema constraints (SPEC §3.5, §3.6, §6).** *Decision:* groupable widgets are container-only; widgets with container form have no required primary; names starting with `ascribe-` and HTML's reserved custom-element names are rejected.
10. **Absolute links in plain output (SPEC §9.4).** Plain-markdown links are "absolute URLs", which needs the site's origin. *Decision:* optional `[consumer] site`; without it, links are root-relative and `ascribe build` warns.
11. **HTML passthrough (SPEC §9.5).** The site output depends on raw HTML (custom elements). *Decision:* keep the key, but the `astro` profile accepts only `true` until a profile needs `false`, rather than defining a degraded site output now.
12. **Heading ids, image attributes, and asset placement (SPEC §9.4, §9.5).** *Decision:* no keys. The profile named by `profile` fixes all three, and the `astro` profile has one way to do each: a `<ascribe-attributes>` marker for heading ids and image attributes ([site-render contract](site-render.md)), and mirrored asset copies with relative image references ([asset contract](assets.md)). A key that accepts one value says nothing, and since unknown keys are errors, adding a key when a second profile needs a choice breaks no existing file, while removing one later would. *Considered:* the keys first drafted, `heading-ids` (`"attribute"` or `"html"`), `image-attributes` (`"attribute"` or `"html"`), `assets` (`"beside-page"` or `"directory"`), and `assets-dir`. Their alternatives were dropped: an `{#id}` attribute block is rewritten by Astro's default smartypants and GFM processing before a plugin sees it (quotes, `--` in ids), which the marker avoids; a raw HTML heading loses the consumer's inline processing and, in Astro, its table-of-contents entry; a raw `<img>` bypasses Astro's image processing; and a shared asset directory needs hashed names to avoid collisions, which mirroring avoids by construction. With them went the loader rule `model-consumer-assets-dir`.
13. **Filter builds on versioned targets (SPEC §9.3).** A filter is "given a target and, for versioned targets, a version". *Decision:* the version is required for versioned targets, and not allowed for versionless ones. *Considered:* a versioned target with no version means "at every version", but then the "state in effect" is undefined.
14. **Project defaults.** *Decision:* `content-root = "docs"`, `output-dir = ".ascribe/build"`; paths relative to `ascribe.toml`, `..` allowed, absolute paths rejected (keeps projects portable). The content root can't be `"."` by default, because the output directory couldn't then sit outside it.
15. **YAML flavor for frontmatter.** *Decision:* the YAML 1.2 core schema (`yes` is a string, `3.10` is a number), and `date` fields accept `YYYY-MM-DD` scalars, quoted or not.
16. **The editor's default build.** *Decision:* the only build, else the build named `site`, else required.
17. **Built-in lifecycle states and note types.** *Decision:* built-ins can be relabeled, and states' `available` flags changed, but not removed; `ga` must stay available; new states must set `available` explicitly; new note types need a `label`; built-in states take part in the one-role rule.
18. **Implicit type and build.** *Decision:* with no `[types]`, one default type `page` with a required `title`; with no `[builds]`, one build `site` (`switch`, `badge`). Declaring any type or build removes the implicit one.
19. **Features referring to features (SPEC §4.4).** *Decision:* not allowed, so there are no chains or cycles.
20. **Phrases in frontmatter (SPEC §5.1).** *Decision:* opt in per field with `phrases = true`, on `string` and `list(string)` fields only; off by default.
21. **Spec version matching (SPEC §11).** *Decision:* `spec` is a quoted string that must exactly equal a version the processor implements; `"0.1"` for now. Revisit compatibility ranges when 0.2 exists.
22. **Code in a page title (SPEC §7.2).** *Decision:* opt in per field with `inline = "code"`, on a content type's top-level `string` fields, and code spans only (§6.3). Every output carries the plain text and the formatted form. *Considered:* reading code spans in every title, which would change the meaning of every existing title with a backtick and make `title` the one string field that isn't plain text; and links or emphasis in titles, which go wrong where a title becomes the text of another link.
23. **Fields the checks read (SPEC §7.2).** A check of a page's description or its review date needs to know which field holds it, and a project names its fields. *Decision:* a `role` on the field (§6.4), so the type that has the field says what it is, and a type without one opts out of the check. *Considered:* a `field` setting in each check's `[checks]` table, which names one field for every type, so two types with different names for their description can't both be checked, and a type without the field is reported as missing it; and fixed names, such as `description`, which take a name from every project.
