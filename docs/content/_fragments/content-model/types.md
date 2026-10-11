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
| `frontmatter` | table of fields | **required** | The frontmatter schema: one entry per field, keyed by field name (`name-word`). Values are field types (§5). It MUST declare `title` as a required string (below). |

**Which type applies to a page.** For each page (not fragments; see §4.3):

1. If exactly one type's `files` match the page, that type applies.
2. If several types' `files` match, it's an error on the page: the page is ambiguous. There's no precedence between types.
3. If none match and a default type exists, the default applies.
4. If none match and there's no default type, it's an error on the page.

A type with neither `files` nor `default = true` could never apply, and is an error (`model-type-unreachable`). The rules in steps 2 and 4 are document diagnostics, listed in [SPEC §8.2]({repo}/blob/main/SPEC.md#82-diagnostics).
