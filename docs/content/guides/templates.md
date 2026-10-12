---
title: Page templates
description: Starting a page of a known kind, such as a how-to, from a template and a guide to what goes in it, written into your project by ascribe templates eject.
available: next
---

A page of a known kind has a shape: a how-to has what to do before you start, then numbered steps, then what to do next. A template is a page with that shape and things to fill in, and its guide says what belongs in each part. Ascribe carries a set of six, and writes them into your project with one command, where they're your files.

## Where the templates come from

The templates are [The Good Docs Project](https://www.thegooddocsproject.dev/template)'s, from their repository, [gitlab.com/tgdp/templates](https://gitlab.com/tgdp/templates), at their release v1.6.0. Their work is under the MIT No Attribution License, which asks for no credit; this is it anyway, with thanks.

Ascribe converts each one into an Ascribe page, with a script, and changes nothing of their wording: what the template says about itself is taken out, its title moves into the frontmatter, and each `{placeholder}` is written as `\{placeholder}`, so it shows as written and isn't read as a phrase. Each guide says at its top where it came from. `ascribe templates list` shows the set, and which release it's from.

The set has six templates, for a how-to, a concept, a tutorial, a reference, a troubleshooting page, and release notes. Each is one of theirs, and has a type of the same name.

## Take the templates

To start a project with the set, run `ascribe init --templates` in an empty folder. It writes the project's first `ascribe.toml`, with a type for every page no template covers, and takes the six templates as `eject` would; the [command reference](../reference/cli/sources.md#cli-init) shows the file.

In a project that has its `ascribe.toml` already, take the whole set:

```sh
ascribe templates eject
```

It writes each template, its guide, and Ascribe's notes on its type into `templates/`, beside `ascribe.toml`, and adds the six types to `ascribe.toml`, below what's there. Each type's pages go in a folder of its name in the content root, and the command says which: `docs/how-to/` for how-tos, `docs/concept/` for concepts, and so on. A type is declared like this:

```toml
[types.how-to]
files = ["how-to/**"]
template = "templates/how-to.md"
guide = ["templates/how-to.guide.md", "templates/how-to.ascribe.md"]

[types.how-to.frontmatter]
title = "string"
description = "string"
```

Each type's page has a title and a description, as every template's frontmatter has. The folders are only a start: to keep a type's pages somewhere else, change its `files`, the line that says which pages are of it. Commit the files, with `templates/templates.lock`, which records what each was copied from. The rules [`ascribe agents sync`](agents.md#agent-instructions) writes say what each type is for with their summary of it, from that file; to say it your own way, give the type a [`purpose`](../reference/content-model.md#41-page-types).

To take only some, name them: `ascribe templates eject how-to concept`. `ascribe templates list` names the six. A type of the same name declared already, or a file where one would be written, is named, and then nothing is written. To keep a type the project declares, with its own `files` and frontmatter, and take the template for it, add `--keep-type`: it adds only the type's `template` and `guide`. `--to` writes them into another folder outside the content root, and the [command reference](../reference/cli/sources.md#cli-templates-eject) has the rest.

`eject` adds to a project, and doesn't start one: without an `ascribe.toml`, it says to run `ascribe init --templates`.

## Change a template

From then on the files are your project's, and nothing in Ascribe writes over them. Change a template as you'd change any page: take out a section your project never has, add one it always does, or write your own placeholder as `\{What goes here}`. Add to the guide or the notes, or add a file of your own to the type's `guide` list.

A change to a template changes what the check looks for, since it compares a page with the template as it is now. A placeholder you take out of the template isn't looked for in pages any more, and one you add is.

## See what's changed in the set

The Good Docs Project keeps improving their templates, and a later `ascribe` carries their newer release. Nothing changes your copies: to see what's different, run:

```sh
ascribe templates status
```

For each template you took, it says whether the set has changed it since your copy, from `templates.lock`'s record of each file as written, and whether you've changed it; then the templates of the set you don't have. A file the set changed and you didn't is listed with the sections that are different. It reads nothing but your project and the set your `ascribe` carries, so it works offline, and its first line says which release that set is from.

To take a change, see it, then copy what you want by hand:

```sh
ascribe templates status --diff how-to
```

`--exit-code` makes it exit with 1 when the set has changed a template you took, or no longer has one, for a CI job that wants to know. The [command reference](../reference/cli/sources.md#cli-templates-status) has the rest, and its JSON.

## In a project that has pages

A project that takes the set late is one with pages written already. Ascribe's own docs took it that way, with 46 pages, and it showed what to expect:

- **A type you declare already is refused,** and nothing is written. To take its template, take the type out of `ascribe.toml`, take the template, then put back what your type had. Until you do, its pages fall to your default type, and `ascribe check` doesn't say so.
- **The new types match no page** until you change their `files`. Eject puts each type's pages in a folder of its name, and yours are somewhere else; a type that matches nothing isn't reported either.
- **Each type's frontmatter is the set's,** a plain `title` and `description`. Put back what your own types declare, such as a title drawn as code (`inline = "code"`), on each one.
- **Pages written before their template have nothing of it left,** so the check reports none of its placeholders. A phrase with a placeholder's name, such as `{product}`, which the tutorial's template has, isn't reported, since the template's brace is escaped and the phrase's isn't.
- **Not every page fits.** A home page that lists the other pages is none of the six: give it a type of your own, with no template. The troubleshooting template's "Cause" and "Solution" headings repeat under every symptom, and the check warns about headings with the same name on a page, so the docs write them as bold paragraphs instead.

The docs didn't change a page that already did its job to match its template heading for heading: a guide says what a kind of page is for, and the template is one way to get there.

## Ascribe's notes

Their guide says what belongs in a page of each kind, and it's written for any tool. Ascribe's notes, `<type>.ascribe.md`, are a minute's reading after it: only what's particular to writing that kind of page in Ascribe, such as a how-to's alternatives by platform as a [variant](../reference/directives.md#variant) group, and a step's command taken from tested code with [`@snippet`](../reference/directives.md#snippet). They're the only words in the set that are Ascribe's, and `templates.lock` records them apart from their files.

## Start a page

In VS Code, run **Ascribe: New Page** from the Command Palette, from the **Pages** view's title bar, or on a folder in the Explorer. It asks for the type, each with what it's for, and the page's title, then makes the page from the type's template with the title set, in the first folder the type's `files` cover, or in the folder you ran it on. The page opens with the first thing to fill in selected, usually the description, so typing replaces it, and **Ascribe: Go to Next Placeholder** selects the next. It never writes over a file. A type with no template gives a page with its frontmatter, so it works in any project ([editor guide](editor.md#start-a-page)).

Without the editor:

@steps
1. Copy the template into the type's folder:

   ```sh
   mkdir -p docs/how-to
   cp templates/how-to.md docs/how-to/rotate-keys.md
   ```

2. Read the type's guide and Ascribe's notes on it, before you write.
3. Replace each `\{placeholder}` with what it asks for, and the title and description in the frontmatter with your own.

An agent starts from the template too. Ask it for a page, with the plugin's `/ascribe:new-page` or the prompt `ascribe agents prompt new-page` prints ([agents guide](agents.md#from-the-command-line)), and the prompt gives it the template with the title set, says where the file goes, and sends it to the guide and the notes, by their paths, before it writes:

```sh
ascribe agents prompt new-page --arg type=how-to --arg title="Rotate keys"
```

There's no command that writes the page for you at the terminal: copying the file is one line, the editor's **New Page** is for a person, and the prompt is for the agent.

## What's left to fill in

`ascribe check` compares a page with its type's template, and reports what it still has of it as advice (`template-placeholder-left`):

- **A placeholder,** the template's text from a `\{` to its closing brace, that the page still has word for word. Its brace has to be escaped on the page too, as it is in the template: a phrase with a placeholder's name, such as `{product}`, is the page's own, and so is a placeholder quoted in inline code or a code block. One that runs across blocks, as the how-to's "See also" opens a brace in a paragraph and closes it at the end of a list, is reported once, where it starts, when the page has all of it.
- **A frontmatter field** with the value the template gives it, such as `title: "Title"`.
- **An instruction,** a paragraph the template has in a block quote, that the page still has whole. A block quote of your own isn't one.

How a line is wrapped doesn't matter, so an editor that rewraps a paragraph doesn't hide a placeholder in it. A page of a type with no template isn't looked at.

The check finds only what was never touched. A placeholder with one word changed, or with the space before its closing brace taken out, as the how-to's `\{Task name }` has, isn't the template's any more, and isn't reported; nor is one you've begun filling in. Nor does it look at a page's shape: a how-to with its steps taken out, and none written, has nothing left of the template to report. Read a page through before publishing it.

To turn it off, or make it a warning, set its level in [`[checks]`](../reference/content-model/checks.md#19-checks):

```toml
[checks]
template-placeholder-left = "warning"
```

## See also

- [`ascribe templates`](../reference/cli/sources.md#cli-templates-eject): `list` and `eject`, and their options.
- [Page types](../reference/content-model.md#41-page-types): a type's `template`, `guide`, and `purpose`.
- [ASC195 `template-placeholder-left`](../reference/diagnostics/content-checks.md#ASC195): what the check reports.
