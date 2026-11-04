---
type: reference
title: Command reference
description: "The `ascribe` commands: their options, outputs, and exit codes."
permalink: cli-reference
---

The `ascribe` command checks, builds, and formats an Ascribe project, and runs the language server the editor uses. Its `explain`, `model`, `outline`, `link`, `refs`, `resolve`, and `search` commands answer questions about a project, for writers and for agents, without changing anything, and `ascribe mcp` offers them to agents as the tools of an MCP server.

Install it in a project with `npm install --save-dev @ascribed/cli` (see [Install](../install.md#npm)) and run it with `npx ascribe`, or from a script in `package.json`. Where there's no Node, install it on your path with Homebrew or the install script ([Install](../install.md#without-node)) and run it as `ascribe`.

## Commands

Each command links to its section, with its options, output, and exit codes, on the page for its kind of work.

| Command | What it does |
|---|---|
| [`ascribe addresses`](../publishing/redirects.md#cli-addresses) | Keeps `ascribe.addresses`, the record of where the addresses you published went: makes it, records a move or a retired address, and lists the redirects each build writes. |
| [`ascribe agents`](../agents/commands.md#cli-agents) | Sets a repository up for agents: writes the project's rules and the skill where agents read them, indexes the docs for a product's agents, and runs an agent's hook. |
| [`ascribe baseline`](../checks/baseline.md#cli-baseline) | Puts a project's existing advice off for now, so `ascribe check` reports only what's new. |
| [`ascribe build`](../checks/check.md#cli-build) | Checks the project, then writes each build's outputs. |
| [`ascribe check`](../checks/check.md#cli-check) | Checks every source file for problems, without building anything. |
| [`ascribe diff`](../review/diff.md#cli-diff) | Shows what changed between a git revision and the working tree, as readers will see it. |
| [`ascribe drift`](../review/diff.md#cli-drift) | Lists the pages whose code examples changed, and whether the words around them changed too. |
| [`ascribe explain`](questions.md#cli-explain) | Says what a diagnostic means, and how to fix it. |
| [`ascribe fmt`](../checks/check.md#cli-fmt) | Rewrites Ascribe constructs into canonical form. |
| [`ascribe init`](../install.md#cli-init) | Starts a project: writes its first `ascribe.toml`, and makes the folder its pages go in. |
| [`ascribe link`](references.md#cli-link) | Says whether a link works, written on a given page, and the destination to write there. |
| [`ascribe lsp`](../editor/language-server.md#cli-lsp) | Runs the language server, for an editor. |
| [`ascribe mcp`](../agents/commands.md#cli-mcp) | Runs an MCP server, for an agent that has no shell, or a host that approves tools one by one. |
| [`ascribe model`](questions.md#cli-model) | Shows the content model as `ascribe` reads it, with the defaults filled in. |
| [`ascribe new`](../templates/index.md#cli-new) | Starts a page from its type's template, with its type and title set. |
| [`ascribe observe`](../examples/sources.md#cli-observe) | Runs the test of a folder of tested examples, and records what it passed with. |
| [`ascribe outline`](questions.md#cli-outline) | Shows a page's title and type, and the headings a link to it can name. |
| [`ascribe permalinks`](../permalinks/command.md#cli-permalinks) | Keeps the list of permalinks, checks a product's names against it, and retires a permalink. |
| [`ascribe preview`](../checks/check.md#cli-preview) | Renders a build's pages as a site, serves it on this computer, and renders it again as you edit. |
| [`ascribe refs`](references.md#cli-refs) | Lists the places that use a page, a fragment, a heading, an image, or an entry of the content model. |
| [`ascribe rename`](references.md#cli-rename) | Renames a phrase, a dimension's value, a heading's id, or a page, and changes everything that uses it. |
| [`ascribe report`](../checks/report-command.md#cli-report) | Reports what state a project is in, section by section. |
| [`ascribe resolve`](questions.md#cli-resolve) | Resolves a page for one build and writes it as a reader of that build sees it. [A page in the site output](site-output.md) says what `--emit site` writes. |
| [`ascribe search`](questions.md#cli-search) | Finds where the pages say something, as their readers read it, and names the line to edit. |
| [`ascribe sources`](../examples/sources.md#cli-sources) | Copies code from other repositories, pins it in `ascribe.lock`, and says what changed when the pin moves. |
| [`ascribe templates`](../templates/command.md#cli-templates) | Writes the page templates into a project, and says what's changed in them. |
| [`ascribe vale`](../checks/vale.md#cli-vale) | Writes Vale's preset out as the project's own configuration. |
| [`ascribe versions`](../versions/release.md#cli-versions) | Lists a project's release lines, says what differs between two, and releases, archives, or retires one. |

### Every command's synopsis

@include: ../_generated/cli-synopsis.md

### Output formats

`--format` takes the same words on every command that has it. Every command that takes `text` takes `concise` too: `ascribe check` lists one line a problem, and a command with no one-line form writes its text.

### Commands that write

Every command that changes a file you commit, such as `ascribe fmt`, `ascribe check --fix`, `ascribe rename`, `ascribe init`, `ascribe sources update`, `ascribe templates eject`, `ascribe vale eject`, `ascribe agents sync`, or `ascribe observe`, takes `--dry-run`, which says what it would change and changes nothing. It also refuses to change a file git couldn't put back as it was: one with changes not committed, one git doesn't track, or one in no repository. The error names the files; commit or stash them, or pass `--force` to write them anyway. A file that isn't there yet is only added, so it's never refused.

```console
$ ascribe addresses move guides/old.md guides/new.md
error: this would change files git can't put back as they are: ascribe.addresses; commit or stash them, or pass --force
```

One that takes `--check`, such as `ascribe fmt`, `ascribe permalinks sync`, `ascribe agents sync`, or `ascribe observe`, changes nothing with it either, and is for CI: it prints each change as a unified diff, and exits with `1` when something would change.

## Options every command accepts

@include: ../_generated/cli-global-options.md

Without `--config`, a command's project is the nearest `ascribe.toml` at or above the current folder, or at or above the path it's given.

A path a command takes is from the current folder, as a shell's is: `ascribe check`, `ascribe fmt`, and `ascribe search` take paths that way alone, and `ascribe new` a path from the current folder inside the content root. The commands that take one page, `outline`, `link --from`, `refs`, `rename`, `resolve`, `diff`, `addresses move`, and `addresses retire`, also take its path from the content root, as links write it.

When there's none, and the folder holds projects in its subfolders, as a repository with several does at its root, the error names their folders, so you can run the command in one, or pass `--config` with its folder: `the projects below it are in docs, handbook: run ascribe in one of them, or pass --config with its folder`. `ascribe agents sync` is the exception: there, it syncs each of them.

## Exit codes

Every command exits with the same three codes, and each command's section has a table that says what they mean for it:

| Code | Meaning |
|---|---|
| `0` | It did what it was asked, whatever it found, unless finding something fails it. |
| `1` | It ran, and found something, such as errors, with `ascribe check` and `ascribe build`; a file that would change, with `--check`; a target that doesn't exist, with `ascribe link` and `ascribe refs`; nothing matched, with `ascribe search`; or a test that failed, with `ascribe observe`. `--exit-code` means two things. On `ascribe report`, `ascribe diff`, `ascribe drift`, and `ascribe templates status`, it turns `1` on for what they find. On `ascribe check`, it sets the level that fails, `error` unless you say `warning`. |
| `2` | It couldn't run: a usage error, no `ascribe.toml`, a content model with errors, a file it can't read or write, or, without `--force`, a file git couldn't put back. The reason goes to standard error. |

A command with nothing to find, such as `ascribe init` or `ascribe new`, has no `1`: it did its work, or it couldn't. A content model with errors is a `2` for every command, `ascribe check` and `ascribe model` included: the errors are shown, and nothing else is done.

A few don't follow the pattern:

- [`ascribe agents hook`](../agents/commands.md#cli-agents-hook) exits with `1` when its input isn't JSON, a failure an agent shows to you and not to the model, and with `0` whatever its checks find.
- [`ascribe lsp`](../editor/language-server.md#cli-lsp) exits with `1` when the editor stops it without asking it to shut down first, as the Language Server Protocol says, and with `0` when it shuts down.
- [`ascribe preview`](../checks/check.md#cli-preview) runs until you stop it, so it has no `0` or `1`: it exits with `2` only when it couldn't start.

## Environment variables

The `ascribe` binary, and the tools that run it, read these. Each is for a test, a CI job, or a machine set up differently from most, and none is needed: unset, each has the default its row gives.

| Variable | Read by | What it does | Default |
|---|---|---|---|
| `ASCRIBE_BIN` | [`@ascribed/astro`](../astro/reference.md#options) | The `ascribe` binary the integration runs, when its `binary` option doesn't name one. | The binary `@ascribed/cli` installed |
| `ASCRIBE_CACHE_DIR` | [`ascribe sources`](../examples/sources.md#cli-sources), and [`ascribe agents hook`](../agents/commands.md#cli-agents-hook)'s check server | The folder Ascribe keeps fetched repositories and the check server's files in. | `ascribe` in your user's cache folder: `$XDG_CACHE_HOME`, `~/.cache`, `~/Library/Caches`, or `%LOCALAPPDATA%` |
| `ASCRIBE_HOOK_SERVER` | [`ascribe agents hook`](../agents/commands.md#cli-agents-hook) | `off` checks in the hook's own process every time, instead of starting a check server that keeps the project loaded. | A check server |
| `ASCRIBE_LINKS_RETRY_WAIT` | [`ascribe report links`](../checks/external-links.md#a-servers-error) | How many seconds to wait before asking again about a link that answered with a server error, such as `0` in a test. | `30` |
| `ASCRIBE_PATH` | [The Claude Code plugin](../agents/claude-code.md#the-plugin) | The `ascribe` binary the plugin runs. | The project's `node_modules/.bin/ascribe`, then `ascribe` on your path |
| `ASCRIBE_TODAY` | Every command that checks, the [language server](../editor/language-server.md#cli-lsp), and the hook's check server | The day, written `YYYY-MM-DD`, the checks that depend on the date [check as of](../checks/check.md#checking-as-of-another-day). A value that isn't a date is reported, and today's is used. | Today's date in UTC |
| `NO_COLOR` | Every command, with `--color auto` | Set to anything, it turns color off at a terminal too. | Color when writing to a terminal |

`ASCRIBE_BLESS`, which rewrites generated files, is for the tests of Ascribe's own repository, and no command reads it.
