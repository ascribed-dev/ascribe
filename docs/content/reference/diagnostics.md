---
title: Diagnostics
description: Every problem Ascribe reports, with its code and fix.
---

Every problem Ascribe reports, with its code, its name, and how to fix it. `ascribe check`, `ascribe build`, and the editor report the same diagnostics, with the same codes.

- An **error** makes `ascribe check` fail, and stops `ascribe build` from writing anything. A **warning** doesn't, unless you pass `--deny-warnings`. **Advice** never does, even with `--deny-warnings`: `ascribe check` lists it after the errors and warnings and counts it apart, and the editor shows it as information.
- A **file-level** diagnostic is about one file on its own. A **page-level** diagnostic is about a page after its includes are expanded and a build's modes are applied, so it can depend on the [build](content-model.md#16-buildsname); the message names the builds it appears in.
- A diagnostic about `ascribe.toml` (a name that starts with `model-`) stops everything else when it's an error: every other check depends on the content model.
- In the messages below, `{name}` stands for a value filled in from your source.

In the editor, many diagnostics offer a quick fix. See [Editing](../guides/editor.md).

## What to do next

Each diagnostic says what kind of next step it has. `ascribe check --format json` gives it as `next`, and the editor in each diagnostic's `data`, so a tool or an agent can offer the right thing.

| Next step | What it means |
|---|---|
| `fix` | Ascribe can make the edit: every instance has a quick fix in the editor. |
| `choose` | You pick among things Ascribe can list: a link's target, a key's allowed values, a declared phrase. |
| `write` | It needs writing or judgment. The entry's fix says what to do; an agent can do it from the diagnostic's prompt. |
| `outside` | Nothing in the source can fix it; it's set somewhere else, such as the site's hosting. |
| `review` | It may be fine as it is. |

## Setting a check's level

A check about the content's quality, rather than whether the project is valid, can be set to another level, or turned off, in [`[checks]`](content-model.md#19-checks) in `ascribe.toml`. Its entry below says it's configurable. The checks of the content's quality are listed under [Content checks](#content-checks); they aren't part of the language's own list of diagnostics (SPEC §8.2). Every other diagnostic is about whether the project is valid, which a project can't lower or turn off.

## Index

@include: ../_generated/diagnostics-index.md

## Source files

@include: ../_generated/diagnostics-source-files.md

## Content checks

Checks of the content's quality, rather than whether the project is valid. Each is configurable in `[checks]`, and its severity is its level when `[checks]` doesn't set one.

@include: ../_generated/diagnostics-content-checks.md

## The content model

@include: ../_generated/diagnostics-content-model.md

@include: ../_generated/diagnostics-retired.md
