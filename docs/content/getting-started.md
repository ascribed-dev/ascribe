---
type: tutorial
title: Getting started
description: Install Ascribe, write a first page, check it, build it, see it, and publish it with Astro.
permalink: getting-started
---

Ascribe is a toolchain for documentation written as code: Markdown with the structure docs need, checked as you write, and built into a website, plain Markdown, and JSON. This guide sets up a project and writes, checks, builds, and previews a first page. Its second half, [Astro](astro/index.md), publishes the page as a site.

It's for someone new to Ascribe who knows Markdown and a terminal. By the end, you'll have a project with a content model, a page that the editor checks as you write it, and a build of it ready for a site.

## Before you start

You need [Node.js](https://nodejs.org) {node} or later, on a [platform Ascribe runs on](install.md#platforms).

## Install the command
@memo: This section and the next are the two installs the landing page and the README show beside their link here (decision 86 in project-docs/decisions.md). Keep them first; scripts/docs-site/facts.test.ts fails when the copies differ.

In your project's directory:

@variant {pm=npm}:
```sh
npm install --save-dev @ascribed/cli
npx ascribe --version
```
@variant {pm=pnpm}:
```sh
pnpm add --save-dev @ascribed/cli
pnpm exec ascribe --version
```
@end

`@ascribed/cli` installs the `ascribe` binary for your platform, and pins its version for everyone who works on the project, and for CI. The latest release is {version}. To install it another way, such as without Node, see [Install](install.md).

## Install the editor

Install the **Ascribe** extension for VS Code from the Marketplace:

```sh
code --install-extension Ascribe.ascribe-vscode
```

It checks your pages as you type, completes directives, [phrases](glossary.md#phrase), and links, and previews pages as the site shows them. It uses the project's `ascribe` when there is one. See [Editing](editor/index.md).

Format your pages with Ascribe, not a general Markdown formatter: one such as Prettier can change what a page builds to as it saves it. When another extension formats Markdown in your project's folder, the extension offers to be its Markdown formatter instead; choose **Use Ascribe**. See [Formatting](editor/writing.md#formatting).

After you install it, its walkthrough, **Get Started with Ascribe**, shows the preview, the actions bar that writes directives for you, and the Ascribe sidebar. It's under **Help → Welcome** whenever you want it again.

## Create `ascribe.toml`
@id {permalink=true}: create-ascribetoml

@summary
`ascribe.toml` is the content model: the project's schema. It marks the project root, and says what each kind of page must have in its frontmatter.

Start the project with the command:

@variant {pm=npm}:
```sh
npx ascribe init
```
@variant {pm=pnpm}:
```sh
pnpm exec ascribe init
```
@end

It writes the project's first `ascribe.toml`, and makes `docs/`, the folder the pages go in. The file says what each part is for:

@snippet: code:examples/getting-started/ascribe.toml#init

Every page is of the one type, `page`, and says so with `type: page` in its frontmatter, with a `title` and a `description`. Output goes to `.ascribe/build/`, and there's one build, `site`. `ascribe init --templates` starts the project with six page types too, each with a template to start a page from; see [Page templates](templates/index.md).

### Add what the page uses

The page below uses what most projects add next: a dimension the content varies by, a few phrases, and the address the site is published at. Add them at the end of `ascribe.toml`:

@snippet: code:examples/getting-started/ascribe.toml#added

`values` are what a page writes, and `label` and `labels` are what readers see: the dimension's name, and each value's. The site's tabs read **npm**, **pnpm**, and **Yarn**; without `labels`, a tab shows the value as written, so the last would read **yarn**.

The [`ascribe.toml` reference](content-model/index.md) covers every section: content types and their frontmatter, availability, features, the glossary, project widgets, and builds.

Add the output directory to `.gitignore`:

```text
.ascribe/
```

[Files Ascribe writes](command/project-files.md) lists the other files Ascribe puts in a project, and which to commit.

## Write a page

Create `docs/install.md`:

@snippet {lang=markdown}: code:examples/getting-started/docs/install.md

- `@note` is a callout; `{type=tip}` is its attribute.
- `@variant` marks alternatives by package manager. The site shows them as tabs, and a build can keep just one. See [Content that varies](writing/variants.md).
- `type: page` says what kind of page it is: a type `ascribe.toml` declares. `title` and `description` are the fields that type asks for.
- `{product}` and `{version}` are phrases from `ascribe.toml`.
- `@steps` marks the list as a procedure.
- The link points at a **file**, `configure.md`. Ascribe writes the URL.

The [directive reference](writing/directives.md) describes every directive, and [phrases, links, and images](writing/inline.md) the rest of what a page writes.

## Check it

```sh
npx ascribe check
```

The link to `configure.md` is reported, since that page doesn't exist yet:

@snippet {lang=text}: code:crates/ascribe-cli/tests/output/getting-started-check.txt

Every diagnostic has a code, and the [diagnostics reference](checks/diagnostics.md) says how to fix each. `ascribe check` exits with 1 when there are errors, so it can gate CI; add `--deny-warnings` to fail on warnings too. Create `docs/configure.md` with the same frontmatter fields as `install.md`, `type: page` among them, and check again.

## Format it

```sh
npx ascribe fmt
```

rewrites directives and attribute blocks into their canonical spelling, and changes nothing else. `ascribe fmt --check` reports what would change, for CI.

## Build it

```sh
npx ascribe build
```

Each build writes three outputs to `.ascribe/build/<build>/`:

- `site/`: Markdown with web components, for Astro;
- `plain/`: plain Markdown with everything resolved, for search indexes and LLMs;
- `json/`: the resolved pages as JSON, for your own tools.

See [`ascribe build`](checks/check.md#cli-build) for its options, and the [command reference](command/index.md) for the rest.

## See it

```sh
npx ascribe preview
```

renders the build's pages as a site into `.ascribe/preview/`, with the sidebar, the page's contents, and search, and serves it on this computer. It prints the check's problems, then the address to open:

```text
previewing build `site` at http://127.0.0.1:4880/
```

Open it in a browser to see `install.md` as a reader does, its note, its alternatives by package manager, and its steps. As you edit a page or `ascribe.toml`, it renders the pages again; reload to see the change. It needs nothing but the command: no Node, no editor, and no Astro. Ctrl-C stops it. The published site has its own look, which the next step gives it. See [`ascribe preview`](checks/check.md#cli-preview).

## Publish it with Astro

Publishing the site is this guide's second half, on its own page: [Astro](astro/index.md) adds Ascribe to an [Astro](https://astro.build) project, writes the site's files with one command, and ends with the site open in your browser. For a new site:

@steps
1. Create an Astro project: `npm create astro@latest`.
2. Move `ascribe.toml` and `docs/` into it, beside `astro.config.mjs`.
3. Follow the guide from its first step.

For an existing site, keep this project where it is, follow the guide in the Astro project, and set the integration's `project` option to this project's directory.

Either way:

- `ascribe.toml`'s `[consumer]` `site` and Astro's `site` must be the same address. The `ascribe.toml` above sets it, and the Astro build fails when the two differ.
- You don't run `ascribe build` yourself: the integration runs it before Astro loads the pages, and again as you edit in `astro dev`.

## Summary

You installed the command and the editor, declared a content model in `ascribe.toml`, and wrote a page with a note, alternatives by package manager, phrases, and a procedure. Then you checked it, formatted it, built its site output, plain Markdown, and JSON, and saw it as a site with `ascribe preview`, ready for Astro to publish.

## Next steps

The [docs' home page](index.md) has a path for each kind of reader, a few pages in order from here:

- [For engineers](index.md#for-engineers): checks in CI, code examples from tested files, and review in CI.
- [For writers](index.md#for-writers): the editor and its preview, content that varies, reuse, a glossary and templates, and review in the editor.
- [For agent builders](index.md#for-agent-builders): `ascribe agents sync`, the MCP server and hooks, and what the outputs give agents that read the docs.
