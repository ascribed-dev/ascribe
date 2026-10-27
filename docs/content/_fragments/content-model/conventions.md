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
- **Declaration order is kept where it matters.** Attribute keys keep the order they're declared in (`[images.attributes]`, `[widgets.<name>.attributes]`), because canonical form writes attributes in declared order ([SPEC §8.3]({repo}/blob/main/SPEC.md#83-canonical-form)). The `[dimensions]` tables keep theirs too: it's the canonical order of `@variant` attributes, and decides which dimension a tab group syncs on (the [element contract]({repo}/blob/main/packages/elements/CONTRACT.md), §3). Order is kept elsewhere for stable output (for example, the order of dimension values in a tab switcher is the order of `values`). Nothing else in the model depends on order.

### 1.2 Names

Several kinds of name appear in the file. Each has a grammar from SPEC Appendix A or from this reference:

| Name | Grammar | Rule | Used for |
|---|---|---|---|
| name-word | SPEC A `name-word` | A letter, then letters, digits, `_`, or `-` | Dimension values, lifecycle states, feature keys, frontmatter field names |
| key | SPEC A `key` | A lowercase letter, then lowercase letters, digits, or `-` | Dimension names, phrase keys, note types, attribute keys, content type names, source names |
| widget name | SPEC A `widget-name` | Lowercase words of letters and digits joined by single hyphens, with at least one hyphen; starting with a letter | Widget names |
| build name | This reference | A letter, then letters, digits, `_`, `-`, or `.` | Build names |

Dimension names follow the stricter `key` rule, not just `name-word`, because they're also written as attribute keys in `@variant {deployment=cloud}` ([SPEC §3.3]({repo}/blob/main/SPEC.md#33-attributes), §4.3).

Build names allow `.` so that names like `self-managed-3.3` work. A name containing `.` must be quoted in a TOML header: `[builds."self-managed-3.3"]`.

Names are case-sensitive and spelled exactly as declared ([SPEC §4.4]({repo}/blob/main/SPEC.md#44-available)).

### 1.3 Paths and patterns

- Paths use `/` as the separator on every platform.
- Filesystem paths in `[project]` and `[sources]` are relative to the project root. They can't be absolute, and they can use `..`.
- A path in `[glossary] pages` (§8) is written like a link destination in a document ([SPEC §5.2]({repo}/blob/main/SPEC.md#52-links)), relative to the content root.
- **Patterns** (globs) are matched against a source file's path relative to the content root, including its extension, with `/` separators. They are case-sensitive. The syntax:

  | Syntax | Matches |
  |---|---|
  | `*` | Any run of characters within one path segment, not including `/` |
  | `**` | Any number of whole segments, including none. It's only ever a whole segment (`a/**/b`, `a/**`, `**/b`) |
  | `?` | Any one character except `/` |
  | `{a,b}` | Either alternative. Alternatives don't nest |
  | `\` | Escapes the next character |

  Every other character matches itself. A pattern doesn't start with `/`, and has no `..` segment. Patterns only ever apply to Markdown source files (`*.md`); other files under the content root are assets. The exception is a source's `include` and `ignore` (§20), which are matched against any file's path relative to the source's folder.

### 1.4 How this reference describes keys

Each section has a table of keys with these columns:

- **Type:** the TOML type. "Field type", "attribute type", and "availability spec" are strings with the grammars in §5 and [SPEC §4.4]({repo}/blob/main/SPEC.md#44-available).
- **Default:** the value when the key is absent, or **required**.
- **Description.**
