---
type: reference
title: Diagnostics
description: Every problem Ascribe reports, with its code and fix.
permalink: diagnostics
---

Every problem Ascribe reports, with its code, its name, and how to fix it. `ascribe check`, `ascribe build`, and the editor report the same diagnostics, with the same codes.

A *problem* is what Ascribe found in your project, at one place. A *diagnostic* is the kind of problem, with a code, that this page and `ascribe explain` describe. In the messages below, `{name}` stands for a value filled in from your source.

The diagnostics are on six pages, by what they're about:

- [Directives](diagnostics/directives.md): attributes, directives and their containers, binding, titles, ids, includes, variants, availability, steps, details, and widgets.
- [Links, pages, and sources](diagnostics/pages.md): links, images, phrases, headings, frontmatter, lists, files, snippets, source copies, acknowledgements, permalinks, memo files, navigation, and the glossary.
- [The addresses file and API specs](diagnostics/project-files.md): `ascribe.addresses`, the addresses a base published, and the specs `[apis]` declares.
- [The content model](diagnostics/content-model.md): `ascribe.toml`: the file, `[project]`, `[sources]`, content types, names, `[releases]`, phrases, and widgets.
- [The content model's builds and site](diagnostics/content-model-builds.md): the rest of `ascribe.toml`: the consumer, builds, the editor, `[permalinks]`, `[crawlers]`, `[listing]`, and `[checks]`.
- [Content checks](diagnostics/content-checks.md): the content's quality, which a project can set the level of: the checks of pages and of the project as a whole.
- [Content checks of outputs and the site](diagnostics/output-checks.md): the rest of the content checks: outputs, prose through Vale, external links, the published site, and tested examples.

Each entry ends with the section of the [directive reference](../writing/directives.md), [directive syntax](../writing/syntax.md), [frontmatter](../writing/frontmatter.md), [phrases, links, and images](../writing/inline.md), or the [`ascribe.toml` reference](../content-model/index.md) about what it checks, which has examples, and links to its part of the specification.

In the editor, many diagnostics offer a quick fix. See [Quick fixes](../editor/problems.md#quick-fixes).

Each entry opens with a line such as "Advice · page level · next step: write · configurable in `[checks]`": the diagnostic's [severity](#severity), its [level](#file-level-and-page-level), its [next step](#what-to-do-next), whether a project can [set its level](#setting-a-checks-level), and, for a check whose problems can be [acknowledged](../glossary.md#acknowledgement), where. It ends with the diagnostic's section of the specification.

## Severity

An error, a warning, and advice are how severe a problem is.

- An **error** makes `ascribe check` fail, and stops `ascribe build` from writing anything. A diagnostic about `ascribe.toml` (a name that starts with `model-`) stops everything else when it's an error: every other check depends on the content model.
- A **warning** doesn't, unless you pass `--deny-warnings`.
- **Advice** never does, even with `--deny-warnings`: `ascribe check` lists it after the errors and warnings and counts it apart, and the editor shows it as information.

A check that's **off by default** reports nothing until a project names it in `[checks]`: see [Setting a check's level](#setting-a-checks-level).

## File level and page level

A **file-level** diagnostic is about one file on its own. A **page-level** diagnostic is about a page after its includes are expanded and a build's modes are applied, so it can depend on the [build](../glossary.md#build); the message names the builds it appears in.

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

A check about the content's quality, rather than whether the project is valid, can be set to another level, or turned off, in [`[checks]`](../content-model/checks.md#19-checks) in `ascribe.toml`. Those checks are listed under [Content checks](diagnostics/content-checks.md) and [Content checks of outputs and the site](diagnostics/output-checks.md), and their entries say they're configurable. Every other diagnostic is about whether the project is valid, which a project can't lower or turn off.

A few content checks matter only to some projects, and are off unless `[checks]` names them; their entries say they're off by default. Name one with a level to turn it on at that level, or with a table of its settings to turn it on at the level its entry gives. [`summary-missing`](diagnostics/content-checks.md#ASC252) and [`summary-long`](diagnostics/content-checks.md#ASC253), for a project that [shows its docs in a product](../permalinks/in-your-product.md), are turned on like this:

```toml
[checks]
summary-missing = "advice"
summary-long = "advice"
```

## Index

@include: ../_generated/diagnostics-index.md
