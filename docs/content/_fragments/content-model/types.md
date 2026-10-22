### 4.1 Page types

A **content type** is a frontmatter schema for a kind of page. A page names its type in its frontmatter, with `type`.

```toml
[frontmatter]
description = "string?"

[types.guide]

[types.reference.frontmatter]
api-version = "string"
```

`<name>` is the type's name (`key` rule). It appears in diagnostics, in generated code (the Zod schema's name), and on each page of the type, as the value of `type`.

@available: next
**Frontmatter every type shares.** `[frontmatter]` declares the fields every page type has, written as a type's own are, so a field every page has is written once. A type's own `frontmatter` adds fields to them. A type that declares a field `[frontmatter]` has replaces it whole, for that type: an index page's type can make a shared `description` optional, or give it a default. The two aren't merged key by key, so a field reads as one declaration, wherever it's written. The implicit `page` type of a project that declares no types has them too.

@include: ../../_generated/content-model-frontmatter.md

Every page type has a required string `title`, whether it's declared or not (below), so a type with nothing else of its own needs no keys: `[types.guide]` above is a type a page can name, with `title` and the shared fields.

@include: ../../_generated/content-model-types.md

**Which type a page is.** A page says what type it is with `type` in its frontmatter, the first key by convention, wherever its file is: a folder can hold pages of every type.

```yaml
---
type: reference
title: Commands
api-version: "3"
---
```

- **A project that declares no types** has one, the implicit `page`, whose fields are a required string `title` and `[frontmatter]`'s. Its pages need no `type`, and a page that has one is an error (`content-type-unknown`), with a fix that removes it.
- **A project that declares types** has no implicit one. A page that names no type is an error on the page (`content-type-missing`), with a fix for each declared type that adds `type:` to the page's frontmatter. A page whose `type` names a type that isn't declared is an error at the value (`content-type-unknown`), with a fix to the closest name when one is close.
- **Fragments** have no type (§4.3), and can't use `type` (`frontmatter-reserved-in-fragment`).

A type's table has no keys that say which pages it applies to: `files` and `default` are unknown keys (`model-unknown-key`), whose message says to name the type on its pages instead.

**A type's template and guide.** Both are paths from the folder `ascribe.toml` is in, and both name files that aren't pages:

```toml
[types.how-to]
template = "templates/how-to.md"
guide = ["templates/how-to.guide.md", "templates/how-to.ascribe.md"]
```

Each file must be there, with its name's case as it is on disk, inside the folder `ascribe.toml` is in, and outside the content root, where it would be read as a page, and the output directory, which a build writes and removes. A symbolic link is followed to where it leads (`model-type-file`). `guide` is always a list, even with one file in it (`model-wrong-type`). The template is read when the content model loads, and must read as a page: valid YAML between its `---` lines, and no error in its Ascribe syntax (`model-type-template`). Either is a problem with the content model, reported at the path, once, rather than on every page made from the template. A type with neither key is as it is without them.

**What a type is for.** `purpose` is one string, for a project's own types, and for a team that wants to say what one of the starter set's types means to it:

```toml
[types.how-to]
purpose = "One task, done step by step, for someone who knows why they're doing it."
```

The rules for agents say it at the end of the type's line. A type `ascribe templates eject` added and that has no `purpose` says their index's summary instead, from the `templates.lock` beside its template. When the rules' lines don't fit in their 4,000 characters, the page types' are cut first, and the cut says `ascribe model --section types` lists the rest.

**A page that still has its template's text.** On a page of a type with a template, `ascribe check` reports as advice each of the template's placeholders the page still has word for word, and a frontmatter field with the template's value (`template-placeholder-left`). The [guide to page templates](../../templates/index.md#whats-left-to-fill-in) says what it finds and what it can't.

@available: ascribe 0.4.0
**A section the template leaves to the page.** A section the template gives nothing under its heading but memos, and that a page of the type still leaves empty, builds as a heading with nothing under it; `ascribe check` reports it as advice at its heading (`template-section-empty`). The [guide to page templates](../../templates/index.md#whats-left-to-fill-in) says what counts as empty.
