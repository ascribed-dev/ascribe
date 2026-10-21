---
type: reference
title: Problems and fixes
description: "The problems the VS Code extension shows as you type, their quick fixes, and applying the safe ones."
---

What the editor reports, and the fixes it offers for it.

## Diagnostics

Every [diagnostic](../checks/diagnostics.md) appears as you type, including in unsaved files, for every file of the project, open or not: fix a heading's id, and the broken links to it in other files clear. Page-level diagnostics are for the [build](../glossary.md#build) named by `[editor] build` in `ascribe.toml`.

When that build is [versioned](../versions/index.md), they're for every line of it, and a problem only some lines have says which, such as "(only in build `site@3.4`)".

The [content checks](../checks/diagnostics/content-checks.md) about the whole project, such as a page nothing links to or a phrase nothing uses, need every page, so they run when you save, not as you type. Until you save a file you've changed, it doesn't show theirs. The image checks run only in `ascribe check`.

In a workspace with several projects, the Problems panel lists only the projects whose language server is running. See [The Problems panel](settings.md#the-problems-panel).

## Quick fixes

Many diagnostics offer a fix (the light bulb, or `Ctrl+.` / `Cmd+.`):

- **Link to the page's file instead of its route**, for a link written as a URL.
- **Quote the attribute value**, for a value that needs quotes.
- **Add a trailing colon** or **Remove the stray colon**, for a container written the wrong way.
- **Remove the blank line before the bound block**.
- **Declare phrase in ascribe.toml**, or **Escape this phrase as literal text**, for an undeclared `{key}`.
- **Add a stable @id for this heading**, for a heading whose id could change.
- Did-you-mean corrections for misspelled directives, attributes, and frontmatter keys.
- **Remove** text on a directive line that isn't part of the directive.

@available: ascribe 0.4.0
- **Rewrite as a memo**, for an [HTML comment](../writing/directives.md#memo) that's a block of its own. One line becomes a line memo and more a container, about the block below it, or at the top of the section when the comment is the section's last block. A comment at the end of a list item, or before a page's first heading, gets no fix, since where its memo belongs is yours to say.

@available: ascribe 0.4.0
- **List `page.md` in `[glossary] pages`**, for a [glossary term](../content-model/content.md#12-glossary) marked on a page that isn't a glossary page. Every term on the page is then linked on every other page, so it's yours to choose.
- **Write a definition of** the term, for a term whose section doesn't open with a paragraph. It puts a paragraph under the heading and its directives, selected, for you to type the definition over.

- **Copy the command**, for an [example](../examples/tested.md) that changed after its last test run or that no test run lists. It copies the line that runs `ascribe observe` in the example's folder, for you to run in a terminal: for PowerShell on Windows, and for `sh` elsewhere. The editor never runs a test itself.

## Apply the safe fixes
@available: ascribe 0.4.0

**Source Action… → Apply the safe fixes in this page** applies, in one edit, every fix marked safe for the problems the editor shows in a page or a fragment, such as linking to a page's file instead of its route, or removing an empty memo, and leaves the rest. It takes them as [`ascribe check --fix`](../checks/check.md#applying-the-safe-fixes) does: two that would edit the same text are one too many, so the first is applied and the other's problem stays, for the next time. The actions bar lists it first among its fixes when the page has two or more. A problem that takes every build to tell, such as an acknowledgement that covers nothing, isn't one the editor shows, so `ascribe check --fix` is where its fix is applied.

To apply them each time you save a page, set `editor.codeActionsOnSave` for Markdown in your settings. Nothing sets it for you:

```json
"[markdown]": {
  "editor.codeActionsOnSave": { "source.fixAll.ascribe": "explicit" }
}
```
