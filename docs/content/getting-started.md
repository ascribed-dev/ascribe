---
type: tutorial
title: Getting started
description: Install Ascribe, write a first page, check it, build it, and publish it with Astro.
permalink: getting-started
---

Ascribe is documentation written as code: Markdown with a small set of directives for the structure documentation needs (notes, procedures, alternatives by platform or product, availability, includes), checked like code, and built into a website, plain Markdown, and JSON. This guide sets up a project and writes, checks, and builds a first page. Its second half, [Astro](guides/astro.md), publishes the page as a site.

It's for someone new to Ascribe who knows Markdown and a terminal. By the end, you'll have a project with a content model, a page that the editor and CI check as you write it, and a build of it ready for a site.

## Before you start

You need [Node.js](https://nodejs.org) {node} or later. Ascribe runs on macOS on Apple silicon, Linux (x64 and arm64, with glibc {glibc} or later (2.39 for 0.1.1 and earlier), which includes the build images of Netlify, Vercel, Cloudflare Pages, and AWS Amplify), and Windows (x64).

## Install the command

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

`@ascribed/cli` installs the `ascribe` binary for your platform, and pins its version for everyone who works on the project, and for CI. The latest release is {version}. If `ascribe` can't find the binary, your package manager left out optional dependencies; reinstall with them enabled.

@note {type=tip}: To try what's on `main` before it's released, install `@ascribed/cli@next` and `@ascribed/astro@next` instead: a build published every night, with no promise of stability.

## Install the editor

Install the **Ascribe** extension for VS Code from the Marketplace:

```sh
code --install-extension Ascribe.ascribe-vscode
```

It checks your pages as you type, completes directives, [phrases](reference/directives.md#phrases), and links, and previews pages as the site shows them. It uses the project's `ascribe` when there is one. See [Editing](guides/editor.md).

Format your pages with Ascribe, not a general Markdown formatter: one such as Prettier can change what a page builds to as it saves it. When another extension formats Markdown in your project's folder, the extension offers to be its Markdown formatter instead; choose **Use Ascribe**. See [Formatting](guides/editor.md#formatting).

@available: ascribe 0.3.0
After you install it, its walkthrough, **Get Started with Ascribe**, shows the preview, the actions bar that writes directives for you, and the Ascribe sidebar. It's under **Help → Welcome** whenever you want it again.

## Create `ascribe.toml`
@id {permalink=true}: create-ascribetoml

`ascribe.toml` is the content model: the project's schema. It marks the project root, and says what each kind of page must have in its frontmatter.

### Start with `ascribe init`
@available: next

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

Every page needs a `title` and a `description`. Output goes to `.ascribe/build/`, and there's one build, `site`. `ascribe init --templates` starts the project with six page types too, each with a template to start a page from; see [Page templates](guides/templates.md).

### Write it by hand

For a project that wants nothing written for it, or with Ascribe 0.3.0 or earlier, which don't have `ascribe init`, write `ascribe.toml` yourself, and make `docs/`. The smallest is a single line:

```toml
spec = "0.1"
```

With nothing else declared, pages live in `docs/`, output goes to `.ascribe/build/`, there's one build, `site`, and every page needs a `title` and has no other field, so leave out the page's `description` below.

### Add what the page uses

The page below uses what most projects add next: a dimension the content varies by, a few phrases, and the address the site is published at. Add them at the end of `ascribe.toml`:

@snippet: code:examples/getting-started/ascribe.toml#added

The [`ascribe.toml` reference](reference/content-model.md) covers every section: content types and their frontmatter, availability, features, the glossary, project widgets, and builds.

Add the output directory to `.gitignore`:

```text
.ascribe/
```

[Files Ascribe writes](reference/project-files.md) lists the other files Ascribe puts in a project, and which to commit.

## Write a page

Create `docs/install.md`:

@snippet {lang=markdown}: code:examples/getting-started/docs/install.md

- `@note` is a callout; `{type=tip}` is its attribute.
- `@variant` marks alternatives by package manager. The site shows them as tabs, and a build can keep just one. See [Content that varies](guides/variants.md).
- `title` and `description` are the fields `ascribe.toml` asks every page for.
- `{product}` and `{version}` are phrases from `ascribe.toml`.
- `@steps` marks the list as a procedure.
- The link points at a **file**, `configure.md`. Ascribe writes the URL.

The [directive reference](reference/directives.md) describes every directive and inline construct.

## Check it

```sh
npx ascribe check
```

The link to `configure.md` is reported, since that page doesn't exist yet:

@snippet {lang=text}: code:crates/ascribe-cli/tests/output/getting-started-check.txt

Every diagnostic has a code, and the [diagnostics reference](reference/diagnostics.md) says how to fix each. `ascribe check` exits with 1 when there are errors, so it can gate CI; add `--deny-warnings` to fail on warnings too. Create `docs/configure.md` with the same frontmatter fields as `install.md`, and check again.

## Build it

```sh
npx ascribe build
```

Each build writes three outputs to `.ascribe/build/<build>/`:

- `site/`: Markdown with web components, for Astro;
- `plain/`: plain Markdown with everything resolved, for search indexes and LLMs;
- `json/`: the resolved pages as JSON, for your own tools.

See the [command reference](reference/cli/check.md#cli-build).

## Format it

```sh
npx ascribe fmt
```

rewrites directives and attribute blocks into their canonical spelling, and changes nothing else. `ascribe fmt --check` reports what would change, for CI.

## Publish it with Astro

Publishing the site is this guide's second half, on its own page: [Astro](guides/astro.md) adds Ascribe to an [Astro](https://astro.build) project, writes the site's files with one command, and ends with the site open in your browser. For a new site:

@steps
1. Create an Astro project: `npm create astro@latest`.
2. Move `ascribe.toml` and `docs/` into it, beside `astro.config.mjs`.
3. Follow the guide from its first step.

For an existing site, keep this project where it is, follow the guide in the Astro project, and set the integration's `project` option to this project's directory.

Either way:

- `ascribe.toml`'s `[consumer]` `site` and Astro's `site` must be the same address. The `ascribe.toml` above sets it, and the Astro build fails when the two differ.
- You don't run `ascribe build` yourself: the integration runs it before Astro loads the pages, and again as you edit in `astro dev`.

## Check it in CI

Run the same command the editor runs, and the formatting check, on every pull request:

@snippet {lang=yaml, title=.github/workflows/check.yml}: code:examples/docs-repository/.github/workflows/check.yml#workflow

It installs the latest release. To run the version your `package.json` pins instead, replace that step with `npm ci`, and run each command with `npx`.

## Summary

You installed the command and the editor, declared a content model in `ascribe.toml`, and wrote a page with a note, alternatives by package manager, phrases, and a procedure. Then you checked it, built its site output, plain Markdown, and JSON, formatted it, and set CI to check every pull request the same way.

## Next steps

- [Content that varies](guides/variants.md): alternatives by platform or product, availability, and builds.
- [Directive reference](reference/directives.md): the language.
- [`ascribe.toml` reference](reference/content-model.md): the content model.
- [Command reference](reference/cli.md): every command, its options, and its exit codes.
- [Diagnostics](reference/diagnostics.md): every problem Ascribe reports, and its fix.
- [Editing](guides/editor.md): the VS Code extension.
- [Astro](guides/astro.md): publishing a site.
- @available: ascribe 0.3.0
  [Agents](guides/agents.md): working with an AI coding agent, such as Claude Code, GitHub Copilot, Cursor, or Codex, starting with `ascribe agents sync`, which gives it your project's rules.
