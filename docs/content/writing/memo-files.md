---
type: reference
title: Memo files
description: "A memo longer than a line or two, in a file of its own in `memos/`: what it's about, where it's placed, and what `ascribe check` reports."
available: ascribe 0.4.0
---

A memo says why a page is written the way it is, for whoever changes it next. One a line or two long goes in the page, as [`@memo`](directives.md#memo). A longer one goes in a file of its own, a **memo file**, in the folder `memos/` beside `ascribe.toml`. Neither is ever published. This page describes memo files, and [Leave a memo for whoever edits next](memos.md) says when to write one; the [Ascribe specification]({repo}/blob/main/SPEC.md#77-memo-files) is the normative definition.

## Write a memo file

A memo file is Markdown with two frontmatter keys, then the memo:

```markdown
---
about: install-agent.md#install-agent
quote: Install the agent package
---

The agent goes first because its installer writes the config file that
`quill init` reads. The other way round, `init` fails with an error that
doesn't mention the agent.
```

- **`about`**, required, names what the memo is about, the way [`ascribe refs`](../command/questions.md#cli-refs) names a target: a page or fragment by its path from the content root (`install-agent.md`), a heading as the path, `#`, and its id (`install-agent.md#install-agent`), a glossary term by its id (`term:api-key`), or an entry of `ascribe.toml` as `phrase:product`, `feature:sso`, `dimension:deployment`, `note:warning`, or `widget:release-note`. A heading in a fragment is named in the fragment. A term is a heading on a glossary page, so a memo about `term:api-key` is on that heading, as one about `glossary.md#api-key` is.
- **`quote`**, which you can leave out, narrows the memo to the block that holds those words: a paragraph, a heading, a list item's paragraph, a code block, or a table, for instance. An entry of `ascribe.toml` has no words to quote; a term's are those of its section.
- **The text** after the frontmatter is the memo, as you write it. Nothing in it is read: a link isn't checked, and a `{key}` isn't a phrase.

The files are every `.md` file in `memos/` and its subfolders. A file's name and its folder mean nothing to Ascribe, so name and arrange them as your team likes; a name or folder that starts with `.` is skipped. A memo is about one thing, with no author, no date, and nothing to finish: a reason, kept for as long as it holds.

`memos/` is outside the content root, so its files aren't pages. In a project whose content root is the folder `ascribe.toml` is in, it can't be: set `[project] content-root` to a folder below it, such as `"docs"`, and move the pages there. A [personal memo](memo-files.md#personal-memos) doesn't need that.

### Personal memos

A memo that's only for you goes in `.ascribe/memos/` instead, in the same format. It's a **personal memo**: `.ascribe/` is the folder the project doesn't commit ([project files](../command/project-files.md)), so it stays on your machine. The editor, `ascribe outline`, and `ascribe refs` read both folders, and mark a personal memo as one. `ascribe check` reads `memos/` only, so what it reports is the same on every machine; the editor reports a personal memo's problems as it reports a team memo's.

A memo is read by someone about to change the page, so it's short: over 2,000 characters, `ascribe check` reports it as advice, [`memo-long`](../checks/diagnostics/content-checks.md#ASC228). `[checks.memo-long] limit` sets another limit, in characters.

## Placed, outdated, and detached

Where a memo is is worked out each time it's read, from the pages as they are, and nothing is written back to the file. A memo is in one of three states:

- **Placed.** With no `quote`, it's on what `about` names. With one, it's on the first block of that page or section, in the order they're written, whose source holds the quote. Line breaks and runs of spaces count as one space, and case counts. A quote is found in one block: words that run across two paragraphs aren't.
- **Outdated.** No block holds the quote: the words were changed since the memo was written. The memo stays on its page or section. It's advice, [`memo-outdated`](../checks/diagnostics/content-checks.md#ASC227): read the memo against the page, and quote the words it's about now, or change or delete it.
- **Detached.** `about` names nothing: the page was deleted, or it moved or the heading's id changed by hand rather than by a rename. It's a warning, [`memo-detached`](../checks/diagnostics/pages.md#ASC226), which names the ids the page has when the heading is what's missing. Point `about` at what the memo is about now.

A rename keeps a memo attached: [`ascribe rename`](../command/questions.md#cli-rename), and in the editor, `F2` on a heading's `@id` or text, on a phrase, and moving a file in the Explorer, rewrite the `about` of each memo that names what changes, in both folders: the page, keeping its `#id` when the page moves; the heading, written on its own page or on a page that includes the fragment it's in; a glossary term, `term:<id>`, when its heading's id changes; and a phrase, `phrase:<key>`. The value is rewritten where it's written, in quotes, as a block scalar, or on the line under `about:`; one written in a form that can't be found in the text, such as with an escape, is written again on `about`'s line. Its `quote` and its text are left as they are.

A memo file that isn't one, with no frontmatter, no `about`, an `about` that isn't a target, or a key besides `about` and `quote`, is an error, [`memo-invalid`](../checks/diagnostics/pages.md#ASC225). `ascribe check` reports each problem on the memo file, at its `about` or `quote` line.

## Where memos are listed

In `examples/quill`, the memo above is about the section of `install-agent.md` that installs the agent:

@snippet {lang=text}: code:crates/ascribe-cli/tests/output/outline-memos.txt

@snippet {lang=text}: code:crates/ascribe-cli/tests/output/refs-memo.txt

- **`ascribe outline`** lists the memos about a page under it, and those about a section or a block in it under its heading, with each one's first line. An outdated memo says so, a personal one is listed as a personal memo, and a detached one isn't listed. With `--format json`, the page and each heading have `memos`, each with `personal` ([`ascribe outline`](../command/questions.md#cli-outline)). The MCP tool `ascribe_outline` returns the same.
- **`ascribe refs`** lists each memo file about its target as a place, with the use `memo`, at the memo's `about` line, and `personal` true for a personal memo. A detached memo is listed under the target it names, so `ascribe refs` finds the memos about a page or heading that's gone.

- **In VS Code**, a CodeLens above a heading or a block counts its memos and opens them, **This page** lists the memos about the heading at the cursor, and **Ascribe: Add a memo** writes one from the cursor, with `about` and `quote` filled in ([Memo files in the editor](../editor/actions.md#memo-files)).

- **Agents' prompts and rules** name the memos about the page they're about, or about what changed, a line each with the memo's path, and never its text. The rules say where to find a page's memos ([Memos in a prompt](../agents/prompts.md#memos-in-a-prompt)).

No build, preview, or report reads a memo file, and a change to one alone isn't a change to `ascribe diff`.
