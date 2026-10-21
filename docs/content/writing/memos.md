---
type: how-to
title: Leave a memo for whoever edits next
description: Keeping the reason a page is written the way it is beside it, in the page or in a memo file, for the people and agents who change it, and never for its readers.
available: ascribe 0.4.0
---

A page's reasons are easy to lose. Why the steps are in this order, why that table isn't prose, what the subject expert said about the caveat: none of it is for readers, so it ends up in a wiki, in a pull request nobody finds again, or in someone's head. The next person to edit the page, or the agent you ask to, changes it without knowing.

A **memo** keeps the reason beside what it's about, where the next editor will meet it, and is never published. This guide says which kind of commentary to write, and how to write each. It's for a writer who has a project already, as [Getting started](../getting-started.md) sets one up.

## Choose where it goes

Ascribe keeps three kinds of commentary apart, and each has its place:

| Write | When | Where it lives |
|---|---|---|
| A memo in the page, `@memo` | The reason is a line or two, and is about a section or a block of this page | In the page, above what it's about |
| A memo file | The reason is longer, or about a whole page, a phrase, or another entry of `ascribe.toml` | A file of its own in `memos/`, beside `ascribe.toml` |
| A pull request comment | It's a question, a suggestion, or anything that wants an answer | In [review](../review/index.md), with the change |

A memo has no replies and nothing to resolve: it's a reason, kept for as long as it holds. When you're discussing a change, discuss it in review; when the discussion settles why the page is the way it is, keep that as a memo.

An HTML comment, `<!-- … -->`, isn't a place for any of these: [`html-comment`](../checks/diagnostics/content-checks.md#ASC223) says why, and how to turn one into a memo.

## Write a memo in the page

Write `@memo:` and the reason, on the line above what it's about. At the top of a section, a memo is about the section; anywhere else, about the block below it:

```markdown
## Rotate a key
@memo: Rotation comes before revocation: people who revoke first lock out their own agents.

Generate a new key, then restart the agent.

@memo: Kept to one sentence; the support macro quotes it.
Revoke the old key once every agent uses the new one.
```

For a reason that takes a few lines, write the container form, closed with `@end`:

```markdown
@memo:
Support found that people who revoke first lock out their own
agents. See [the incident](notes/2026-08.md).
@end
Revoke the old key once every agent uses the new one.
```

The [`@memo` reference](directives.md#memo) has every rule, among them what reads a memo and what it changes.

## Write a memo file

When the reason is longer than a few lines, or it's about something with no line to stand above, write a memo file: a Markdown file in `memos/` that names what it's about in its frontmatter.

@steps
1. Make the folder `memos/` beside `ascribe.toml`, if the project doesn't have one. It can't be inside the content root, so a project whose pages are beside `ascribe.toml` moves them to a folder of their own first.
2. Make a file in it, with any name that ends in `.md`, and say what the memo is about with `about`: a page, as its path from the content root, or a heading, as `page.md#id`.

   ```markdown
   ---
   about: install-agent.md#install-agent
   quote: Install the agent package
   ---

   The agent goes first because its installer writes the config file that
   `quill init` reads.
   ```

3. To narrow the memo to one block of that section, add `quote`, with words from the block as they're written in the source.
4. Write the reason after the frontmatter, as long as it needs to be.
5. Run `ascribe check`. It reports a memo file that names nothing, or whose quote it can't find.

In VS Code, **Ascribe: Add a memo** does the first three steps for you, from the cursor and the text you've selected: see [Memo files in the editor](../editor/actions.md#memo-files).

When `ascribe check` later reports a memo file as outdated or detached, read it against the page, and see [Placed, outdated, and detached](memo-files.md#placed-outdated-and-detached) for what to change. The [memo files reference](memo-files.md) has the format and every rule.

## Keep a memo to yourself

A memo for you alone, such as what you were told and haven't confirmed, goes in `.ascribe/memos/` instead of `memos/`, in the same format, and stays on your machine. **Ascribe: Add a personal memo** writes one. [Personal memos](memo-files.md#personal-memos) says what reads it.

An agent reads a memo before it changes the page: [Agent instructions](../agents/instructions.md) and [Memos in a prompt](../agents/prompts.md#memos-in-a-prompt) say how it's told.

## See also

- [`@memo`](directives.md#memo): the directive, in the directive reference.
- [Memo files](memo-files.md): the format, the states, and the checks.
- [Memo files in the editor](../editor/actions.md#memo-files): the CodeLens, adding a memo, and the **Memos** section of **This page**.
- [`ascribe outline`](../command/questions.md#cli-outline) and [`ascribe refs`](../command/questions.md#cli-refs), which list the memos about a page and a target.
