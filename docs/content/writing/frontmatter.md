---
type: reference
title: Frontmatter
description: "Pages and fragments, and the keys a file's frontmatter takes: the reserved ones, and those its content type declares."
---

A file may start with YAML frontmatter between `---` lines. Its content type in `ascribe.toml` says which keys it takes, and every page takes a `title` ([`[types.<name>]`](../content-model/index.md#4-typesname-and-fragments)). Nine keys more are reserved: every page takes them, whatever its type, and no type can declare them. The [Ascribe specification]({repo}/blob/main/SPEC.md#21-files) is the normative definition.

```markdown
---
type: guide
title: Install the Quill agent
available: cloud, self-managed preview 3.3
---
```

## Files

([SPEC §2]({repo}/blob/main/SPEC.md#2-documents))

A project's source files are the `.md` files under its **content root** (`docs/` unless `ascribe.toml` says otherwise). Files and directories whose names start with `.` are skipped, and so is a directory below the content root that holds an `ascribe.toml`, with everything in it, unless it's the project's own folder: that is another project, with sources of its own. Each file is a page or a fragment:

- A **page** is published on its own.
- A **fragment** exists only to be included in other files. A file is a fragment when any part of its path starts with `_` (`_warning.md`, `_snippets/setup.md`), or when it matches one of the `[fragments] patterns` in `ascribe.toml`.

A fragment takes only `intended` of the reserved keys.

## Reserved keys

| Key | Its value | Says |
|---|---|---|
| `type` | A content type's name | [What type the page is](frontmatter.md#a-pages-type) |
| `available` | An availability spec | Where the page applies, as [`@available`](directives.md#available) does for a section |
| `variant` | A mapping of dimensions to values | Which [variant](frontmatter.md#whole-page-variants) of a page it is |
| `permalink` | A name | The page's [permalink](frontmatter.md#a-pages-permalink) |
| `listing` | `listed`, `unquoted`, or `unlisted` | Whether readers that aren't people are offered the page: [how a page is listed](../publishing/index.md#how-a-page-is-listed) |
| `replaces` | A link to a page | The page [it takes the place of](frontmatter.md#a-page-that-replaces-another) |
| `nav-label` | A string on one line | The page's [name in the sidebar](../navigation/index.md#a-page-s-label) and the previous and next links, when it's shorter than the title |
| `draft` | `true` or `false` | Whether the page is a [draft](frontmatter.md#a-draft), which no build publishes |
| `intended` | A list of `check` and `reason` | A check's problem in the page or fragment that's [intended](directives.md#intended) |

## A page's type

A page says what type it is with `type`, the name of a [content type](../content-model/index.md#41-page-types) `ascribe.toml` declares, conventionally its first key. In a project that declares types, every page names one, wherever its file is; in a project that declares none, every page is of the implicit type `page`, and names none. A fragment has no type, and can't use `type`.

## A page's permalink

A page has a [permalink](directives.md#permalinks) in its frontmatter, and its address leads to the top of the page:

```yaml
---
title: Command reference
permalink: cli-reference
---
```

A page's permalink follows the rules of an id, and shares one set of names with the headings': a page and a heading can't have the same one in a build. A fragment can't have one, since it isn't published on its own. The site output keeps `permalink` in the page's frontmatter, so a layout can show the page's permanent address.

## Whole-page variants

When a whole page differs by dimension, write separate pages and give each a `variant`, with the dimensions and values of a [`@variant`](directives.md#variant) arm:

```yaml
---
title: Connect to Quill Cloud
variant:
  deployment: cloud
---
```

A build that selects another deployment drops the page.

## A page that replaces another

A page rewritten for a newer release, at a new path, says which page it takes the place of with `replaces`, written as a link to that page is:

```markdown
---
type: guide
title: Configure the agent
available: self-managed 3.5
replaces: configure-legacy.md
---
```

In a build versioned by a target, a link to either page lands on whichever one the line has, and a site's list of a page's lines leads from one to the other ([links between lines](../versions/lines.md#links-between-lines)). In any other build the two are different pages.

- It names a page, without `#` and an id. A page can't replace itself, and a chain of `replaces` can't come back to where it started.
- No line publishes both pages, and no line publishes two pages that replace the same page: mark the new page with the line it starts in, and the old one with where it ends, `self-managed (ga 3.0, removed 3.5)`.
- A fragment can't use it, and a content type can't declare a field named `replaces`.

## A draft

A page that isn't finished says so with `draft: true`, and no build publishes it until the line goes:

```yaml
---
title: Rotate keys automatically
draft: true
---
```

- Every build drops it, whatever its modes: it has no page in the site, plain, or JSON output, no entry in the sidebar, the page records, or `llms.txt`, and its permalink isn't published. In VS Code, the [build lens](../editor/preview.md#the-build-lens) dims it whole, as it dims a page a build's modes drop.
- `ascribe check` still checks it, as content no build publishes, so it's ready the day it's published.
- What it still has of its type's [template](../templates/index.md#whats-left-to-fill-in) is advice, where a page that publishes has an error, and a section's `pages` in `ascribe.nav` needn't name it yet. A page started with [`ascribe new`](../templates/index.md#cli-new) or the editor's **New Page** is a draft.
- A link to it from a published page is [`link-page-dropped`](../checks/diagnostics/pages.md#ASC121) in every build: take the link out until the page is published, or publish it. A draft can link to another draft, and include fragments.
- Remove `draft: true`, or write `draft: false`, to publish it in every build that would otherwise publish it.
- A fragment can't use it, and a content type can't declare a field named `draft`.
