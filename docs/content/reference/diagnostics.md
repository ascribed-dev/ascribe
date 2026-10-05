---
title: Diagnostics
description: Every problem Ascribe reports, with its code and fix.
---

Every problem Ascribe reports, with its code, its name, and how to fix it. `ascribe check`, `ascribe build`, and the editor report the same diagnostics, with the same codes.

- An **error** makes `ascribe check` fail, and stops `ascribe build` from writing anything. A **warning** doesn't, unless you pass `--deny-warnings`.
- A **file-level** diagnostic is about one file on its own. A **page-level** diagnostic is about a page after its includes are expanded and a build's modes are applied, so it can depend on the build; the message names the builds it appears in.
- A diagnostic about `ascribe.toml` (a name that starts with `model-`) stops everything else when it's an error: every other check depends on the content model.
- In the messages below, `{name}` stands for a value filled in from your source.

In the editor, many diagnostics offer a quick fix. See [Editing](../guides/editor.md).

## Source files

@include: ../_generated/diagnostics-source-files.md

## The content model

@include: ../_generated/diagnostics-content-model.md

@include: ../_generated/diagnostics-retired.md
