**Unknown frontmatter keys.** A page whose frontmatter has a key its type doesn't declare (and that isn't reserved) is an error on the page.

### 4.2 Frontmatter values

Frontmatter is YAML. For type checking, processors parse it with the YAML 1.2 **core schema**: `true` and `false` are booleans (`yes`, `no`, `on`, and `off` are strings), and `3.10` is the number 3.1. So a `string` field whose value is `3.10` unquoted is a type error, and the message suggests quoting it.

### 4.3 Fragments

@snippet: code:examples/content-models/full.toml#fragments

@include: ../../_generated/content-model-fragments.md

Content types never apply to fragments: a fragment names no type, and `type` in a fragment's frontmatter is an error (`frontmatter-reserved-in-fragment`).
