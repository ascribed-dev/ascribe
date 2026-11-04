---
type: concept
title: How Ascribe works
description: "The whole system on one page: how pages and `ascribe.toml` go through the checks into builds, what each build writes, how a site is published, and where the same checks run."
---

Ascribe is one program that reads a project's pages and its content model, checks them, and builds them into a website, plain Markdown, and JSON. Everything else, the editor, CI, review, and agents, runs that same program's checks. This page shows how the parts fit, with a link to each part's own pages.

```text
  What you write              What Ascribe does             What it writes

  pages (.md) ------+                                 +--> site ----------> Astro --> your host
  ascribe.toml -----+--> checks --> builds -----------+--> plain Markdown
  code, OpenAPI ----+       ^                         +--> JSON
                            |
          +----------+------+------+----------+
        editor       CI         review      agents
```

## What you write

A project is a folder of **pages**: Markdown files with YAML frontmatter, `@` directives for what Markdown has no syntax for, such as notes, tabs, and content that varies, and `{key}` [phrases](glossary.md#phrase) for text written once. [Write a page](writing/write-a-page.md) starts there.

Beside them, **`ascribe.toml`** is the [content model](glossary.md#content-model): the kinds of page the project has and what each must say in its frontmatter, the phrases, the [dimensions](glossary.md#dimension) content varies along, and the [builds](glossary.md#build). It's the project's schema, and the checks hold every page to it ([The content model](content-model/index.md)). `ascribe.nav` sets the order of the sidebar ([Navigation](navigation/index.md)).

A page can also show code from the product's own files with `@snippet`, so the example changes when the code does ([Examples and drift](examples/index.md)), and document an API from its OpenAPI description ([OpenAPI](openapi/index.md)).

## The checks

[`ascribe check`](checks/check.md#cli-check) resolves every page as each build will publish it, with its includes, phrases, and variants filled in, and checks it against the content model: its frontmatter, its directives, its links and anchors, its examples, and the rest. Each problem it finds is a diagnostic with a name and a fix, and each one is in [the diagnostics reference](checks/diagnostics.md).

Every other part of Ascribe runs these checks rather than its own, so a problem is reported the same way wherever it's found.

## Builds and their outputs

A **build** is a named set of choices in `ascribe.toml`: which variants a page keeps, and which availability, so one project can publish a site for each product, platform, or release. [`ascribe build`](checks/check.md#cli-build) runs the checks first, and writes nothing if they find an error. Then it writes three outputs for each build:

- **site**: Markdown with Ascribe's elements, such as `<ascribe-note>` and `<ascribe-tabs>`, for a site generator to render.
- **plain Markdown**: every directive resolved to ordinary Markdown with no HTML, for search indexes, AI agents, and export.
- **JSON**: each page's resolved tree, for your own tools.

Where each is written, and what it holds, is in [the outputs of `ascribe build`](checks/check.md#outputs) and [the output-layout contract](contracts/output-layout.md).

## Publishing the site

Ascribe builds files and doesn't host them. [`@ascribed/astro`](astro/index.md) publishes the site output as an [Astro](https://astro.build) site: it runs the build before Astro loads content, renders the pages with Ascribe's elements, and draws the sidebar from `ascribe.nav`. Whatever serves your site now serves the files it writes ([Deploy the site](astro/deploy.md)).

A project that writes for agents also publishes the plain Markdown beside the site, with `llms.txt` to list it ([Publish for AI agents](astro/agents.md)).

## The same checks everywhere

The checks run in four more places, and each reports what `ascribe check` reports:

- **The editor.** The VS Code extension checks a page as you type, through the [language server](editor/language-server.md), `ascribe lsp`, which any editor can run ([Editing](editor/index.md)).
- **CI.** A workflow runs `ascribe check` on every pull request, so a problem nobody saw in the editor still fails it ([Checks in CI](checks/ci.md)).
- **Review.** Review shows a pull request's changed pages rendered and marked, and counts the problems `ascribe check` finds beside the changes ([How review works](review/how-it-works.md)).
- **Agents.** An agent gets the checks as tools from [the MCP server](agents/mcp.md), and [hooks](agents/hooks.md) run them after each file it writes and before it finishes ([Agents](agents/index.md)).

## Related resources

- [](getting-started.md)
- [](content-model/index.md)
- [](checks/check.md)
- [](astro/index.md)
- [](review/how-it-works.md)
- [](agents/index.md)
