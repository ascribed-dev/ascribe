**Unknown frontmatter keys.** A page whose frontmatter has a key its type doesn't declare (and that isn't reserved) is an error on the page.

### 4.2 Frontmatter values

Frontmatter is YAML. For type checking, processors parse it with the YAML 1.2 **core schema**: `true` and `false` are booleans (`yes`, `no`, `on`, and `off` are strings), and `3.10` is the number 3.1. So a `string` field whose value is `3.10` unquoted is a type error, and the message suggests quoting it.

### 4.3 Fragments

@snippet: code:examples/content-models/full.toml#fragments

| Key | Type | Default | Description |
|---|---|---|---|
| `patterns` | array of patterns | `[]` | Additional fragment patterns ([SPEC §2.2]({repo}/blob/main/SPEC.md#22-pages-and-fragments)). A file is a fragment if any segment of its path begins with `_`, or its path matches one of these patterns. |
| `frontmatter` | table of fields | `{}` (no fields) | The fragment schema ([SPEC §2.2]({repo}/blob/main/SPEC.md#22-pages-and-fragments)): the frontmatter every fragment is validated against. Fields as in §5. `title` is not required. |

Content types never apply to fragments: a fragment names no type, and `type` in a fragment's frontmatter is an error (`frontmatter-reserved-in-fragment`).
