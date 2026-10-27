---
type: how-to
title: Prose, through Vale
description: "Checking your pages' prose with Vale, from Ascribe's quiet preset or your own Vale configuration, in the editor and in `ascribe check`."
---

[Vale](https://vale.sh) checks prose: a word written twice, a misspelling, a term spelled two ways, and whatever else its rules look for. Ascribe runs it for you and reports what it finds as Ascribe diagnostics, at the right place in your source, beside the rest of your problems in the editor and in `ascribe check`.

Ascribe gives Vale your prose and nothing else. Frontmatter, directive lines, attribute blocks, and code never reach it, so Vale has nothing to say about Ascribe's syntax, and a phrase's text is checked where you wrote `{key}`. You install Vale; Ascribe never downloads it or anything for it.

## Before you start

Install Vale 3.16 or later: see [Vale's installation page](https://docs.vale.sh/topics/installation). Check that `vale --version` works in the shell you run `ascribe` from.

## Start from the quiet preset

@steps
1. Turn it on in `ascribe.toml`:

   ```toml
   [checks.vale]
   preset = "quiet"
   ```

2. Add `.ascribe/` to your `.gitignore`, if it isn't there. Ascribe writes the preset there, beside the build's output, and writes it again whenever it's needed ([Files Ascribe writes](../command/project-files.md)).

That's all: no `.vale.ini` to write, and no `vale sync`. Open a page in VS Code, or save one, and Vale's alerts appear in the Problems panel. In a terminal, `ascribe check --vale` checks every page.

The `quiet` preset has only rules that are almost never wrong. Each was run on Ascribe's own docs and on three large documentation sets, and kept only if it was wrong at most once in a hundred alerts:

| Rule | What it finds |
|---|---|
| `Ascribe.Repeated` | A word written twice in a row, such as `the the`. The editor offers to remove the second. |
| `Ascribe.Typos` | A common misspelling, such as `recieve` or `seperate`. The editor offers the correction. |

Its alerts are advice: shown, and never failing `ascribe check`. Text in bold isn't checked, since in documentation it's usually a label from a user interface, which you don't choose.

A rule that's wrong for your project goes in `off`, by the name its alerts give it:

```toml
[checks.vale]
preset = "quiet"
off = ["Ascribe.Typos"]
```

To quiet a rule in one place, use Vale's own comments, which Ascribe passes through to it:

```markdown
<!-- vale Ascribe.Repeated = NO -->
That that is, is.
<!-- vale Ascribe.Repeated = YES -->
```

A rule added to the preset in a later release makes new advice appear when you upgrade. The changelog says so each time, under **Behavior change**.

## Bring your own Vale configuration

A project that already uses Vale points `config` at its `.vale.ini`, from the project root, instead of naming a preset:

```toml
[checks.vale]
config = ".vale.ini"
```

Ascribe runs Vale with that configuration and its styles, as `vale` would. Two things change:

- **Remove the patterns that hid Ascribe's syntax.** A configuration written for Markdown with directives often has `BlockIgnores` or `TokenIgnores` for lines starting with `@`, for `{key}`, or for attribute blocks. Vale no longer sees any of those, so the patterns only hide prose that happens to match them.
- **Use a `[*.md]` section for your pages.** Vale reads each page's prose as a Markdown file, whatever the page's own name.

Vale's comments, your vocabularies, and the rest of your configuration work as before.

## The project's own words

Your project already lists words Vale shouldn't take for misspellings: phrase values, glossary terms and their aliases, and the labels of dimensions, their values, and features. Ascribe writes them to a Vale vocabulary named `Ascribe`, under `.ascribe/vale/`, and turns it on for the preset and for your own configuration alike. Add a glossary term, and Vale accepts it the next time it runs. Ascribe never writes into a vocabulary of your own.

## Where Vale runs

- **In the editor**, on a page when you open it and when you save it, never as you type. A slow Vale never delays anything else, and an edit removes the alerts at and after the place you changed until the next save.
- **In `ascribe check`**, with `--vale`, or always with `in-check = true`. Vale runs once, on the pages the check reports on. Its `suggestion`, `warning`, and `error` alerts are advice, warnings, and errors; set `max-level = "advice"` to keep your own configuration's warnings from failing `--deny-warnings`.
- **In the agents' hooks**, only with `in-check = true`, and only Vale's error-level alerts reach the agent. See [Agents](../agents/hooks.md).
- **Never in the MCP server's tools**, which say they only read: `ascribe_check` reports that the prose wasn't checked.

When Vale isn't installed, fails, or takes too long where the project asked for its prose checked everywhere, with `--vale` or `in-check = true`, you get one advice on `[checks.vale]` in `ascribe.toml` (`prose-not-checked`) saying why, and everything else is checked as usual. Without `in-check`, the editor checks the prose wherever Vale runs and says nothing where it doesn't, so a machine without Vale isn't told so. If you set `in-check` and check prose only in CI, turn that advice off on other machines with `prose-not-checked = "off"` in `[checks]`.

To run a Vale that isn't on your `PATH`, set `command` to its path from the project root:

```toml
[checks.vale]
preset = "quiet"
command = "tools/vale"
```

A path is a program in the repository, so the editor asks you before it runs it, once a session, and the agents' hooks don't run it; with `in-check = true`, each says so in `prose-not-checked`. To have Vale run everywhere it can, install it on your `PATH` and leave `command` out.

## From the preset to a configuration of your own

When turning rules off isn't enough, say to add a style package, run [`ascribe vale eject`](#cli-vale-eject) in the project. It writes the preset out as your own configuration, and from then on the files are yours to change: Ascribe never writes them again. The project's vocabulary is still Ascribe's to write, under `.ascribe/vale/`.

## `ascribe vale`
@id {permalink=true}: cli-vale

@summary
Sets up Vale for a project. Vale itself is run by `ascribe check --vale` and the editor.

### `ascribe vale eject`
@id {permalink=true}: cli-vale-eject
@summary: Writes the preset `[checks.vale]` names out as the project's own Vale configuration, `.vale.ini` and a `.vale/` styles folder, and points `[checks.vale]` at it.

Writes the preset `[checks.vale]` names out as the project's own Vale configuration: `.vale.ini` and a `.vale/` styles folder at the project root, with the rules `off` names turned off in `.vale.ini`. Then it changes `[checks.vale]` in `ascribe.toml` to `config = ".vale.ini"`, in place of `preset` and `off`, keeping its other keys and comments.

It takes no options but the ones every command accepts ([command reference](../command/index.md#options-every-command-accepts)). It lists each file it wrote, `ascribe.toml` last.

| Code | Meaning |
|---|---|
| `0` | The preset is the project's own now |
| `2` | It wrote nothing: no `ascribe.toml`, a content model with errors, a `[checks.vale]` that names no preset, or a `.vale.ini` or `.vale` that's there already. Or a file couldn't be written; the files it wrote before that stay. |

## See also

- [`[checks.vale]`](../content-model/checks.md#checksvale): every key.
- [Prose, through Vale](diagnostics/output-checks.md#prose-through-vale): the two diagnostics.
