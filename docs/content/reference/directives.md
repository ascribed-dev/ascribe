---
title: Directive reference
description: "The language: directives, attributes, titles, phrases, links, and images."
---

Ascribe source is CommonMark with a few additions: **directives**, lines that start with `@` and annotate the content around them; **title lines**, which give a directive a title; and **phrases**, reusable strings written `{key}`. Everything else is ordinary Markdown, and an Ascribe page read by a plain Markdown renderer still reads sensibly.

This page describes the language as you write it. The [Ascribe specification]({repo}/blob/main/SPEC.md) is the normative definition; each section links to its part of the spec.

## Contents

- [Files](#files)
- [Directive lines](#directive-lines)
- [Attributes](#attributes)
- [Forms: line and container](#forms-line-and-container)
- [Groups](#groups)
- [Titles](#titles)
- [Binding: what a directive applies to](#binding-what-a-directive-applies-to)
- [Lists and block quotes](#lists-and-block-quotes)
- [Built-in directives](#built-in-directives): [`@id`](#id), [`@include`](#include), [`@variant`](#variant), [`@available`](#available), [`@note`](#note), [`@steps`](#steps), [`@details`](#details), [`@snippet`](#snippet)
- [Project widgets](#project-widgets)
- [Phrases](#phrases)
- [Links](#links)
- [Images](#images)
- [Heading ids](#heading-ids)
- [Glossary terms](#glossary-terms)
- [Escapes](#escapes)
- [Canonical form](#canonical-form)

## Files

([SPEC §2]({repo}/blob/main/SPEC.md#2-documents))

A project's source files are the `.md` files under its **content root** (`docs/` unless `ascribe.toml` says otherwise). Files and directories whose names start with `.` are skipped, and so is a directory below the content root that holds an `ascribe.toml`, with everything in it, unless it's the project's own folder: that is another project, with sources of its own. Each file is a page or a fragment:

- A **page** is published on its own.
- A **fragment** exists only to be included in other files. A file is a fragment when any part of its path starts with `_` (`_warning.md`, `_snippets/setup.md`), or when it matches one of the `[fragments] patterns` in `ascribe.toml`.

A file may start with YAML frontmatter between `---` lines. Its content type in `ascribe.toml` says which keys it takes; every page takes a `title`. Two keys are reserved for pages: `available` ([availability](#available)) and `variant` ([whole-page variants](#whole-page-variants)).

```markdown
---
title: Install the Quill agent
available: cloud, self-managed preview 3.3
---
```

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
- The name is a **known** directive: a built-in one, `end`, or a [project widget](#project-widgets) declared in `ascribe.toml`.
- The line isn't inside a code block or a raw HTML block.

Anything else is text. `@astrojs/react` at the start of a line and `support@example.com` in a sentence are both plain text. A line shaped like a directive but with an unknown name (`@warning:`) is also text, and Ascribe warns about it and suggests the closest directive; write `\@warning:` if you mean the text.

Spaces between the parts don't matter, and nothing else may be on the line: `@steps foo` is an error. Built-in names never contain a hyphen; project widget names always do.

## Attributes

([SPEC §3.3]({repo}/blob/main/SPEC.md#33-attributes))

Attributes are `key=value` pairs in braces, separated by commas. They use the same syntax after a directive's name and after an image.

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
| [`@id`](#id) | line | an id | Give a heading a stable id |
| [`@include`](#include) | line | a path | Include a file or a section |
| [`@variant`](#variant) | container, grouped | none | Mark alternative content |
| [`@available`](#available) | line | an availability spec or feature key | Say where content applies |
| [`@note`](#note) | line, container | text, optional | A callout |
| [`@steps`](#steps) | line | none | Mark an ordered list as a procedure |
| [`@details`](#details) | line, container | none | Collapsible content |
| [`@snippet`](#snippet) | line | an address | A code example from a file |

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

### `@include`

([SPEC §4.2]({repo}/blob/main/SPEC.md#42-include))

Includes another file, or one heading's section of it, in place.

```markdown
@include: _snippets/prerequisites.md
@include {heading=false}: guides/setup.md#install
```

- **Paths** are relative to the file they're written in, or to the content root when they start with `/`. Only source files (`.md` files under the content root) can be included.
- `#id` includes only the section of the heading with that id. `{heading=false}` leaves out that section's own heading; it has no effect without `#id`, and Ascribe warns.
- Included content becomes part of the page: phrases and builds apply to it, and its ids must not collide with the page's.
- **Relative paths inside a fragment resolve from the fragment**, not the page including it, so a fragment's links and images work wherever it's included.
- Included headings keep the levels they're written with.
- An include cycle is an error.

Links target pages, not fragments: link to the page that includes a fragment.

@available: next
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
- `@variant` varies whole blocks. For a single term that differs, use a [phrase](#phrases).

Each build decides what happens to a group: `switch` keeps every arm, for readers to switch between (the site shows tabs), and a selection such as `{ deployment = "cloud" }` keeps only the matching arms. See [builds](content-model.md#16-buildsname).

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

Directives inside the list's items follow the [list rules](#lists-and-block-quotes): indent them to the item's content.

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
@available: next

([SPEC §4.8]({repo}/blob/main/SPEC.md#48-snippet))

Takes a code example from a file outside the content, and puts it in the page as a fenced code block. There's no copy: Ascribe reads the file whenever it checks, builds, or diffs, so changing the file changes every page that uses it.

```markdown
@snippet: code:examples/quill/ascribe.toml#dimensions
@snippet {lang=shell, title="Install"}: code:scripts/install.sh
```

- **The address** is `<source>:<path>`, optionally followed by `#<region>`. `<source>` is a source declared in `ascribe.toml` ([`[sources.<name>]`](content-model.md#18-sourcesname)), and `<path>` is the file's path relative to the source's folder (for a [source in another repository](content-model.md#a-source-in-another-repository), its path in that repository), with `/` on every platform. There's no relative form: an address doesn't depend on where the page is.
- The file must exist with exactly that name, the source's `include` and `ignore` must take it in, and it must be text. A symbolic link is followed, but only to a file in the source's folder that `include` and `ignore` take in.
- Without `#<region>`, the snippet is the whole file.
- **Attributes:** `lang` sets the code block's language, which is otherwise the file's extension (`toml` for `ascribe.toml`); `title` gives the code block a title; `phrases=true` substitutes phrases in the code, as in a fence that opts in ([Phrases](#phrases)).
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
- Tag lines and removed lines never appear in a snippet. What's left is dedented by its common indentation.
- A tag counts only in a line comment: `//` (C, Go, Java, JavaScript, Rust, TypeScript, and others), `#` (Python, Ruby, shell, TOML, YAML, and others), `--` (Lua, SQL, Haskell, Elm), `;` (INI, Lisp), or `<!-- -->` (HTML, XML, Markdown). A file whose extension isn't in the [table in the spec]({repo}/blob/main/SPEC.md#48-snippet) can be used whole, but not by region.
- Bluehawk's other tags (`state`, `replace`, `uncomment`, and `emphasize`) are reserved. A file that uses one can't be used by a snippet yet, so its code is never shown with a tag left in.

Every problem is reported at the `@snippet` line: an address without a source, a source or file that doesn't exist, a region that doesn't exist (with the names the file has), and tags that don't balance, with the tag's line in the code file as related information. See [Source files](diagnostics.md#source-files) in the diagnostics reference.

## Project widgets

([SPEC §6]({repo}/blob/main/SPEC.md#6-project-widgets))

A project widget is a directive a project declares in `ascribe.toml`, for needs the built-in directives don't cover. Its name contains a hyphen, so it's easy to tell from a built-in:

```markdown
@quill-labspace {lab=first-sync}
```

A declared widget works exactly like a built-in directive: its declaration sets its forms, primary, attributes, binding, title, and whether it's grouped. In the site output it becomes a custom element with its name; in plain Markdown, its declared fallback text. See [`[widgets.<name>]`](content-model.md#14-widgetsname).

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

When `ascribe.toml` declares a glossary, Ascribe links terms to their definitions in each built page. Authors don't mark terms in source. See [`[glossary]`](content-model.md#12-glossary).

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
