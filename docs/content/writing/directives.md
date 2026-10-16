---
type: reference
title: Directive reference
description: "The language: directives, attributes, titles, phrases, links, and images."
permalink: directives
---

Ascribe source is CommonMark with a few additions: **directives**, lines that start with `@` and annotate the content around them; **title lines**, which give a directive a title; and **phrases**, reusable strings written `{key}`. Everything else is ordinary Markdown, and an Ascribe page read by a plain Markdown renderer still reads sensibly.

This page describes the language as you write it. The [Ascribe specification]({repo}/blob/main/SPEC.md) is the normative definition; each section links to its part of the spec.

## Contents

- [Files](directives.md#files)
- [Directive lines](directives.md#directive-lines)
- [Attributes](directives.md#attributes)
- [Forms: line and container](directives.md#forms-line-and-container)
- [Groups](directives.md#groups)
- [Titles](directives.md#titles)
- [Binding: what a directive applies to](directives.md#binding-what-a-directive-applies-to)
- [Lists and block quotes](directives.md#lists-and-block-quotes)
- [Built-in directives](directives.md#built-in-directives): [`@id`](directives.md#id), [`@include`](directives.md#include), [`@variant`](directives.md#variant), [`@available`](directives.md#available), [`@note`](directives.md#note), [`@steps`](directives.md#steps), [`@details`](directives.md#details), [`@snippet`](directives.md#snippet), [`@intended`](directives.md#intended)
- [Project widgets](directives.md#project-widgets)
- [Phrases](directives.md#phrases)
- [Links](directives.md#links)
- [Images](directives.md#images)
- [Heading ids](directives.md#heading-ids)
- [Glossary terms](directives.md#glossary-terms)
- [Escapes](directives.md#escapes)
- [Canonical form](directives.md#canonical-form)

## Files

([SPEC §2]({repo}/blob/main/SPEC.md#2-documents))

A project's source files are the `.md` files under its **content root** (`docs/` unless `ascribe.toml` says otherwise). Files and directories whose names start with `.` are skipped, and so is a directory below the content root that holds an `ascribe.toml`, with everything in it, unless it's the project's own folder: that is another project, with sources of its own. Each file is a page or a fragment:

- A **page** is published on its own.
- A **fragment** exists only to be included in other files. A file is a fragment when any part of its path starts with `_` (`_warning.md`, `_snippets/setup.md`), or when it matches one of the `[fragments] patterns` in `ascribe.toml`.

A file may start with YAML frontmatter between `---` lines. Its content type in `ascribe.toml` says which keys it takes; every page takes a `title`. Some keys are reserved for pages: `type`, `available` ([availability](directives.md#available)), and `variant` ([whole-page variants](directives.md#whole-page-variants)), with `permalink`, `listing`, and [`replaces`](directives.md#a-page-that-replaces-another).

```markdown
---
type: guide
title: Install the Quill agent
available: cloud, self-managed preview 3.3
---
```

@available: ascribe 0.3.1
A page says what type it is with `type`, the name of a [content type](../content-model/index.md#41-page-types) `ascribe.toml` declares, conventionally its first key. In a project that declares types, every page names one, wherever its file is; in a project that declares none, every page is of the implicit type `page`, and names none. A fragment has no type, and can't use `type`.

### A page that replaces another
@available: ascribe 0.3.2

A page rewritten for a newer release, at a new path, says which page it takes the place of with `replaces`, written as a link to that page is:

```markdown
---
type: guide
title: Configure the agent
available: self-managed 3.5
replaces: configure-legacy.md
---
```

In a build versioned by a target ([versions](../versions/index.md)), a link to the old page goes to the new one in a line that has the new one, and the other way round, so the pages that link to either need no marks; and a site's list of a page's lines leads from one to the other. In any other build the two are different pages.

- It names a page, without `#` and an id. A page can't replace itself, and a chain of `replaces` can't come back to where it started.
- No line publishes both pages, and no line publishes two pages that replace the same page: mark the new page with the line it starts in, and the old one with where it ends, `self-managed (ga 3.0, removed 3.5)`.
- A fragment can't use it, and a content type can't declare a field named `replaces`.

## Directive lines

([SPEC §3.1]({repo}/blob/main/SPEC.md#31-syntax), [§3.2]({repo}/blob/main/SPEC.md#32-recognition))

A directive line has a name, optional attributes in braces, and an optional primary after a colon:

```text
@<name> {<attributes>}: <primary>
```

```markdown
@note {type=caution}: Back up your database first.
@include: _snippets/prerequisites.md
@steps
```

A line is a directive only when all of these hold:

- `@` is the first character of the line (up to three spaces of indentation are allowed, as for a heading).
- The name is a **known** directive: a built-in one, `end`, or a [project widget](directives.md#project-widgets) declared in `ascribe.toml`.
- The line isn't inside a code block or a raw HTML block.

Anything else is text. `@astrojs/react` at the start of a line and `support@example.com` in a sentence are both plain text. A line shaped like a directive but with an unknown name (`@warning:`) is also text, and Ascribe warns about it and suggests the closest directive; write `\@warning:` if you mean the text.

Spaces between the parts don't matter, and nothing else may be on the line: `@steps foo` is an error. Built-in names never contain a hyphen; project widget names always do.

## Attributes

([SPEC §3.3]({repo}/blob/main/SPEC.md#33-attributes))

Attributes are `key=value` pairs in braces, separated by commas. They use the same syntax after a directive's name, after an image, and at the end of a table row's first cell ([table rows](directives.md#table-rows)).

```markdown
@note {type=caution}: …
@include {heading=false}: guides/setup.md#install
![The settings page](settings.png){width=600}
```

- **Keys** are lowercase: a letter, then letters, digits, or hyphens. Each directive accepts only the keys its schema declares.
- **Values** are a token (`type=caution`, `since=3.4`, `width=600px`), a quoted string (`label="Using other images"`, with `\"` and `\\` inside), or, for keys declared as sets, a **value set** of tokens joined by `|` (`platform=cloud|on-prem`).
- A value must be quoted when it contains a space or any of `,` `|` `{` `}` `=` `"`.
- Booleans are written out: `open=true`. A key with no value is an error.
- `{}` is allowed and means no attributes.

A value's type comes from the directive's schema, never from how it looks: `3.10` is a string where the schema says string.

## Forms: line and container

([SPEC §3.5]({repo}/blob/main/SPEC.md#35-forms))

A directive is either a single line, or a **container** that holds blocks until `@end`. The line itself says which: **a colon with nothing after it opens a container**; any other directive line is a single line.

```markdown
@note {type=caution}: A one-line note.

@note {type=caution}:
A container note.

It holds several paragraphs.
@end
```

- `@end` closes the innermost open container. There are no named closers.
- A container must close before the block around it ends: its list item, block quote, group arm, or the document.
- A trailing colon on a directive with no container form is an error, and so is a container-only directive (such as `@variant`) without one.
- A text primary that happens to end in a colon is still a primary: `@note: Important:` is a one-line note.

Containers may nest. Ascribe warns when they're more than two levels deep ([SPEC §3.10]({repo}/blob/main/SPEC.md#310-nesting)).

## Groups

([SPEC §3.6]({repo}/blob/main/SPEC.md#36-groups))

Some directives, `@variant` among them, form **groups**: a run of openers of the same directive is one group of alternatives. Each opener starts an **arm** and ends the previous one, and one `@end` closes the whole group.

```markdown
@variant {deployment=cloud}:
Sign in to Quill Cloud and copy an API key.

@variant {deployment=self-managed}:
Point the agent at your server.
@end
```

A group can't be nested directly inside an arm of a group of the same directive. For `@variant`, put both dimensions on one arm instead.

## Titles

([SPEC §3.7]({repo}/blob/main/SPEC.md#37-titles))

A **title line** starts with `.` followed by a character that isn't a space or another `.`. It sits directly above a directive that takes a title (a note, `@details`, or a labeled `@variant` arm), with no blank line between.

```markdown
.Try it without installing
@note {type=tip}
You can run Quill in the browser with no local setup.
```

- A title line starts a block: it follows a blank line, a heading, a directive line, or the start of its container.
- A `.` line that isn't directly above a directive that takes a title is ordinary text, so `.NET is a framework` is safe. Directly above a directive that doesn't take a title, it stays text and Ascribe warns; start it with `\.` to silence the warning.
- A title whose text starts with a dot escapes it: `.\.NET` is the title `.NET`.

The editor shows title lines distinctly, so a paragraph that accidentally became a title is easy to spot.

## Binding: what a directive applies to

([SPEC §3.8]({repo}/blob/main/SPEC.md#38-binding))

A directive in line form applies to something, according to its **binding**:

| Binding | Applies to | Directives |
|---|---|---|
| Self | Its own primary, or nothing | `@include`; `@note` with a primary |
| Heading | The heading above it, and that heading's section | `@id` |
| Following block | The next block in the same container | `@steps`; `@details` and `@note` without a colon |
| Heading or block | At the top of a section, the section; anywhere else, the next block | `@available` |

**Heading-bound directives** go at the top of their section, directly under the heading, before any other content. Several may stack:

```markdown
## Streaming sync
@id: streaming-sync
@available: cloud, self-managed preview 3.4
```

**Following-block directives** go directly above the block they describe, touching it. A blank line between them is allowed but warned about, since it hides what the directive annotates. Several stack, and all apply to the block the last one touches. A heading can't be bound this way.

The short version: **at the top of a section, a directive describes the section; anywhere else, it describes the block it touches.**

## Lists and block quotes

([SPEC §3.9]({repo}/blob/main/SPEC.md#39-directives-inside-lists-and-blockquotes))

Directives follow CommonMark's container rules, like headings and code fences:

- A directive indented to a list item's content belongs to that item, and binds blocks within it.
- A container opens and closes within one list item, or contains whole lists.
- A directive line never continues a paragraph, so an unindented directive right after a list item ends the list. Ascribe warns when this happens.
- A directive indented four or more spaces past its container's content is part of an indented code block, and is text. Ascribe warns about that too.
- Block quotes work the same way, with `>` in place of indentation.

## Built-in directives

([SPEC §4]({repo}/blob/main/SPEC.md#4-built-in-directives))

| Directive | Forms | Primary | Purpose |
|---|---|---|---|
| [`@id`](directives.md#id) | line | an id | Give a heading a stable id |
| [`@include`](directives.md#include) | line | a path | Include a file or a section |
| [`@variant`](directives.md#variant) | container, grouped | none | Mark alternative content |
| [`@available`](directives.md#available) | line | an availability spec or feature key | Say where content applies |
| [`@note`](directives.md#note) | line, container | text, optional | A callout |
| [`@steps`](directives.md#steps) | line | none | Mark an ordered list as a procedure |
| [`@details`](directives.md#details) | line, container | none | Collapsible content |
| [`@snippet`](directives.md#snippet) | line | an address | A code example from a file |
| [`@intended`](directives.md#intended) | line | a reason | Say a check's problem is intended |

### `@id`

([SPEC §4.1]({repo}/blob/main/SPEC.md#41-id))

Gives a heading an explicit, stable id, used for its anchor and by links and includes.

```markdown
## Configuration
@id: config-setup
```

- It goes directly under its heading.
- An id uses letters, digits, hyphens, underscores, and periods, so an existing anchor such as `ece_setup` or `v1.2` can be kept.
- Ids must be unique on a page, including ids that come from included fragments.

Without `@id`, a heading's id is its slug, computed from its text. Add `@id` to a heading whose text contains a phrase or duplicates another heading's: Ascribe warns about both, because their ids can change without the heading being edited.

#### Permalinks
@available: ascribe 0.3.0

`{permalink=true}` makes the id a **permalink** as well: a name for the section that something outside the docs can link to, which keeps working when the page is renamed, moved, or rewritten.

```markdown
## `ascribe check`
@id {permalink=true}: cli-check
```

- **Each permalink has an address of its own,** `go/<id>` under the site's base path, or under [`permalink-path`](../content-model/output.md#consumer), which forwards to the section wherever it is. With [`@ascribed/astro`](../permalinks/index.md#with-astro), `/go/cli-check/` sends the reader to `/docs/reference/cli/#cli-check`. A product links to the address, not to the page: `ascribe check --help` ends with `https://ascribed-dev.com/go/cli-check/`.
- **An address can say what its reader uses,** with the content model's [dimensions](../content-model/content.md#dimensions) as parameters: `/go/install-agent/?pm=pnpm` opens the section with the `pnpm` tab showing, as if the reader had picked it. A parameter the docs don't know is ignored. [Linking from a product](../permalinks/index.md#linking-from-a-product) has more.
- **The id is still the heading's anchor,** and links inside the docs name it as they name any id: `cli.md#cli-check`. A permalink is for links from outside the docs; inside, link to the file.
- **A build can publish a permalink in one place only,** and `ascribe check` reports one it publishes twice (`permalink-duplicate`). Names that differ only in case, such as `Setup` and `setup`, are one name, since a site may serve every spelling of an address at one. A heading in a fragment is published on every page that includes it, so a marked heading belongs in a fragment only one page of each build includes. Two pages that no build publishes together may share a permalink.
- **A permalink that's removed is reported when the project commits its list,** `ascribe.permalinks`, which [`ascribe permalinks sync`](../examples/sources.md#cli-permalinks-sync) writes. Delete the id or the attribute, and `ascribe check` warns (`permalink-removed`), since the address stops working and something outside the docs may still link to it. Without the list, nothing reports it. Moving the heading to another page isn't removing it: run `sync` again, and the list follows. A permalink removed on purpose is retired, with a reason its address shows, and one name can lead to another as an alias, both in [`[permalinks]`](../content-model/checks.md#20-permalinks). [The permalinks list](../contracts/permalinks-list.md) has the details.

A whole page can have a permalink too, in its frontmatter, and its address leads to the top of the page:

```yaml
---
title: Command reference
permalink: cli-reference
---
```

A page's permalink follows the rules of an id, and shares one set of names with the headings': a page and a heading can't have the same one in a build. A fragment can't have one, since it isn't published on its own. The site output keeps `permalink` in the page's frontmatter, so a layout can show the page's permanent address.

### `@include`

([SPEC §4.2]({repo}/blob/main/SPEC.md#42-include))

Includes another file, or one heading's section of it, in place.

```markdown
@include: _snippets/prerequisites.md
@include {heading=false}: guides/setup.md#install
```

- **Paths** are relative to the file they're written in, or to the content root when they start with `/`. Only source files (`.md` files under the content root) can be included. A symbolic link is followed, but only to another source file: a source file whose link leads out of the content root, or to a hidden, non-Markdown, or other project's file, can't be read, so no other file on disk ends up in a page.
- `#id` includes only the section of the heading with that id. `{heading=false}` leaves out that section's own heading; it has no effect without `#id`, and Ascribe warns.
- Included content becomes part of the page: phrases and [builds](../content-model/output.md#16-buildsname) apply to it, and its ids must not collide with the page's.
- **Relative paths inside a fragment resolve from the fragment**, not the page including it, so a fragment's links and images work wherever it's included.
- Included headings keep the levels they're written with.
- An include cycle is an error.

@available: ascribe 0.3.0
A page's includes can make it only so large: far larger than any real page, but not as large as a few files that each include the next twice make it, doubling the page at every step. The include in the page's own file that takes the page past the limit is an error, `include-limit`, whose message gives the limit, and it and every include after it on the page are left out.

Links target pages, not fragments: link to the page that includes a fragment.

@available: ascribe 0.2.0
A link can name a heading the fragment brings in, as `setup.md#prerequisites`, just as it names one of the page's own.

### `@variant`

([SPEC §4.3]({repo}/blob/main/SPEC.md#43-variant))

Marks alternatives: content that differs by a dimension declared in `ascribe.toml`, such as a package manager or a deployment.

**Dimensional arms** carry dimension values as attributes:

````markdown
@variant {pm=npm}:
```shell
npm install -g @quill/agent
```
@variant {pm=pnpm}:
```shell
pnpm add -g @quill/agent
```
@end
````

**Labeled arms** carry a title instead, for one-off alternatives that aren't a declared dimension:

```markdown
.Using Docker Hardened Images
@variant:
…

.Using other images
@variant:
…
@end
```

- `@variant` is container-only and grouped: every arm's opener ends in a colon, and one `@end` closes the group.
- Each arm has attributes or a title, never both and never neither, and a group's arms are all one kind.
- A value set (`{platform=cloud|on-prem}`) matches any of its values. Several keys on one arm (`{deployment=cloud, pm=npm}`) must all match.
- The arms of a group share at least one dimension.
- `@variant` varies whole blocks. For a single term that differs, use a [phrase](directives.md#phrases).

Each build decides what happens to a group: `switch` keeps every arm, for readers to switch between (the site shows tabs), and a selection such as `{ deployment = "cloud" }` keeps only the matching arms. See [builds](../content-model/output.md#16-buildsname).

#### Whole-page variants

When a whole page differs by dimension, write separate pages and give each a `variant` in its frontmatter:

```yaml
---
title: Connect to Quill Cloud
variant:
  deployment: cloud
---
```

A build that selects another deployment drops the page.

### `@available`

([SPEC §4.4]({repo}/blob/main/SPEC.md#44-available))

Says where content applies and its lifecycle state. Unlike `@variant`, it doesn't select content: every reader sees it, marked with its availability.

```markdown
@available: cloud, self-managed preview 3.4
```

- At the top of a section, it applies to the section. Anywhere else, it applies to the block it touches.
- For a whole page, use the frontmatter key `available`, with the same spec.
- Several `@available` lines on one heading or block all apply: the content is available only where all of them allow it.

#### Availability specs

A spec is a comma-separated list of **targets**, each with an optional lifecycle:

| Spec | Meaning |
|---|---|
| `cloud` | Generally available on `cloud` |
| `self-managed 3.3` | Generally available on `self-managed` since version 3.3 |
| `self-managed preview 3.4` | In preview on `self-managed` since version 3.4 |
| `self-managed (preview 3.3, ga 3.5, deprecated 4.0)` | A history: each state lasts until the next begins |
| `cloud, self-managed preview 3.3` | Two targets |

- A target is a dimension value, or a dimension's name, which stands for all its values. A dimension name takes a state but no version: `deployment beta`.
- A state is a lifecycle state: `preview`, `beta`, `ga`, `deprecated`, `removed`, or one the project declares. No state means `ga`.
- A history is in chronological order. A versionless target (declared in `ascribe.toml`) takes one state and no version.
- There are no ranges, alternatives, or negation. Commas separate targets; `|` belongs to `@variant`.

**Scope.** A section or block's spec can't go beyond the page or section around it: it can't add a target the enclosing scope doesn't list, or name an earlier version. Content with no spec inherits its enclosing scope's availability.

**Feature keys.** A spec that is a single word matching a key in `[features]` stands for that feature's declared spec, so a feature going GA takes one edit in `ascribe.toml`:

```markdown
@available: streaming-sync
```

Each build decides what happens to availability: `badge` keeps everything and marks it, and a filter such as `{ filter = "self-managed 3.3" }` removes what isn't available there.

#### Table rows
@available: ascribe 0.2.0

A table row takes a spec from an attribute block at the end of its first cell, with the key `available`. Quote a spec that has spaces or commas:

```markdown
| Option | Meaning |
|---|---|
| `timeout` | Seconds to wait for the agent. |
| `stream` {available="self-managed preview 3.4"} | Streams results as they arrive. |
| `sync` {available=streaming-sync} | Keeps the local copy in sync. |
```

- The block goes last in the row's first cell, after a space, and isn't part of the cell's text. The header row can't have one.
- `available` is the only key a row takes. Its spec is checked like an `@available` line's, and can't go beyond the table's own availability.
- A `badge` build marks the row at the end of its first cell, and a filter build removes the row where it isn't available.

There's no form for part of a sentence: a filter would break the sentence, and a badge in the middle of one gets in the way. Write a value that differs by release as prose, such as "glibc 2.28 or later (2.39 for 0.1.1 and earlier)".

### `@note`

([SPEC §4.5]({repo}/blob/main/SPEC.md#45-note))

A callout. Its `type` is `note` (the default), `tip`, `important`, `warning`, `caution`, or a type the project declares in `[notes]`.

```markdown
@note {type=caution}: Back up your database first.

.Try it without installing
@note {type=tip}
You can run Quill in the browser with no local setup.

@note {type=warning}:
Several blocks,

- including lists.
@end
```

- With a primary, the primary is the note. It can continue onto the next lines, like a paragraph.
- Without a colon, the note is the next block.
- With a trailing colon, the note is a container.
- A note may have a title.

### `@steps`

([SPEC §4.6]({repo}/blob/main/SPEC.md#46-steps))

Marks an ordered list as a procedure. It goes directly above the list.

```markdown
@steps
1. Install the agent package.
2. Verify the install.
3. Create `quill.yaml`.
```

Directives inside the list's items follow the [list rules](directives.md#lists-and-block-quotes): indent them to the item's content.

### `@details`

([SPEC §4.7]({repo}/blob/main/SPEC.md#47-details))

Content readers can expand or collapse. The title is required: it's what readers see while the content is collapsed.

```markdown
.Show the full configuration reference
@details:
…
@end
```

Without a colon, `@details` applies to the next block; with one, it's a container.

### `@snippet`
@available: ascribe 0.2.0

([SPEC §4.8]({repo}/blob/main/SPEC.md#48-snippet))

Takes a code example from a file outside the content, and puts it in the page as a fenced code block. There's no copy: Ascribe reads the file whenever it checks, builds, or diffs, so changing the file changes every page that uses it.

```markdown
@snippet: code:examples/quill/ascribe.toml#dimensions
@snippet {lang=shell, title="Install"}: code:scripts/install.sh
```

- **The address** is `<source>:<path>`, optionally followed by `#<region>`. `<source>` is a source declared in `ascribe.toml` ([`[sources.<name>]`](../content-model/output.md#18-sourcesname)), and `<path>` is the file's path relative to the source's folder (for a [source in another repository](../content-model/output.md#a-source-in-another-repository), its path in that repository), with `/` on every platform. There's no relative form: an address doesn't depend on where the page is.
- The file must exist with exactly that name, the source's `include` and `ignore` must take it in, and it must be text. A symbolic link is followed, but only to a file in the source's folder that `include` and `ignore` take in.
- Without `#<region>`, the snippet is the whole file.
- **Attributes:** `lang` sets the code block's language, which is otherwise the file's extension (`toml` for `ascribe.toml`); `title` gives the code block a title; `phrases=true` substitutes phrases in the code, as in a fence that opts in ([Phrases](directives.md#phrases)).
- It's a block, allowed wherever a code block is: in a list item, a note, or a variant's arm. A `@note`, `@details`, or widget directly above it applies to it.

Mark a region in the code with tags in comments. They're [Bluehawk](https://github.com/mongodb-university/Bluehawk)'s, so files already tagged for it work as they are:

```toml
# :snippet-start: dimensions
[dimensions.platform]
values = ["linux", "macos", "windows"]
labels = { macos = "macOS" }  # :remove:
# :snippet-end:
```

- `:snippet-start: <name>` opens a region, and `:snippet-end:` closes the innermost open one. `:snippet-end: <name>` closes the region with that name, so regions can overlap as well as nest. A region name is letters, digits, `-`, `_`, and `.`, and is used once in a file.
- `:remove-start:` and `:remove-end:` leave out the lines between them, and a line that ends with `:remove:` in a comment is left out itself.
- Tag lines and removed lines never appear in a snippet, nor do blank lines at the start and end of what's left, so a formatter's blank line before an end tag doesn't end up in the page. What's left is dedented by its common indentation.
- A tag counts only in a line comment: `//` (C, Go, Java, JavaScript, Rust, TypeScript, and others), `#` (Python, Ruby, shell, TOML, YAML, and others), `--` (Lua, SQL, Haskell, Elm), `;` (INI, Lisp), or `<!-- -->` (HTML, XML, Markdown). Astro, Svelte, and Vue components take either `//` or `<!-- -->`, so a region can be marked in the frontmatter or script and in the markup. A file whose extension isn't in the [table in the spec]({repo}/blob/main/SPEC.md#48-snippet) can be used whole, but not by region.
- Bluehawk's other tags (`state`, `replace`, `uncomment`, and `emphasize`) are reserved. A file that uses one can't be used by a snippet yet, so its code is never shown with a tag left in.

Every problem is reported at the `@snippet` line: an address without a source, a source or file that doesn't exist, a region that doesn't exist (with the names the file has), and tags that don't balance, with the tag's line in the code file as related information. See [Source files](../checks/diagnostics/directives.md) in the diagnostics reference.

### `@intended`
@available: ascribe 0.3.0

([SPEC §4.9]({repo}/blob/main/SPEC.md#49-intended))

Says that a problem a check reports is intended: you've looked at it and are keeping it. Write the check's name and why.

```markdown
@intended {check=link-dead}: help.example.com refuses link checkers; checked by hand.
Read the [vendor's deployment notes](https://help.example.com/deploy) first.
```

- **Only review checks.** A check whose next step is *review* reports something that may be fine as it is, such as a page nothing links to. Only those can be acknowledged; a problem that's always wrong, such as a link to a page that doesn't exist, is fixed instead. The [diagnostics reference](../checks/diagnostics.md) says where each review check is acknowledged.
- **Where it's written** depends on the check, which reports its problems at one kind of place:
  - **A block:** `@intended` directly above it, as above. It covers the check's problems anywhere in the block, including inside a container. Above a block of a fragment, it covers that block in every page that includes it.
  - **A page or a fragment:** the `intended` frontmatter key, a list of entries with a `check` and a `reason`:

    ```yaml
    intended:
      - check: page-orphan
        reason: Linked from the site's sidebar, which Ascribe can't see.
    ```

  - **A content model entry or an image:** an [`[[intended]]` table](../content-model/checks.md#intended-acknowledgements) in `ascribe.toml`.
- **The reason is required,** and is the rest of the line. It's for the next person who reads the page, and `ascribe check` lists it with the problem it covers.
- **It changes no output.** Nothing renders for `@intended`, and the `intended` key isn't in any output's frontmatter.

An acknowledged problem isn't reported, but `ascribe check` and `ascribe build` count it in their summary (`1 acknowledged`), and list it, with its reason, in their [JSON report](../command/index.md). When an acknowledgement covers nothing in any build, because the problem was fixed or the check changed, `ascribe check` reports it as advice, `intended-unused`, with a fix that removes it. That's reported only when every build is checked: not with `--build` or `--editor-build`, and not in the editor.

In the editor, a review check's problem has a quick fix, **Mark as intended, and write why**, that writes the acknowledgement in the right place for you to fill in the reason. Its placeholder reason is reported as an error until you replace it, so the fix alone silences nothing.

## Project widgets

([SPEC §6]({repo}/blob/main/SPEC.md#6-project-widgets))

A project widget is a directive a project declares in `ascribe.toml`, for needs the built-in directives don't cover. Its name contains a hyphen, so it's easy to tell from a built-in:

```markdown
@quill-labspace {lab=first-sync}
```

A declared widget works exactly like a built-in directive: its declaration sets its forms, primary, attributes, binding, title, and whether it's grouped. In the site output it becomes a custom element with its name; in plain Markdown, its declared fallback text. See [`[widgets.<name>]`](../content-model/content.md#14-widgetsname).

## Phrases

([SPEC §5.1]({repo}/blob/main/SPEC.md#51-phrases))

A phrase inserts a string declared in `ascribe.toml`'s `[phrases]`, such as a product name, a version, or a URL base:

```markdown
Sign in to {cloud} and copy an API key.
See the [streaming API reference]({api}streaming).
```

- `{key}` is a phrase only when `key` is declared; otherwise it's literal text. Ascribe warns about undeclared `{key}` text in prose, since declaring the key later would silently change it. Write `\{key}` for text you mean.
- Phrases apply in prose, headings, link text, and link destinations, and in frontmatter fields declared with `phrases = true`.
- In a fenced code block, only when its info string has `phrases=true`: ` ```yaml phrases=true `. Never in code spans, indented code, or raw HTML.
- Values are inserted as literal text.
- `{{key}}` puts the value between literal braces, which is usually a leftover from another tool's templates; Ascribe warns.

## Links

([SPEC §5.2]({repo}/blob/main/SPEC.md#52-links))

Links point at **files**, not URLs. Ascribe writes each output's URLs.

```markdown
[Rotate API keys](keys.md#rotate-keys)
See [](keys.md#rotate-keys).
```

- Paths are relative to the file, or to the content root when they start with `/`. `#id` names a heading on the target page by its id: one of its own, or one from a fragment it includes.
- The file, and the id, must exist. A link to a fragment is an error: link to a page that includes it.
- **Empty link text** is filled in with the target's title: the heading's text with an id, or the page's `title`.
- External URLs (`https://…`) pass through unchanged.
- A destination that looks like a published URL (`/guides/install/`) instead of a file gets a warning, and the editor offers to convert it.
- In each build, a link's target page must be published. To link from shared content to a page a build drops, put the link in a `@variant` arm the same build removes.

### Links between lines
@available: ascribe 0.3.2

In a build versioned by a target ([versions](../versions/index.md)), a link to a page a line doesn't publish isn't an error. It goes, in order, to:

1. The page that takes its place in the line: the page that [replaces it](directives.md#a-page-that-replaces-another), or that it replaces, following a chain of them to the page the line has.
2. The page in the nearest line that has it, the closest newer line first, then the closest older: a **link out** of the line. Its text stays what you wrote, and the output says which line it leads to: the site output marks it for the stylesheet, which shows "(3.5)" after it, and the plain output writes " (3.5)" after the link.

It's [`link-page-dropped`](../checks/diagnostics/pages.md) only when no live line publishes the page. A link's `#id` is kept when the page it lands on has a heading with that id, and dropped otherwise, with [`link-fragment-dropped`](../checks/diagnostics/content-checks.md) advice. A link from the current line out to an older line gets [`link-into-older-line`](../checks/diagnostics/content-checks.md) advice: the current line usually needs a page there. `ascribe link --from <page> <target>` says where a link goes in each line.

## Images

([SPEC §5.3]({repo}/blob/main/SPEC.md#53-images))

Images are CommonMark images. Attributes, such as a width, go in a block directly after the image, with no space between; `ascribe.toml`'s `[images.attributes]` declares which are accepted.

```markdown
![The Quill settings page](settings.png){width=600}
```

- A local image must exist, and names must match exactly, including case.
- Ascribe warns about an image without alt text.
- The image is copied into each output, so outputs don't need the source files.

## Heading ids

([SPEC §5.5]({repo}/blob/main/SPEC.md#55-heading-ids))

Every heading has an id: its `@id` if it has one, and otherwise a slug of its text, computed the way the site's slugger computes it (GitHub's, for Astro). Links and includes name headings by these ids. Ascribe warns about a heading whose id could change without the heading being edited (its text has a phrase, or duplicates another heading's) and about a heading whose slug is empty; add `@id` to fix each.

## Glossary terms

([SPEC §5.4]({repo}/blob/main/SPEC.md#54-glossary-terms))

When `ascribe.toml` declares a glossary, Ascribe links terms to their definitions in each built page. Authors don't mark terms in source. See [`[glossary]`](../content-model/content.md#12-glossary).

## Escapes

([SPEC §2.3]({repo}/blob/main/SPEC.md#23-escapes))

Ascribe uses CommonMark's backslash escapes, so an escaped character renders as itself in any Markdown renderer:

| Escape | Prevents |
|---|---|
| `\@` | A directive |
| `\{` | A phrase |
| `\.` | A title line |

They're rarely needed: a directive needs a known name at the start of a line, a phrase needs a declared key, and a title needs a directive on the next line.

## Canonical form

([SPEC §8.3]({repo}/blob/main/SPEC.md#83-canonical-form))

Ascribe accepts any spacing the syntax allows, and has one canonical spelling for each construct: one space between a name and `{`, no spaces inside braces or around `=` and `|`, attributes in the order their schema declares, values quoted only when needed, and no blank line between a directive and its block. `ascribe fmt` rewrites source into canonical form, changing only Ascribe constructs and never how a page renders; `ascribe fmt --check` reports files that would change.

A general Markdown formatter that reflows paragraphs, such as Prettier with `proseWrap: always`, doesn't know that title and directive lines start new blocks, and joins them with the text below. Format Ascribe sources with `ascribe fmt`, and exclude them from other formatters.
