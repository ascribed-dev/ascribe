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

@include: ../field-type-grammar.md

### 5.2 Table form
