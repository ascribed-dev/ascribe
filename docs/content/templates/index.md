---
type: how-to
title: Page templates
description: Starting a page of a known kind, such as a how-to, from a template and a guide to what goes in it, written into your project by ascribe templates eject.
---

A page of a known kind has a shape: a how-to has what to do before you start, then numbered steps, then what to do next. A template is a page with that shape and things to fill in, and its guide says what belongs in each part. Ascribe carries a set of six, and writes them into your project with one command, where they're your files.

## Where the templates come from

The templates are [The Good Docs Project](https://www.thegooddocsproject.dev/template)'s, from their repository, [gitlab.com/tgdp/templates](https://gitlab.com/tgdp/templates), at their release v1.6.0. Their work is under the MIT No Attribution License, which asks for no credit; this is it anyway, with thanks.

Ascribe converts each one into an Ascribe page and changes nothing of their wording: each `{placeholder}` is written as `\{placeholder}`, so it shows as written and isn't read as a phrase. Each guide says at its top where it came from, and `ascribe templates list` shows the set and its release.

@available: ascribe 0.4.0
A template's instructions to the writer, such as `{This section is optional}`, are [memos](../writing/directives.md#memo): `@memo: This section is optional`. A memo is never published: follow it, then delete it or keep it. A paragraph that marks where the page's own text goes, such as the how-to's `{Optional: Explanatory text}` in a step, stays a placeholder.

The set has six templates, for a how-to, a concept, a tutorial, a reference, a troubleshooting page, and release notes. Each is one of theirs, and has a type of the same name.

## Take the templates

To start a project with the set, run `ascribe init --templates` in an empty folder: it writes the project's first `ascribe.toml` and takes the six templates as `eject` would ([command reference](../install.md#cli-init)).

In a project that has its `ascribe.toml` already, take the whole set:

```sh
ascribe templates eject
```

It writes each template, its guide, and Ascribe's notes on its type into `templates/`, beside `ascribe.toml`, with `templates/templates.lock`, which records what each was copied from, and adds the six types to `ascribe.toml`, below what's there. A type is declared like this:

```toml
[types.how-to]

[types.how-to.frontmatter]
title = "string"
description = "string"
```

The type names no template: a type with no `template` and no `guide` takes both from the lock, which records them under the type's name. A type that names either keeps what it names, which is how a project without the lock, or with a template written by hand, gives a type its template. In VS Code, the hover on the type's name in `ascribe.toml` says where its template comes from.

A page says what type it is in its frontmatter, `type: how-to`, as each template does, so a page of any type can go in any folder. Commit the files, the lock with them. The rules [`ascribe agents sync`](../agents/instructions.md) writes say what each type is for; to say it your own way, give the type a [`purpose`](../content-model/index.md#41-page-types).

To take only some, name them: `ascribe templates eject how-to concept`. A type the project declares already is refused, and nothing is written; to keep it and take its template, add `--keep-type`. The [command reference](command.md#cli-templates-eject) has the rest.

## Change a template

From then on the files are your project's, and nothing in Ascribe writes over them. Change a template as you'd change any page: take out a section your project never has, add one it always does, or write your own placeholder as `\{What goes here}`. Add to the guide or the notes, or add a file of your own to the type's `guide` list.

The check compares a page with the template as it is now, so a placeholder you take out isn't looked for any more, and one you add is.

## See what's changed in the set

A later `ascribe` carries The Good Docs Project's newer release. Nothing changes your copies: to see what's different, run:

```sh
ascribe templates status
```

For each template you took, it says whether the set has changed it since your copy, and whether you've changed it. To take a change, see it, then copy what you want by hand:

```sh
ascribe templates status --diff how-to
```

The [command reference](command.md#cli-templates-status) has its options, for a CI job too, and its JSON.

## In a project that has pages

A project with pages written already can take the set too; Ascribe's own docs did. Its pages name no type yet if the project declared none, and `ascribe check` offers a fix for each. Pages written before their template have none of its placeholders, so the check reports nothing on them, and a page that's none of the six, such as a home page that lists the others, gets a type of your own with no template. A guide says what a kind of page is for; a page that already does its job needn't match its template heading for heading.

## Ascribe's notes

Their guide says what belongs in a page of each kind, for any tool. Ascribe's notes, `<type>.ascribe.md`, are a minute's reading after it: only what's particular to writing that kind of page in Ascribe, such as a how-to's alternatives by platform as a [variant](../glossary.md#variant) group, and a step's command taken from tested code with [`@snippet`](../writing/directives.md#snippet).

## Start a page

In VS Code, run **Ascribe: New Page**. It asks for the type and the page's title, then makes the page from the type's template, with the first thing to fill in selected; **Ascribe: Go to Next Placeholder** selects the next ([editor guide](../editor/actions.md#start-a-page)).

Without the editor:

@steps
1. Copy the template to where the page goes. It names its type already, `type: how-to`:

   ```sh
   cp templates/how-to.md docs/rotate-keys.md
   ```

2. Read the type's guide and Ascribe's notes on it, before you write.
3. Replace each `\{placeholder}` with what it asks for, and the title and description in the frontmatter with your own.

An agent starts from the template too. Ask it for a page with the plugin's `/ascribe:new-page`, or the prompt `ascribe agents prompt new-page` prints ([agents guide](../agents/prompts.md#from-the-command-line)), which gives it the template and sends it to the guide and the notes before it writes:

```sh
ascribe agents prompt new-page --arg type=how-to --arg title="Rotate keys"
```

## What's left to fill in

`ascribe check` compares a page with its type's template and reports, as advice, what it still has of it: a placeholder or a frontmatter field the page has word for word ([`template-placeholder-left`](../checks/diagnostics/content-checks.md#ASC195)), and a section the template gives only memos that the page leaves empty ([`template-section-empty`](../checks/diagnostics/content-checks.md#ASC229)). A memo the page keeps isn't reported. In VS Code, the sidebar's [This page](../editor/sidebar.md#this-page) says how many are left, and **Go to Next Placeholder** goes to the next one.

The check finds only what was never touched, so read a page through before publishing it. To turn either check off, or make it a warning, set its level in [`[checks]`](../content-model/checks.md#19-checks).

## See also

- [`ascribe templates`](command.md): `list`, `eject`, and `status`, their options, and their output.
- [Page types](../content-model/index.md#41-page-types): a type's `template`, `guide`, and `purpose`, and the template it takes from the lock.
