---
type: reference
title: Diagnostics
description: Every problem Ascribe reports, with its code and fix.
permalink: diagnostics
---

Every problem Ascribe reports, with its code, its name, and how to fix it. `ascribe check`, `ascribe build`, and the editor report the same diagnostics, with the same codes.

A *problem* is what Ascribe found in your project, at one place. A *diagnostic* is the kind of problem, with a code, that this page and `ascribe explain` describe. An error, a warning, and advice are how severe a problem is.

- An **error** makes `ascribe check` fail, and stops `ascribe build` from writing anything. A **warning** doesn't, unless you pass `--deny-warnings`. **Advice** never does, even with `--deny-warnings`: `ascribe check` lists it after the errors and warnings and counts it apart, and the editor shows it as information.
- A **file-level** diagnostic is about one file on its own. A **page-level** diagnostic is about a page after its includes are expanded and a build's modes are applied, so it can depend on the [build](../glossary.md#build); the message names the builds it appears in.
- A diagnostic about `ascribe.toml` (a name that starts with `model-`) stops everything else when it's an error: every other check depends on the content model.
- In the messages below, `{name}` stands for a value filled in from your source.

The diagnostics are on four pages, by what they're about:

- [Directives](diagnostics/directives.md): attributes, directives and their containers, titles, ids, includes, variants, availability, and widgets.
- [Links, pages, and sources](diagnostics/pages.md): links, images, phrases, headings, frontmatter, lists, files, snippets, source copies, acknowledgements, and permalinks.
- [The content model](diagnostics/content-model.md): `ascribe.toml`.
- [Content checks](diagnostics/content-checks.md): the content's quality, which a project can set the level of.

Each entry ends with the section of the [directive reference](../writing/directives.md) or the [`ascribe.toml` reference](../content-model/index.md) about what it checks, which has examples, and links to its part of the specification.

In the editor, many diagnostics offer a quick fix. See [Editing](../editor/index.md).

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

A check about the content's quality, rather than whether the project is valid, can be set to another level, or turned off, in [`[checks]`](../content-model/checks.md#19-checks) in `ascribe.toml`. Those checks are listed under [Content checks](diagnostics/content-checks.md), and their entries say they're configurable. Every other diagnostic is about whether the project is valid, which a project can't lower or turn off.

## Index

@include: ../_generated/diagnostics-index.md
