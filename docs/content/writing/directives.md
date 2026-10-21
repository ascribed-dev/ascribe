---
type: reference
title: Directive reference
description: "Each built-in directive: what it does, its attributes, and its rules; and project widgets."
permalink: directives
---

Each built-in directive, in the order of the [Ascribe specification]({repo}/blob/main/SPEC.md#4-built-in-directives), the normative definition, whose part each section links to. How a directive is written, and what it applies to, is in [directive syntax](syntax.md).

@include: ../_generated/directives-builtins.md

## `@id`

([SPEC §4.1]({repo}/blob/main/SPEC.md#41-id))

Gives a heading an explicit, stable id, used for its anchor and by links and includes.

```markdown
## Configuration
@id: config-setup
```

- It goes directly under its heading.
- An id uses letters, digits, hyphens, underscores, and periods, so an existing anchor such as `ece_setup` or `v1.2` can be kept.
- Ids must be unique on a page, including ids that come from included fragments.

Without `@id`, a heading's id is the slug of its text ([heading ids](inline.md#heading-ids)).

### Permalinks

`{permalink=true}` makes the id a **permalink** as well: a name for the section that something outside the docs can link to, at an address of its own, `go/<id>` under the site's base path or [`permalink-path`](../content-model/output.md#consumer), which forwards to the section wherever it is. [Permalinks](../permalinks/index.md) says how a product links to one, how its CI checks those links, and what to do when a section moves or goes away.

```markdown
## `ascribe check`
@id {permalink=true}: cli-check
```

- **The id is still the heading's anchor,** and links inside the docs name it as they name any id: `checks/check.md#cli-check`. A permalink is for links from outside the docs; inside, link to the file.
- **A build publishes a permalink in one place only** (`permalink-duplicate`), and names that differ only in case, such as `Setup` and `setup`, are one name. So a marked heading belongs in a fragment only one page of each build includes. Two pages that no build publishes together may share a permalink.
- **A permalink that's removed is a warning** (`permalink-removed`) once the project commits [its list](../contracts/permalinks-list.md); restore the name or [retire it](../permalinks/index.md#moving-aliasing-and-retiring). Moving the heading to another page isn't removing it.
- **A page has a permalink** in its frontmatter, from the same set of names: [a page's permalink](frontmatter.md#a-pages-permalink).

## `@include`

([SPEC §4.2]({repo}/blob/main/SPEC.md#42-include))

Includes another file, or one heading's section of it, in place.

```markdown
@include: _snippets/prerequisites.md
@include {heading=false}: guides/setup.md#install
```

- **Paths** are relative to the file they're written in, or to the content root when they start with `/`. Only source files (`.md` files under the content root) can be included. A symbolic link is followed only to another source file, so no other file on disk ends up in a page.
- `#id` includes only the section of the heading with that id. `{heading=false}` leaves out that section's own heading; it has no effect without `#id`, and Ascribe warns.
- Included content becomes part of the page: phrases and [builds](../glossary.md#build) apply to it, and its ids must not collide with the page's.
- **Relative paths inside a fragment resolve from the fragment**, not the page including it, so a fragment's links and images work wherever it's included.
- Included headings keep the levels they're written with.
- An include cycle is an error.

A page's includes can expand it only so far, far beyond any real page: the include in the page's own file that takes it past the limit is an error, `include-limit`, whose message gives the limit, and it and every include after it on the page are left out.

A link names the page that includes a fragment, never the fragment, and can name a heading the fragment brings in ([links](inline.md#links)).

## `@variant`

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
- `@variant` varies whole blocks. For a single term that differs, use a [phrase](../glossary.md#phrase).

Each build decides what happens to a group: `switch` keeps every arm, for readers to switch between (the site shows tabs), and a selection such as `{ deployment = "cloud" }` keeps only the matching arms. See [builds](../glossary.md#build). A whole page that differs by dimension is a page for each, with [`variant`](frontmatter.md#whole-page-variants) in its frontmatter.

## `@available`

([SPEC §4.4]({repo}/blob/main/SPEC.md#44-available))

Says where content applies and its lifecycle state. Unlike `@variant`, it doesn't select content: every reader sees it, marked with its availability.

```markdown
@available: cloud, self-managed preview 3.4
```

- At the top of a section, it applies to the section, and anywhere else to the block it touches ([binding](syntax.md#binding-what-a-directive-applies-to)).
- For a whole page, use the frontmatter key [`available`](frontmatter.md#reserved-keys), with the same spec.
- Several `@available` lines on one heading or block all apply: the content is available only where all of them allow it.

### Availability specs

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

### Table rows

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

## `@note`

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

## `@steps`

([SPEC §4.6]({repo}/blob/main/SPEC.md#46-steps))

Marks an ordered list as a procedure. It goes directly above the list.

```markdown
@steps
1. Install the agent package.
2. Verify the install.
3. Create `quill.yaml`.
```

Directives inside the list's items are indented to the item's content ([lists](syntax.md#lists-and-block-quotes)).

## `@details`

([SPEC §4.7]({repo}/blob/main/SPEC.md#47-details))

Content readers can expand or collapse. The title is required: it's what readers see while the content is collapsed.

```markdown
.Show the full configuration reference
@details:
…
@end
```

Without a colon, `@details` applies to the next block; with one, it's a container.

## `@snippet`

([SPEC §4.8]({repo}/blob/main/SPEC.md#48-snippet))

Takes a code example from a file outside the content, and puts it in the page as a fenced code block. There's no copy: Ascribe reads the file whenever it checks, builds, or diffs, so changing the file changes every page that uses it.

```markdown
@snippet: code:examples/quill/ascribe.toml#dimensions
@snippet {lang=shell, title="Install"}: code:scripts/install.sh
```

- **The address** is `<source>:<path>`, optionally followed by `#<region>`. `<source>` is a source declared in `ascribe.toml` ([`[sources.<name>]`](../content-model/output.md#18-sourcesname)), and `<path>` is the file's path relative to the source's folder (for a [source in another repository](../content-model/output.md#a-source-in-another-repository), its path in that repository), with `/` on every platform. There's no relative form: an address doesn't depend on where the page is.
- The file must exist with exactly that name, the source's `include` and `ignore` must take it in, and it must be text. A symbolic link is followed, but only to a file in the source's folder that `include` and `ignore` take in.
- Without `#<region>`, the snippet is the whole file.
- **Attributes:** `lang` sets the code block's language, which is otherwise the file's extension (`toml` for `ascribe.toml`); `title` gives the code block a title; `phrases=true` substitutes phrases in the code, as in a fence that opts in ([phrases](inline.md#phrases)).
- It's a block, allowed wherever a code block is: in a list item, a note, or a variant's arm. A `@note`, `@details`, or widget directly above it applies to it.

A region is marked in the code with [Bluehawk](https://github.com/mongodb-university/Bluehawk)'s tags in comments, `:snippet-start: <name>` and `:snippet-end:`: [mark a region in the code](../examples/snippets.md#2-mark-a-region-in-the-code) has the tags and the comments they're read in.

Every problem is reported at the `@snippet` line ([source files](../checks/diagnostics/directives.md), in the diagnostics reference).

@available: ascribe 0.4.0
In VS Code, a code file says which pages show it: see [code files a page shows](../editor/writing.md#code-files-a-page-shows).

## `@intended`

([SPEC §4.9]({repo}/blob/main/SPEC.md#49-intended))

Says that a problem a check reports is intended: you've looked at it and are keeping it. Write the check's name and why.

```markdown
@intended {check=link-dead}: help.example.com refuses link checkers; checked by hand.
Read the [vendor's deployment notes](https://help.example.com/deploy) first.
```

- **Only review checks,** whose next step is *review*, since what they report may be fine as it is, such as a page nothing links to. A problem that's always wrong is fixed instead. The [diagnostics reference](../checks/diagnostics.md) says where each review check is acknowledged.
- **Where it's written** depends on the check, which reports its problems at one kind of place:
  - **A block:** `@intended` directly above it, as above. It covers the check's problems anywhere in the block, including inside a container. Above a block of a fragment, it covers that block in every page that includes it.
  - **A heading:** `@intended` at the top of its section, among the directives under the heading. It covers the check's problems at the heading, and, when no blank line comes between, in the block directly under the `@intended` line, so a section's first block is acknowledged as any other block is. It covers nothing else in the section. An unused glossary term is acknowledged this way:

    ```markdown
    ## Workspace
    @term
    @intended {check=glossary-term-unused}: Linked from the product's own screens.

    The space a team's projects are in.
    ```
  - **A page or a fragment:** the [`intended`](frontmatter.md#reserved-keys) frontmatter key, a list of entries with a `check` and a `reason`:

    ```yaml
    intended:
      - check: page-orphan
        reason: Linked from the site's sidebar, which Ascribe can't see.
    ```

  - **A content model entry or an image:** an [`[[intended]]` table](../content-model/checks.md#intended-acknowledgements) in `ascribe.toml`.
- **The reason is required,** and is the rest of the line: `ascribe check` lists it with the problem it covers.
- **It changes no output.** Nothing renders for `@intended`, and the `intended` key isn't in any output's frontmatter.

An acknowledged problem isn't reported, but `ascribe check` and `ascribe build` count it in their summary (`1 acknowledged`), and list it, with its reason, in their [JSON report](../command/index.md). When an acknowledgement covers nothing in any build, because the problem was fixed or the check changed, `ascribe check` reports it as advice, `intended-unused`, with a fix that removes it. That's reported only when every build is checked: not with `--build` or `--editor-build`, and not in the editor.

In the editor, a review check's problem has a quick fix, **Mark as intended, and write why**, that writes the acknowledgement in the right place for you to fill in the reason. Its placeholder reason is reported as an error until you replace it, so the fix alone silences nothing.

## `@memo`
@available: ascribe 0.4.0

([SPEC §4.10]({repo}/blob/main/SPEC.md#410-memo))

Keeps a note for the people and agents who maintain the page, such as why a section comes first, beside what it's about. It's never published. [Leave a memo for whoever edits next](memos.md) says when to write one, when a [memo file](memo-files.md) is better, and when a pull request comment is.

```markdown
## Rotate a key
@memo:
Rotation comes before revocation: support found that people who revoke
first lock out their own agents. See [the incident](notes/2026-08.md).
@end

Generate a new key, then restart the agent.

@memo: Kept to one sentence; the support macro quotes it.
Revoke the old key once every agent uses the new one.
```

- **What it's about** is decided as for `@available` ([binding](syntax.md#binding-what-a-directive-applies-to)). A container memo touches the block that starts on the line after its `@end`. Several memos stack, in either form, and `ascribe fmt` removes a blank line between a memo and its block.
- **A line memo** is the rest of its line, and needs some text. **A container memo** is every line up to a line that's only `@end` at the memo's own indent, as written. It takes no attributes. A container memo with nothing in it, or only blank lines, is almost always a slip: `ascribe check` reports it as advice, [`memo-empty`](../checks/diagnostics/content-checks.md#ASC224), whose fix removes it.
- **Nothing reads a memo,** nor a [memo file](memo-files.md)'s text. A link in one isn't checked and isn't a use of the page it names, a `{key}` isn't a phrase, an `@include` isn't followed, a `#` line isn't a heading, and Vale isn't given the text. A code fence that doesn't close, or a line such as `@note:`, doesn't change the page around it. The one line a container memo can't hold is the `@end` that closes it.
- **It changes no output.** No build, preview, or report shows a memo, in a page, a fragment, or a memo file, and a change to one alone isn't a change to [`ascribe diff`](../review/diff.md) or to review.
- **An HTML comment isn't a memo:** [`html-comment`](../checks/diagnostics/content-checks.md#ASC223) says why, and how to rewrite one as a memo.

## `@term`
@available: ascribe 0.4.0

([SPEC §4.11]({repo}/blob/main/SPEC.md#411-term))

Makes a heading of a glossary page a glossary term. The heading's text is the term, and the paragraph under it is its definition, which a built page shows where it links the term, and the editor on hover.

```markdown
## Rollout
@id: rollout
@term {aliases=rollouts}

A plan for turning a flag on for a growing share of users: by percentage, by segment, or on a schedule.
```

- It goes directly under the heading, with the heading's `@id` if it has one. It takes no primary.
- **Attributes:** `aliases`, other forms that count as the term, separated by `|` (`aliases="API keys|keys"`); `match`, which occurrences are linked, `first`, `every`, or `marked`; and `case-sensitive`, `true` or `false`. The last two default to `[glossary]`'s settings.
- A term is only a term on a page `[glossary] pages` lists, or in a fragment such a page includes. Anywhere else, `@term` is an error, `term-off-glossary-page`, whose fix lists the page in `pages`. See [Glossary terms](inline.md#glossary-terms).
- The paragraph under the heading must be the first thing in the section, after the heading's directives: a section that opens with anything else has no definition, which is an error, `glossary-definition-missing`. Keep it to a sentence or two, since it's shown where there's no room for more: one over 300 characters is reported as advice, `glossary-definition-long`.
- It renders nothing: the heading and its section are published as they would be without it.

To set up a glossary from the start, see [Set up a glossary](../content-model/glossary.md).

## Project widgets

([SPEC §6]({repo}/blob/main/SPEC.md#6-project-widgets))

A project widget is a directive a project declares in `ascribe.toml`, for needs the built-in directives don't cover. Its name contains a hyphen, so it's easy to tell from a built-in:

```markdown
@quill-labspace {lab=first-sync}
```

A declared widget works like a built-in directive, with the forms, primary, attributes, binding, title, and grouping its declaration sets. In the site output it becomes a custom element with its name; in plain Markdown, its declared fallback text. See [`[widgets.<name>]`](../content-model/content.md#14-widgetsname).

