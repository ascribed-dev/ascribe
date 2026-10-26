# Ascribe Specification

Version 0.1

## Contents

1. [Introduction](#1-introduction)
2. [Documents](#2-documents)
3. [Directives](#3-directives)
4. [Built-in directives](#4-built-in-directives)
5. [Inline constructs](#5-inline-constructs)
6. [Project widgets](#6-project-widgets)
7. [Content model](#7-content-model)
8. [Validation](#8-validation)
9. [Compilation](#9-compilation)
10. [Authoring environment](#10-authoring-environment)
11. [Versioning](#11-versioning)

Appendices:

- [A. Grammar](#appendix-a-grammar)
- [B. Complete example](#appendix-b-complete-example)
- [C. Design rationale](#appendix-c-design-rationale)
- [D. References](#appendix-d-references)

---

## 1. Introduction

### 1.1 Purpose and scope

Ascribe is a markup language, content model, and toolchain for documentation written as code. It makes a documentation set's structure (its pages, reusable content, variants, cross-references, and metadata) explicit, validated while authoring, and compiled for publication.

This specification defines:

- the **Ascribe markup language**: CommonMark extended with directives and a small set of inline constructs;
- the **content model**: the schema contract that declares what a documentation set may contain;
- **validation**: the diagnostics a conforming processor reports;
- **compilation**: how a processor resolves a documentation set and emits output;
- requirements for the **authoring environment**.

The reference implementation is a compiler written in Rust, which runs as a command-line tool and as a language server, and a VS Code extension that hosts the language server. The command line and the editor share one parser and validator.

### 1.2 Design principles

These principles govern the language. They are non-normative, but every normative rule in this document follows from them.

1. **Authoring should feel like writing, not programming.** Source files are markdown a person can read without any tooling. Where source readability and a renderer's display conflict, source readability wins.
2. **Directives annotate content structure.** They mark what content *is* (a callout, a procedure, a variant, a reusable region), never how it looks and never what it computes.
3. **No behavior.** The language has no conditionals, loops, variables, operators, or computed content. Variance is declarative membership, not evaluation.
4. **Structure over grammar.** When a need can be met by a file convention or a content-model declaration instead of new syntax, it is.
5. **Containers are rare.** Most directives occupy a single line. Nesting is discouraged, and every nesting need has a flat alternative.
6. **Constrain the grammar, don't strangle it.** `@` marks directives, but other constructs use whatever notation reads best (for example, `{key}` for phrases).
7. **Validate while authoring.** The rules a build enforces are the same rules the editor reports as you type.
8. **Strict inside, tolerant outside.** Processors are strict about what Ascribe source means, not about how it's spaced; compiled output degrades to readable plain markdown.

### 1.3 Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHALL**, **SHALL NOT**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY**, and **OPTIONAL** in this document are to be interpreted as described in RFC 2119 and RFC 8174 when, and only when, they appear in all capitals.

- A **conforming document** is an Ascribe source file that produces no errors (§8) under a given content model.
- A **processor** is any software that reads Ascribe source: a parser, validator, compiler, or authoring environment.
- A **conforming processor** parses documents as this specification describes and reports every error listed in §8. It MAY report additional warnings.

Sections marked *non-normative*, examples, and notes are informative.

### 1.4 Relationship to CommonMark

An Ascribe document is a CommonMark document. Everything CommonMark defines keeps its meaning, except where this specification assigns meaning to text CommonMark treats as ordinary paragraph content (directive lines, title lines, phrases, and attribute blocks after images and at the end of a table row's first cell).

Processors MAY support common CommonMark extensions, such as GitHub Flavored Markdown tables. This specification doesn't depend on them.

### 1.5 Notational conventions

- Syntax is given in ABNF (RFC 5234) in [Appendix A](#appendix-a-grammar); sections refer to its rules by name.
- `SP` is a single space (U+0020). "Line start" means the first character after any indentation or blockquote markers required by the enclosing CommonMark container, allowing up to three further spaces of indentation, as CommonMark allows before a heading (§3.9).
- Examples show Ascribe source unless labeled otherwise.

---

## 2. Documents

### 2.1 Files

An Ascribe source file is a CommonMark file with the extension `.md`. A file MAY begin with YAML frontmatter delimited by lines containing only `---`. The content model (§7) defines which frontmatter keys each content type accepts; this specification reserves eight keys: `type`, `available` (§4.4), `variant` (§4.3), `intended` (§4.9), `permalink`, `listing`, `replaces`, and `nav-label` (below).

- Frontmatter MUST be valid YAML. A reserved key whose value isn't the shape its section defines (an availability spec, a mapping of dimensions to values, a name, one of `listing`'s values, a link destination, or a string on one line) is a frontmatter value of the wrong type (§8.2).
- `type` says which content type a page is (§7.2). Its value is the name of a type the content model declares. In a project that declares types, every page MUST name one, and a page that names none, or names one that isn't declared, is an error (§8.2). In a project that declares none, its pages have the implicit type `page` and name no type. Tools that write frontmatter put `type` first.

  ```yaml
  ---
  type: how-to
  title: Rotate an API key
  ---
  ```
- `permalink` gives a page a **permalink**: a name for the page that something outside the documentation can link to, which a build publishes at an address of its own that leads to the page wherever it is (§9.5). Its value is a name with the characters of an `@id` (§4.1), and it shares one set of names with the headings' permalinks: a build MUST publish each name once, whether on a page or a heading, and names that differ only in case are one name.

  ```yaml
  ---
  title: Command reference
  permalink: cli-reference
  ---
  ```
- `listing` says whether a page is offered to readers that aren't people, such as search engines and AI agents. Its value is one of `listed`, `unquoted`, and `unlisted`, in that order from open to closed; a page that doesn't set it is `listed`. A `listed` page is offered: a compiler lists it in `llms.txt` (§9.4). An `unquoted` page is listed and asks not to be quoted. An `unlisted` page asks not to be listed, and isn't offered: a compiler leaves it out of `llms.txt`, and out of anything else it writes that offers pages. It is still built, at its address. The content model MAY set a listing for the whole site, for the pages under a folder, or for the pages of a content type (§7.2). A page's listing is then the strictest of its own and the content model's that apply to it: a page MAY set a stricter value than they do, or the same, and a page that sets a looser one is an error (§8.2).

  ```yaml
  ---
  title: Migrating from 3.x
  listing: unlisted
  ---
  ```
- `replaces` says which page this page takes the place of, for a build versioned by a target (§9.3): a page rewritten for a newer release line, at a new path. Its value is a link destination (§5.2) naming another page, without an `#id`. The page it names MUST exist and be a page, a page MUST NOT replace itself, and no chain of `replaces` may come back to the page it starts from. In each line of a versioned build, a page and the page it replaces MUST NOT both be published, and two pages that replace the same page MUST NOT both be published (§8.2). A versioned build follows it when a link names a page a line doesn't publish (§5.2); in any other build the two are different pages.

  ```yaml
  ---
  title: Install the agent
  available: self-managed 3.5
  replaces: install-legacy.md
  ---
  ```
- `nav-label` is a page's name where it's an entry in a list: a sidebar, and the links to the previous and next pages (§7.8). Its value is a string on one line, read as the page's `title` is, code spans and phrases included where `title` takes them. A page without one is named by its title there. Elsewhere, `llms.txt` (§9.4) among them, a page keeps its title. A processor SHOULD advise where a `nav-label` is longer than the title it shortens.

  ```yaml
  ---
  title: Create and rotate API keys
  nav-label: Keys
  ---
  ```
- The source files are exactly the files under the content root (§2.2) whose names end in `.md`. A file or directory whose name begins with `.` is skipped, along with everything in it. So is a directory below the content root that holds a file named `ascribe.toml`, other than the project's own folder: it belongs to another project. The content root itself is never skipped. A symbolic link on the way to a source file is followed, and the file it leads to MUST be a source file itself (under the content root, named as above, and not in another project's folder), so a link can't put any other file on disk in a page; one that leads elsewhere is an error on that file, and it isn't a source another file can include. A source file MUST be valid UTF-8; one that can't be read, or isn't UTF-8, is an error on that file, and processors still check the rest.

### 2.2 Pages and fragments

A documentation set's source files live under a **content root**. Each file is either a page or a fragment.

- A **page** is published on its own and appears in navigation.
- A **fragment** exists only to be included by other files (§4.2). It isn't published on its own and doesn't appear in navigation.

A file is a fragment if any segment of its path, relative to the content root, begins with `_` (for example `_warning.md` or `_snippets/prerequisites.md`), or if its path matches a fragment pattern declared in the content model (§7.2). Otherwise it's a page.

Frontmatter on fragments is validated against the content model's fragment schema, not a page schema. Fragments MUST NOT use the reserved keys `type`, `available`, `variant`, `permalink`, `listing`, `replaces`, and `nav-label`; a fragment has no content type, and availability inside a fragment is written with `@available`. A fragment MAY use `intended` (§4.9).

### 2.3 Escapes

Ascribe reuses CommonMark's backslash escapes. `@`, `{`, and `.` are ASCII punctuation, so CommonMark already allows escaping them, and an escaped character renders as itself even in processors that don't understand Ascribe.

| Escape | Prevents |
|---|---|
| `\@` | A directive (§3) |
| `\{` | A phrase (§5.1) |
| `\.` | A title line (§3.7) |

Escapes are rarely needed, because these constructs are only recognized in narrow positions: a directive requires a known keyword at line start, a phrase requires a declared key, and a title line requires a directive on the next line.

---

## 3. Directives

A **directive** is a line that annotates content structure. Directives begin with `@`.

### 3.1 Syntax

A **directive line** has this shape (ABNF rule `directive-line`):

```
@<name> [{<attributes>}] [: <primary>]
@<name> [{<attributes>}]:
```

- **Name.** The directive's keyword (§3.2).
- **Attributes.** OPTIONAL. Metadata as `key=value` pairs in braces (§3.3).
- **Primary.** OPTIONAL. The directive's main value: a path, an identifier, or text (§3.4). Introduced by `:`.
- **Trailing colon.** A `:` with no primary after it, only the end of the line or whitespace, opens a container (§3.5). Every container opener ends this way, and no other directive line does. A text primary that happens to end in `:` is still a primary: `@note: Important:` is a one-line note.

Spacing between these parts doesn't matter: processors accept any spaces or tabs between the name, the attributes, the colon, and the primary, including none, and ignore whitespace at the end of the line. Canonical form (§8.3) fixes one spelling.

Attributes, when present, always come before the primary. Nothing else may appear on a directive line: text that fits none of these parts, such as `@steps foo`, `@note hello: text`, anything after `@end`, or a second word after an identifier primary (§3.4), is an error (§8.2).

```
@note {type=caution}: Back up your database first.
@include {heading=false}: guides/setup.md#install
@steps
@note {type=caution}:
```

An **end line** consists of `@end` alone. It closes a container (§3.5).

### 3.2 Recognition

A line is a directive line only if all of the following hold:

1. `@` is at line start (§1.5).
2. The name is a **known keyword**: a built-in directive (§4), `end`, or a project widget declared in the content model (§6).
3. The line is not inside a fenced code block, an indented code block, or a raw HTML block.

Recognition is block structure, which comes before inline structure. So, as with a heading, a directive line interrupts a paragraph even when a code span opened on an earlier line of that paragraph hasn't closed; the span's opening backticks are then literal text.

Otherwise the line is ordinary text. In particular, `@` followed by an unknown word is literal (`@astrojs/react`, `@timestamp`), and so is an `@` that follows a letter or digit (`support@example.com`).

A line that has the shape of a directive but an unknown name, meaning `@word` at line start followed by `{`, `:`, or the end of the line, is still ordinary text, but processors SHOULD warn about it and suggest the closest known directive (`@warning:` → "did you mean `@note {type=warning}:`?"). Writing `\@` silences the warning. Prose `@` doesn't take this shape in practice.

Built-in keywords never contain a hyphen. Project widget names always contain one (§6). The keyword set is closed: it grows only by revisions to this specification (built-ins) or by content-model declarations (project widgets).

### 3.3 Attributes

Attributes use one grammar everywhere: after a directive name, after an image (§5.3), and at the end of a table row's first cell (§4.4).

```
{key=value, key=value}
```

- **Keys** are lowercase: a letter followed by letters, digits, or hyphens (`key`). Each directive's schema declares the keys it accepts (§7.2).
- **Pairs** are separated by commas. Spaces and tabs inside the braces, around commas, around `=`, and around `|` are ignored. An empty attribute block (`{}`) is allowed and means no attributes.
- **Values** take one of three forms:

  | Form | Syntax | Example |
  |---|---|---|
  | Token | Any characters except whitespace and `,` `\|` `{` `}` `=` `"` | `type=caution`, `heading=false`, `since=3.4`, `width=600px` |
  | Quoted string | Double quotes; `\"` and `\\` escape inside | `label="Using other images"` |
  | Value set | Tokens joined by `\|`, only where the schema declares the key set-valued | `platform=cloud\|on-prem` |

- A single value MUST be quoted if it contains whitespace or any of `,` `|` `{` `}` `=` `"`. In a value set, `|` separates the members, each member is a token, and the set is never quoted. Single quotes have no special meaning.
- A value that breaks only the quoting rule, such as `{label=Using other images}` or `{lab=a=b}`, is reported as an unquoted value, because quoting it is the fix. A block whose structure can't be read as keys, `=`, and values (an unclosed quote or brace, a missing value, or text between pairs) is reported as a block that doesn't parse (§8.2).
- **Types come from the schema, never from how a value looks.** The grammar captures only the form (token, string, or set); the content model declares each key's type (string, enumeration, boolean, number) and processors validate against it.
- **Booleans are explicit:** `key=true` or `key=false`. A bare key with no value is an error.
- **An unclosed block** runs to the end of its line, so the rest of the directive line can't be read: whether it has a colon or a primary, and so its form and binding, are unknown. Processors report the unclosed block and nothing else about that line.

Attributes carry semantic metadata only. There are no class or id shorthands (`.class`, `#id`).

### 3.4 The primary

Each directive's schema declares whether it takes a primary and of which kind:

- An **identifier primary** (a path, id, or key) is a single token that ends at the first whitespace. Anything after the token on the line is an error (§3.1).
- A **text primary** (callout text) starts after the colon and continues onto the following lines exactly as a paragraph does: until a blank line, a directive line, or any other line that would interrupt a paragraph. It's parsed as CommonMark inline content, so it may contain emphasis, links, and phrases. Hard-wrapped text therefore stays together:

  ```
  @note {type=caution}: Back up your database
  before you upgrade.
  ```

  A text primary is always inline content, never a block. A line that would turn a paragraph into something else stays text: a setext underline `===` or a table delimiter row continues the primary, and a leading `[label]: /url` is text, not a link reference definition. A `---` line can't underline a primary, so it ends the primary and is a thematic break. Likewise, an ordered list that doesn't start at 1 can't interrupt a paragraph, so a line such as `2. Two.` continues a text primary; a blank line before it starts the list.

- A **line primary** (an availability spec, or a reason) is the rest of the directive line, with surrounding whitespace removed. It may contain spaces, but it isn't parsed as inline content and it never continues onto the following lines, so a paragraph directly below the directive is the block it binds. Only `@available` (§4.4), `@intended` (§4.9), and `@memo` (§4.10) take one.

### 3.5 Forms

A directive takes one of two forms.

**Line form.** The directive is a single line with no end line. What it applies to depends on its schema's binding (§3.8): its own primary, the heading above it, the block below it, or nothing (it stands alone, like `@include`).

**Container form.** The directive line opens a container, which holds the blocks that follow until an end line closes it:

```
@note {type=caution}:
First paragraph.

Second paragraph, still inside the callout.
@end
```

A single keyword can permit both forms. There's never a separate keyword for the container version of a directive. Each directive's schema declares which forms it permits (§7.2).

**Choosing a form.** A directive's form is decided by its own line, never by content further down: **a directive line whose `:` has nothing after it (an empty primary) opens a container, and any other directive line is in line form.** Trailing whitespace after the colon doesn't count, and neither does a colon at the end of a non-empty primary.

Each directive's schema declares which forms it permits (§7.2), and processors check the line against it:

- A trailing colon on a directive with no container form is an error.
- A container-only directive (such as `@variant`) without its trailing colon is an error.
- A directive that permits both forms (such as `@note`) is a container with the colon and in line form without it.

After either error, the line keeps the form its own line gives it: a trailing colon still opens a container, and a container-only directive missing its colon is treated as if the colon were there. The error is reported once, at that line, and the container's `@end` still closes it.

**Closing.** `@end` closes the innermost open container. There are no named closers. Every container MUST be closed before its enclosing block (list item, blockquote, group arm, or document) ends.

**Raw containers.** A directive's schema can make its container **raw**: its lines are text, as a fenced code block's are, from the line after the opener up to a line that's only `@end` at the opener's indent, with nothing after it but whitespace. Nothing in them is a block, a directive, an inline, or a heading, so a fence that doesn't close, a `#` line, or another opener inside doesn't change the page around it. The lines keep the prefixes of the container the opener is in (list indentation, `>` markers), and the closing line must have them too. A raw container can't hold a line that's only `@end` at its opener's indent, since that line closes it; an `@end` at another indent is text. `@memo` (§4.10) is the one built-in directive with a raw container.

Because the form is visible on the directive's own line, mistakes are reported where they happen. A container note whose trailing colon is missing binds only its first block, and its `@end` is then an end line with no open container, which is an error. A trailing colon added by mistake leaves a container unclosed, which is also an error.

### 3.6 Groups

A directive whose schema declares it **groupable** forms groups of alternatives. `@variant` (§4.3) is groupable, and project widgets MAY be (§6).

A run of openers of the same groupable directive forms one **group**. Each opener starts an **arm** and ends the previous one. A single end line closes the whole group.

```
@variant {deployment=cloud}:
Sign in to Quill Cloud and copy an API key.

@variant {deployment=self-managed}:
Point the agent at your server.
@end
```

- An arm contains every block from its opener up to the next opener in the group, or up to the group's end line.
- The group's end line belongs to the group, not to its last arm.
- A group with one arm is valid.
- An opener joins the nearest open group of the same directive within the same CommonMark container (the document, a list item, or a blockquote), even when other containers are still open inside the current arm. Those containers are reported as unclosed at the opener's line, so a missing `@end` is reported where the next arm begins rather than at the end of the document.
- As a result, a group can't be nested directly inside an arm of another group of the same directive. For `@variant`, combine the dimensions on one arm instead (§4.3).
- Diagnostics about a group as a whole (it isn't closed, it mixes kinds of arm, its arms share no dimension, or none of its arms survives a build) are reported at the group's first opener. Diagnostics about one arm are reported at that arm's opener.

### 3.7 Titles

A **title line** gives a directive a title. It's a line whose first character is `.`, followed by a character that is neither whitespace nor `.`, placed directly above a directive line whose schema accepts a title. The title is the rest of the line after the `.`, parsed as CommonMark inline content.

```
.Try it without installing
@note {type=tip}
You can run Quill in the browser with no local setup.
```

- The title line MUST be directly above the directive line, with no blank line between.
- A title line MUST begin a block: it follows a blank line, a heading, a directive line, or the start of its container. A `.` line that continues a paragraph is ordinary text.
- If the next line isn't a directive that accepts a title, the `.` line is ordinary text. Prose such as `.NET is a framework` is therefore unaffected. When it sits directly on top of a directive that doesn't accept a title, it stays ordinary text, but processors SHOULD warn that it may be a misplaced title (§8.2); `\.` silences the warning.
- A title whose text starts with a dot escapes it, as CommonMark escapes any punctuation: `.\.NET` is the title `.NET`. `..NET` isn't a title line.
- A line that starts with `.` and a space (`. Try it`) is never a title. When it sits directly above a directive that accepts a title, processors SHOULD warn that it was probably meant as one.
- Titles serve as a note's heading, a `@details` summary, and a labeled `@variant` arm's label (§4.3).

### 3.8 Binding

A directive in line form attaches to content according to its schema's **binding**:

| Binding | Applies to |
|---|---|
| Self | The directive's own primary, or nothing (it stands alone) |
| Preceding heading | The heading at the start of its section, and that section |
| Following block | The next block in the same container |

**Heading-bound directives** go at the top of their section: under the heading, before any other content. Blank lines between the heading and these directives don't matter. Several MAY stack, one per line:

```
## Streaming sync
@id: streaming-sync
@available: cloud, self-managed preview 3.4
```

A heading's **section** is the heading plus all content up to the next heading of the same or a higher level.

**Following-block directives** bind the next block (paragraph, list, code block, blockquote, table, or container) within the same container. They belong directly above that block, touching it: what a directive annotates is what it touches. A blank line between them is allowed, but processors SHOULD warn, because it hides what the directive annotates, and canonical form removes it. Binding a heading is an error, and so is a following-block directive with no block after it in its container.

Following-block directives **stack**: several in a row all bind the block the last of them touches. Any block except a heading can be bound, including a thematic break or a raw HTML block. A line-form directive whose content is its own text primary, such as a one-line `@note: …`, is a block too, so `@available` directly above it binds the note. So is `@snippet`, which becomes a code block (§4.8). A directive that renders nothing of its own, such as `@id`, `@include`, or an end line, isn't a block, so a following-block directive directly above one has nothing to bind.

**A container that binds.** A container's content is what it holds, and a container has no binding, except where a directive's schema says its container binds as its line form does; `@memo` (§4.10) is the one built-in whose container does. What such a container is about is decided from its opening line, as its line form's is: at the top of a section, the section; anywhere else, the next block after its end line in the same container. Touching is measured from the end line: the block it's about starts on the line after it, and a blank line there is the warning a following-block directive gets. With nothing after its end line in its container, outside the top of a section, it's the error a following-block directive with no block gets, reported at the opening line. It stacks as following-block directives do, in either order: a following-block directive directly above it binds what it binds.

**A summary.** `@summary` (§4.12) binds by whether it has a primary, as `@note` does, but its text is its section's, not a block of the page: with a primary, it binds the section, and so goes at the top of it, as a heading-bound directive does; without one, it binds the following block. It renders nothing of its own, so with a primary it isn't a block that a following-block directive can bind.

Sections are found within one container: a heading-bound directive binds a heading in its own document, list item, blockquote, directive container, or arm, never one outside it. A container's first blocks have no heading above them, as at the start of a document.

The two rules together: **at the top of a section, a directive describes the section; anywhere else, it describes the block it touches.**

Before a document's first heading there is no section. A directive that can only bind a heading, such as `@id`, is an error there, and one that can bind either way, such as `@available`, binds the block it touches. Availability for the whole page belongs in frontmatter (§4.4).

### 3.9 Directives inside lists and blockquotes

Directives follow CommonMark's container rules, just as headings and code fences do.

1. **Indentation decides ownership.** A directive line indented to a list item's content column belongs to that list item.
2. **Binding stays inside the item.** A following-block directive in a list item binds the next block within that item.
3. **Containers don't straddle items.** A container or group opens and closes within one list item, or it contains whole lists. An end line in a different list item or blockquote from its opener doesn't close that opener: the end line is an error, and the opener, still open, is also reported unclosed when its own container ends. Within one container, up to three extra spaces before the end line (§1.5) don't matter.
4. **Directive lines interrupt paragraphs and never continue them.** Like a heading, a directive line starts a new block. An unindented directive line directly after a list item therefore ends the list.
5. **Over-indentation makes code.** A directive line indented four or more spaces beyond its container's content column is part of an indented code block, and so it's literal text. One to three extra spaces are allowed, as before a heading; a line indented less than a list item's content column is outside that item.
6. **Blockquotes work the same way**, with `>` markers in place of indentation.

The three list warnings (§8.2) apply narrowly. A directive line ends a list when it directly follows the list, with no blank line, in the same container; end lines and blockquotes don't count. Every directive line inside an indented code block is reported, at its `@name`. A `@steps` list's numbering is continued when the next ordered list starts at the number after the `@steps` list's last item and only line-form directive lines stand between the two.

### 3.10 Nesting

Containers MAY nest, and `@end` always closes the innermost one. Processors SHOULD warn when containers nest more than two levels deep (§8). A group counts as one level; its arms don't add another. Depth counts every open container around a directive, through list items and blockquotes, which aren't levels themselves.

---

## 4. Built-in directives

| Directive | Forms | Primary | Binding | Purpose |
|---|---|---|---|---|
| `@id` | line | identifier | top of section | Give a heading a stable id |
| `@include` | line | identifier (path) | self | Transclude a file or a region |
| `@variant` | container, groupable | none (label via title) | — | Mark alternative content |
| `@available` | line | availability spec or feature key | top of section, else following block | Declare where content applies |
| `@note` | line, container | text, optional | self (with primary), else following block; container with a trailing `:` | Callout |
| `@steps` | line | none | following block | Mark an ordered list as a procedure |
| `@details` | line, container | none | following block; container with a trailing `:` | Collapsible content |
| `@snippet` | line | identifier (address) | self | Take a code example from a file |
| `@intended` | line | line (reason) | top of section, else following block | Acknowledge a check's problem as intended |
| `@memo` | line, container (raw) | line (text), in line form | top of section, else following block, in either form | Keep a note for the people who maintain the page, never published |
| `@term` | line | none | top of section | Make a glossary page's heading a term |
| `@summary` | line | text, optional | top of section (with primary), else following block | Give a section a summary |
| `@operation` | line | identifier (link to an operation) | self | Show an operation of a declared API, linked to its reference |

### 4.1 `@id`

Gives a heading an explicit, stable id.

```
## Configuration
@id: config-setup
```

- **Form:** line. **Binding:** preceding heading (at the top of the section, §3.8). **Primary:** REQUIRED identifier (letters, digits, hyphens, underscores, and periods). Underscores and periods are allowed because slugs can contain underscores (`snake_case-names`) and existing sites' anchors use both (`ece_setup`, `v1.2`), so an `@id` can keep a published URL fragment.
- The id replaces the heading's slug, as both its source id and its page id (§5.5). An id that isn't valid still replaces the slug, so a link that uses it works once the id is fixed, and a heading with several `@id` lines takes the first; each mistake is reported once.
- The id MUST be unique within its page.
- The id names the heading's section, which links (§5.2) and `@include` (§4.2) can both target.
- **Attribute:** `permalink`, a boolean, `false` when absent. `{permalink=true}` makes the id a **permalink** too: a name for the section that something outside the documentation can link to, which a build publishes at an address of its own that leads to the heading wherever its page is (§9.5).

  ```
  ## `ascribe check`
  @id {permalink=true}: cli-check
  ```

  A build MUST publish a permalink in one place only, and a page's own permalink (§2.1) shares one set of names with the headings'. Names that differ only in case are one name, since a site may serve every spelling of an address at one. A heading in a fragment is published on every page that includes it, so its permalink is published on each. Two pages that no build publishes together may share a permalink.

### 4.2 `@include`

Transcludes a file or a heading's section into the current document.

```
@include: _snippets/prerequisites.md
@include {heading=false}: guides/setup.md#install
```

- **Form:** line. **Binding:** self. **Primary:** REQUIRED path, optionally followed by `#` and an id.
- **Paths** are relative to the including file, or, when they begin with `/`, relative to the content root. Processors MUST NOT resolve a bare filename by searching other directories. A path is percent-decoded as a link destination is (§5.2), so `my%20snippet.md` names `my snippet.md`, and an empty `#` (`file.md#`) includes the whole file.
- Only a source file (§2.1) can be included: a file outside the content root, or that isn't Markdown, is reported as a target that doesn't exist. So is a file whose symbolic links lead to a file that isn't a source file (§2.1).
- With `#id`, only the section of the heading with that source id (§5.5) is included.
- **Attributes:**

  | Key | Type | Default | Meaning |
  |---|---|---|---|
  | `heading` | boolean | `true` | When `false`, the included section's own heading is omitted. It applies only with `#id`; on an include of a whole file it has no effect, and processors SHOULD warn |

- The target file and id MUST exist. Include cycles are an error: an include is a cycle when expanding it would, directly or through other includes, include the same section of the same file again, so its expansion would never end.
- Includes without a cycle can still make a page very large: a few files that each include the next twice double it at every step. A processor MAY limit how many blocks a page's includes expand it to, set far above what a real page reaches. The include in the page's own file that takes the page past the limit is an error (page level), and it and every include after it on the page are left unexpanded.
- Included content becomes part of the including page. Phrase substitution and build modes (§9) apply to it as they do to the rest of the page.
- Included headings keep the levels they're written with. Include a section where its levels fit the page.
- Bindings are decided in the file where a directive is written. With `{heading=false}`, a heading-bound directive under the omitted heading, such as `@available`, still describes the rest of the included section, as it does in the fragment.
- **Relative paths resolve from the file they're written in.** Every piece of content keeps its source file. Link destinations, image sources, and nested include paths inside a fragment resolve against the fragment's location, not the including page's. A fragment that links to `keys.md` means the `keys.md` next to the fragment, wherever it's included.
- **Ids are checked on the expanded page.** Headings and `@id`s in included content become ids of the including page, and every id on a page MUST be unique after expansion. Including the same fragment twice on one page, or including a fragment whose ids collide with the page's own, is an error reported at the include site (§8.1).
- **Links target pages, not fragments.** A fragment isn't published on its own, so a link to a fragment file is an error. Link to the page that includes it instead: a page's linkable ids are its own source ids (§5.5) and those of the headings its includes bring in, so a link to `setup.md#prerequisites` works when `prerequisites` is a heading in a fragment `setup.md` includes. Like any heading, one from a fragment can be removed by a build, and a link to it is then an error in that build (§9.3).

### 4.3 `@variant`

Marks alternatives: content that differs by a declared dimension, or labeled one-off alternatives.

**Dimensional arms** carry dimension values as attributes:

````
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

**Labeled arms** carry a label as a title line (§3.7), for alternatives that don't correspond to a declared dimension:

```
.Using Docker Hardened Images
@variant:
…

.Using other images
@variant:
…
@end
```

- **Form:** container only, groupable (§3.6). Every arm opener ends in `:`. **Primary:** none.
- Each arm has either attributes (a dimensional arm) or a title (a labeled arm), never both and never neither.
- **Dimensional arms:**
  - Each attribute key MUST be a declared dimension, and each value a declared value of it (§7.2).
  - A value set (`platform=cloud|on-prem`) means the arm applies to any of those values.
  - Several keys on one arm mean the arm applies only when all of them match.
  - Negation and any other operators are not part of the language.
  - All arms in a group MUST share at least one dimension key.
  - To vary by two dimensions at once, put both on one arm (`{deployment=cloud, pm=npm}`); groups don't nest directly (§3.6).
- **Labeled arms:**
  - A labeled group is local to its page. Its labels aren't validated against the content model and don't synchronize across pages.
  - A group's arms MUST be either all labeled or all dimensional.
- **Granularity.** `@variant` varies whole blocks. For a single differing term, use a phrase (§5.1). For larger differences inside a paragraph, split the paragraph into blocks.
- **Semantics.** `@variant` only marks content. The build decides whether to keep every arm for readers to switch between, or only the arms matching the build (§9.3).
- **Whole-page variance.** When an entire page differs by dimension, write separate pages rather than wrapping a page in `@variant`, and declare each page's dimension values in the reserved frontmatter key `variant`. Its value is a mapping from dimension names to a value or a list of values:

  ```yaml
  ---
  title: Connect to Quill Cloud
  variant:
    deployment: cloud
  ---
  ```

  The meaning matches a dimensional arm: a list is a value set, and several dimensions must all match. Names and values are validated like `@variant` attributes. A page without `variant` applies to every dimension value.

### 4.4 `@available`

Declares where content applies and its lifecycle state. Unlike `@variant`, it doesn't select content: every reader sees the content, annotated with its availability.

```
@available: cloud, self-managed preview 3.4
```

- **Form:** line. **Primary:** REQUIRED line primary (§3.4): an availability spec or feature key.
- **Binding:**
  - At the top of a section, before any other content, it applies to the whole section, whether or not a blank line separates it from the heading.
  - Anywhere else, it binds the following block it touches (§3.8).
- **Page level:** the frontmatter key `available` holds the same spec and applies to the whole page:

  ```yaml
  ---
  title: Install the Quill agent
  available: cloud, self-managed preview 3.3
  ---
  ```

- **Table row:** a row of a table (where the processor supports tables, §1.4) takes a spec from the `available` attribute of an attribute block (§3.3) that ends its first cell. A spec with spaces or commas is quoted, as any such value is:

  ```
  | Option | Meaning |
  |---|---|
  | `anchors` {available="self-managed preview 3.4"} | Marks each block with its source. |
  ```

  - The block is the last thing in the first cell of a body row, after a space or alone in the cell, and closes at the end of the cell. It isn't part of the cell's content. A `{` that touches the text before it, a block followed by more text, a block in code, in another cell, or in the header row, and braces with no `=` in them (a phrase candidate, §5.1) stay text.
  - `available` is the only attribute a row takes. Its value is a spec or a feature key, checked as an `@available` primary is.
  - The row's spec is a scope inside the table's, so it MUST NOT exceed the availability the table has (§4.4 Scope). A filter build removes a row that isn't available (§9.3); the header row has no spec of its own, so a table always keeps it.
  - There is no inline form. Removing part of a sentence breaks the sentence, and a badge in the middle of one is noise: a value that differs by release is written as prose ("glibc 2.28 or later (2.39 for 0.1.1 and earlier)").

#### Availability specs

A spec is a comma-separated list of **targets**, each optionally followed by its lifecycle (ABNF rule `availability`).

| Spec | Meaning |
|---|---|
| `cloud` | Generally available on `cloud` |
| `self-managed 3.3` | Generally available on `self-managed` since version 3.3 |
| `self-managed preview 3.4` | In preview on `self-managed` since version 3.4 |
| `self-managed (preview 3.3, ga 3.5, deprecated 4.0)` | A history: each state starts at its version and lasts until the next state begins |
| `cloud, self-managed preview 3.3` | Two targets |
| `self-managed preview planned` | In preview on `self-managed` from its planned line, as `self-managed preview 3.6` is when 3.6 is that line |

- A **target** is a value of a declared dimension, or the name of a dimension, which stands for all of its values. A dimension name takes a state but never a version, since its values don't share one version line: write `deployment beta`, or name the value, as in `self-managed beta 3.4`.
- A **state** is a declared lifecycle state (§7.2). A target with no state is generally available (`ga`). A bare version means generally available since that version. A state with no version on a versioned target (`self-managed beta`) is in effect at every version, as a bare target is.
- Each lifecycle state declares whether content in that state **counts as available**. By default every state counts as available except `removed`.
- States in a history MUST be in chronological order, compared using the content model's version scheme. A target that the content model declares as versionless takes a single state and no versions.
- The language has no version ranges, alternatives, or negation. Each state names only the version where it begins.
- **`planned`** is a version word: written where a version goes, it means the target's planned line (§7.2), and the spec reads as it would with that line's version in its place, wherever the spec is checked, filtered, compared, or shown. So content can be marked for a release whose work isn't done without writing the number twice: `self-managed planned`, `cloud, self-managed preview planned`, `self-managed (preview 3.5, ga planned)`. A target with no planned line, or more than one, is an error at the word, since `planned` names no single version there; a versionless target or a dimension name takes no version at all. `planned` is never a lifecycle state, dimension name, dimension value, or feature key (§7.2), and a feature's spec doesn't use it: a feature names its versions. When the line is released, each `planned` for its target is rewritten to the line's version.
- **Commas here aren't `|`.** In `@variant`, `|` means "any of these values", a single membership test (`platform=cloud|on-prem`). In an availability spec, `,` separates targets, each with its own lifecycle (`cloud, self-managed preview 3.3`).
- Targets, dimension names, and states are names (ABNF rule `name-word`): a letter followed by letters, digits, `_`, or `-`. They're spelled exactly as the content model declares them.

#### Scope

- When a scope has a spec, only the targets it lists apply there.
- A scope with no spec inherits its enclosing scope's availability. A page with no spec applies everywhere.
- A section or block spec MUST NOT exceed its enclosing scope: it can't list a target the enclosing scope doesn't, or name a version earlier than the enclosing scope does.
- When a spec lists a target both directly and through its dimension name (`deployment, cloud removed`), the direct entry decides that target's state. If it lists the same target twice, the first entry decides.
- Several `@available` lines on one heading or block all apply: the content is available only where every one of them allows it, and each is checked against its enclosing scope.

#### Feature keys

A primary (or frontmatter value) consisting of a single token that matches a key in the content model's features registry is replaced by that feature's declared spec, and wherever the directive's annotation is kept (§9.3, §9.4), it shows that spec, not the key. A feature going generally available then takes one edit. A bare word in a spec is therefore a dimension value, a dimension name, or a feature key, and the content model guarantees it can only be one of them (§7.2).

### 4.5 `@note`

A callout.

```
@note {type=caution}: Back up your database first.
```

```
.Try it without installing
@note {type=tip}
You can run Quill in the browser with no local setup.
```

- **Forms:**
  1. **With a primary** (`@note {type=tip}: text`), the primary is the note's content. It may continue onto the following lines, like a paragraph (§3.4).
  2. **With no colon** (`@note {type=tip}`), the note binds the following block.
  3. **With a trailing colon** (`@note {type=tip}:`), the note is a container that holds blocks until `@end`.
- **Title:** accepted (§3.7).
- **Attributes:**

  | Key | Type | Default | Meaning |
  |---|---|---|---|
  | `type` | note type | `note` | The kind of callout |

- **Note types** are an enumeration declared in the content model. The default types are `note`, `tip`, `important`, `warning`, and `caution`. Projects MAY add types. A type is data, not a new keyword.

### 4.6 `@steps`

Marks an ordered list as a procedure.

```
@steps
1. Install the agent package.
2. Verify the install.
3. Create `quill.yaml`.
```

- **Form:** line. **Binding:** following block. **Primary:** none.
- The bound block MUST be an ordered list.
- The list remains an ordinary CommonMark list. Directives inside its items follow §3.9.

### 4.7 `@details`

Content that readers can expand or collapse.

```
.Show the full configuration reference
@details:
…
@end
```

- **Forms:** with no colon, binds the following block; with a trailing colon, a container. **Primary:** none.
- **Title:** REQUIRED (§3.7). It's the text a reader sees while the content is collapsed.

### 4.8 `@snippet`

Takes a code example from a file outside the content, by name, and puts it in the page as a fenced code block. There's no copy to fall behind: the file is read whenever the page is checked or built, so changing the file changes every page that uses it.

```
@snippet: code:examples/quill/ascribe.toml#dimensions
@snippet {lang=shell, title="Install"}: code:scripts/install.sh
```

- **Form:** line. **Binding:** self. **Primary:** REQUIRED address (an identifier primary).
- **The address** is `<source>:<path>`, optionally followed by `#` and a region (ABNF rule `snippet-address`):
  - `<source>` names a source declared in the content model (§7.3). There's no other way to name a file outside the content root, and no relative form: an address doesn't depend on where the page is, or on where the code is checked out.
  - `<path>` is the file's path relative to the source's folder, with `/` between segments on every platform. It has no empty, `.`, or `..` segments, and doesn't start with `/`. It's percent-decoded as a link destination is (§5.2). The file MUST exist, with exactly this name (as for assets, §9.4), and its source's file patterns MUST include it (§7.3). A symbolic link on the way to the file is followed, and the file it leads to MUST be in the source's folder (not above it), at a path the source's patterns include, so a link can't lead a snippet out of its source.
  - `#<region>` names a region of the file (below). Without one, the snippet is the whole file, with every tag line and removed line left out.
- **The file** MUST be text: valid UTF-8, with no NUL character. Its line endings don't matter; the snippet's lines end in a line feed.
- **What it becomes.** A `@snippet` is a block, allowed wherever a fenced code block is (in a list item, a container, or an arm), and a following-block directive directly above it binds it (§3.8). It becomes a fenced code block whose info string is the language, then `title="…"` when the directive has a title attribute, then `tested="…"` when its file is a tested example with a current observation (§7.3), then `phrases=true` when it opts in to phrases. With no language, the info string is empty, or starts with `text` when a title, a tested line, or `phrases=true` follows, so its first word is always a language. With `phrases=true`, declared phrases in the snippet are substituted as they are in a fence that opts in (§5.1).
- **Attributes:**

  | Key | Type | Default | Meaning |
  |---|---|---|---|
  | `lang` | string | the file's extension | The code block's language |
  | `title` | string | none | A title for the code block, written into its info string as `title="…"` |
  | `phrases` | boolean | `false` | Whether phrases in the code are substituted (§5.1) |

  The language defaults to the file's extension, lowercased, without the dot (`toml` for `ascribe.toml`). A file with no extension has no language unless `lang` gives one.

#### Regions

A region is marked in the file with tags in comments. The tags are Bluehawk's, so files already tagged for it can be used as they are.

```toml
# :snippet-start: dimensions
[dimensions.platform]
values = ["linux", "macos", "windows"]
labels = { macos = "macOS" }  # :remove:
# :snippet-end:
```

- **Tag lines.** A tag counts only in a line comment, in the comment syntax of the file's extension (the table below). A component file, with a script and markup, takes either marker on any line. A **tag line** is a line holding only optional whitespace, the comment marker, optional whitespace, the tag, and optional whitespace (then the closing `-->` for `<!--`).
- **`:snippet-start: <name>`** opens a region and **`:snippet-end:`** closes the innermost open one; `:snippet-end: <name>` closes the open region with that name, so regions may overlap as well as nest. A region holds the lines between its start and end tag lines. A region name is letters, digits, `-`, `_`, and `.` (ABNF rule `region-name`), and MUST be unique in its file.
- **`:remove-start:`** and **`:remove-end:`** leave out the lines between them, and a line that ends with the comment marker, optional whitespace, and **`:remove:`** (then `-->` for `<!--`) is left out itself. Removal blocks may nest, and apply to every snippet of the file.
- **Tag lines never appear in a snippet**, nor do removed lines. Of the lines that remain, those holding only whitespace at the start and the end are dropped, so where a formatter puts a tag line doesn't change the snippet. The rest are dedented by their common leading whitespace; lines holding only whitespace are emptied and don't count toward it.
- Every start tag MUST have an end tag, and every end tag a start. An unclosed tag or an end with nothing to close makes every snippet of the file an error.
- **Reserved tags.** Bluehawk's other tags are reserved for later revisions: `state`, `state-remove`, `state-uncomment`, `replace`, `uncomment`, and `emphasize`, each as `-start` and `-end` tag lines, and `:emphasize:` and `:uncomment:` at the end of a line, as `:remove:` is written. A file that uses one can't be used by a snippet, so its code is never shown with a tag left in.

The comment syntax of each extension:

| Marker | Extensions |
|---|---|
| `//` | `c`, `cc`, `cjs`, `cpp`, `cs`, `cts`, `cxx`, `dart`, `go`, `gradle`, `groovy`, `h`, `hpp`, `java`, `js`, `jsonc`, `jsx`, `kt`, `kts`, `mjs`, `mts`, `php`, `proto`, `rs`, `scala`, `swift`, `ts`, `tsx`, `zig` |
| `#` | `bash`, `cfg`, `conf`, `ex`, `exs`, `fish`, `hcl`, `nix`, `pl`, `pm`, `ps1`, `py`, `r`, `rb`, `sh`, `tf`, `toml`, `yaml`, `yml`, `zsh` |
| `--` | `elm`, `hs`, `lua`, `sql` |
| `;` | `asm`, `clj`, `cljs`, `el`, `ini`, `lisp`, `scm` |
| `<!--` … `-->` | `htm`, `html`, `md`, `mdx`, `svg`, `xml` |
| `//` or `<!--` … `-->` | `astro`, `svelte`, `vue` |

Extensions are compared without regard to case. A file whose extension isn't in the table has no tags: it can be used whole, but not by region. The table grows with revisions of this specification.

**No history, no network.** A snippet reads the file as it is in the working tree, through its source, so checking and building need neither version-control history nor a network. A source in another repository is read through the copies in the project (§7.4).

### 4.9 `@intended`

Says that a problem a processor reports is intended: the author has looked at it and keeps it. An acknowledgement names one check and gives a reason, and it applies to one page, one block, one heading, or one content model entry.

```
@intended {check=link-external-broken}: help.example.com refuses link checkers; checked by hand.
Read the [vendor's deployment notes](https://help.example.com/deploy) first.
```

- **Form:** line. **Binding:** the heading, at the top of a section (§3.8); otherwise the following block. **Primary:** REQUIRED reason (a line primary, §3.4): plain text, the rest of the line.
- **Attributes:**

  | Key | Type | Default | Meaning |
  |---|---|---|---|
  | `check` | string | none, REQUIRED | The check whose problem is intended, by its name |

- **What can be acknowledged.** A processor's checks are its own, beyond the diagnostics of §8.2. A check that reports something that may be fine as it is (a page nothing links to, a page longer than an agent reads) is a **review check**, and only a review check can be acknowledged; a problem that makes a document non-conforming never can. Each review check reports its problems at one kind of place: a page or fragment as a whole, one block, or one entry of the content model or one image, which have no page. Its problems are acknowledged at that place, and nowhere else:
  - **A block:** `@intended` directly above it. The acknowledgement covers that check's problems located anywhere in the block, including in a container's content. In a fragment, it covers the block in every page that includes it.
  - **A heading:** `@intended` at the top of its section, among the directives under the heading. The acknowledgement covers that check's problems located at the heading, and, when no blank line comes between, those in the block directly under the `@intended` line, past the directives stacked with it, so a section's first block is acknowledged as any other block is. It covers nothing else in the section:

    ```
    ## Workspace
    @term
    @intended {check=glossary-term-unused}: Linked from the product's own UI.
    ```

  - **A page or a fragment:** the reserved frontmatter key `intended`, a list of mappings with a `check` and a `reason`:

    ```yaml
    intended:
      - check: page-orphan
        reason: Linked from the site's sidebar, which the content model can't see.
    ```

  - **A content model entry or an image:** `[[intended]]` in the content model (§7.2), with a `check`, a `reason`, and exactly one of `phrase`, `feature`, or `image` (a path under the content root):

    ```toml
    [[intended]]
    check = "phrase-unused"
    phrase = "old-product-name"
    reason = "Kept for the 2.x pages restored in the next release."
    ```

- **The reason** is REQUIRED and not empty. A processor that writes an acknowledgement for the author with a placeholder reason MUST report one that still has it as an error, so that writing an acknowledgement never silences a problem without a reason.
- **What it does.** A problem an acknowledgement covers isn't reported. Processors SHOULD count acknowledged problems, and list them with their reasons where they list problems.
- **When nothing needs it.** An acknowledgement that covers no problem in any build is itself reported as advice (§8.2), so acknowledgements don't outlive what they excused. Processors report it only when they've checked every build.
- **Never in an output.** An acknowledgement changes no output: the site, plain, and JSON outputs (§9.4) are what they'd be without it. `@intended` renders nothing, and the `intended` key isn't in a page's frontmatter in any output.

### 4.10 `@memo`

Keeps a note for the people and agents who maintain a page: why it's written the way it is, beside what it's about. A memo is never published, and nothing reads what it says, apart from the two words that say a file is generated.

```
## Rotate a key
@memo:
Rotation comes before revocation: support found that people who revoke
first lock out their own agents. See [the incident](notes/2026-08.md).
@end

Generate a new key, then restart the agent.

@memo: Kept to one sentence; the support macro quotes it.
Revoke the old key once every agent uses the new one.
```

- **Forms:** line and container. **Binding:** top of section, else following block, in either form (§3.8). **Primary:** in line form, REQUIRED text (a line primary, §3.4): plain text, the rest of the line. **Attributes:** none.
- **The container is raw** (§3.5): its lines are the memo's text, as written. It's about what its opening line says it's about, and the block it's about starts directly after its `@end`.
- **Nothing reads a memo.** Its text isn't inline content: a `{key}` in it isn't a phrase, a link isn't checked and isn't a use of its target, an `@include` isn't followed, a `#` line isn't a heading and gets no id, and a processor that checks prose doesn't check it.
- **Never in an output.** A memo changes no output: the site, plain, and JSON outputs (§9.4) are what they'd be without it. A memo in a fragment is part of the fragment, and no page that includes the fragment publishes it.
- **Never a change.** Comparing two versions of a page compares what they publish, so a page whose only change is to a memo hasn't changed.
- **A generated file.** A memo among a file's top-level blocks, not inside a container, a list, or a blockquote, whose text begins `Generated from` or `Generated by` says another program writes the file: an edit to it is lost the next time that program runs, and the memo's text says what to change instead and how to run the program again. Processors SHOULD report the first such memo as related information of each diagnostic located in the file, with a note that gives the memo's text, and MUST NOT offer a fix that edits the file. The rest of the memo's text is read by nobody.

```
@memo: Generated from options.toml by scripts/options.ts; edit the options, then run `node scripts/options.ts` to rewrite it.
| Option | What it does |
|---|---|
```

### 4.11 `@term`

Makes a heading of a glossary page a glossary term (§5.4): the heading's text is the term, and the paragraph under it is its definition.

```
## Rollout
@id: rollout
@term {aliases=rollouts}

A plan for turning a flag on for a growing share of users.
```

- **Form:** line. **Binding:** preceding heading (at the top of the section, §3.8). **Primary:** none.
- **Attributes:**

  | Key | Type | Default | Meaning |
  |---|---|---|---|
  | `aliases` | string | none | Other forms of the term that are linked as it is, separated by `\|`, such as `"API keys\|keys"` |
  | `match` | `first`, `every`, or `marked` | the glossary's | Which occurrences are linked: the first on each page, every one, or none, for a word that also has an ordinary sense, whose author links the occurrences that mean the term |
  | `case-sensitive` | boolean | the glossary's | Whether an occurrence must match the term's case |

- `@term` makes a term only on a page the glossary lists (§7.2), or in a fragment such a page includes. Elsewhere, it isn't a term, and it's an error (§8.2).
- **Never in an output.** `@term` renders nothing: the heading and its section are published as they'd be without it.

### 4.12 `@summary`

Gives a section a **summary**: a sentence or two that says what the section is about, for a reader shown it apart from the page, such as a product's embedded help.

```
## ASC149 `page-size`
@id {permalink=true}: ASC149
@summary: A page's Markdown is as long as the limit or longer.
```

```
## `ascribe versions release`
@id {permalink=true}: cli-versions-release

@summary
Releases a line.

It rewrites each mark's `planned` for its target to the line's version.
```

- **Forms:** line only.
  1. **With a primary** (`@summary: text`), the primary is the summary, and the page's body doesn't show it, as a page's `description` isn't in its body. It binds its section (§3.8), so it goes at the top of the section, after the heading and its `@id`; anywhere else it's an error, as a heading-bound directive is (§8.2). For when nothing on the page reads well alone, as for a lead-in to a list.
  2. **With no colon** (`@summary`), the summary is the following block, which renders where it is: a sentence the page already shows, so nothing is written twice. It goes anywhere in its section, directly above the block. The block MUST be a paragraph; to pick a first sentence, make it a paragraph of its own.
- **Primary:** OPTIONAL text (§3.4): inline content, as a note's, with code spans, links, emphasis, and phrases. **Attributes:** none. **Title:** none.
- **Its section** is the section of the nearest heading before it in the page after includes are expanded (§9.2), so a summary in a fragment is the summary of the section it lands in, on each page that includes it. A following-form summary before a page's first heading is the summary of the page's opening.
- **One per section.** A second summary in a section is an error, at the second (page level, §8.2). A subsection's summary is its own.
- **Not in a `@variant` arm.** A summary inside an arm, at any depth, is an error, since a summary can't differ by arm, and so is one an include brings into an arm. Either is reported on the page (§8.2), and is no section's summary in any build, including one that picks that arm.
- **Optional.** Nothing requires a summary, and nothing guesses one from a section's prose.
- **In the outputs.** The site and plain outputs write nothing for the primary form, and the following form's block as they would without the directive. The JSON output gives a heading its section's summary (§9.4).

### 4.13 `@operation`

Puts an operation of a declared API (§7.11) in the page: its method, its path, its summary, and whether it's deprecated, linked to the operation's address in the published reference.

```
@operation: ../api/openapi.yaml#getUser
```

- **Form:** line. **Binding:** self. **Primary:** REQUIRED identifier: a link to an operation, written as a link's destination names one (§5.2): the spec of an API the content model declares, relative to the file it's written in, or to the content root when it begins with `/`, then `#` and the operation's `operationId`, or its method in lowercase and its path. **Attributes:** none. **Title:** none.
- **It's checked as a link to the operation is** (§5.2): the spec MUST have the operation; one named by its method and path when it has an id is advice to name it by the id; and one with no id needs a reference that has an address for it. A primary that isn't a link to an operation of a declared API, such as a page, another file, the spec with no `#`, or a URL, is an error (§8.2).
- **Only what names the operation.** It shows the operation's method, path, summary, and deprecation, never its parameters, request, or responses: a page that wants them links to the reference.
- **It's a block,** so a following-block directive directly above it, such as `@available`, binds it (§3.8).
- **In the outputs** (§9.4): the site output writes an `<ascribe-operation>` element (§9.7), the plain output a line of text linked to the operation's address, `GET /users/{id}: Get a user`, with `(deprecated)` after it when the spec marks the operation deprecated, and the JSON output a block of type `operation`. For an `@operation` that names no operation the spec has, the site and plain outputs write nothing, and the JSON output keeps the directive.

---

## 5. Inline constructs

These constructs appear within text. None of them is a directive.

### 5.1 Phrases

A **phrase** inserts a reusable string, such as a product name, version, or URL base, from the content model's phrases registry (§7.2).

```
Sign in to {cloud} and copy an API key.
{cloud}'s streaming sync is enabled for new {cloud}-hosted workspaces.
See the [streaming API reference]({api}streaming).
```

- **Syntax:** a key in braces: `{key}` (ABNF rule `phrase`).
- **Recognition:** `{key}` is a phrase only if the key is declared in the phrases registry. Otherwise it's literal text.
- **Boundaries:** braces delimit the phrase, so it can sit directly against punctuation, letters, or hyphens.
- **Where phrases apply:**
  - Prose, headings, link text, and link destinations, including the destinations of link reference definitions (`[ref]: {api}streaming`) and autolinks (`<https://{host}/status>`).
  - Fenced code blocks only when the info string contains the word `phrases=true` (for example ` ```yaml phrases=true `).
  - Frontmatter fields, as the content model declares.
- **Where phrases never apply:** code spans, indented code blocks, and raw HTML.
- **Escapes in code.** A backslash doesn't escape in code, so in a fenced block with `phrases=true`, `\{key}` is a backslash followed by a phrase, and a fence's content never differs from its source except where phrases are substituted. For a literal `{key}` in code, leave `phrases=true` off that fence.
- **Distinction from attributes:** `{…}` directly after a directive name, or directly after an image and closed on the same line, is an attribute block (§3.3), not a phrase, whatever it contains. So is the block that ends a table row's first cell (§4.4). So `![Logo](logo.png){cloud}` is an attribute block with a bare key, which is an error; to put a phrase right after an image, separate it with a space or escape it (`\{cloud}`).
- **Values** are inserted as literal text. They aren't scanned for phrases or markup, and they don't vary by case, number, or argument.
- **Destinations are checked after substitution.** A destination's phrases are substituted before it's resolved and checked, as in the build, so `[reference]({api}streaming)` names an external URL, not a missing file.
- **Keeping source and output in agreement.** Whether `{key}` is a phrase depends on a registry the reader can't see, so processors SHOULD report two cases:
  - `{key}` text in prose whose key isn't declared, which is literal today and would silently become a phrase if the key were declared later (`\{` silences it). Prose here is every inline position: paragraphs, headings, link text, alt text, table cells, titles, and text primaries, but not destinations, fences with `phrases=true`, or frontmatter;
  - when a key is added to the registry, the pages whose existing literal `{key}` text would change.

  Processors SHOULD also warn about a declared phrase written directly between two more braces, `{{key}}`: it's `{`, the phrase, and `}`, so the value appears between literal braces. That spelling is almost always a substitution from another tool (Hugo, Jinja, Elastic's docs) that wasn't converted; `\{` silences the warning for a brace that's meant.

### 5.2 Links

Links are CommonMark links. Their destinations are **file paths**.

```
[Rotate API keys](keys.md#rotate-keys)
See [](keys.md#rotate-keys).
```

- **Paths** are relative to the linking file, or relative to the content root when they begin with `/`. An optional `#id` names a heading of the target page by its source id: one of the target file's own headings, or one that a fragment it includes brings in (§4.2). The compiled link points at that heading's page id (§5.5).
- The target file, and the id if present, MUST exist. A destination that is only `#id` names a heading in the file it's written in; in a fragment, that's a heading of the fragment itself, and the compiled link points at that heading's page id on each page that includes the fragment. In each build, the target page MUST also be published: a link to a page the build drops (§9.3) is an error in that build. To link to such a page from shared content, put the link in a `@variant` arm that the same build removes.
- **Links between lines.** In a line of a versioned build (§9.3), a link to a page the line doesn't publish resolves, in order, to: the page that replaces it, or that it replaces, in this line, following a chain of `replaces` (§2.1) to the page this line publishes; then the page in the nearest line that publishes it, the closest newer line first, then the closest older, as a **link out** of the line. It's an error in that line only when no live line publishes the page. A link's `#id` is kept when the page it lands on has a heading with that id there, and is otherwise dropped, with advice. A link out keeps its text; empty text is the title the page has in the line it leads to. Outputs say which line a link out leads to (§9.4).
- **Empty link text** is replaced by the target's title: the heading text when an id is given, and the page title otherwise. A page's title is its frontmatter `title`, which every content type requires (§7.2). When the content model reads code spans in `title`, the link text keeps them.
- **External URLs** (with a scheme such as `https:`) are passed through unchanged.
- **Operations.** A link to the spec of an API the content model declares (§7.11), with a fragment, is a link to one of its operations: the fragment, percent-decoded, is the operation's `operationId`, or its method and path, which the spec MUST have. `[Get a user](../api/openapi.yaml#getUser)` names `getUser`. A fragment that is a method in lowercase, then the path as the spec writes it, names the operation with that method and path, so an operation with no id can be linked: `[Delete a user](../api/openapi.yaml#delete/users/{id})`. A fragment that is both an operation's id and another's method and path names the one with the id. The compiled link is the operation's address in the API's published reference; an operation with no id has one only in Redoc's (§7.11), so with a template, a link to one is an error, and a link by method and path to an operation that has an id is advice to link to the id. A link to an operation the spec marks `deprecated` is advice. A link to the spec with no fragment is a link to the file, an asset (§9.4), when the spec is inside the project's folder; when it's outside, no output has a place for the file, and the link is an error. A link to any other file of the kind, such as a YAML file no API declares, is an asset. A JSON code block's `request` or `response` marker starts with a link to an operation, resolved and checked as one (§7.11), and compiles to nothing.
- **Routes.** At compile time, paths are rewritten into the consumer's URLs using the consumer profile (§9.5). Source files never contain routes. A destination that looks like a published route rather than a file path produces a warning offering conversion. A local destination looks like a route when it names no existing file and its last segment has no file extension or it ends in `/` (`/guides/install/`, `../guides/install`); it then gets that warning instead of a missing-file error.

### 5.3 Images

Images are CommonMark images. Alt text and titles use CommonMark's own syntax. Other attributes go in an attribute block placed directly after the image, with no space between. This applies to every form of CommonMark image: inline (`![alt](src)`) and reference (`![alt][ref]`, `![alt][]`, `![alt]`):

```
![The Quill settings page](settings.png){width=600}
```

- The attribute grammar is §3.3's. The content model declares which image attributes are accepted (§7.2).
- The block MUST close on the same line as the image. Any `{…}` directly after an image and closed on its line is the image's attribute block, even with no `=` in it (§5.1). A `{` directly after an image with no `}` on its line stays text, and processors report it as an attribute block that doesn't parse.
- A local image source MUST exist, and an image MUST have one: an image with an empty source is an error. A reference image's source is its link reference definition's destination.
- Processors SHOULD warn when an image has no alt text.
- Presentation choices such as borders or shadows are not image attributes; they belong to the consumer's styling.

### 5.4 Glossary terms

A glossary is one or more pages, which the content model lists (§7.2). Each heading on a glossary page marked `@term` (§4.11), written on the page or in a fragment it includes, is a **glossary term**:

- **Its text** is the heading's text (§5.5), and its aliases are its `@term`'s. **Its id** is the heading's source id.
- **Its definition** is the first block of its section's own content, when that's a paragraph, as plain text: phrases substituted for the build, emphasis and code reduced to their text, a link to its text, an image to its alt text, inline HTML dropped, and whitespace runs one space. Directives that bind the heading, and `@available`, `@intended`, and `@memo` above the paragraph, come before the section's own content. A paragraph a fragment brings in counts. A section that opens with groups (§3.6), such as `@variant`, has the paragraph each arm opens with, in the builds that keep that arm. A section that opens with anything else has no definition, which is an error (§8.2). The definition is resolved for each build: a build that removes the paragraph leaves the term without a definition in that build.
- **It's linked** to its heading on the first of its glossary pages the build publishes. A heading that two glossary pages show, through a fragment, is one term.
- Two terms, or a term and an alias, with the same text, compared ignoring case unless both terms are case-sensitive, are an error at the second, which isn't a term (§8.2).
- Two terms with the same id, which their headings' texts can give them on two pages, are an error at the second, which isn't a term (§8.2).

Processors link occurrences of glossary terms to their definitions according to each term's settings, else the glossary's. Authors don't mark glossary terms where they're used, except for a term whose settings turn automatic linking off (`match = "marked"`): processors don't link that term, and authors link the occurrences that mean it to its heading with an ordinary link (§5.2), for words that also have ordinary senses.

**A link an author writes to a term's heading is a use of the term,** as a linked occurrence is, and outputs show it as one. Exactly a link whose destination is the term's heading by its id is; a link to the glossary page, or to another heading in the term's section, is an ordinary link. Where a term is linked only at its first occurrence, such a link counts as that occurrence, so a processor doesn't also link a later one on the page.

Terms are matched in each resolved page's text, after phrases are substituted and a build's modes are applied, so "the first occurrence" is the first a reader sees. Emphasized text counts as prose. A term isn't linked on a glossary page that shows it, and a term whose heading a build doesn't publish, or that has no definition in the build, isn't linked in that build.

### 5.5 Heading ids

Every heading has two ids: one for referring to it in source, and one for its anchor on a published page.

- **Source id.** The id that includes (§4.2) and links (§5.2) use to name a heading in a given file. It's the heading's `@id` if it has one. Otherwise it's the heading's **slug**, computed from the heading's text (with phrases substituted) by the slugging algorithm the consumer profile names (§9.5), with duplicates within the file numbered the way that algorithm numbers them. Source ids depend only on the file, never on a build.
- **Page id.** The heading's anchor on the page it's published in, for a given build. It's the `@id` if there is one; otherwise it's computed like the source id, but on the expanded page after includes and build modes (§9.2), so duplicates are numbered across the whole page. A compiled link points at its target heading's page id.

For a heading with `@id`, both ids are the `@id`, which is what makes it stable.

- An `@id` directive under the heading replaces the slug (§4.1).
- **A heading's text**, for its slug, is the text content of the rendered heading: its text, code spans, link text, and emphasized text, with phrases substituted. Images and raw HTML contribute nothing.
- A heading without `@id` whose slug is empty, because its text is only punctuation or emoji, can't be linked to reliably, and processors SHOULD warn.
- Explicit ids don't take part in slug numbering: a slug is numbered only against earlier slugs. A heading whose slug equals another heading's `@id` on the same page therefore duplicates that id, which is an error (§4.1).
- Processors SHOULD warn when a heading without `@id` contains a phrase, or when its slug is the same as another heading's on the same page (`Set up` and `Set-up` both slug to `set-up`), so that its id is numbered. In either case its id can change without the heading itself being edited. Headings with an `@id` don't count, since explicit ids aren't numbered.

---

## 6. Project widgets

A **project widget** is a directive defined by a documentation set rather than by this specification. Project widgets are the extension mechanism for needs the built-in vocabulary doesn't cover.

```
@quill-labspace {lab=first-sync}
```

- **Names** MUST contain at least one hyphen (ABNF rule `widget-name`). Built-in names never do, so a reader can tell the two apart at a glance.
- Every project widget MUST be declared in the content model with its permitted forms, primary kind, attribute schema, binding, title acceptance, and whether it's groupable (§7.2).
- A declared widget is recognized, parsed, and validated exactly like a built-in directive. An undeclared hyphenated name isn't a directive (§3.2).
- A widget MAY declare a plain-text fallback, used by the plain-markdown output (§9.4).
- Project widgets follow all of §3, including end lines, groups, titles, binding, and container rules.

---

## 7. Content model

### 7.1 Role

The content model is a documentation set's schema. It is the single contract shared by the authoring environment, the validator, and the compiler: all three read the same declarations, so they can't disagree about what's valid.

The content model is a TOML file named `ascribe.toml` at the project root. Processors read it directly. Consumers' own schemas are generated from it rather than maintained separately; for Astro, that's the content collection's Zod schema (§9.6).

A content model with errors is reported, and nothing else is checked, since every other check depends on it. The warnings of a content model that loads are reported with the rest of the diagnostics.

TOML keeps the content model's values unambiguous: strings are always quoted, so a value such as `no` or `3.10` can't change type the way it can in YAML.

### 7.2 Declarations

A content model declares the following.

| Declaration | Contents | Used by |
|---|---|---|
| Content types | A frontmatter schema per page type, each with a required string `title`, fields every page type shares, and a fragment schema. Fields are typed as string, number, boolean, date, enumeration, list, or object, and may be optional or have a default. A page type's string field may opt in to code spans, so a title can show code, and a page type's fields may have roles. A page names its type in its frontmatter, with `type` | §2, §5.2 |
| Fragment patterns | Additional globs that mark files as fragments | §2.2 |
| Directive schemas | For each project widget: forms, primary kind, attributes, binding, title, groupable, plain fallback | §3, §6 |
| Dimensions | Each dimension's name, values, and display labels, and which values are versionless | §4.3, §4.4 |
| Versions | How versions are compared | §4.4 |
| Releases | For a versioned target, its release lines: those that are live, those planned, each newer than every live line, and those archived, each older; when a live line's support ends; and the label of a planned version. A line is a version as written, and holds every version that starts with its parts: `3.4` holds `3.4`, `3.4.2`, and `3.4.2.1`. Every line of one target has the same number of parts, and the newest live line is the target's current line. A version in a planned line is planned | §4.4, §9.3, §9.4 |
| Lifecycle states | Default `preview`, `beta`, `ga`, `deprecated`, `removed`, each declaring whether it counts as available (by default, all but `removed` do); extensible | §4.4 |
| Features registry | Feature keys, each with a name and an availability spec | §4.4 |
| Note types | Default `note`, `tip`, `important`, `warning`, `caution`; extensible | §4.5 |
| Phrases registry | Phrase keys and values. A value is text, or a versioned target whose release line it is: the line a line's output is for, and the current line elsewhere | §5.1 |
| Glossary | The glossary's pages, and its matching settings | §5.4 |
| Image attributes | Accepted image attribute keys and types | §5.3 |
| Consumer profile | Routing, slugging, heading ids, HTML passthrough, images | §9.5 |
| Builds | Named builds and their modes | §9.3 |
| Sources | Named sets of files outside the content that pages may take code from: each one's folder or repository, and which of its files are readable | §4.8, §7.3, §7.4 |
| APIs | Named OpenAPI specs that pages link into: each one's file, and where its reference is published | §5.2, §7.11 |
| Acknowledgements | Problems of review checks about a phrase, feature, glossary term, or image that are intended, each with a reason | §4.9 |
| Crawlers | What each kind of crawler may do, by purpose (gathering training data, building an AI search index, fetching at a person's request), and the project's own `robots.txt` lines, with paths from the project's base path. A crawler is named in one group; processors reject a content model that names one twice when loading it | §9.4 |
| Listing | How pages are listed (§2.1): the site's default, and a value for the pages under a folder, by its path from the content root, or of a content type. A folder's value MUST NOT be looser than that of a folder it's under, and neither a folder's nor a type's looser than the default; processors reject a content model that does when loading it | §2.1 |

Built-in directive schemas are defined by this specification, not by the content model. A content model MAY extend the enumerations they use (note types, lifecycle states).

Dimension names, dimension values, lifecycle states, and feature keys are names (ABNF rule `name-word`). A name MUST NOT be used in more than one of these roles, and a dimension value MUST NOT belong to more than one dimension; processors reject a content model that does either when loading it. `planned` is the version word (§4.4) and takes none of these roles. This keeps every bare word in an availability spec unambiguous (§4.4). Dimension names are also attribute keys (§4.3), so they follow the attribute key rule (ABNF rule `key`, §3.3).

The site output writes image attributes onto `<img>` elements and project widgets' attributes onto custom elements (§9.4), so a content model MUST NOT declare an attribute key that HTML already gives a meaning there: `src`, `alt`, or `title` for images; `heading` or `primary` for widgets, which the site output uses for a widget's title and identifier primary; and, for both, HTML's global attributes (such as `id`, `class`, `style`, `title`, `hidden`, and `slot`), ARIA attributes (`aria-…`), and event-handler attributes such as `onclick`. Keys that merely begin with `on`, such as `online`, are allowed. Processors reject a content model that does when loading it.

A string field is plain text unless the content model opts it in to code spans. Such a field's value is read as CommonMark inline content that recognizes only code spans and backslash escapes; everything else in it is literal text. The outputs carry both its plain text, the value without markup, and its formatted form, which keeps the code spans: the site output writes the plain text as the field's value and the formatted form beside it, and a page title's formatted form is used for its level-1 heading in plain markdown and for empty link text (§5.2). Phrases in such a field are substituted outside its code spans only (§5.1).

A page type's top-level field MAY have a role, which says what the field is for, whatever its name, so a tool can find it: `description`, on a string field, is the page's description; `review-date`, on a date field, is the date its content is due for review; and `owner`, on a string or enum field, names who looks after the page. A content type has at most one field with each role. A field named `description`, `review-date`, or `owner` has that role, when its type is one the role takes, unless the content model gives the role to another of the type's fields. Roles change no output.

Every page type has a required string field `title`, the page's title, whether the content model declares it or not; a declaration of it changes only how it's read, such as with code spans, and processors reject a content model that declares it optional, with a default, or of another type when loading it. A content model MAY declare fields every page type shares. A page type has each of them unless it declares a field of the same name, which replaces the shared one whole for that type, and it MAY declare no fields of its own.

A page's content type is the one its frontmatter's `type` names (§2.1), wherever the page is: a type applies to pages by what they say, not by where their files are, so a folder can hold pages of every type. A content model that declares no types has one, the implicit `page`, whose fields are `title` and the shared ones; its pages name no type. A content model that declares types has no implicit one: each of its pages names one of the declared types.

### 7.3 Sources

A **source** is a named set of files outside a project's content that its pages may refer to: the code a snippet comes from (§4.8). Pages name a file through a source, by address (`<source>:<path>`), never by a path that leaves the project's folder. A project with no source can read nothing outside its own folder and its content root (§9.4).

```toml
[sources.code]
path = ".."                               # relative to the project's folder
include = ["crates/**", "examples/**"]    # what's readable; everything else isn't
ignore = ["**/target/**"]

[sources.api]
git = "https://github.com/acme/api.git"   # another repository (§7.4)
branch = "main"
include = ["src/**", "examples/**"]
```

- A project MAY declare several sources. A source's name follows the attribute key rule (ABNF rule `key`).
- A source is a folder in the project's repository, given by `path`, or another repository, given by `git` (§7.4). It has exactly one of the two. Pages address both kinds the same way, so moving code from one kind of source to the other changes the content model and no page.
- `path` is the source's folder, relative to the project's folder (the directory containing `ascribe.toml`). It MUST exist and be a directory. When the project is in a version-control repository, it MUST be inside that same repository.
- `include` lists glob patterns (the content model's pattern syntax, as for `files`) matched against a file's path relative to the source's folder. A file is readable when a pattern matches it and no pattern in `ignore` does. Without `include`, every file in the folder matches. Whatever `ignore` lists, a file in a folder named `node_modules` or `.git`, at any depth, is never readable: a processor leaves both out before the source's own patterns.
- Which files a source makes readable decides nothing else: a source's files aren't source files (§2.1), assets (§9.4), or pages.
- **Tested examples.** A source MAY have, at its folder's root, an `ascribe.examples.toml` that says which of its files are examples, and beside it, in `examples.observations/`, the observations `ascribe observe` writes: what a test run passed, and where. A file is an example when the config's `include` matches it and its `ignore` doesn't. A snippet of an example that the observation lists with the hash its file has now carries what it was tested with in its code block's info string (§4.8). The config and the observations are read whatever the source's `include` says, and no snippet can address one: the source doesn't make `ascribe.examples.toml` or a file under `examples.observations/` readable. A processor reads both files and MUST NOT run any command in them. Their formats are in the tested examples contract (`docs/content/contracts/tested-examples.md`). For a source in another repository, they're copies (§7.4).

### 7.4 Sources in another repository

A source with `git` is a repository other than the project's. Checking and building still read only the project's own files: the files its snippets use are **copied** into the project, and `ascribe.lock` records the commit they were copied from.

- `git` is the repository's URL, as `git` accepts it: `https://`, `http://`, `ssh://`, `git://`, or `file://`, or the `user@host:path` form. `branch` is the branch an update follows; without it, the repository's default branch. `branch` without `git` is an error, as is a source with both `git` and `path`, or with neither.
- **The copies** of a source named `<name>` are in the folder `sources/<name>/`, in the project's folder, each at the path it has in its repository. That folder is the source's folder: an address's path is relative to it, and `include` and `ignore` apply as for any source. It MUST NOT be inside the content root, where a copied `.md` file would be a page. Beside the files snippets use, the copies are the repository's `ascribe.examples.toml` and every file in its `examples.observations/` at the commit, when it has them, whatever `include` says (§7.3).
- **The lock**, `ascribe.lock`, beside `ascribe.toml`, is a TOML file:

  ```toml
  version = 1

  [[source]]
  name = "api"
  git = "https://github.com/acme/api.git"
  commit = "9f2c41d0e0c4a1b2c3d4e5f60718293a4b5c6d7e"
  missing = ["examples/quickstart.ts"]

  [source.files]
  "src/auth.rs" = "sha256:5d41402abc4b2a76b9719d911017c592ae2fd2b1f6b3b0f0d5ce64b4c7d0e1a2"
  ```

  `version` is `1`. Each `[[source]]` pins one source: its name, its `git` as the content model gave it when it was pinned, its commit as 40 or 64 hexadecimal digits, and each copy's path with the SHA-256 of its bytes, as `sha256:` and 64 lowercase hexadecimal digits. `missing`, which may be left out, lists the paths snippets named that weren't in the repository at that commit, so weren't copied; no path is both copied and missing. No two entries name the same source.
- Processors check the lock and the copies whenever they check the project, from the files alone: each entry names a source the content model declares with `git`, and the same `git`; each file the lock lists is in the source's folder with that hash; each file in the source's folder is listed; and each copy, other than the tested examples' config and observations, is used by a snippet. A snippet whose file has no copy is an error that says how to copy it, or, when the lock lists the file as `missing`, that it isn't in the repository at the pinned commit.
- **No history, no network.** Copying, and moving a pin to another commit, are a processor's commands of their own, and the only operations that reach another repository. Checking and building read the copies, so the same commit of the project checks and builds the same way on any machine.

### 7.5 The permalinks list

A project MAY commit a list of its permalinks (§2.1, §4.1), `ascribe.permalinks`, beside `ascribe.toml`, so that a permalink that goes away is noticed, and so that something outside the documentation can check the names it links to against the list alone. Without the file, nothing below applies.

```toml
version = 1

[permalinks]
ASC036 = "_generated/diagnostics-source-files.md#ASC036"
cli-check = "reference/cli.md#cli-check"
cli-reference = "reference/cli.md"
```

- `version` is `1`. `[permalinks]` has one entry for each name, keyed by the name, whose value is where the name is **declared**: the content path of the file it's written in, and, for a heading, `#` and its id. A heading in a fragment is declared in the fragment. A place is where the name is written in the source, not a published address, which differs by build.
- The names `ascribe.addresses` records (§7.6) have an entry each: `[aliases]` one for each name moved to another permalink, whose value is that permalink's name; `[moved]` one for each name moved to a page, whose value is where, as the file writes it; and `[retired]` one for each retired permalink, whose value is its reason. Each table is written only when it has an entry, and a name is in one table. A name is a permalink of the documentation when `[permalinks]`, `[aliases]`, or `[moved]` has it; one in `[retired]` was, and has been retired on purpose.
- The list is the project's, not one build's: a permalink is declared by a page's `permalink` frontmatter, or by an `@id {permalink=true}`, in any source file, whether or not a build publishes it. A name declared in several places (on pages no build publishes together) has one entry, at any of them.
- Processors check the list whenever they check the project, from the files alone: each declared name has an entry, at one of the places it's declared, each name `ascribe.addresses` records an entry as the file has it, and each entry is still one of the project's names. A name without its entry, or with an entry at a place that no longer declares it, means the list is **behind**; moving a permalink to another page or heading makes the list behind, and doesn't lose it. An entry the project has no name for any more is a **lost** permalink: something outside the documentation may still link to it, which no check can see. Retiring it, or moving it (§7.6), is how its removal is recorded.
- A processor writes the list on request, adding each declared name, moving each entry to where its name is declared now, and recording each name `ascribe.addresses` records. It MUST NOT drop an entry the project no longer has, so writing the list never hides a removal.

### 7.6 Retired and moved permalinks

What happened to a permalink that no heading or page declares any more is recorded in `ascribe.addresses` (§7.10), with the rest of where the documentation's addresses went, keyed by `go:` and the name:

```toml
[moved]
"go:agent-settings" = "go:settings"
"go:rotate-keys" = "guides/security.md#rotate-keys"

[retired]
"go:legacy-sync" = "The sync agent was removed in 4.0"
```

- A name **moved** to another permalink, `go:` and its name, is an **alias**: a second name for that permalink, leading where it does, through other moved names. It MUST end at a permalink a heading or page declares, or at a retired one, and is retired with it then; a chain that comes back to its start leads nowhere. Only a `go:` key's value can be a `go:` name.
- A name moved to a place is written as any `[moved]` value: a page's content path, and optionally `#` and a heading's id. It MUST lead to a page, and to a heading on it.
- A **retired** permalink is one ended on purpose. Its reason is the author's text, shown to a reader whose link leads there.
- A name the content declares is the content's: an entry for it does nothing.
- Each build's list of permalinks (§9.5) has a moved name where it has what the name ends at. A consumer publishes its address as it does a permalink's. A retired permalink, or a name moved to one, has an address that says what was there was removed, and why.
- `ascribe.toml` has no `[permalinks]` table. A processor reports one, and MAY move its entries on request: each of `[permalinks.retired]`, as retired, or as moved to its `to` when it has one, and each of `[permalinks.aliases]` as moved to the name it leads to. A base (§7.10) whose `ascribe.toml` has the table is read as if they had been moved.

Bringing a retired or moved permalink back is deleting its entry and declaring the name again.

### 7.7 Memo files

A project MAY keep memos longer than a line or two in files of their own: a **memo file** says why a page, a section, a block, or a content model entry is the way it is, for the people and agents who maintain it. It's never published, and nothing in it is resolved. The team's memo files are the `.md` files in the folder `memos/`, beside `ascribe.toml`, and in its subfolders; a file or folder whose name begins with `.` is skipped. A file's name and its subfolder mean nothing.

```markdown
---
about: install-agent.md#install-agent
quote: Install the agent package
---

The agent goes first because its installer writes the config file that
`quill init` reads.
```

- The frontmatter has two keys. `about`, required, names what the memo is about, as a target of `ascribe refs` is written: a page or a fragment by its content path, a heading as the content path, `#`, and its id, a glossary term as `term:<id>`, or a content model entry as `phrase:<key>`, `feature:<key>`, `dimension:<name>`, `note:<type>`, or `widget:<name>`. A heading in a fragment is named in the fragment. A glossary term is its heading (§5.4): a memo about `term:<id>` is about that heading, where it's written. `quote`, optional, narrows the memo to the block whose source holds it; a content model entry has no text to quote. A memo is about one thing.
- The body is CommonMark, shown as written. Nothing in it is a phrase, a link a processor checks, an include, or a heading with an id.
- A memo's place is worked out each time it's read, and nothing is written back to the file. With no `quote`, it's **placed** on what `about` names. With one, the blocks of what `about` names (a page's or fragment's, or a heading's section) are searched in source order, each block's source as written, whitespace runs read as one space and case kept; the first that holds the quote is the memo's block, and the memo is placed there. A quote across two blocks isn't found. A quote that no block holds makes the memo **outdated**: it stays on the page or section. An `about` that names nothing in the project makes the memo **detached**.
- `memos/` MUST NOT be inside the content root, where its files would be pages; it is when the content root is the project's folder.
- Memo files are no part of any output, and a change to one is no change to a page.

### 7.8 Navigation

A project MAY say what order its sections and pages come in, and their labels, in `ascribe.nav`, beside `ascribe.toml`. Every sidebar a processor or a consumer draws is the project's **navigation**: the file's order over the default, or the default without the file.

```toml
version = 1

[[sections]]
folder = ""

[[sections]]
folder = "reference/"
label = "Reference"

[[sections]]
folder = "guides/"
pages = ["install.md", "configure.md"]
```

- A **section** is the content root, or a folder under it with a page at any depth. Fragments (§2.2) are no section's pages, so a folder whose name begins with `_` isn't a section. A page is in its folder's section, and in no other.
- A section's **own page** is its `index.md`, or else the page beside the folder with the folder's name: `checks/diagnostics.md` for `checks/diagnostics/`. A page beside its folder is also one of the parent section's pages. The content root's own page is its `index.md`, the home page. A folder MUST NOT have both an `index.md` and a page beside it with its name; until it doesn't, its `index.md` is its own page.
- `version` is `1`. Each `[[sections]]` table lists one section: `folder`, its path from the content root ending in `/`, or `""` for the content root; optionally `pages`, the file names of pages in that folder, in order, which may name the page beside a nested folder but not the section's own page; and optionally `label`, one line. Nothing else may be written. The tables' order is the sections' order.
- **The default order:** sections in path order, the content root first. In each, its own page first, then its pages by their content type, in the order the content model declares types (§7.2), then by title as a reader sees it: as plain text where the title is read with inline markup, with the phrases it takes as the content model declares them, and then as written. A nested section below the content root whose own page is beside its folder sits right after that page among its parent's pages; any other comes after the parent's pages, in path order. A top section is never placed by its own page: the top sections all come after the content root's pages, in path order, whether or not a top section's own page is beside its folder.
- **The file's order over it:** the sections the file lists come first, in its order, among their siblings, and the rest after, in path order. In a section that lists `pages`, those pages come first, in its order, and the rest after, by title; below the content root, a nested section whose own page is named moves with it, and at the content root, naming a top section's own page moves no section. A section with no `pages` has the default order.
- A section's **label** is its `label`, else its own page's title, else its folder's name with `-` and `_` as spaces and its first letter capitalized. The content root's is "Start" unless `label` says otherwise. A page's label is its `nav-label` (§2.1), else its title.
- **Reading order** is a section's own page, then its pages in order, a nested section whose own page is beside its folder right after that page, with its pages and nested sections, and the other nested sections after them, each from its own page; from the content root, depth first, except that the content root's own page beside a top folder is read with that folder's section: the root's own page and its pages that aren't beside a folder come first, then each top section from its own page, as a sidebar groups them. A page's previous and next are the pages before and after it in reading order, and the home page has no previous.
- **One navigation for a project.** A build's navigation is its share: the pages it publishes and offers (§2.1), so a page that's `unlisted` isn't in it, and the sections left with one.
- A file that can't be read as above is an error, and the project has the default order, as if it had no file.
- A file that reads is an error where it names what the project doesn't have, and what it names wrongly is passed over: a `folder` that isn't a section; a `pages` entry that isn't a page in its folder, or that names the section's `index.md`; and a folder listed a second time, or a page named a second time in one section's `pages`, the second counting for nothing.
- A processor SHOULD advise where the file leaves out what it could order: a page of a section that lists `pages`, and a section whose siblings the file lists, unless it's below the content root and its own page beside its folder places it among its parent's pages, where a table wouldn't move it. Each comes after what's listed, as above.

### 7.9 Taking from another project

A project MAY take tables and keys from another project's content model, so that two projects that share a fact, such as a site's address or a product's releases, write it once. It names the file, and what it takes:

```toml
[from."../../docs/ascribe.toml"]
take = ["releases", "dimensions.pm", "consumer.site"]
```

- The key after `from` is the other file's path, relative to the project's folder. It MAY lead outside the folder, and when the project is in a version-control repository, it MUST be inside that same repository, as a source's `path` is (§7.3). The file MUST exist and be a content model a processor can read as TOML; it's read as a file outside the content root, never as one of the project's sources.
- `take` lists what's taken, each a table or a key, as a dotted path of bare keys: `releases` takes the whole table, `consumer.site` one key of `[consumer]`. What's named MUST be in the other file.
- What's taken is read as if written in this file, and every rule of the content model holds it there, with one exception: a source's `path` in a taken table is relative to the other file's folder, where it was written, and a processor reads it as the same folder from this project's.
- A key taken and also written in this file is an error: a reader of either file can tell where a value comes from. So is a name taken twice, from one file or two, or a table taken with a key in it.
- `spec`, `[project]`, and `[from]` can't be taken: they say what this project is. The other file's own `[from]` isn't followed, so what's taken is always written in the file named.
- A processor that edits a content model in place edits a key where it's written: an edit of a taken key, or of a key inside a taken table, is made in the file it's taken from, at the same path, and a key `take` names that's renamed is renamed in `take` too. A rename of a name the other file declares, such as a phrase or a dimension's value, is also made where that file's project uses it, in its pages, so that project isn't left using the old name. An edit that changes the other file MUST say which file it changed. An edit it can't make in the other file, such as one of a table written partly in each, or one that would write a path the other file would read from its own folder, it MUST refuse, naming the file.
- An authoring environment SHOULD watch the other file as it watches `ascribe.toml`, and SHOULD say, at a name in `take`, where the taken key is written.

### 7.10 The addresses file

A project MAY commit a record of where the addresses its documentation published went, `ascribe.addresses`, beside `ascribe.toml`, so that no address stops working without someone deciding it should. Without the file, nothing below applies.

```toml
# Where addresses the docs published went. Written by `ascribe rename`, the
# editor, `ascribe addresses`, and `ascribe check --fix`; edit only
# [retired]'s reasons.
version = 1

[moved]
"/docs/intro.html" = "getting-started.md"
"guides/install.md#install-with-npm" = "guides/install.md#install-npm"
"guides/keys.md" = "security/keys.md"

[former]
base-path = ["/docs/"]

[retired]
"reference/legacy-sync.md" = "The sync agent was removed in 4.0"
```

- `version` is `1`; a version a processor doesn't know is an error naming a newer one. The tables are `[moved]`, `[former]`, and `[retired]`, each optional; a table or key the file doesn't define is an error.
- A **key** of `[moved]` or `[retired]` is a source: a page's content path, standing for every address the page was published at; a file's content path, `#`, and a heading's id, with the file the `@id` is written in, so a fragment's id is one key however many pages include it; `go:` and a permalink's name (§7.6); or an address from the host's root, starting with `/`, published before the project used a processor. A `[moved]` value is a page's content path, and optionally `#` and a heading's id, or, for a `go:` key, `go:` and a permalink's name. A `[retired]` value is the reason, the author's text, which MUST NOT be empty or the placeholder a processor writes.
- `[former]` lists the routing settings the documentation was published under before, each oldest first: `base-path`, `permalink-path`, and `agents`, each a list of values, and `builds`, a table from a build's old name to its new one. `trailing-slash` isn't one: a host matches an address in either form.
- An **address** a build publishes is a page's route, its Markdown version's when the build publishes one, and each heading's explicit id as the page's route, `#`, and the id; each name the build answers to under the permalinks' path (§7.5); and each `llms.txt` file it publishes. A generated id isn't one: it changes with the heading's text.
- Processors compare the addresses a **base** published, the documentation as a revision of its repository had it, with the addresses the project publishes now, build by build, matching builds by name. A build the base had for a line that the project's releases (§7.2) archive now left on purpose: its addresses are accounted for, and it isn't compared. An address the base published and no build of that name publishes now is **lost** unless an entry accounts for it: `[moved]` has its source, or the address itself, and the value is a page a build publishes now; `[retired]` has its source, or the address; or an availability mark takes its page out of that build while another build publishes it. An id is compared only while its page is at the same address: what happened to the page decides it. Each source lost is reported once. A permalink's name is the permalinks list's to keep (§7.5), and an `llms.txt` file has no source, so either is lost only by a setting.
- A **settings change** is a routing setting the base published under that the project doesn't now: `base-path`, the permalinks' path, `agents` turned off, or a build renamed, which a processor MAY take to be a build the base had and the project doesn't, when a build only the project has publishes more than half of its pages. The addresses it moved, the same sources' under the new value, or a Markdown version or `llms.txt` file no longer published, are reported once for the change, in place of once for each source, naming it; the change is the decision, so a processor SHOULD offer to record each old value in `[former]` and MAY do so without asking. A change `[former]` records accounts for every address it moved, and a build `builds` renames is compared with the build of its new name. A permalinks' path that's its default in both moved with the base path, and `base-path` records it.
- A processor that moves a source on request records the move in `[moved]`, when the file exists, in the same change as the links it updates; moving a source back to an entry's key removes the entry. It writes the file whole, each table sorted, one entry per line, keeping the header comment.
- A processor that retires a release line records, when the file exists, each page address the line's builds published that goes to a page in the current line, the same page or the one that replaces it, as a `[moved]` key from the host's root leading to that page, in the same change as the releases it edits. An address that goes to the line's archive isn't recorded: a `[moved]` value is a page of the project.
- A processor that moves a page records, with it, each entry whose value is on that page, as leading to the same place on the new one; one that retires a page retires each entry leading to it, with the same reason. A heading id's rename on a page is recorded as a move of that id.
- A `[moved]` value that is another entry's key is a **chain**: the entry leads where the chain ends, and a chain that comes back to its start leads nowhere. A processor checks each entry against the base and the project now: a value that isn't a page a build publishes, a key whose source is back, an address the key had that another source publishes, an entry the base's file doesn't have whose key the base never published, a key twice in a table or in both, a chain, an id the key had that is live on the page it leads to, many entries leading to one index page, and the file deleted while the base has it.
- A build's redirects come from the file alone, never the base: for each `[moved]` entry, each address its key had, as the build's routing makes it now, leads to where its value, or its chain, ends, permanently, unless the build publishes that address or it's the same address once case and a final `/` are ignored. Each `[retired]` key's addresses are gone, with the reason; a processor that builds a site serves a page at each that says why, with a 404.
- For each `[former]` value, every address each source would have had under it leads to the same source's address now, permanently, with the same exceptions: under a former `base-path`, each page's, its Markdown version's, each `llms.txt` file's, each `[moved]` and `[retired]` key's, and each permalink's while `permalink-path` is its default; under a former `permalink-path`, each permalink's. With `agents = [true]` and `agents` off now, each Markdown version leads to its page, and each `llms.txt` file to the page at the base path. Each leads to the address now, never to another former one, so a second change composes with the first. A pair an entry makes is written before one a former value makes from the same address.

### 7.11 APIs

A project MAY declare the OpenAPI specs its pages link into (§5.2), each by name, with its file and where its reference is published:

```toml
[apis.public]
spec = "api/openapi.yaml"
reference = { renderer = "redoc", at = "/api/" }
```

- An API's name follows the attribute key rule (ABNF rule `key`).
- `spec` is one file, YAML or JSON (JSON when its name ends in `.json`), relative to the project's folder. It MAY be outside the folder, and when the project is in a version-control repository, it MUST be inside that same repository, as a source's `path` is (§7.3). It MUST exist and be readable as YAML or JSON. It's read as a file outside the content root, never as a source.
- A processor reads, of the spec, each operation in `paths`: its method, its path, and its `operationId`, `summary`, first tag, and `deprecated`, the same way in Swagger 2.0 and in OpenAPI 3.0 to 3.2. It doesn't validate the spec, apply overlays, or follow a `$ref` into another file: a spec split across files is declared by its bundled file, the one its renderer is given.
- `reference` says where an operation's address is. `{ renderer = "redoc", at = "<address>" }` names Redoc's page: an operation's address is `<address>#operation/<operationId>`, and one with no id's is `<address>#paths/<pointer>/<method>`, its path as a JSON pointer token (`~0` for `~`, `~1` for `/`) and its method in lowercase, each percent-encoded, an address of Redoc 3's. Any other value is a template of the address, whose placeholders are `{operationId}`, which it MUST have, and `{tag}`, the operation's first tag, each as the spec writes it and percent-encoded. An address that isn't absolute (with a scheme, or starting with `//`) is relative to the consumer profile's base path (§9.5).

A fenced code block whose language is `json` is an **example** of an operation's request body when its info string has `request="<link>"`, and of one of its responses when it has `response="<link> <status>"`, beside `title` and `tested` (§4.8):

````markdown
```json request="../api/openapi.yaml#createUser"
{ "name": "Ada" }
```

```json response="../api/openapi.yaml#createUser 201"
{ "id": "u_1", "name": "Ada" }
```
````

- The value starts with a link to an operation (§5.2), resolved and checked as a link is. A response's status follows after a space, as the spec's key is written (`201`, `2XX`, `default`). A media type MAY follow after another space, and MUST when the body has more than one JSON media type (`application/json`, or any `…+json`); otherwise the body's one JSON media type is meant. A marked block that isn't a `json` block, a value whose link isn't to an operation, and an operation with no body the marker names are errors.
- A processor reads, of the operation, its request body and its responses, by media type: in Swagger 2.0, the schema of its `in: body` parameter in each of its `consumes`, and each response's schema in each of its `produces`, `application/json` when it lists none; in OpenAPI 3, the `content` of its `requestBody` and of each response. A `$ref` to a request body, a response, or a parameter in the same file is followed.
- The block's code MUST be JSON that agrees with the body's schema. The schema's `$ref`s into the same file are followed, each fragment percent-encoded first. OpenAPI 3.1's and 3.2's schemas are JSON Schema 2020-12. OpenAPI 3.0's and Swagger 2.0's are read as JSON Schema draft 4, after a translation: `nullable: true` and `x-nullable: true` add `null` to the type and to any `enum`, or, in a schema with no type, make `{ "type": "null" }` an alternative; `type: file` is dropped; and a `$ref`'s siblings are ignored. In a request, a `readOnly` property isn't required, and in a response, a `writeOnly` one isn't. JSON Schema's formats are asserted, and OpenAPI's own (`int32`, `int64`, `float`, `double`, `byte`, `binary`, `password`) aren't. A pattern the processor can't read is left out, with advice, and the rest of the schema is checked. A schema with a `$ref` into another file isn't checked, with advice.
- Each disagreement is an error at its place in the block: the value, a property the schema doesn't allow at its key, and a missing required property at its object's opening brace. Code that isn't JSON is an error where reading it stopped.

---

## 8. Validation

### 8.1 Validation levels

Validation happens at two levels.

- **File level.** Each source file on its own: syntax, attributes, directive schemas, frontmatter, and whether referenced files exist.
- **Page level.** Each page after includes are expanded, availability is resolved, and a build's modes are applied (§9.2), once per build. This covers checks that depend on the assembled page: id uniqueness, link targets that are ids, and anything a build removes. A build reports page-level problems only in content it publishes: content a build removes isn't checked for that build, which is what lets a link to a page the build drops sit in an arm the build removes. Content that no build publishes is still checked, as if one build kept everything, and its problems are reported as belonging to no build. A problem that appears in several builds is reported once, naming them. A fragment that no page includes has no page-level problems; its file-level ones are still reported.

A page-level diagnostic is reported at the source location that causes it. A diagnostic about what a link or image names is reported at its destination as written, for an inline link or image, and at the link or image itself for a reference form. When the cause is inside a fragment, it's reported at the include site, and processors SHOULD also report it in the fragment, as related information rather than as a second diagnostic. A problem with a snippet (§4.8) is a file-level problem of the file the `@snippet` is written in, reported at the directive; when its cause is in the code file, such as a tag, processors SHOULD also report that place, as related information.

When several places together cause a diagnostic, it's reported once, at the later one: the second of two duplicate ids or headings (at the `@id` line for an explicit id, at the heading for a slug), the second include of a fragment included twice, the include that closes a cycle, and the later of two declarations in the content model. A frontmatter problem with no line of its own, such as a missing field, is reported at the file's first line.

### 8.2 Diagnostics

Conforming processors MUST report every error below, and SHOULD report the warnings. An error means the document isn't conforming; a build MUST fail on errors. Processors MAY report the advice, which never fails a build.

| Construct | Condition | Severity |
|---|---|---|
| Attributes | Unknown key for the directive, image, or table row | Error |
| Attributes | Value doesn't match the key's declared type | Error |
| Attributes | Bare key without a value | Error |
| Attributes | Unquoted value containing a reserved character | Error |
| Attributes | Attribute block that doesn't parse (such as an unclosed quote or brace, or `=` with no value) | Error |
| Attributes | The same key given more than once | Error |
| Directives | Directive-shaped line (`@word` followed by `{`, `:`, or end of line) with an unknown name | Warning |
| Directives | Primary given to a directive that takes none, or a required primary missing | Error |
| Directives | Text on a directive line that fits no part of it: after the name or attributes, after `@end`, or after an identifier primary | Error |
| Container | Container not closed before its enclosing block ends | Error |
| Container | Trailing `:` on a directive with no container form | Error |
| Container | Container-only directive without a trailing `:` | Error |
| Container | Container still open when the next arm of its group begins (reported at that arm's opener) | Error |
| Container | End line with no open container | Error |
| Container | End line indented differently from its opener | Error |
| Container | Nesting deeper than two levels | Warning |
| Binding | Following-block directive with no following block in its container | Error |
| Binding | Following-block directive bound to a heading | Error |
| Binding | Blank line between a following-block directive and its block | Warning |
| Binding | Heading-bound directive that isn't at the top of its section | Error |
| Title | Title given to a directive that doesn't accept one | Warning |
| Title | A `. ` line (dot and space) directly above a directive that accepts a title | Warning |
| `@id` | Duplicate id on a page, including ids from included content (page level) | Error |
| `@id` | Permalink published by one build in more than one place, names compared ignoring case: on two pages, or as a page's own and a heading's (page level) | Error |
| `@id` | Id, or a page's `permalink`, containing characters other than letters, digits, hyphens, underscores, and periods | Error |
| `@include` | Target file doesn't exist | Error |
| `@include` | Target id doesn't exist in the target file (page level) | Error |
| `@include` | Include cycle | Error |
| `@include` | A page's includes expand it past the processor's limit (page level) | Error |
| `@include` | `{heading=false}` without an `#id`, which has no effect | Warning |
| `@variant` | No arm of a group survives a build's selection (page level) | Warning |
| `@variant` | Unknown dimension or value | Error |
| `@variant` | Group mixes labeled and dimensional arms | Error |
| `@variant` | Arm has both a title and attributes, or neither | Error |
| `@variant` | Dimensional arms share no dimension key | Error |
| `@available` | Spec that doesn't parse, in a directive or in `available` frontmatter | Error |
| `@available` | Unknown target or state | Error |
| `@available` | History out of chronological order | Error |
| `@available` | Versions given for a versionless target | Error |
| `@available` | Spec exceeds its enclosing scope | Error |
| `@available` | Version newer than every release line its target declares | Error |
| `@available` | `planned` for a target with no planned line, or more than one | Error |
| `@steps` | Bound block isn't an ordered list | Error |
| `@details` | Missing title | Error |
| `@snippet` | Address that isn't `<source>:<path>`, optionally with `#<region>` | Error |
| `@snippet` | Source the content model doesn't declare | Error |
| `@snippet` | File doesn't exist, or its source doesn't include it | Error |
| `@snippet` | File isn't text | Error |
| `@snippet` | Region doesn't exist in the file | Error |
| `@snippet` | The file's tags are unbalanced, name a region twice, or use a reserved tag | Error |
| Source copies | `ascribe.lock` isn't valid TOML, or doesn't have the shape §7.4 gives it | Error |
| Source copies | `ascribe.lock` pins a source the content model doesn't declare with `git`, or pins it with another `git` | Error |
| Source copies | A file `ascribe.lock` lists is missing from the source's folder, or its hash differs | Error |
| Source copies | A file in a source's copies folder that `ascribe.lock` doesn't list | Error |
| Source copies | A copy that no snippet uses | Warning |
| Permalinks list | `ascribe.permalinks` isn't valid TOML, or doesn't have the shape §7.5 gives it | Error |
| Permalinks list | A permalink the content declares, or a name `ascribe.addresses` moves or retires, that `ascribe.permalinks` doesn't list as it is | Warning |
| Permalinks list | A name `ascribe.permalinks` lists that the project no longer has: not declared, and not moved or retired in `ascribe.addresses` | Warning |
| Addresses file | `ascribe.addresses` isn't valid TOML, or doesn't have the shape §7.10 gives it | Error |
| Addresses file | A page's address, or its Markdown version's, that the base published and nothing accounts for | Warning |
| Addresses file | A heading's explicit id that the base published on a page still published, and nothing accounts for | Advice |
| Addresses file | A build the base has and the working tree doesn't, other than a line archived now | Advice |
| Addresses file | A `[moved]` value that isn't a page, or a heading on one, in every build that published its key, or, for a permalink, a permalink the content declares | Error |
| Addresses file | A key of `[moved]` or `[retired]` whose source is back: a page, a heading's `@id`, or a permalink the content declares | Advice |
| Addresses file | A key of `[moved]` or `[retired]` whose address another source publishes now | Error |
| Addresses file | An entry the base's `ascribe.addresses` doesn't have, whose key the base never published | Advice |
| Addresses file | A key written twice in a table, or in both `[moved]` and `[retired]` | Error |
| Addresses file | A `[moved]` value that's another entry's key | Advice |
| Addresses file | An `#id` entry from a page that's gone whose id is live on the page its page's addresses lead to | Error |
| Addresses file | `ascribe.addresses` deleted while the base has it | Error |
| Addresses file | Many addresses forwarded to one index page or the home page | Advice |
| APIs | A declared spec that doesn't exist, can't be read, or isn't YAML or JSON | Error |
| APIs | A declared spec with no operations | Advice |
| Memo files | A memo file that can't be read, has no frontmatter or frontmatter that isn't valid YAML, has no `about`, has an `about` that isn't written as a target, has a key other than `about` and `quote`, or has a `quote` with an `about` that names a content model entry other than a glossary term; or a `memos/` folder inside the content root | Error |
| Memo files | A memo file's `about` that names nothing in the project | Warning |
| Navigation | `ascribe.nav` isn't valid TOML, or doesn't have the shape §7.8 gives it | Error |
| Navigation | A `pages` entry in `ascribe.nav` that isn't a page in its section's folder, or that names the section's own `index.md` | Error |
| Navigation | A `folder` in `ascribe.nav` that isn't a section | Error |
| Navigation | A folder `ascribe.nav` lists twice, or a page a section's `pages` names twice | Error |
| Navigation | A folder with an `index.md` and a page beside it with the folder's name | Error |
| Glossary | Two terms, or a term and an alias, with the same text, or two terms with the same id | Error |
| Glossary | A term whose section doesn't start with a paragraph | Error |
| Glossary | `@term` on a heading that no page the glossary lists shows | Error |
| `@summary` | Following form whose block isn't a paragraph | Error |
| `@summary` | A second summary in one section, including summaries from included content (page level) | Error |
| `@summary` | Summary inside a `@variant` arm, including one an include brings into an arm (page level) | Error |
| Project widget | Violates its declared schema | Error |
| Links | Target file doesn't exist | Error |
| Links | Target id doesn't exist in the target file (page level) | Error |
| Links | Target is a fragment | Error |
| Links | Target id is removed by a build (page level, per build) | Error |
| Links | Target page isn't published by a build (page level, per build) | Error |
| Links | Destination is a route rather than a file path | Warning |
| Links | Target is an operation the declared spec doesn't have | Error |
| Links | Target is a declared spec outside the project's folder, with no operation | Error |
| Links | Target is an operation with no id, by its method and path, and the API's reference is a template | Error |
| Links | Target is an operation that has an id, by its method and path | Advice |
| Links | Target is an operation the spec marks deprecated | Advice |
| Links | An `@operation`'s link isn't to an operation of a declared API | Error |
| Examples | A marked example that disagrees with its body's schema | Error |
| Examples | A marked example that isn't JSON, or isn't a `json` block | Error |
| Examples | A marked example whose operation has no body it names | Error |
| Examples | A marked example whose schema can't all be checked | Advice |
| Images | Local source doesn't exist | Error |
| Images | Missing alt text | Warning |
| Images | Required image attribute missing | Error |
| Phrases | `{key}` in prose whose key isn't declared | Warning |
| Phrases | A declared `{key}` directly between two more braces (`{{key}}`), usually a substitution left over from another tool | Warning |
| Headings | No `@id`, and the heading contains a phrase | Warning |
| Headings | No `@id`, and the heading's slug is empty (its text is only punctuation or emoji) | Warning |
| Headings | No `@id`, and the heading's slug is the same as another heading's on the page, so its id is numbered (page level) | Warning |
| Frontmatter | Key the file's content type or the fragment schema doesn't declare, other than a reserved key on a page or `intended` on a fragment | Error |
| Frontmatter | Required field missing | Error |
| Frontmatter | Value doesn't match the field's declared type | Error |
| Frontmatter | Reserved key (`available`, `variant`, `permalink`, `listing`, `replaces`, `type`, `nav-label`) on a fragment | Error |
| Frontmatter | `listing` looser than a rule of the content model the page is under | Error |
| Frontmatter | `replaces` that doesn't name a page | Error |
| Frontmatter | A page that `replaces` itself, or a chain of `replaces` that comes back to where it started (page level) | Error |
| Frontmatter | A page and the page it `replaces`, both published by one line of a versioned build (page level, per build) | Error |
| Frontmatter | Two pages that `replaces` the same page, both published by one line of a versioned build (page level, per build) | Error |
| Frontmatter | `listing` looser than `[listing] older-lines`, on a page an older line of a versioned build publishes (page level, per build) | Error |
| Frontmatter | Page names no content type, in a project that declares types | Error |
| Frontmatter | Page's `type` names no declared content type | Error |
| Frontmatter | Frontmatter that isn't valid YAML | Error |
| Files | Source file that can't be read, or isn't valid UTF-8 | Error |
| Lists | Unindented directive line ends a list | Warning |
| Lists | Directive line over-indented into an indented code block | Warning |
| Lists | An ordered list continues the numbering of a list bound by `@steps` right after it ends (usually an unindented directive split the list) | Warning |
| `@intended` | Names no check, a check that doesn't exist, a check that isn't a review check, or a check reported somewhere else | Error |
| `@intended` | An `intended` frontmatter entry that isn't a mapping with a `check` and a reason, or an acknowledgement whose reason is still a processor's placeholder | Error |
| `@intended` | An acknowledgement that matches no problem in any build | Advice |
| Content model | A name used in more than one role (dimension name, dimension value, lifecycle state, or feature key), or a dimension value in more than one dimension | Error |

### 8.3 Canonical form

Processors accept any spacing the grammar allows (§3.1, §3.3). Each construct still has one canonical spelling, which is what a formatter writes. Processors SHOULD offer to rewrite source into canonical form, and SHOULD report non-canonical source as a formatting issue, not as an error.

- One space between a directive name and `{`.
- No space inside braces: `{type=caution}`.
- No spaces around `=` or `|`. Pairs separated by `, `.
- Attributes in the order the schema declares them.
- Values quoted only when necessary.
- No empty attribute block: `@note`, not `@note {}`.
- No trailing whitespace after a container's `:`.
- No blank line between a following-block directive and its block, or between a container that binds and the block after its end line.
- `:` directly after the name or attribute block, followed by one space before a primary, or ending the line for a container.
- Directive lines inside a list item indented exactly to the item's content column.

A formatter changes only Ascribe constructs, and never how a page renders:

- It leaves alone a construct that has an error, and fixes only what's certain: a construct with a warning is still formatted.
- It removes trailing whitespace only after a container's `:`. After a text primary, trailing spaces can be a hard line break.
- Directive lines in a blockquote keep the `>` and one space. Directive lines on a list marker's own line, and indentation written with tabs, are left as written.
- It keeps a blank line between a following-block directive and its block when removing it would change how the page renders (a list's tightness), when the directive has a text primary, or when anything but blank lines sits in the gap (a link reference definition).
- It leaves an attribute block with a bare or repeated key as written. An undeclared key keeps its place, and the rest are ordered and respaced. A quoted value that needs no quotes loses them.
- It removes an empty attribute block after an image, and leaves an image's attribute block in a table cell, and a table row's attribute block, as written.
- Title lines are left as written, and so are a raw container's lines (§3.5). Its opener and end line keep their indentation when it holds an `@end` at another indent, which moving them could make close it.

---

## 9. Compilation

### 9.1 Pipeline

A compiler processes a documentation set in four stages:

```
source files
  → parse and validate    the same parser and validator the authoring environment uses
  → resolve               independent of the consumer
  → resolved tree
  → emit                  one emitter per output
```

Because the compiler and the authoring environment share one parser and validator, a command-line check reports exactly what the editor reports.

### 9.2 Resolution

Compilers MUST produce results equivalent to applying these steps in order:

1. **Includes and snippets.** Replace each `@include` with its target content (§4.2), recursively, and each `@snippet` with its code block (§4.8). Included content keeps its source file, for resolving relative paths.
2. **Availability.** Resolve feature keys and inherited scopes (§4.4).
3. **Build modes.** Apply the build's variant and availability modes (§9.3).
4. **Phrases.** Substitute phrases (§5.1).
5. **Heading ids.** Assign every heading its page id (§5.5). Source ids are computed per file, before step 1, because includes and links use them.
6. **Links.** Fill in empty link text from target titles, and rewrite file-path destinations into routes (§5.2).
7. **Glossary.** Link glossary terms (§5.4).

Titles and bindings are attached during parsing. Page-level validation (§8.1) runs on each page once heading ids and links are resolved (after step 6), for each build.

### 9.3 Build modes

A content model declares named builds. Each build sets a variant mode and an availability mode. The source is the same for every build.

```yaml
builds:
  site:      { variants: switch, availability: badge }
  cloud-pdf: { variants: { deployment: cloud }, availability: { filter: cloud } }
  sm-3.3:    { variants: switch, availability: { filter: self-managed 3.3 } }
```

#### Variant mode

- **`switch`** keeps every arm of every group, for readers to switch between, and keeps every page.
- **Selection**, a mapping from dimension names to a value or a list of values (for example `{ deployment: cloud }`), selects only along the dimensions it names. Everything keyed on other dimensions behaves as in `switch`.

Under a selection, an arm or page **conflicts** with the build when, for some dimension the selection names, the arm's attributes or the page's `variant` frontmatter name that dimension but none of the selected values. Then:

- **Pages** that conflict are dropped. Pages without a `variant` key never conflict.
- **Dimensional groups:** conflicting arms are removed. If one arm remains, its content replaces the group. If several remain, they stay a group, rendered as in `switch`. If none remain, the group is removed, and processors SHOULD warn (§8.2).
- **Groups keyed only on unselected dimensions**, and **labeled groups**, are unaffected and rendered as in `switch`.

For example, a build selecting `{ deployment: cloud }` reduces a `deployment` group to its cloud arm, and leaves a `pm` group as a full switcher.

#### Availability mode

- **`badge`** keeps all content and annotates it with its availability.
- **`filter`**, given a target and, for versioned targets, a version (for example `self-managed 3.3`), removes content that isn't available for them. Content that remains is annotated as in `badge`. A page whose frontmatter `available` makes it unavailable is dropped from the build, like a conflicting page.

Content is **available** for target *T* at version *V* when its effective availability (§4.4, after inheritance):

1. has no spec at all, or lists *T*, directly or through *T*'s dimension name; and
2. has a state in effect for *T* at *V* that counts as available (§7.2).

The state in effect at *V* is the last state in the target's history whose start version is at or before *V*. If *V* precedes the first state's start version, no state is in effect, and the content isn't available. A bare target with no version is in effect at every version. A versionless target has a single state, which is in effect at every version, so content marked `cloud removed` is never available for `cloud`. Content that has no spec, or reaches *T* through inheritance without a state, is treated as generally available.

A build MAY instead be **versioned** by a versioned target *T* whose live release lines are declared (§7.2). Its availability mode is `badge`, and it gives one output for each live line *L*. Content is **in** *L* when its effective availability has no spec, or doesn't list *T*, directly or through *T*'s dimension name, or has a state that counts as available in effect for *T* at some version in *L*. Content that isn't in *L* is removed from *L*'s output as `filter` removes it, and what remains is annotated as in `badge`, except that a version at or before the oldest live line isn't shown, since every live line has that content. In a history, the states that start before the oldest live line aren't shown, and the state in effect at it is shown with its own start version, so the history still names when it starts; a history whose last state is in effect there shows that state alone. The current line's output also keeps content whose versions are all later than every live line; an older line's doesn't. A link to a page a line doesn't publish resolves as §5.2 says, to the page that replaces it or to the page in another line.

### 9.4 Outputs

A compiler MUST provide the site output and the plain-markdown output. It MAY provide the JSON output.

**Site output: markdown plus web components.** CommonMark with custom elements for constructs that need presentation or interaction. It depends on no consumer component system. Ascribe provides an element library for it (§9.7).

**Plain-markdown output.** Fully resolved CommonMark with no HTML, for LLM consumption, search indexing, and export. Links are absolute URLs.

- A page begins with its title as a level-1 heading, then its page-level availability line, if it has one. Its other frontmatter isn't included.
- A page of a versioned build (§9.3) with more than one live line is preceded by a blockquote that says which line it documents, by the target's label and the line's number, and the last day of the line's support when the content model gives one. An older line's also names the current line and links to the page's canonical address (§9.4, page records) when that isn't its own, and to the current line's root page otherwise.
- Raw HTML in the source keeps its text and loses its tags, so `<kbd>Ctrl</kbd>` becomes `Ctrl`. Comments, and the contents of `<script>` and `<style>`, are dropped.
- An inline `<br>` becomes a hard line break, or `; ` where the text stays on one line (a table cell or a heading), so code spans either side of it stay apart. A line break in the source right after a `<br>` is the same line break, and nothing is added for a `<br>` at the start or end of a paragraph, cell, or heading.
- An availability line gives each target's label, then its state and the version it begins at, as in the element library (§9.7): `Self-managed (preview, 3.4+)`, `Self-managed (GA, 3.3+)`, or, for a history, `Self-managed (preview 3.3, GA 3.5, deprecated 4.0)`. A target whose last version is in a planned release line (§7.2) gives the content model's planned label in place of that version's state, and no `+`: `Self-managed (planned, 3.6)`. Targets are separated by `; `.
- A dimensional arm's label joins the labels of one attribute's values with ` / `, and several attributes with `, `. A note without a title leads with its type's label alone (`**Tip**`).
- A widget becomes its plain fallback followed by its content, unless the widget drops its content; its title, primary, and attributes aren't shown.
- Image attributes aren't part of plain markdown and are left out. A table keeps each column's alignment.
- A `@variant` group with more than one arm left, which the site output makes a switcher, opens with a line saying a reader sees one of its arms at a time, and by which dimension's label when the arms are dimensional: `A reader sees one of the next 3 sections at a time, chosen by package manager.` Each arm is then a section under a heading one level below the section the group is in (at most level 6), naming each of its dimensions and values, as `### Package manager: pnpm`, or with a labeled arm's title.
- A heading inside a `@variant` arm that stays a labeled section ends with the arm's label in parentheses, as `## Install (pnpm)`, and with each enclosing arm's, outermost first, when groups nest: so every section's headings stay distinct.

**Outputs for agents.** A content model MAY ask for the outputs AI agents read (`[consumer] agents = true`), following the Web Documentation Delivery Spec 0.6.0. A compiler that supports it lays the plain-markdown output out by URL, so that it's published beside the site: each page at its route with `.md`, opening with a blockquote that links to the index; and the index, `llms.txt`, listing every page the build publishes and offers (§2.1), by its title, in the sections of the build's navigation (§7.8): the content root's pages first, under its label, then each top section, its nested sections' pages in reading order, each linked to its Markdown version with its description. An index too long for one fetch is split into one per top section. In the site output, each page opens with a visually hidden pointer to the index and to the page's Markdown (§9.7). Every link in these outputs is absolute, so they need the site's origin: a content model that gives it (`[consumer] site`) publishes them unless it says `agents = false`, and one that doesn't publishes them only by asking, which without the origin makes the content model invalid. In a versioned build with more than one live line (§9.3), each line has its own, under its base path: its `llms.txt` lists the line's pages, its heading names the line, and an older line's opens with a paragraph that links to the current line's `llms.txt`.

**Page records.** The site output holds a record of each page the build publishes: its route, its source path, its title and description as plain text, its canonical address when the content model gives the site's origin, its `listing` (§2.1) and where it comes from (the page, a rule of the content model, or neither), and its Markdown version's address when the build publishes one; and the content model's `robots.txt` lines, each path in an `Allow` or `Disallow` line from the host's root, with a group for each kind of crawler the content model disallows, naming every crawler of that kind the processor knows. A page's canonical address is its own, except in an older line of a versioned build (§9.3), where it's the address in the current line of the same page, or of the page the current line publishes in its place, following `replaces` (§2.1) as a link does (§5.2); a page the current line has neither of keeps its own. Every output that says something about a page to a reader that isn't a person, `llms.txt` among them, is written from the records, so no two of them disagree.

**Navigation.** The site output holds the build's navigation (§7.8), a tree: the content root's section, each section with its folder, its label, its own page, its pages, and its nested sections, in order, and each page with its source path, its label, and its content type, joined to its record on the source path. A page's label is its `nav-label`, else its title, and a section's as §7.8 gives it, each as plain text, and in the output's formatted form where the title has one. A build with no pages has an empty root. The JSON output holds the same tree, its formatted labels in its own form.

**JSON output.** The resolved tree, for custom consumers. A heading carries its section's summary (§4.12): its text, and which form gives it. An `@operation` that names an operation is a block of type `operation`, with its API, its `operationId` when it has one, its method, path, summary when it has one, whether it's deprecated, and its address (§4.13).

| Source | Site output | Plain-markdown output |
|---|---|---|
| `@note {type=tip}` with title | `<ascribe-note type="tip" heading="…">` wrapping the content | A blockquote beginning `**Tip: …**` |
| `@steps` | `<ascribe-steps>` wrapping the list | The ordered list |
| `@variant` group, `switch` | `<ascribe-tabs sync="…">` containing one `<ascribe-tab value="…" label="…">` per arm | A line saying a reader sees one arm at a time, then each arm as a section under a heading naming its dimension and value, its headings ending with the label |
| `@variant` group, selection | The arms that survive the selection (§9.3): one arm becomes plain content; several stay a `<ascribe-tabs>` group | One arm becomes plain content; several stay a switcher's sections, as in `switch` |
| `@details` | `<details>` with the title in `<summary>` | The title in bold, then the content |
| `@available`, `badge` | A `<ascribe-availability>` element; page-level availability passed through as frontmatter | A line such as "Available: Quill Cloud (GA); self-managed (preview, 3.4+)" |
| `@available`, `filter` | Unavailable content removed; the rest annotated as in `badge` | Unavailable content removed; the rest annotated as in `badge` |
| Table row's `available`, `badge` | A `<ascribe-availability scope="row">` element at the end of the row's first cell, on one line, whose text is the plain output's: its targets between a visually hidden "(Available: " and ")" | The text "(Available: …)" at the end of the row's first cell |
| Table row's `available`, `filter` | Unavailable rows removed; the rest annotated as in `badge` | Unavailable rows removed; the rest annotated as in `badge` |
| Project widget | A custom element with the widget's name and attributes | The widget's plain fallback, or nothing |
| `@snippet` | A fenced code block | A fenced code block |
| `@summary` | Nothing for `@summary: text`; for `@summary`, the paragraph as it'd be without it | The same as the site output |
| `@operation` | An `<ascribe-operation>` element with the operation's API, method, path, id, and deprecation, holding a link to its address whose text is its method in capitals, its path, and `: ` and its summary when it has one, then ` (deprecated)` when it's deprecated | A paragraph of the same text, its link absolute |
| Phrases, includes, links, glossary | Resolved into ordinary markdown | Resolved; links made absolute |
| A link out of a line (§5.2) | The link, followed directly by an attribute marker setting `data-ascribe-release-line` on its `<a>` to the line it leads to | The link, then a space and the line in parentheses, outside its text: `[Configure tracing](…) (3.5)` |

Labels for dimension values come from the content model's display labels.

In the site output, emitters MUST place a blank line after each opening tag and before each closing tag of an element that wraps markdown, so that CommonMark parses the wrapped content as markdown.

**Assets.** Every output is self-contained: it works without access to the source files. Local files a page references (image sources, and link targets that aren't pages) are copied into the output, and references to them are rewritten to point at the copies. A reference resolves from the file it's written in, so an image referenced inside an included fragment is the one beside the fragment (§4.2). The consumer profile decides where copies go and how references to them are written (§9.5), so that a consumer's own image processing still applies.

A reference MUST resolve to a file inside the project root (the directory containing `ascribe.toml`) or the content root, and not inside the output directory; any other reference is treated as a file that doesn't exist (§8.2). The one exception is a link to an operation of a declared API (§5.2), whose spec MAY be outside the project's folder (§7.11): it names the operation, not the file, which isn't copied. File names MUST match exactly, including case, on every platform, so a project checks the same everywhere. References inside raw HTML aren't assets: they pass through unchanged, and the files they name aren't copied.

### 9.5 Consumer profile

The consumer profile, declared in the content model, describes how the site output fits a specific consumer:

- **Routing:** how file paths map to URLs, including any locale prefix.
- **Slugging:** the algorithm the consumer uses for heading ids.
- **Heading ids:** how to emit an explicit id so the consumer keeps its own heading and table-of-contents processing.
- **HTML passthrough:** whether the consumer renders raw HTML in markdown.
- **Images and assets:** how to emit image attributes, where copied assets go, and how references to them are written, so the consumer's image processing still applies.
- **Outputs for agents:** whether builds publish `llms.txt` and each page's Markdown (§9.4), and the site's origin their links need. With the origin, they're published unless the content model turns them off; without it, they aren't.
- **Permalinks:** where each permalink's address is published (§4.1). A compiler lists each build's permalinks with the page and anchor each leads to, and the consumer publishes at each address a page that sends the reader there.

### 9.6 Astro

Astro is the primary consumer, through its content collections.

- Compiled pages from the site output are loaded into an Astro content collection.
- The collection's schema MUST be generated from the content model, as a Zod schema.
- Page layouts are the project's own.
- Page-level frontmatter, including `type` and `available`, reaches the layout as collection data, except `listing`, which reaches the site in the page's record (§9.4). A field that reads code spans (§7.2) arrives as its plain text, with its formatted form as HTML under the reserved `formatted`. Every page's frontmatter also has the reserved `source`: the page's path from the content root, with `/` between its parts, so a layout can link to the page's source without reading the path the collection loaded it from. `available` arrives as a list of its targets, each with what a layout needs to show it: the target, its dimension, states, versions, and display text (element contract §4).

An Astro integration SHOULD provide the collection configuration and generated schema, load the element library, and supply the markdown processing needed for explicit heading ids.

### 9.7 Element library

The element library implements the custom elements used by the site output.

- Elements render into the light DOM, so site styles apply and content stays visible to search engines and assistive technology.
- Elements are styled with CSS and themed through CSS custom properties.
- Only elements that require interaction use JavaScript. Of the built-ins, that's `<ascribe-tabs>`.
- Without JavaScript, `<ascribe-tabs>` displays every arm with its label.
- The pointer for agents (§9.4) is hidden from sight, not from the page's text.

---

## 10. Authoring environment

An authoring environment is a processor that edits Ascribe source interactively. It SHOULD provide the following.

- **Diagnostics** from §8, reported as the author types.
- **Completion** for:
  - directive names and attribute keys and values;
  - dimension values and lifecycle states;
  - phrase keys and feature keys;
  - include paths;
  - link targets, searched by page and heading title and inserted as file paths.
- **Hover:**
  - for a link or include, the project-relative target path, title, and a plain-text preview of the first paragraph of the page or target heading's section, with declared phrases replaced and the preview cut at 280 characters;
  - for a phrase, its value;
  - for an availability spec or feature key, its resolved availability in the words of §9.4, with a feature's name and key;
  - for a directive or attribute key, its description from the schema; for a dimension key, its values with their labels.
- **Go to definition:** links and includes go to the target heading when they name a source id, otherwise to the top of the file. A missing id falls back to the existing file; a missing file has no target. Images and asset links go to their file. An `@id` primary goes to its heading. Declared phrases and feature keys used as a whole availability spec go to their entries in `ascribe.toml`; dimension values and lifecycle states have no definition.
- **Navigation:** a CodeLens or equivalent that names a link's or include's target file and opens it.
- **Document links:** inline link and image destinations as written, and an include's whole primary, are clickable when their target exists. They open the file or heading; an external link opens its URL, after declared destination phrases are replaced, when its scheme is `http`, `https`, or `mailto`, compared without case. An external link with any other scheme has no document link.
- **Inline hints:** just inside the `[` of an empty-text link, show the existing source target's title (the heading's text for a source id), with declared phrases replaced and a tooltip naming the destination. Images, missing targets, and targets without a title receive no hint. Hints use source titles without applying build modes.
- **Refactoring:**
  - Renaming or moving a file updates links and includes that point to it.
  - Changing a heading's id updates links to it.
  - Renaming a phrase key updates its uses.
- **Formatting** into canonical form (§8.3), with a formatter that understands Ascribe.
- **Distinct display of title lines**, so a paragraph that accidentally became a title is easy to spot.

Link completion searches pages and the headings of each page's own source file by title, case-insensitively; it does not offer other files' fragments or headings introduced by includes. A page without a title uses its file name. Matches are ranked by title prefix, word prefix, then substring, with paths disambiguating identical titles and pages before headings on a tie. Before any query is typed, offer pages and the current file's own headings. After `#`, offer the ids the named page can be linked by (§4.2), its fragments' included, matching its ids and heading titles. Reference-definition destinations are completed like inline link destinations; image-source completion is not required.

Include completion offers every source file, including fragments, and the named file's source ids after `#`. Insert paths relative to the file being edited, or relative to the content root when an include was begun with `/`; a link to the current file's heading inserts `#id`. Percent-encode destinations as needed (§5.2), except in angle-delimited link destinations. Show the target's content path with each link completion to distinguish identical titles. Search on the server, return at most 100 items, and mark truncated lists incomplete so the client can request again as the author types.

Target hovers identify fragments, missing files or ids, route-style links, and case mismatches. An include's CodeLens names its target file and, when applicable, its section, and opens that target; a missing file has no lens.

Source files store real file paths, but authors should rarely need to read or type them.

*Note (non-normative): the language server's include lens uses `ascribe.openFile` through `workspace/executeCommand`, followed by `window/showDocument` for clients that support it. Clients without that capability retain document links. Go to definition in `ascribe.toml` currently recognizes `[phrases]` and `[features.<key>]` table forms only. Reference-form links and their definition lines currently have no document link because the parser does not expose the definition destination span; their hover, go to definition, and hints still work. These are editor implementation choices and limitations, not restrictions on valid source or content models.*

*Note (non-normative): VS Code has no API for hiding text within a line. Hover, CodeLens, and inlay hints are the dependable ways to keep paths out of the author's way.*

*Note (non-normative): general CommonMark formatters don't know that title and directive lines start new blocks. To them, a title line, a directive line, and the text below are one paragraph, so a formatter that reflows paragraphs (for example, Prettier with `proseWrap: always`) joins them into one line and breaks the page. Ascribe projects should format with an Ascribe-aware formatter, or exclude Ascribe sources from other formatters.*

---

## 11. Versioning

Versions of this specification are numbered. A documentation set's content model declares the version it targets. Changes to the grammar or the built-in vocabulary are made only in new versions of this specification.

---

## Appendix A. Grammar

ABNF (RFC 5234). `SP`, `DIGIT`, `ALPHA`, `DQUOTE`, and `VCHAR` are the RFC 5234 core rules. Container indentation and blockquote markers (§3.9) are stripped before these rules apply.

```abnf
OWS             = *( SP / HTAB )                          ; optional spaces or tabs
RWS             = 1*( SP / HTAB )                         ; required spaces or tabs

directive-line  = "@" name [ OWS attributes ] [ OWS ":" [ OWS primary ] ] OWS
                                        ; ":" with no primary after it opens a container
end-line        = "@end" OWS
title-line      = "." title-start *title-char
title-start     = %x21-2D / %x2F-7E / UTF8-non-ascii      ; not space, not "."
title-char      = %x20-7E / UTF8-non-ascii

name            = builtin-name / widget-name
builtin-name    = LOWER *( LOWER / DIGIT )
widget-name     = builtin-name 1*( "-" 1*( LOWER / DIGIT ) )

attributes      = "{" OWS [ attribute *( OWS "," OWS attribute ) OWS ] "}"
attribute       = key OWS "=" OWS value
key             = LOWER *( LOWER / DIGIT / "-" )
value           = token / quoted / value-set
token           = 1*tchar
tchar           = %x21 / %x23-2B / %x2D-3C / %x3E-7A / %x7E / UTF8-non-ascii
                                        ; any visible character except " , = { | }
value-set       = token 1*( OWS "|" OWS token )
quoted          = DQUOTE *( qchar / "\" DQUOTE / "\\" ) DQUOTE
qchar           = %x20-21 / %x23-5B / %x5D-7E / UTF8-non-ascii

primary         = identifier / text
identifier      = 1*( VCHAR / UTF8-non-ascii )            ; no whitespace
text            = 1*( %x20-7E / UTF8-non-ascii )          ; first line only; continues
                                        ; onto following lines like a paragraph (§3.4),
                                        ; as a note's or a summary's text (§4.5, §4.12)

phrase          = "{" key "}"

snippet-address = key ":" snippet-path [ "#" region-name ]   ; §4.8
snippet-path    = path-segment *( "/" path-segment )
path-segment    = 1*( %x21-22 / %x24-2E / %x30-7E / UTF8-non-ascii )
                                        ; no "/", "#", or whitespace; not "." or ".."
region-name     = 1*( ALPHA / DIGIT / "-" / "_" / "." )

availability    = entry *( OWS "," OWS entry ) / feature-key
entry           = target [ RWS detail ]
detail          = when / state [ RWS when ] / "(" OWS history OWS ")"
history         = state RWS when *( OWS "," OWS state RWS when )
when            = version / %s"planned"           ; the target's planned line (§4.4)
target          = name-word
state           = name-word
feature-key     = name-word
name-word       = ALPHA *( ALPHA / DIGIT / "_" / "-" )    ; the "name" of §4.4 and §7.2
version         = 1*DIGIT *( "." 1*DIGIT )

LOWER           = %x61-7A
UTF8-non-ascii  = %x80-10FFFF
```

---

## Appendix B. Complete example

*Non-normative.* A complete page, assuming a content model that declares:

- the dimensions `pm` (`npm`, `pnpm`, `yarn`) and `deployment` (`cloud`, `self-managed`), with `cloud` versionless;
- the phrases `product` (Quill), `cloud` (Quill Cloud), `version` (3.4.1), and `api` (`https://api.quill.dev/v3/`);
- the content type `page`, with a `title` and a `description`, which the page names with `type: page`.

````markdown
---
type: page
title: Install the Quill agent
description: Install and configure the Quill agent to sync your docs to Quill Cloud or a self-managed Quill server.
available: cloud, self-managed preview 3.3
---

The {product} agent watches your docs repository and syncs changes to {product}. This page covers installing the agent with a package manager, configuring it, and connecting it to {cloud} or a self-managed server. If you only want to try {product}, see [Try {product} in the browser](quickstart.md#try-in-browser).

.Try it without installing
@note {type=tip}
You can run {product} in the browser at play.quill.dev with no local setup.

## Prerequisites
@id {permalink=true}: prerequisites

@include: _fragments/prerequisites.md

## Install the agent
@id {permalink=true}: install-agent

@steps
1. Install the agent package:

   @variant {pm=npm}:
   ```shell
   npm install -g @quill/agent
   ```
   @variant {pm=pnpm}:
   ```shell
   pnpm add -g @quill/agent
   ```
   @variant {pm=yarn}:
   ```shell
   yarn global add @quill/agent
   ```
   @end

2. Verify the install:

   ```shell
   quill --version
   ```

   The command prints the installed version, {version}.

   @note
   The agent needs write access to your repository's `.quill/` directory.

3. Create `quill.yaml` at the root of your repository:

   ```yaml phrases=true
   agent:
     version: {version}
     watch: docs/
   ```

## Connect to {product}
@id {permalink=true}: connect

@variant {deployment=cloud}:
Sign in to {cloud} and copy an API key from **Settings → Keys**, then add it to `quill.yaml`:

```yaml
cloud:
  api_key: ${QUILL_KEY}
```

@variant {deployment=self-managed}:
Point the agent at your server. Self-managed servers must run {product} Server 3.3 or later.

```yaml
server:
  url: https://quill.internal.example.com
```
@end

## Streaming sync
@id: streaming-sync
@available: cloud, self-managed preview 3.4

Streaming sync pushes changes as you save, instead of on each commit.

{cloud}'s streaming sync is enabled by default for new {cloud}-hosted workspaces.

For event formats, see the [streaming API reference]({api}streaming).

## Troubleshooting
@id: troubleshooting

@note {type=warning}:
If the agent exits immediately, check the log at `~/.quill/agent.log`.

A common cause is an expired API key. Generate a new key, then restart the agent. See [](keys.md#rotate-keys).
@end
````

Notes on the example:

- `@quill/agent` inside code is literal (§3.2).
- `${QUILL_KEY}` is literal because its fence doesn't opt in to phrases (§5.1).
- The troubleshooting note's trailing colon makes it a container, so both paragraphs are inside the callout until `@end` (§3.5).
- The streaming-sync section's availability fits inside the page's (§4.4).

---

## Appendix C. Design rationale

*Non-normative.* Why the language is shaped the way it is.

**Markdown, not MDX.** MDX mixes content with code, so every author has to work in JSX. Plain markdown keeps existing tooling and stays readable. MDX-style components are replaced by directives in source and by web components in output.

**`@` rather than `:::` directives.** The colon-fenced "generic directives" syntax was proposed for CommonMark in 2018 and never adopted; its only real implementation is the remark-directive library, and related dialects (MyST, Pandoc, Docusaurus) each differ from it. Ascribe couldn't version or own a grammar built on it. `@` lines also read better unrendered, and `@` almost never collides with prose: across roughly 4,500 pages of Astro, Elastic, and Docker documentation, no prose `@` matched an Ascribe keyword.

**One line grammar, attributes first.** Placing attributes before the primary keeps metadata next to the name, and gives a directive's line and container forms the same head. Braces were chosen over brackets because brackets collide with markdown link syntax.

**A bare `@end`, and containers kept rare.** Named closers are verbose, and counting delimiters (as in `:::` fences) is hard to read. A bare `@end` is simple, as in Ruby and Lua. The price is that deeply nested containers become hard to match by eye, so the language keeps them rare: most directives are single lines, and variance uses sibling arms instead of nesting. Every container opener ends in a colon, and nothing else does, so neither a reader nor a parser has to look ahead to find out whether an `@end` is coming.

**Groups whose arms close each other.** Tabs, switches, and steppers were the largest source of nesting in the surveyed corpora. Letting each arm end the previous one, as Ruby's `elsif` and HTML's `<li>` do, handles them with one closer and no nesting.

**`@note` rather than GitHub alerts.** GitHub's `> [!NOTE]` syntax renders on GitHub, but it requires a `>` on every line and can't carry a title. Source readability outranks renderer compatibility, so Ascribe keeps `@note` with a following-block form. In every corpus surveyed, 79–90% of callouts were a single block.

**Titles on their own line.** AsciiDoc's `.Title` convention keeps a title as readable text rather than a quoted attribute. The same line labels one-off `@variant` arms, so the language has one way to name things.

**Free spacing, one canonical spelling.** Invisible differences in spacing never change meaning or cause errors; a formatter writes the canonical form. Binding follows what a reader sees: a directive annotates the block it touches, or the whole section when it sits at the top.

**Phrases as `{key}`.** Substitutions are heavily used (about 49,000 in Elastic's documentation), and 40% of them are directly followed by text (`'s`, plurals, hyphenated compounds). An `@`-prefixed form ends at whitespace, so it fails in those positions, and it also looks like a directive or a social handle. Braces are bounded and read as a placeholder.

**No `@ref`; empty-text links instead.** Only about 10% of surveyed in-site links used the target's own title as their text, so a dedicated directive wasn't worth it. Elastic's docs-builder fills in empty link text from the target, which covers the need with plain markdown.

**File paths, not routes.** File paths can be validated without knowing the consumer's routing, work on GitHub, update when files move, and stay correct in translated copies.

**Lifecycle states named by their start version.** Following Swift's `@available`, each state names the version where it begins, so availability needs no ranges or logic.

**No image style flags.** Elastic's most common image attribute, `screenshot`, only adds a shadow. That's styling, which belongs to the consumer.

**Project widgets marked by a hyphen.** Following HTML custom elements, shape distinguishes extensions from built-ins. An experimental prefix such as `x-` was avoided, since RFC 6648 documents how such prefixes outlive the experiment.

**Plain markdown and web components as output.** Output that depends on no consumer's component system (Starlight, Hugo shortcodes) keeps Ascribe portable, while Astro content collections remain the primary target.

---

## Appendix D. References

- CommonMark Spec — https://spec.commonmark.org/
- RFC 2119, *Key words for use in RFCs to Indicate Requirement Levels*
- RFC 8174, *Ambiguity of Uppercase vs Lowercase in RFC 2119 Key Words*
- RFC 5234, *Augmented BNF for Syntax Specifications: ABNF*
- RFC 6648, *Deprecating the "X-" Prefix and Similar Constructs in Application Protocols*
- WHATWG HTML Standard, custom elements — https://html.spec.whatwg.org/multipage/custom-elements.html
- AsciiDoc Language — https://docs.asciidoctor.org/asciidoc/latest/
- Djot — https://djot.net/
- Swift `@available` attribute — https://docs.swift.org/swift-book/documentation/the-swift-programming-language/attributes/
- Elastic docs-builder — https://github.com/elastic/docs-builder
- Astro content collections — https://docs.astro.build/en/guides/content-collections/
- TOML — https://toml.io/
- Zod — https://zod.dev/
