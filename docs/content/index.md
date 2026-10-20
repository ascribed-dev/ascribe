---
type: index
title: Ascribe documentation
description: Guides and references for writing documentation with Ascribe, and the contracts for implementers.
permalink: docs
---

- [Getting started](getting-started.md): install Ascribe, write a first page, check it, build it, and publish it with Astro.
- [Install](install.md): every way to install `ascribe`, with npm or on your path without Node.
- [Content that varies](writing/variants.md): writing one page for readers on different platforms or products, with alternatives, availability, and a build for each audience.
- [Directive reference](writing/directives.md): the language: directives, attributes, titles, phrases, links, and images.
- @available: ascribe 0.4.0
  [Memos](writing/memos.md): keeping the reason a page is written the way it is beside it, for the people and agents who change it, and never for its readers.
- @available: ascribe 0.4.0
  [Glossaries](content-model/glossary.md): setting up a glossary, so every page links the terms it uses to their definitions.
- [`ascribe.toml` reference](content-model/index.md): the content model: content types, dimensions, availability, phrases, widgets, and builds.
- @available: ascribe 0.3.0
  [Prose, through Vale](checks/vale.md): checking your pages' prose with Vale, in the editor and in `ascribe check`.
- @available: ascribe 0.3.0
  [The project report](checks/report.md): what state the whole project is in, from the problems and pages it counts to the published site, and running it every day in CI.
- [Diagnostics](checks/diagnostics.md): every problem Ascribe reports, with its code and fix.
- @available: ascribe 0.2.0
  [Review](review/index.md): reviewing a pull request as readers will see it, in the page preview, the site preview, or a report from CI, and commenting there.
- [Editing](editor/index.md): the VS Code extension, workspaces with several projects, and other editors.
- @available: ascribe 0.3.0
  [Agents](agents/index.md): helping an AI coding agent write and fix pages, with instructions it reads on its own and a skill that teaches it the check-and-fix loop.
- @available: ascribe 0.2.0
  [Examples and drift](examples/index.md): keeping pages from falling behind the code they describe, with code examples taken from tested files and a report of the pages whose examples changed.
- [Astro](astro/index.md): publishing a site with `@ascribed/astro`.
- [Element library reference](astro/elements.md): the custom elements a site draws, and the CSS custom properties that theme them.
- @available: ascribe 0.3.0
  [Permalinks](permalinks/index.md): giving a section an address your product can link to that keeps working when the docs move, checking those links in the product's CI, and moving, aliasing, or retiring one.
- @available: ascribe 0.3.1
  [Page templates](templates/index.md): starting a page of a known kind, such as a how-to, from a template and a guide to what goes in it.
- [Command reference](command/index.md): every `ascribe` command, its options, outputs, and exit codes.
- [Files Ascribe writes](command/project-files.md): every file Ascribe puts in a project, and which to commit.
- [Glossary](glossary.md): the terms these docs use.

## For implementers

These contracts say what an emitter writes and what a consumer can rely on. They matter if you're changing Ascribe's outputs or writing a consumer, and not if you're writing documentation with it.

- [The `ascribe.toml` format](contracts/content-model.md): the normative contract for the content model file, with the rules a loader enforces and the decisions behind them. The [reference](content-model/index.md) above is where to look up a key while writing one.
- [Output layout](contracts/output-layout.md): where `ascribe build` writes each build's output, how it records what it wrote, and how it replaces a previous output.
- [Assets](contracts/assets.md): where assets go in each output and how pages refer to them.
- [JSON reports](contracts/json-reports.md): the schemas of what the commands write with `--format json`.
- [The permalinks list](contracts/permalinks-list.md): the format of `ascribe.permalinks`, the list of a project's permalinks a product can check its links against.
- [Site render](contracts/site-render.md): the markers the site output writes and how a consumer renders them. The fixtures are in [`tests/render/`]({repo}/tree/main/tests/render/).

The [Ascribe specification]({repo}/blob/main/SPEC.md) is the normative definition of the language. The [changelog]({repo}/blob/main/CHANGELOG.md) lists what each release changed.
