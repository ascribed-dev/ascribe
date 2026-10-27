A file containing only `spec = "0.1"` is valid. It means:

| Section | When absent |
|---|---|
| `[project]` | `content-root = "docs"`, `output-dir = ".ascribe/build"` |
| `[types]` | One implicit default page type, named `page`, with frontmatter `title = "string"` and nothing else. If `[types]` declares any type, there's no implicit type. |
| `[fragments]` | Only paths with a `_` segment are fragments. The fragment schema has no fields, so a fragment's frontmatter can't have any keys. |
| `[notes]` | The five built-in note types. |
| `[phrases]` | No phrases. Every `{…}` is literal text. |
| `[glossary]` | No glossary. |
| `[images]` | No image attributes. |
| `[widgets]` | No project widgets. |
| `[dimensions]` | No dimensions. Any `@variant` arm with attributes, or `variant` frontmatter, is an error. |
| `[versions]` | `scheme = "numeric"` |
| `[lifecycle]` | The five built-in states. |
| `[features]` | No features. |
| `[consumer]` | `profile = "astro"` with its defaults. |
| `[builds]` | One implicit build, `site`, with `variants = "switch"` and `availability = "badge"`. If `[builds]` declares any build, there's no implicit one. |
| `[listing]` | No rules. A page that doesn't set `listing` is `listed`. |
| `[crawlers]` | No lines. |
| `[permalinks]` | No retired permalinks and no aliases. |
| `[sources]` | No sources. Nothing outside the project's folder can be read. |
| `[checks]` | Every check at its own level. |
| `[editor]` | `build` as in §23. |
