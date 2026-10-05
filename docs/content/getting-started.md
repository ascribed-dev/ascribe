---
title: Getting started
description: Install Ascribe, write a first page, check it, build it, and publish it with Astro.
---

Ascribe is documentation written as code: Markdown with a small set of directives for the structure documentation needs (notes, procedures, alternatives by platform or product, availability, includes), checked like code, and built into a website, plain Markdown, and JSON. This guide sets up a project, writes a first page, and publishes it with Astro.

You need [Node.js](https://nodejs.org) {node} or later. Ascribe runs on macOS on Apple silicon, Linux (x64 and arm64, with glibc 2.28 or later, which includes the build images of Netlify, Vercel, Cloudflare Pages, and AWS Amplify), and Windows (x64).

## Install the command

In your project's directory:

```sh
npm install --save-dev @ascribed/cli
npx ascribe --version
```

`@ascribed/cli` installs the `ascribe` binary for your platform, and pins its version for everyone who works on the project, and for CI. If `npx ascribe` can't find the binary, your package manager left out optional dependencies; reinstall with them enabled.

To try what's on `main` before it's released, install `@ascribed/cli@next` and `@ascribed/astro@next` instead: a build published every night, with no promise of stability.

## Install the editor

Install the **Ascribe** extension for VS Code from the Marketplace:

```sh
code --install-extension Ascribe.ascribe-vscode
```

It checks your pages as you type, completes directives, phrases, and links, and previews pages as the site shows them. It uses the project's `ascribe` when there is one. See [Editing](guides/editor.md).

## Create `ascribe.toml`

`ascribe.toml` is the content model: the project's schema. It marks the project root, and the smallest one is a single line:

```toml
spec = "0.1"
```

With nothing else declared, pages live in `docs/`, output goes to `.ascribe/build/`, every page needs a `title`, and there's one build, `site`. A more useful start declares a dimension your content varies by, and a few phrases:

```toml
spec = "0.1"

[dimensions.pm]
label = "Package manager"
values = ["npm", "pnpm", "yarn"]

[phrases]
product = "Quill"
version = "3.4.1"

[consumer]
site = "https://docs.example.com"
```

The [`ascribe.toml` reference](reference/content-model.md) covers every section: content types and their frontmatter, availability, features, the glossary, project widgets, and builds.

Add the output directory to `.gitignore`:

```text
.ascribe/
```

## Write a page

Create `docs/install.md`:

````markdown
---
title: Install Quill
---

@note {type=tip}: {product} {version} needs Node.js 22 or later.

## Install the package

@variant {pm=npm}:
```sh
npm install quill
```
@variant {pm=pnpm}:
```sh
pnpm add quill
```
@variant {pm=yarn}:
```sh
yarn add quill
```
@end

## Set it up

@steps
1. Create `quill.yaml`.
2. Run `quill init`.

Next, [configure it](configure.md).
````

- `@note` is a callout; `{type=tip}` is its attribute.
- `@variant` marks alternatives by package manager. The site shows them as tabs, and a build can keep just one.
- `{product}` and `{version}` are phrases from `ascribe.toml`.
- `@steps` marks the list as a procedure.
- The link points at a **file**, `configure.md`. Ascribe writes the URL.

The [directive reference](reference/directives.md) describes every directive and inline construct.

## Check it

```sh
npx ascribe check
```

The link to `configure.md` is reported, since that page doesn't exist yet:

```text
[ASC036] Error: `configure.md` doesn't exist
```

Every diagnostic has a code, and the [diagnostics reference](reference/diagnostics.md) says how to fix each. `ascribe check` exits with 1 when there are errors, so it can gate CI; add `--deny-warnings` to fail on warnings too. Create `docs/configure.md` with a `title`, and check again.

## Build it

```sh
npx ascribe build
```

Each build writes three outputs to `.ascribe/build/<build>/`:

- `site/`: Markdown with web components, for Astro;
- `plain/`: plain Markdown with everything resolved, for search indexes and LLMs;
- `json/`: the resolved pages as JSON, for your own tools.

See the [command reference](reference/cli.md#ascribe-build).

## Format it

```sh
npx ascribe fmt
```

rewrites directives and attribute blocks into their canonical spelling, and changes nothing else. `ascribe fmt --check` reports what would change, for CI.

## Publish it with Astro

In an [Astro](https://astro.build) project, with `ascribe.toml` beside `astro.config.mjs`:

```sh
npm install @ascribed/astro
```

```js
// astro.config.mjs
import { defineConfig } from "astro/config";
import ascribe from "@ascribed/astro";

export default defineConfig({
  site: "https://docs.example.com",
  integrations: [ascribe({ build: "site" })],
});
```

Then define the content collection, a route, and a layout that loads the elements. [Astro](guides/astro.md) walks through each file.

## Check it in CI

Run the same command the editor runs:

```yaml phrases=true
# .github/workflows/docs.yml
name: Docs
on: [pull_request]
jobs:
  check:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-node@v4
        with:
          node-version: {node}
      - run: npm ci
      - run: npx ascribe check --deny-warnings
      - run: npx ascribe fmt --check
```

## Next

- [Directive reference](reference/directives.md): the language.
- [`ascribe.toml` reference](reference/content-model.md): the content model.
- [Command reference](reference/cli.md): `check`, `build`, `fmt`, and `lsp`.
- [Diagnostics](reference/diagnostics.md): every problem Ascribe reports, and its fix.
- [Editing](guides/editor.md): the VS Code extension.
- [Astro](guides/astro.md): publishing a site.
