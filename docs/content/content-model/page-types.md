---
type: how-to
title: Define your page types and fields
description: Turn a project's pages into kinds of page, each with the frontmatter it needs, and let `ascribe check` find the pages that don't have it yet.
---

A page type says what a kind of page has in its frontmatter: a how-to names the team that owns it and who it's for, a reference only the team. Once `ascribe.toml` declares the types, every page names its own, and `ascribe check` reports each page that's missing a field its type needs, or has a value its type doesn't allow. The editor reads the same types, so it offers them when you write a page's `type:`, and reports the same problems as you type.

This guide takes a project from the one type every project starts with to two types that share some of their fields. It's for whoever decides what a project's pages hold, in a project that has pages already, as [Getting started](../getting-started.md) or [Bring existing docs to Ascribe](../existing-docs.md) sets one up. Every key it uses is in the [`ascribe.toml` reference](index.md#4-typesname-and-fragments).

## Where a project starts

A project that declares no types has one, the implicit `page`. Its pages need no `type`, and their frontmatter is a `title` and whatever `[frontmatter]` declares:

```toml
spec = "0.1"

[frontmatter]
description = "string"
```

This guide's project has two pages, a how-to and a reference:

```yaml
---
title: Install the agent
description: Install the agent on a server, and check that it reports in.
---
```

```yaml
---
title: API reference
description: Every endpoint the agent reports to.
---
```

`ascribe check` finds nothing wrong with either. To find the pages that need something, such as an owner, declare the kinds of page and what each needs.

To start from a set of types that's written already, a how-to, a reference, a tutorial, a concept, and a troubleshooting page, each with a template, take the [page templates](../templates/index.md#take-the-templates) instead, and then change their fields as this guide does.

## Declare the types

@steps
1. Add a table for each kind of page, named in lowercase letters, digits, and `-`:

    ```toml
    [types.how-to]

    [types.reference]
    ```

    A type with no keys of its own has `title` and every field `[frontmatter]` declares, so it needs nothing more yet.

2. Run `ascribe check`. A project that declares types has no implicit `page`, so every page now has to say which type it is:

    ```text
    docs/api.md:1: [ASC196] this page doesn't say what type it is; give it `type:` in its frontmatter, with one of: how-to, reference → Add `type: how-to`
    docs/install.md:1: [ASC196] this page doesn't say what type it is; give it `type:` in its frontmatter, with one of: how-to, reference → Add `type: how-to`
    checked 2 files for every build, 2 with problems: 2 errors, 0 warnings, 0 advice
    ```

3. Give each page its type, as the first key of its frontmatter:

    ```yaml
    ---
    type: reference
    title: API reference
    description: Every endpoint the agent reports to.
    ---
    ```

    In VS Code, each problem's quick fix adds `type:` with one of the declared types. A folder can hold pages of every type, so a page's type is in its frontmatter, never in its path.

## Add the fields

The fields every type has go in `[frontmatter]`, written once. A field only one type has goes in that type's own `frontmatter`.

@steps
1. Give every page an owning team, as a required field in `[frontmatter]` whose value is one of a list:

    ```toml
    [frontmatter]
    description = "string"
    team = { type = "enum(docs, platform, support)", role = "owner" }
    ```

    A field is required unless its type ends in `?`, as `"string?"`. `role = "owner"` says this is the field that names a page's owner, so [`ascribe report inventory`](../checks/report-command.md#cli-report) counts pages by it. A field named `owner` would have the role without it.

2. Give how-tos an audience, with a value for the pages that don't set one:

    ```toml
    [types.how-to.frontmatter]
    audience = { type = "enum(admin, developer)", default = "developer" }
    ```

    A field with a default is never missing: a how-to that leaves `audience` out is for developers. Only how-tos can set it, since only their type declares it.

3. Run `ascribe check`. It reports each page that doesn't have the new required field:

    ```text
    docs/api.md:1: [ASC048] the required field `team` is missing; the content type `reference` needs it
    docs/install.md:1: [ASC048] the required field `team` is missing; the content type `how-to` needs it
    checked 2 files for every build, 2 with problems: 2 errors, 0 warnings, 0 advice
    ```

    A value that isn't one of the list is an error too, as `team: writers` is:

    ```text
    docs/api.md:4: [ASC049] `team` must be one of: docs, platform, support, but it's a string
    ```

4. Add the field to each page that's missing it:

    ```yaml
    ---
    type: how-to
    title: Install the agent
    team: platform
    description: Install the agent on a server, and check that it reports in.
    ---
    ```

    `ascribe check` then finds nothing wrong, and `ascribe report inventory` counts the pages by type and by team:

    ```text
    Inventory
      2 pages.
      By type: how-to 1, reference 1.
      By owner: docs 1, platform 1.
    ```

The whole content model, with both types:

```toml
spec = "0.1"

[frontmatter]
description = "string"
team = { type = "enum(docs, platform, support)", role = "owner" }

[types.how-to.frontmatter]
audience = { type = "enum(admin, developer)", default = "developer" }

[types.reference]
```

## Adding a field to pages you have

A new required field is an error on every page of its types until each has it, so a project with many pages can add one in steps:

1. Declare it optional, as `"enum(docs, platform, support)?"`.
2. Fill it in page by page.
3. Make it required once `ascribe check` finds no page without it.

A default is the other way: the pages that leave the field out take its value, and need no change.

A type can also make a field `[frontmatter]` declares different for its own pages, by declaring a field of the same name: an index page's type can make a shared `description` optional. The type's field replaces the shared one whole, for that type.

## See what each type has

`ascribe model --section types` lists each type's fields, with where each is declared:

```text
- `how-to`: frontmatter `title` (string, required, implicit), `description` (string, required, from `[frontmatter]`), `team` (enum(docs, platform, support), required, from `[frontmatter]`), `audience` (enum(admin, developer), own)
- `reference`: frontmatter `title` (string, required, implicit), `description` (string, required, from `[frontmatter]`), `team` (enum(docs, platform, support), required, from `[frontmatter]`)
```

In VS Code, hovering a frontmatter key in a page says the same for that field. `ascribe agents sync` writes the types into the rules agents read, so an agent writing a page gives it the fields its type needs.

## See also

- [`[types.<name>]` and `[fragments]`](index.md#4-typesname-and-fragments): every key a type has, and the rules.
- [Field and attribute types](index.md#5-field-and-attribute-types): every type a field can have, and the [field roles](index.md#54-field-roles).
- [Page templates](../templates/index.md): a template and a guide for each type, and starting a page from one.
- [Frontmatter](../writing/frontmatter.md): what a page writes at its top.
