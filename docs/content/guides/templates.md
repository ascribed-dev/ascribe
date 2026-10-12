---
title: Page templates
description: Starting a page of a known kind, such as a how-to, from a template and a guide to what goes in it, written into your project by ascribe templates eject.
available: next
---

A page of a known kind has a shape: a how-to has what to do before you start, then numbered steps, then what to do next. A template is a page with that shape and things to fill in, and its guide says what belongs in each part. Ascribe carries a set of them, and writes the ones you ask for into your project, where they're your files.

## Where the templates come from

The templates are [The Good Docs Project](https://www.thegooddocsproject.dev/template)'s, from their repository, [gitlab.com/tgdp/templates](https://gitlab.com/tgdp/templates), at their release v1.6.0. Their work is under the MIT No Attribution License, which asks for no credit; this is it anyway, with thanks.

Ascribe converts each one into an Ascribe page, with a script, and changes nothing of their wording: what the template says about itself is taken out, its title moves into the frontmatter, and each `{placeholder}` is written as `\{placeholder}`, so it shows as written and isn't read as a phrase. Each guide says at its top where it came from. `ascribe templates list` shows the set, and which release it's from.

The set has six templates, for a how-to, a concept, a tutorial, a reference, a troubleshooting page, and release notes. Each is one of theirs, and has a type of the same name.

## Write a template into your project

In a project, the folder with its `ascribe.toml`:

```sh
ascribe templates eject how-to
```

It writes the template, its guide, and Ascribe's notes on the type into `templates/`, beside `ascribe.toml`, and adds the how-to type to `ascribe.toml`, below what's there:

```toml
[types.how-to]
files = ["how-to/**"]
template = "templates/how-to.md"
guide = ["templates/how-to.guide.md", "templates/how-to.ascribe.md"]

[types.how-to.frontmatter]
title = "string"
description = "string"
```

A page in the content root's `how-to/` folder is a how-to now. Commit the files, with `templates/templates.lock`, which records what each was copied from.

From then on they're your project's. Change the template, add to the guide or the notes, or add a file of your own to the type's `guide` list: nothing in Ascribe writes over them. `--to` writes them into another folder outside the content root, and the [command reference](../reference/cli/sources.md#cli-templates-eject) has the rest.

`eject` adds to a project, and doesn't start one. Without an `ascribe.toml`, write one first: a file named `ascribe.toml`, beside the folder your pages go in (`docs`, unless you say otherwise), with the line `spec = "0.1"`.

## Ascribe's notes

Their guide says what belongs in a page of each kind, and it's written for any tool. Ascribe's notes, `<type>.ascribe.md`, are a minute's reading after it: only what's particular to writing that kind of page in Ascribe, such as a how-to's alternatives by platform as a [variant](../reference/directives.md#variant) group, and a step's command taken from tested code with [`@snippet`](../reference/directives.md#snippet). They're the only words in the set that are Ascribe's, and `templates.lock` records them apart from their files.

## Start a page

Copy the template into the type's folder, and read the guide and the notes before you write:

```sh
mkdir -p docs/how-to
cp templates/how-to.md docs/how-to/rotate-keys.md
```

Then replace each `\{placeholder}` with what it asks for, and the title and description in the frontmatter with your own.

## What's left to fill in

`ascribe check` compares a page with its type's template, and reports what it still has of it as advice (`template-placeholder-left`):

- **A placeholder,** the template's text from a `\{` to its closing brace, that the page still has word for word. One that runs across blocks, as the how-to's "See also" opens a brace in a paragraph and closes it at the end of a list, is reported once, where it starts, when the page has all of it.
- **A frontmatter field** with the value the template gives it, such as `title: "Title"`.
- **An instruction,** a paragraph the template has in a block quote, that the page still has whole. A block quote of your own isn't one.

How a line is wrapped doesn't matter, so an editor that rewraps a paragraph doesn't hide a placeholder in it. A page of a type with no template isn't looked at.

The check finds only what was never touched. A placeholder with one word changed, or with the space before its closing brace taken out, as the how-to's `\{Task name }` has, isn't the template's any more, and isn't reported; nor is one you've begun filling in. Read a page through before publishing it. The check compares a page with the template as it is now, so a placeholder you remove from your template is no longer looked for.

To turn it off, or make it a warning, set its level in [`[checks]`](../reference/content-model/checks.md#19-checks):

```toml
[checks]
template-placeholder-left = "warning"
```
