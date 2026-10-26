---
type: index
title: Ascribe documentation
description: Guides and references for writing documentation with Ascribe, and the contracts for implementers.
permalink: docs
---

## Start

- [Getting started](getting-started.md): install Ascribe, write a first page, check it, build it, and publish it with Astro.
- [Install](install.md): every way to install `ascribe`, with npm or on your path without Node.

## Write

- [Directive syntax](writing/syntax.md): the language: how a directive is written, its attributes, forms, titles, and what it applies to.
- [Directive reference](writing/directives.md): each built-in directive, its attributes, and its rules.
- [Phrases, links, and images](writing/inline.md) and [frontmatter](writing/frontmatter.md): the rest of what a page writes.
- [Content that varies](writing/variants.md): writing one page for readers on different platforms or products, with alternatives, availability, and a build for each audience.
- [Memos](writing/memos.md): keeping the reason a page is written the way it is beside it, for the people and agents who change it, and never for its readers.
- [Glossaries](content-model/glossary.md): setting up a glossary, so every page links the terms it uses to their definitions.
- [Page templates](templates/index.md): starting a page of a known kind, such as a how-to, from a template and a guide to what goes in it.
- [Examples and drift](examples/index.md): keeping pages from falling behind the code they describe, with code examples taken from tested files and a report of the pages whose examples changed.
- [Link to your API reference](openapi/index.md): linking a guide to an operation in your OpenAPI spec, checked against the spec and written as the operation's address.
- [Editing](editor/index.md): the VS Code extension, workspaces with several projects, and other editors.

## Check

- [Checks in CI](checks/ci.md): checking every pull request with the same command the editor runs, and the formatting check, in GitHub Actions.
- [Prose, through Vale](checks/vale.md): checking your pages' prose with Vale, in the editor and in `ascribe check`.
- [The project report](checks/report.md): what state the whole project is in, from the problems and pages it counts to the published site, and running it every day in CI.
- [Diagnostics](checks/diagnostics.md): every problem Ascribe reports, with its code and fix.

## Review

- [Review](review/index.md): reviewing a pull request as readers will see it, in the page preview, the site preview, or the HTML preview from CI, and commenting there.

## Publish

- [Astro](astro/index.md): publishing a site with `@ascribed/astro`.
- [Navigation](navigation/index.md): the order a project's sections and pages come in, from `ascribe.nav` or the default, and their labels, which every sidebar is drawn from.
- [Publishing](publishing/index.md): what a project tells search engines and AI crawlers about its pages, and what it delivers to agents.
- [Keep the addresses you've published](publishing/redirects.md): recording where a page went when it moves or goes, so a link to its old address still leads somewhere.
- [Permalinks](permalinks/index.md): giving a section an address your product can link to that keeps working when the docs move, checking those links in the product's CI, and moving, aliasing, or retiring one.
- [Publish a site for each version](versions/index.md): building the docs of every version a reader can be on from one source, a site for each release line, and moving them on when a line ships.

## Agents

- [Agents](agents/index.md): helping an AI coding agent write and fix pages, with instructions it reads on its own and a skill that teaches it the check-and-fix loop.

## Reference

- [`ascribe.toml` reference](content-model/index.md): the content model: content types, dimensions, availability, phrases, widgets, and builds.
- [Command reference](command/index.md): every `ascribe` command, its options, outputs, and exit codes.
- [Files Ascribe writes](command/project-files.md): every file Ascribe puts in a project, and which to commit.
- [Element library reference](astro/elements.md): the custom elements a site draws, and the CSS custom properties that theme them.
- [Glossary](glossary.md): the terms these docs use.

## For implementers

These contracts say what an emitter writes and what a consumer can rely on. They matter if you're changing Ascribe's outputs or writing a consumer, and not if you're writing documentation with it.

- [The `ascribe.toml` format](contracts/content-model.md): the normative contract for the content model file, with the rules a loader enforces and the decisions behind them. The [reference](content-model/index.md) above is where to look up a key while writing one.
- [Output layout](contracts/output-layout.md): where `ascribe build` writes each build's output, how it records what it wrote, and how it replaces a previous output.
- [Assets](contracts/assets.md): where assets go in each output and how pages refer to them.
- [Navigation](contracts/navigation.md): what `_ascribe/nav.json` holds, the sections and pages in order with their labels, and how a site draws a sidebar from it.
- [Archive](contracts/archive.md): what `ascribe versions archive` writes into a folder, and what each page holds.
- [JSON reports](contracts/json-reports.md): the schemas of what the commands write with `--format json`.
- [The permalinks list](contracts/permalinks-list.md): the format of `ascribe.permalinks`, the list of a project's permalinks a product can check its links against.
- [Tested examples](contracts/tested-examples.md): the formats of `ascribe.examples.toml` and of the observation `ascribe observe` writes beside it.
- [Site render](contracts/site-render.md): the markers the site output writes and how a consumer renders them. The fixtures are in [`tests/render/`]({repo}/tree/main/tests/render/).

The [Ascribe specification]({repo}/blob/main/SPEC.md) is the normative definition of the language. The [changelog]({repo}/blob/main/CHANGELOG.md) lists what each release changed.
