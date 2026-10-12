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
| `template` {available=next} | string (path) | none | The page a page of the type starts from: a file from the folder `ascribe.toml` is in, outside the content root (below). |
| `guide` {available=next} | array of paths | `[]` | What to read before writing a page of the type, in the order to read it, such as a guide and a project's own notes on it: files from the folder `ascribe.toml` is in, outside the content root (below). |

**Which type applies to a page.** For each page (not fragments; see §4.3):

1. If exactly one type's `files` match the page, that type applies.
2. If several types' `files` match, it's an error on the page: the page is ambiguous. There's no precedence between types.
3. If none match and a default type exists, the default applies.
4. If none match and there's no default type, it's an error on the page.

A type with neither `files` nor `default = true` could never apply, and is an error (`model-type-unreachable`). The rules in steps 2 and 4 are document diagnostics, listed in [SPEC §8.2]({repo}/blob/main/SPEC.md#82-diagnostics).

@available: next
**A type's template and guide.** Both are paths from the folder `ascribe.toml` is in, and both name files that aren't pages:

```toml
[types.how-to]
files = ["how-to/**"]
template = "templates/how-to.md"
guide = ["templates/how-to.guide.md", "templates/how-to.ascribe.md"]
```

Each file must be there, with its name's case as it is on disk, inside the folder `ascribe.toml` is in, and outside the content root, where it would be read as a page, and the output directory, which a build writes and removes. A symbolic link is followed to where it leads (`model-type-file`). `guide` is always a list, as `files` is, even with one file in it (`model-wrong-type`). The template is read when the content model loads, and must read as a page: valid YAML between its `---` lines, and no error in its Ascribe syntax (`model-type-template`). Either is a problem with the content model, reported at the path, once, rather than on every page made from the template. A type with neither key is as it is without them.

@available: next
**A page that still has its template's text.** On a page of a type with a template, `ascribe check` reports as advice each of the template's placeholders the page still has word for word, a frontmatter field with the template's value, and a paragraph the template quotes as an instruction (`template-placeholder-left`). The [guide to page templates](../../guides/templates.md#whats-left-to-fill-in) says what it finds and what it can't.
