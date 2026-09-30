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

An Ascribe document is a CommonMark document. Everything CommonMark defines keeps its meaning, except where this specification assigns meaning to text CommonMark treats as ordinary paragraph content (directive lines, title lines, phrases, and attribute blocks after images).

Processors MAY support common CommonMark extensions, such as GitHub Flavored Markdown tables. This specification doesn't depend on them.

### 1.5 Notational conventions

- Syntax is given in ABNF (RFC 5234) in [Appendix A](#appendix-a-grammar); sections refer to its rules by name.
- `SP` is a single space (U+0020). "Line start" means the first character after any indentation or blockquote markers required by the enclosing CommonMark container, allowing up to three further spaces of indentation, as CommonMark allows before a heading (§3.9).
- Examples show Ascribe source unless labeled otherwise.

---

## 2. Documents

### 2.1 Files

An Ascribe source file is a CommonMark file with the extension `.md`. A file MAY begin with YAML frontmatter delimited by lines containing only `---`. The content model (§7) defines which frontmatter keys each content type accepts; this specification reserves two keys: `available` (§4.4) and `variant` (§4.3).

- Frontmatter MUST be valid YAML. A reserved key whose value isn't the shape its section defines (an availability spec, or a mapping of dimensions to values) is a frontmatter value of the wrong type (§8.2).
- The source files are exactly the files under the content root (§2.2) whose names end in `.md`. A file or directory whose name begins with `.` is skipped, along with everything in it. A source file MUST be valid UTF-8; one that can't be read, or isn't UTF-8, is an error on that file, and processors still check the rest.

### 2.2 Pages and fragments

A documentation set's source files live under a **content root**. Each file is either a page or a fragment.

- A **page** is published on its own and appears in navigation.
- A **fragment** exists only to be included by other files (§4.2). It isn't published on its own and doesn't appear in navigation.

A file is a fragment if any segment of its path, relative to the content root, begins with `_` (for example `_warning.md` or `_snippets/prerequisites.md`), or if its path matches a fragment pattern declared in the content model (§7.2). Otherwise it's a page.

Frontmatter on fragments is validated against the content model's fragment schema, not a page schema. Fragments MUST NOT use the reserved keys `available` and `variant`; availability inside a fragment is written with `@available`.

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

Attributes use one grammar everywhere: after a directive name, and after an image (§5.3).

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

- A **line primary** (an availability spec) is the rest of the directive line, with surrounding whitespace removed. It may contain spaces, but it isn't parsed as inline content and it never continues onto the following lines, so a paragraph directly below the directive is the block it binds. Only `@available` (§4.4) takes one.

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

Following-block directives **stack**: several in a row all bind the block the last of them touches. Any block except a heading can be bound, including a thematic break or a raw HTML block. A line-form directive whose content is its own text primary, such as a one-line `@note: …`, is a block too, so `@available` directly above it binds the note. A directive that renders nothing of its own, such as `@id`, `@include`, or an end line, isn't a block, so a following-block directive directly above one has nothing to bind.

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

### 4.2 `@include`

Transcludes a file or a heading's section into the current document.

```
@include: _snippets/prerequisites.md
@include {heading=false}: guides/setup.md#install
```

- **Form:** line. **Binding:** self. **Primary:** REQUIRED path, optionally followed by `#` and an id.
- **Paths** are relative to the including file, or, when they begin with `/`, relative to the content root. Processors MUST NOT resolve a bare filename by searching other directories. A path is percent-decoded as a link destination is (§5.2), so `my%20snippet.md` names `my snippet.md`, and an empty `#` (`file.md#`) includes the whole file.
- Only a source file (§2.1) can be included: a file outside the content root, or that isn't Markdown, is reported as a target that doesn't exist.
- With `#id`, only the section of the heading with that source id (§5.5) is included.
- **Attributes:**

  | Key | Type | Default | Meaning |
  |---|---|---|---|
  | `heading` | boolean | `true` | When `false`, the included section's own heading is omitted. It applies only with `#id`; on an include of a whole file it has no effect, and processors SHOULD warn |

- The target file and id MUST exist. Include cycles are an error: an include is a cycle when expanding it would, directly or through other includes, include the same section of the same file again, so its expansion would never end.
- Included content becomes part of the including page. Phrase substitution and build modes (§9) apply to it as they do to the rest of the page.
- Included headings keep the levels they're written with. Include a section where its levels fit the page.
- Bindings are decided in the file where a directive is written. With `{heading=false}`, a heading-bound directive under the omitted heading, such as `@available`, still describes the rest of the included section, as it does in the fragment.
- **Relative paths resolve from the file they're written in.** Every piece of content keeps its source file. Link destinations, image sources, and nested include paths inside a fragment resolve against the fragment's location, not the including page's. A fragment that links to `keys.md` means the `keys.md` next to the fragment, wherever it's included.
- **Ids are checked on the expanded page.** Headings and `@id`s in included content become ids of the including page, and every id on a page MUST be unique after expansion. Including the same fragment twice on one page, or including a fragment whose ids collide with the page's own, is an error reported at the include site (§8.1).
- **Links target pages, not fragments.** A fragment isn't published on its own, so a link to a fragment file is an error. A page's linkable ids are its own source ids (§5.5), not those of the fragments it includes: a link naming an id that exists only inside an included fragment is an error, and processors name the fragment when they report it. Link to the page that includes it.

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

#### Availability specs

A spec is a comma-separated list of **targets**, each optionally followed by its lifecycle (ABNF rule `availability`).

| Spec | Meaning |
|---|---|
| `cloud` | Generally available on `cloud` |
| `self-managed 3.3` | Generally available on `self-managed` since version 3.3 |
| `self-managed preview 3.4` | In preview on `self-managed` since version 3.4 |
| `self-managed (preview 3.3, ga 3.5, deprecated 4.0)` | A history: each state starts at its version and lasts until the next state begins |
| `cloud, self-managed preview 3.3` | Two targets |

- A **target** is a value of a declared dimension, or the name of a dimension, which stands for all of its values. A dimension name takes a state but never a version, since its values don't share one version line: write `deployment beta`, or name the value, as in `self-managed beta 3.4`.
- A **state** is a declared lifecycle state (§7.2). A target with no state is generally available (`ga`). A bare version means generally available since that version. A state with no version on a versioned target (`self-managed beta`) is in effect at every version, as a bare target is.
- Each lifecycle state declares whether content in that state **counts as available**. By default every state counts as available except `removed`.
- States in a history MUST be in chronological order, compared using the content model's version scheme. A target that the content model declares as versionless takes a single state and no versions.
- The language has no version ranges, alternatives, or negation. Each state names only the version where it begins.
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
- **Distinction from attributes:** `{…}` directly after a directive name, or directly after an image and closed on the same line, is an attribute block (§3.3), not a phrase, whatever it contains. So `![Logo](logo.png){cloud}` is an attribute block with a bare key, which is an error; to put a phrase right after an image, separate it with a space or escape it (`\{cloud}`).
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

- **Paths** are relative to the linking file, or relative to the content root when they begin with `/`. An optional `#id` names a heading in the target file by its source id; the compiled link points at that heading's page id (§5.5). Only the target file's own headings have source ids there; headings it includes from fragments don't (§4.2).
- The target file, and the id if present, MUST exist. A destination that is only `#id` names a heading in the file it's written in; in a fragment, that's a heading of the fragment itself, and the compiled link points at that heading's page id on each page that includes the fragment. In each build, the target page MUST also be published: a link to a page the build drops (§9.3) is an error in that build. To link to such a page from shared content, put the link in a `@variant` arm that the same build removes.
- **Empty link text** is replaced by the target's title: the heading text when an id is given, and the page title otherwise. A page's title is its frontmatter `title`, which every content type requires (§7.2).
- **External URLs** (with a scheme such as `https:`) are passed through unchanged.
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

The content model MAY declare a glossary (§7.2). Processors link occurrences of glossary terms to their definitions according to the glossary's settings. Authors don't mark glossary terms in source.

Terms are matched in each resolved page's text, after phrases are substituted and a build's modes are applied, so "the first occurrence" is the first a reader sees. Emphasized text counts as prose. A term isn't linked on the page it links to, and a term whose target page or heading a build doesn't publish isn't linked in that build.

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
| Content types | A frontmatter schema per page type, each requiring a string `title`, and a fragment schema. Fields are typed as string, number, boolean, date, enumeration, list, or object, and may be optional or have a default. Each type names the pages it applies to by path pattern, and at most one type is the default for pages no pattern matches | §2, §5.2 |
| Fragment patterns | Additional globs that mark files as fragments | §2.2 |
| Directive schemas | For each project widget: forms, primary kind, attributes, binding, title, groupable, plain fallback | §3, §6 |
| Dimensions | Each dimension's name, values, and display labels, and which values are versionless | §4.3, §4.4 |
| Version scheme | How versions are compared | §4.4 |
| Lifecycle states | Default `preview`, `beta`, `ga`, `deprecated`, `removed`, each declaring whether it counts as available (by default, all but `removed` do); extensible | §4.4 |
| Features registry | Feature keys, each with a name and an availability spec | §4.4 |
| Note types | Default `note`, `tip`, `important`, `warning`, `caution`; extensible | §4.5 |
| Phrases registry | Phrase keys and values | §5.1 |
| Glossary | Terms, definitions, and matching settings | §5.4 |
| Image attributes | Accepted image attribute keys and types | §5.3 |
| Consumer profile | Routing, slugging, heading ids, HTML passthrough, images | §9.5 |
| Builds | Named builds and their modes | §9.3 |

Built-in directive schemas are defined by this specification, not by the content model. A content model MAY extend the enumerations they use (note types, lifecycle states).

Dimension names, dimension values, lifecycle states, and feature keys are names (ABNF rule `name-word`). A name MUST NOT be used in more than one of these roles, and a dimension value MUST NOT belong to more than one dimension; processors reject a content model that does either when loading it. This keeps every bare word in an availability spec unambiguous (§4.4). Dimension names are also attribute keys (§4.3), so they follow the attribute key rule (ABNF rule `key`, §3.3).

The site output writes image attributes onto `<img>` elements and project widgets' attributes onto custom elements (§9.4), so a content model MUST NOT declare an attribute key that HTML already gives a meaning there: `src`, `alt`, or `title` for images; `heading` or `primary` for widgets, which the site output uses for a widget's title and identifier primary; and, for both, HTML's global attributes (such as `id`, `class`, `style`, `title`, `hidden`, and `slot`), ARIA attributes (`aria-…`), and event-handler attributes such as `onclick`. Keys that merely begin with `on`, such as `online`, are allowed. Processors reject a content model that does when loading it.

A page's content type is the one whose path patterns match it. A page matched by more than one type's patterns is an error; there's no precedence between types. A page no type matches gets the default type, and it's an error if there isn't one.

---

## 8. Validation

### 8.1 Validation levels

Validation happens at two levels.

- **File level.** Each source file on its own: syntax, attributes, directive schemas, frontmatter, and whether referenced files exist.
- **Page level.** Each page after includes are expanded, availability is resolved, and a build's modes are applied (§9.2), once per build. This covers checks that depend on the assembled page: id uniqueness, link targets that are ids, and anything a build removes. A build reports page-level problems only in content it publishes: content a build removes isn't checked for that build, which is what lets a link to a page the build drops sit in an arm the build removes. Content that no build publishes is still checked, as if one build kept everything, and its problems are reported as belonging to no build. A problem that appears in several builds is reported once, naming them. A fragment that no page includes has no page-level problems; its file-level ones are still reported.

A page-level diagnostic is reported at the source location that causes it. A diagnostic about what a link or image names is reported at its destination as written, for an inline link or image, and at the link or image itself for a reference form. When the cause is inside a fragment, it's reported at the include site, and processors SHOULD also report it in the fragment, as related information rather than as a second diagnostic.

When several places together cause a diagnostic, it's reported once, at the later one: the second of two duplicate ids or headings (at the `@id` line for an explicit id, at the heading for a slug), the second include of a fragment included twice, the include that closes a cycle, and the later of two declarations in the content model. A frontmatter problem with no line of its own, such as a missing field, is reported at the file's first line.

### 8.2 Diagnostics

Conforming processors MUST report every error below, and SHOULD report the warnings. An error means the document isn't conforming; a build MUST fail on errors.

| Construct | Condition | Severity |
|---|---|---|
| Attributes | Unknown key for the directive or image | Error |
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
| `@id` | Id containing characters other than letters, digits, hyphens, underscores, and periods | Error |
| `@include` | Target file doesn't exist | Error |
| `@include` | Target id doesn't exist in the target file (page level) | Error |
| `@include` | Include cycle | Error |
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
| `@steps` | Bound block isn't an ordered list | Error |
| `@details` | Missing title | Error |
| Project widget | Violates its declared schema | Error |
| Links | Target file doesn't exist | Error |
| Links | Target id doesn't exist in the target file (page level) | Error |
| Links | Target is a fragment | Error |
| Links | Target id exists only inside a fragment the target page includes (page level) | Error |
| Links | Target id is removed by a build (page level, per build) | Error |
| Links | Target page isn't published by a build (page level, per build) | Error |
| Links | Destination is a route rather than a file path | Warning |
| Images | Local source doesn't exist | Error |
| Images | Missing alt text | Warning |
| Images | Required image attribute missing | Error |
| Phrases | `{key}` in prose whose key isn't declared | Warning |
| Phrases | A declared `{key}` directly between two more braces (`{{key}}`), usually a substitution left over from another tool | Warning |
| Headings | No `@id`, and the heading contains a phrase | Warning |
| Headings | No `@id`, and the heading's slug is empty (its text is only punctuation or emoji) | Warning |
| Headings | No `@id`, and the heading's slug is the same as another heading's on the page, so its id is numbered (page level) | Warning |
| Frontmatter | Key the file's content type or the fragment schema doesn't declare, other than a reserved key on a page | Error |
| Frontmatter | Required field missing | Error |
| Frontmatter | Value doesn't match the field's declared type | Error |
| Frontmatter | Reserved key (`available`, `variant`) on a fragment | Error |
| Frontmatter | Page matches more than one content type, or matches none and there's no default type | Error |
| Frontmatter | Frontmatter that isn't valid YAML | Error |
| Files | Source file that can't be read, or isn't valid UTF-8 | Error |
| Lists | Unindented directive line ends a list | Warning |
| Lists | Directive line over-indented into an indented code block | Warning |
| Lists | An ordered list continues the numbering of a list bound by `@steps` right after it ends (usually an unindented directive split the list) | Warning |
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
- No blank line between a following-block directive and its block.
- `:` directly after the name or attribute block, followed by one space before a primary, or ending the line for a container.
- Directive lines inside a list item indented exactly to the item's content column.

A formatter changes only Ascribe constructs, and never how a page renders:

- It leaves alone a construct that has an error, and fixes only what's certain: a construct with a warning is still formatted.
- It removes trailing whitespace only after a container's `:`. After a text primary, trailing spaces can be a hard line break.
- Directive lines in a blockquote keep the `>` and one space. Directive lines on a list marker's own line, and indentation written with tabs, are left as written.
- It keeps a blank line between a following-block directive and its block when removing it would change how the page renders (a list's tightness), when the directive has a text primary, or when anything but blank lines sits in the gap (a link reference definition).
- It leaves an attribute block with a bare or repeated key as written. An undeclared key keeps its place, and the rest are ordered and respaced. A quoted value that needs no quotes loses them.
- It removes an empty attribute block after an image, and leaves an image's attribute block in a table cell as written.
- Title lines are left as written.

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

1. **Includes.** Replace each `@include` with its target content (§4.2), recursively. Included content keeps its source file, for resolving relative paths.
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

### 9.4 Outputs

A compiler MUST provide the site output and the plain-markdown output. It MAY provide the JSON output.

**Site output: markdown plus web components.** CommonMark with custom elements for constructs that need presentation or interaction. It depends on no consumer component system. Ascribe provides an element library for it (§9.7).

**Plain-markdown output.** Fully resolved CommonMark with no HTML, for LLM consumption, search indexing, and export. Links are absolute URLs.

- A page begins with its title as a level-1 heading, then its page-level availability line, if it has one. Its other frontmatter isn't included.
- Raw HTML in the source keeps its text and loses its tags, so `<kbd>Ctrl</kbd>` becomes `Ctrl`. Comments, and the contents of `<script>` and `<style>`, are dropped.
- An availability line gives each target's label, then its state and the version it begins at, as in the element library (§9.7): `Self-managed (preview, 3.4+)`, `Self-managed (GA, 3.3+)`, or, for a history, `Self-managed (preview 3.3, GA 3.5, deprecated 4.0)`. Targets are separated by `; `.
- A dimensional arm's label joins the labels of one attribute's values with ` / `, and several attributes with `, `. A note without a title leads with its type's label alone (`**Tip**`).
- A widget becomes its plain fallback followed by its content, unless the widget drops its content; its title, primary, and attributes aren't shown.
- Image attributes aren't part of plain markdown and are left out. A table keeps each column's alignment.

**JSON output.** The resolved tree, for custom consumers.

| Source | Site output | Plain-markdown output |
|---|---|---|
| `@note {type=tip}` with title | `<ascribe-note type="tip" heading="…">` wrapping the content | A blockquote beginning `**Tip: …**` |
| `@steps` | `<ascribe-steps>` wrapping the list | The ordered list |
| `@variant` group, `switch` | `<ascribe-tabs sync="…">` containing one `<ascribe-tab value="…" label="…">` per arm | Each arm as a section with a bold label |
| `@variant` group, selection | The arms that survive the selection (§9.3): one arm becomes plain content; several stay a `<ascribe-tabs>` group | One arm becomes plain content; several stay labeled sections |
| `@details` | `<details>` with the title in `<summary>` | The title in bold, then the content |
| `@available`, `badge` | A `<ascribe-availability>` element; page-level availability passed through as frontmatter | A line such as "Available: Quill Cloud (GA); self-managed (preview, 3.4+)" |
| `@available`, `filter` | Unavailable content removed; the rest annotated as in `badge` | Unavailable content removed; the rest annotated as in `badge` |
| Project widget | A custom element with the widget's name and attributes | The widget's plain fallback, or nothing |
| Phrases, includes, links, glossary | Resolved into ordinary markdown | Resolved; links made absolute |

Labels for dimension values come from the content model's display labels.

In the site output, emitters MUST place a blank line after each opening tag and before each closing tag of an element that wraps markdown, so that CommonMark parses the wrapped content as markdown.

**Assets.** Every output is self-contained: it works without access to the source files. Local files a page references (image sources, and link targets that aren't pages) are copied into the output, and references to them are rewritten to point at the copies. A reference resolves from the file it's written in, so an image referenced inside an included fragment is the one beside the fragment (§4.2). The consumer profile decides where copies go and how references to them are written (§9.5), so that a consumer's own image processing still applies.

A reference MUST resolve to a file inside the project root (the directory containing `ascribe.toml`) or the content root, and not inside the output directory; any other reference is treated as a file that doesn't exist (§8.2). File names MUST match exactly, including case, on every platform, so a project checks the same everywhere. References inside raw HTML aren't assets: they pass through unchanged, and the files they name aren't copied.

### 9.5 Consumer profile

The consumer profile, declared in the content model, describes how the site output fits a specific consumer:

- **Routing:** how file paths map to URLs, including any locale prefix.
- **Slugging:** the algorithm the consumer uses for heading ids.
- **Heading ids:** how to emit an explicit id so the consumer keeps its own heading and table-of-contents processing.
- **HTML passthrough:** whether the consumer renders raw HTML in markdown.
- **Images and assets:** how to emit image attributes, where copied assets go, and how references to them are written, so the consumer's image processing still applies.

### 9.6 Astro

Astro is the primary consumer, through its content collections.

- Compiled pages from the site output are loaded into an Astro content collection.
- The collection's schema MUST be generated from the content model, as a Zod schema.
- Page layouts are the project's own.
- Page-level frontmatter, including `available`, reaches the layout as collection data. `available` arrives as a list of its targets, each with what a layout needs to show it: the target, its dimension, states, versions, and display text (element contract §4).

An Astro integration SHOULD provide the collection configuration and generated schema, load the element library, and supply the markdown processing needed for explicit heading ids.

### 9.7 Element library

The element library implements the custom elements used by the site output.

- Elements render into the light DOM, so site styles apply and content stays visible to search engines and assistive technology.
- Elements are styled with CSS and themed through CSS custom properties.
- Only elements that require interaction use JavaScript. Of the built-ins, that's `<ascribe-tabs>`.
- Without JavaScript, `<ascribe-tabs>` displays every arm with its label.

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
- **Document links:** inline link and image destinations as written, and an include's whole primary, are clickable when their target exists. They open the file or heading; external links open their URL after declared destination phrases are replaced.
- **Inline hints:** just inside the `[` of an empty-text link, show the existing source target's title (the heading's text for a source id), with declared phrases replaced and a tooltip naming the destination. Images, missing targets, and targets without a title receive no hint. Hints use source titles without applying build modes.
- **Refactoring:**
  - Renaming or moving a file updates links and includes that point to it.
  - Changing a heading's id updates links to it.
  - Renaming a phrase key updates its uses.
- **Formatting** into canonical form (§8.3), with a formatter that understands Ascribe.
- **Distinct display of title lines**, so a paragraph that accidentally became a title is easy to spot.

Link completion searches pages and the headings of each page's own source file by title, case-insensitively; it does not offer other files' fragments or headings introduced by includes. A page without a title uses its file name. Matches are ranked by title prefix, word prefix, then substring, with paths disambiguating identical titles and pages before headings on a tie. Before any query is typed, offer pages and the current file's own headings. After `#`, offer the named file's source ids, matching its ids and heading titles. Reference-definition destinations are completed like inline link destinations; image-source completion is not required.

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
                                        ; onto following lines like a paragraph (§3.4)

phrase          = "{" key "}"

availability    = entry *( OWS "," OWS entry ) / feature-key
entry           = target [ RWS detail ]
detail          = version / state [ RWS version ] / "(" OWS history OWS ")"
history         = state RWS version *( OWS "," OWS state RWS version )
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
- the phrases `product` (Quill), `cloud` (Quill Cloud), `version` (3.4.1), and `api` (`https://api.quill.dev/v3/`).

````markdown
---
title: Install the Quill agent
description: Install and configure the Quill agent to sync your docs to Quill Cloud or a self-managed Quill server.
available: cloud, self-managed preview 3.3
---

The {product} agent watches your docs repository and syncs changes to {product}. This page covers installing the agent with a package manager, configuring it, and connecting it to {cloud} or a self-managed server. If you only want to try {product}, see [Try {product} in the browser](quickstart.md#try-in-browser).

.Try it without installing
@note {type=tip}
You can run {product} in the browser at play.quill.dev with no local setup.

## Prerequisites
@id: prerequisites

@include: _fragments/prerequisites.md

## Install the agent
@id: install-agent

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
@id: connect

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
