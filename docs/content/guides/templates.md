---
title: Page templates
description: Starting a page of a known kind, such as a how-to, from a template and a guide to what goes in it, written into your project by ascribe templates eject.
available: next
---

A page of a known kind has a shape: a how-to has what to do before you start, then numbered steps, then what to do next. A template is a page with that shape and things to fill in, and its guide says what belongs in each part. Ascribe carries a set of them, and writes the ones you ask for into your project, where they're your files.

## Where the templates come from

The templates are [The Good Docs Project](https://www.thegooddocsproject.dev/template)'s, from their repository, [gitlab.com/tgdp/templates](https://gitlab.com/tgdp/templates), at their release v1.6.0. Their work is under the MIT No Attribution License, which asks for no credit; this is it anyway, with thanks.

Ascribe converts each one into an Ascribe page, with a script, and changes nothing of their wording: what the template says about itself is taken out, its title moves into the frontmatter, and each `{placeholder}` is written as `\{placeholder}`, so it shows as written and isn't read as a phrase. Each guide says at its top where it came from. `ascribe templates list` shows the set, and which release it's from.

The set has one template for now, the how-to.

## Write a template into your project

In a project, the folder with its `ascribe.toml`:

```sh
ascribe templates eject how-to
```

It writes the template and its guide into `templates/`, beside `ascribe.toml`, and adds the how-to type to `ascribe.toml`, below what's there:

```toml
[types.how-to]
files = ["how-to/**"]
template = "templates/how-to.md"
guide = ["templates/how-to.guide.md"]

[types.how-to.frontmatter]
title = "string"
description = "string"
```

A page in the content root's `how-to/` folder is a how-to now. Commit the files, with `templates/templates.lock`, which records what each was copied from.

From then on they're your project's. Change the template, add to the guide, or add a file of your own notes to the type's `guide` list: nothing in Ascribe writes over them. `--to` writes them into another folder outside the content root, and the [command reference](../reference/cli/sources.md#cli-templates-eject) has the rest.

`eject` adds to a project, and doesn't start one. Without an `ascribe.toml`, write one first: a file named `ascribe.toml`, beside the folder your pages go in (`docs`, unless you say otherwise), with the line `spec = "0.1"`.

## Start a page

Copy the template into the type's folder, and read the guide before you write:

```sh
mkdir -p docs/how-to
cp templates/how-to.md docs/how-to/rotate-keys.md
```

Then replace each `\{placeholder}` with what it asks for, and the title and description in the frontmatter with your own.
