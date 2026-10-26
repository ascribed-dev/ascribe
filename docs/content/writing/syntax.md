---
type: reference
title: Directive syntax
description: "How a directive is written: its line, attributes, forms, groups, titles, what it applies to, and escapes."
---

Ascribe source is CommonMark with a few additions: **directives**, lines that start with `@` and annotate the content around them; **title lines**, which give a directive a title; and **phrases**, reusable strings written `{key}`. Everything else is ordinary Markdown, and an Ascribe page read by a plain Markdown renderer still reads sensibly.

This page is how a directive is written. Each built-in directive is in the [directive reference](directives.md), [phrases, links, and images](inline.md) are on a page of their own, and so is [frontmatter](frontmatter.md). The [Ascribe specification]({repo}/blob/main/SPEC.md) is the normative definition; each section links to its part of the spec.

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

Anything else is text. `@astrojs/react` at the start of a line and `support@example.com` in a sentence are both plain text. A line shaped like a directive but with an unknown name (`@warning:`) is also text, and Ascribe warns about it and suggests the closest directive; write `\@warning:` if you mean the text. Names are case-sensitive, so `@Note:` is text too, and Ascribe suggests `@note`.

Spaces between the parts don't matter, and nothing else may be on the line: `@steps foo` is an error. Built-in names never contain a hyphen; project widget names always do.

Attributes written without their braces, as in `@note type=warning: You need Node 20.`, are the same error, and it's the one reported for the line: the errors that would follow from it, such as the note binding the heading below, aren't. When the keys are the directive's attributes, the fix adds the braces (`@note {type=warning}: …`), and `ascribe check --fix` applies it when a primary follows the colon.

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
- Booleans are written out: `permalink=true`, `heading=false`. A key with no value is an error.
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
| Self | Its own primary, or nothing | `@include`, `@snippet`; `@note` with a primary |
| Heading | The heading above it, and that heading's section | `@id`, `@term`; `@summary` with a primary |
| Following block | The next block in the same container | `@steps`; `@details`, `@note`, and `@summary` without a colon |
| Heading or block | At the top of a section, the section; anywhere else, the next block | `@available`, `@intended`, `@memo` |

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
